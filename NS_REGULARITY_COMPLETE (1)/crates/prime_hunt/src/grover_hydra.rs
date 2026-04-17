//! Grover-Hydra — Dense Toric Grover Heads on the Coprime Lattice
//!
//! ARCHITECTURE:
//!
//! Classical Hydra (current):
//!   For each n: compute carry_pattern(n) → lookup atlas → test or skip
//!   Cost: O(d) per candidate, O(N·d) total
//!
//! Grover-Hydra (this module):
//!   Build oracle from atlas (prime-rich strata = marked states)
//!   Run Dense Toric Grover on the CRT product torus
//!   Measure → collapse to a prime-rich address with amplified probability
//!   Cost: O(√(N/k)) iterations where k = prime-rich strata population
//!
//! Each Hydra head is a Grover search on a DIFFERENT CRT sub-torus:
//!   Head A (Grover tier, p≡3 mod 4): F_p² exact amplitudes, zero decoherence
//!   Head B (Classical tier, p≡1 mod 4): classical carry bit enumeration
//!   Head C (Grover tier): different operator topology → different oracle
//!
//! The multi-head intersection works the SAME way:
//!   Classical: concatenate carry patterns → joint address
//!   Grover: tensor product of per-head search spaces → joint measurement
//!
//! T6 determines the oracle weight per lane:
//!   SP(p,d) = (2p-1-c_p(d))/p² → forbidden count F(p,d)
//!   Survivors k(p,d) = p - F(p,d)·p = p·(1-SP(p,d))
//!
//! Ramanujan-Grover formula gives iteration count:
//!   T_opt(p,d) = ⌊π/4 · p / √(2p-1-c_p(d))⌋
//!
//! T16 determines head type:
//!   p ≡ 3 (mod 4): F_p² substrate available → Grover head
//!   p ≡ 1 (mod 4): no F_p² → classical head (still contributes via carry)

use crate::crt;
use crate::exact::Rational;
use crate::coprime_families::{CramTopology, Lane, Root, Post};

fn mulmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 * b as u128) % m as u128) as u64 }

// ── Grover Lane Classification (matches WP1 two-tier oracle) ────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroverTier {
    /// p ≡ 3 (mod 4): F_p² = F_p[i]/(i²+1) exists, exact complex arithmetic
    /// Dense Toric Grover runs natively with zero decoherence
    Quantum,
    /// p ≡ 1 (mod 4): -1 is QR, i²+1 splits, no F_p² extension
    /// Classical carry bit computation (still sees QR structure via T16)
    Classical,
}

pub fn classify_grover_tier(p: u64) -> GroverTier {
    if p % 4 == 3 { GroverTier::Quantum } else { GroverTier::Classical }
}

// ── Ramanujan-Grover Formula (from WP1) ─────────────────────────────

/// T_opt(p,d) = ⌊π/4 · p / √(2p-1-c_p(d))⌋
/// Computed in exact integer arithmetic (no π, no √ — use integer approx)
///
/// For the permille approximation:
///   T_opt ≈ (785 × p) / (1000 × isqrt(2p-1-c_p(d)))
/// where 785/1000 ≈ π/4
pub fn ramanujan_grover_iterations(p: u64, d: u64) -> GroverSpec {
    let c: i64 = if d % p == 0 { p as i64 - 1 } else { -1 };
    let forbidden_weight = (2 * p as i64 - 1 - c) as u64;
    // forbidden_weight = 2p-1-c_p(d)
    //   p∤d: = 2p-1+1 = 2p (two forbidden residues)
    //   p|d: = 2p-1-(p-1) = p (one forbidden residue)

    let f_per_lane = forbidden_weight / p;
    let survivors_per_lane = p - f_per_lane;

    // T_opt = ⌊π/4 · √(p / F_per_lane)⌋ (standard Grover with N=p, k=survivors)
    // Wait — Grover searches for MARKED items. If we mark the SURVIVORS (non-forbidden),
    // then k = survivors_per_lane, N = p, and T_opt = ⌊π/4 · √(N/k)⌋
    // But if we mark the FORBIDDEN items to AVOID, k = f_per_lane, and we search for UNmarked.
    //
    // For the prime-hunting oracle: we mark PRIME-RICH strata.
    // For the sieve oracle: we mark FORBIDDEN residues.
    //
    // The Ramanujan-Grover formula marks forbidden:
    //   T_opt = ⌊π/4 · p / √(forbidden_weight)⌋
    //
    // Integer square root
    let fw = forbidden_weight;
    let isqrt_fw = integer_sqrt(fw);

    // T_opt ≈ (785 × p) / (1000 × isqrt_fw)
    let t_opt = if isqrt_fw > 0 {
        (785 * p) / (1000 * isqrt_fw)
    } else { p };

    // Classical cost = p (enumerate all residues)
    let speedup = Rational::new(p, t_opt.max(1));

    let tier = classify_grover_tier(p);

    GroverSpec {
        prime: p,
        gap: d,
        tier,
        forbidden_weight: fw,
        forbidden_per_lane: f_per_lane,
        survivors_per_lane,
        t_opt,
        classical_cost: p,
        speedup,
        ramanujan_c: c,
    }
}

fn integer_sqrt(n: u64) -> u64 {
    if n == 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + n / x) / 2; }
    x
}

#[derive(Debug)]
pub struct GroverSpec {
    pub prime: u64,
    pub gap: u64,
    pub tier: GroverTier,
    pub forbidden_weight: u64,
    pub forbidden_per_lane: u64,
    pub survivors_per_lane: u64,
    pub t_opt: u64,
    pub classical_cost: u64,
    pub speedup: Rational,
    pub ramanujan_c: i64,
}

// ── Multi-Head Grover-Hydra ─────────────────────────────────────────

/// A Grover-Hydra head: one CRT sub-torus with its Grover/classical spec.
#[derive(Debug)]
pub struct GroverHead {
    pub label: String,
    pub specs: Vec<GroverSpec>,  // one per lane prime
    pub total_torus_size: u64,   // product of lane primes
    pub total_survivors: u64,    // product of per-lane survivors
    pub composite_t_opt: u64,    // ⌊π/4 · √(N/k)⌋ for full torus
    pub composite_speedup: Rational,
}

pub fn build_grover_head(label: &str, primes: &[u64], gap: u64) -> GroverHead {
    let specs: Vec<GroverSpec> = primes.iter()
        .map(|&p| ramanujan_grover_iterations(p, gap))
        .collect();

    let n: u64 = primes.iter().product();
    let k: u64 = specs.iter().map(|s| s.survivors_per_lane).product();

    // Composite T_opt = ⌊π/4 · √(N/k)⌋
    let ratio = n / k.max(1);
    let isqrt_ratio = integer_sqrt(ratio);
    let composite_t = (785 * isqrt_ratio) / 1000;

    GroverHead {
        label: label.to_string(),
        specs,
        total_torus_size: n,
        total_survivors: k,
        composite_t_opt: composite_t.max(1),
        composite_speedup: Rational::new(n, composite_t.max(1)),
    }
}

/// Full Grover-Hydra: multiple heads, each on different sub-torus.
pub fn build_grover_hydra(gap: u64) -> Vec<GroverHead> {
    vec![
        // Head A: Safe Basis Grover tier (p≡3 mod 4)
        build_grover_head("Grover-3,7,11", &[3, 7, 11], gap),

        // Head B: Safe Basis Classical tier (p≡1 mod 4)
        build_grover_head("Classical-5,13", &[5, 13], gap),

        // Head C: Extended Grover (large connector primes)
        build_grover_head("Extended-Grover", &[23, 31, 43], gap),

        // Head D: Full Safe Basis (mixed tier)
        build_grover_head("Full-S6", &[2, 3, 5, 7, 11, 13], gap),

        // Head E: Augmented (Safe + QR primes, mixed tier)
        build_grover_head("Augmented", &[3, 5, 7, 11, 13, 29, 37], gap),
    ]
}

/// Atlas-as-oracle: given the coprime lattice atlas from T12/T14,
/// compute the oracle weight (fraction of torus that is prime-rich).
/// This determines the Grover iteration count for the atlas-targeted search.
pub fn atlas_oracle_weight(
    prime_rich_count: u32,
    total_addresses: u32,
    torus_size: u64,
) -> AtlasOracle {
    // Oracle marks prime-rich addresses on the torus
    // k = number of marked states = prime_rich_count × (torus_size / total_addresses)
    // Approximate: each address has ~equal population
    let states_per_addr = torus_size / total_addresses as u64;
    let marked = prime_rich_count as u64 * states_per_addr;

    let t_opt = if marked > 0 && marked < torus_size {
        let ratio = torus_size / marked;
        (785 * integer_sqrt(ratio)) / 1000
    } else { 1 };

    AtlasOracle {
        torus_size,
        marked_states: marked,
        unmarked_states: torus_size - marked,
        t_opt,
        speedup: Rational::new(torus_size, t_opt.max(1)),
        prime_rich_fraction: Rational::new(prime_rich_count as u64, total_addresses as u64),
    }
}

#[derive(Debug)]
pub struct AtlasOracle {
    pub torus_size: u64,
    pub marked_states: u64,
    pub unmarked_states: u64,
    pub t_opt: u64,
    pub speedup: Rational,
    pub prime_rich_fraction: Rational,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grover_tier_classification() {
        assert_eq!(classify_grover_tier(3), GroverTier::Quantum);
        assert_eq!(classify_grover_tier(7), GroverTier::Quantum);
        assert_eq!(classify_grover_tier(11), GroverTier::Quantum);
        assert_eq!(classify_grover_tier(5), GroverTier::Classical);
        assert_eq!(classify_grover_tier(13), GroverTier::Classical);
    }

    #[test]
    fn test_ramanujan_grover_formula() {
        // p=3, d=2: c_3(2) = -1, fw = 2·3-1+1 = 6, F=2, survivors=1
        let spec = ramanujan_grover_iterations(3, 2);
        assert_eq!(spec.forbidden_weight, 6);
        assert_eq!(spec.forbidden_per_lane, 2);
        assert_eq!(spec.survivors_per_lane, 1);
        assert_eq!(spec.tier, GroverTier::Quantum);
    }

    #[test]
    fn test_ramanujan_grover_divisible() {
        // p=3, d=6: c_3(6) = 2, fw = 6-1-2 = 3, F=1, survivors=2
        let spec = ramanujan_grover_iterations(3, 6);
        assert_eq!(spec.forbidden_weight, 3);
        assert_eq!(spec.forbidden_per_lane, 1);
        assert_eq!(spec.survivors_per_lane, 2);
    }

    #[test]
    fn test_safe_basis_composite() {
        // Full Safe Basis {2,3,5,7,11,13}, gap=2
        let head = build_grover_head("S6", &[2, 3, 5, 7, 11, 13], 2);
        assert_eq!(head.total_torus_size, 30030);
        // Survivors should match WP1: 1485 for twins on Safe Basis
        // survivors = (2-1)·(3-2)·(5-2)·(7-2)·(11-2)·(13-2) = 1·1·3·5·9·11 = 1485
        assert_eq!(head.total_survivors, 1485);
    }

    #[test]
    fn test_atlas_oracle() {
        // 515 prime-rich out of 600 addresses, torus size 15015
        let oracle = atlas_oracle_weight(515, 600, 15015);
        assert!(oracle.marked_states > 0);
        assert!(oracle.speedup.num > oracle.speedup.den); // speedup > 1
    }

    #[test]
    fn test_extended_grover_speedup() {
        // Large connector prime: p=10007, gap=2
        let spec = ramanujan_grover_iterations(10007, 2);
        // T_opt should be much less than p
        assert!(spec.t_opt < spec.classical_cost / 10,
            "Expected significant speedup, got T_opt={} vs classical={}",
            spec.t_opt, spec.classical_cost);
    }
}
