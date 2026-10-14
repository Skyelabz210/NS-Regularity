//! # CRAM Extended Division — 8-Lane Residue Arithmetic (6 NTT + 2 Small)
//!
//! ## What changed from cram_div.rs
//!
//! `cram_div.rs` uses 4 NTT primes (769,929,1153,1217) + 2 small-prime lanes.
//! corridor_probe_v3 showed the 4-prime BS raw corridor (M_NTT/2 ≈ 5.01×10^11)
//! is consumed to >99.99% by ABC at amp=100 after 5 steps. The corridor is saturated.
//!
//! This module extends to 6 NTT primes (adding 1409 and 1601):
//!
//! ```text
//!   M_NTT_6 = 769·929·1153·1217·1409·1601 = 2,261,331,666,182,114,209
//!   M_NTT_6/2 (BS limit) = 1,130,665,833,091,057,104  ≈ 1.13×10^18
//! ```
//!
//! Factor of improvement: 2.25 million× (observed max|raw| = 5.01×10^11 fills
//! only 0.000044% of the new corridor).
//!
//! ## Extended basis layout
//!
//! ```text
//!   res[0..6]  — NTT primes [769, 929, 1153, 1217, 1409, 1601]
//!   res[6]     — 2^31 lane
//!   res[7]     — 5^9 lane
//! ```
//!
//! Division contracts are UNCHANGED (d still divides M_small = 2^31·5^9).
//! q_bound_min becomes M_NTT_6 × partial_mod — orders of magnitude larger.
//!
//! ## D-012 compliance
//! All Garner loops are ASCENDING. Hardcoded — descending order is silent corruption.

use crate::ns3d::modinv;
use crate::ns3d_n32_ntt6::{NTT_PRIMES_6, G_C_6, M_NTT_6};
use crate::cram_div::{MOD_2POW, MOD_5POW, M_SMALL, gcd_u64};

// ============================================================================
// 8-lane extended basis
// ============================================================================

pub const NTT_LANES: usize = 6;
pub const SMALL_LANES: usize = 2;
pub const EXT_LANES: usize = NTT_LANES + SMALL_LANES; // 8

/// All 8 basis moduli: 6 NTT primes then 2^31 and 5^9.
pub const EXT_BASIS_8: [u64; EXT_LANES] = [
    769, 929, 1153, 1217, 1409, 1601, // NTT primes (indices 0..6)
    MOD_2POW, MOD_5POW,               // small lanes (indices 6..8)
];

/// 8-lane extended residue. `res[0..6]` mod NTT_PRIMES_6; `res[6]` mod 2^31;
/// `res[7]` mod 5^9.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtRes8 {
    pub res: [u64; EXT_LANES],
}

impl ExtRes8 {
    pub fn from_i128(x: i128) -> Self {
        let mut r = [0u64; EXT_LANES];
        for i in 0..EXT_LANES {
            r[i] = (x.rem_euclid(EXT_BASIS_8[i] as i128)) as u64;
        }
        ExtRes8 { res: r }
    }

    /// Reconstruct `x mod M_small` from the two small-prime lanes.
    /// Same formula as ExtRes::small_part — uses lanes [6] (2^31) and [7] (5^9).
    pub fn small_part(&self) -> i128 {
        let a = self.res[6] as i128;
        let b = self.res[7] as i128;
        let inv_2pow_mod_5pow = modinv(MOD_2POW % MOD_5POW, MOD_5POW) as i128;
        let m5 = MOD_5POW as i128;
        let k = ((b - a).rem_euclid(m5) * inv_2pow_mod_5pow).rem_euclid(m5);
        a + (MOD_2POW as i128) * k
    }

    /// The 6 NTT residues as a fixed-size array.
    #[inline]
    pub fn ntt_res(&self) -> [u64; 6] {
        [self.res[0], self.res[1], self.res[2], self.res[3], self.res[4], self.res[5]]
    }
}

// ============================================================================
// Unsigned 6-prime Garner — returns x in [0, M_NTT_6)
// D-012: ascending j-loop. DO NOT reorder.
// ============================================================================

/// Garner reconstruction over 6 NTT primes, returning x in [0, M_NTT_6).
/// Used internally by division and sign-detection.
pub(crate) fn garner6_to_u128(r: &[u64; 6]) -> u128 {
    let p = NTT_PRIMES_6;
    let g = G_C_6;

    // a0..a5 are the mixed-radix digits (ascending j — D-012 compliant)
    let a0 = r[0] as u128;
    let a1 = ((r[1] as i128 - a0 as i128).rem_euclid(p[1] as i128) as u64
              * g[1] % p[1]) as u128;
    let p01 = p[0] as u128 * p[1] as u128;
    let t2 = (a0 + p[0] as u128 * a1) % p[2] as u128;
    let a2 = ((r[2] as i128 - t2 as i128).rem_euclid(p[2] as i128) as u64
              * g[2] % p[2]) as u128;
    let p012 = p01 * p[2] as u128;
    let t3 = (a0 + p[0] as u128 * a1 + p01 * a2) % p[3] as u128;
    let a3 = ((r[3] as i128 - t3 as i128).rem_euclid(p[3] as i128) as u64
              * g[3] % p[3]) as u128;
    let p0123 = p012 * p[3] as u128;
    let t4 = (a0 + p[0] as u128 * a1 + p01 * a2 + p012 * a3) % p[4] as u128;
    let a4 = ((r[4] as i128 - t4 as i128).rem_euclid(p[4] as i128) as u64
              * g[4] % p[4]) as u128;
    let p01234 = p0123 * p[4] as u128;
    let t5 = (a0 + p[0] as u128 * a1 + p01 * a2 + p012 * a3 + p0123 * a4) % p[5] as u128;
    let a5 = ((r[5] as i128 - t5 as i128).rem_euclid(p[5] as i128) as u64
              * g[5] % p[5]) as u128;

    // x = sum, accumulated mod M_NTT_6 to stay in u128
    let m6 = M_NTT_6;
    let mut x = a0 % m6;
    x = (x + p[0] as u128 % m6 * (a1 % m6)) % m6;
    x = (x + p01 % m6 * (a2 % m6)) % m6;
    x = (x + p012 % m6 * (a3 % m6)) % m6;
    x = (x + p0123 % m6 * (a4 % m6)) % m6;
    x = (x + p01234 % m6 * (a5 % m6)) % m6;
    x
}

// ============================================================================
// Full 8-lane sign reconstruction
// ============================================================================

/// Reconstruct the signed i128 value from all 8 lanes.
/// Valid for |x| < M_ext_8/2 ≈ 4.7×10^33.
pub fn ext_to_i128_8(er: &ExtRes8) -> i128 {
    let m_ntt = M_NTT_6;
    let m_small = MOD_2POW as u128 * MOD_5POW as u128;
    let q_ntt = garner6_to_u128(&er.ntt_res());
    let q_small = er.small_part();
    let m_ntt_inv = modinv((m_ntt % m_small) as u64, m_small as u64) as u128;
    let s = ((q_small as i128 - q_ntt as i128).rem_euclid(m_small as i128) as u128
            * m_ntt_inv % m_small);
    let val = q_ntt + m_ntt * s;
    let m_ext = m_ntt * m_small;
    if val < m_ext / 2 { val as i128 } else { val as i128 - m_ext as i128 }
}

// ============================================================================
// NODE-P5B-1: 8-lane exact positive division
// ============================================================================

/// Exact integer division `⌊x/d⌋` in the 8-lane extended basis.
///
/// Precondition: d | M_small = 2^31 · 5^9.
///
/// Per-divisor q_bound_min with 6 NTT primes (2.25M× improvement over 4-prime):
/// ```text
///   d=N^6=2^30:  q_bound = M_NTT_6 * 2     ≈ 4.52×10^18
///   d=S^2=10^8:  q_bound = M_NTT_6 * 5     ≈ 1.13×10^19
///   d=4S=40000:  q_bound = M_NTT_6 * 3125  ≈ 7.07×10^21
///   d=2S=20000:  q_bound = M_NTT_6 * 3125  ≈ 7.07×10^21
///   d=N^2=1024:  q_bound = M_NTT_6 * 1953125 ≈ 4.41×10^24
/// ```
pub fn cram_div_exact_pos_8(x: &ExtRes8, d: u64) -> ExtRes8 {
    debug_assert!(d > 0);
    debug_assert!(M_SMALL % d as u128 == 0,
        "cram_div_exact_pos_8: d={} does not divide M_small", d);

    let small = x.small_part();
    let r = small.rem_euclid(d as i128);

    // NTT lanes: q mod p_i = (r_i - r) · d^{-1} mod p_i  for all 6 primes
    let mut q_res = [0u64; EXT_LANES];
    let r_mod = |p: u64| (r.rem_euclid(p as i128)) as u64;
    for i in 0..6 {
        let p = EXT_BASIS_8[i];
        let diff = (x.res[i] + p - r_mod(p) % p) % p;
        let d_inv = modinv(d % p, p);
        q_res[i] = ((diff as u128 * d_inv as u128) % p as u128) as u64;
    }

    // Small lanes via K-Elim lift (same as cram_div but with 6-prime Garner)
    let q_2pow = small_lane_div(x.res[6], r as i64, d, MOD_2POW);
    let q_5pow = small_lane_div(x.res[7], r as i64, d, MOD_5POW);
    q_res[6] = lift_to_lane_8(q_2pow.0, q_2pow.1, MOD_2POW, &q_res[0..6]);
    q_res[7] = lift_to_lane_8(q_5pow.0, q_5pow.1, MOD_5POW, &q_res[0..6]);

    ExtRes8 { res: q_res }
}

/// Returns `(q_partial, partial_mod)`: q ≡ q_partial (mod partial_mod),
/// where partial_mod = lane_mod / gcd(d, lane_mod).
fn small_lane_div(x_lane: u64, r: i64, d: u64, lane_mod: u64) -> (u64, u64) {
    let xr = (x_lane as i128 - r as i128).rem_euclid(lane_mod as i128) as u64;
    let g = gcd_u64(d, lane_mod);
    let lane_g = lane_mod / g;
    let d_g = d / g;
    let xr_over_g = xr / g;
    let d_g_inv = modinv(d_g % lane_g, lane_g);
    let q_partial = ((xr_over_g as u128 * d_g_inv as u128) % lane_g as u128) as u64;
    (q_partial, lane_g)
}

/// Lift `(q_partial, partial_mod)` to `q mod lane_mod` using the 6-prime
/// NTT Garner. Valid when `q < M_NTT_6 · partial_mod`.
fn lift_to_lane_8(q_partial: u64, partial_mod: u64, lane_mod: u64,
                   q_ntt: &[u64]) -> u64 {
    debug_assert_eq!(q_ntt.len(), 6, "lift_to_lane_8: need 6 NTT residues");
    let ntt6: [u64; 6] = q_ntt.try_into().expect("6 residues");
    let q_ntt_full = garner6_to_u128(&ntt6);

    if partial_mod == lane_mod {
        return q_partial;
    }
    let m_ntt = M_NTT_6;
    let m_ntt_mod_p = (m_ntt % partial_mod as u128) as u64;
    let m_ntt_inv = modinv(m_ntt_mod_p, partial_mod);
    let q_ntt_mod_p = (q_ntt_full % partial_mod as u128) as u64;
    let s = (((q_partial + partial_mod - q_ntt_mod_p % partial_mod) % partial_mod) as u128
            * m_ntt_inv as u128 % partial_mod as u128) as u64;
    ((q_ntt_full + m_ntt * s as u128) % lane_mod as u128) as u64
}

// ============================================================================
// NODE-P5B-2: DivCtx8 + trunc_div_signed_8 + 8-lane arithmetic
// ============================================================================

/// Pre-computed division context for 8-lane arithmetic.
pub struct DivCtx8 {
    pub d: u64,
    /// d^{-1} mod p_i for all 6 NTT primes.
    pub inv_d_ntt: [u64; 6],
    /// Tighter partial_mod: min(2^31/gcd(d,2^31), 5^9/gcd(d,5^9)).
    pub partial_mod_min: u128,
    /// q_bound = M_NTT_6 × partial_mod_min. Safe as long as |q| < q_bound.
    pub q_bound: u128,
}

impl DivCtx8 {
    pub fn build(d: u64) -> Self {
        assert!(M_SMALL % d as u128 == 0,
            "DivCtx8::build: d={} must divide M_small", d);
        let mut inv_d_ntt = [0u64; 6];
        for i in 0..6 {
            let p = EXT_BASIS_8[i];
            inv_d_ntt[i] = modinv(d % p, p);
        }
        let g2 = gcd_u64(d, MOD_2POW) as u128;
        let g5 = gcd_u64(d, MOD_5POW) as u128;
        let partial_mod_min = (MOD_2POW as u128 / g2).min(MOD_5POW as u128 / g5);
        let q_bound = M_NTT_6 * partial_mod_min;
        DivCtx8 { d, inv_d_ntt, partial_mod_min, q_bound }
    }
}

/// Truncating signed division `⌊x/d⌋` in the 8-lane basis.
/// Bit-identical to canonical Rust `/` for |q| < ctx.q_bound.
pub fn trunc_div_signed_8(x: &ExtRes8, ctx: &DivCtx8) -> ExtRes8 {
    // Full 8-lane sign reconstruction — required because |x| may exceed M_NTT_6/2.
    let is_neg = {
        let m_ntt = M_NTT_6;
        let m_small = MOD_2POW as u128 * MOD_5POW as u128;
        let q_ntt = garner6_to_u128(&x.ntt_res());
        let q_small = x.small_part();
        let m_ntt_inv = modinv((m_ntt % m_small) as u64, m_small as u64) as u128;
        let s = ((q_small as i128 - q_ntt as i128).rem_euclid(m_small as i128) as u128
                * m_ntt_inv % m_small);
        let val = q_ntt + m_ntt * s;
        val >= m_ntt * m_small / 2
    };

    if is_neg {
        let neg_x = ext8_neg(x);
        let q_pos = cram_div_exact_pos_8(&neg_x, ctx.d);
        ext8_neg(&q_pos)
    } else {
        cram_div_exact_pos_8(x, ctx.d)
    }
}

/// Signed exact division asserting zero remainder.
/// Panics if d does not divide x exactly.
pub fn exact_div_8_zero_rem(x: &ExtRes8, ctx: &DivCtx8) -> ExtRes8 {
    let small = x.small_part();
    let r = small.rem_euclid(ctx.d as i128);
    assert_eq!(r, 0,
        "exact_div_8_zero_rem: d={} does not divide x (rem={})", ctx.d, r);
    trunc_div_signed_8(x, ctx)
}

// ============================================================================
// 8-lane arithmetic helpers
// ============================================================================

#[inline]
pub fn ext8_add(a: &ExtRes8, b: &ExtRes8) -> ExtRes8 {
    let mut r = [0u64; EXT_LANES];
    for i in 0..EXT_LANES {
        r[i] = ((a.res[i] as u128 + b.res[i] as u128) % EXT_BASIS_8[i] as u128) as u64;
    }
    ExtRes8 { res: r }
}

#[inline]
pub fn ext8_sub(a: &ExtRes8, b: &ExtRes8) -> ExtRes8 {
    let mut r = [0u64; EXT_LANES];
    for i in 0..EXT_LANES {
        let m = EXT_BASIS_8[i];
        r[i] = ((a.res[i] + m - b.res[i] % m) % m) as u64;
    }
    ExtRes8 { res: r }
}

#[inline]
pub fn ext8_mul(a: &ExtRes8, b: &ExtRes8) -> ExtRes8 {
    let mut r = [0u64; EXT_LANES];
    for i in 0..EXT_LANES {
        r[i] = ((a.res[i] as u128 * b.res[i] as u128) % EXT_BASIS_8[i] as u128) as u64;
    }
    ExtRes8 { res: r }
}

#[inline]
pub fn ext8_neg(a: &ExtRes8) -> ExtRes8 {
    let mut r = [0u64; EXT_LANES];
    for i in 0..EXT_LANES {
        let m = EXT_BASIS_8[i];
        r[i] = (m - a.res[i] % m) % m;
    }
    ExtRes8 { res: r }
}

#[inline]
pub fn ext8_smul(a: &ExtRes8, k: i128) -> ExtRes8 {
    let mut r = [0u64; EXT_LANES];
    for i in 0..EXT_LANES {
        let m = EXT_BASIS_8[i];
        let k_mod = k.rem_euclid(m as i128) as u64;
        r[i] = ((a.res[i] as u128 * k_mod as u128) % m as u128) as u64;
    }
    ExtRes8 { res: r }
}

#[inline]
pub fn ext8_from_i64(v: i64) -> ExtRes8 {
    ExtRes8::from_i128(v as i128)
}

// ============================================================================
// INSTRUMENTED DIVISION — P5B-8H.1
// ============================================================================

/// Instrumented wrapper: records (true_dividend, true_quotient) into witness.
/// The dividend is ext_to_i128_8(x) — the pre-division integer, never q × d.
/// All five NS division sites must call this, not bare trunc_div_signed_8.
pub fn trunc_div_signed_8_witnessed(
    x: &ExtRes8,
    ctx: &DivCtx8,
    witness: &mut crate::assault_witness::DivWitness,
) -> ExtRes8 {
    let dividend = ext_to_i128_8(x);
    let q = trunc_div_signed_8(x, ctx);
    let quotient = ext_to_i128_8(&q);
    witness.record(dividend, quotient);
    q
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cram_div::ExtRes;

    /// small_part gives the same result as ExtRes for the same value.
    #[test]
    fn small_part_8_matches_4lane() {
        let vals: &[i128] = &[0, 1, 1023, 1024, 100_000, 1_000_000_000_000,
                               4_000_000_000_000_000, -1, -1023, -100_000];
        for &x in vals {
            let er4 = ExtRes::from_i128(x);
            let er8 = ExtRes8::from_i128(x);
            assert_eq!(er8.small_part(), er4.small_part(),
                "small_part mismatch at x={}", x);
        }
    }

    /// garner6_to_u128 is consistent with ExtRes8 residues.
    #[test]
    fn garner6_roundtrip() {
        let m6 = M_NTT_6;
        let mut h: u64 = 0xC0FFEE11;
        for _ in 0..300 {
            h = h.wrapping_mul(6364136223846793005).wrapping_add(1);
            let x = (h as u128) % m6;
            let r6: [u64; 6] = NTT_PRIMES_6.map(|p| (x % p as u128) as u64);
            let g6 = garner6_to_u128(&r6);
            assert_eq!(g6, x % m6, "garner6 mismatch at x={}", x);
        }
    }

    /// ext_to_i128_8 correctly recovers signed values including those > M_NTT/2.
    #[test]
    fn ext_to_i128_8_signed_range() {
        let vals: &[i128] = &[
            0, 1, -1, 1_000_000_000, -1_000_000_000,
            1_000_000_000_000, -1_000_000_000_000,
            i64::MAX as i128, i64::MIN as i128,
        ];
        for &x in vals {
            let er8 = ExtRes8::from_i128(x);
            let recon = ext_to_i128_8(&er8);
            assert_eq!(recon, x, "ext_to_i128_8 mismatch at x={}", x);
        }
    }

    /// cram_div_exact_pos_8 matches canonical division for all NS divisors.
    #[test]
    fn div8_exact_pos_matches_canonical() {
        let divisors: [u64; 5] = [1u64 << 30, 100_000_000, 40_000, 20_000, 1_024];
        let test_xs: Vec<i128> = vec![
            0, 1, 1023, 1024, 100_000, 1_000_000,
            1_000_000_000_000, 4_000_000_000_000_000,
        ];
        for d in divisors {
            for &x in &test_xs {
                let er8 = ExtRes8::from_i128(x);
                let q8 = cram_div_exact_pos_8(&er8, d);
                let canonical_q = x / d as i128;
                let canonical_er8 = ExtRes8::from_i128(canonical_q);
                assert_eq!(q8.res, canonical_er8.res,
                    "div8_exact_pos d={} x={}: got {:?} expected {:?} (q={})",
                    d, x, q8.res, canonical_er8.res, canonical_q);
            }
        }
    }

    /// trunc_div_signed_8 matches canonical division for signed inputs including
    /// values > M_NTT_4/2 (the old failure regime).
    #[test]
    fn trunc_div_signed_8_signed_range() {
        let divisors: [u64; 5] = [1u64 << 30, 100_000_000, 40_000, 20_000, 1_024];
        // Include values > M_NTT_4/2 ≈ 5e11 that broke the 4-prime sign detection
        let test_xs: Vec<i128> = vec![
            0, 1, -1, 1023, -1023, 1_000_000, -1_000_000,
            1_000_000_000_000, -1_000_000_000_000,  // > M_NTT_4/2 — old failure point
            5_000_000_000_000i128, -5_000_000_000_000i128,
            i64::MAX as i128 / 2, -(i64::MAX as i128) / 2,
        ];
        for d in divisors {
            let ctx = DivCtx8::build(d);
            for &x in &test_xs {
                let er8 = ExtRes8::from_i128(x);
                let q8 = trunc_div_signed_8(&er8, &ctx);
                let canonical_q = x / d as i128;
                let canonical_er8 = ExtRes8::from_i128(canonical_q);
                assert_eq!(q8.res, canonical_er8.res,
                    "trunc_div_signed_8 d={} x={}: got {:?} expected {:?} (q={})",
                    d, x, q8.res, canonical_er8.res, canonical_q);
            }
        }
    }

    /// DivCtx8 builds valid 6-prime inverses.
    #[test]
    fn divcxt8_inverses_correct() {
        let divisors: [u64; 5] = [1u64 << 30, 100_000_000, 40_000, 20_000, 1_024];
        for d in divisors {
            let ctx = DivCtx8::build(d);
            for i in 0..6 {
                let p = EXT_BASIS_8[i];
                let check = ((ctx.inv_d_ntt[i] as u128 * (d as u128 % p as u128)) % p as u128) as u64;
                assert_eq!(check, 1,
                    "DivCtx8 d={}: inv_d_ntt[{}] * d mod {} = {} ≠ 1", d, i, p, check);
            }
        }
    }

    /// q_bound for 8-lane DivCtx8 is 2.25M× larger than 4-lane bound.
    #[test]
    fn q_bound_8_much_larger() {
        use crate::cram_div::{M_NTT, div_contract_partial_mod};
        let divisors: [u64; 5] = [1u64 << 30, 100_000_000, 40_000, 20_000, 1_024];
        for d in divisors {
            let ctx8 = DivCtx8::build(d);
            let partial = div_contract_partial_mod(d);
            let old_bound = M_NTT * partial;
            let new_bound = ctx8.q_bound;
            let ratio = new_bound / old_bound.max(1);
            assert!(ratio > 2_000_000,
                "d={}: q_bound ratio={}× expected >2M× (old={} new={})",
                d, ratio, old_bound, new_bound);
        }
    }

    /// Stress: trunc_div_signed_8 against canonical at i64-range values.
    #[test]
    fn stress_trunc_div8_i64_range() {
        let divisors: [u64; 5] = [1u64 << 30, 100_000_000, 40_000, 20_000, 1_024];
        let mut h: u64 = 0xDEAD8;
        for _ in 0..200 {
            h = h.wrapping_mul(6364136223846793005).wrapping_add(1);
            let x = (h as i128 & ((1i128 << 62) - 1)) - (1i128 << 61); // signed range
            for &d in &divisors {
                let ctx = DivCtx8::build(d);
                let er8 = ExtRes8::from_i128(x);
                let q8 = trunc_div_signed_8(&er8, &ctx);
                let canonical_q = x / d as i128;
                let canonical_er8 = ExtRes8::from_i128(canonical_q);
                assert_eq!(q8.res, canonical_er8.res,
                    "stress d={} x={}: got {:?} exp {:?} (q={})",
                    d, x, q8.res, canonical_er8.res, canonical_q);
            }
        }
    }
}
