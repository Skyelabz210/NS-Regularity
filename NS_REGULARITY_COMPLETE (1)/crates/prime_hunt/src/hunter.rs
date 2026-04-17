//! Hydra Prime Finder — Use coprime lattice structure to find primes faster.
//!
//! Strategy:
//! 1. Calibrate: map coprime strata on small range, measure prime density per stratum
//! 2. Target: identify prime-rich strata (above threshold)
//! 3. Hunt: on large range, only test candidates in prime-rich strata
//! 4. The coprime composites in prime-poor strata are SKIPPED
//!
//! This inverts the sieve: instead of eliminating composites,
//! we identify WHERE primes live in the coprime lattice and go there.

use crate::multi_hydra::{MultiHydra, Head, build_twin_hydra, build_progressive};
use crate::coprime_families::{CramTopology, Lane, Root, Post, Champions};
use crate::crt;

/// Result of a Hydra prime hunt.
#[derive(Debug)]
pub struct HuntResult {
    pub primes_found: Vec<u64>,
    pub range: (u64, u64),
    pub total_in_range: u64,
    pub coprimes_in_range: u64,
    pub candidates_tested: u64,      // primality tests actually performed
    pub candidates_skipped: u64,     // skipped (in prime-poor strata)
    pub false_negatives: u64,        // primes we missed by skipping
    pub speedup: f64,                // coprimes_in_range / candidates_tested
    pub recall: f64,                 // primes_found / actual_primes
    pub strata_targeted: u32,
    pub strata_skipped: u32,
}

/// Calibration data for one stratum.
#[derive(Debug, Clone)]
struct StratumProfile {
    sig_id: u64,
    coprime_count: u32,
    prime_count: u32,
    density: f64,
    targeted: bool,
}

/// The Hydra Prime Hunter.
pub struct HydraPrimeHunter {
    hydra: MultiHydra,
    profiles: Vec<StratumProfile>,
    density_threshold: f64,
    calibrated: bool,
}

fn is_prime_u64(n: u64) -> bool {
    if n < 2 { return false; } if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5u64;
    while i.saturating_mul(i) <= n {
        if n % i == 0 || n % (i+2) == 0 { return false; } i += 6;
    }
    true
}

impl HydraPrimeHunter {
    pub fn new(hydra: MultiHydra) -> Self {
        Self { hydra, profiles: Vec::new(), density_threshold: 0.0, calibrated: false }
    }

    /// Calibrate on [cal_start, cal_limit): learn which strata are prime-rich.
    pub fn calibrate(&mut self, cal_start: u64, cal_limit: u64, density_percentile: f64) {
        let mut strata: std::collections::HashMap<u64, (u32, u32)> = std::collections::HashMap::new();

        for n in cal_start.max(2)..cal_limit {
            let coprime = self.hydra.heads.iter().all(|h|
                crt::gcd(n as u128, h.topology.modulus as u128) == 1
            );
            if !coprime { continue; }

            let sig = self.hydra.observe(n);
            let sig_id = sig.to_id();
            let entry = strata.entry(sig_id).or_insert((0, 0));
            entry.0 += 1;
            if is_prime_u64(n) { entry.1 += 1; }
        }

        // Build profiles
        self.profiles = strata.into_iter().map(|(sig_id, (cc, pc))| {
            StratumProfile {
                sig_id,
                coprime_count: cc,
                prime_count: pc,
                density: if cc > 0 { pc as f64 / cc as f64 } else { 0.0 },
                targeted: false,
            }
        }).collect();

        // Sort by density descending
        self.profiles.sort_by(|a, b| b.density.partial_cmp(&a.density).unwrap());

        // Determine threshold: target top percentile of strata
        // Find density cutoff that captures density_percentile of all primes
        let total_primes: u32 = self.profiles.iter().map(|p| p.prime_count).sum();
        let target_primes = (total_primes as f64 * density_percentile) as u32;

        let mut captured = 0u32;
        let mut threshold = 0.0f64;
        for profile in &mut self.profiles {
            if captured < target_primes || density_percentile >= 0.999 {
                profile.targeted = true;
                captured += profile.prime_count;
                threshold = profile.density;
            } else {
                profile.targeted = false;
            }
        }

        self.density_threshold = threshold;
        self.calibrated = true;
    }

    /// Hunt for primes in [hunt_start, hunt_limit).
    /// Only tests candidates in targeted (prime-rich) strata.
    pub fn hunt(&self, hunt_start: u64, hunt_limit: u64) -> HuntResult {
        assert!(self.calibrated, "Must calibrate before hunting");

        let targeted_ids: std::collections::HashSet<u64> = self.profiles.iter()
            .filter(|p| p.targeted)
            .map(|p| p.sig_id)
            .collect();

        let all_known_ids: std::collections::HashSet<u64> = self.profiles.iter()
            .map(|p| p.sig_id)
            .collect();

        let strata_targeted = targeted_ids.len() as u32;
        let strata_skipped = self.profiles.iter().filter(|p| !p.targeted).count() as u32;

        let mut primes_found = Vec::new();
        let mut total = 0u64;
        let mut coprimes = 0u64;
        let mut tested = 0u64;
        let mut skipped = 0u64;
        let mut false_negatives = 0u64;

        for n in hunt_start.max(2)..hunt_limit {
            total += 1;
            let coprime = self.hydra.heads.iter().all(|h|
                crt::gcd(n as u128, h.topology.modulus as u128) == 1
            );
            if !coprime { continue; }
            coprimes += 1;

            let sig = self.hydra.observe(n);
            let sig_id = sig.to_id();

            if targeted_ids.contains(&sig_id) {
                // Prime-rich stratum: test primality
                tested += 1;
                if is_prime_u64(n) {
                    primes_found.push(n);
                }
            } else if !all_known_ids.contains(&sig_id) {
                // UNKNOWN stratum (not seen in calibration): test to be safe
                tested += 1;
                if is_prime_u64(n) {
                    primes_found.push(n);
                }
            } else {
                // Prime-poor stratum: skip
                skipped += 1;
                // Track false negatives for accuracy measurement
                if is_prime_u64(n) {
                    false_negatives += 1;
                }
            }
        }

        let actual_primes = primes_found.len() as u64 + false_negatives;
        let recall = if actual_primes > 0 {
            primes_found.len() as f64 / actual_primes as f64
        } else { 1.0 };
        let speedup = if tested > 0 {
            coprimes as f64 / tested as f64
        } else { 1.0 };

        HuntResult {
            primes_found,
            range: (hunt_start, hunt_limit),
            total_in_range: total,
            coprimes_in_range: coprimes,
            candidates_tested: tested,
            candidates_skipped: skipped,
            false_negatives,
            speedup,
            recall,
            strata_targeted,
            strata_skipped,
        }
    }

    pub fn profiles(&self) -> &[StratumProfile] { &self.profiles }
    pub fn threshold(&self) -> f64 { self.density_threshold }
    pub fn targeted_count(&self) -> usize {
        self.profiles.iter().filter(|p| p.targeted).count()
    }
    pub fn total_strata(&self) -> usize { self.profiles.len() }
}

/// Run a complete hunt with reporting.
pub fn full_hunt(
    hydra: MultiHydra,
    cal_range: (u64, u64),
    hunt_range: (u64, u64),
    recall_target: f64,  // e.g. 0.95 for 95% recall
) -> (HydraPrimeHunter, HuntResult) {
    let mut hunter = HydraPrimeHunter::new(hydra);
    hunter.calibrate(cal_range.0, cal_range.1, recall_target);
    let result = hunter.hunt(hunt_range.0, hunt_range.1);
    (hunter, result)
}

/// Sweep across recall targets to find the Pareto frontier.
pub fn recall_speedup_sweep(
    hydra_builder: fn() -> MultiHydra,
    cal_range: (u64, u64),
    hunt_range: (u64, u64),
) -> Vec<(f64, HuntResult)> {
    let targets = [0.50, 0.60, 0.70, 0.80, 0.85, 0.90, 0.95, 0.99, 1.00];
    let mut results = Vec::new();

    for &target in &targets {
        let hydra = hydra_builder();
        let (_, result) = full_hunt(hydra, cal_range, hunt_range, target);
        results.push((target, result));
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hunt_finds_primes() {
        let hydra = build_twin_hydra();
        let (_, result) = full_hunt(hydra, (2, 10_000), (10_000, 50_000), 0.95);
        assert!(!result.primes_found.is_empty());
        // Every prime we found must actually be prime
        for &p in &result.primes_found {
            assert!(is_prime_u64(p), "{} reported as prime but isn't", p);
        }
    }

    #[test]
    fn test_full_recall() {
        // At 100% target, we should find all primes (zero false negatives)
        let hydra = build_twin_hydra();
        let (_, result) = full_hunt(hydra, (2, 5_000), (5_000, 20_000), 1.00);
        assert_eq!(result.false_negatives, 0);
        assert!((result.recall - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_speedup_exists() {
        // At <100% recall, we should get speedup > 1
        let hydra = build_twin_hydra();
        let (_, result) = full_hunt(hydra, (2, 10_000), (10_000, 50_000), 0.80);
        assert!(result.speedup > 1.0, "Expected speedup > 1, got {}", result.speedup);
        assert!(result.candidates_skipped > 0);
    }
}
