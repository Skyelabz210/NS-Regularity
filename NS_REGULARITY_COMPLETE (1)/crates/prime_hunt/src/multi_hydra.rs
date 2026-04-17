//! Multi-Headed Coprime Family Analyzer
//!
//! Each Hydra head is a DIFFERENT CRAM lens on the coprime lattice.
//! The multi-head carry signature (concatenation of all heads' patterns)
//! is the high-dimensional fingerprint that resolves coprime family structure.
//!
//! Single head (5 lanes): 2⁵ = 32 strata
//! Two heads:             32 × 32 = 1,024 strata
//! Three heads:           32 × 32 × 32 = 32,768 strata
//!
//! Each head can be tuned to see different structure:
//! - Head A: gap-2 (twin) topology → sees twin-pair symmetry
//! - Head B: gap-6 (sexy) topology → sees sexy-pair symmetry
//! - Head C: S⁶ champion → sees maximal CRAM distortion
//! - Head D: Mul+Id everywhere → sees multiplicative structure
//! The INTERSECTION of their views reveals what no single head can.

use crate::coprime_families::{
    CramTopology, Lane, Root, Post, Champions,
};
use crate::crt;

/// A single Hydra head: one CRAM topology producing one carry pattern per input.
#[derive(Debug, Clone)]
pub struct Head {
    pub label: String,
    pub topology: CramTopology,
    pub target_gap: u64,  // what this head is tuned to observe
}

impl Head {
    pub fn new(label: &str, topology: CramTopology, target_gap: u64) -> Self {
        Self { label: label.to_string(), topology, target_gap }
    }

    /// Compute this head's carry pattern for element n.
    pub fn observe(&self, n: u64) -> Vec<u8> {
        self.topology.carry_pattern(n, self.target_gap)
    }

    pub fn k(&self) -> usize { self.topology.k() }
}

/// Multi-head carry signature: concatenation of all heads' patterns.
/// This is the high-dimensional fingerprint in the coprime lattice.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MultiSignature {
    pub bits: Vec<u8>,
    pub total_lanes: usize,
}

impl MultiSignature {
    pub fn to_id(&self) -> u64 {
        let mut id = 0u64;
        for (i, &bit) in self.bits.iter().enumerate() {
            if i >= 64 { break; } // cap at 64 bits
            id |= (bit as u64) << i;
        }
        id
    }

    pub fn to_string_compact(&self) -> String {
        self.bits.iter().map(|&b| if b == 1 { '1' } else { '0' }).collect()
    }
}

/// The Multi-Headed Hydra: multiple CRAM lenses observing simultaneously.
pub struct MultiHydra {
    pub heads: Vec<Head>,
    pub total_lanes: usize,
    pub total_strata: u64,  // 2^total_lanes (theoretical max)
}

impl MultiHydra {
    pub fn new(heads: Vec<Head>) -> Self {
        let total_lanes: usize = heads.iter().map(|h| h.k()).sum();
        let total_strata = if total_lanes <= 20 { 1u64 << total_lanes } else { 0 }; // cap display
        Self { heads, total_lanes, total_strata }
    }

    /// Compute the full multi-head signature for element n.
    pub fn observe(&self, n: u64) -> MultiSignature {
        let mut bits = Vec::with_capacity(self.total_lanes);
        for head in &self.heads {
            bits.extend_from_slice(&head.observe(n));
        }
        MultiSignature { bits, total_lanes: self.total_lanes }
    }

    /// Map the coprime lattice: for each coprime n in [start, limit),
    /// compute multi-signature and bin by stratum.
    pub fn map_lattice(&self, start: u64, limit: u64) -> LatticeMap {
        let modulus = self.heads.iter()
            .map(|h| h.topology.modulus)
            .fold(1u64, |acc, m| {
                // Use LCM since heads may share primes
                let g = crt::gcd(acc as u128, m as u128) as u64;
                acc / g * m
            });

        let mut strata: std::collections::HashMap<u64, StratumData> = std::collections::HashMap::new();
        let mut total_coprimes = 0u32;
        let mut total_primes = 0u32;

        let is_prime = |n: u64| -> bool {
            if n < 2 { return false; } if n < 4 { return true; }
            if n % 2 == 0 || n % 3 == 0 { return false; }
            let mut i = 5u64;
            while i.saturating_mul(i) <= n {
                if n % i == 0 || n % (i+2) == 0 { return false; } i += 6;
            }
            true
        };

        for n in start.max(2)..limit {
            // Must be coprime to ALL head moduli
            let coprime = self.heads.iter().all(|h|
                crt::gcd(n as u128, h.topology.modulus as u128) == 1
            );
            if !coprime { continue; }

            total_coprimes += 1;
            let sig = self.observe(n);
            let sig_id = sig.to_id();
            let is_p = is_prime(n);
            if is_p { total_primes += 1; }

            let entry = strata.entry(sig_id).or_insert_with(|| StratumData {
                signature: sig.to_string_compact(),
                coprime_count: 0, prime_count: 0, composite_count: 0,
            });
            entry.coprime_count += 1;
            if is_p { entry.prime_count += 1; } else { entry.composite_count += 1; }
        }

        // Convert to sorted vec
        let mut stratum_vec: Vec<(u64, StratumData)> = strata.into_iter().collect();
        stratum_vec.sort_by(|a, b| {
            let da = a.1.prime_density();
            let db = b.1.prime_density();
            db.partial_cmp(&da).unwrap()
        });

        let populated = stratum_vec.len() as u32;
        let avg_density = if total_coprimes > 0 {
            total_primes as f64 / total_coprimes as f64
        } else { 0.0 };

        // Gini
        let densities: Vec<f64> = stratum_vec.iter()
            .filter(|(_, d)| d.coprime_count > 0)
            .map(|(_, d)| d.prime_density())
            .collect();
        let gini = compute_gini(&densities);

        // Spread
        let max_d = densities.iter().copied().fold(0.0f64, f64::max);
        let min_d = densities.iter().copied().fold(1.0f64, f64::min);

        LatticeMap {
            total_coprimes,
            total_primes,
            populated_strata: populated,
            theoretical_strata: self.total_strata,
            strata: stratum_vec,
            avg_density,
            gini,
            density_spread: max_d - min_d,
            total_lanes: self.total_lanes,
            head_count: self.heads.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StratumData {
    pub signature: String,
    pub coprime_count: u32,
    pub prime_count: u32,
    pub composite_count: u32,
}

impl StratumData {
    pub fn prime_density(&self) -> f64 {
        if self.coprime_count > 0 { self.prime_count as f64 / self.coprime_count as f64 }
        else { 0.0 }
    }
}

#[derive(Debug)]
pub struct LatticeMap {
    pub total_coprimes: u32,
    pub total_primes: u32,
    pub populated_strata: u32,
    pub theoretical_strata: u64,
    pub strata: Vec<(u64, StratumData)>,
    pub avg_density: f64,
    pub gini: f64,
    pub density_spread: f64,
    pub total_lanes: usize,
    pub head_count: usize,
}

fn compute_gini(values: &[f64]) -> f64 {
    if values.is_empty() { return 0.0; }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    if mean == 0.0 { return 0.0; }
    let abs_diff: f64 = values.iter()
        .flat_map(|&a| values.iter().map(move |&b| (a - b).abs()))
        .sum();
    abs_diff / (2.0 * n * n * mean)
}

// ── Pre-built Multi-Hydra Configurations ────────────────────────────

pub fn build_twin_hydra() -> MultiHydra {
    MultiHydra::new(vec![
        Head::new("Twin-Mul", CramTopology::new(vec![
            Lane { prime: 3,  root: Root::Mul, post: Post::Id },
            Lane { prime: 5,  root: Root::Sub, post: Post::Sqr },
            Lane { prime: 7,  root: Root::Add, post: Post::Sqr },
            Lane { prime: 11, root: Root::Mul, post: Post::Sqr },
            Lane { prime: 13, root: Root::Sub, post: Post::Id },
        ]), 2),
        Head::new("QR-Sqr", CramTopology::new(vec![
            Lane { prime: 3,  root: Root::Add, post: Post::Sqr },
            Lane { prime: 5,  root: Root::Add, post: Post::Sqr },
            Lane { prime: 7,  root: Root::Add, post: Post::Sqr },
            Lane { prime: 11, root: Root::Add, post: Post::Sqr },
            Lane { prime: 13, root: Root::Add, post: Post::Sqr },
        ]), 2),
        Head::new("Mul-Struct", CramTopology::new(vec![
            Lane { prime: 3,  root: Root::Mul, post: Post::Id },
            Lane { prime: 5,  root: Root::Mul, post: Post::Id },
            Lane { prime: 7,  root: Root::Mul, post: Post::Id },
            Lane { prime: 11, root: Root::Mul, post: Post::Id },
            Lane { prime: 13, root: Root::Mul, post: Post::Id },
        ]), 2),
    ])
}

pub fn build_multi_gap_hydra() -> MultiHydra {
    MultiHydra::new(vec![
        Head::new("Gap2-Twin", Champions::twin(), 2),
        Head::new("Gap6-Sexy", CramTopology::new(vec![
            Lane { prime: 3,  root: Root::Mul, post: Post::Id },
            Lane { prime: 5,  root: Root::Add, post: Post::Sqr },
            Lane { prime: 7,  root: Root::Sub, post: Post::Sqr },
            Lane { prime: 11, root: Root::Mul, post: Post::Sqr },
            Lane { prime: 13, root: Root::Add, post: Post::Id },
        ]), 6),
        Head::new("Gap30-Deep", CramTopology::new(vec![
            Lane { prime: 3,  root: Root::Sub, post: Post::Sqr },
            Lane { prime: 5,  root: Root::Mul, post: Post::Id },  // 5|30 → Mul sees zero structure
            Lane { prime: 7,  root: Root::Add, post: Post::Sqr },
            Lane { prime: 11, root: Root::Sub, post: Post::Neg },
            Lane { prime: 13, root: Root::Add, post: Post::Sqr },
        ]), 30),
    ])
}

/// Progressive Hydra: add heads one at a time to measure resolution gain.
pub fn build_progressive(gap: u64) -> Vec<(String, MultiHydra)> {
    let head_a = Head::new("A:Twin", Champions::twin(), gap);
    let head_b = Head::new("B:QR", CramTopology::new(vec![
        Lane { prime: 3,  root: Root::Add, post: Post::Sqr },
        Lane { prime: 5,  root: Root::Add, post: Post::Sqr },
        Lane { prime: 7,  root: Root::Add, post: Post::Sqr },
        Lane { prime: 11, root: Root::Add, post: Post::Sqr },
        Lane { prime: 13, root: Root::Add, post: Post::Sqr },
    ]), gap);
    let head_c = Head::new("C:Mul", CramTopology::new(vec![
        Lane { prime: 3,  root: Root::Mul, post: Post::Id },
        Lane { prime: 5,  root: Root::Mul, post: Post::Id },
        Lane { prime: 7,  root: Root::Mul, post: Post::Id },
        Lane { prime: 11, root: Root::Mul, post: Post::Id },
        Lane { prime: 13, root: Root::Mul, post: Post::Id },
    ]), gap);

    vec![
        ("1-head (A)".into(), MultiHydra::new(vec![head_a.clone()])),
        ("2-head (A+B)".into(), MultiHydra::new(vec![head_a.clone(), head_b.clone()])),
        ("3-head (A+B+C)".into(), MultiHydra::new(vec![head_a, head_b, head_c])),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_signature() {
        let hydra = build_twin_hydra();
        assert_eq!(hydra.heads.len(), 3);
        assert_eq!(hydra.total_lanes, 15); // 5+5+5
        let sig = hydra.observe(17);
        assert_eq!(sig.bits.len(), 15);
    }

    #[test]
    fn test_progressive_resolution() {
        let configs = build_progressive(2);
        assert_eq!(configs.len(), 3);
        assert_eq!(configs[0].1.total_lanes, 5);
        assert_eq!(configs[1].1.total_lanes, 10);
        assert_eq!(configs[2].1.total_lanes, 15);
    }

    #[test]
    fn test_lattice_map_basic() {
        let hydra = MultiHydra::new(vec![
            Head::new("test", Champions::baseline(), 2),
        ]);
        let map = hydra.map_lattice(2, 1000);
        assert!(map.total_coprimes > 0);
        assert!(map.total_primes > 0);
        assert!(map.populated_strata > 0);
        // Totals must be consistent
        let sum_c: u32 = map.strata.iter().map(|(_, d)| d.coprime_count).sum();
        assert_eq!(sum_c, map.total_coprimes);
    }
}
