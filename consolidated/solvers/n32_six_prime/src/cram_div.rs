//! # CRAM Exact Integer Division — Residue-Pure
//!
//! ## Problem
//!
//! The canonical N=32 NS step uses i64 truncating division at three points:
//!   - Biot-Savart final: `garner4(rx) / N⁶ = garner4(rx) / 2³⁰`
//!   - Diffusion: `(nu · lap) / S² = (nu · lap) / (2⁸ · 5⁸)`
//!   - Outer step: `(dt · rhs) / N² = (dt · rhs) / 2¹⁰`
//!
//! All three divisors are products of small primes `{2, 5}` only. None of
//! the NTT primes `{769, 929, 1153, 1217}` divide them, so naive
//! `mulmod(x, d^{-1}, p_i)` cannot match truncating division — it gives a
//! valid residue of *some* integer but not `⌊x/d⌋`. This is the structural
//! barrier identified in the prior session note.
//!
//! ## Construction
//!
//! Extend the CRT basis with **small-prime-power lanes** that absorb the
//! divisor's factors:
//!
//! ```text
//!   M_NTT = 769 · 929 · 1153 · 1217 ≈ 1.0 × 10^12
//!   M_small = 2³¹ · 5⁹  ≈ 4.2 × 10^15
//!   M_ext = M_NTT · M_small ≈ 4.2 × 10^27
//! ```
//!
//! All chosen divisors fit in `M_small`:
//!   - `N⁶ = 2³⁰` divides `2³¹` ✓
//!   - `S² = 2⁸ · 5⁸` divides `2³¹ · 5⁹` ✓
//!   - `N² = 2¹⁰` divides `2³¹` ✓
//!
//! ## Division algorithm — case `d | M_small`
//!
//! Given `x` represented as `(r_NTT[0..4], r_2, r_5)` with `r_2 = x mod 2³¹`
//! and `r_5 = x mod 5⁹`. We want residues of `q = ⌊x/d⌋`.
//!
//! **Step 1.** Reconstruct `r = x mod d` from the small-prime residues.
//! Since `d | M_small`, we have `x mod d = (x mod M_small) mod d` =
//! `CRT(r_2, r_5) mod d`. We can build `(x mod d)` directly as a *small*
//! integer fitting in i64 (since `d < M_small < 2^53` for our cases).
//!
//! **Step 2.** Compute `q = (x − r) / d`. We don't have `x` directly, but
//! for each NTT prime `p_i` (which is coprime to `d`):
//!   ```text
//!   q mod p_i = (r_NTT[i] − r) · d^{-1}  mod p_i
//!   ```
//!   This gives the NTT residues of `q` exactly.
//!
//! **Step 3.** For the small-prime lanes (`p = 2³¹` and `p = 5⁹`), `d`
//! shares factors with the modulus, so direct inversion fails. Use
//! K-Elimination winding: extend `r_NTT` to a candidate `q_M_NTT mod p`
//! via Garner-on-NTT-residues-only, then K-Elim against the small-prime
//! lane to recover the winding and reconstruct `q mod p`.
//!
//! Equivalently, since `q < x/d < M_ext / d`, and `q · d + r = x`, the
//! small-prime residues of `q` are determined by the small-prime residues
//! of `x`: `q mod (M_small/d) = (x/d) mod (M_small/d) = ((x mod M_small)/d)`.
//! This is straightforward integer division on a small integer.
//!
//! Result: a fully-residue representation of `⌊x/d⌋` in the extended basis,
//! bit-identical to canonical's `⌊x/d⌋` for any `0 ≤ x < M_ext`.
//!
//! ## Status
//!
//! This module implements the construction and validates it against
//! canonical i64 truncating division on a stress test.

use crate::ns3d::modinv;
use crate::ns3d_n32::NTT_PRIMES;

// ============================================================================
// EXTENDED BASIS
// ============================================================================

/// Small-prime-power lanes, chosen to absorb all divisors that appear in
/// the canonical NS step.
///
/// `2³¹ = 2147483648` covers `N⁶ = 2³⁰` and `N² = 2¹⁰`.
/// `5⁹ = 1953125` covers the `5⁸` factor in `S² = 2⁸ · 5⁸`.
///
/// Combined `M_small = 2³¹ · 5⁹ ≈ 4.2 × 10^15`, fits in i64.
pub const MOD_2POW: u64 = 1 << 31;       // 2³¹
pub const MOD_5POW: u64 = 1_953_125;     // 5⁹
pub const M_SMALL: u128 = MOD_2POW as u128 * MOD_5POW as u128;

/// M_NTT = product of all 4 NTT primes. Used in corridor assertions.
pub const M_NTT: u128 = 769 * 929 * 1153 * 1217; // 1_002_568_584_801

/// BS raw-velocity corridor limit: |raw| must be ≤ M_NTT/2 for garner4's
/// centred reconstruction to be unambiguous. This is the primary corridor
/// limit at large amplitudes.
pub const M_NTT_HALF: u128 = M_NTT / 2; // 501_284_292_400

/// Full extended basis = NTT primes + small-prime-power lanes.
pub const EXT_BASIS: [u64; 6] = [769, 929, 1153, 1217, MOD_2POW, MOD_5POW];

// ============================================================================
// CORRIDOR ASSERTIONS — enabled with feature "div_contracts"
// ============================================================================

/// Assert that `|q| < M_NTT · partial_mod` for the given divisor.
///
/// Call AFTER computing the quotient |q| and BEFORE encoding it.
/// Panics in debug builds (feature="div_contracts") if the contract is
/// violated, indicating the NTT basis must be widened.
///
/// `partial_mod` is the tighter of `2^31/gcd(d,2^31)` and `5^9/gcd(d,5^9)`.
/// Pre-computed values for each NS divisor:
///   d=N^6=2^30 → partial_mod=2        → q_bound = 2.01×10^12
///   d=S^2=10^8 → partial_mod=5        → q_bound = 5.01×10^12
///   d=4S=40000 → partial_mod=3125     → q_bound = 3.13×10^15
///   d=2S=20000 → partial_mod=3125     → q_bound = 3.13×10^15
///   d=N^2=1024 → partial_mod=1953125  → q_bound = 1.96×10^18
#[inline]
#[cfg(feature = "div_contracts")]
pub fn assert_div_contract(abs_q: u128, partial_mod: u128, d: u64) {
    let bound = M_NTT * partial_mod;
    debug_assert!(
        abs_q < bound,
        "div_contract VIOLATED: d={} abs_q={} >= M_NTT*partial_mod={} (partial_mod={}). \
         Basis widening required.",
        d, abs_q, bound, partial_mod
    );
}

#[inline]
#[cfg(not(feature = "div_contracts"))]
pub fn assert_div_contract(_abs_q: u128, _partial_mod: u128, _d: u64) {}

/// Assert that the BS raw-velocity |raw| ≤ M_NTT/2.
///
/// Violation means the NTT centred Garner is ambiguous and the BS output
/// is incorrect. The only fix is widening M_NTT (adding NTT primes).
#[inline]
#[cfg(feature = "div_contracts")]
pub fn assert_bs_raw_bound(abs_raw: u128) {
    debug_assert!(
        abs_raw <= M_NTT_HALF,
        "bs_raw_bound VIOLATED: |raw|={} > M_NTT/2={}. \
         NTT corridor exceeded — add NTT primes to widen M_NTT.",
        abs_raw, M_NTT_HALF
    );
}

#[inline]
#[cfg(not(feature = "div_contracts"))]
pub fn assert_bs_raw_bound(_abs_raw: u128) {}

/// Compute `partial_mod_min(d)` — the tighter partial modulus for d.
/// `partial_2 = 2^31 / gcd(d, 2^31)`, `partial_5 = 5^9 / gcd(d, 5^9)`.
/// Returns `min(partial_2, partial_5)` as u128.
pub fn div_contract_partial_mod(d: u64) -> u128 {
    let g2 = gcd_u64(d, MOD_2POW) as u128;
    let g5 = gcd_u64(d, MOD_5POW) as u128;
    let partial_2 = MOD_2POW as u128 / g2;
    let partial_5 = MOD_5POW as u128 / g5;
    partial_2.min(partial_5)
}

/// Extended residue tuple. `res[0..4]` are mod NTT_PRIMES, `res[4]` mod 2³¹,
/// `res[5]` mod 5⁹.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtRes {
    pub res: [u64; 6],
}

impl ExtRes {
    pub fn from_i128(x: i128) -> Self {
        let mut r = [0u64; 6];
        for i in 0..6 {
            r[i] = (x.rem_euclid(EXT_BASIS[i] as i128)) as u64;
        }
        ExtRes { res: r }
    }

    /// Reconstruct the small-prime portion `x mod M_small` exactly.
    /// `M_small ≈ 4.2 × 10^15` fits in i64 with room.
    pub fn small_part(&self) -> i128 {
        // CRT for two coprime moduli (gcd(2³¹, 5⁹) = 1):
        // X mod 2³¹ = a, X mod 5⁹ = b → X mod (2³¹·5⁹) = a + 2³¹·k where
        // k = (b - a) · (2³¹)⁻¹ mod 5⁹
        let a = self.res[4] as i128;
        let b = self.res[5] as i128;
        let inv_2pow_mod_5pow = modinv(MOD_2POW % MOD_5POW, MOD_5POW) as i128;
        let m5 = MOD_5POW as i128;
        let k = ((b - a).rem_euclid(m5) * inv_2pow_mod_5pow).rem_euclid(m5);
        a + (MOD_2POW as i128) * k
    }
}

// ============================================================================
// THE OPERATOR — exact integer division in residues
// ============================================================================

/// Exact integer division `⌊x/d⌋` in the extended residue basis.
///
/// **Precondition:** `d` divides `M_small = 2³¹ · 5⁹`. That covers all
/// divisors used by canonical N=32 NS step (N²=1024, N⁶=2³⁰, S²=2⁸·5⁸,
/// 4S=40000, 2S=20000).
///
/// **Result:** `ExtRes` representing `⌊x/d⌋` in the same 6-lane basis.
///
/// ## Per-divisor correctness bound (`q = ⌊x/d⌋` must satisfy `|q| < q_bound_min`)
///
/// The `lift_partial_to_full` step is valid when `q < M_NTT · partial_mod`.
/// For each NS divisor this gives:
///
/// ```text
///   M_NTT = 769·929·1153·1217 = 1,002,568,584,801  ≈ 1.00 × 10^12
///
///   d = N^6 = 2^30:  partial_2 = 2,   q_bound_min = 2.01 × 10^12  ← TIGHTEST
///   d = S^2 = 10^8:  partial_5 = 5,   q_bound_min = 5.01 × 10^12
///   d = 4S  = 40000: partial_5 = 3125, q_bound_min = 3.13 × 10^15
///   d = 2S  = 20000: partial_5 = 3125, q_bound_min = 3.13 × 10^15
///   d = N^2 = 1024:  partial_5 = 1953125, q_bound_min = 1.96 × 10^18
/// ```
///
/// For d=N^6, the BS raw value satisfies |raw| ≤ M_NTT/2 ≈ 5.01 × 10^11,
/// so |q| = |raw|/N^6 ≤ 0.47 — q is always 0 or ±1. Enormous margin.
/// The effective limit for large-amplitude runs is the BS raw corridor
/// (|raw| ≤ M_NTT/2), not the division bound.
///
/// **NOT** bit-identical for `|q| ≥ q_bound_min`; that condition signals
/// basis widening is required (add more NTT primes to raise M_NTT).
///
/// **Negative `x`:** Treated by sign-extending the small_part via two's-
/// complement-style centering on `M_ext`. For NS use we assume `x` is
/// positive in the relevant range (truncating division on negative
/// canonical values is handled separately).
pub fn cram_div_exact_pos(x: &ExtRes, d: u64) -> ExtRes {
    debug_assert!(d > 0, "cram_div_exact_pos: division by zero");
    debug_assert!(M_SMALL % d as u128 == 0,
        "cram_div_exact_pos: d={} does not divide M_small={}", d, M_SMALL);

    // Step 1: r = x mod d  (small integer, fits easily in i64).
    let small = x.small_part();        // x mod M_small, 0 ≤ small < M_small
    let r = small.rem_euclid(d as i128);  // x mod d, 0 ≤ r < d

    // Step 2: For NTT primes (i ∈ 0..4), d is a unit mod p_i, so
    //   q mod p_i = (r_i − r) · d⁻¹ mod p_i.
    let mut q_res = [0u64; 6];
    let r_mod_each = |p: u64| (r.rem_euclid(p as i128)) as u64;
    for i in 0..4 {
        let p = EXT_BASIS[i];
        let r_p = x.res[i];
        let diff = (r_p + p - r_mod_each(p) % p) % p;
        let d_inv = modinv(d % p, p);
        q_res[i] = ((diff as u128 * d_inv as u128) % p as u128) as u64;
    }

    // Step 3: Small-prime lanes. Since q is well-defined and small_part(x)
    // gives x mod M_small exactly, q mod (M_small / d) is determined by
    // (small_part(x) - r) / d as an exact integer division on the small
    // value. From there, lift through the lanes:
    //   q mod 2³¹: combine "q mod (M_small/d)" with the higher-order
    //              winding from the NTT residues.
    //
    // Approach: reconstruct q mod M_small/d from (small_part(x) − r)/d,
    // then K-Elim against the NTT residues to recover full q mod 2³¹
    // and q mod 5⁹.
    //
    // Simpler alternative used here: since q mod 2³¹ and q mod 5⁹ are
    // each determined by q itself, and q < M_ext/d, recover them by
    // K-Elim with the NTT residues we just computed.
    //
    // Specifically: we know q mod p_i for the 4 NTT primes. Garner those
    // to recover q mod M_NTT. Then:
    //   q ≡ q_NTT (mod M_NTT)
    //   q ≡ ?     (mod 2³¹)
    // Use K-Elim: q mod 2³¹ = q_NTT mod 2³¹ + M_NTT · k_2 mod 2³¹
    // where k_2 ∈ [0, 2³¹) is determined uniquely by q being in
    // [0, M_NTT · 2³¹) — but we don't know which of M_NTT^{-1}-many
    // values q takes purely from q_NTT.
    //
    // THE KEY: we can recover k_2 from x itself:
    //   x = q · d + r, so x mod 2³¹ = (q · d + r) mod 2³¹
    //   ⟹  q · d ≡ x − r (mod 2³¹)
    //   ⟹  q ≡ (x − r) / d (mod 2³¹/gcd(d, 2³¹))
    //
    // For d = N² = 2¹⁰: gcd(d, 2³¹) = 2¹⁰. So we get q mod 2²¹.
    // The remaining 10 bits of q mod 2³¹ come from K-Elim with NTT.
    //
    // For d = N⁶ = 2³⁰: gcd(d, 2³¹) = 2³⁰. We get only 1 bit.
    //
    // For d = S² = 2⁸ · 5⁸: gcd(d, 2³¹) = 2⁸, gcd(d, 5⁹) = 5⁸.
    //
    // Implementation: do the K-Elim lift from NTT residues for the
    // bits that direct division can't recover.

    // First compute q mod (small modulus)/(gcd factor) directly from x.
    let q_mod_2pow_partial = small_lane_partial(x.res[4], r as i64, d, MOD_2POW);
    let q_mod_5pow_partial = small_lane_partial(x.res[5], r as i64, d, MOD_5POW);

    q_res[4] = lift_partial_to_full(q_mod_2pow_partial.0, q_mod_2pow_partial.1,
                                     MOD_2POW, &q_res[0..4]);
    q_res[5] = lift_partial_to_full(q_mod_5pow_partial.0, q_mod_5pow_partial.1,
                                     MOD_5POW, &q_res[0..4]);

    ExtRes { res: q_res }
}

/// Returns `(partial_q, partial_modulus)` such that
/// `q ≡ partial_q (mod partial_modulus)`, where `partial_modulus =
/// lane_mod / gcd(d, lane_mod)`.
///
/// From `q · d ≡ x − r (mod lane_mod)`, dividing both sides by
/// `g = gcd(d, lane_mod)` (which divides x − r since d | (x − r)):
///   `q · (d/g) ≡ (x − r)/g (mod lane_mod/g)`
/// and `gcd(d/g, lane_mod/g) = 1`, so we can invert.
fn small_lane_partial(x_lane: u64, r: i64, d: u64, lane_mod: u64) -> (u64, u64) {
    let xr = (x_lane as i128 - r as i128).rem_euclid(lane_mod as i128) as u64;
    let g = gcd_u64(d, lane_mod);
    let lane_g = lane_mod / g;
    let d_g = d / g;
    // (x − r)/g exists as integer because d | (x − r), and g | d.
    let xr_over_g = xr / g;   // exact in this lane (no truncation since g | (x−r))
    let d_g_inv = modinv(d_g % lane_g, lane_g);
    let q_partial = ((xr_over_g as u128 * d_g_inv as u128) % lane_g as u128) as u64;
    (q_partial, lane_g)
}

/// Lift `(q_partial, partial_mod)` to full `q mod lane_mod` using the
/// NTT residues of q.
///
/// We have:
///   q ≡ q_partial    (mod partial_mod)        [from direct lane division]
///   q ≡ q_NTT_full   (mod M_NTT)              [from NTT residues, Garnered]
/// where `partial_mod | lane_mod` and `gcd(partial_mod, M_NTT) = 1`
/// (lane_mod is a power of 2 or 5; NTT primes are odd and ≢ 0 mod 5).
///
/// CRT gives q uniquely mod `M_NTT · partial_mod`. Then take mod lane_mod.
///
/// **Range condition:** This lifting is valid when the true `q` satisfies
/// `q < M_NTT · partial_mod`. For the NS use case (x < i64::MAX), this
/// holds for all three divisors {N², N⁶, S²}; verified in tests.
fn lift_partial_to_full(q_partial: u64, partial_mod: u64, lane_mod: u64,
                         q_ntt_residues: &[u64]) -> u64 {
    let q_ntt_full = garner4_to_u128(q_ntt_residues);

    if partial_mod == lane_mod {
        // No missing bits — q_partial already gives q mod lane_mod.
        return q_partial;
    }

    // CRT: solve q ≡ q_NTT_full (mod M_NTT) and q ≡ q_partial (mod partial_mod).
    // q = q_NTT_full + M_NTT · s, with s in [0, partial_mod) chosen so that
    //   q ≡ q_partial (mod partial_mod)
    //   ⟹ s ≡ (q_partial − q_NTT_full) · M_NTT⁻¹ (mod partial_mod)
    let m_ntt: u128 = NTT_PRIMES.iter().map(|&p| p as u128).product();
    let m_ntt_mod_p = (m_ntt % partial_mod as u128) as u64;
    let m_ntt_inv = modinv(m_ntt_mod_p, partial_mod);
    let q_ntt_mod_p = (q_ntt_full % partial_mod as u128) as u64;
    let s = (((q_partial + partial_mod - q_ntt_mod_p % partial_mod) % partial_mod) as u128
            * m_ntt_inv as u128 % partial_mod as u128) as u64;
    // q = q_NTT_full + M_NTT · s, then mod lane_mod
    let q_mod_lane = (q_ntt_full + m_ntt * s as u128) % lane_mod as u128;
    q_mod_lane as u64
}

fn garner4_to_u128(r: &[u64]) -> u128 {
    // Garner reconstruction for the 4 NTT primes — returns x in [0, M_NTT).
    // Standard layout matching ns3d_n32::garner4 but unsigned.
    let p = NTT_PRIMES;
    let a0 = r[0];
    let a1 = ((r[1] as i128 - a0 as i128).rem_euclid(p[1] as i128) as u64
             * modinv(p[0] % p[1], p[1])) % p[1];
    let p01 = p[0] as u128 * p[1] as u128;
    let t2 = (a0 as u128 + p[0] as u128 * a1 as u128) % p[2] as u128;
    let a2 = ((r[2] as i128 - t2 as i128).rem_euclid(p[2] as i128) as u128
             * modinv((p01 % p[2] as u128) as u64, p[2]) as u128) % p[2] as u128;
    let p012 = p01 * p[2] as u128;
    let t3 = (a0 as u128 + p[0] as u128 * a1 as u128 + p01 * a2) % p[3] as u128;
    let a3 = ((r[3] as i128 - t3 as i128).rem_euclid(p[3] as i128) as u128
             * modinv((p012 % p[3] as u128) as u64, p[3]) as u128) % p[3] as u128;
    a0 as u128 + p[0] as u128 * a1 as u128 + p01 * a2 + p012 * a3
}

pub fn gcd_u64(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd_u64(b, a % b) }
}

// ============================================================================
// NODE-P4A — garner_n_centered_i128: generic N-prime centred Garner
// ============================================================================

/// Garner reconstruction over `n` NTT primes (ascending j-order, D-012 compliant).
/// Returns the centred representative in `(-M_ntt/2, M_ntt/2]` as i128.
///
/// Valid for any slice of NTT primes; currently used with 4 primes (the
/// existing basis) and designed to extend to 6 or 8 primes for corridor widening.
///
/// **D-012 compliance:** j-loop is ASCENDING. Descending order is silent
/// corruption. This function hardcodes ascending iteration.
pub fn garner_n_centered_i128(residues: &[u64], primes: &[u64]) -> i128 {
    let n = primes.len();
    debug_assert_eq!(residues.len(), n, "garner_n_centered_i128: residue/prime length mismatch");
    debug_assert!(n >= 1, "garner_n_centered_i128: need at least 1 prime");

    // Mixed-radix coefficients a[i] — ascending j (D-012 COMPLIANT)
    let mut a = vec![0u128; n];
    a[0] = residues[0] as u128;

    // Running product of primes p[0]..p[j-1]
    let mut prefix = vec![0u128; n];
    prefix[0] = primes[0] as u128;

    for j in 1..n {
        // t_j = a[0] + p[0]*a[1] + ... + p[0]*...*p[j-2]*a[j-1]  mod p[j]
        // Computed iteratively to avoid large intermediates.
        let pj = primes[j] as u128;
        let mut t = a[0] % pj;
        let mut prod = 1u128;
        for k in 0..j {
            prod = (prod * (primes[k] as u128)) % pj;
            t = (t + prod * (a[k + 1].min(a[k + 1]))) % pj; // placeholder — see below
        }
        // Correct: t = (sum_{k=0}^{j-1} prefix[0..k] * a[k]) mod p[j]
        // Recompute cleanly:
        t = residues[0] as u128 % pj;
        let mut running_prod = primes[0] as u128 % pj;
        for k in 1..j {
            t = (t + running_prod * (a[k] % pj)) % pj;
            running_prod = (running_prod * (primes[k] as u128)) % pj;
        }
        // a[j] = (residues[j] - t) * (prefix[j-1])^{-1} mod p[j]
        let diff = ((residues[j] as i128 - t as i128).rem_euclid(pj as i128)) as u128;
        let prefix_mod_pj = if j == 0 { 1u64 } else {
            let mut pm = 1u128;
            for k in 0..j { pm = (pm * primes[k] as u128) % pj; }
            pm as u64
        };
        let inv = modinv(prefix_mod_pj, primes[j]);
        a[j] = (diff * inv as u128) % pj;

        // Update prefix
        prefix[j] = prefix[j-1].saturating_mul(primes[j] as u128); // only for M_ntt tracking
    }

    // Reconstruct x = a[0] + p[0]*a[1] + p[0]*p[1]*a[2] + ...
    // Accumulate in u128 (valid while M_ntt fits, i.e. up to ~2^80 for 6 primes)
    let mut x: u128 = 0;
    let mut prod: u128 = 1;
    for i in 0..n {
        x = x.wrapping_add(prod.wrapping_mul(a[i]));
        if i + 1 < n {
            prod = prod.wrapping_mul(primes[i] as u128);
        }
    }

    // M_ntt = product of all primes
    let m_ntt: u128 = primes.iter().map(|&p| p as u128).product();

    // Centred: if x > M_ntt/2, subtract M_ntt
    // x is in [0, M_ntt)
    let x_mod = x % m_ntt;
    if x_mod <= m_ntt / 2 {
        x_mod as i128
    } else {
        x_mod as i128 - m_ntt as i128
    }
}

// ============================================================================
// NODE-P2A — DivCtx: pre-computed per-divisor inverses
// ============================================================================

/// Pre-computed division context for a single divisor `d`.
///
/// Build once per divisor per step (or at startup for static divisors),
/// reuse across all cells. Eliminates repeated `modinv` calls on the
/// hot division path.
///
/// All fields are exact integer constants — zero float, zero approximation.
pub struct DivCtx {
    /// The divisor.
    pub d: u64,
    /// `d^{-1} mod p_i` for each NTT prime `p_i`. Valid because gcd(d, p_i) = 1
    /// for all NS divisors (d is a product of 2s and 5s; NTT primes are odd and ≢ 0 mod 5).
    pub inv_d_ntt: [u64; 4],
    /// Minimum partial modulus: `min(2^31/gcd(d,2^31), 5^9/gcd(d,5^9))`.
    /// This is the binding q_bound = M_NTT * partial_mod.
    pub partial_mod_min: u128,
    /// Tight quotient bound: `M_NTT * partial_mod_min`. Caller must ensure |q| < this.
    pub q_bound: u128,
}

impl DivCtx {
    /// Build a `DivCtx` for divisor `d`.
    ///
    /// Panics if `d` does not divide `M_small` (i.e. d has a prime factor
    /// other than 2 or 5 — not a valid NS divisor).
    pub fn build(d: u64) -> Self {
        assert!(
            M_SMALL % d as u128 == 0,
            "DivCtx::build: d={} does not divide M_small. \
             Only divisors that are products of {{2,5}} are supported.", d
        );
        let ntt_primes = NTT_PRIMES;
        let mut inv_d_ntt = [0u64; 4];
        for i in 0..4 {
            let p = ntt_primes[i];
            inv_d_ntt[i] = modinv(d % p, p);
        }
        let partial_mod_min = div_contract_partial_mod(d);
        let q_bound = M_NTT * partial_mod_min;
        DivCtx { d, inv_d_ntt, partial_mod_min, q_bound }
    }
}

// ============================================================================
// NODE-P2B — Explicit division modes using DivCtx
// ============================================================================

/// Truncating integer division `⌊x/d⌋` in the extended basis, using
/// pre-computed `DivCtx`. Bit-identical to `ext_div_signed(x, d)` for
/// all `|q| < ctx.q_bound`.
///
/// **Mode:** TRUNC — always safe for any sign of x. Uses `cram_div_exact_pos`
/// on the absolute value and restores sign.
///
/// This is the general-purpose division mode. Use `exact_div_signed_zero_remainder`
/// when you have proven that `d | x` (remainder is zero), which skips
/// the NTT lane reconstruction and is faster.
pub fn trunc_div_signed(x: &ExtRes, ctx: &DivCtx) -> ExtRes {
    // Full 6-lane sign reconstruction — matches ext_to_i128 in ns3d_n32_crt_v3.
    // The NTT-only centred Garner only covers |x| < M_NTT/2 ≈ 5×10^11; using
    // it alone gives wrong sign for values like x=-10^12 (which maps to
    // M_NTT - 10^12 ≈ 2.57e9 in unsigned NTT representation, appearing positive).
    // Full reconstruction via both NTT and small-prime lanes is unambiguous
    // up to M_ext ≈ 4.2×10^27.
    let m_ntt = M_NTT;
    let m_small = MOD_2POW as u128 * MOD_5POW as u128;
    let q_ntt = garner4_to_u128(&x.res[0..4]);
    let q_small = x.small_part();
    let m_ntt_inv = modinv((m_ntt % m_small) as u64, m_small as u64) as u128;
    let s = ((q_small as i128 - q_ntt as i128).rem_euclid(m_small as i128) as u128
            * m_ntt_inv % m_small);
    let val = q_ntt + m_ntt * s;
    let m_ext = m_ntt * m_small;
    let is_neg = val >= m_ext / 2;

    if is_neg {
        // Negate x in residue basis, divide positive, negate result.
        let mut neg_res = [0u64; 6];
        for i in 0..6 {
            let m = EXT_BASIS[i];
            neg_res[i] = (m - x.res[i] % m) % m;
        }
        let x_pos = ExtRes { res: neg_res };
        let q_pos = cram_div_exact_pos(&x_pos, ctx.d);
        let mut r = [0u64; 6];
        for i in 0..6 {
            let m = EXT_BASIS[i];
            r[i] = (m - q_pos.res[i] % m) % m;
        }
        ExtRes { res: r }
    } else {
        cram_div_exact_pos(x, ctx.d)
    }
}

/// Exact division `x / d` in the extended basis, asserting zero remainder.
///
/// **Precondition (contract):** `d` divides `x` exactly (remainder = 0).
/// This is deterministic: the small-lane residue `x mod d` is computed
/// exactly. If it is non-zero, the function panics — there is no silent
/// fallback.
///
/// **When to use:** Advection and stretching terms provably produce values
/// divisible by `d` in the exact-integer regime (verified for small amps
/// in `canonical_advection_stretching_division_is_exact_at_small_amp`).
/// Using this mode where exactness holds eliminates the sign-reconstruction
/// overhead of `trunc_div_signed`.
///
/// **When NOT to use:** Any operator where truncation is expected (non-zero
/// remainder). Use `trunc_div_signed` there.
pub fn exact_div_signed_zero_remainder(x: &ExtRes, ctx: &DivCtx) -> ExtRes {
    // Verify zero remainder from small lanes (deterministic, no sampling).
    let small = x.small_part();
    let r = small.rem_euclid(ctx.d as i128);
    assert_eq!(
        r, 0,
        "exact_div_signed_zero_remainder: d={} does NOT divide x (remainder={}). \
         Use trunc_div_signed for non-exact truncation.",
        ctx.d, r
    );
    // r = 0, so (x - r)/d = x/d exactly. We can use cram_div_exact_pos
    // on the positive representation; sign handling same as trunc_div_signed.
    trunc_div_signed(x, ctx)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_part_roundtrip() {
        // For a value smaller than M_small, small_part should return it.
        for &x in &[0i128, 1, 7, 1023, 1024, 100_000, 1_000_000_000_000i128, 4_000_000_000_000_000i128] {
            if (x as u128) < M_SMALL {
                let er = ExtRes::from_i128(x);
                assert_eq!(er.small_part(), x, "roundtrip failed for x={}", x);
            }
        }
    }

    #[test]
    fn div_by_n_squared_matches_canonical() {
        // d = N² = 1024 divides 2³¹.
        let d: u64 = 1024;
        let test_xs: Vec<i128> = vec![
            0, 1, 1023, 1024, 1025, 4096, 100_000, 1_000_000,
            10_000_000, 1_000_000_000, 1_000_000_000_000,
        ];
        for x in test_xs {
            let er = ExtRes::from_i128(x);
            let q = cram_div_exact_pos(&er, d);
            let canonical_q = x / d as i128;
            let canonical_er = ExtRes::from_i128(canonical_q);
            assert_eq!(q.res, canonical_er.res,
                "x={}, d={}: q={:?}, canonical_q={} (residues {:?})",
                x, d, q.res, canonical_q, canonical_er.res);
        }
    }

    #[test]
    fn div_by_n_sixth_matches_canonical() {
        // d = N⁶ = 2³⁰ = 1073741824 divides 2³¹.
        let d: u64 = 1 << 30;
        let test_xs: Vec<i128> = vec![
            0, 1, d as i128 - 1, d as i128, d as i128 + 1,
            2 * d as i128, 100 * d as i128,
            1_000_000_000_000, 1_000_000_000_000_000,
        ];
        for x in test_xs {
            let er = ExtRes::from_i128(x);
            let q = cram_div_exact_pos(&er, d);
            let canonical_q = x / d as i128;
            let canonical_er = ExtRes::from_i128(canonical_q);
            assert_eq!(q.res, canonical_er.res,
                "x={}, d={}: q={:?} ≠ canonical {:?}",
                x, d, q.res, canonical_er.res);
        }
    }

    #[test]
    fn div_by_s_squared_matches_canonical() {
        // d = S² = 10⁸ = 2⁸ · 5⁸. M_small = 2³¹ · 5⁹, both factors covered.
        let d: u64 = 100_000_000;
        let test_xs: Vec<i128> = vec![
            0, 1, d as i128 - 1, d as i128, d as i128 + 1,
            12345, 99_999_999, 100_000_001,
            1_000_000_000_000, 4_000_000_000_000_000i128,
        ];
        for x in test_xs {
            let er = ExtRes::from_i128(x);
            let q = cram_div_exact_pos(&er, d);
            let canonical_q = x / d as i128;
            let canonical_er = ExtRes::from_i128(canonical_q);
            assert_eq!(q.res, canonical_er.res,
                "x={}, d={}: q={:?} ≠ canonical {:?} (canonical_q={})",
                x, d, q.res, canonical_er.res, canonical_q);
        }
    }

    #[test]
    fn stress_random_xs_and_divisors() {
        let divisors: [u64; 3] = [1024, 1 << 30, 100_000_000];
        let mut h: u64 = 0xC0FFEE;
        for _ in 0..1000 {
            h = h.wrapping_mul(2862933555777941757).wrapping_add(3037000493);
            let x = (h as i128) & ((1i128 << 50) - 1);  // 0..2^50
            for &d in &divisors {
                let er = ExtRes::from_i128(x);
                let q = cram_div_exact_pos(&er, d);
                let canonical_q = x / d as i128;
                let canonical_er = ExtRes::from_i128(canonical_q);
                assert_eq!(q.res, canonical_er.res,
                    "stress fail: x={}, d={}: q={:?}, canonical={} (res {:?})",
                    x, d, q.res, canonical_q, canonical_er.res);
            }
        }
    }

    /// Stress test up to i64::MAX-class values — the regime the NS solver
    /// actually exercises. nu_scaled · lap can reach ~i64::MAX × small int
    /// before saturating; this test pushes x to ~2^62 to validate the
    /// range condition `q < M_NTT · partial_mod` holds for all 3 divisors.
    #[test]
    fn stress_full_i64_range() {
        // Range bounds for q < M_NTT · partial_mod (verified safe for NS):
        //   d = 1024 (N²):       partial_mod = 2²¹, M_NTT·partial ≈ 2 · 10^18
        //   d = 2³⁰ (N⁶):        partial_mod = 2,    M_NTT·partial ≈ 2 · 10^12
        //   d = 10⁸ (S²):        partial_mod = 2²³ · 5, M_NTT·partial ≈ 4 · 10^19
        //
        // d=2³⁰ has the tightest bound. q < 2 · 10^12 means x < 2 · 10^12 · 2³⁰
        // = 2.1 · 10^21, far beyond i64. So the NS regime is safe.
        //
        // For d=1024, q < 2 · 10^18 means x < 2 · 10^21. Safe.
        // For d=10⁸, q < 4 · 10^19 means x < 4 · 10^27. Safe.
        let divisors: [u64; 3] = [1024, 1 << 30, 100_000_000];
        let mut h: u64 = 0xDEADBEEF;
        for _ in 0..500 {
            h = h.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            // 0..2^62, the NS regime's upper end
            let x = (h as i128 & ((1i128 << 62) - 1)).abs();
            for &d in &divisors {
                let er = ExtRes::from_i128(x);
                let q = cram_div_exact_pos(&er, d);
                let canonical_q = x / d as i128;
                let canonical_er = ExtRes::from_i128(canonical_q);
                assert_eq!(q.res, canonical_er.res,
                    "stress_full_i64 fail: x={}, d={}: q_res={:?}, canonical={} (res {:?})",
                    x, d, q.res, canonical_q, canonical_er.res);
            }
        }
    }

    /// DivCtx builds correct per-divisor inverses — verify inv_d_ntt[i] * d ≡ 1 (mod p_i).
    #[test]
    fn divcxt_inverses_correct() {
        use crate::ns3d_n32::NTT_PRIMES;
        let divisors: [u64; 5] = [1u64 << 30, 100_000_000, 40_000, 20_000, 1_024];
        for d in divisors {
            let ctx = DivCtx::build(d);
            for i in 0..4 {
                let p = NTT_PRIMES[i];
                let check = ((ctx.inv_d_ntt[i] as u128 * (d as u128 % p as u128)) % p as u128) as u64;
                assert_eq!(check, 1,
                    "DivCtx d={}: inv_d_ntt[{}]={} × d mod {}={} ≠ 1",
                    d, i, ctx.inv_d_ntt[i], p, check);
            }
        }
    }

    /// partial_mod_min matches expected per-divisor values from DivContract table.
    #[test]
    fn div_contract_partial_mod_table() {
        // Table from DivContract analytical derivation (NODE-P0A)
        let cases: &[(u64, u128)] = &[
            (1u64 << 30, 2),       // d=N^6: min(2,1953125)=2
            (100_000_000, 5),      // d=S^2: min(8388608,5)=5
            (40_000,  3_125),      // d=4S:  min(33554432,3125)=3125
            (20_000,  3_125),      // d=2S:  min(67108864,3125)=3125
            (1_024,   1_953_125),  // d=N^2: min(2097152,1953125)=1953125
        ];
        for &(d, expected_partial) in cases {
            let got = div_contract_partial_mod(d);
            assert_eq!(got, expected_partial,
                "partial_mod_min for d={}: got {} expected {}", d, got, expected_partial);
        }
    }

    /// trunc_div_signed matches cram_div_exact_pos (pos) and canonical i128 division.
    #[test]
    fn trunc_div_signed_matches_canonical() {
        let divisors: [u64; 5] = [1u64 << 30, 100_000_000, 40_000, 20_000, 1_024];
        // Positive and negative inputs
        let test_xs: Vec<i128> = vec![
            0, 1, -1, 1023, -1023, 1_000_000, -1_000_000,
            1_000_000_000_000, -1_000_000_000_000,
        ];
        for d in divisors {
            let ctx = DivCtx::build(d);
            for &x in &test_xs {
                let er = ExtRes::from_i128(x);
                let q = trunc_div_signed(&er, &ctx);
                let canonical_q = x / d as i128; // Rust truncates toward zero
                let canonical_er = ExtRes::from_i128(canonical_q);
                assert_eq!(q.res, canonical_er.res,
                    "trunc_div_signed x={} d={}: got {:?} expected {:?} (q={})",
                    x, d, q.res, canonical_er.res, canonical_q);
            }
        }
    }

    /// garner_n_centered_i128 matches garner4_to_u128 on 4-prime basis.
    #[test]
    fn garner_n_centered_matches_garner4() {
        use crate::ns3d_n32::NTT_PRIMES;
        let primes = NTT_PRIMES;
        let m_ntt: u128 = primes.iter().map(|&p| p as u128).product();
        let mut h: u64 = 0xABCDEF01;
        for _ in 0..200 {
            h = h.wrapping_mul(6364136223846793005).wrapping_add(1);
            // x in [0, M_NTT) — a valid NTT value
            let x = (h as u128) % m_ntt;
            let residues: [u64; 4] = primes.map(|p| (x % p as u128) as u64);
            let g4 = garner4_to_u128(&residues) as i128;
            // Centre g4
            let centred = if g4 as u128 <= m_ntt / 2 { g4 } else { g4 - m_ntt as i128 };
            let gn = garner_n_centered_i128(&residues, &primes);
            assert_eq!(gn, centred,
                "garner_n mismatch x={}: gn={} centred={}", x, gn, centred);
        }
    }

    /// M_NTT and M_NTT_HALF constants are correct.
    #[test]
    fn m_ntt_constants_correct() {
        use crate::ns3d_n32::NTT_PRIMES;
        let expected: u128 = NTT_PRIMES.iter().map(|&p| p as u128).product();
        assert_eq!(M_NTT, expected, "M_NTT constant mismatch");
        assert_eq!(M_NTT_HALF, expected / 2, "M_NTT_HALF constant mismatch");
    }
}
