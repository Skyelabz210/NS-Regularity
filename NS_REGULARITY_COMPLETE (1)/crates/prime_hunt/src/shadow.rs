//! Shadow (Lane 11) and Boundary (Lane 13) layer analysis.
//! Three-tier prime classification. Ramanujan spectral diagnostics.

use crate::crt::{ramanujan_sum_prime, factorize, gcd};

/// The Safe Basis — architectural constant throughout QMNF.
pub const SAFE_BASIS: [u128; 6] = [2, 3, 5, 7, 11, 13];

/// Ramanujan primes S_R = {5, 7, 11}
pub const S_R: [u128; 3] = [5, 7, 11];

/// SCALE parameter for the structural sieve
pub const SCALE: u128 = 10_000; // 2^4 × 5^4

/// Three-tier classification of a prime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimeTier {
    /// Tier I: Stability floor (2, 3 and their powers)
    Stability,
    /// Tier III: Accessible Ramanujan (5, 7, 11)
    AccessibleRamanujan,
    /// Tier 0: Boundary / Face-13 (13)
    Boundary,
    /// Tier 1: Turbulent (17, 19, 23, ...)
    Turbulent,
    /// Not prime
    Composite,
}

impl std::fmt::Display for PrimeTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimeTier::Stability => write!(f, "Tier-I (Stability)"),
            PrimeTier::AccessibleRamanujan => write!(f, "Tier-III (Ramanujan)"),
            PrimeTier::Boundary => write!(f, "Tier-0 (Boundary)"),
            PrimeTier::Turbulent => write!(f, "Tier-1 (Turbulent)"),
            PrimeTier::Composite => write!(f, "Composite"),
        }
    }
}

/// Classify a prime into tiers.
pub fn classify_prime(p: u128) -> PrimeTier {
    if !crate::crt::is_prime(p) && p != 4 && p != 8 && p != 9 && p != 16 && p != 27 {
        // Check if it's a prime power of 2 or 3
        let factors = factorize(p);
        if factors.len() == 1 && (factors[0].0 == 2 || factors[0].0 == 3) {
            return PrimeTier::Stability;
        }
        return PrimeTier::Composite;
    }
    match p {
        2 | 3 => PrimeTier::Stability,
        5 | 7 | 11 => PrimeTier::AccessibleRamanujan,
        13 => PrimeTier::Boundary,
        _ if crate::crt::is_prime(p) => PrimeTier::Turbulent,
        _ => PrimeTier::Composite,
    }
}

/// Shadow analysis state for a residue differential.
#[derive(Debug, Clone)]
pub struct ShadowState {
    /// Lane 11 residue of the differential
    pub shadow_residue: u128,
    /// Whether Lane 11 is zero (shadow bond active)
    pub active: bool,
    /// Count of near-negation lanes (diff ≡ p-1 or p-2 mod p)
    pub near_negation_count: u32,
    /// Count of half-turn lanes (diff ≡ (p+1)/2 mod p)
    pub half_turn_count: u32,
    /// Shadow strength: composite metric
    pub strength: u32,
}

impl ShadowState {
    /// Compute shadow state from a full residue differential.
    /// diff_full: differences mod [2, 3, 5, 7, 11] (first 5 of SAFE_BASIS)
    pub fn from_diff(diff_full: &[u128; 5]) -> Self {
        let shadow_residue = diff_full[4]; // Lane 11
        let active = shadow_residue == 0;

        let primes = [2u128, 3, 5, 7, 11];
        let mut near_negation_count = 0u32;
        let mut half_turn_count = 0u32;

        for (i, &p) in primes.iter().enumerate() {
            let d = diff_full[i];
            if d != 0 && (p - d) <= 2 {
                near_negation_count += 1;
            }
            if d == (p + 1) / 2 {
                half_turn_count += 1;
            }
        }

        let strength = (if active { 3 } else { 0 })
            + near_negation_count
            + half_turn_count;

        Self {
            shadow_residue,
            active,
            near_negation_count,
            half_turn_count,
            strength,
        }
    }
}

/// Boundary analysis state (Lane 13).
#[derive(Debug, Clone)]
pub struct BoundaryState {
    /// Lane 13 residue
    pub boundary_residue: u128,
    /// Whether Lane 13 is zero
    pub active: bool,
    /// Threshold classification
    pub threshold_type: ThresholdType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdType {
    Stable,
    ThresholdEvent,
    RegimeShift,
    StructuralFracture,
}

impl std::fmt::Display for ThresholdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThresholdType::Stable => write!(f, "stable"),
            ThresholdType::ThresholdEvent => write!(f, "threshold_event"),
            ThresholdType::RegimeShift => write!(f, "regime_shift"),
            ThresholdType::StructuralFracture => write!(f, "structural_fracture"),
        }
    }
}

impl BoundaryState {
    /// Compute from full 6-lane differential [2, 3, 5, 7, 11, 13].
    pub fn from_diff(diff_full: &[u128; 6]) -> Self {
        let boundary_residue = diff_full[5];
        let active = boundary_residue == 0;
        let safe_zeros = diff_full[..5].iter().filter(|&&d| d == 0).count();

        let threshold_type = if !active {
            ThresholdType::Stable
        } else if safe_zeros >= 3 {
            ThresholdType::StructuralFracture
        } else if safe_zeros >= 1 {
            ThresholdType::RegimeShift
        } else {
            ThresholdType::ThresholdEvent
        };

        Self { boundary_residue, active, threshold_type }
    }
}

/// Compute residue differential mod each of SAFE_BASIS.
pub fn residue_differential(a: u128, b: u128) -> [u128; 6] {
    let mut diff = [0u128; 6];
    for (i, &p) in SAFE_BASIS.iter().enumerate() {
        let ra = a % p;
        let rb = b % p;
        diff[i] = if rb >= ra { rb - ra } else { p - (ra - rb) };
    }
    diff
}

/// Spectral analysis: compute Ramanujan sum interaction scores
/// between two candidates across the S_R primes.
pub fn spectral_interaction(a: u128, b: u128) -> SpectralReport {
    let diff = if b > a { b - a } else { a - b };

    let mut scores = Vec::new();
    let mut total_power: i128 = 0;

    for &q in &S_R {
        let c = ramanujan_sum_prime(q, diff);
        let is_interaction = c == -1; // Both "off" — shadow-correlated
        scores.push(SpectralScore { prime: q, c_q: c, interaction_mode: is_interaction });
        total_power += c * c; // power = sum of squared Ramanujan sums
    }

    // Boundary contribution
    let c_13 = ramanujan_sum_prime(13, diff);
    scores.push(SpectralScore { prime: 13, c_q: c_13, interaction_mode: c_13 == -1 });

    SpectralReport { diff, scores, total_power }
}

#[derive(Debug)]
pub struct SpectralScore {
    pub prime: u128,
    pub c_q: i128,
    pub interaction_mode: bool,
}

#[derive(Debug)]
pub struct SpectralReport {
    pub diff: u128,
    pub scores: Vec<SpectralScore>,
    pub total_power: i128,
}

/// Check if a number is admissible (coprime to SCALE).
pub fn is_admissible(n: u128) -> bool {
    gcd(n, SCALE) == 1
}

/// 819-tier decomposition: (v % 9, v % 7, v % 13)
pub fn tier_819(v: u128) -> (u128, u128, u128) {
    (v % 9, v % 7, v % 13)
}

/// Find three-tier products: N = a² × b × c where b ∈ {5,7,11}, c = 13
pub fn find_three_tier_products(max_n: u128) -> Vec<ThreeTierProduct> {
    let mut results = Vec::new();
    let stability_bases: Vec<u128> = vec![2, 3, 4, 8, 9, 16, 27];
    let accessible = [5u128, 7, 11];
    let boundary = 13u128;

    for &a in &stability_bases {
        let a_sq = a * a;
        if a_sq >= max_n { continue; }
        for &b in &accessible {
            let n = a_sq * b * boundary;
            if n < max_n {
                results.push(ThreeTierProduct {
                    n,
                    stability_root: a,
                    accessible_prime: b,
                    boundary_prime: boundary,
                    is_819: n == 819,
                });
            }
        }
    }
    results.sort_by_key(|r| r.n);
    results
}

#[derive(Debug)]
pub struct ThreeTierProduct {
    pub n: u128,
    pub stability_root: u128,
    pub accessible_prime: u128,
    pub boundary_prime: u128,
    pub is_819: bool,
}

impl std::fmt::Display for ThreeTierProduct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}² × {} × {} {}",
            self.n, self.stability_root, self.accessible_prime,
            self.boundary_prime, if self.is_819 { "★ (819)" } else { "" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_prime() {
        assert_eq!(classify_prime(2), PrimeTier::Stability);
        assert_eq!(classify_prime(3), PrimeTier::Stability);
        assert_eq!(classify_prime(5), PrimeTier::AccessibleRamanujan);
        assert_eq!(classify_prime(7), PrimeTier::AccessibleRamanujan);
        assert_eq!(classify_prime(11), PrimeTier::AccessibleRamanujan);
        assert_eq!(classify_prime(13), PrimeTier::Boundary);
        assert_eq!(classify_prime(17), PrimeTier::Turbulent);
        assert_eq!(classify_prime(23), PrimeTier::Turbulent);
    }

    #[test]
    fn test_shadow_active() {
        // diff divisible by 11 → shadow active
        let diff = [0u128, 0, 0, 0, 0]; // All zero
        let ss = ShadowState::from_diff(&diff);
        assert!(ss.active);
        assert_eq!(ss.shadow_residue, 0);
    }

    #[test]
    fn test_boundary_structural_fracture() {
        // Lane 13 zero + 3 safe zeros → structural fracture
        let diff = [0u128, 0, 0, 1, 5, 0];
        let bs = BoundaryState::from_diff(&diff);
        assert!(bs.active);
        assert_eq!(bs.threshold_type, ThresholdType::StructuralFracture);
    }

    #[test]
    fn test_three_tier_products() {
        let products = find_three_tier_products(5000);
        let has_819 = products.iter().any(|p| p.is_819);
        assert!(has_819, "819 = 3²×7×13 must appear");
    }

    #[test]
    fn test_admissible() {
        assert!(is_admissible(3));
        assert!(is_admissible(7));
        assert!(!is_admissible(2));  // 2 | SCALE
        assert!(!is_admissible(5));  // 5 | SCALE
        assert!(!is_admissible(10)); // 10 | SCALE
    }

    #[test]
    fn test_tier_819() {
        // 819 = 3²×7×13, so 819 % 9 = 0, 819 % 7 = 0, 819 % 13 = 0
        assert_eq!(tier_819(819), (0, 0, 0));
    }
}
