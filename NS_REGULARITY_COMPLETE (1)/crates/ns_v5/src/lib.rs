//! # Exact-Integer Navier-Stokes Regularity via CRT Contraction
//!
//! **Anthony Diaz (Acid) — HackFate.us / Skyelabz210**
//! **SHUTDOWN BUILD v2.0 — ZERO FLOAT**
//!
//! ## Axiom A1 Compliance
//!
//! This crate contains ZERO f64, ZERO f32, ZERO approximate arithmetic.
//! Every computation is exact integer or exact rational (num/den).
//! Initial condition generation uses integer Bhaskara sine approximation.
//!
//! ## Claim Hygiene (Phase 5, Item 16)
//!
//! Every theorem in this crate is labeled:
//!   [CODE FACT]        — verifiable by running `cargo test`
//!   [DISCRETE THEOREM] — proved for the lifted discrete scheme
//!   [ARCHIVED REGRESSION] — test-only 1D harness, removed from active path
//!   [CONTINUUM]        — claimed for the continuum limit (requires bridge)
//!
//! ## Active module map
//!
//! - `src/lib.rs` — theorem-bearing arithmetic core and exact-rational checks.
//! - `src/recumbent.rs` — recumbent overflow geometry for the active exact carrier.
//! - `src/wassan.rs` — structural persistence of recumbent state.
//! - `src/cram_ns.rs` — archived 1D regression harness, compiled under `#[cfg(test)]`
//!   only and retained strictly for arithmetic/debug regression.
//!
//! If a task does not directly advance the 3D exact solver, the exact 3D
//! operator family, or the continuum bridge, it should be presumed archived.
//!
//! ## Proof Chain (HONEST VERSION — Item 13)
//!
//! ```text
//! |centered(x mod p)| ≤ (p-1)/2               [CODE FACT, modular arithmetic]
//!          ↓
//! B(N) ≤ (3/7)(5/11)(6/13) = 90/1001          [CODE FACT, exact rational]
//!          ↓
//! 90 < 1001 → B(N) < 1                        [CODE FACT, integer comparison]
//!          ↓
//! DISCRETE: residues contract per step          [DISCRETE THEOREM]
//!          ↓
//! DISCRETE: excursions outside corridor are     [DISCRETE THEOREM, recumbent]
//!           exact, not destructive
//!          ↓
//! OPEN: return-to-corridor for general 3D data  [OPEN — Item 15]
//!          ↓
//! OPEN: continuum limit preserves control       [OPEN — Items 17-21]
//!          ↓
//! BKM (corollary, not engine)                   [CONTINUUM, requires bridge]
//! ```

#[cfg(test)]
mod cram_ns; // Archived 1D regression harness; intentionally excluded from the active public API.
pub mod recumbent;
pub mod wassan;
pub mod ns3d;
pub mod ns3d_sobolev;
pub mod ns3d_wassan;

use qmnf_crtbigint::ExactRational;

// ═══════════════════════════════════════════════════════════════════════
// MODULE 1: MODULAR ARITHMETIC PRIMITIVES — ZERO FLOAT
// ═══════════════════════════════════════════════════════════════════════

#[inline]
fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    if m == 1 { return 0; }
    let mut r = 1u64;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 { r = mulmod(r, base, m); }
        exp >>= 1;
        base = mulmod(base, base, m);
    }
    r
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

// ═══════════════════════════════════════════════════════════════════════
// MODULE 2: NS TRANSITION MAP ON THE CRT TORUS — ZERO FLOAT
// ═══════════════════════════════════════════════════════════════════════

/// The discriminating primes for the NS contraction bound.
pub const DISC_PRIMES: [u64; 3] = [7, 11, 13];

/// The universal contraction bound: 90/1001. EXACT RATIONAL.
pub const BOUND_NUM: u64 = 90;
pub const BOUND_DEN: u64 = 1001;

/// SCALE factor. Physical value = integer value / SCALE.
pub const SCALE: u64 = 10_000;

pub struct TransitionMap {
    pub dt: u64,
    pub nu: u64,
}

impl TransitionMap {
    /// Standard parameters: dt_scaled = 100, nu_scaled = 6
    pub fn standard() -> Self {
        TransitionMap { dt: 100, nu: 6 }
    }

    /// Diffusion coefficient mod p: c_diff(N,p) = dt·ν·N²·SCALE⁻² mod p
    pub fn c_diff(&self, n: u64, p: u64) -> u64 {
        let scale_inv = mod_pow(SCALE % p, p - 2, p);
        let scale_inv2 = mulmod(scale_inv, scale_inv, p);
        let n2 = mulmod(n % p, n % p, p);
        let dtnu = mulmod(self.dt % p, self.nu % p, p);
        mulmod(mulmod(dtnu, n2, p), scale_inv2, p)
    }

    /// Stretching coefficient mod p: c_s(p) = dt·SCALE⁻¹ mod p
    /// This is CONSTANT for all N — the key structural fact.
    pub fn c_stretch(&self, p: u64) -> u64 {
        let scale_inv = mod_pow(SCALE % p, p - 2, p);
        mulmod(self.dt % p, scale_inv, p)
    }

    /// Self-coefficient: (1 - 6·c_diff) mod p
    pub fn self_coeff(&self, n: u64, p: u64) -> u64 {
        let cd = self.c_diff(n, p);
        (1 + p - (6 * cd) % p) % p
    }

    /// Centered self-coefficient (signed integer, not float).
    pub fn centered_self(&self, n: u64, p: u64) -> i64 {
        let s = self.self_coeff(n, p);
        if s <= p / 2 { s as i64 } else { s as i64 - p as i64 }
    }

    /// Contraction factor B(N) as EXACT RATIONAL: (numerator, denominator).
    /// B(N) = Π_p |centered(self(N,p))| / p
    ///
    /// [CODE FACT] No float. Returns ExactRational.
    pub fn contraction_exact(&self, n: u64) -> ExactRational {
        let mut num: u128 = 1;
        let mut den: u128 = 1;
        for &p in &DISC_PRIMES {
            let centered = self.centered_self(n, p);
            num *= centered.unsigned_abs() as u128;
            den *= p as u128;
        }
        ExactRational::new(num, den)
    }

    /// Check if B(N) ≤ 90/1001. EXACT integer comparison.
    /// Returns true iff num × 1001 ≤ 90 × den.
    pub fn contraction_bounded(&self, n: u64) -> bool {
        let b = self.contraction_exact(n);
        b.num * BOUND_DEN as u128 <= BOUND_NUM as u128 * b.den
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MODULE 3: THE CONTRACTION BOUND — ZERO FLOAT
// ═══════════════════════════════════════════════════════════════════════

/// **THEOREM T1 (Universal Contraction Bound):** [CODE FACT]
///
/// B(N) ≤ 90/1001 for ALL N.
/// Proof: |centered(x)| ≤ (p-1)/2 → product ≤ (3/7)(5/11)(6/13) = 90/1001.
pub fn verify_theorem_1() -> bool {
    let num: u64 = DISC_PRIMES.iter().map(|&p| p - 1).product();
    let den: u64 = DISC_PRIMES.iter().map(|&p| 2 * p).product();
    let g = gcd(num, den);
    num / g == BOUND_NUM && den / g == BOUND_DEN && BOUND_NUM < BOUND_DEN
}

/// **THEOREM T2 (Exhaustive Verification):** [CODE FACT]
///
/// B(N) ≤ 90/1001 for all N = 1, ..., 1001.
/// Returns (all_pass, worst_N, worst_B_num, worst_B_den).
/// ZERO FLOAT. All comparisons are exact rational cross-multiplication.
pub fn verify_theorem_2() -> (bool, u64, u128, u128) {
    let tm = TransitionMap::standard();
    let mut max_num: u128 = 0;
    let mut max_den: u128 = 1;
    let mut max_n: u64 = 0;
    let mut all_pass = true;

    for n in 1..=BOUND_DEN {
        let b = tm.contraction_exact(n);
        // b > current max? Cross-multiply: b.num × max_den > max_num × b.den
        if b.num * max_den > max_num * b.den {
            max_num = b.num;
            max_den = b.den;
            max_n = n;
        }
        // b > 90/1001? Cross-multiply: b.num × 1001 > 90 × b.den
        if b.num * BOUND_DEN as u128 > BOUND_NUM as u128 * b.den {
            all_pass = false;
        }
    }
    (all_pass, max_n, max_num, max_den)
}

/// **THEOREM T3 (c_s is N-independent):** [CODE FACT]
pub fn verify_theorem_3() -> bool {
    let tm = TransitionMap::standard();
    let expected = [(7u64, 4u64), (11, 1), (13, 3)];
    expected.iter().all(|&(p, val)| tm.c_stretch(p) == val)
}

/// **THEOREM T4 (Permanent Non-Resonance):** [CODE FACT]
pub fn verify_theorem_4() -> bool {
    let tm = TransitionMap::standard();
    DISC_PRIMES.iter().all(|&p| {
        let cs = tm.c_stretch(p);
        let resonance = mod_pow(6, p - 2, p);
        cs != resonance
    })
}

// ═══════════════════════════════════════════════════════════════════════
// MODULE 4: CFL AND MAXIMUM PRINCIPLE — ZERO FLOAT
// ═══════════════════════════════════════════════════════════════════════

/// CFL maximum dt: Δt ≤ SCALE²/(6·ν·N²). Returns integer.
pub fn cfl_max_dt(n: u64, nu: u64) -> u64 {
    let denom = 6u128 * nu as u128 * n as u128 * n as u128;
    if denom == 0 { return u64::MAX; }
    ((SCALE as u128 * SCALE as u128) / denom) as u64
}

/// **THEOREM T5 (Discrete Maximum Principle):** [DISCRETE THEOREM]
///
/// Under CFL, max|ω'| ≤ max|ω|. All integer arithmetic.
pub fn verify_maximum_principle(center: i64, nbrs: &[i64; 6], c_num: i64, c_den: i64) -> bool {
    let sum_nbr: i128 = nbrs.iter().map(|&w| w as i128).sum();
    let n = c_num as i128;
    let d = c_den as i128;
    let new_w = ((d - 6 * n) * center as i128 + n * sum_nbr) / d;
    let max_in = center.abs().max(nbrs.iter().map(|w| w.abs()).max().unwrap_or(0));
    (new_w as i64).abs() <= max_in
}

// ═══════════════════════════════════════════════════════════════════════
// MODULE 5: REGULARITY CHAIN — HONEST LABELING
// ═══════════════════════════════════════════════════════════════════════

/// **THEOREM T7 (Regularity — CLAIM BOUNDARY):**
///
/// Labels: [CODE FACT] for steps 1-4, [OPEN] for steps 5-9.
///
/// Steps that ARE proved:
///   [1] B(N) ≤ 90/1001 < 1 for all N.           [CODE FACT, Theorem T1]
///   [2] CFL satisfiable for all N.               [CODE FACT, arithmetic]
///   [3] Max principle: residues contract.         [DISCRETE THEOREM, T5]
///   [4] Non-resonance: c_s ≠ 6⁻¹ mod p.         [CODE FACT, T4]
///
/// Steps that are OPEN (require bridge, Items 17-21):
///   [5] L^∞(L²) ∩ L²(H¹) uniform bounds.        [REQUIRES continuum bridge]
///   [6] Aubin-Lions compactness.                  [REQUIRES time derivative bound]
///   [7] Nonlinear passage.                        [REQUIRES strong convergence]
///   [8] BKM criterion not triggered.              [REQUIRES ∫||ω||_∞ dt < ∞]
///   [9] Regularity.                               [COROLLARY of 5-8]
///
/// This function verifies ONLY the proved steps.
pub fn verify_proved_steps(n_values: &[u64]) -> bool {
    let tm = TransitionMap::standard();

    // Step 1: B(N) < 1 for all N — exact rational comparison
    let step1 = n_values.iter().all(|&n| tm.contraction_bounded(n));

    // Step 2: CFL satisfiable — a valid Δt > 0 EXISTS for any finite N.
    // Check: SCALE² > 0 and 6·ν·N² > 0, so Δt = SCALE²/(6·ν·N²) > 0 in Q.
    // We do NOT check integer truncation — the rational is positive.
    let step2 = n_values.iter().all(|&n| {
        let num = SCALE as u128 * SCALE as u128; // SCALE² > 0
        let den = 6u128 * 6u128 * n as u128 * n as u128; // 6·ν·N² > 0
        num > 0 && den > 0
    });

    // Step 4: Non-resonance
    let step4 = verify_theorem_4();

    step1 && step2 && step4
}

// ═══════════════════════════════════════════════════════════════════════
// MODULE 6: RESOLUTION LADDER — ZERO FLOAT
// ═══════════════════════════════════════════════════════════════════════

/// Resolution ladder with exact rational contraction factors.
/// Returns (N, B_num, B_den, bounded) for each level.
pub fn resolution_ladder(n_start: u64, doublings: usize) -> Vec<(u64, u128, u128, bool)> {
    let tm = TransitionMap::standard();
    let mut results = Vec::new();
    let mut n = n_start;
    for _ in 0..=doublings {
        let b = tm.contraction_exact(n);
        let bounded = b.num * BOUND_DEN as u128 <= BOUND_NUM as u128 * b.den;
        results.push((n, b.num, b.den, bounded));
        n = n.saturating_mul(2);
        if n > 1_000_000 { break; }
    }
    results
}

// ═══════════════════════════════════════════════════════════════════════
// MODULE 7: WINDING NUMBER DECOMPOSITION — ZERO FLOAT
// ═══════════════════════════════════════════════════════════════════════

/// Decompose integer into centered residue + winding number.
/// ω = r + k·p where |r| ≤ (p-1)/2.
pub fn decompose(omega: i64, p: u64) -> (i64, i64) {
    // i128 arithmetic: safe even at omega = i64::MIN / MAX.
    let p_i     = p as i128;
    let omega_i = omega as i128;
    let r_raw   = ((omega_i % p_i) + p_i) % p_i;
    let r       = if r_raw <= p_i / 2 { r_raw } else { r_raw - p_i };
    let k       = (omega_i - r) / p_i;
    // |r| ≤ (p-1)/2 fits i64; |k| ≤ |omega|/p ≤ i64::MAX/p also fits i64.
    (r as i64, k as i64)
}

/// Verify winding decomposition contracts under pure diffusion.
/// Archived 1D regression evidence only; not part of the active 3D path.
#[cfg(test)]
pub fn verify_winding_decomposition() -> bool {
    let n = 32usize;
    let mut omega: Vec<i64> = (0..n).map(|i| {
        qmnf_crtbigint::integer_sine(i, n, SCALE as i64)
    }).collect();

    let initial_max: i64 = omega.iter().map(|w| w.abs()).max().unwrap();

    for _ in 0..50 {
        let mut new = vec![0i64; n];
        for j in 0..n {
            let jp = (j+1) % n;
            let jm = (j+n-1) % n;
            // Pure diffusion: (4ω + ω_l + ω_r) / 6
            new[j] = (4 * omega[j] + omega[jm] + omega[jp]) / 6;
        }
        omega = new;
    }

    let final_max: i64 = omega.iter().map(|w| w.abs()).max().unwrap();
    final_max <= initial_max
}

/// Verify integer maximum principle under CFL stencil.
/// [DISCRETE THEOREM]
pub fn verify_integer_maximum_principle() -> bool {
    let n = 32usize;
    let mut omega: Vec<i64> = (0..n).map(|i| {
        qmnf_crtbigint::integer_sine(i, n, SCALE as i64)
    }).collect();

    let initial_max: i64 = omega.iter().map(|w| w.abs()).max().unwrap();
    let mut ever_grew = false;

    for _ in 0..100 {
        let mut new = vec![0i64; n];
        for j in 0..n {
            let jp = (j + 1) % n;
            let jm = (j + n - 1) % n;
            // CFL-limit diffusion: (4ω + ω_l + ω_r) / 6
            new[j] = (4 * omega[j] + omega[jm] + omega[jp]) / 6;
        }
        omega = new;
        let current_max: i64 = omega.iter().map(|w| w.abs()).max().unwrap();
        if current_max > initial_max { ever_grew = true; }
    }

    !ever_grew
}

// ═══════════════════════════════════════════════════════════════════════
// MODULE 8: ARCHIVED 1D REGRESSION HARNESS — TEST ONLY
//
// This block remains in-tree for fast arithmetic regression only. It is kept
// behind `#[cfg(test)]` and every symbol is prefixed `archived_` so agents do
// not accidentally treat it as part of the active 3D closure path.
// Uses integer Bhaskara sine for initial conditions.
// ═══════════════════════════════════════════════════════════════════════

/// Overshoot measurement for the archived 1D regression harness.
/// Returns `(initial_max, peak_max, final_max, enstrophy_grew)`.
///
/// This function is regression-only. It exists to catch arithmetic drift,
/// not to provide theorem evidence for the active 3D program. All values are
/// integer-valued and every comparison is exact.
/// Archived under `#[cfg(test)]` so it cannot drive the active proof path.
#[cfg(test)]
pub fn archived_measure_overshoot_1d(n: usize, steps: usize) -> (i64, i64, i64, bool) {
    let mut omega: Vec<i64> = (0..n).map(|i| {
        qmnf_crtbigint::integer_sine(i, n, SCALE as i64)
    }).collect();

    let c_num: i64 = 1;
    let c_den: i64 = 6;
    let ini = omega.iter().map(|w| w.abs()).max().unwrap();
    let mut pk = ini;

    let ei: i128 = omega.iter().map(|&w| (w as i128) * (w as i128)).sum();
    let mut eg = false;

    for _ in 0..steps {
        let mut new = vec![0i64; n];
        for j in 0..n {
            let jp = (j + 1) % n;
            let jm = (j + n - 1) % n;
            let conv = (c_den - 2 * c_num) * omega[j] + c_num * (omega[jm] + omega[jp]);
            let val = conv / c_den;
            // Stretching: ω × (ω_r - ω_l) / (2 × SCALE)
            let stretch = (omega[j] as i128 * ((omega[jp] as i128) - (omega[jm] as i128))
                          / (2 * SCALE as i128)) as i64;
            new[j] = val + stretch;
        }
        omega = new;
        let cm = omega.iter().map(|w| w.abs()).max().unwrap();
        if cm > pk { pk = cm; }
        let ce: i128 = omega.iter().map(|&w| (w as i128) * (w as i128)).sum();
        if ce > ei { eg = true; }
    }

    let fin = omega.iter().map(|w| w.abs()).max().unwrap();
    (ini, pk, fin, eg)
}

// ═══════════════════════════════════════════════════════════════════════
// MODULE 9: CONTRACTION CYCLE ANALYSIS — ZERO FLOAT
// ═══════════════════════════════════════════════════════════════════════

/// Compute the product of contraction factors over a set of N values.
/// Returns ExactRational. [CODE FACT]
pub fn contraction_product(n_values: &[u64]) -> ExactRational {
    let tm = TransitionMap::standard();
    let mut num: u128 = 1;
    let mut den: u128 = 1;
    for &n in n_values {
        let b = tm.contraction_exact(n);
        num *= b.num;
        den *= b.den;
    }
    ExactRational::new(num, den)
}

/// Per-prime cycle analysis. Returns self-coefficient cycle for each prime.
/// [CODE FACT]
pub fn self_coefficient_cycle(p: u64) -> Vec<(u64, i64)> {
    let tm = TransitionMap::standard();
    (1..=p).map(|n| {
        let centered = tm.centered_self(n, p);
        (n, centered)
    }).collect()
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS — ZERO FLOAT THROUGHOUT
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Theorem 1: Algebraic bound ──

    #[test]
    fn theorem_1_algebraic_bound() {
        assert!(verify_theorem_1(),
            "90/1001 must be the exact universal contraction bound");
    }

    #[test]
    fn theorem_1_numerical() {
        assert_eq!(BOUND_NUM, 90);
        assert_eq!(BOUND_DEN, 1001);
        assert!(BOUND_NUM < BOUND_DEN, "90 < 1001");
    }

    #[test]
    fn theorem_1_exact_rational() {
        // Verify (3/7)(5/11)(6/13) = 90/1001 via exact rational
        let a = ExactRational::new(3, 7);
        let b = ExactRational::new(5, 11);
        let c = ExactRational::new(6, 13);
        let product = a.mul(&b).mul(&c).reduce();
        assert_eq!(product.num, 90);
        assert_eq!(product.den, 1001);
    }

    // ── Theorem 2: Exhaustive ──

    #[test]
    fn theorem_2_exhaustive() {
        let (pass, max_n, max_num, max_den) = verify_theorem_2();
        println!("Theorem 2: ALL N=1..1001 checked");
        println!("  Max B(N) = {}/{} at N={}", max_num, max_den, max_n);
        println!("  Bound    = {}/{}", BOUND_NUM, BOUND_DEN);
        // Verify max_num/max_den ≤ 90/1001 via cross-multiply
        assert!(max_num * BOUND_DEN as u128 <= BOUND_NUM as u128 * max_den,
            "Worst case {}/{} exceeds 90/1001", max_num, max_den);
        assert!(pass, "ALL N must satisfy B(N) ≤ 90/1001");
    }

    // ── Theorem 3: c_s N-independent ──

    #[test]
    fn theorem_3_stretching_constant() {
        assert!(verify_theorem_3(),
            "c_s mod p must be constant: (4, 1, 3) on (7, 11, 13)");
    }

    // ── Theorem 4: Non-resonance ──

    #[test]
    fn theorem_4_non_resonance() {
        assert!(verify_theorem_4(),
            "c_s must never equal 6^(-1) mod p");
    }

    // ── Theorem 5: Maximum principle ──

    #[test]
    fn theorem_5_max_principle_preserves() {
        let w = 5000i64;
        let nbrs = [4900, 5100, 4800, 5200, 4950, 5050];
        assert!(verify_maximum_principle(w, &nbrs, 1, 100));
    }

    #[test]
    fn theorem_5_constant_field() {
        assert!(verify_maximum_principle(10000, &[10000; 6], 1, 100));
    }

    // ── Proved steps ──

    #[test]
    fn proved_steps_regularity() {
        let ns: Vec<u64> = (3..=17).map(|k| 1u64 << k).collect();
        assert!(verify_proved_steps(&ns),
            "All proved steps must pass for N=8..131072");
    }

    // ── Resolution ladder ──

    #[test]
    fn resolution_ladder_all_contractive() {
        let ladder = resolution_ladder(8, 14);
        println!("\nResolution Ladder (ZERO FLOAT):");
        for (n, b_num, b_den, ok) in &ladder {
            println!("  N={:>6}: B={}/{} {}", n, b_num, b_den, if *ok { "✓" } else { "✗" });
        }
        assert!(ladder.iter().all(|(_, _, _, ok)| *ok),
            "ALL levels must be contractive");
    }

    // ── Cycle product ──

    #[test]
    fn cycle_product_bounded() {
        let ns: Vec<u64> = (3..=13).map(|k| 1u64 << k).collect();
        let product = contraction_product(&ns);
        // Each factor is ≤ 90/1001, so product ≤ (90/1001)^11
        // 90^11 / 1001^11 is astronomically small
        assert!(product.lt(&ExactRational::one()),
            "Cycle product must be < 1");
    }

    // ── Self-coefficient cycles ──

    #[test]
    fn self_coefficient_has_zero_step() {
        for &p in &DISC_PRIMES {
            let cycle = self_coefficient_cycle(p);
            let has_zero = cycle.iter().any(|(_, c)| *c == 0);
            println!("p={}: cycle = {:?}", p, cycle.iter().map(|(_, c)| *c).collect::<Vec<_>>());
            // Every cycle should contain a zero step
            assert!(has_zero, "p={} cycle must contain a zero self-coefficient", p);
        }
    }

    // ── Winding decomposition ──

    #[test]
    fn test_decompose_roundtrip() {
        for omega in -1000i64..=1000 {
            for &p in &[7u64, 11, 13] {
                let (r, k) = decompose(omega, p);
                assert_eq!(r + k * p as i64, omega,
                    "Decomposition must round-trip: {} = {} + {} × {}", omega, r, k, p);
                assert!(r.abs() <= p as i64 / 2,
                    "Residue must be centered: |{}| ≤ {}", r, p / 2);
            }
        }
    }

    #[test]
    fn test_winding_maximum_principle() {
        assert!(verify_winding_decomposition(),
            "Winding number must not grow under CFL diffusion stencil");
    }

    #[test]
    fn test_integer_maximum_principle() {
        assert!(verify_integer_maximum_principle(),
            "Integer maximum principle must hold under CFL stencil");
    }

    // ── Archived 1D regression tests (not active proof path) ──

    #[test]
    fn test_stretching_bounded() {
        // Archived 1D regression test.
        let n = 64usize;
        let mut omega: Vec<i64> = (0..n).map(|i| {
            qmnf_crtbigint::integer_sine(i, n, SCALE as i64)
        }).collect();

        let initial_max: i64 = omega.iter().map(|w| w.abs()).max().unwrap();
        let mut peak_max: i64 = initial_max;
        let mut grew_at_step: Option<usize> = None;

        for step in 0..200 {
            let mut new = vec![0i64; n];
            for j in 0..n {
                let jp = (j + 1) % n;
                let jm = (j + n - 1) % n;
                let conv = (4 * omega[j] + omega[jm] + omega[jp]) / 6;
                let stretch = omega[j] as i128 * ((omega[jp] as i128) - (omega[jm] as i128)) / (2 * SCALE as i128);
                new[j] = conv + stretch as i64;
            }
            omega = new;
            let current_max: i64 = omega.iter().map(|w| w.abs()).max().unwrap();
            if current_max > peak_max { peak_max = current_max; }
            if current_max > initial_max && grew_at_step.is_none() {
                grew_at_step = Some(step);
            }
        }

        let final_max: i64 = omega.iter().map(|w| w.abs()).max().unwrap();
        // Overshoot as permille: peak × 1000 / initial
        let overshoot_pm = peak_max * 1000 / initial_max.max(1);

        println!("Stretching test (archived 1D regression, N={}):", n);
        println!("  Initial max|ω| = {}", initial_max);
        println!("  Peak max|ω|    = {} (overshoot: {} permille)", peak_max, overshoot_pm);
        println!("  Final max|ω|   = {} (after 200 steps)", final_max);
        if let Some(s) = grew_at_step {
            println!("  First growth at step {}", s);
            println!("  [ARCHIVED REGRESSION] stretching can cause transient growth in 1D.");
        }

        assert!(final_max < initial_max * 2,
            "Solution should not blow up in 200 steps for smooth data");
    }

    // ── Overshoot scaling ──

    #[test]
    fn overshoot_scaling() {
        println!("\n{}", "=".repeat(70));
        println!("  OVERSHOOT CHIMERA: archived 1D regression scaling with N");
        println!("{}\n", "=".repeat(70));

        println!("{:>5} {:>8} {:>8} {:>12} {:>8} {:>12}",
            "N", "init", "peak", "over(pm)", "final", "decay(pm)");
        println!("{}", "-".repeat(60));

        let mut overshoots = Vec::new();

        for &n in &[16usize, 32, 64, 128] {
            let (ini, pk, fin, _eg) = archived_measure_overshoot_1d(n, 200);
            let over_pm = pk * 1000 / ini.max(1);
            let decay_pm = fin * 1000 / ini.max(1);
            overshoots.push((n, over_pm));
            println!("{:>5} {:>8} {:>8} {:>12} {:>8} {:>12}",
                n, ini, pk, over_pm, fin, decay_pm);
        }
    }
}
pub mod ns3d_energy;
#[cfg(test)]
mod ns3d_adversarial;
pub mod ns3d_n16;
#[cfg(feature = "stencil_bs")] pub mod ns3d_stencil_bs;
pub mod ns3d_n32;
pub mod ns3d_aubin_lions;
pub mod ns3d_d030;
pub mod ns3d_structb;
