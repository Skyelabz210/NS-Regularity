//! Coprime Family Analyzer — Heterogeneous CRAM CRT
//!
//! POLARITY: We are NOT eliminating composites. We are MAPPING the coprime
//! lattice. Primes are the boundary. Coprime composites are the substance.
//!
//! Each CRAM carry pattern defines a STRATUM of the coprime lattice M_H.
//! The distribution of coprimes across strata — and the prime density
//! within each stratum — reveals the internal structure of the lattice.
//!
//! CRAM operators are DISTORTION LENSES on the CRT torus:
//! - Sqr creates 2-to-1 folding → reveals quadratic residue symmetry
//! - Neg reflects → reveals additive inverse structure
//! - Mul by d couples lanes → reveals cross-lane correlation
//! - Add/Sub translate → baseline (Dirichlet-uniform) reference
//!
//! The heterogeneous topology (different operators per lane) is essential:
//! it allows each CRT dimension to be observed through the lens that
//! best resolves its contribution to the coprime family structure.
//!
//! T6: SP(p,d) = (2p − 1 − c_p(d)) / p²  predicts the BASE distribution
//! CRAM operators then DISTORT that base, creating resolution the base lacks.

use crate::crt::{self, gcd, is_prime, factorize};

// ── CRAM Operator Types ─────────────────────────────────────────────

/// Root binary operator: applied to (n mod p, d mod p)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Root { Add, Sub, Mul, Div }

/// Unary post-processor: applied to root output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Post { Id, Neg, Sqr, Inv }

impl std::fmt::Display for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self { Root::Add => write!(f, "ADD"), Root::Sub => write!(f, "SUB"),
                     Root::Mul => write!(f, "MUL"), Root::Div => write!(f, "DIV") }
    }
}
impl std::fmt::Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self { Post::Id => write!(f, "Id"), Post::Neg => write!(f, "Neg"),
                     Post::Sqr => write!(f, "Sqr"), Post::Inv => write!(f, "Inv") }
    }
}

// ── Modular Arithmetic (u64, overflow-safe) ─────────────────────────

fn addmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 + b as u128) % m as u128) as u64 }
fn submod(a: u64, b: u64, m: u64) -> u64 { if a >= b { (a - b) % m } else { m - ((b - a) % m) } }
fn mulmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 * b as u128) % m as u128) as u64 }

fn mod_inv(a: u64, m: u64) -> Option<u64> {
    let (mut old_r, mut r) = (a as i128, m as i128);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 { let q = old_r / r; let t = r; r = old_r - q*r; old_r = t; let t = s; s = old_s - q*s; old_s = t; }
    if old_r != 1 { None } else { Some(((old_s % m as i128 + m as i128) as u128 % m as u128) as u64) }
}

/// Public wrapper for cross-module access
pub fn mod_inv_pub(a: u64, m: u64) -> Option<u64> { mod_inv(a, m) }

fn is_prime_u64(n: u64) -> bool {
    if n < 2 { return false; } if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5u64;
    while i.saturating_mul(i) <= n { if n % i == 0 || n % (i+2) == 0 { return false; } i += 6; }
    true
}

// ── CRAM Lane & Topology ────────────────────────────────────────────

/// Per-lane configuration: a prime and its heterogeneous operator pair.
#[derive(Debug, Clone, Copy)]
pub struct Lane {
    pub prime: u64,
    pub root: Root,
    pub post: Post,
}

/// Heterogeneous CRAM topology: each lane has independent operators.
#[derive(Debug, Clone)]
pub struct CramTopology {
    pub lanes: Vec<Lane>,
    pub modulus: u64, // product of all lane primes
}

impl CramTopology {
    pub fn new(lanes: Vec<Lane>) -> Self {
        let modulus = lanes.iter().fold(1u64, |a, l| a * l.prime);
        Self { lanes, modulus }
    }

    pub fn k(&self) -> usize { self.lanes.len() }

    /// Apply the full CRAM pipeline to inputs (a, b).
    /// Returns per-lane outputs (NOT reconstructed — we want the pattern).
    pub fn apply_lanes(&self, a: u64, b: u64) -> Vec<u64> {
        self.lanes.iter().map(|lane| {
            let p = lane.prime;
            let ar = a % p;
            let br = b % p;

            // Root operator
            let inter = match lane.root {
                Root::Add => addmod(ar, br, p),
                Root::Sub => submod(ar, br, p),
                Root::Mul => mulmod(ar, br, p),
                Root::Div => {
                    if br == 0 { 0 }
                    else { mod_inv(br, p).map(|inv| mulmod(ar, inv, p)).unwrap_or(0) }
                }
            };

            // Post-processor
            match lane.post {
                Post::Id => inter,
                Post::Neg => if inter == 0 { 0 } else { p - inter },
                Post::Sqr => mulmod(inter, inter, p),
                Post::Inv => {
                    if inter == 0 { 0 }
                    else { mod_inv(inter, p).unwrap_or(0) }
                }
            }
        }).collect()
    }

    /// Compute carry pattern: threshold each lane output at p/2.
    pub fn carry_pattern(&self, n: u64, d: u64) -> Vec<u8> {
        let outputs = self.apply_lanes(n, d);
        self.lanes.iter().zip(outputs.iter()).map(|(lane, &out)| {
            if out * 2 >= lane.prime { 1 } else { 0 }
        }).collect()
    }

    /// Ternary carry pattern: threshold at p/3 and 2p/3.
    /// Returns 0, 1, or 2 per lane — 2 bits of information vs 1.
    /// Encode as two binary bits per lane for address computation.
    pub fn ternary_pattern(&self, n: u64, d: u64) -> Vec<u8> {
        let outputs = self.apply_lanes(n, d);
        let mut pattern = Vec::with_capacity(self.lanes.len() * 2);
        for (lane, &out) in self.lanes.iter().zip(outputs.iter()) {
            let p = lane.prime;
            let lo = (p + 2) / 3;      // ⌈p/3⌉
            let hi = (2 * p + 1) / 3;   // ⌈2p/3⌉
            // Bit 0: is output in lower third?
            pattern.push(if out < lo { 1 } else { 0 });
            // Bit 1: is output in upper third?
            pattern.push(if out >= hi { 1 } else { 0 });
        }
        pattern
    }

    /// Format topology as string
    pub fn label(&self) -> String {
        self.lanes.iter()
            .map(|l| format!("{}{}+{}", l.root, l.prime, l.post))
            .collect::<Vec<_>>().join(",")
    }

    /// Validity: fraction of inputs where all lanes succeed (DIV/INV can fail).
    /// Returns (num, den) exact rational.
    pub fn validity(&self) -> (u64, u64) {
        let mut num = 1u64;
        let mut den = 1u64;
        for lane in &self.lanes {
            let p = lane.prime;
            if matches!(lane.root, Root::Div) { num *= p - 1; den *= p; }
            if matches!(lane.post, Post::Inv) { num *= p - 1; den *= p; }
        }
        let g = gcd(num as u128, den as u128) as u64;
        (num / g, den / g)
    }

    /// Quartic carry pattern: n^4 mod p threshold.
    /// For p with v₂(p-1) ≥ 2, this separates where Sqr is blind.
    pub fn quartic_pattern(&self, n: u64, _d: u64) -> Vec<u8> {
        self.lanes.iter().map(|lane| {
            let p = lane.prime;
            let r = n % p;
            let n4 = ((r as u128).pow(4) % p as u128) as u64;
            if n4 * 2 >= p { 1 } else { 0 }
        }).collect()
    }

    /// Optimal power pattern: use k = 2^v₂(p-1) per lane.
    /// Guarantees separation on EVERY lane (Power Tower Theorem).
    pub fn optimal_power_pattern(&self, n: u64) -> Vec<u8> {
        self.lanes.iter().map(|lane| {
            let p = lane.prime;
            let r = n % p;
            if r == 0 { return 0; }
            let mut pm1 = p - 1;
            let mut v2 = 0u32;
            while pm1 % 2 == 0 { pm1 /= 2; v2 += 1; }
            let k = 1u64 << v2;
            let rk = pow_mod_co(r, k, p);
            if rk * 2 >= p { 1 } else { 0 }
        }).collect()
    }
}

// ── Champion Topologies ─────────────────────────────────────────────

pub struct Champions;

impl Champions {
    /// CRAM-S⁶: MUL₂,SUB₃,ADD₅,ADD₇,MUL₁₁,DIV₁₃ + SQR all
    /// Validity: 12/13
    pub fn s6() -> CramTopology {
        CramTopology::new(vec![
            Lane { prime: 2,  root: Root::Mul, post: Post::Sqr },
            Lane { prime: 3,  root: Root::Sub, post: Post::Sqr },
            Lane { prime: 5,  root: Root::Add, post: Post::Sqr },
            Lane { prime: 7,  root: Root::Add, post: Post::Sqr },
            Lane { prime: 11, root: Root::Mul, post: Post::Sqr },
            Lane { prime: 13, root: Root::Div, post: Post::Sqr },
        ])
    }

    /// CRAM-S⁴: MUL₂,SUB₃,ADD₅,DIV₇ + SQR all
    /// Validity: 6/7
    pub fn s4() -> CramTopology {
        CramTopology::new(vec![
            Lane { prime: 2, root: Root::Mul, post: Post::Sqr },
            Lane { prime: 3, root: Root::Sub, post: Post::Sqr },
            Lane { prime: 5, root: Root::Add, post: Post::Sqr },
            Lane { prime: 7, root: Root::Div, post: Post::Sqr },
        ])
    }

    /// Twin-optimized heterogeneous: Mul₃+Id, Sub₅+Sqr, Add₇+Sqr, Mul₁₁+Sqr, Sub₁₃+Id
    pub fn twin() -> CramTopology {
        CramTopology::new(vec![
            Lane { prime: 3,  root: Root::Mul, post: Post::Id },
            Lane { prime: 5,  root: Root::Sub, post: Post::Sqr },
            Lane { prime: 7,  root: Root::Add, post: Post::Sqr },
            Lane { prime: 11, root: Root::Mul, post: Post::Sqr },
            Lane { prime: 13, root: Root::Sub, post: Post::Id },
        ])
    }

    /// Plain Add+Id — Dirichlet baseline (no distortion)
    pub fn baseline() -> CramTopology {
        CramTopology::new(vec![
            Lane { prime: 3,  root: Root::Add, post: Post::Id },
            Lane { prime: 5,  root: Root::Add, post: Post::Id },
            Lane { prime: 7,  root: Root::Add, post: Post::Id },
            Lane { prime: 11, root: Root::Add, post: Post::Id },
            Lane { prime: 13, root: Root::Add, post: Post::Id },
        ])
    }
}

// ── Stratum: One carry pattern's coprime population ─────────────────

/// A stratum is a region of the coprime lattice defined by a carry pattern.
#[derive(Debug)]
pub struct Stratum {
    pub pattern: Vec<u8>,
    pub pattern_id: u32,
    pub coprime_count: u32,     // total coprimes in this stratum
    pub prime_count: u32,       // primes among them
    pub composite_count: u32,   // coprime composites among them
    pub prime_density: f64,     // prime_count / coprime_count
}

// ── Coprime Family Map ──────────────────────────────────────────────

/// The full coprime family decomposition under a given CRAM topology.
#[derive(Debug)]
pub struct CoprimeFamilyMap {
    pub topology_label: String,
    pub gap: u64,
    pub range: (u64, u64),
    pub total_coprimes: u32,
    pub total_primes: u32,
    pub strata: Vec<Stratum>,
    pub modulus: u64,

    // Structural metrics
    pub stratum_count: u32,        // how many distinct patterns observed
    pub empty_strata: u32,         // patterns with zero coprimes
    pub prime_rich_strata: u32,    // patterns with above-average prime density
    pub prime_poor_strata: u32,    // patterns with below-average prime density
    pub max_density_stratum: u32,  // pattern ID with highest prime density
    pub min_density_stratum: u32,  // pattern ID with lowest prime density (among populated)
    pub density_spread: f64,       // max - min prime density
    pub gini_coefficient: f64,     // inequality measure of prime distribution across strata
}

/// Build a coprime family map for a given topology, gap, and range.
pub fn map_coprime_families(
    topo: &CramTopology,
    gap: u64,
    start: u64,
    limit: u64,
) -> CoprimeFamilyMap {
    let k = topo.k();
    let total_patterns = 1u32 << k;
    let modulus = topo.modulus;

    // Initialize strata
    let mut strata_data: Vec<(u32, u32, u32)> = vec![(0, 0, 0); total_patterns as usize];
    // (coprime_count, prime_count, composite_count)

    let mut total_coprimes = 0u32;
    let mut total_primes = 0u32;

    for n in start.max(2)..limit {
        // Must be coprime to the modulus to be in M_H
        if gcd(n as u128, modulus as u128) != 1 { continue; }

        let pattern = topo.carry_pattern(n, gap);

        // Pattern → ID
        let mut pattern_id = 0u32;
        for (i, &bit) in pattern.iter().enumerate() {
            pattern_id |= (bit as u32) << i;
        }

        let is_p = is_prime_u64(n);

        strata_data[pattern_id as usize].0 += 1; // coprime
        total_coprimes += 1;

        if is_p {
            strata_data[pattern_id as usize].1 += 1; // prime
            total_primes += 1;
        } else {
            strata_data[pattern_id as usize].2 += 1; // coprime composite
        }
    }

    let avg_density = if total_coprimes > 0 {
        total_primes as f64 / total_coprimes as f64
    } else { 0.0 };

    // Build strata
    let mut strata = Vec::new();
    let mut empty = 0u32;
    let mut prime_rich = 0u32;
    let mut prime_poor = 0u32;
    let mut max_d = 0.0f64;
    let mut min_d = 1.0f64;
    let mut max_d_id = 0u32;
    let mut min_d_id = 0u32;

    for pid in 0..total_patterns {
        let (cc, pc, coc) = strata_data[pid as usize];
        let pattern: Vec<u8> = (0..k).map(|i| ((pid >> i) & 1) as u8).collect();
        let density = if cc > 0 { pc as f64 / cc as f64 } else { 0.0 };

        if cc == 0 {
            empty += 1;
        } else {
            if density > avg_density { prime_rich += 1; }
            else { prime_poor += 1; }

            if density > max_d { max_d = density; max_d_id = pid; }
            if density < min_d { min_d = density; min_d_id = pid; }
        }

        strata.push(Stratum {
            pattern,
            pattern_id: pid,
            coprime_count: cc,
            prime_count: pc,
            composite_count: coc,
            prime_density: density,
        });
    }

    // Gini coefficient of prime distribution across strata
    let gini = compute_gini(&strata);

    CoprimeFamilyMap {
        topology_label: topo.label(),
        gap,
        range: (start, limit),
        total_coprimes,
        total_primes,
        strata,
        modulus,
        stratum_count: total_patterns - empty,
        empty_strata: empty,
        prime_rich_strata: prime_rich,
        prime_poor_strata: prime_poor,
        max_density_stratum: max_d_id,
        min_density_stratum: min_d_id,
        density_spread: max_d - min_d,
        gini_coefficient: gini,
    }
}

fn compute_gini(strata: &[Stratum]) -> f64 {
    let populated: Vec<f64> = strata.iter()
        .filter(|s| s.coprime_count > 0)
        .map(|s| s.prime_density)
        .collect();
    if populated.is_empty() { return 0.0; }
    let n = populated.len() as f64;
    let mean = populated.iter().sum::<f64>() / n;
    if mean == 0.0 { return 0.0; }
    let abs_diff_sum: f64 = populated.iter()
        .flat_map(|&a| populated.iter().map(move |&b| (a - b).abs()))
        .sum();
    abs_diff_sum / (2.0 * n * n * mean)
}

/// Compare two topologies on the same range/gap.
/// Returns which one better resolves coprime family structure.
pub fn compare_topologies(
    topo_a: &CramTopology,
    topo_b: &CramTopology,
    gap: u64,
    limit: u64,
) -> TopologyComparison {
    let map_a = map_coprime_families(topo_a, gap, 2, limit);
    let map_b = map_coprime_families(topo_b, gap, 2, limit);

    TopologyComparison {
        label_a: topo_a.label(),
        label_b: topo_b.label(),
        spread_a: map_a.density_spread,
        spread_b: map_b.density_spread,
        gini_a: map_a.gini_coefficient,
        gini_b: map_b.gini_coefficient,
        populated_a: map_a.stratum_count,
        populated_b: map_b.stratum_count,
        map_a,
        map_b,
    }
}

#[derive(Debug)]
pub struct TopologyComparison {
    pub label_a: String,
    pub label_b: String,
    pub spread_a: f64,
    pub spread_b: f64,
    pub gini_a: f64,
    pub gini_b: f64,
    pub populated_a: u32,
    pub populated_b: u32,
    pub map_a: CoprimeFamilyMap,
    pub map_b: CoprimeFamilyMap,
}

/// Ramanujan-predicted sieve power for reference
pub fn ramanujan_sieve_power(p: u64, d: u64) -> (u64, u64) {
    let c: i64 = if d % p == 0 { p as i64 - 1 } else { -1 };
    let num = (2 * p as i64 - 1 - c) as u64;
    let den = p * p;
    let g = gcd(num as u128, den as u128) as u64;
    (num / g, den / g)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cram_apply_heterogeneous() {
        // S⁶ champion: different operator per lane
        let topo = Champions::s6();
        assert_eq!(topo.k(), 6);
        assert_eq!(topo.modulus, 30030);

        // Apply to n=17, d=2
        let outputs = topo.apply_lanes(17, 2);
        assert_eq!(outputs.len(), 6);
        // Each output must be in [0, p)
        for (lane, &out) in topo.lanes.iter().zip(outputs.iter()) {
            assert!(out < lane.prime, "output {} >= prime {}", out, lane.prime);
        }
    }

    #[test]
    fn test_carry_pattern_binary() {
        let topo = Champions::baseline();
        let pattern = topo.carry_pattern(17, 2);
        assert_eq!(pattern.len(), 5);
        for &bit in &pattern { assert!(bit <= 1); }
    }

    #[test]
    fn test_coprime_family_map_basic() {
        let topo = Champions::baseline();
        let map = map_coprime_families(&topo, 2, 2, 1000);
        assert!(map.total_coprimes > 0);
        assert!(map.total_primes > 0);
        assert!(map.stratum_count > 0);
        // Total primes + composites across strata must equal totals
        let sum_p: u32 = map.strata.iter().map(|s| s.prime_count).sum();
        let sum_c: u32 = map.strata.iter().map(|s| s.coprime_count).sum();
        assert_eq!(sum_p, map.total_primes);
        assert_eq!(sum_c, map.total_coprimes);
    }

    #[test]
    fn test_heterogeneous_resolves_more() {
        // Heterogeneous topology should have higher Gini (more structure)
        // than baseline (Add+Id everywhere)
        let comp = compare_topologies(
            &Champions::twin(),
            &Champions::baseline(),
            2, 5000,
        );
        // The heterogeneous topology should see MORE density variation
        // (higher Gini = more inequality = more resolved structure)
        // This is the core claim of CRAM heterogeneity
        println!("Twin topo Gini: {:.4}, Baseline Gini: {:.4}",
            comp.gini_a, comp.gini_b);
        // We don't assert which is higher yet — let the data speak
    }

    #[test]
    fn test_s6_validity() {
        let topo = Champions::s6();
        let (n, d) = topo.validity();
        // S⁶ has one DIV lane (p=13), no INV posts
        // Validity = (13-1)/13 = 12/13
        assert_eq!((n, d), (12, 13));
    }

    #[test]
    fn test_ramanujan_sieve_power() {
        let (n, d) = ramanujan_sieve_power(3, 2);
        assert_eq!((n, d), (2, 3));
        let (n, d) = ramanujan_sieve_power(3, 6);
        assert_eq!((n, d), (1, 3));
    }
}

fn pow_mod_co(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result = 1u64;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 { result = ((result as u128 * base as u128) % modulus as u128) as u64; }
        exp >>= 1;
        base = ((base as u128 * base as u128) % modulus as u128) as u64;
    }
    result
}
