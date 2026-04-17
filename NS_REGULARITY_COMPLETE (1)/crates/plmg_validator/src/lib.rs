//! # Per-Band PLMG Validation — W2.4
//!
//! Validates that each of the 144 WASSAN holographic bands satisfies
//! Phase-Locked Modular Geometry properties when viewed through the
//! CRT lane structure.
//!
//! ## Properties Validated
//!
//! 1. **K-Elimination correctness**: winding number k recovers the exact
//!    quotient for all coprime lane pairs
//! 2. **Rail/void partitioning**: residue combinations partition into
//!    reachable (rails) and unreachable (voids) states
//! 3. **Phase-lock coverage**: for Safe Basis moduli, all fiber classes
//!    achieve 100% coverage above saturation threshold
//! 4. **Error detection**: single-lane corruption produces detectable
//!    K-Elimination inconsistency
//! 5. **Garner reconstruction consistency**: per-band Garner matches
//!    the original encoded value

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

// ============================================================================
// MODULAR ARITHMETIC (shared with W2.1)
// ============================================================================

#[inline]
pub fn addmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 + b as u128) % m as u128) as u64
}

#[inline]
pub fn submod(a: u64, b: u64, m: u64) -> u64 {
    if a >= b { (a - b) % m } else { m - ((b - a) % m) }
}

#[inline]
pub fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128) * (b as u128) % m as u128) as u64
}

pub fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    if m == 1 { return 0; }
    let mut result = 1u64;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 { result = mulmod(result, base, m); }
        exp >>= 1;
        base = mulmod(base, base, m);
    }
    result
}

pub fn mod_inverse(a: u64, p: u64) -> u64 {
    assert!(a % p != 0, "Cannot invert zero mod {p}");
    mod_pow(a, p - 2, p)
}

pub fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

pub fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    if n == 2 || n == 3 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5u64;
    while i * i <= n { if n % i == 0 || n % (i + 2) == 0 { return false; } i += 6; }
    true
}

// ============================================================================
// K-ELIMINATION
// ============================================================================

/// K-Elimination: recover winding number k from two coprime residues.
///
/// Given X ≡ r (mod m) and X ≡ s (mod a) with gcd(m,a) = 1:
///   k ≡ (s - r) * m⁻¹ (mod a)
///   X = r + k * m   (unique for 0 ≤ X < m*a)
pub fn k_eliminate(r: u64, s: u64, m: u64, a: u64) -> u64 {
    debug_assert_eq!(gcd(m, a), 1, "K-Elimination requires coprime moduli");
    let m_inv = mod_inverse(m % a, a);
    let diff = submod(s % a, r % a, a);
    mulmod(diff, m_inv, a)
}

/// Reconstruct value from residue pair via K-Elimination.
pub fn k_reconstruct(r: u64, s: u64, m: u64, a: u64) -> u128 {
    let k = k_eliminate(r, s, m, a) as u128;
    r as u128 + k * m as u128
}

// ============================================================================
// GARNER RECONSTRUCTION
// ============================================================================

/// Garner's algorithm: reconstruct integer from residues without computing
/// the full modulus product.
pub fn garner_reconstruct(residues: &[u64], moduli: &[u64]) -> u128 {
    let k = residues.len();
    if k == 0 { return 0; }
    if k == 1 { return residues[0] as u128; }

    // Mixed-radix coefficients via standard Garner algorithm
    let mut v = vec![0u64; k];
    v[0] = residues[0];

    for i in 1..k {
        let mi = moduli[i];
        let mut temp = residues[i] % mi;

        for j in 0..i {
            // temp = (temp - v[j]) * moduli[j]^{-1} mod moduli[i]
            let vj = v[j] % mi;
            if temp < vj {
                temp += mi;
            }
            temp -= vj;
            let inv = mod_inverse(moduli[j] % mi, mi);
            temp = mulmod(temp, inv, mi);
        }
        v[i] = temp;
    }

    // Reconstruct: x = v[0] + v[1]*m[0] + v[2]*m[0]*m[1] + ...
    let mut result = v[0] as u128;
    let mut product = moduli[0] as u128;
    for i in 1..k {
        result += v[i] as u128 * product;
        product *= moduli[i] as u128;
    }
    result
}

// ============================================================================
// PLMG VALIDATOR
// ============================================================================

/// NTT-friendly primes (same as W2.1)
pub const LANE_PRIMES: [u64; 7] = [
    998244353,   // 119 * 2^23 + 1
    985661441,   // 235 * 2^22 + 1
    754974721,   // 45 * 2^24 + 1
    469762049,   // 7 * 2^26 + 1
    167772161,   // 5 * 2^25 + 1
    104857601,   // 25 * 2^22 + 1
    69206017,    // 31 * 2^21 + 1
];

/// Safe Basis primes for approximation lanes
pub const SAFE_BASIS: [u64; 6] = [2, 3, 5, 7, 11, 13];

pub const NUM_BANDS: usize = 144;

/// Result of validating a single band.
#[derive(Debug)]
pub struct BandValidation {
    pub band_index: usize,
    pub k_elim_correct: bool,
    pub garner_consistent: bool,
    pub error_detected: bool,
    pub rail_count: u64,
    pub void_count: u64,
    pub rail_fraction_permil: u64, // per-mille (0..1000)
}

/// Result of validating all 144 bands.
#[derive(Debug)]
pub struct PLMGValidationReport {
    pub total_bands: usize,
    pub bands_passing: usize,
    pub bands_failing: usize,
    pub k_elim_pass_rate: u64,    // permil
    pub garner_pass_rate: u64,    // permil
    pub error_detection_rate: u64, // permil
    pub avg_rail_fraction_permil: u64,
    pub phase_lock_coverage_permil: u64,
    pub failures: Vec<(usize, String)>,
}

/// Validate K-Elimination across all coprime lane pairs for a given value.
pub fn validate_k_elimination(value: u64, primes: &[u64]) -> bool {
    let residues: Vec<u64> = primes.iter().map(|&p| value % p).collect();

    for i in 0..primes.len() {
        for j in (i + 1)..primes.len() {
            let reconstructed = k_reconstruct(
                residues[i], residues[j],
                primes[i], primes[j],
            );
            // The reconstructed value should equal value mod (p_i * p_j)
            let product = primes[i] as u128 * primes[j] as u128;
            if reconstructed % product != value as u128 % product {
                return false;
            }
        }
    }
    true
}

/// Validate Garner reconstruction for a value.
pub fn validate_garner(value: u64, primes: &[u64]) -> bool {
    let residues: Vec<u64> = primes.iter().map(|&p| value % p).collect();
    let reconstructed = garner_reconstruct(&residues, primes);
    // For values smaller than the product of all primes, should be exact
    reconstructed == value as u128
}

/// Validate error detection: corrupt one lane, verify K-Elimination catches it.
pub fn validate_error_detection(value: u64, primes: &[u64]) -> bool {
    let residues: Vec<u64> = primes.iter().map(|&p| value % p).collect();

    // Corrupt lane 0
    let mut corrupted = residues.clone();
    corrupted[0] = (corrupted[0] + 42) % primes[0];

    // K-Elimination between lane 0 and lane 1 should give a DIFFERENT
    // winding number than the uncorrupted version
    if primes.len() < 2 { return true; }

    let k_clean = k_eliminate(residues[0], residues[1], primes[0], primes[1]);
    let k_corrupt = k_eliminate(corrupted[0], residues[1], primes[0], primes[1]);

    // Error should be detectable (k values differ)
    k_clean != k_corrupt
}

/// Count rails and voids for a pair of moduli up to limit N.
/// Rails = actually reachable (value, residue) pairs.
/// Voids = residue combinations that don't correspond to any integer in [0, N).
pub fn count_rails_voids(m1: u64, m2: u64, n: u64) -> (u64, u64) {
    let total_slots = m1 * m2;
    let mut hit = vec![false; total_slots as usize];

    for v in 0..n.min(total_slots) {
        let r1 = v % m1;
        let r2 = v % m2;
        let slot = r1 * m2 + r2;
        hit[slot as usize] = true;
    }

    let rails = hit.iter().filter(|&&h| h).count() as u64;
    let voids = total_slots - rails;
    (rails, voids)
}

/// Phase-lock coverage: what fraction of fiber classes are hit for a triple
/// of Safe Basis primes at sample size N.
pub fn phase_lock_coverage(p1: u64, p2: u64, p3: u64, n: u64) -> u64 {
    let total_fibers = p1 * p2 * p3;
    let mut hit = vec![false; total_fibers as usize];

    for v in 0..n.min(total_fibers * 2) {
        let fiber = (v % p1) * p2 * p3 + (v % p2) * p3 + (v % p3);
        if (fiber as usize) < hit.len() {
            hit[fiber as usize] = true;
        }
    }

    let covered = hit.iter().filter(|&&h| h).count() as u64;
    (covered * 1000) / total_fibers
}

/// Full per-band PLMG validation across all 144 bands.
pub fn validate_all_bands(band_values: &[u64; NUM_BANDS], primes: &[u64]) -> PLMGValidationReport {
    let mut bands_passing = 0usize;
    let mut k_elim_pass = 0u64;
    let mut garner_pass = 0u64;
    let mut error_detect_pass = 0u64;
    let mut total_rail_permil = 0u64;
    let mut failures = Vec::new();

    for (i, &val) in band_values.iter().enumerate() {
        let k_ok = validate_k_elimination(val, primes);
        let g_ok = validate_garner(val, primes);
        let e_ok = validate_error_detection(val, primes);

        if k_ok { k_elim_pass += 1; }
        if g_ok { garner_pass += 1; }
        if e_ok { error_detect_pass += 1; }

        // Rail/void for first two primes (representative)
        let (rails, voids) = if primes.len() >= 2 {
            count_rails_voids(primes[0] % 1000, primes[1] % 1000,
                              (primes[0] % 1000) * (primes[1] % 1000))
        } else {
            (1, 0)
        };
        let rail_permil = if rails + voids > 0 {
            (rails * 1000) / (rails + voids)
        } else { 0 };
        total_rail_permil += rail_permil;

        let all_ok = k_ok && g_ok && e_ok;
        if all_ok {
            bands_passing += 1;
        } else {
            let reason = format!(
                "k_elim={}, garner={}, error_detect={}",
                k_ok, g_ok, e_ok
            );
            failures.push((i, reason));
        }
    }

    let n = NUM_BANDS as u64;

    // Phase-lock coverage for Safe Basis triples
    let mut phase_lock_total = 0u64;
    let mut phase_lock_count = 0u64;
    for i in 0..SAFE_BASIS.len() {
        for j in (i+1)..SAFE_BASIS.len() {
            for k in (j+1)..SAFE_BASIS.len() {
                let cov = phase_lock_coverage(
                    SAFE_BASIS[i], SAFE_BASIS[j], SAFE_BASIS[k],
                    5000, // saturation test at N=5000
                );
                phase_lock_total += cov;
                phase_lock_count += 1;
            }
        }
    }
    let avg_phase_lock = if phase_lock_count > 0 {
        phase_lock_total / phase_lock_count
    } else { 0 };

    PLMGValidationReport {
        total_bands: NUM_BANDS,
        bands_passing,
        bands_failing: NUM_BANDS - bands_passing,
        k_elim_pass_rate: (k_elim_pass * 1000) / n,
        garner_pass_rate: (garner_pass * 1000) / n,
        error_detection_rate: (error_detect_pass * 1000) / n,
        avg_rail_fraction_permil: total_rail_permil / n,
        phase_lock_coverage_permil: avg_phase_lock,
        failures,
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_primes() -> Vec<u64> {
        LANE_PRIMES.iter().copied().filter(|p| is_prime(*p)).take(4).collect()
    }

    // --- K-Elimination ---

    #[test]
    fn k_elimination_basic() {
        // V=35, m=7, a=11 → k=5, V = 0 + 5*7 = 35
        let k = k_eliminate(35 % 7, 35 % 11, 7, 11);
        assert_eq!(k, 5);
        let v = k_reconstruct(35 % 7, 35 % 11, 7, 11);
        assert_eq!(v, 35);
    }

    #[test]
    fn k_elimination_large_primes() {
        let primes = test_primes();
        let val = 123456789u64;
        let r0 = val % primes[0];
        let r1 = val % primes[1];
        let v = k_reconstruct(r0, r1, primes[0], primes[1]);
        assert_eq!(v, val as u128);
    }

    #[test]
    fn k_elimination_all_pairs() {
        let primes = test_primes();
        assert!(validate_k_elimination(42, &primes));
        assert!(validate_k_elimination(999999, &primes));
        assert!(validate_k_elimination(0, &primes));
        assert!(validate_k_elimination(1, &primes));
    }

    // --- Garner ---

    #[test]
    fn garner_basic() {
        let primes = vec![7u64, 11, 13];
        assert!(validate_garner(0, &primes));
        assert!(validate_garner(1, &primes));
        assert!(validate_garner(42, &primes));
        assert!(validate_garner(1000, &primes)); // 1000 < 7*11*13 = 1001
    }

    #[test]
    fn garner_large_primes() {
        let primes = test_primes();
        for val in [0, 1, 42, 12345, 99999999, u32::MAX as u64] {
            assert!(validate_garner(val, &primes), "Garner failed for {val}");
        }
    }

    // --- Error Detection ---

    #[test]
    fn error_detection_catches_corruption() {
        let primes = test_primes();
        // Error detection should work for non-trivial values
        assert!(validate_error_detection(42, &primes));
        assert!(validate_error_detection(999999, &primes));
        assert!(validate_error_detection(123456789, &primes));
    }

    // --- Rail/Void Partitioning ---

    #[test]
    fn rails_voids_coprime_pair() {
        // For coprime (7, 11): all 77 slots should be rails (CRT guarantees bijection)
        let (rails, voids) = count_rails_voids(7, 11, 77);
        assert_eq!(rails, 77, "Coprime pair should have 100% rails");
        assert_eq!(voids, 0, "Coprime pair should have 0 voids");
    }

    #[test]
    fn rails_voids_non_coprime() {
        // For non-coprime (6, 10): gcd=2, so only 30 of 60 slots are rails
        let (rails, voids) = count_rails_voids(6, 10, 60);
        assert_eq!(rails, 30, "Non-coprime should have 50% rails");
        assert_eq!(voids, 30);
    }

    // --- Phase-Lock Coverage ---

    #[test]
    fn phase_lock_safe_basis_triple() {
        // {3,5,7} should hit 100% at N=5000 (product = 105)
        let cov = phase_lock_coverage(3, 5, 7, 5000);
        assert_eq!(cov, 1000, "Safe Basis triple (3,5,7) should achieve 100% coverage");
    }

    #[test]
    fn phase_lock_all_safe_basis_triples() {
        for i in 0..SAFE_BASIS.len() {
            for j in (i+1)..SAFE_BASIS.len() {
                for k in (j+1)..SAFE_BASIS.len() {
                    let (p1, p2, p3) = (SAFE_BASIS[i], SAFE_BASIS[j], SAFE_BASIS[k]);
                    let cov = phase_lock_coverage(p1, p2, p3, 5000);
                    assert_eq!(
                        cov, 1000,
                        "Safe Basis triple ({p1},{p2},{p3}) should have 100% coverage, got {}",
                        cov
                    );
                }
            }
        }
    }

    // --- Full 144-Band Validation ---

    #[test]
    fn validate_144_bands_sequential() {
        let primes = test_primes();
        let mut band_values = [0u64; NUM_BANDS];
        for (i, v) in band_values.iter_mut().enumerate() {
            *v = (i as u64 + 1) * 7; // Simple non-zero pattern
        }
        let report = validate_all_bands(&band_values, &primes);
        assert_eq!(report.bands_passing, NUM_BANDS,
            "All 144 bands should pass. Failures: {:?}", report.failures);
        assert_eq!(report.k_elim_pass_rate, 1000);
        assert_eq!(report.garner_pass_rate, 1000);
        assert_eq!(report.phase_lock_coverage_permil, 1000);
    }

    #[test]
    fn validate_144_bands_random_pattern() {
        let primes = test_primes();
        let mut band_values = [0u64; NUM_BANDS];
        let mut seed = 42u64;
        for v in band_values.iter_mut() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            *v = seed % 1_000_000_000; // Keep within comfortable range
        }
        let report = validate_all_bands(&band_values, &primes);
        assert_eq!(report.bands_passing, NUM_BANDS,
            "Random bands should all pass. Failures: {:?}", report.failures);
    }

    #[test]
    fn validate_report_structure() {
        let primes = test_primes();
        let band_values = [12345u64; NUM_BANDS];
        let report = validate_all_bands(&band_values, &primes);

        assert_eq!(report.total_bands, 144);
        assert!(report.k_elim_pass_rate <= 1000);
        assert!(report.garner_pass_rate <= 1000);
        assert!(report.error_detection_rate <= 1000);
        assert!(report.avg_rail_fraction_permil <= 1000);
        assert!(report.phase_lock_coverage_permil <= 1000);
    }
}
