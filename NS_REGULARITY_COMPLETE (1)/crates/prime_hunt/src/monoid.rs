//! Monoid M_H: The multiplicative monoid of the zero-shadow locus.
//!
//! M_H = { n : gcd(n, 730730) = 1 }
//! where 730730 = 2 × 5 × 7 × 11 × 13 × 73
//!
//! Irreducible elements of M_H = "good primes" = primes coprime to 730730
//! Composite elements of M_H = "polluted composites"
//!
//! Test: Does this characterization yield anything beyond a standard sieve step?

use crate::crt::{gcd, factorize, is_prime, mod_pow, ramanujan_sum_prime};
use crate::shadow::S_R;
use crate::helical::CR;

/// The full coprimeness modulus.
/// M = 2 × 5 × 7 × 11 × 13 × 73 = 730730
/// Note: 3 is NOT in M because 3 ∤ CR and 3 ∉ S_R.
pub const M_COPRIME: u128 = 2 * 5 * 7 * 11 * 13 * 73;

/// Prime factors of M
pub const M_FACTORS: [u128; 6] = [2, 5, 7, 11, 13, 73];

/// Test membership in M_H: n is in the monoid iff gcd(n, M) = 1
pub fn in_monoid(n: u128) -> bool {
    n > 0 && gcd(n, M_COPRIME) == 1
}

/// Shadow entropy: sum of Ramanujan sums c_q(n) for q ∈ S_R
pub fn shadow_entropy(n: u128) -> i128 {
    S_R.iter().map(|&q| ramanujan_sum_prime(q, n)).sum()
}

/// Classify an element of M_H
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonoidClass {
    /// Not in M_H
    Outside,
    /// Irreducible generator (prime coprime to M)
    GoodPrime,
    /// Composite element with at least one factor ≤ 13 (e.g., 27=3³)
    PollutedComposite,
    /// Composite element with all factors > 13 (e.g., 289=17²)
    PureCompositeCoprimeBelow13,
    /// The number 1 (identity element)
    Identity,
}

impl std::fmt::Display for MonoidClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonoidClass::Outside => write!(f, "OUTSIDE"),
            MonoidClass::GoodPrime => write!(f, "GOOD_PRIME"),
            MonoidClass::PollutedComposite => write!(f, "POLLUTED"),
            MonoidClass::PureCompositeCoprimeBelow13 => write!(f, "PURE_COMPOSITE"),
            MonoidClass::Identity => write!(f, "IDENTITY"),
        }
    }
}

pub fn classify_monoid(n: u128) -> MonoidClass {
    if n == 1 { return MonoidClass::Identity; }
    if !in_monoid(n) { return MonoidClass::Outside; }
    if is_prime(n) { return MonoidClass::GoodPrime; }

    // Composite in M_H — check if any factor is ≤ 13
    // Factors of M are {2,5,7,11,13,73}. Since gcd(n,M)=1, n is coprime
    // to all of those. But n might have factor 3 (not in M).
    let factors = factorize(n);
    let has_small_factor = factors.iter().any(|&(p, _)| p <= 13);
    if has_small_factor {
        MonoidClass::PollutedComposite
    } else {
        MonoidClass::PureCompositeCoprimeBelow13
    }
}

/// Enumerate M_H elements in [lo, hi] with classification.
pub fn enumerate_monoid(lo: u128, hi: u128) -> MonoidCensus {
    let mut census = MonoidCensus {
        range: (lo, hi),
        total: 0,
        good_primes: Vec::new(),
        polluted: Vec::new(),
        pure_composites: Vec::new(),
        outside: 0,
    };

    for n in lo..=hi {
        match classify_monoid(n) {
            MonoidClass::Outside => census.outside += 1,
            MonoidClass::GoodPrime => {
                census.total += 1;
                census.good_primes.push(n);
            }
            MonoidClass::PollutedComposite => {
                census.total += 1;
                census.polluted.push(n);
            }
            MonoidClass::PureCompositeCoprimeBelow13 => {
                census.total += 1;
                census.pure_composites.push(n);
            }
            MonoidClass::Identity => census.total += 1,
        }
    }
    census
}

#[derive(Debug)]
pub struct MonoidCensus {
    pub range: (u128, u128),
    pub total: u128,
    pub good_primes: Vec<u128>,
    pub polluted: Vec<u128>,
    pub pure_composites: Vec<u128>,
    pub outside: u128,
}

/// Exponential reach test: compute b^k, verify monoid membership,
/// return helical coordinates.
pub fn exponential_reach(base: u128, max_exp: u32) -> Vec<ExpReachPoint> {
    assert!(in_monoid(base), "Base {} not in monoid", base);

    let mut results = Vec::new();
    let mut power = 1u128;

    for k in 1..=max_exp {
        // Use checked multiplication to detect overflow
        power = match power.checked_mul(base) {
            Some(v) => v,
            None => break, // overflow
        };

        let theta = power % CR;
        let big_k = power / CR;
        let entropy = shadow_entropy(power);
        let coprime_cr = gcd(power, CR) == 1;
        let coprime_m = gcd(power, M_COPRIME) == 1;
        let prime = if power < 1_000_000 { is_prime(power) } else { false }; // skip expensive check for huge

        results.push(ExpReachPoint {
            base,
            exponent: k,
            value: power,
            theta,
            winding_k: big_k,
            entropy,
            coprime_cr,
            coprime_m,
            is_prime: prime,
            class: classify_monoid(power),
        });
    }
    results
}

#[derive(Debug)]
pub struct ExpReachPoint {
    pub base: u128,
    pub exponent: u32,
    pub value: u128,
    pub theta: u128,
    pub winding_k: u128,
    pub entropy: i128,
    pub coprime_cr: bool,
    pub coprime_m: bool,
    pub is_prime: bool,
    pub class: MonoidClass,
}

/// Modular exponential reach: compute θ = b^k mod CR without computing b^k.
/// This is the O(log k) fast path.
pub fn modular_exp_reach(base: u128, exponent: u128) -> u128 {
    mod_pow(base % CR, exponent, CR)
}

/// Algorithm 1 test: the κ-filter from the document.
/// κ = (θ - v_anchor) · L⁻¹ mod 17
/// Claim: |κ| ≤ 1 and irreducible → prime
pub fn algorithm1_test(n: u128, v_anchor: u128) -> Alg1Result {
    if !in_monoid(n) {
        return Alg1Result {
            n, theta: 0, kappa: 0, passes_kappa: false,
            is_irreducible: false, predicted_prime: false, actual_prime: false,
        };
    }

    let theta = n % CR;
    // κ = (θ - v_anchor) · L⁻¹ mod 17
    // L = CR = 18980
    // Need L⁻¹ mod 17: 18980 mod 17 = 18980 - 1116*17 = 18980 - 18972 = 8
    // 8⁻¹ mod 17: 8*15 = 120 = 7*17+1, so 8⁻¹ = 15
    let l_inv_mod_17 = 15u128; // 18980⁻¹ mod 17

    let diff = if theta >= v_anchor {
        (theta - v_anchor) % 17
    } else {
        (17 - ((v_anchor - theta) % 17)) % 17
    };
    let kappa = (diff * l_inv_mod_17) % 17;

    // Map to signed: if kappa > 8, it's "negative"
    let kappa_signed = if kappa > 8 { kappa as i128 - 17 } else { kappa as i128 };
    let passes_kappa = kappa_signed.abs() <= 1;

    // Check irreducibility (= primality for elements of M_H that are prime)
    let actual_prime = is_prime(n);
    let is_irreducible = actual_prime; // In M_H, irreducible ⟺ prime

    let predicted_prime = passes_kappa && is_irreducible;

    Alg1Result {
        n,
        theta,
        kappa: kappa_signed,
        passes_kappa,
        is_irreducible,
        predicted_prime,
        actual_prime,
    }
}

#[derive(Debug)]
pub struct Alg1Result {
    pub n: u128,
    pub theta: u128,
    pub kappa: i128,
    pub passes_kappa: bool,
    pub is_irreducible: bool,
    pub predicted_prime: bool,
    pub actual_prime: bool,
}

/// The critical test: Does the κ-filter actually help?
/// Compare Algorithm 1 predictions vs simple monoid membership.
pub fn algorithm1_bulk_test(limit: u128, v_anchor: u128) -> Alg1BulkResult {
    let mut true_pos = 0u32;   // predicted prime AND is prime
    let mut false_pos = 0u32;  // predicted prime AND NOT prime
    let mut true_neg = 0u32;   // predicted NOT prime AND NOT prime
    let mut false_neg = 0u32;  // predicted NOT prime AND IS prime
    let mut kappa_passes = 0u32;
    let mut kappa_passes_prime = 0u32;
    let mut kappa_passes_composite = 0u32;
    let mut total_in_monoid = 0u32;
    let mut total_primes = 0u32;
    let mut total_composites = 0u32;

    // Kappa distribution
    let mut kappa_hist = [0u32; 17];

    for n in 2..=limit {
        if !in_monoid(n) { continue; }
        total_in_monoid += 1;

        let result = algorithm1_test(n, v_anchor);
        let prime = result.actual_prime;

        if prime { total_primes += 1; } else { total_composites += 1; }

        // Track kappa distribution
        let k_idx = ((result.kappa % 17 + 17) % 17) as usize;
        kappa_hist[k_idx] += 1;

        if result.passes_kappa {
            kappa_passes += 1;
            if prime { kappa_passes_prime += 1; } else { kappa_passes_composite += 1; }
        }

        // Algorithm 1 uses kappa AND irreducibility
        // But irreducibility IS primality, so this is circular
        // Instead, test: does kappa alone filter?
        match (result.passes_kappa, prime) {
            (true, true) => true_pos += 1,
            (true, false) => false_pos += 1,
            (false, false) => true_neg += 1,
            (false, true) => false_neg += 1,
        }
    }

    Alg1BulkResult {
        limit,
        total_in_monoid,
        total_primes,
        total_composites,
        kappa_passes,
        kappa_passes_prime,
        kappa_passes_composite,
        true_pos,
        false_pos,
        true_neg,
        false_neg,
        kappa_hist,
    }
}

#[derive(Debug)]
pub struct Alg1BulkResult {
    pub limit: u128,
    pub total_in_monoid: u32,
    pub total_primes: u32,
    pub total_composites: u32,
    pub kappa_passes: u32,
    pub kappa_passes_prime: u32,
    pub kappa_passes_composite: u32,
    pub true_pos: u32,
    pub false_pos: u32,
    pub true_neg: u32,
    pub false_neg: u32,
    pub kappa_hist: [u32; 17],
}

/// Multiplicative closure test: verify that products of monoid elements stay in monoid.
pub fn closure_test(elements: &[u128]) -> ClosureResult {
    let mut tested = 0u32;
    let mut passed = 0u32;
    let mut failed = Vec::new();

    for i in 0..elements.len() {
        for j in i..elements.len() {
            let product = match elements[i].checked_mul(elements[j]) {
                Some(v) => v,
                None => continue,
            };
            tested += 1;
            if in_monoid(product) {
                passed += 1;
            } else {
                failed.push((elements[i], elements[j], product));
            }
        }
    }

    ClosureResult { tested, passed, failed }
}

#[derive(Debug)]
pub struct ClosureResult {
    pub tested: u32,
    pub passed: u32,
    pub failed: Vec<(u128, u128, u128)>,
}

/// Density analysis: what fraction of M_H at winding K are prime?
pub fn density_by_winding(max_k: u32) -> Vec<(u32, u32, u32, f64)> {
    let mut results = Vec::new();

    for k in 0..max_k {
        let lo = k as u128 * CR;
        let hi = lo + CR - 1;
        let mut primes = 0u32;
        let mut in_mon = 0u32;

        for n in lo..=hi {
            if n < 2 { continue; }
            if in_monoid(n) {
                in_mon += 1;
                if is_prime(n) {
                    primes += 1;
                }
            }
        }

        let density = if in_mon > 0 { primes as f64 / in_mon as f64 } else { 0.0 };
        results.push((k, primes, in_mon, density));
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_m_coprime_factorization() {
        assert_eq!(M_COPRIME, 730730);
        assert_eq!(factorize(M_COPRIME), vec![
            (2, 1), (5, 1), (7, 1), (11, 1), (13, 1), (73, 1)
        ]);
    }

    #[test]
    fn test_3_not_in_m() {
        // 3 is NOT a factor of M, so 3 IS in the monoid
        assert!(in_monoid(3));
        assert!(in_monoid(9));
        assert!(in_monoid(27));
        // But 2, 5, 7, 11, 13, 73 are NOT in the monoid
        assert!(!in_monoid(2));
        assert!(!in_monoid(5));
        assert!(!in_monoid(7));
        assert!(!in_monoid(11));
        assert!(!in_monoid(13));
        assert!(!in_monoid(73));
    }

    #[test]
    fn test_monoid_classification() {
        assert_eq!(classify_monoid(17), MonoidClass::GoodPrime);
        assert_eq!(classify_monoid(19), MonoidClass::GoodPrime);
        assert_eq!(classify_monoid(27), MonoidClass::PollutedComposite); // 3³, has factor ≤ 13
        assert_eq!(classify_monoid(289), MonoidClass::PureCompositeCoprimeBelow13); // 17²
        assert_eq!(classify_monoid(323), MonoidClass::PureCompositeCoprimeBelow13); // 17×19
        assert_eq!(classify_monoid(10), MonoidClass::Outside); // 2×5
        assert_eq!(classify_monoid(1), MonoidClass::Identity);
    }

    #[test]
    fn test_monoid_closure() {
        // 27 × 17 = 459
        assert!(in_monoid(459));
        assert_eq!(shadow_entropy(459), -3);
        assert_eq!(gcd(459, CR), 1);
    }

    #[test]
    fn test_exponential_reach_27() {
        let points = exponential_reach(27, 5);
        assert_eq!(points.len(), 5);
        assert_eq!(points[0].value, 27);
        assert_eq!(points[0].theta, 27);
        assert!(points[0].coprime_m);
        assert_eq!(points[1].value, 729);
        assert_eq!(points[1].theta, 729);
        assert!(points[1].coprime_m);
        assert_eq!(points[2].value, 19683);
        assert_eq!(points[2].theta, 19683 % CR); // 703
        assert!(points[2].coprime_m);
    }

    #[test]
    fn test_modular_exp_fast() {
        // 27^5 mod 18980
        let theta_fast = modular_exp_reach(27, 5);
        let theta_slow = 14348907u128 % CR; // 27^5 = 14348907
        assert_eq!(theta_fast, theta_slow);
    }

    #[test]
    fn test_73_excluded() {
        // 73 is a factor of CR (18980 = 4×5×13×73), so gcd(73, CR) = 73
        assert!(!in_monoid(73));
        assert_eq!(gcd(73, CR), 73);
    }

    #[test]
    fn test_shadow_entropy_monoid() {
        // All monoid elements should have entropy = -3
        for n in 2..500u128 {
            if in_monoid(n) {
                assert_eq!(shadow_entropy(n), -3,
                    "n={} in monoid but entropy={}", n, shadow_entropy(n));
            }
        }
    }
}
