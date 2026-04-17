//! Helical Prime Pattern: Tests the CR=18980 cylinder hypothesis.
//!
//! Claim: Primes form a triple-helix on S¹ × ℤ with circumference CR=18980.
//! Test: Do the residue distributions, gap structures, and shadow signatures
//! support or refute this geometric picture?

use crate::crt::{self, gcd, factorize, is_prime, euler_totient, ramanujan_sum_prime};
use crate::shadow::S_R;

/// Calendar Round circumference
pub const CR: u128 = 18980; // 2² × 5 × 13 × 73

/// Helical coordinate of a number on the CR cylinder.
#[derive(Debug, Clone)]
pub struct HelicalCoord {
    pub n: u128,
    pub theta: u128,   // n mod CR (toroidal position)
    pub k: u128,       // n / CR (winding number)
}

impl HelicalCoord {
    pub fn new(n: u128) -> Self {
        Self { n, theta: n % CR, k: n / CR }
    }
}

/// Shadow entropy: sum of c_q(n)/(q-1) for q ∈ S_R
/// For primes > 11: always -3 (each term is -1/(q-1) summed... actually -1 per term)
/// For composites divisible by q: that term becomes +1 instead of -1/(q-1)
pub fn shadow_entropy(n: u128) -> i128 {
    let mut entropy = 0i128;
    for &q in &S_R {
        entropy += ramanujan_sum_prime(q, n);
    }
    entropy
}

/// Full helical analysis of primes up to limit.
pub fn helical_analysis(limit: u128) -> HelicalReport {
    let primes: Vec<u128> = (2..=limit).filter(|&n| is_prime(n)).collect();

    // 1. Map primes to helical coordinates
    let coords: Vec<HelicalCoord> = primes.iter().map(|&p| HelicalCoord::new(p)).collect();

    // 2. Gap analysis with helical decomposition
    let mut gaps = Vec::new();
    for i in 1..primes.len() {
        let gap = primes[i] - primes[i - 1];
        let dtheta_raw = (primes[i] % CR) as i128 - (primes[i - 1] % CR) as i128;
        let dk = (primes[i] / CR) as i128 - (primes[i - 1] / CR) as i128;

        // Cylindrical gap: minimum arc distance on S¹
        let dtheta_cyl = if dtheta_raw.abs() <= CR as i128 / 2 {
            dtheta_raw
        } else if dtheta_raw > 0 {
            dtheta_raw - CR as i128
        } else {
            dtheta_raw + CR as i128
        };

        gaps.push(GapRecord {
            p1: primes[i - 1],
            p2: primes[i],
            linear_gap: gap,
            dtheta: dtheta_cyl,
            dk,
            same_winding: dk == 0,
        });
    }

    // 3. Shadow entropy distribution
    let mut entropy_hist = std::collections::HashMap::new();
    for &p in &primes {
        let e = shadow_entropy(p);
        *entropy_hist.entry(e).or_insert(0u32) += 1;
    }

    // 4. Residue class analysis mod CR
    let phi_cr = euler_totient(CR);
    let mut theta_dist = std::collections::HashMap::new();
    for coord in &coords {
        *theta_dist.entry(coord.theta).or_insert(0u32) += 1;
    }

    // 5. Winding distribution
    let max_k = coords.last().map(|c| c.k).unwrap_or(0);
    let mut k_counts = vec![0u32; (max_k + 1) as usize];
    for coord in &coords {
        k_counts[coord.k as usize] += 1;
    }

    // 6. θ coprimality with CR
    let mut coprime_to_cr = 0u32;
    let mut not_coprime_to_cr = 0u32;
    for coord in &coords {
        if coord.theta > 1 {
            if gcd(coord.theta, CR) == 1 {
                coprime_to_cr += 1;
            } else {
                not_coprime_to_cr += 1;
            }
        }
    }

    // 7. Gap classification
    let mut gap_class = std::collections::HashMap::new();
    for g in &gaps {
        let class = if g.linear_gap == 1 { "unit" }
            else if g.linear_gap == 2 { "twin" }
            else if g.linear_gap == 4 { "cousin" }
            else if g.linear_gap == 6 { "sexy" }
            else if g.linear_gap % 2 == 0 && g.linear_gap <= 12 { "small_even" }
            else if g.linear_gap % 2 == 0 { "large_even" }
            else { "odd" };
        *gap_class.entry(class).or_insert(0u32) += 1;
    }

    // 8. Helix A verification: residues mod 30
    let mut mod30_prime_residues = std::collections::HashSet::new();
    for &p in &primes {
        if p > 5 {
            mod30_prime_residues.insert(p % 30);
        }
    }

    // 9. Cross-winding residue persistence
    // Do the same θ values appear across multiple windings?
    let mut theta_windings: std::collections::HashMap<u128, Vec<u128>> = std::collections::HashMap::new();
    for coord in &coords {
        theta_windings.entry(coord.theta).or_default().push(coord.k);
    }
    let multi_winding_thetas = theta_windings.iter()
        .filter(|(_, ks)| ks.len() > 1)
        .count();

    HelicalReport {
        limit,
        total_primes: primes.len(),
        phi_cr,
        coords,
        gaps,
        entropy_hist,
        theta_count: theta_dist.len(),
        k_counts,
        coprime_to_cr,
        not_coprime_to_cr,
        gap_classification: gap_class,
        mod30_residues: mod30_prime_residues,
        multi_winding_thetas,
        total_distinct_thetas: theta_windings.len(),
    }
}

#[derive(Debug)]
pub struct GapRecord {
    pub p1: u128,
    pub p2: u128,
    pub linear_gap: u128,
    pub dtheta: i128,   // cylindrical angular gap
    pub dk: i128,       // winding gap
    pub same_winding: bool,
}

#[derive(Debug)]
pub struct HelicalReport {
    pub limit: u128,
    pub total_primes: usize,
    pub phi_cr: u128,
    pub coords: Vec<HelicalCoord>,
    pub gaps: Vec<GapRecord>,
    pub entropy_hist: std::collections::HashMap<i128, u32>,
    pub theta_count: usize,
    pub k_counts: Vec<u32>,
    pub coprime_to_cr: u32,
    pub not_coprime_to_cr: u32,
    pub gap_classification: std::collections::HashMap<&'static str, u32>,
    pub mod30_residues: std::collections::HashSet<u128>,
    pub multi_winding_thetas: usize,
    pub total_distinct_thetas: usize,
}

/// Test the "shadow minima" claim: are primes exactly the numbers
/// with shadow_entropy = -3?
pub fn test_shadow_minima(limit: u128) -> ShadowMinimaReport {
    let mut primes_at_minus3 = 0u32;
    let mut primes_not_minus3 = 0u32;
    let mut composites_at_minus3 = 0u32;
    let mut composites_not_minus3 = 0u32;
    let mut false_positives = Vec::new();

    for n in 2..=limit {
        let e = shadow_entropy(n);
        let prime = is_prime(n);

        match (prime, e == -3) {
            (true, true) => primes_at_minus3 += 1,
            (true, false) => primes_not_minus3 += 1,
            (false, true) => {
                composites_at_minus3 += 1;
                if false_positives.len() < 50 {
                    false_positives.push(n);
                }
            }
            (false, false) => composites_not_minus3 += 1,
        }
    }

    ShadowMinimaReport {
        limit,
        primes_at_minus3,
        primes_not_minus3,
        composites_at_minus3,
        composites_not_minus3,
        false_positives,
    }
}

#[derive(Debug)]
pub struct ShadowMinimaReport {
    pub limit: u128,
    pub primes_at_minus3: u32,
    pub primes_not_minus3: u32,
    pub composites_at_minus3: u32,
    pub composites_not_minus3: u32,
    pub false_positives: Vec<u128>,
}

/// Test the "19001 is first prime of K=1" claim and analyze
/// first primes per winding.
pub fn first_primes_per_winding(max_k: u32) -> Vec<(u32, u128, u128)> {
    let mut results = Vec::new();
    for k in 0..max_k {
        let base = k as u128 * CR;
        for offset in 0..CR {
            let n = base + offset;
            if n >= 2 && is_prime(n) {
                let theta = offset;
                results.push((k, n, theta));
                break;
            }
        }
    }
    results
}

/// Analyze the factorization of first-prime θ values.
pub fn first_prime_theta_factors(max_k: u32) -> Vec<(u32, u128, u128, Vec<(u128, u32)>)> {
    first_primes_per_winding(max_k)
        .into_iter()
        .map(|(k, n, theta)| {
            let factors = if theta > 1 { factorize(theta) } else { vec![] };
            (k, n, theta, factors)
        })
        .collect()
}

/// Test gap structure claim: "gaps of 2,4,6 capture 98.8% of gaps"
pub fn gap_concentration_test(limit: u128) -> GapConcentration {
    let primes: Vec<u128> = (2..=limit).filter(|&n| is_prime(n)).collect();
    let total_gaps = primes.len() - 1;

    let mut gap_hist = std::collections::HashMap::new();
    for i in 1..primes.len() {
        let gap = primes[i] - primes[i - 1];
        *gap_hist.entry(gap).or_insert(0u32) += 1;
    }

    let small_gaps: u32 = gap_hist.iter()
        .filter(|(&g, _)| g <= 6)
        .map(|(_, &c)| c)
        .sum();

    let medium_gaps: u32 = gap_hist.iter()
        .filter(|(&g, _)| g > 6 && g <= 12)
        .map(|(_, &c)| c)
        .sum();

    let large_gaps: u32 = gap_hist.iter()
        .filter(|(&g, _)| g > 12)
        .map(|(_, &c)| c)
        .sum();

    let mut sorted_gaps: Vec<_> = gap_hist.into_iter().collect();
    sorted_gaps.sort_by_key(|&(g, _)| g);

    GapConcentration {
        limit,
        total_gaps,
        small_246: small_gaps,
        medium_8_12: medium_gaps,
        large_gt_12: large_gaps,
        pct_small: small_gaps as f64 / total_gaps as f64 * 100.0,
        pct_medium: medium_gaps as f64 / total_gaps as f64 * 100.0,
        pct_large: large_gaps as f64 / total_gaps as f64 * 100.0,
        gap_histogram: sorted_gaps,
    }
}

#[derive(Debug)]
pub struct GapConcentration {
    pub limit: u128,
    pub total_gaps: usize,
    pub small_246: u32,
    pub medium_8_12: u32,
    pub large_gt_12: u32,
    pub pct_small: f64,
    pub pct_medium: f64,
    pub pct_large: f64,
    pub gap_histogram: Vec<(u128, u32)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cr_factorization() {
        let f = factorize(CR);
        assert_eq!(f, vec![(2, 2), (5, 1), (13, 1), (73, 1)]);
    }

    #[test]
    fn test_shadow_entropy_prime() {
        // For p=17 (prime, not in S_R): c_5(17)=-1, c_7(17)=-1, c_11(17)=-1
        assert_eq!(shadow_entropy(17), -3);
        assert_eq!(shadow_entropy(19), -3);
        assert_eq!(shadow_entropy(97), -3);
    }

    #[test]
    fn test_shadow_entropy_composite() {
        // 15 = 3×5: c_5(15) = 4 (since 5|15), c_7(15) = -1, c_11(15) = -1
        assert_eq!(shadow_entropy(15), 4 - 1 - 1); // = 2
        // 35 = 5×7: c_5(35) = 4, c_7(35) = 6, c_11(35) = -1
        assert_eq!(shadow_entropy(35), 4 + 6 - 1); // = 9
    }

    #[test]
    fn test_19001_prime() {
        assert!(is_prime(19001));
        assert_eq!(19001 % CR, 21);
        assert_eq!(factorize(21), vec![(3, 1), (7, 1)]);
        assert_eq!(gcd(21, CR), 1); // coprime to CR
    }

    #[test]
    fn test_helical_coord() {
        let c = HelicalCoord::new(19001);
        assert_eq!(c.theta, 21);
        assert_eq!(c.k, 1);
    }
}
