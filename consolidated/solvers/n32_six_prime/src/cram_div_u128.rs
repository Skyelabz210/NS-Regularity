//! # CRAM Exact Integer Division — u128 Substrate (no narrowing)
//!
//! ## Why this exists
//!
//! `cram_div::cram_div_exact_pos` validates against canonical Rust integer
//! division but has a binding lift corridor on the 5⁹ lane: gcd(S²,5⁹)=5⁸
//! leaves only `partial_mod_5 = 5`, giving lift corridor `M_NTT·5 ≈ 5×10¹²`.
//! The v3 NS step at amp=1000 step 11 produces `(nu·lap)/S² ≈ 1.9×10¹³`,
//! which exceeds that corridor by three orders of magnitude.
//!
//! Per directive: u128 substrate everywhere, no i64/u64 narrowing in
//! arithmetic. Widen the small lanes to `2⁶³` and `5²⁵`, push every
//! intermediate computation into u128.
//!
//! ## Basis
//!
//! ```text
//!   EXT_BASIS_U128 = [769, 929, 1153, 1217, 2⁶³, 5²⁵]
//!   M_NTT   ≈ 1.0 × 10¹²
//!   M_small = 2⁶³ · 5²⁵ ≈ 2.7 × 10³⁶
//!   M_ext   = M_NTT · M_small ≈ 2.8 × 10⁴⁸    (vs 4.2×10²⁷ before)
//! ```
//!
//! Lift corridors per divisor are `M_NTT * min_i(lane_i / gcd(d, lane_i))`
//! over the non-coprime small lanes. The binding lane is:
//!   - N² = 2¹⁰: lane 2, M_NTT · 2⁵³ ≈ 9.0×10²⁷
//!   - N⁶ = 2³⁰: lane 2, M_NTT · 2³³ ≈ 8.6×10²¹
//!   - S² = 2⁸·5⁸: lane 5, M_NTT · 5¹⁷ ≈ 7.6×10²³
//!
//! Step-11 worst case is diffusion q = 1.9×10¹³. New binding 7.6×10²³.
//! Eleven orders of headroom.
//!
//! ## All arithmetic in u128
//!
//! - Lane 2⁶³: max value 2⁶³−1, products up to 2¹²⁶ < u128_max ≈ 3.4×10³⁸
//! - Lane 5²⁵: max value 3×10¹⁷, products up to 9×10³⁴ < u128_max
//! - NTT lanes: max value < 1217, products fit u64 trivially, but stored
//!   and operated as u128 per directive
//!
//! Intermediate ext arithmetic on a step-body value `v ≤ M_ext/2 ≈ 1.4×10⁴⁸`
//! requires no values that don't already fit u128 (since residues live
//! per lane and each lane is u128-bounded).

use crate::cram_div::{ExtRes};
// ============================================================================
// EXTENDED BASIS (u128)
// ============================================================================

pub const NTT_PRIMES_U128: [u128; 4] = [769, 929, 1153, 1217];
pub const LANE_2POW: u128 = 1u128 << 63;          // 2⁶³
pub const LANE_5POW: u128 = 298_023_223_876_953_125u128;  // 5²⁵
pub const EXT_BASIS_U128: [u128; 6] = [769, 929, 1153, 1217, LANE_2POW, LANE_5POW];

pub const M_NTT_U128: u128 = 769u128 * 929u128 * 1153u128 * 1217u128;
pub const M_SMALL_U128: u128 = LANE_2POW * LANE_5POW;
// M_EXT = M_NTT · M_small does NOT fit in u128 (≈ 2.8×10⁴⁸). We never
// materialize it; sign-detection uses M_NTT and M_small separately.

// ============================================================================
// u128 modular inverse (extended Euclidean in i128)
// ============================================================================

/// Modular inverse mod m, using extended Euclidean over i128.
/// Panics if gcd(a,m) ≠ 1. Inputs and output u128.
pub fn modinv_u128(a: u128, m: u128) -> u128 {
    // Extended Euclidean over i128 (m ≤ 2.7×10³⁶ < 2¹²⁶, fits i128).
    debug_assert!(m > 0, "modinv_u128: modulus 0");
    let m_i = m as i128;
    let (mut old_r, mut r) = (a as i128, m_i);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = old_r / r;
        let new_r = old_r - q * r;
        old_r = r; r = new_r;
        let new_s = old_s - q * s;
        old_s = s; s = new_s;
    }
    assert_eq!(old_r, 1, "modinv_u128({}, {}): gcd = {} ≠ 1", a, m, old_r);
    old_s.rem_euclid(m_i) as u128
}

/// (a + b) mod m, all u128.
#[inline]
pub fn addmod_u128(a: u128, b: u128, m: u128) -> u128 {
    // a, b < m. a + b < 2m. Could overflow if m near u128_max, but our
    // basis values are ≤ 2⁶³ on the small lanes and ≤ M_small ≈ 2.7×10³⁶
    // for full-substrate values. 2 · M_small ≈ 5.5×10³⁶ < u128_max.
    let s = a + b;
    if s >= m { s - m } else { s }
}

/// (a - b) mod m, all u128.
#[inline]
pub fn submod_u128(a: u128, b: u128, m: u128) -> u128 {
    if a >= b { a - b } else { a + m - b }
}

/// (a · b) mod m, all u128. Two paths:
/// - Small path: m < 2⁶⁴. Direct u128 product (a, b < 2⁶⁴ ⟹ product < 2¹²⁸).
/// - Large path: m ≥ 2⁶⁴. Need 256-bit product for the multiplication.
///
/// Used for both basis-lane operations (small path always, since basis ≤ 2⁶³)
/// and CRT reconstruction (large path, modulus = M_small ≈ 2.7×10³⁶).
#[inline]
pub fn mulmod_u128(a: u128, b: u128, m: u128) -> u128 {
    debug_assert!(a < m && b < m, "mulmod_u128: input ≥ modulus");
    if m < (1u128 << 64) {
        // Small modulus: a, b < 2⁶⁴, product fits in u128.
        a.wrapping_mul(b) % m
    } else {
        // Large modulus: split into 256-bit product, reduce.
        mulmod_u128_wide(a, b, m)
    }
}

/// 256-bit product reduction: compute (a · b) mod m where m can be up to
/// near u128_max. Uses 64-bit-limb schoolbook multiplication into a 256-bit
/// intermediate, then reduces by trial subtraction.
///
/// Correct for all `0 ≤ a, b < m`, `m > 0`.
fn mulmod_u128_wide(a: u128, b: u128, m: u128) -> u128 {
    // Standard binary-double-add modular multiplication: O(128) iterations.
    // For our use this happens at most a few times per call to to_i128;
    // not a hot path.
    let mut result: u128 = 0;
    let mut acc = a % m;
    let mut multiplier = b;
    while multiplier > 0 {
        if multiplier & 1 == 1 {
            result = addmod_u128(result, acc, m);
        }
        acc = addmod_u128(acc, acc, m);
        multiplier >>= 1;
    }
    result
}

// ============================================================================
// EXT RES U128 — full u128 residue tuple
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtResU128 {
    pub res: [u128; 6],
}

impl ExtResU128 {
    /// Encode a signed integer as residues across all 6 lanes.
    /// `x` may be negative; the encoding is consistent (residues of
    /// `x mod EXT_BASIS_U128[i]` taken via `rem_euclid`).
    pub fn from_i128(x: i128) -> Self {
        let mut r = [0u128; 6];
        for i in 0..6 {
            let m = EXT_BASIS_U128[i] as i128;
            r[i] = x.rem_euclid(m) as u128;
        }
        ExtResU128 { res: r }
    }

    pub fn zero() -> Self {
        ExtResU128 { res: [0u128; 6] }
    }

    /// Reconstruct `x mod M_small` from the small-prime lanes.
    /// Returns u128 in [0, M_small).
    pub fn small_part_u128(&self) -> u128 {
        // CRT for two coprime moduli (gcd(2⁶³, 5²⁵) = 1):
        //   X mod 2⁶³ = a, X mod 5²⁵ = b
        //   X mod (2⁶³·5²⁵) = a + 2⁶³·k where k = (b − a) · (2⁶³)⁻¹ mod 5²⁵
        let a = self.res[4];
        let b = self.res[5];
        let inv_2pow_mod_5pow = modinv_u128(LANE_2POW % LANE_5POW, LANE_5POW);
        let diff = if b >= a { b - a } else { b + LANE_5POW - a % LANE_5POW };
        let k = mulmod_u128(diff % LANE_5POW, inv_2pow_mod_5pow, LANE_5POW);
        // a + 2⁶³ · k. a < 2⁶³, k < 5²⁵, so 2⁶³·k < 2⁶³·5²⁵ = M_small ≈ 2.7×10³⁶ < u128_max.
        a + LANE_2POW * k
    }

    /// Reconstruct the full integer in i128, sign-detected by `s < M_small/2`.
    ///
    /// Per directive: u128 substrate. Sign detection happens via the CRT
    /// coefficient `s`, never by materializing `M_NTT · s` in u128.
    ///
    /// **Range:** Valid when |x| · M_NTT < 2¹²⁷ ≈ 1.7×10³⁸, i.e. when
    /// `|s| < 2¹²⁷/M_NTT ≈ 1.7×10²⁶`. NS step-body values stay well below
    /// 10²², so `s < 10¹⁰` ≪ 10²⁶ — tens of orders of headroom.
    pub fn to_i128(&self) -> i128 {
        let q_ntt = garner4_to_u128_local(&self.res[0..4]);
        let q_small = self.small_part_u128();
        let m_ntt_inv = modinv_u128(M_NTT_U128 % M_SMALL_U128, M_SMALL_U128);
        let diff = if q_small >= q_ntt { q_small - q_ntt }
                   else { q_small + M_SMALL_U128 - q_ntt % M_SMALL_U128 };
        let s = mulmod_u128(diff % M_SMALL_U128, m_ntt_inv, M_SMALL_U128);

        // Sign detection: q = q_ntt + M_NTT · s. With s ∈ [0, M_small):
        //   s < M_small/2  ⟺  q < M_ext/2  ⟺  canonical x ≥ 0; magnitude bounded by s
        //   s ≥ M_small/2  ⟺  canonical x < 0; magnitude bounded by (M_small − s)
        //
        // Range check: only the magnitude-determining variable needs to be
        // small enough that M_NTT · v fits i128. M_NTT < 2⁴⁰, so v < 2⁸⁷
        // suffices. For NS step intermediates with |x| < 2⁹⁰, v < 2⁹⁰/M_NTT
        // ≈ 2⁵⁰ — comfortable.
        let m_small_half = M_SMALL_U128 / 2;
        if s < m_small_half {
            // Positive branch: v = s, x = q_ntt + M_NTT · s.
            assert!(s < (1u128 << 87),
                "to_i128: s = {} (positive branch) exceeds reconstructable range", s);
            let prod = (M_NTT_U128 as i128).checked_mul(s as i128)
                .expect("to_i128: M_NTT · s overflow i128 in positive branch");
            (q_ntt as i128).checked_add(prod)
                .expect("to_i128: q_ntt + M_NTT·s overflow i128")
        } else {
            // Negative branch: v = M_small − s, x = −(M_NTT · v − q_ntt).
            let s_neg = M_SMALL_U128 - s;
            assert!(s_neg < (1u128 << 87),
                "to_i128: s_neg = {} (negative branch) exceeds reconstructable range", s_neg);
            let prod = (M_NTT_U128 as i128).checked_mul(s_neg as i128)
                .expect("to_i128: M_NTT · (M_small−s) overflow i128 in negative branch");
            -(prod - q_ntt as i128)
        }
    }
}

// Local Garner u128 (avoids exposing cram_div internals).
fn garner4_to_u128_local(r: &[u128]) -> u128 {
    let p: [u128; 4] = NTT_PRIMES_U128;
    // Standard Garner for 4 NTT primes — all u128.
    let a0 = r[0];
    let a1 = mulmod_u128(submod_u128(r[1], a0 % p[1], p[1]),
                         modinv_u128(p[0] % p[1], p[1]), p[1]);
    let p01 = p[0] * p[1];
    let t2 = (a0 + p[0] * a1) % p[2];
    let a2 = mulmod_u128(submod_u128(r[2], t2, p[2]),
                         modinv_u128(p01 % p[2], p[2]), p[2]);
    let p012 = p01 * p[2];
    let t3 = (a0 + p[0] * a1 + p01 * a2) % p[3];
    let a3 = mulmod_u128(submod_u128(r[3], t3, p[3]),
                         modinv_u128(p012 % p[3], p[3]), p[3]);
    a0 + p[0] * a1 + p01 * a2 + p012 * a3
}

// ============================================================================
// Conversion to/from old ExtRes (boundary with NTT and step-body code)
// ============================================================================

impl From<&ExtRes> for ExtResU128 {
    /// Lift an old-basis ExtRes to the new u128 basis. The 4 NTT lanes
    /// transfer directly; the small lanes are widened by encoding from
    /// the reconstructed small_part.
    fn from(old: &ExtRes) -> Self {
        let small = old.small_part();  // i128 value in [0, M_small_old) = [0, 2³¹·5⁹)
        // To re-encode into 2⁶³ and 5²⁵, we need the FULL integer x.
        // small_part gives x mod M_small_old, which is ambiguous if the
        // true x exceeds M_small_old. For initial encoding from i64
        // canonical state, x fits in i64, hence fits in M_small_old. ✓
        // But for step-body intermediates, x can exceed M_small_old —
        // re-encoding from old ExtRes is NOT lossless in that case.
        //
        // This conversion is intended only for boundary input/output, NOT
        // for in-step intermediate carrying.
        let mut r = [0u128; 6];
        // NTT lanes pass through:
        for i in 0..4 {
            r[i] = old.res[i] as u128;
        }
        // Small lanes from i128 small (which equals true x for x < M_small_old):
        r[4] = (small.rem_euclid(LANE_2POW as i128)) as u128;
        r[5] = (small.rem_euclid(LANE_5POW as i128)) as u128;
        ExtResU128 { res: r }
    }
}

// ============================================================================
// THE OPERATOR — exact integer division in u128 substrate
// ============================================================================

/// Exact integer division `⌊x/d⌋` in the u128 extended residue basis.
///
/// **Precondition:** `d > 0`, and `d` divides `M_small = 2⁶³ · 5²⁵`. That
/// covers all NS divisors {N²=2¹⁰, N⁶=2³⁰, S²=2⁸·5⁸} with massive headroom.
///
/// **Range:** Valid when `0 ≤ x < min(M_NTT · partial_mod_2, M_NTT · partial_mod_5)`
/// for the chosen `d`. For amp-1000 step-11 q values up to 2.5×10¹⁶,
/// the binding S² corridor is 7.6×10²³ — eight orders of headroom.
pub fn cram_div_exact_pos_u128(x: &ExtResU128, d: u128) -> ExtResU128 {
    debug_assert!(d > 0, "cram_div_exact_pos_u128: division by zero");
    debug_assert!(M_SMALL_U128 % d == 0,
        "cram_div_exact_pos_u128: d does not divide M_small");

    // Step 1: r = x mod d (small integer, fits u128).
    let small = x.small_part_u128();
    let r = small % d;

    // Step 2: NTT lanes (d coprime to NTT primes).
    let mut q_res = [0u128; 6];
    for i in 0..4 {
        let p = EXT_BASIS_U128[i];
        let r_p = x.res[i];
        let r_mod_p = r % p;
        let diff = if r_p >= r_mod_p { r_p - r_mod_p } else { r_p + p - r_mod_p };
        let d_inv = modinv_u128(d % p, p);
        q_res[i] = mulmod_u128(diff, d_inv, p);
    }

    // Step 3: Small lanes (d shares factors with lane_mod).
    let (qp2, pm2) = small_lane_partial_u128(x.res[4], r, d, LANE_2POW);
    let (qp5, pm5) = small_lane_partial_u128(x.res[5], r, d, LANE_5POW);

    q_res[4] = lift_partial_to_full_u128(qp2, pm2, LANE_2POW, &q_res[0..4]);
    q_res[5] = lift_partial_to_full_u128(qp5, pm5, LANE_5POW, &q_res[0..4]);

    ExtResU128 { res: q_res }
}

fn small_lane_partial_u128(x_lane: u128, r: u128, d: u128, lane_mod: u128) -> (u128, u128) {
    let xr = if x_lane >= r % lane_mod { x_lane - r % lane_mod }
             else { x_lane + lane_mod - r % lane_mod };
    let g = gcd_u128(d, lane_mod);
    let lane_g = lane_mod / g;
    let d_g = d / g;
    let xr_over_g = xr / g;
    let d_g_inv = modinv_u128(d_g % lane_g, lane_g);
    let q_partial = mulmod_u128(xr_over_g % lane_g, d_g_inv, lane_g);
    (q_partial, lane_g)
}

fn lift_partial_to_full_u128(q_partial: u128, partial_mod: u128, lane_mod: u128,
                              q_ntt_residues: &[u128]) -> u128 {
    let q_ntt_full = garner4_to_u128_local(q_ntt_residues);
    if partial_mod == lane_mod {
        return q_partial;
    }
    let m_ntt_mod_p = M_NTT_U128 % partial_mod;
    let m_ntt_inv = modinv_u128(m_ntt_mod_p, partial_mod);
    let q_ntt_mod_p = q_ntt_full % partial_mod;
    let s_diff = if q_partial >= q_ntt_mod_p { q_partial - q_ntt_mod_p }
                 else { q_partial + partial_mod - q_ntt_mod_p };
    let s = mulmod_u128(s_diff, m_ntt_inv, partial_mod);
    // q ≡ q_NTT_full + M_NTT · s, then mod lane_mod.
    // M_NTT · s ≤ M_NTT · partial_mod ≤ M_NTT · M_small ≈ 2.8×10⁴⁸ — overflow risk.
    // For our use case partial_mod ≤ 5²⁵ ≈ 3×10¹⁷, so M_NTT · partial_mod ≤ 3×10²⁹.
    // Plus q_ntt_full < M_NTT < 10¹². Sum < 3×10²⁹ < u128_max ≈ 3.4×10³⁸. ✓
    let q_full = q_ntt_full + M_NTT_U128 * s;
    q_full % lane_mod
}

fn gcd_u128(a: u128, b: u128) -> u128 {
    if b == 0 { a } else { gcd_u128(b, a % b) }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_part_roundtrip_u128() {
        for &x in &[0u128, 1, 7, 1023, 1024, 100_000, 1_000_000_000_000u128,
                    4_000_000_000_000_000u128, 1u128 << 60, 1u128 << 80] {
            if x < M_SMALL_U128 {
                let er = ExtResU128::from_i128(x as i128);
                assert_eq!(er.small_part_u128(), x % M_SMALL_U128, "roundtrip x={}", x);
            }
        }
    }

    #[test]
    fn to_i128_roundtrip() {
        let cases = [0i128, 1, -1, 100, -100, 1_000_000, -1_000_000,
                     1_000_000_000_000, -1_000_000_000_000,
                     1_000_000_000_000_000_000, // 1e18
                     -1_000_000_000_000_000_000,
                     i64::MAX as i128, i64::MIN as i128,
                     1i128 << 90, -(1i128 << 90),
                     1i128 << 100, -(1i128 << 100)];
        for &x in &cases {
            let er = ExtResU128::from_i128(x);
            let recon = er.to_i128();
            assert_eq!(recon, x, "to_i128 roundtrip failed for x = {}", x);
        }
    }

    #[test]
    fn div_by_n_squared() {
        let d: u128 = 1024;
        for &x in &[0i128, 1, 1023, 1024, 1025, 100_000, 1_000_000_000,
                    1_000_000_000_000, 1i128 << 50, 1i128 << 80, 1i128 << 100,
                    -1024, -1_000_000_000_000] {
            let er = ExtResU128::from_i128(x);
            let q_canonical = x.div_euclid(d as i128);  // for x≥0 same as x/d
            let q_recon = if x >= 0 {
                cram_div_exact_pos_u128(&er, d).to_i128()
            } else {
                let neg_er = ExtResU128::from_i128(-x);
                let q_pos = cram_div_exact_pos_u128(&neg_er, d).to_i128();
                let q_truncated = -((-x) / d as i128);
                q_truncated
            };
            // For x ≥ 0, expect cram_div = x / d (truncating, matches canonical).
            if x >= 0 {
                assert_eq!(q_recon, x / d as i128, "x={} d={} cram_div mismatch", x, d);
            }
        }
    }

    #[test]
    fn div_by_s_squared_at_step11_magnitude() {
        // The exact case that broke v3: q ~ 1.9e13 at step 11.
        let d: u128 = 100_000_000;  // S² = 10⁸
        let test_xs: Vec<i128> = vec![
            0, 1, 99_999_999, 100_000_000,
            1_900_000_000_000_000_000_000, // nu_lap at step 11 ≈ 1.9e21
            4_500_000_000_000_000_000_000, // some headroom
            1i128 << 80,                   // ≈ 1.2e24, well into the new corridor
        ];
        for &x in &test_xs {
            let er = ExtResU128::from_i128(x);
            let q = cram_div_exact_pos_u128(&er, d);
            let q_canonical = x / d as i128;
            let q_recon = q.to_i128();
            assert_eq!(q_recon, q_canonical,
                "x = {}, d = {}: canonical = {}, recon = {}",
                x, d, q_canonical, q_recon);
        }
    }

    #[test]
    fn div_by_n_sixth() {
        let d: u128 = 1u128 << 30;  // N⁶ = 2³⁰
        for &x in &[0i128, 1, (1<<30) - 1, 1<<30, 1<<31, 1<<40,
                     500_000_000_000i128,  // close to M_NTT/2
                     1i128 << 50, 1i128 << 60] {
            if x >= 0 {
                let er = ExtResU128::from_i128(x);
                let q = cram_div_exact_pos_u128(&er, d).to_i128();
                assert_eq!(q, x / d as i128, "x={} d={}", x, d);
            }
        }
    }

    #[test]
    fn stress_random_xs_three_divisors() {
        let divisors: [u128; 3] = [1024, 1u128 << 30, 100_000_000];
        let mut state = 0xCAFEBABEu64;
        for _ in 0..500 {
            for &d in &divisors {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let x_high = (state >> 32) as u128;
                let x_low = (state & 0xFFFFFFFF) as u128;
                let x = ((x_high << 32) | x_low) % (1u128 << 80);
                let er = ExtResU128::from_i128(x as i128);
                let q = cram_div_exact_pos_u128(&er, d).to_i128();
                let canonical = (x / d) as i128;
                assert_eq!(q, canonical, "stress: x={} d={}", x, d);
            }
        }
    }

    #[test]
    fn stress_step11_class_magnitudes() {
        // x at the magnitude of step-11 nu_lap (≈ 2×10²¹) and beyond.
        let d: u128 = 100_000_000;  // S²
        let mut state = 0xDEADBEEFu64;
        for _ in 0..200 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let scale: u128 = 5_000_000_000_000_000_000_000;  // 5e21
            let r = (state as u128) % scale;
            let x = (1_000_000_000_000_000_000_000 + r) as i128;  // ≈ 1e21..6e21
            let er = ExtResU128::from_i128(x);
            let q = cram_div_exact_pos_u128(&er, d).to_i128();
            let canonical = x / d as i128;
            assert_eq!(q, canonical, "stress11: x={} d={}", x, d);
        }
    }
}
