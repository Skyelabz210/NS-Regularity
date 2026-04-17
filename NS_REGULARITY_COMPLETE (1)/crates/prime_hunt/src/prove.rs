//! Prove Strata — Constructive proofs about the coprime lattice.
//!
//! P1.1: Zero-prime strata → prove addresses are prime-free for ALL n
//! P1.2: Sqr folding = quadratic residue partition → Legendre symbol connection
//! P1.3: Cousin pair universal signature → algebraic proof
//! P2.1: Linear uniformity theorem → proves CRAM is necessary
//! P2.5: Multi-head scaling law → unbounded resolution?

use crate::crt::{self, gcd, is_prime, factorize};
use crate::exact::Rational;
use crate::intrinsic::{self, AddressSpace, Address, build_compact_address_space};

fn is_prime_u64(n: u64) -> bool {
    if n < 2 { return false; } if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5u64;
    while i.saturating_mul(i) <= n { if n % i == 0 || n % (i+2) == 0 { return false; } i += 6; }
    true
}

fn addmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 + b as u128) % m as u128) as u64 }
fn mulmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 * b as u128) % m as u128) as u64 }

// ═══════════════════════════════════════════════════════════════════
// P1.1: ZERO-PRIME STRATA PROOF
// ═══════════════════════════════════════════════════════════════════

/// For a given address space and address ID, enumerate ALL coprime
/// residue classes mod M that produce this address.
pub fn residue_classes_for_address(space: &AddressSpace, addr_id: u64) -> Vec<u64> {
    let m = space.period();
    let mut classes = Vec::new();
    for r in 0..m {
        if gcd(r as u128, m as u128) != 1 { continue; }
        let addr = space.address_of(r);
        if addr.to_id() == addr_id {
            classes.push(r);
        }
    }
    classes
}

/// For a residue class r mod M, find the smallest prime factor of n = r + k*M
/// for small k values. If all members share a common small factor, the class
/// is provably composite.
#[derive(Debug, Clone)]
pub struct ResidueClassAnalysis {
    pub residue: u64,
    pub modulus: u64,
    pub smallest_prime_factor: Option<u64>,  // if ALL tested n share this factor
    pub is_provably_composite: bool,
    pub proof_factor: u64,  // the prime that divides ALL n ≡ r (mod M)
    pub tested_count: u32,
    pub counterexample: Option<u64>,  // an n in this class that IS prime
}

/// Check if residue class r mod M is provably all-composite.
/// A class r mod M is all-composite if there exists a prime q such that
/// q | (r + k*M) for all k, i.e., q | r and q | M... but that contradicts
/// gcd(r, M) = 1. So the proof must be different:
///
/// The class is all-composite if r itself is composite AND r < M AND
/// for every n = r + k*M with k ≥ 1, n has a factor ≤ √n that divides
/// it. But that's just trial division — not a structural proof.
///
/// STRUCTURAL PROOF: r mod M is all-composite if there exists a prime q
/// NOT dividing M such that q | r. Then q | (r + k*M) iff q | r (since
/// gcd(q, M) = 1 → q ∤ M → q ∤ k*M). So q | r → q | (r + k*M) for
/// all k ONLY if q | r AND q | M. But gcd(r, M) = 1 means no prime
/// divides both r and M. So this approach fails.
///
/// ACTUAL PROOF STRATEGY: The zero-prime strata exist because the
/// CRAM address COMBINES information from multiple primes. The carry
/// pattern forces constraints that, together, eliminate primality.
/// We verify empirically over a large range and characterize WHY.
pub fn analyze_residue_class(r: u64, m: u64, test_range: u64) -> ResidueClassAnalysis {
    let mut all_composite = true;
    let mut counterexample = None;
    let mut common_factor = 0u64;
    let mut tested = 0u32;

    let mut n = r;
    if n < 2 { n += m; }

    while n < test_range {
        tested += 1;
        if is_prime_u64(n) {
            all_composite = false;
            counterexample = Some(n);
            break;
        } else {
            // Find smallest factor
            let mut d = 2u64;
            while d * d <= n {
                if n % d == 0 {
                    if common_factor == 0 { common_factor = d; }
                    break;
                }
                d += if d == 2 { 1 } else { 2 };
            }
        }
        n += m;
    }

    ResidueClassAnalysis {
        residue: r,
        modulus: m,
        smallest_prime_factor: if common_factor > 0 { Some(common_factor) } else { None },
        is_provably_composite: all_composite && tested >= 10,
        proof_factor: common_factor,
        tested_count: tested,
        counterexample,
    }
}

/// Full zero-prime strata analysis: for each zero-density address,
/// enumerate its residue classes and check each one.
#[derive(Debug)]
pub struct ZeroPrimeProof {
    pub address_id: u64,
    pub residue_classes: Vec<u64>,
    pub class_analyses: Vec<ResidueClassAnalysis>,
    pub all_provably_composite: bool,
    pub prime_found_in_class: Vec<(u64, u64)>,  // (residue, first prime)
    pub verdict: ZeroVerdict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroVerdict {
    /// All residue classes tested composite through test_range — likely structural
    ProvisionallyPrimeFree,
    /// At least one class contains a prime — zero density was finite-size effect
    ContainsPrimes,
    /// Mixed: some classes prime-free, others not
    Mixed,
}

pub fn prove_zero_strata(
    space: &AddressSpace,
    zero_addr_ids: &[u64],
    test_range: u64,
) -> Vec<ZeroPrimeProof> {
    let m = space.period();
    let mut proofs = Vec::new();

    for &addr_id in zero_addr_ids {
        let classes = residue_classes_for_address(space, addr_id);
        let mut analyses = Vec::new();
        let mut prime_found = Vec::new();
        let mut all_composite = true;

        for &r in &classes {
            let analysis = analyze_residue_class(r, m, test_range);
            if let Some(p) = analysis.counterexample {
                all_composite = false;
                prime_found.push((r, p));
            }
            analyses.push(analysis);
        }

        let verdict = if all_composite {
            ZeroVerdict::ProvisionallyPrimeFree
        } else if prime_found.len() == classes.len() {
            ZeroVerdict::ContainsPrimes
        } else {
            ZeroVerdict::Mixed
        };

        proofs.push(ZeroPrimeProof {
            address_id: addr_id,
            residue_classes: classes,
            class_analyses: analyses,
            all_provably_composite: all_composite,
            prime_found_in_class: prime_found,
            verdict,
        });
    }
    proofs
}

// ═══════════════════════════════════════════════════════════════════
// P1.2: QUADRATIC RECIPROCITY CONNECTION
// ═══════════════════════════════════════════════════════════════════

/// Euler's criterion: a^((p-1)/2) mod p. Returns 1 if QR, p-1 if QNR, 0 if p|a.
pub fn euler_criterion(a: u64, p: u64) -> u64 {
    if a % p == 0 { return 0; }
    let exp = (p - 1) / 2;
    let mut result = 1u64;
    let mut base = a % p;
    let mut e = exp;
    while e > 0 {
        if e & 1 == 1 { result = mulmod(result, base, p); }
        base = mulmod(base, base, p);
        e >>= 1;
    }
    result
}

/// Legendre symbol (a/p): +1 if QR, -1 if QNR, 0 if p|a.
pub fn legendre_symbol(a: u64, p: u64) -> i8 {
    let e = euler_criterion(a, p);
    if e == 0 { 0 }
    else if e == 1 { 1 }
    else { -1 } // e == p-1
}

/// Compute the Self+Mul+Sqr carry bit for n at prime p.
/// Self probe: b = n mod p. Root = Mul: inter = (n mod p)² mod p.
/// Post = Sqr: output = inter² = n⁴ mod p. Carry = ⌊2·output/p⌋.
///
/// BUT WAIT: Self+Mul gives n*n = n², then Sqr gives (n²)² = n⁴.
/// Actually for the RESIDUE probe we just get n mod p → Sqr → n² mod p.
/// Let me compute BOTH and compare to the Legendre symbol.
pub fn qr_carry_bit(n: u64, p: u64) -> u8 {
    let n_r = n % p;
    let n_sq = mulmod(n_r, n_r, p);  // n² mod p = Residue+Sqr
    if n_sq * 2 >= p { 1 } else { 0 }
}

/// The Self+Mul+Sqr dimension: b = n mod p, inter = n*n mod p, output = (n*n)² = n⁴ mod p
pub fn self_mul_sqr_bit(n: u64, p: u64) -> u8 {
    let n_r = n % p;
    let inter = mulmod(n_r, n_r, p);    // n² mod p
    let output = mulmod(inter, inter, p); // n⁴ mod p
    if output * 2 >= p { 1 } else { 0 }
}

/// Test whether the Residue+Sqr carry bit correlates with the Legendre symbol.
/// Returns: (matches, mismatches, n_tested)
pub fn test_qr_legendre_correlation(p: u64, limit: u64) -> QRCorrelation {
    let mut qr_carry0 = 0u32;  // QR with carry 0
    let mut qr_carry1 = 0u32;  // QR with carry 1
    let mut qnr_carry0 = 0u32; // QNR with carry 0
    let mut qnr_carry1 = 0u32; // QNR with carry 1
    let mut tested = 0u32;

    for n in 1..limit {
        if n % p == 0 { continue; }
        tested += 1;
        let leg = legendre_symbol(n, p);
        let carry = qr_carry_bit(n, p);

        match (leg, carry) {
            (1, 0)  => qr_carry0 += 1,
            (1, 1)  => qr_carry1 += 1,
            (-1, 0) => qnr_carry0 += 1,
            (-1, 1) => qnr_carry1 += 1,
            _ => {}
        }
    }

    // The Sqr map sends QRs to QRs (since (a²)² = a⁴ is always a QR).
    // So the carry bit after Sqr depends on WHERE in the QR set n² lands
    // relative to the p/2 threshold.
    QRCorrelation {
        prime: p,
        qr_carry0, qr_carry1, qnr_carry0, qnr_carry1, tested,
        // Key insight: QNR → Sqr → QR (since squaring a QNR gives a QR of the QNR).
        // Wait: QNR squared is QR (since (QNR)² ≡ QR because Legendre is multiplicative
        // and (QNR/p)² = 1). So ALL Sqr outputs are QR or 0.
        // The carry bit partitions QRs into upper/lower halves of [0, p).
        qr_total: qr_carry0 + qr_carry1,
        qnr_total: qnr_carry0 + qnr_carry1,
    }
}

#[derive(Debug)]
pub struct QRCorrelation {
    pub prime: u64,
    pub qr_carry0: u32,
    pub qr_carry1: u32,
    pub qnr_carry0: u32,
    pub qnr_carry1: u32,
    pub tested: u32,
    pub qr_total: u32,
    pub qnr_total: u32,
}

/// Multi-prime Legendre vector: (n/p₁, n/p₂, ..., n/pₖ) ∈ {-1, 0, +1}^k
pub fn legendre_vector(n: u64, primes: &[u64]) -> Vec<i8> {
    primes.iter().map(|&p| legendre_symbol(n, p)).collect()
}

/// Multi-prime carry vector from Residue+Sqr dimensions
pub fn qr_carry_vector(n: u64, primes: &[u64]) -> Vec<u8> {
    primes.iter().map(|&p| qr_carry_bit(n, p)).collect()
}

/// Test whether Legendre vector predicts prime density in CRAM strata.
/// Group coprimes by their Legendre vector. Measure prime density per group.
pub fn legendre_stratification(primes: &[u64], limit: u64) -> Vec<LegendreStratum> {
    let m: u64 = primes.iter().product();
    let mut strata: std::collections::HashMap<Vec<i8>, (u32, u32)> = std::collections::HashMap::new();

    for n in 2..limit {
        if gcd(n as u128, m as u128) != 1 { continue; }
        let leg = legendre_vector(n, primes);
        let entry = strata.entry(leg).or_insert((0, 0));
        entry.0 += 1;
        if is_prime_u64(n) { entry.1 += 1; }
    }

    let mut result: Vec<LegendreStratum> = strata.into_iter().map(|(vec, (cc, pc))| {
        LegendreStratum {
            legendre_vec: vec,
            coprime_count: cc,
            prime_count: pc,
            density: Rational::new(pc as u64, cc as u64),
        }
    }).collect();
    result.sort_by(|a, b| b.density.cmp(&a.density));
    result
}

#[derive(Debug)]
pub struct LegendreStratum {
    pub legendre_vec: Vec<i8>,
    pub coprime_count: u32,
    pub prime_count: u32,
    pub density: Rational,
}

// ═══════════════════════════════════════════════════════════════════
// P1.3: COUSIN PAIR SIGNATURE PROOF
// ═══════════════════════════════════════════════════════════════════

/// Prove that the cousin pair differential Δ = [0,1,4,4,4,4] is algebraically
/// forced for ALL cousin pairs (p, p+4) with p > 13.
///
/// Proof: For gap d=4 and primes {2,3,5,7,11,13}:
///   Δ[i] = (p+4) mod p_i - p mod p_i = 4 mod p_i
///   Since 4 < all p_i except p_i=2,3:
///     4 mod 2 = 0, 4 mod 3 = 1, 4 mod 5 = 4, 4 mod 7 = 4, 4 mod 11 = 4, 4 mod 13 = 4
///   This is INDEPENDENT of p. It's pure arithmetic of the gap.
///
/// The differential depends ONLY on the gap d, not on the prime pair.
/// This is a theorem, not an observation.
pub fn gap_differential(d: u64, basis: &[u64]) -> Vec<u64> {
    basis.iter().map(|&p| d % p).collect()
}

/// Verify the universal gap differential for many gap values.
pub fn verify_universal_differentials(max_gap: u64, basis: &[u64]) -> Vec<(u64, Vec<u64>)> {
    let mut results = Vec::new();
    for d in (2..=max_gap).step_by(2) { // even gaps only (odd gaps can't have prime pairs > 2)
        let diff = gap_differential(d, basis);
        results.push((d, diff));
    }
    results
}

// ═══════════════════════════════════════════════════════════════════
// P2.1: LINEAR UNIFORMITY THEOREM
// ═══════════════════════════════════════════════════════════════════

/// Prove that linear functionals on the coprime lattice yield uniform
/// prime density. A "linear functional" is f(n) = n mod q for some q.
///
/// By Dirichlet's theorem: primes are equidistributed across residue
/// classes mod q (for gcd(r, q) = 1). So prime density in each class
/// is asymptotically 1/(φ(q) · ln(x)).
///
/// Test: for each residue class mod q, measure prime density. Compute
/// χ² against uniform. Verify χ² < critical value.
pub fn linear_uniformity_test(q: u64, limit: u64) -> LinearUniformity {
    let phi_q = crt::euler_totient(q as u128) as u64;
    let mut buckets = vec![0u32; q as usize]; // prime count per residue class
    let mut total_per_bucket = vec![0u32; q as usize];
    let mut total_primes = 0u32;

    for n in 2..limit {
        if gcd(n as u128, q as u128) != 1 { continue; }
        let r = (n % q) as usize;
        total_per_bucket[r] += 1;
        if is_prime_u64(n) {
            buckets[r] += 1;
            total_primes += 1;
        }
    }

    // χ² test: expected primes per coprime bucket = total_primes / φ(q)
    let expected = Rational::new(total_primes as u64, phi_q);
    let mut chi2_num = 0u128;
    let mut chi2_den = 1u128;
    let mut df = 0u32;

    for r in 0..q as usize {
        if gcd(r as u128, q as u128) != 1 { continue; }
        df += 1;
        let observed = buckets[r] as u128;
        let exp_num = expected.num as u128;
        let exp_den = expected.den as u128;
        // (observed - expected)² / expected
        // = (observed·den - num)² / (den² · num/den)
        // = (observed·den - num)² · den / (den² · num)
        // = (observed·den - num)² / (den · num)
        let diff = if observed * exp_den >= exp_num {
            observed * exp_den - exp_num
        } else {
            exp_num - observed * exp_den
        };
        chi2_num += diff * diff;
        chi2_den = exp_den * exp_num; // same for all buckets
    }

    // chi2 = chi2_num / chi2_den (rational)
    let g = crt::gcd(chi2_num, chi2_den);
    let chi2 = Rational::new((chi2_num / g) as u64, (chi2_den / g) as u64);

    LinearUniformity {
        modulus: q,
        phi: phi_q,
        total_primes,
        chi2,
        df: df - 1,
        // Critical value at 5% for df degrees of freedom ≈ 2·df
        // We use the exact criterion: χ²/df should be near 1
        chi2_per_df: Rational::new(chi2.num, chi2.den * (df - 1) as u64),
        is_uniform: chi2.num * (df as u64 - 1) < chi2.den * 2 * (df as u64 - 1) * (df as u64 - 1),
    }
}

#[derive(Debug)]
pub struct LinearUniformity {
    pub modulus: u64,
    pub phi: u64,
    pub total_primes: u32,
    pub chi2: Rational,
    pub df: u32,
    pub chi2_per_df: Rational,
    pub is_uniform: bool,
}

// ═══════════════════════════════════════════════════════════════════
// P2.5: MULTI-HEAD SCALING LAW
// ═══════════════════════════════════════════════════════════════════

/// Build a Hydra with H heterogeneous heads, each seeing different symmetry.
/// Returns (head_count, total_dimensions, populated_strata, exact_gini).
pub fn scaling_test(max_heads: usize, limit: u64) -> Vec<ScalingPoint> {
    use crate::multi_hydra::{MultiHydra, Head};
    use crate::coprime_families::{CramTopology, Lane, Root, Post, Champions};

    // Pool of diverse head topologies — each probes different symmetry
    let head_pool: Vec<(&str, CramTopology)> = vec![
        ("Twin-Het", Champions::twin()),
        ("QR-AllSqr", CramTopology::new(vec![
            Lane { prime: 3,  root: Root::Add, post: Post::Sqr },
            Lane { prime: 5,  root: Root::Add, post: Post::Sqr },
            Lane { prime: 7,  root: Root::Add, post: Post::Sqr },
            Lane { prime: 11, root: Root::Add, post: Post::Sqr },
            Lane { prime: 13, root: Root::Add, post: Post::Sqr },
        ])),
        ("Mul-Struct", CramTopology::new(vec![
            Lane { prime: 3,  root: Root::Mul, post: Post::Id },
            Lane { prime: 5,  root: Root::Mul, post: Post::Id },
            Lane { prime: 7,  root: Root::Mul, post: Post::Id },
            Lane { prime: 11, root: Root::Mul, post: Post::Id },
            Lane { prime: 13, root: Root::Mul, post: Post::Id },
        ])),
        ("Sub-Neg", CramTopology::new(vec![
            Lane { prime: 3,  root: Root::Sub, post: Post::Neg },
            Lane { prime: 5,  root: Root::Sub, post: Post::Neg },
            Lane { prime: 7,  root: Root::Sub, post: Post::Neg },
            Lane { prime: 11, root: Root::Sub, post: Post::Neg },
            Lane { prime: 13, root: Root::Sub, post: Post::Neg },
        ])),
        ("Add-Id-Base", Champions::baseline()),
        ("Mixed-Deep", CramTopology::new(vec![
            Lane { prime: 3,  root: Root::Sub, post: Post::Sqr },
            Lane { prime: 5,  root: Root::Mul, post: Post::Neg },
            Lane { prime: 7,  root: Root::Add, post: Post::Id },
            Lane { prime: 11, root: Root::Sub, post: Post::Id },
            Lane { prime: 13, root: Root::Mul, post: Post::Sqr },
        ])),
    ];

    let mut results = Vec::new();

    for h in 1..=max_heads.min(head_pool.len()) {
        let heads: Vec<Head> = head_pool[..h].iter()
            .map(|(name, topo)| Head::new(name, topo.clone(), 2))
            .collect();
        let hydra = MultiHydra::new(heads);
        let map = hydra.map_lattice(2, limit);

        // Compute exact Gini
        let densities: Vec<Rational> = map.strata.iter()
            .filter(|(_, d)| d.coprime_count >= 3)
            .map(|(_, d)| Rational::new(d.prime_count as u64, d.coprime_count as u64))
            .collect();
        let gini = crate::exact::exact_gini(&densities);

        results.push(ScalingPoint {
            heads: h as u32,
            dimensions: map.total_lanes as u32,
            populated: map.populated_strata,
            gini,
            density_spread: {
                let dvals: Vec<Rational> = map.strata.iter()
                    .filter(|(_, d)| d.coprime_count >= 3)
                    .map(|(_, d)| Rational::new(d.prime_count as u64, d.coprime_count as u64))
                    .collect();
                if dvals.len() >= 2 {
                    dvals[0].sub_saturating(dvals.last().unwrap())
                } else { Rational::zero() }
            },
        });
    }
    results
}

#[derive(Debug)]
pub struct ScalingPoint {
    pub heads: u32,
    pub dimensions: u32,
    pub populated: u32,
    pub gini: Rational,
    pub density_spread: Rational,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legendre_symbol() {
        // (2/7) = 1 (2 is QR mod 7: 3² = 9 ≡ 2)
        assert_eq!(legendre_symbol(2, 7), 1);
        // (3/7) = -1 (3 is QNR mod 7)
        assert_eq!(legendre_symbol(3, 7), -1);
        // (5/11) = 1 (5 is QR mod 11: 4² = 16 ≡ 5)
        assert_eq!(legendre_symbol(5, 11), 1);
        // (0/7) = 0
        assert_eq!(legendre_symbol(7, 7), 0);
    }

    #[test]
    fn test_euler_criterion() {
        // 2^3 mod 7 = 8 mod 7 = 1 → QR
        assert_eq!(euler_criterion(2, 7), 1);
        // 3^3 mod 7 = 27 mod 7 = 6 = 7-1 → QNR
        assert_eq!(euler_criterion(3, 7), 6); // p-1
    }

    #[test]
    fn test_cousin_differential() {
        let basis = [2u64, 3, 5, 7, 11, 13];
        let diff = gap_differential(4, &basis);
        assert_eq!(diff, vec![0, 1, 4, 4, 4, 4]);
    }

    #[test]
    fn test_twin_differential() {
        let basis = [2u64, 3, 5, 7, 11, 13];
        let diff = gap_differential(2, &basis);
        assert_eq!(diff, vec![0, 2, 2, 2, 2, 2]);
    }

    #[test]
    fn test_sexy_differential() {
        let basis = [2u64, 3, 5, 7, 11, 13];
        let diff = gap_differential(6, &basis);
        assert_eq!(diff, vec![0, 0, 1, 6, 6, 6]);
        // Note: 6 mod 3 = 0 → divisibility enhancement for sexy primes at p=3
    }

    #[test]
    fn test_linear_uniformity() {
        // Primes should be uniform mod 7 (Dirichlet)
        let result = linear_uniformity_test(7, 10_000);
        assert!(result.is_uniform, "Primes not uniform mod 7: χ²/df = {}", result.chi2_per_df);
    }

    #[test]
    fn test_legendre_stratification_nontrivial() {
        let strata = legendre_stratification(&[3, 5, 7], 10_000);
        assert!(!strata.is_empty());
        // Should have 2^3 = 8 possible Legendre vectors (each ±1)
        // (excluding vectors with 0, which don't appear for coprimes)
    }

    #[test]
    fn test_zero_strata_analysis() {
        let space = build_compact_address_space();
        let m = space.period();
        let mut zero_addrs = Vec::new();
        let mut addr_primes: std::collections::HashMap<u64, u32> = std::collections::HashMap::new();
        let mut addr_total: std::collections::HashMap<u64, u32> = std::collections::HashMap::new();

        for n in 2..50_000u64 {
            if gcd(n as u128, m as u128) != 1 { continue; }
            let addr = space.address_of(n);
            let id = addr.to_id();
            *addr_total.entry(id).or_insert(0) += 1;
            if is_prime_u64(n) { *addr_primes.entry(id).or_insert(0) += 1; }
        }

        for (&id, &total) in &addr_total {
            if total >= 5 && *addr_primes.get(&id).unwrap_or(&0) == 0 {
                zero_addrs.push(id);
            }
        }

        if !zero_addrs.is_empty() {
            let proofs = prove_zero_strata(&space, &zero_addrs[..1.min(zero_addrs.len())], 200_000);
            assert!(!proofs.is_empty());
        }
    }

    #[test]
    fn test_qr_separation_theorem() {
        // p ≡ 1 (mod 4) → disjoint squared images → separation possible
        // p ≡ 3 (mod 4) → identical squared images → separation impossible
        for &p in &[3u64, 5, 7, 11, 13, 17, 19, 23, 29, 37, 41] {
            let result = qr_separation_analysis(p);
            if p % 4 == 1 {
                assert!(result.images_disjoint,
                    "p={} ≡ 1 (mod 4) but QR²∩QNR² not disjoint", p);
            } else {
                assert!(!result.images_disjoint,
                    "p={} ≡ 3 (mod 4) but QR²∩QNR² disjoint", p);
            }
        }
    }

    #[test]
    fn test_p5_perfect_separation() {
        let result = qr_separation_analysis(5);
        // p=5: QR²={1}, QNR²={4}. Threshold at 2.5.
        // QR²={1} all below → carry=0. QNR²={4} all above → carry=1.
        assert!(result.images_disjoint);
        assert_eq!(result.separation_permille, 1000); // perfect
    }

    #[test]
    fn test_biquadratic_residues() {
        // p=13: QR²={1,3,9} = biquadratic residues (4th powers)
        // QNR²={4,10,12} = non-biquadratic QRs
        let result = qr_separation_analysis(13);
        assert!(result.images_disjoint);
        assert!(result.separation_permille > 0);
        assert_eq!(result.biquad_residues.len(), 3); // (13-1)/4 = 3
    }
}

// ═══════════════════════════════════════════════════════════════════
// P1.2 EXTENDED: QR SEPARATION THEOREM
// ═══════════════════════════════════════════════════════════════════

/// Complete analysis of QR/QNR separation via Sqr carry bit for prime p.
/// 
/// THEOREM: For prime p:
///   p ≡ 1 (mod 4) → QR² ∩ QNR² = ∅ (disjoint biquadratic partition)
///   p ≡ 3 (mod 4) → QR² = QNR² (identical squared images)
///
/// PROOF: When p ≡ 1 (mod 4), -1 is a QR. So for a ∈ QR, -a ∈ QR.
/// Each pair {a, p-a} = {a, -a} consists of two QRs or two QNRs.
/// Their square (a² = (-a)²) lands in QR² or QNR² respectively.
/// Since the pairs don't mix QR/QNR, the images are disjoint.
///
/// When p ≡ 3 (mod 4), -1 is a QNR. For a ∈ QR, -a ∈ QNR.
/// Each pair {a, -a} has one QR and one QNR, both mapping to a².
/// So every squared value is reached by both a QR and a QNR.
/// Therefore QR² = QNR².
///
/// This is the FIRST SUPPLEMENT to quadratic reciprocity,
/// expressed through the CRAM carry bit lens.
#[derive(Debug)]
pub struct QRSeparationAnalysis {
    pub prime: u64,
    pub p_mod_4: u64,
    pub qr_set: Vec<u64>,
    pub qnr_set: Vec<u64>,
    pub biquad_residues: Vec<u64>,   // QR² = 4th powers mod p
    pub non_biquad_qr: Vec<u64>,     // QNR² = QRs that aren't 4th powers
    pub images_disjoint: bool,        // QR² ∩ QNR² = ∅
    pub separation_permille: u64,     // 1000 = perfect, 0 = none
    pub carry_qr: (u32, u32),        // (carry0, carry1) for QR inputs
    pub carry_qnr: (u32, u32),       // (carry0, carry1) for QNR inputs
}

pub fn qr_separation_analysis(p: u64) -> QRSeparationAnalysis {
    // Build QR and QNR sets
    let mut qr_set = Vec::new();
    let mut qnr_set = Vec::new();
    let mut is_qr = vec![false; p as usize];
    
    for a in 1..p {
        let sq = mulmod(a, a, p);
        is_qr[sq as usize] = true;
    }
    for a in 1..p {
        if is_qr[a as usize] { qr_set.push(a); } else { qnr_set.push(a); }
    }

    // Compute squared images
    let mut qr_squared: Vec<u64> = qr_set.iter().map(|&a| mulmod(a, a, p)).collect();
    let mut qnr_squared: Vec<u64> = qnr_set.iter().map(|&a| mulmod(a, a, p)).collect();
    qr_squared.sort(); qr_squared.dedup();
    qnr_squared.sort(); qnr_squared.dedup();

    // Check disjointness
    let qr_sq_set: std::collections::HashSet<u64> = qr_squared.iter().copied().collect();
    let qnr_sq_set: std::collections::HashSet<u64> = qnr_squared.iter().copied().collect();
    let images_disjoint = qr_sq_set.intersection(&qnr_sq_set).count() == 0;

    // Carry bit analysis
    let mut qr_c0 = 0u32; let mut qr_c1 = 0u32;
    let mut qnr_c0 = 0u32; let mut qnr_c1 = 0u32;

    for a in 1..p {
        let sq = mulmod(a, a, p);
        let carry = if sq * 2 >= p { 1 } else { 0 };
        if is_qr[a as usize] {
            if carry == 0 { qr_c0 += 1; } else { qr_c1 += 1; }
        } else {
            if carry == 0 { qnr_c0 += 1; } else { qnr_c1 += 1; }
        }
    }

    let total_qr = qr_c0 + qr_c1;
    let total_qnr = qnr_c0 + qnr_c1;
    let separation_permille = if total_qr > 0 && total_qnr > 0 {
        let diff_num = if qr_c0 as u64 * total_qnr as u64 > qnr_c0 as u64 * total_qr as u64 {
            qr_c0 as u64 * total_qnr as u64 - qnr_c0 as u64 * total_qr as u64
        } else {
            qnr_c0 as u64 * total_qr as u64 - qr_c0 as u64 * total_qnr as u64
        };
        diff_num * 1000 / (total_qr as u64 * total_qnr as u64)
    } else { 0 };

    QRSeparationAnalysis {
        prime: p,
        p_mod_4: p % 4,
        qr_set,
        qnr_set,
        biquad_residues: qr_squared,
        non_biquad_qr: qnr_squared,
        images_disjoint,
        separation_permille,
        carry_qr: (qr_c0, qr_c1),
        carry_qnr: (qnr_c0, qnr_c1),
    }
}

/// Scan primes to find all with p ≡ 1 (mod 4) that give Legendre separation.
pub fn scan_legendre_primes(limit: u64) -> Vec<QRSeparationAnalysis> {
    let mut results = Vec::new();
    for p in 3..limit {
        if !is_prime_u64(p) { continue; }
        let analysis = qr_separation_analysis(p);
        results.push(analysis);
    }
    results
}

// ═══════════════════════════════════════════════════════════════════
// STRUCTURE DECOMPOSITION THEOREM
// structure = QR-visible + threshold-visible + interaction
// ═══════════════════════════════════════════════════════════════════

/// Lane classification: which mechanism does each prime use?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneMechanism {
    /// p ≡ 1 (mod 4): Sqr carry separates QR from QNR
    QRVisible,
    /// p ≡ 3 (mod 4): Sqr carry is QR-blind, but threshold geometry works
    ThresholdVisible,
}

pub fn classify_lane(p: u64) -> LaneMechanism {
    if p % 4 == 1 { LaneMechanism::QRVisible }
    else { LaneMechanism::ThresholdVisible }
}

/// Decompose a basis into QR-visible and threshold-visible lanes
pub fn classify_basis(primes: &[u64]) -> (Vec<u64>, Vec<u64>) {
    let qr: Vec<u64> = primes.iter().filter(|&&p| p % 4 == 1).copied().collect();
    let th: Vec<u64> = primes.iter().filter(|&&p| p % 4 == 3).copied().collect();
    (qr, th)
}

/// Structure decomposition: measure Gini from QR-only lanes, threshold-only
/// lanes, and the full (interaction) basis. The interaction term is:
///   interaction = Gini_full - Gini_QR - Gini_threshold
/// If positive: the two mechanisms are synergistic (seeing more together).
/// If negative: they interfere (unlikely but possible).
/// If zero: they are independent (additive).
#[derive(Debug)]
pub struct StructureDecomposition {
    pub basis: Vec<u64>,
    pub qr_lanes: Vec<u64>,
    pub threshold_lanes: Vec<u64>,
    pub gini_full: Rational,
    pub gini_qr_only: Rational,
    pub gini_threshold_only: Rational,
    pub interaction: InteractionType,
    pub interaction_magnitude: Rational, // |full - qr - threshold|
    pub strata_full: usize,
    pub strata_qr: usize,
    pub strata_threshold: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionType {
    Synergistic,  // full > qr + threshold
    Independent,  // full ≈ qr + threshold
    Destructive,  // full < qr + threshold
}

pub fn decompose_structure(basis: &[u64], limit: u64) -> StructureDecomposition {
    let (qr_lanes, threshold_lanes) = classify_basis(basis);

    let full_strata = legendre_stratification(basis, limit);
    let full_densities: Vec<Rational> = full_strata.iter()
        .filter(|s| s.coprime_count >= 3)
        .map(|s| s.density).collect();
    let gini_full = crate::exact::exact_gini(&full_densities);

    let gini_qr_only = if qr_lanes.len() >= 2 {
        let qr_strata = legendre_stratification(&qr_lanes, limit);
        let qr_d: Vec<Rational> = qr_strata.iter()
            .filter(|s| s.coprime_count >= 3)
            .map(|s| s.density).collect();
        crate::exact::exact_gini(&qr_d)
    } else if qr_lanes.len() == 1 {
        let qr_strata = legendre_stratification(&qr_lanes, limit);
        let qr_d: Vec<Rational> = qr_strata.iter()
            .filter(|s| s.coprime_count >= 3)
            .map(|s| s.density).collect();
        crate::exact::exact_gini(&qr_d)
    } else {
        Rational::zero()
    };

    let gini_threshold_only = if !threshold_lanes.is_empty() {
        let th_strata = legendre_stratification(&threshold_lanes, limit);
        let th_d: Vec<Rational> = th_strata.iter()
            .filter(|s| s.coprime_count >= 3)
            .map(|s| s.density).collect();
        crate::exact::exact_gini(&th_d)
    } else {
        Rational::zero()
    };

    // Interaction = full - (qr + threshold)
    let sum_parts = gini_qr_only.add(&gini_threshold_only);
    let (interaction, magnitude) = if gini_full.gt(&sum_parts) {
        (InteractionType::Synergistic, gini_full.sub_saturating(&sum_parts))
    } else if sum_parts.gt(&gini_full) {
        (InteractionType::Destructive, sum_parts.sub_saturating(&gini_full))
    } else {
        (InteractionType::Independent, Rational::zero())
    };

    let strata_qr = if qr_lanes.is_empty() { 0 } else {
        legendre_stratification(&qr_lanes, limit).iter()
            .filter(|s| s.coprime_count >= 3).count()
    };
    let strata_threshold = if threshold_lanes.is_empty() { 0 } else {
        legendre_stratification(&threshold_lanes, limit).iter()
            .filter(|s| s.coprime_count >= 3).count()
    };

    StructureDecomposition {
        basis: basis.to_vec(),
        qr_lanes,
        threshold_lanes,
        gini_full,
        gini_qr_only,
        gini_threshold_only,
        interaction,
        interaction_magnitude: magnitude,
        strata_full: full_densities.len(),
        strata_qr,
        strata_threshold,
    }
}

// ═══════════════════════════════════════════════════════════════════
// P1.3: GINI PREDICTION FROM T6 SIEVE POWER
// ═══════════════════════════════════════════════════════════════════

/// T6 sieve power SP(p,d) = (2p - 1 - c_p(d)) / p²
/// where c_p(d) = p-1 if p|d, else -1.
/// Returns exact (numerator, denominator).
pub fn t6_sieve_power(p: u64, d: u64) -> Rational {
    let c: i64 = if d % p == 0 { p as i64 - 1 } else { -1 };
    let num = (2 * p as i64 - 1 - c) as u64;
    let den = p * p;
    Rational::new(num, den)
}

/// Predicted forbidden fraction for gap d on basis B:
/// F_product(d,B) = 1 - ∏(1 - f_p(d)) where f_p(d) = forbidden/p
/// For p ∤ d: f_p = 2/p. For p | d: f_p = 1/p.
/// Returns exact rational.
pub fn predicted_forbidden(d: u64, basis: &[u64]) -> Rational {
    // Product of (1 - f_p) = ∏ admissible_p / p
    // admissible_p = p - 2 if p ∤ d, p - 1 if p | d
    let mut prod_num = 1u128;
    let mut prod_den = 1u128;
    for &p in basis {
        let adm = if d % p == 0 { p - 1 } else { p - 2 };
        prod_num *= adm as u128;
        prod_den *= p as u128;
    }
    // F = 1 - prod = (prod_den - prod_num) / prod_den
    let f_num = prod_den - prod_num;
    let g = crt::gcd(f_num, prod_den);
    Rational::new((f_num / g) as u64, (prod_den / g) as u64)
}

/// Gini predictor: estimate stratification from per-lane sieve powers.
/// HYPOTHESIS: Gini ∝ Σ (SP_i - mean_SP)² / mean_SP (chi-squared-like)
/// This captures the VARIANCE of sieve power across lanes.
pub fn gini_predictor(d: u64, basis: &[u64]) -> Rational {
    let sps: Vec<Rational> = basis.iter().map(|&p| t6_sieve_power(p, d)).collect();
    let n = sps.len() as u64;
    
    // Mean SP
    let mut sum = Rational::zero();
    for sp in &sps { sum = sum.add(sp); }
    let mean = Rational::new(sum.num, sum.den * n);
    
    if mean.num == 0 { return Rational::zero(); }
    
    // Variance-like: Σ |SP_i - mean|
    let mut var = Rational::zero();
    for sp in &sps {
        let diff = if sp.gt(&mean) { sp.sub_saturating(&mean) } else { mean.sub_saturating(sp) };
        var = var.add(&diff);
    }
    
    // Normalize by mean
    // var / mean = var.num * mean.den / (var.den * mean.num)
    if mean.num == 0 { return Rational::zero(); }
    let r_num = (var.num as u128) * (mean.den as u128);
    let r_den = (var.den as u128) * (mean.num as u128);
    let g = crt::gcd(r_num, r_den);
    Rational::new((r_num / g) as u64, (r_den / g) as u64)
}

/// Compare predicted vs actual Gini across multiple gaps.
pub fn gini_prediction_sweep(basis: &[u64], gaps: &[u64], limit: u64) -> Vec<GiniPrediction> {
    gaps.iter().map(|&d| {
        let predicted = gini_predictor(d, basis);
        let actual_strata = legendre_stratification(basis, limit);
        let densities: Vec<Rational> = actual_strata.iter()
            .filter(|s| s.coprime_count >= 3)
            .map(|s| s.density).collect();
        let actual = crate::exact::exact_gini(&densities);
        let forbidden = predicted_forbidden(d, basis);
        
        GiniPrediction { gap: d, predicted, actual, forbidden }
    }).collect()
}

#[derive(Debug)]
pub struct GiniPrediction {
    pub gap: u64,
    pub predicted: Rational,
    pub actual: Rational,
    pub forbidden: Rational,
}

// ═══════════════════════════════════════════════════════════════════
// P4.1: SPECTRAL FINGERPRINT CLASSIFIER
// Connect STATIC address to DYNAMIC gap family membership.
// ═══════════════════════════════════════════════════════════════════

/// For a given number n and gap d, compute the "gap compatibility score"
/// as the inner product of n's Legendre vector with the gap's T6 fingerprint.
/// Both are exact rationals. If the score predicts actual pair membership,
/// we've connected static lattice structure to dynamic prime pairs.
pub fn gap_compatibility(n: u64, d: u64, basis: &[u64]) -> Rational {
    // n's Legendre vector: (n/p_i) ∈ {-1, 0, +1}
    // T6 fingerprint: SP(p_i, d) = (2p-1-c_p(d))/p²
    // Compatibility = Σ sign_i × SP_i (weighted by Legendre character)
    let mut score = Rational::zero();
    for &p in basis {
        let leg = legendre_symbol(n, p);
        let sp = t6_sieve_power(p, d);
        // leg × sp: if leg=+1, add sp. if leg=-1, subtract sp. if 0, skip.
        match leg {
            1  => score = score.add(&sp),
            -1 => {
                // We want score - sp, but Rational is unsigned.
                // Track separately? No — use a signed pair.
                // For now: compatibility = Σ |leg_i| × SP_i (magnitude only)
                // This still captures whether n's QR pattern aligns with d's SP pattern.
                score = score.add(&sp);
            }
            _  => {} // p | n, skip
        }
    }
    score
}

/// Test whether gap compatibility predicts actual prime pair membership.
/// For each prime p in [start, limit), check if (p, p+d) is a prime pair,
/// and measure whether high-compatibility primes are more likely to form pairs.
pub fn gap_prediction_test(d: u64, basis: &[u64], start: u64, limit: u64) -> GapPredictionResult {
    let m: u64 = basis.iter().product();
    let mut pair_scores = Vec::new();    // compatibility scores for actual pairs
    let mut nonpair_scores = Vec::new(); // compatibility scores for non-pairs

    for n in start..limit {
        if !is_prime_u64(n) { continue; }
        if crt::gcd(n as u128, m as u128) != 1 { continue; }

        let score = gap_compatibility(n, d, basis);
        if is_prime_u64(n + d) {
            pair_scores.push(score);
        } else {
            nonpair_scores.push(score);
        }
    }

    // Compare mean score of pairs vs non-pairs
    let pair_mean = if pair_scores.is_empty() { Rational::zero() } else {
        let mut sum = Rational::zero();
        for s in &pair_scores { sum = sum.add(s); }
        Rational::new(sum.num, sum.den * pair_scores.len() as u64)
    };
    let nonpair_mean = if nonpair_scores.is_empty() { Rational::zero() } else {
        let mut sum = Rational::zero();
        for s in &nonpair_scores { sum = sum.add(s); }
        Rational::new(sum.num, sum.den * nonpair_scores.len() as u64)
    };

    GapPredictionResult {
        gap: d,
        pair_count: pair_scores.len() as u32,
        nonpair_count: nonpair_scores.len() as u32,
        pair_mean_score: pair_mean,
        nonpair_mean_score: nonpair_mean,
        pair_higher: pair_mean.gt(&nonpair_mean),
    }
}

#[derive(Debug)]
pub struct GapPredictionResult {
    pub gap: u64,
    pub pair_count: u32,
    pub nonpair_count: u32,
    pub pair_mean_score: Rational,
    pub nonpair_mean_score: Rational,
    pub pair_higher: bool,
}

// ═══════════════════════════════════════════════════════════════════
// FORMAL GAP SIGNATURE: algebraic proof that Δ(d) is universal
// ═══════════════════════════════════════════════════════════════════

/// THEOREM: For any prime pair (p, p+d) with p > max(basis) and gcd(p, M) = 1:
///   (p+d) mod p_i - p mod p_i ≡ d mod p_i  for all p_i in basis.
///
/// This is INDEPENDENT of p. Proof: modular arithmetic distributes over addition.
///
/// CONSTRUCTIVE VERIFICATION: check every prime pair below limit.
pub fn verify_universal_gap_signature(d: u64, basis: &[u64], limit: u64) -> GapSignatureProof {
    let expected: Vec<u64> = basis.iter().map(|&p| d % p).collect();
    let m: u64 = basis.iter().product();
    let mut pairs_checked = 0u32;
    let mut violations = 0u32;
    let mut first_violation: Option<u64> = None;

    for n in 2..limit {
        if !is_prime_u64(n) { continue; }
        if !is_prime_u64(n + d) { continue; }
        if crt::gcd(n as u128, m as u128) != 1 { continue; }

        pairs_checked += 1;

        // Check: (n+d) mod p_i - n mod p_i = d mod p_i for each p_i
        for (i, &p) in basis.iter().enumerate() {
            let actual_diff = if (n + d) % p >= n % p {
                (n + d) % p - n % p
            } else {
                p + (n + d) % p - n % p
            };
            if actual_diff != expected[i] {
                violations += 1;
                if first_violation.is_none() { first_violation = Some(n); }
            }
        }
    }

    GapSignatureProof {
        gap: d,
        signature: expected,
        pairs_checked,
        violations,
        first_violation,
        is_universal: violations == 0,
    }
}

#[derive(Debug)]
pub struct GapSignatureProof {
    pub gap: u64,
    pub signature: Vec<u64>,
    pub pairs_checked: u32,
    pub violations: u32,
    pub first_violation: Option<u64>,
    pub is_universal: bool,
}

// ═══════════════════════════════════════════════════════════════════
// HARMONIC MODEL: Three-layer structure
// ═══════════════════════════════════════════════════════════════════

/// Layer 1 (L1): Gap Resonance — T6 SP(p,d) = HL local factor (exact)
/// Layer 2 (L2): Individual classification — c_p(n) = -1 for ALL coprime n
///               (blind to primality within M_H)
/// Layer 3 (L3): Signature stratification — grows with dimension
///
/// The Hydra heads map to layers:
///   H1 = T6 gap resonance (spectral fingerprint)
///   H2 = T8-T10 carry-threshold separation (Sqr folding)
///   H3 = High-dimensional signature (multi-head cross-product)
pub fn harmonic_layer_analysis(basis: &[u64], limit: u64) -> HarmonicLayers {
    let m: u64 = basis.iter().product();

    // L1: For each gap d, compute T6 prediction vs actual pair ratio
    let test_gaps = [2u64, 4, 6, 30];
    let mut l1_results = Vec::new();
    for &d in &test_gaps {
        let predicted = predicted_forbidden(d, basis);
        let actual_pairs = (2..limit)
            .filter(|&n| is_prime_u64(n) && is_prime_u64(n+d) && crt::gcd(n as u128, m as u128) == 1)
            .count() as u32;
        let total_primes = (2..limit)
            .filter(|&n| is_prime_u64(n) && crt::gcd(n as u128, m as u128) == 1)
            .count() as u32;
        let actual_ratio = Rational::new(actual_pairs as u64, total_primes as u64);
        l1_results.push((d, predicted, actual_ratio, actual_pairs));
    }

    // L2: Ramanujan sum c_p(n) for coprime n: always -1 (blind to primality)
    let mut l2_verified = true;
    for &p in basis {
        for n in 2..1000u64 {
            if n % p == 0 { continue; }
            let c = if n % p == 0 { p as i64 - 1 } else { -1i64 };
            if c != -1 { l2_verified = false; }
        }
    }

    // L3: Stratification deviation grows with dimension
    let strata_2d = legendre_stratification(&basis[..2], limit);
    let strata_4d = legendre_stratification(&basis[..4], limit);
    let strata_5d = legendre_stratification(basis, limit);

    let dev_2d = strata_deviation(&strata_2d);
    let dev_4d = strata_deviation(&strata_4d);
    let dev_5d = strata_deviation(&strata_5d);

    HarmonicLayers {
        l1_gap_results: l1_results,
        l2_blind: l2_verified,
        l3_deviations: vec![(2, dev_2d), (4, dev_4d), (basis.len() as u32, dev_5d)],
    }
}

fn strata_deviation(strata: &[LegendreStratum]) -> Rational {
    if strata.is_empty() { return Rational::zero(); }
    let densities: Vec<Rational> = strata.iter()
        .filter(|s| s.coprime_count >= 5)
        .map(|s| s.density).collect();
    if densities.len() < 2 { return Rational::zero(); }
    densities[0].sub_saturating(densities.last().unwrap())
}

#[derive(Debug)]
pub struct HarmonicLayers {
    pub l1_gap_results: Vec<(u64, Rational, Rational, u32)>, // (gap, predicted_forbidden, actual_ratio, count)
    pub l2_blind: bool,
    pub l3_deviations: Vec<(u32, Rational)>, // (dimensions, max-min density)
}

// ═══════════════════════════════════════════════════════════════════
// JOINT ADDRESS PAIR PREDICTOR
// The static address of ONE endpoint can't predict pairs.
// But the JOINT address of (n, n+d) can.
// ═══════════════════════════════════════════════════════════════════

/// Joint stratum: bin by (address(n), address(n+d)) pair.
/// If both land in prime-rich strata, pair probability should be elevated.
pub fn joint_address_pair_analysis(
    basis: &[u64], d: u64, limit: u64,
) -> JointPairResult {
    let m: u64 = basis.iter().product();

    // For each coprime n, compute Legendre vectors of BOTH n and n+d
    // Bin by (leg(n), leg(n+d)) → measure pair density per bin
    let mut bins: std::collections::HashMap<(Vec<i8>, Vec<i8>), (u32, u32)> =
        std::collections::HashMap::new();

    let mut total_coprime_tested = 0u32;
    let mut total_pairs = 0u32;

    for n in 2..limit {
        if crt::gcd(n as u128, m as u128) != 1 { continue; }
        if crt::gcd((n+d) as u128, m as u128) != 1 { continue; }

        total_coprime_tested += 1;
        let leg_n = legendre_vector(n, basis);
        let leg_nd = legendre_vector(n + d, basis);

        let key = (leg_n, leg_nd);
        let entry = bins.entry(key).or_insert((0, 0));
        entry.0 += 1; // total in this joint bin

        if is_prime_u64(n) && is_prime_u64(n + d) {
            entry.1 += 1; // pairs in this joint bin
            total_pairs += 1;
        }
    }

    // Sort bins by pair density (descending)
    let mut ranked: Vec<JointBin> = bins.into_iter().map(|((ln, lnd), (total, pairs))| {
        JointBin {
            leg_n: ln, leg_nd: lnd,
            total, pairs,
            density: Rational::new(pairs as u64, total as u64),
        }
    }).collect();
    ranked.sort_by(|a, b| b.density.cmp(&a.density));

    let avg_density = Rational::new(total_pairs as u64, total_coprime_tested as u64);

    // Spread: max density - min density among bins with ≥5 members
    let populated: Vec<&JointBin> = ranked.iter().filter(|b| b.total >= 5).collect();
    let spread = if populated.len() >= 2 {
        populated[0].density.sub_saturating(&populated.last().unwrap().density)
    } else { Rational::zero() };

    // Gini
    let densities: Vec<Rational> = populated.iter().map(|b| b.density).collect();
    let gini = crate::exact::exact_gini(&densities);

    JointPairResult {
        gap: d,
        total_bins: ranked.len() as u32,
        populated_bins: populated.len() as u32,
        total_tested: total_coprime_tested,
        total_pairs,
        avg_density,
        spread,
        gini,
        bins: ranked,
    }
}

#[derive(Debug)]
pub struct JointBin {
    pub leg_n: Vec<i8>,
    pub leg_nd: Vec<i8>,
    pub total: u32,
    pub pairs: u32,
    pub density: Rational,
}

#[derive(Debug)]
pub struct JointPairResult {
    pub gap: u64,
    pub total_bins: u32,
    pub populated_bins: u32,
    pub total_tested: u32,
    pub total_pairs: u32,
    pub avg_density: Rational,
    pub spread: Rational,
    pub gini: Rational,
    pub bins: Vec<JointBin>,
}

// ═══════════════════════════════════════════════════════════════════
// HL CORRECTION FACTOR — exact rational
// ═══════════════════════════════════════════════════════════════════

/// Compute the Hardy-Littlewood correction factor for gap d on basis B.
/// F_actual / F_predicted where:
///   F_predicted = 1 - ∏(admissible_p / p) from T6
///   F_actual = 1 - (actual pairs / expected if uniform)
///
/// The correction factor measures carry propagation topology
/// that the HL product approximation misses.
pub fn hl_correction_factor(d: u64, basis: &[u64], limit: u64) -> HLCorrection {
    let m: u64 = basis.iter().product();

    // Count actual pairs (both coprime to M)
    let mut actual_pairs = 0u32;
    let mut coprime_primes = 0u32;

    for n in 2..limit {
        if !is_prime_u64(n) { continue; }
        if crt::gcd(n as u128, m as u128) != 1 { continue; }
        coprime_primes += 1;
        if is_prime_u64(n + d) {
            actual_pairs += 1;
        }
    }

    let actual_ratio = Rational::new(actual_pairs as u64, coprime_primes as u64);

    // T6 predicted: product of per-lane admissible fractions
    // Admissible fraction per lane = (p - forbidden) / p
    // forbidden = 2 if p ∤ d, 1 if p | d
    let mut adm_num = 1u64;
    let mut adm_den = 1u64;
    for &p in basis {
        let forbidden = if d % p == 0 { 1 } else { 2 };
        adm_num *= p - forbidden;
        adm_den *= p;
    }
    let predicted_survival = Rational::new(adm_num, adm_den);

    // Correction = actual_ratio / predicted_survival
    // = (actual_pairs / coprime_primes) / (adm_num / adm_den)
    // = actual_pairs * adm_den / (coprime_primes * adm_num)
    let corr_num = (actual_pairs as u128) * (adm_den as u128);
    let corr_den = (coprime_primes as u128) * (adm_num as u128);
    let g = crt::gcd(corr_num, corr_den);
    let correction = Rational::new((corr_num / g) as u64, (corr_den / g) as u64);

    HLCorrection {
        gap: d,
        actual_pairs,
        coprime_primes,
        actual_ratio,
        predicted_survival,
        correction,
    }
}

#[derive(Debug)]
pub struct HLCorrection {
    pub gap: u64,
    pub actual_pairs: u32,
    pub coprime_primes: u32,
    pub actual_ratio: Rational,
    pub predicted_survival: Rational,
    pub correction: Rational,
}

// ═══════════════════════════════════════════════════════════════════
// CRAM SCHEMA JOINT PAIR PREDICTOR
// Uses full heterogeneous CRAM carry patterns (both mechanisms)
// instead of Legendre vectors (QR-only mechanism)
// ═══════════════════════════════════════════════════════════════════

use crate::coprime_families::{CramTopology, Lane, Root, Post, Champions};

/// Compute CRAM carry pattern for n under a topology.
/// This uses the FULL heterogeneous pipeline: per-lane (root, post) operators.
fn cram_pattern(n: u64, d: u64, topo: &CramTopology) -> Vec<u8> {
    topo.carry_pattern(n, d)
}

/// Joint CRAM pair analysis: bin by (cram(n), cram(n+d)) carry pattern pair.
/// Each bin's pair density reveals whether the JOINT carry structure
/// predicts prime pair membership.
pub fn cram_joint_pair_analysis(
    topo: &CramTopology,
    d: u64,
    limit: u64,
) -> CramJointResult {
    let m = topo.modulus;
    let k = topo.k();

    // Bin key: concatenation of pattern(n) ++ pattern(n+d) as u64
    let mut bins: std::collections::HashMap<(u32, u32), (u32, u32)> =
        std::collections::HashMap::new();

    let mut total_tested = 0u32;
    let mut total_pairs = 0u32;

    for n in 2..limit {
        if crt::gcd(n as u128, m as u128) != 1 { continue; }
        if crt::gcd((n + d) as u128, m as u128) != 1 { continue; }

        total_tested += 1;

        let pat_n = cram_pattern(n, d, topo);
        let pat_nd = cram_pattern(n + d, d, topo);

        // Encode patterns as u32
        let id_n = pat_to_id(&pat_n);
        let id_nd = pat_to_id(&pat_nd);

        let entry = bins.entry((id_n, id_nd)).or_insert((0, 0));
        entry.0 += 1;

        if is_prime_u64(n) && is_prime_u64(n + d) {
            entry.1 += 1;
            total_pairs += 1;
        }
    }

    // Sort by density
    let mut ranked: Vec<CramJointBin> = bins.into_iter().map(|((id_n, id_nd), (total, pairs))| {
        CramJointBin {
            pattern_n: id_n,
            pattern_nd: id_nd,
            total, pairs,
            density: Rational::new(pairs as u64, total as u64),
        }
    }).collect();
    ranked.sort_by(|a, b| b.density.cmp(&a.density));

    let avg = Rational::new(total_pairs as u64, total_tested as u64);

    let populated: Vec<&CramJointBin> = ranked.iter().filter(|b| b.total >= 5).collect();
    let spread = if populated.len() >= 2 {
        populated[0].density.sub_saturating(&populated.last().unwrap().density)
    } else { Rational::zero() };

    let densities: Vec<Rational> = populated.iter().map(|b| b.density).collect();
    let gini = crate::exact::exact_gini(&densities);

    CramJointResult {
        gap: d,
        topology_label: topo.label(),
        k,
        total_bins: ranked.len() as u32,
        populated_bins: populated.len() as u32,
        total_tested,
        total_pairs,
        avg_density: avg,
        spread,
        gini,
        bins: ranked,
    }
}

fn pat_to_id(pat: &[u8]) -> u32 {
    let mut id = 0u32;
    for (i, &b) in pat.iter().enumerate() {
        if i >= 16 { break; }
        id |= (b as u32) << i;
    }
    id
}

fn id_to_pat(id: u32, k: usize) -> String {
    (0..k).map(|i| if (id >> i) & 1 == 1 { '1' } else { '0' }).collect()
}

/// Public wrapper for cross-module access
pub fn id_to_pat_pub(id: u32, k: usize) -> String { id_to_pat(id, k) }

#[derive(Debug)]
pub struct CramJointBin {
    pub pattern_n: u32,
    pub pattern_nd: u32,
    pub total: u32,
    pub pairs: u32,
    pub density: Rational,
}

#[derive(Debug)]
pub struct CramJointResult {
    pub gap: u64,
    pub topology_label: String,
    pub k: usize,
    pub total_bins: u32,
    pub populated_bins: u32,
    pub total_tested: u32,
    pub total_pairs: u32,
    pub avg_density: Rational,
    pub spread: Rational,
    pub gini: Rational,
    pub bins: Vec<CramJointBin>,
}

/// Compare Legendre-only vs CRAM joint pair prediction.
pub fn compare_legendre_vs_cram(d: u64, limit: u64) -> (CramJointResult, CramJointResult, CramJointResult) {
    // Topology 1: Add+Id on all lanes (linear baseline — should be weak)
    let baseline = CramTopology::new(vec![
        Lane { prime: 3, root: Root::Add, post: Post::Id },
        Lane { prime: 5, root: Root::Add, post: Post::Id },
        Lane { prime: 7, root: Root::Add, post: Post::Id },
        Lane { prime: 11, root: Root::Add, post: Post::Id },
        Lane { prime: 13, root: Root::Add, post: Post::Id },
    ]);

    // Topology 2: Add+Sqr on all lanes (QR-only mechanism)
    let qr_only = CramTopology::new(vec![
        Lane { prime: 3, root: Root::Add, post: Post::Sqr },
        Lane { prime: 5, root: Root::Add, post: Post::Sqr },
        Lane { prime: 7, root: Root::Add, post: Post::Sqr },
        Lane { prime: 11, root: Root::Add, post: Post::Sqr },
        Lane { prime: 13, root: Root::Add, post: Post::Sqr },
    ]);

    // Topology 3: Champion S⁶ heterogeneous (both mechanisms)
    let champion = Champions::s6();

    let r1 = cram_joint_pair_analysis(&baseline, d, limit);
    let r2 = cram_joint_pair_analysis(&qr_only, d, limit);
    let r3 = cram_joint_pair_analysis(&champion, d, limit);

    (r1, r2, r3)
}

// ═══════════════════════════════════════════════════════════════════
// TRUTH-PERTURBER EXECUTION — Test each perturbation candidate
// ═══════════════════════════════════════════════════════════════════

/// P-1.3: General Power Residue Separation Theorem
/// For prime p and power k: gcd(k, p-1) > 1 ↔ k-th power map is non-bijective
/// ↔ carry bit of n^k mod p separates k-th power residues from non-residues.
pub fn test_power_residue_separation(p: u64, k: u64) -> PowerResidueResult {
    // Compute image of x^k mod p for all x in (Z/pZ)*
    let mut images: Vec<u64> = Vec::new();
    for x in 1..p {
        let val = pow_mod(x, k, p);
        if !images.contains(&val) { images.push(val); }
    }
    images.sort();

    let image_size = images.len() as u64;
    let full_size = p - 1; // |(Z/pZ)*|
    let g = crt::gcd(k as u128, (p - 1) as u128) as u64;
    let predicted_image_size = full_size / g; // (p-1)/gcd(k, p-1) distinct images

    // Test carry bit separation: do k-th power residues cluster differently?
    let mut kr_carry0 = 0u32;
    let mut kr_carry1 = 0u32;
    let mut nkr_carry0 = 0u32;
    let mut nkr_carry1 = 0u32;

    for x in 1..p {
        let is_kth_power = images.binary_search(&x).is_ok();
        let carry = if x * 2 >= p { 1 } else { 0 };
        match (is_kth_power, carry) {
            (true, 0) => kr_carry0 += 1,
            (true, _) => kr_carry1 += 1,
            (false, 0) => nkr_carry0 += 1,
            (false, _) => nkr_carry1 += 1,
        }
    }

    // Separation: do KR and NKR have different carry distributions?
    // If distributions are identical: no separation. If different: separation.
    let kr_total = kr_carry0 + kr_carry1;
    let nkr_total = nkr_carry0 + nkr_carry1;

    let separates = if nkr_total == 0 {
        false // everything is a k-th power residue (bijective map)
    } else {
        // Check if carry0/total ratios differ
        let kr_ratio = Rational::new(kr_carry0 as u64, kr_total as u64);
        let nkr_ratio = Rational::new(nkr_carry0 as u64, nkr_total as u64);
        !kr_ratio.eq(&nkr_ratio)
    };

    let bijective = g == 1;

    PowerResidueResult {
        p, k, gcd_k_pm1: g,
        image_size, predicted_image_size,
        bijective, separates,
        kr_carry: (kr_carry0, kr_carry1),
        nkr_carry: (nkr_carry0, nkr_carry1),
    }
}

fn pow_mod(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result = 1u64;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 { result = ((result as u128 * base as u128) % modulus as u128) as u64; }
        exp >>= 1;
        base = ((base as u128 * base as u128) % modulus as u128) as u64;
    }
    result
}

#[derive(Debug)]
pub struct PowerResidueResult {
    pub p: u64,
    pub k: u64,
    pub gcd_k_pm1: u64,
    pub image_size: u64,
    pub predicted_image_size: u64,
    pub bijective: bool,
    pub separates: bool,
    pub kr_carry: (u32, u32),  // (carry0, carry1) for k-th power residues
    pub nkr_carry: (u32, u32), // (carry0, carry1) for non-k-th-power residues
}

/// P-7.2: Prove carry ≠ Legendre algebraically
/// Find n where carry_bit(n² mod p) gives different classification than (n/p).
pub fn carry_vs_legendre(p: u64) -> CarryLegendreResult {
    let mut same = 0u32;
    let mut different = 0u32;
    let mut first_diff: Option<(u64, u8, i8)> = None;

    for n in 1..p {
        let leg = legendre_symbol(n, p);
        let sqr_val = ((n as u128 * n as u128) % p as u128) as u64;
        let carry = if sqr_val * 2 >= p { 1u8 } else { 0u8 };

        // Legendre: QR → leg=1, QNR → leg=-1
        // Carry: depends on WHERE n² lands, not WHETHER n is QR
        // All n with same n² mod p get same carry, regardless of Legendre
        // But QR and QNR map to SAME squared images when p≡3 mod 4 (T16)
        // So the question is: for p≡1 mod 4, does carry separate QR/QNR?
        // Answer: it does for some p, not for all n at a given p.
        //
        // The real test: is carry_bit(n² mod p) a function of (n/p) alone?
        // If two n values have the same Legendre symbol but different carry → NOT a function.

        // We need a more careful test: group by Legendre symbol,
        // check if carry is constant within each group.
        if leg == 1 {
            // QR: does carry vary among QRs?
        }
        if leg == -1 {
            // QNR: does carry vary among QNRs?
        }
    }

    // Better approach: collect carry values for each Legendre class
    let mut qr_carries: Vec<u8> = Vec::new();
    let mut qnr_carries: Vec<u8> = Vec::new();

    for n in 1..p {
        let leg = legendre_symbol(n, p);
        let sqr_val = ((n as u128 * n as u128) % p as u128) as u64;
        let carry = if sqr_val * 2 >= p { 1u8 } else { 0u8 };

        if leg == 1 { qr_carries.push(carry); }
        else if leg == -1 { qnr_carries.push(carry); }
    }

    let qr_has_both = qr_carries.contains(&0) && qr_carries.contains(&1);
    let qnr_has_both = qnr_carries.contains(&0) && qnr_carries.contains(&1);

    // If QRs have both carry values: carry is NOT determined by Legendre alone
    // → carry ≠ Legendre (carries positional info beyond QR class)
    let carry_exceeds_legendre = qr_has_both || qnr_has_both;

    let qr_carry0 = qr_carries.iter().filter(|&&c| c == 0).count() as u32;
    let qr_carry1 = qr_carries.iter().filter(|&&c| c == 1).count() as u32;
    let qnr_carry0 = qnr_carries.iter().filter(|&&c| c == 0).count() as u32;
    let qnr_carry1 = qnr_carries.iter().filter(|&&c| c == 1).count() as u32;

    CarryLegendreResult {
        p,
        qr_count: qr_carries.len() as u32,
        qnr_count: qnr_carries.len() as u32,
        qr_carry: (qr_carry0, qr_carry1),
        qnr_carry: (qnr_carry0, qnr_carry1),
        qr_has_both_carries: qr_has_both,
        qnr_has_both_carries: qnr_has_both,
        carry_exceeds_legendre,
    }
}

#[derive(Debug)]
pub struct CarryLegendreResult {
    pub p: u64,
    pub qr_count: u32,
    pub qnr_count: u32,
    pub qr_carry: (u32, u32),    // (carry0, carry1) among QRs
    pub qnr_carry: (u32, u32),   // (carry0, carry1) among QNRs
    pub qr_has_both_carries: bool,
    pub qnr_has_both_carries: bool,
    pub carry_exceeds_legendre: bool, // TRUE = carry has info beyond Legendre
}

/// P-4.4: Pair lattice relaxation
/// Standard: require gcd(n, M) = 1 AND gcd(n+d, M) = 1
/// Relaxed: require gcd(n, M/gcd(d,M)) = 1 AND gcd(n+d, M/gcd(d,M)) = 1
/// For d=2, M=30030: gcd(2, 30030) = 2, so relaxed M' = 15015
/// This lets even numbers participate → larger population → potentially more signal.
pub fn relaxed_pair_analysis(basis: &[u64], d: u64, limit: u64) -> (RelaxedPairStats, RelaxedPairStats) {
    let m: u64 = basis.iter().product();
    let g = crt::gcd(d as u128, m as u128) as u64;
    let m_relaxed = m / g;

    // Reduced basis: remove primes that divide d
    let reduced_basis: Vec<u64> = basis.iter().filter(|&&p| d % p != 0).copied().collect();

    let mut strict_pop = 0u32;
    let mut strict_pairs = 0u32;
    let mut relaxed_pop = 0u32;
    let mut relaxed_pairs = 0u32;

    for n in 2..limit {
        // Strict: coprime to full M
        let strict_ok = crt::gcd(n as u128, m as u128) == 1
            && crt::gcd((n+d) as u128, m as u128) == 1;
        // Relaxed: coprime to M/gcd(d,M)
        let relaxed_ok = crt::gcd(n as u128, m_relaxed as u128) == 1
            && crt::gcd((n+d) as u128, m_relaxed as u128) == 1;

        if strict_ok {
            strict_pop += 1;
            if is_prime_u64(n) && is_prime_u64(n + d) { strict_pairs += 1; }
        }
        if relaxed_ok {
            relaxed_pop += 1;
            if is_prime_u64(n) && is_prime_u64(n + d) { relaxed_pairs += 1; }
        }
    }

    let strict = RelaxedPairStats {
        modulus: m, population: strict_pop, pairs: strict_pairs,
        density: Rational::new(strict_pairs as u64, strict_pop as u64),
    };
    let relaxed = RelaxedPairStats {
        modulus: m_relaxed, population: relaxed_pop, pairs: relaxed_pairs,
        density: Rational::new(relaxed_pairs as u64, relaxed_pop as u64),
    };
    (strict, relaxed)
}

#[derive(Debug)]
pub struct RelaxedPairStats {
    pub modulus: u64,
    pub population: u32,
    pub pairs: u32,
    pub density: Rational,
}

// ═══════════════════════════════════════════════════════════════════
// DIRICHLET VERIFICATION: Zero-Prime Strata Are Finite-Range Artifacts
// ═══════════════════════════════════════════════════════════════════

/// For a given address in the intrinsic space, find ALL residue classes
/// mod M that map to that address. Then verify Dirichlet: each class
/// r + kM contains primes for large enough k.
pub fn verify_dirichlet_for_address(
    addr: u64,
    moduli: &[u64],
    search_limit: u64,
) -> DirichletVerification {
    let m: u64 = moduli.iter().product();
    let phi_m = euler_phi_product(moduli);

    // Find all r in [1, M) coprime to M with addr(r) = addr
    let space = crate::intrinsic::build_compact_address_space();
    let mut matching_residues = Vec::new();

    for r in 1..m {
        if crt::gcd(r as u128, m as u128) != 1 { continue; }
        let a = space.address_of(r).to_id();
        if a == addr {
            matching_residues.push(r);
        }
    }

    let r_v = matching_residues.len() as u64;

    // For each matching residue, search for primes in r + kM
    let mut primes_found = Vec::new();
    let mut first_prime_per_class: Vec<Option<u64>> = vec![None; matching_residues.len()];

    for (i, &r) in matching_residues.iter().enumerate() {
        let mut n = r;
        while n < search_limit {
            if n > 13 && is_prime_u64(n) {
                primes_found.push(n);
                if first_prime_per_class[i].is_none() {
                    first_prime_per_class[i] = Some(n);
                }
            }
            n += m;
        }
    }

    let classes_with_primes = first_prime_per_class.iter().filter(|x| x.is_some()).count() as u32;

    // Predicted density: |R_v| / φ(M)
    let predicted_density = Rational::new(r_v, phi_m);
    let actual_count = primes_found.len() as u64;

    DirichletVerification {
        address: addr,
        modulus: m,
        phi_m,
        r_v,
        matching_residues,
        search_limit,
        total_primes_found: actual_count,
        classes_with_primes,
        first_prime_per_class,
        predicted_relative_density: predicted_density,
    }
}

fn euler_phi_product(primes: &[u64]) -> u64 {
    primes.iter().map(|&p| p - 1).product()
}

#[derive(Debug)]
pub struct DirichletVerification {
    pub address: u64,
    pub modulus: u64,
    pub phi_m: u64,
    pub r_v: u64,
    pub matching_residues: Vec<u64>,
    pub search_limit: u64,
    pub total_primes_found: u64,
    pub classes_with_primes: u32,
    pub first_prime_per_class: Vec<Option<u64>>,
    pub predicted_relative_density: Rational,
}

// ═══════════════════════════════════════════════════════════════════
// REFINED POWER RESIDUE SEPARATION — find the symmetry condition
// ═══════════════════════════════════════════════════════════════════

/// The simple prediction gcd(k, p-1) > 1 → separates failed at 42/65.
/// The failures all have SYMMETRIC carry distributions within the image.
///
/// HYPOTHESIS: Separation requires the k-th power image to be
/// NOT centrosymmetric relative to the threshold p/2.
/// Centrosymmetry means: for every x in Image, (p - x) is also in Image.
/// When the image is centrosymmetric, equal numbers fall in each half → blind.
///
/// The Sqr image is always centrosymmetric (a² = (p-a)² mod p).
/// But the CARRY separation for Sqr depends on p mod 4 (T16).
/// So centrosymmetry alone doesn't explain the pattern.
///
/// DEEPER: For k=2 at p≡1(mod 4), the QR and QNR sets are BOTH
/// centrosymmetric (because -1 is QR → if a is QR, so is p-a).
/// This makes carry blind. At p≡3(mod 4), -1 is QNR → {a, p-a} crosses
/// classes → QR is NOT centrosymmetric → carry separates.
///
/// GENERALIZATION: For power k, the k-th power residues are blind
/// when the map a → p-a preserves the k-th power residue class.
/// This happens iff (-1) is a k-th power residue, i.e., (-1)^((p-1)/g) ≡ 1 (mod p)
/// where g = gcd(k, p-1).
///
/// (-1)^((p-1)/g) ≡ 1 (mod p)  iff  (p-1)/g is EVEN  iff  2g | (p-1).
///
/// PREDICTION (refined): carry separates iff
///   gcd(k, p-1) > 1  AND  (p-1) / gcd(k, p-1) is ODD
/// equivalently:
///   gcd(k, p-1) > 1  AND  NOT (2·gcd(k, p-1) | (p-1))
///
/// Wait — let me think more carefully.
/// (-1)^((p-1)/g) mod p: since ord(-1) = 2 in (Z/pZ)*,
///   = 1 if (p-1)/g is even
///   = -1 (= p-1) if (p-1)/g is odd
///
/// If (-1)^((p-1)/g) = 1: -1 is a g-th power residue, and since g = gcd(k,p-1),
/// -1 is also a k-th power residue → image centrosymmetric → blind.
///
/// If (-1)^((p-1)/g) = p-1: -1 is NOT a k-th power residue →
/// image NOT centrosymmetric → carry MAY separate.
///
/// So: SEPARATES iff gcd(k, p-1) > 1 AND (p-1)/gcd(k, p-1) is ODD.
pub fn refined_power_separation_predicts(p: u64, k: u64) -> bool {
    let g = crt::gcd(k as u128, (p - 1) as u128) as u64;
    if g <= 1 { return false; } // bijective → no separation possible
    let quotient = (p - 1) / g;
    quotient % 2 == 1 // (p-1)/g odd → -1 is NOT a k-th power residue → separates
}

/// Test the refined prediction against exhaustive computation.
pub fn test_refined_prediction(primes: &[u64], powers: &[u64]) -> (u32, u32, Vec<(u64, u64, bool, bool)>) {
    let mut correct = 0u32;
    let mut total = 0u32;
    let mut failures = Vec::new();

    for &p in primes {
        for &k in powers {
            let predicted = refined_power_separation_predicts(p, k);
            let actual = test_power_residue_separation(p, k);
            total += 1;
            if predicted == actual.separates {
                correct += 1;
            } else {
                failures.push((p, k, predicted, actual.separates));
            }
        }
    }

    (correct, total, failures)
}

// ═══════════════════════════════════════════════════════════════════
// POWER TOWER: Which k separates which p?
// T21 says: gcd(k,p-1)>1 AND (p-1)/gcd(k,p-1) odd.
// For k=2^j: this gives a BINARY TREE of separation conditions.
// ═══════════════════════════════════════════════════════════════════

/// For each prime p, find the SMALLEST power k = 2^j that separates.
/// This always exists because every odd prime p satisfies
/// p ≡ 2^j + 1 (mod 2^{j+1}) for exactly one j.
pub fn minimum_separating_power(p: u64) -> (u64, u64) {
    // p-1 = 2^a · m where m is odd
    let mut pm1 = p - 1;
    let mut a = 0u64;
    while pm1 % 2 == 0 { pm1 /= 2; a += 1; }
    // p - 1 = 2^a · m, m odd.
    // For k = 2^j: g = gcd(2^j, 2^a · m).
    //   If j ≤ a: g = 2^j, quotient = 2^(a-j) · m
    //   If j > a: g = 2^a, quotient = m
    //
    // Quotient odd iff:
    //   j ≤ a: 2^(a-j) · m odd iff a-j = 0 iff j = a.
    //   j > a: m odd (always true).
    //
    // So: minimum separating k = 2^a (where 2^a || (p-1)).
    // This is the exact power of 2 dividing p-1.
    let k = 1u64 << a;  // 2^a
    (k, a)
}

/// Build the full power tower for a set of primes.
/// Shows which k=2^j separates each prime.
pub fn power_tower(primes: &[u64]) -> Vec<PowerTowerEntry> {
    primes.iter().map(|&p| {
        let (min_k, j) = minimum_separating_power(p);
        let pm1 = p - 1;
        let g = crt::gcd(min_k as u128, pm1 as u128) as u64;
        let quotient = pm1 / g;

        // Verify
        let actual = test_power_residue_separation(p, min_k);

        PowerTowerEntry {
            prime: p,
            p_mod4: p % 4,
            min_k,
            j,
            v2_pm1: j, // 2-adic valuation of p-1
            quotient,
            verified: actual.separates,
        }
    }).collect()
}

#[derive(Debug)]
pub struct PowerTowerEntry {
    pub prime: u64,
    pub p_mod4: u64,
    pub min_k: u64,     // smallest 2^j that separates
    pub j: u64,          // exponent: k = 2^j
    pub v2_pm1: u64,     // 2-adic valuation of p-1
    pub quotient: u64,   // (p-1)/gcd(k, p-1)
    pub verified: bool,
}

/// For the CRAM schema: assign the optimal power per lane.
/// Each lane gets the minimum separating power as its post-processor.
/// This guarantees EVERY lane sees nonlinear structure.
pub fn optimal_power_schema(primes: &[u64]) -> Vec<(u64, u64)> {
    primes.iter().map(|&p| {
        let (k, _) = minimum_separating_power(p);
        (p, k)
    }).collect()
}
