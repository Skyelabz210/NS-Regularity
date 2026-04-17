//! Prime Hunting Sieve: Coprimality-first prime detection.
//!
//! Core insight: Primes are gaps in the shadow network — residues on the CRT
//! torus that no K-Elimination winding from parked composites can reach.

use crate::crt::{k_eliminate, factorize, is_prime, ramanujan_sum_prime};
use crate::gear::SafeAnchor;
use crate::shadow::{
    classify_prime, is_admissible, residue_differential, spectral_interaction,
    ShadowState, PrimeTier, S_R, SAFE_BASIS,
};

/// Result of a prime hunt scan.
#[derive(Debug)]
pub struct HuntResult {
    pub candidate: u128,
    pub is_prime: bool,
    pub tier: PrimeTier,
    pub shadow_signature: ShadowSignature,
    pub k_vort_profile: KVortProfile,
    pub admissible: bool,
}

/// Shadow network signature for a candidate.
#[derive(Debug)]
pub struct ShadowSignature {
    /// Residues mod each of SAFE_BASIS
    pub residues: [u128; 6],
    /// Whether the candidate is shadow-active on Lane 11
    pub lane_11_zero: bool,
    /// Whether the candidate is boundary-active on Lane 13
    pub lane_13_zero: bool,
    /// Count of zero residues across safe basis
    pub zero_count: u32,
    /// Spectral power (sum of Ramanujan sums squared against neighbors)
    pub spectral_power: i128,
}

/// K-Elimination winding profile.
#[derive(Debug)]
pub struct KVortProfile {
    /// Winding numbers from various anchor configurations
    pub windings: Vec<(u128, u128, u128)>, // (m_safe, anchor, k)
    /// Whether any winding hits 0 mod 11
    pub shadow_winding: bool,
    /// Whether any winding carries 11^k for k >= 2
    pub deep_shadow: bool,
    /// Highest power of 11 found in any winding
    pub max_11_power: u32,
}

/// Analyze a single candidate number.
pub fn analyze_candidate(n: u128) -> HuntResult {
    let prime = is_prime(n);
    let tier = classify_prime(n);
    let admissible = is_admissible(n);

    // Shadow signature: residues mod safe basis
    let mut residues = [0u128; 6];
    for (i, &p) in SAFE_BASIS.iter().enumerate() {
        residues[i] = n % p;
    }

    let lane_11_zero = residues[4] == 0;
    let lane_13_zero = residues[5] == 0;
    let zero_count = residues.iter().filter(|&&r| r == 0).count() as u32;

    // Spectral power: how this candidate interacts with its neighbors
    let spectral_power = if n > 1 {
        let sp_minus = spectral_interaction(n, n - 1);
        let sp_plus = spectral_interaction(n, n + 1);
        sp_minus.total_power + sp_plus.total_power
    } else {
        0
    };

    let shadow_sig = ShadowSignature {
        residues,
        lane_11_zero,
        lane_13_zero,
        zero_count,
        spectral_power,
    };

    // K-vort profile: try multiple anchor configurations
    let mut windings = Vec::new();
    let mut shadow_winding = false;
    let mut deep_shadow = false;
    let mut max_11_power = 0u32;

    // Configuration 1: basis {2,3,5}, anchor 7
    if let Some(k) = k_eliminate(n % 30, 30, n % 7, 7) {
        windings.push((30, 7, k));
        check_11_power(k, &mut shadow_winding, &mut deep_shadow, &mut max_11_power);
    }

    // Configuration 2: basis {2,3,5,7}, anchor 11
    if let Some(k) = k_eliminate(n % 210, 210, n % 11, 11) {
        windings.push((210, 11, k));
        check_11_power(k, &mut shadow_winding, &mut deep_shadow, &mut max_11_power);
    }

    // Configuration 3: basis {2,3,5,7}, anchor 13
    if let Some(k) = k_eliminate(n % 210, 210, n % 13, 13) {
        windings.push((210, 13, k));
        check_11_power(k, &mut shadow_winding, &mut deep_shadow, &mut max_11_power);
    }

    // Configuration 4: basis {2,3,5,7,11}, anchor 13
    if let Some(k) = k_eliminate(n % 2310, 2310, n % 13, 13) {
        windings.push((2310, 13, k));
        check_11_power(k, &mut shadow_winding, &mut deep_shadow, &mut max_11_power);
    }

    let k_vort_profile = KVortProfile {
        windings,
        shadow_winding,
        deep_shadow,
        max_11_power,
    };

    HuntResult {
        candidate: n,
        is_prime: prime,
        tier,
        shadow_signature: shadow_sig,
        k_vort_profile,
        admissible,
    }
}

fn check_11_power(k: u128, shadow_winding: &mut bool, deep_shadow: &mut bool, max_power: &mut u32) {
    if k == 0 { return; }
    if k % 11 == 0 {
        *shadow_winding = true;
        let mut power = 0u32;
        let mut val = k;
        while val % 11 == 0 {
            power += 1;
            val /= 11;
        }
        if power >= 2 { *deep_shadow = true; }
        if power > *max_power { *max_power = power; }
    }
}

/// The Shadow Gap Sieve: identify primes by finding residues that escape
/// all shadow network coverage.
///
/// Core algorithm:
/// 1. Fix a CRT basis B = {p_1, ..., p_k}
/// 2. For each residue r in [0, product(B)), compute its shadow coverage
/// 3. Residues with zero shadow coverage are prime candidates
/// 4. Verify candidates with Miller-Rabin
pub fn shadow_gap_sieve(limit: u128) -> SieveResult {
    let mut primes_found = Vec::new();
    let mut composites_in_gaps = Vec::new();
    let mut shadow_covered: Vec<u128> = Vec::new();

    // Sieve basis: {2, 3, 5, 7, 11, 13}
    // Product = 30030
    let basis_product: u128 = SAFE_BASIS.iter().product(); // 30030

    // For each residue class mod 30030, determine shadow coverage
    let mut gap_residues = Vec::new();

    for r in 0..basis_product.min(limit) {
        let mods: Vec<u128> = SAFE_BASIS.iter().map(|&p| r % p).collect();

        // A residue is "shadow covered" if it shares a factor with any basis prime
        let covered = SAFE_BASIS.iter().any(|&p| r % p == 0);

        if !covered && r > 1 {
            gap_residues.push(r);
        }
    }

    // Now walk actual numbers, checking residue classes
    let mut n = 2u128;
    while n <= limit {
        if is_prime(n) {
            let result = analyze_candidate(n);
            primes_found.push(result);
        }
        n += 1;
    }

    // Count how many gap residues correspond to actual primes
    let gap_prime_count = gap_residues.iter()
        .filter(|&&r| r > 1 && is_prime(r))
        .count();

    SieveResult {
        limit,
        basis_product,
        total_gap_residues: gap_residues.len(),
        gap_prime_count,
        primes_found,
        composites_in_gaps,
        gap_residues,
    }
}

/// Optimized shadow gap sieve: walks only admissible residue classes
/// (coprime to SCALE = 10000 = 2^4 × 5^4)
pub fn admissible_sieve(limit: u128) -> AdmissibleSieveResult {
    // Step 1: Compute the wheel. Numbers coprime to 2 and 5 follow
    // a repeating pattern mod 10: {1, 3, 7, 9}
    let wheel = [1u128, 3, 7, 9];

    let mut primes = Vec::new();
    let mut total_checked = 0u128;
    let mut shadow_hits = 0u128;
    let mut boundary_hits = 0u128;

    // Start from 3 (smallest admissible prime)
    for base in (0..limit).step_by(10) {
        for &offset in &wheel {
            let n = base + offset;
            if n < 2 || n > limit { continue; }
            total_checked += 1;

            // Quick shadow check before expensive primality test
            let r11 = n % 11;
            let r13 = n % 13;

            if r11 == 0 { shadow_hits += 1; }
            if r13 == 0 { boundary_hits += 1; }

            // Skip multiples of 3, 7, 11, 13 (they're composite unless they ARE the prime)
            if n > 13 && (n % 3 == 0 || n % 7 == 0 || n % 11 == 0 || n % 13 == 0) {
                continue;
            }

            if is_prime(n) {
                primes.push(n);
            }
        }
    }

    AdmissibleSieveResult {
        limit,
        primes,
        total_checked,
        shadow_hits,
        boundary_hits,
    }
}

#[derive(Debug)]
pub struct SieveResult {
    pub limit: u128,
    pub basis_product: u128,
    pub total_gap_residues: usize,
    pub gap_prime_count: usize,
    pub primes_found: Vec<HuntResult>,
    pub composites_in_gaps: Vec<u128>,
    pub gap_residues: Vec<u128>,
}

#[derive(Debug)]
pub struct AdmissibleSieveResult {
    pub limit: u128,
    pub primes: Vec<u128>,
    pub total_checked: u128,
    pub shadow_hits: u128,
    pub boundary_hits: u128,
}

/// Cousin pair (gap-4) detector with spectral analysis.
/// Finds pairs (p, p+4) where both are prime.
pub fn find_cousin_pairs(limit: u128) -> Vec<CousinPair> {
    let mut pairs = Vec::new();
    let mut p = 3u128;
    while p + 4 <= limit {
        if is_prime(p) && is_prime(p + 4) {
            // Spectral analysis on the torus Z/pZ × Z/(p+4)Z
            let interaction = spectral_interaction(p, p + 4);

            // Ramanujan correlation: c_7(4) × c_11(4)
            let c7_4 = ramanujan_sum_prime(7, 4);
            let c11_4 = ramanujan_sum_prime(11, 4);
            let shadow_correlated = c7_4 == -1 && c11_4 == -1;

            // Residue differential mod safe basis
            let diff = residue_differential(p, p + 4);
            let shadow = ShadowState::from_diff(&[diff[0], diff[1], diff[2], diff[3], diff[4]]);

            pairs.push(CousinPair {
                p,
                q: p + 4,
                spectral_power: interaction.total_power,
                shadow_correlated,
                shadow_strength: shadow.strength,
                diff_mod_safe: diff,
            });
        }
        p += 2; // Only odd numbers
    }
    pairs
}

#[derive(Debug)]
pub struct CousinPair {
    pub p: u128,
    pub q: u128,
    pub spectral_power: i128,
    pub shadow_correlated: bool,
    pub shadow_strength: u32,
    pub diff_mod_safe: [u128; 6],
}

/// Displacement power analysis: track how "missing" primes appear
/// in displacements between periods.
pub fn displacement_power_analysis(periods: &[(String, u128)]) -> Vec<DisplacementReport> {
    let mut reports = Vec::new();
    let eclipse = 11960u128;

    for (name, &(ref _n, period)) in periods.iter().enumerate() {
        let disp = eclipse % period;
        if disp == 0 { continue; }

        let factors = factorize(disp);
        let mut missing_prime_powers = Vec::new();

        // Check which primes are "missing" from the base period
        // but appear in the displacement
        for &q in &S_R {
            if period % q != 0 {
                // q doesn't divide the period — it's "missing"
                // Check if it appears in the displacement
                if let Some(&(_, power)) = factors.iter().find(|&&(p, _)| p == q) {
                    missing_prime_powers.push((q, power));
                }
            }
        }
        // Also check boundary prime 13
        if period % 13 != 0 {
            if let Some(&(_, power)) = factors.iter().find(|&&(p, _)| p == 13) {
                missing_prime_powers.push((13, power));
            }
        }

        reports.push(DisplacementReport {
            period_name: periods[name].0.clone(),
            period,
            displacement: disp,
            factors: factors.clone(),
            missing_prime_powers,
        });
    }
    reports
}

#[derive(Debug)]
pub struct DisplacementReport {
    pub period_name: String,
    pub period: u128,
    pub displacement: u128,
    pub factors: Vec<(u128, u32)>,
    pub missing_prime_powers: Vec<(u128, u32)>,
}

/// K-Vort accumulator: track winding number evolution over time
/// to detect patterns in how shadow primes appear.
pub fn k_vort_accumulator_scan(
    basis: &[u128],
    anchor: u128,
    steps: u128,
) -> KVortScan {
    let mut sa = SafeAnchor::new(basis, anchor);
    let mut winding_history = Vec::new();
    let mut shadow_return_times = Vec::new();
    let mut max_k = 0u128;

    for step in 1..=steps {
        sa.add(1);
        if let Some(k) = sa.k_vort_fast() {
            if k > max_k { max_k = k; }

            // Track when winding hits 0 mod 11
            if k > 0 && k % 11 == 0 {
                shadow_return_times.push(step);
            }

            // Sample periodically to avoid massive output
            if step % (steps / 100).max(1) == 0 || step <= 100 {
                winding_history.push((step, k));
            }
        }
    }

    // Analyze inter-return intervals
    let mut intervals = Vec::new();
    for i in 1..shadow_return_times.len() {
        intervals.push(shadow_return_times[i] - shadow_return_times[i - 1]);
    }

    KVortScan {
        basis: basis.to_vec(),
        anchor,
        steps,
        max_k,
        shadow_return_count: shadow_return_times.len(),
        shadow_return_times,
        return_intervals: intervals,
        winding_history,
    }
}

#[derive(Debug)]
pub struct KVortScan {
    pub basis: Vec<u128>,
    pub anchor: u128,
    pub steps: u128,
    pub max_k: u128,
    pub shadow_return_count: usize,
    pub shadow_return_times: Vec<u128>,
    pub return_intervals: Vec<u128>,
    pub winding_history: Vec<(u128, u128)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_candidate_prime() {
        let r = analyze_candidate(17);
        assert!(r.is_prime);
        assert_eq!(r.tier, PrimeTier::Turbulent);
        assert!(r.admissible);
    }

    #[test]
    fn test_analyze_candidate_composite() {
        let r = analyze_candidate(15);
        assert!(!r.is_prime);
    }

    #[test]
    fn test_admissible_sieve_small() {
        let result = admissible_sieve(100);
        // Wheel {1,3,7,9} mod 10 misses 2 and 5 (factors of SCALE)
        // 23 primes found via wheel + small-prime filter
        assert!(result.primes.len() >= 23);
        assert!(result.primes.contains(&3));
        assert!(result.primes.contains(&97));
    }

    #[test]
    fn test_cousin_pairs() {
        let pairs = find_cousin_pairs(100);
        // Known cousin pairs up to 100: (3,7), (7,11), (13,17), (37,41), (43,47), (67,71), (79,83)
        assert!(pairs.len() >= 7);
        assert!(pairs.iter().any(|p| p.p == 3 && p.q == 7));
        assert!(pairs.iter().any(|p| p.p == 7 && p.q == 11));
    }

    #[test]
    fn test_displacement_analysis() {
        let periods = vec![
            ("Mars".to_string(), 780u128),
            ("Venus".to_string(), 584),
            ("Saturn".to_string(), 378),
        ];
        let reports = displacement_power_analysis(&periods);
        // Saturn displacement = 11960 % 378 = 242 = 2 × 11²
        let saturn = reports.iter().find(|r| r.period_name == "Saturn").unwrap();
        assert_eq!(saturn.displacement, 242);
        // 11 is missing from Saturn's period (378 = 2×3³×7) but appears as 11² in displacement
        assert!(saturn.missing_prime_powers.iter().any(|&(p, k)| p == 11 && k == 2));
    }

    #[test]
    fn test_k_vort_scan() {
        // With anchor=7, k ∈ {0..6}, never ≡ 0 mod 11
        let scan = k_vort_accumulator_scan(&[2, 3, 5], 7, 1000);
        assert_eq!(scan.shadow_return_count, 0); // k < 11 always
        assert!(!scan.winding_history.is_empty());

        // With anchor=13, k can hit multiples of 11
        let scan2 = k_vort_accumulator_scan(&[2, 3, 5, 7], 13, 10000);
        assert!(scan2.shadow_return_count > 0);
    }
}
