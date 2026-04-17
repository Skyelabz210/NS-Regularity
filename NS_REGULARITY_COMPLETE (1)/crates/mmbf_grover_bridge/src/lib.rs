// ═══════════════════════════════════════════════════════════════════════════════
// MMBF × DENSE TORIC GROVER: BRIDGE ARCHITECTURE
// ═══════════════════════════════════════════════════════════════════════════════
//
// The Ramanujan-Hydra Spectral Correspondence (T6) provides the ORACLE.
// Dense Toric Grover provides the SEARCH ENGINE.
// K-Elimination provides EXACT RECONSTRUCTION.
// The CRT torus is the SHARED SUBSTRATE.
//
// ─────────────────────────────────────────────────────────────────────────────
// SECTION 1: THE VARIABLE DICTIONARY
// ─────────────────────────────────────────────────────────────────────────────
//
// MMBF Component          │ Dense Toric Grover Component     │ Connection
// ─────────────────────────┼──────────────────────────────────┼────────────
// CRT torus T(P₆)         │ Torus T² = ℤ_M × ℤ_A           │ SAME OBJECT
// Sieve power SP(p,d)     │ Oracle weight per residue class  │ T6-1 IS the oracle
// Forbidden class {0,-d}  │ Marked states (Grover target)    │ Direct identification
// Twin candidates          │ Unmarked states (search result)  │ Grover amplifies these
// Ramanujan sum c_p(d)    │ Phase contribution per lane      │ Spectral basis
// K-Elimination κ         │ Helix level / winding number     │ SAME OPERATION
// Normalized sep σ_p(d)   │ Per-lane oracle weight           │ T6-2 gives distribution
// Interaction modes        │ Multi-qubit entanglement         │ Cross-lane coupling
//
// ─────────────────────────────────────────────────────────────────────────────
// SECTION 2: THE ORACLE FROM T6-1
// ─────────────────────────────────────────────────────────────────────────────
//
// Grover needs an oracle O that marks "bad" states (those to avoid) or
// "good" states (those to find). In the MMBF twin-prime sieve:
//
//   For each prime p in the Safe Basis {3, 5, 7, 11, 13}:
//     FORBIDDEN residues for gap d: {0, -d mod p}
//     Number of forbidden: F(p,d) = 2 - δ(p,d)     [Lemma R1]
//     Sieve power:         SP(p,d) = F(p,d)/p       [Definition R1]
//     Via Ramanujan:        SP(p,d) = (2p-1-c_p(d))/p²  [T6-1]
//
// The oracle is SEPARABLE across CRT lanes:
//
//   O(n) = 1  iff  ∃ p ∈ P₆: n ≡ 0 (mod p) OR n ≡ -d (mod p)
//   O(n) = 0  iff  n survives all sieve primes (twin candidate)
//
// In Grover language: we MARK the forbidden states (O(n)=1) and
// AMPLIFY the surviving states (O(n)=0). After O(√(N/k)) iterations
// where k = number of survivors, we measure a twin candidate.
//
// For the S₆ torus with d=2:
//   N = 30030 (total residues)
//   k = 1485  (twin candidates, from harness)
//   Optimal iterations = ⌊π/4 · √(30030/1485)⌋ = ⌊π/4 · √20.22⌋
//                       = ⌊π/4 · 4.497⌋ = ⌊3.535⌋ = 3
//
// This is EXACTLY the iteration count where Dense Toric peaks for similar
// ratios — the 96.13% result at iteration 3 for N=16 (k=1) scales to
// ~96% at iteration 3-4 for N=30030 (k=1485).
//
// ─────────────────────────────────────────────────────────────────────────────
// SECTION 3: THE ADMISSIBILITY CONSTRAINT
// ─────────────────────────────────────────────────────────────────────────────
//
// Dense Toric Grover requires F_p² arithmetic, which needs p ≡ 3 (mod 4)
// so that i² = -1 has a solution in F_p.
//
// The Admissibility Alternation Theorem (from MMBF audit):
//   For prime p ≡ 3 (mod 4): p^k ≡ 3 (mod 4) iff k is odd
//
// Safe Basis admissibility for F_p² Grover:
//   p=2:  2 ≡ 2 (mod 4) → INADMISSIBLE (but p=2 is special: F_4 = F_{2²} works)
//   p=3:  3 ≡ 3 (mod 4) → ADMISSIBLE ✓
//   p=5:  5 ≡ 1 (mod 4) → INADMISSIBLE
//   p=7:  7 ≡ 3 (mod 4) → ADMISSIBLE ✓
//   p=11: 11 ≡ 3 (mod 4) → ADMISSIBLE ✓
//   p=13: 13 ≡ 1 (mod 4) → INADMISSIBLE
//
// For CRT lanes using inadmissible primes (5, 13), use the classical
// sieve (no Grover acceleration). For admissible primes (3, 7, 11),
// deploy Dense Toric Grover per-lane.
//
// ARCHITECTURAL CONSEQUENCE: Two-tier oracle
//   Tier 1 (Classical): p ∈ {2, 5, 13} — direct residue check
//   Tier 2 (Grover):    p ∈ {3, 7, 11}  — amplitude amplification
//
// ─────────────────────────────────────────────────────────────────────────────
// SECTION 4: ITERATION COUNT FROM RAMANUJAN SPECTRUM
// ─────────────────────────────────────────────────────────────────────────────
//
// For a single CRT lane with prime p and gap d:
//   Search space:  N_lane = p
//   Survivors:     k_lane = p - F(p,d) = p - 2 + δ(p,d)
//   Marked:        F(p,d) = 2 - δ(p,d)
//
// Grover optimal iterations per lane:
//   T_opt(p,d) = ⌊π/4 · √(p / F(p,d))⌋
//
// For p ∤ d (F=2):  T_opt = ⌊π/4 · √(p/2)⌋
// For p | d (F=1):  T_opt = ⌊π/4 · √p⌋
//
// Via T6-1, expressing in terms of Ramanujan sums:
//   F(p,d) = (2p - 1 - c_p(d)) / p · p = p · SP(p,d)
//
// So: T_opt(p,d) = ⌊π/4 · √(1 / SP(p,d))⌋
//
// THE RAMANUJAN-GROVER FORMULA:
//   T_opt(p,d) = ⌊π/4 · p / √(2p - 1 - c_p(d))⌋
//
// This is a CLOSED-FORM iteration count from Ramanujan sums.
// No empirical search needed — T6-1 predicts the optimal Grover depth.
//
// ─────────────────────────────────────────────────────────────────────────────
// SECTION 5: THE FULL PIPELINE
// ─────────────────────────────────────────────────────────────────────────────
//
// INPUT:  Target gap d (e.g., d=2 for twin primes)
// OUTPUT: Twin prime candidate n in [1, N₆]
//
// STEP 1: SPECTRAL ANALYSIS (T6-1)
//   For each p in Safe Basis:
//     Compute c_p(d) = ramanujan_sum(p, d)
//     Compute SP(p,d) = (2p - 1 - c_p(d)) / p²
//     Compute T_opt(p,d) = ⌊π/4 · p / √(2p - 1 - c_p(d))⌋
//     Classify: admissible (p ≡ 3 mod 4) or classical
//
// STEP 2: PER-LANE SEARCH
//   For admissible p ∈ {3, 7, 11}:
//     Deploy Dense Toric Grover on ℤ/pℤ
//     Oracle: mark residues {0, -d mod p}
//     Run T_opt(p,d) iterations
//     Measure: obtain surviving residue r_p
//
//   For classical p ∈ {2, 5, 13}:
//     Direct enumeration of surviving residues
//     Select r_p uniformly from survivors
//
// STEP 3: CRT RECONSTRUCTION
//   Combine {r_p : p ∈ P₆} via Chinese Remainder Theorem
//   Obtain n ∈ [1, N₆] with n ≡ r_p (mod p) for all p
//   n is a twin prime candidate: gcd(n, N₆) = 1 AND gcd(n+d, N₆) = 1
//
// STEP 4: K-ELIMINATION VERIFICATION
//   For candidate n, verify via K-Elimination:
//     κ = (s - r) · M⁻¹ (mod A)
//     Confirms n is in the rail structure (not void)
//
// ─────────────────────────────────────────────────────────────────────────────
// SECTION 6: WHAT THIS GAINS VS CLASSICAL
// ─────────────────────────────────────────────────────────────────────────────
//
// HONEST ASSESSMENT:
//
// For the S₆ basis (primes up to 13), the search spaces are TINY:
//   p=3:  3 residues, F=2 → 1 survivor. No search needed.
//   p=5:  5 residues, F=2 → 3 survivors. Trivial.
//   p=7:  7 residues, F=2 → 5 survivors. Trivial.
//   p=11: 11 residues, F=2 → 9 survivors. Trivial.
//   p=13: 13 residues, F=2 → 11 survivors. Trivial.
//
// Grover acceleration is UNNECESSARY for small primes. Classical
// enumeration is O(p) which is O(1) for fixed basis primes.
//
// WHERE GROVER MATTERS: Extended bases and large prime lanes.
//
// For the Infinite Scaffold at depth n, connector primes p_c ~ 10^4:
//   p_c = 10007: search space = 10007 residues
//   F(p_c, d) = 2 for most d
//   Classical: O(10007) per lane
//   Grover:    O(√5003) ≈ O(71) per lane → 141× speedup
//
// For WASSAN moduli p_min > 2√N + 1:
//   At N = 10^6: p_min ≈ 2003, Grover gives √1001 ≈ 32× speedup
//   At N = 10^12: p_min ≈ 2·10^6, Grover gives √10^6 = 1000× speedup
//   At N = 10^18: p_min ≈ 2·10^9, Grover gives √10^9 ≈ 31623× speedup
//
// The Ramanujan-Grover formula predicts ALL of these iteration counts
// from c_p(d) alone. No empirical calibration needed.
//
// ─────────────────────────────────────────────────────────────────────────────
// SECTION 7: THE INTERACTION MODE CONNECTION
// ─────────────────────────────────────────────────────────────────────────────
//
// The harness showed: marginal = 61.6%, interaction = 38.4% (pairwise).
// The full Radon decomposition claims: marginal = 1.2%, interaction = 98.8%.
//
// In Grover terms:
//   Marginal = single-lane oracle queries (classical sieve per prime)
//   Interaction = cross-lane entanglement (multi-qubit diffusion)
//
// Dense Toric Grover on the FULL torus (not per-lane) would exploit
// cross-lane correlations directly. The diffusion operator on
// T(P₆) = ℤ/3 × ℤ/5 × ℤ/7 × ℤ/11 × ℤ/13 couples ALL lanes
// simultaneously, accessing the interaction reservoir.
//
// Per-lane Grover: accesses only marginal power (61.6%)
// Full-torus Grover: accesses marginal + interaction (100%)
//
// The 2.5× gain formula (simplified) maps to:
//   Gain = (marginal + interaction) / marginal
//        = 1 / marginal_fraction
//        ≈ 1 / 0.616 ≈ 1.62× (pairwise model)
//   or ≈ 1 / 0.012 ≈ 83× (Radon model, if validated)
//
// This is the theoretical maximum speedup from deploying full-torus
// Grover vs per-lane Grover. The Ramanujan spectral structure
// determines which interactions are accessible.
//
// ═══════════════════════════════════════════════════════════════════════════════

use std::collections::HashSet;

/// Ramanujan sum for prime p (exact integer)
fn ramanujan_sum(p: u64, d: u64) -> i64 {
    if d % p == 0 { p as i64 - 1 } else { -1 }
}

/// Sieve power SP(p,d) as rational (num, den)
fn sieve_power(p: u64, d: u64) -> (i64, u64) {
    let c = ramanujan_sum(p, d);
    let num = 2 * p as i64 - 1 - c;
    let den = p * p;
    (num, den)
}

/// Forbidden count F(p,d)
fn forbidden_count(p: u64, d: u64) -> u64 {
    if d % p == 0 { 1 } else { 2 }
}

/// Optimal Grover iterations for a single CRT lane
/// Paper formula: T_opt = ⌊π/4 · p / √(2p - 1 - c_p(d))⌋
///
/// Integer-only computation using scaling factor S = 10^6:
///   numerator = (π/4 · S) · p = 785398 · p
///   denominator = isqrt(2p - 1 - c_p(d)) · S (via scaled sqrt)
///   T_opt = numerator / denominator
fn optimal_iterations_lane(p: u64, d: u64) -> u64 {
    let c = ramanujan_sum(p, d);
    let denom_sq = (2 * p as i64 - 1 - c) as u64; // 2p - 1 - c_p(d)
    if denom_sq == 0 { return 0; }

    // Scale p by 10^12 before sqrt to maintain precision
    // √(denom_sq) with scaling: isqrt(denom_sq * 10^12) = √denom_sq * 10^6
    let scaled_denom_sq = denom_sq as u128 * 1_000_000_000_000u128;
    let sqrt_denom_scaled = integer_sqrt_128(scaled_denom_sq); // ≈ √denom_sq · 10^6

    if sqrt_denom_scaled == 0 { return 0; }

    // T = π/4 · p / √(2p-1-c) = (785398 · p) / sqrt_denom_scaled
    let numerator = 785398u128 * p as u128;
    (numerator / sqrt_denom_scaled) as u64
}

/// Integer square root for u64
fn integer_sqrt(n: u64) -> u64 {
    if n == 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Integer square root for u128 (needed for scaled computations)
fn integer_sqrt_128(n: u128) -> u128 {
    if n == 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Check if prime is admissible for F_p² (p ≡ 3 mod 4)
fn is_fp2_admissible(p: u64) -> bool {
    p % 4 == 3
}

/// Full MMBF-Grover pipeline for gap d on Safe Basis
fn mmbf_grover_analysis(safe_basis: &[u64], d: u64) {
    println!("═══════════════════════════════════════════════════════");
    println!("MMBF × DENSE TORIC GROVER — Gap d = {}", d);
    println!("═══════════════════════════════════════════════════════");
    println!();

    let mut total_survivors: u64 = 1;
    let mut total_space: u64 = 1;

    for &p in safe_basis {
        if p == 2 {
            // p=2 special case
            let f = forbidden_count(2, d);
            let surv = 2 - f;
            println!("  p={:>2}: F={}, survivors={}, SPECIAL (F_4)", p, f, surv);
            total_survivors *= surv;
            total_space *= p;
            continue;
        }

        let c = ramanujan_sum(p, d);
        let (sp_num, sp_den) = sieve_power(p, d);
        let f = forbidden_count(p, d);
        let surv = p - f;
        let t_opt = optimal_iterations_lane(p, d);
        let admissible = is_fp2_admissible(p);

        println!("  p={:>2}: c_p({})={:>3}, SP={}/{}, F={}, surv={}, T_opt={}, {}",
            p, d, c, sp_num, sp_den, f, surv, t_opt,
            if admissible { "GROVER ✓" } else { "CLASSICAL" });

        total_survivors *= surv;
        total_space *= p;
    }

    println!();
    println!("  Total search space (N₆): {}", total_space);
    println!("  Total twin candidates:   {}", total_survivors);
    println!("  Twin fraction:           {}/{}", total_survivors, total_space);

    // Full-torus Grover iteration count
    let full_ratio = total_space * 10000 / total_survivors;
    let full_sqrt = integer_sqrt(full_ratio);
    let full_t_opt = 7854 * full_sqrt / 10000;
    println!("  Full-torus T_opt:        {} iterations", full_t_opt);

    // Grover vs classical speedup for extended primes
    println!();
    println!("  Extended basis speedups:");
    for &p_ext in &[1009u64, 10007, 100003, 1000003] {
        let f = forbidden_count(p_ext, d);
        let classical = p_ext;
        let grover = integer_sqrt(p_ext * 10000 / f) * 7854 / 10000;
        let speedup = if grover > 0 { classical / grover } else { 0 };
        let admissible = is_fp2_admissible(p_ext);
        println!("    p={:>8}: classical O({}), Grover O({}), speedup {}×{}",
            p_ext, classical, grover, speedup,
            if admissible { " [F_p² ✓]" } else { "" });
    }
}

fn main() {
    let safe_basis = vec![2u64, 3, 5, 7, 11, 13];

    // Twin primes (d=2)
    mmbf_grover_analysis(&safe_basis, 2);
    println!();

    // Cousin primes (d=4)
    mmbf_grover_analysis(&safe_basis, 4);
    println!();

    // Sexy primes (d=6)
    mmbf_grover_analysis(&safe_basis, 6);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramanujan_grover_formula() {
        // For p=7, d=2: c_7(2) = -1, SP = (14-1+1)/49 = 14/49 = 2/7
        // F = 2, T_opt = floor(π/4 · √(7/2)) = floor(0.7854 · 1.871) = 1
        assert_eq!(ramanujan_sum(7, 2), -1);
        assert_eq!(forbidden_count(7, 2), 2);
        assert_eq!(optimal_iterations_lane(7, 2), 1);
    }

    #[test]
    fn test_admissibility() {
        assert!(!is_fp2_admissible(2));   // 2 mod 4 = 2
        assert!(is_fp2_admissible(3));    // 3 mod 4 = 3
        assert!(!is_fp2_admissible(5));   // 5 mod 4 = 1
        assert!(is_fp2_admissible(7));    // 7 mod 4 = 3
        assert!(is_fp2_admissible(11));   // 11 mod 4 = 3
        assert!(!is_fp2_admissible(13));  // 13 mod 4 = 1
    }

    #[test]
    fn test_extended_prime_speedup() {
        // p = 10007, d = 2, F = 2
        // Classical: O(10007)
        // Grover: O(√5003) ≈ O(70)
        let t = optimal_iterations_lane(10007, 2);
        assert!(t > 50 && t < 80, "Expected ~55-70, got {}", t);
    }

    #[test]
    fn test_divisibility_enhancement_in_grover() {
        // When p | d: F=1, search space halved → Grover needs MORE iterations
        // p=7, d=7: F=1, T_opt = floor(π/4 · √7) = floor(2.08) = 2
        // p=7, d=2: F=2, T_opt = floor(π/4 · √3.5) = floor(1.47) = 1
        let t_div = optimal_iterations_lane(7, 7);
        let t_nodiv = optimal_iterations_lane(7, 2);
        assert!(t_div >= t_nodiv,
            "Divisibility should increase iterations: div={}, nodiv={}", t_div, t_nodiv);
    }

    #[test]
    fn test_t6_3_in_grover_context() {
        // T6-3 says: divisibility REDUCES sieve power by 1/p
        // In Grover: fewer marked states → more iterations needed
        // This is CONSISTENT: weaker oracle → slower amplification
        for p in [3, 5, 7, 11, 13] {
            let f_nodiv = forbidden_count(p, 1);  // p ∤ 1
            let f_div = forbidden_count(p, p);     // p | p
            assert_eq!(f_nodiv, 2);
            assert_eq!(f_div, 1);
            // Fewer forbidden = weaker oracle = more Grover iterations
        }
    }
}
