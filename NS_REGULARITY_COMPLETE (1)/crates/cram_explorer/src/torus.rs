//! # CRT Torus — Configurable Exploration Substrate
//!
//! A CRT torus Z/P_odd × Z/2^k with:
//! - Arbitrary odd-prime sets (Safe Basis or exact-lane primes)
//! - Dynamic 2-power depth k (the key insight from Phase 2)
//! - K-Elimination lift/project operations
//! - Per-lane decomposition and carry analysis
//!
//! The torus is the arena. Maps live on it. The projector filters it.

use crate::mod_arith::*;
use crate::k_elim::*;

/// A point on the CRT torus: (r_odd, r_2) where
/// r_odd ∈ Z/P_odd and r_2 ∈ Z/2^k.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TorusPoint {
    pub r_odd: i64,
    pub r_2: i64,
}

impl TorusPoint {
    pub fn new(r_odd: i64, r_2: i64) -> Self {
        TorusPoint { r_odd, r_2 }
    }
}

impl std::fmt::Display for TorusPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({},{})", self.r_odd, self.r_2)
    }
}

/// Configuration for a CRT torus.
#[derive(Clone, Debug)]
pub struct TorusConfig {
    /// The odd primes defining the torus
    pub odd_primes: Vec<i64>,
    /// Product of odd primes
    pub p_odd: i64,
    /// 2-power exponent k (torus has Z/2^k component)
    pub k: u32,
    /// 2^k
    pub p_two: i64,
    /// Total states = p_odd * 2^k
    pub total: i64,
    /// Precomputed: 2^{-1} mod p_odd
    pub inv2_mod_p_odd: i64,
    /// Precomputed: p_odd^{-1} mod 2^k
    pub inv_p_odd_mod_p2: i64,
}

impl TorusConfig {
    /// Create a new torus configuration.
    /// `odd_primes`: list of odd primes (must be pairwise coprime, all > 2)
    /// `k`: 2-power exponent
    pub fn new(odd_primes: &[i64], k: u32) -> Self {
        assert!(!odd_primes.is_empty(), "Need at least one odd prime");
        assert!(odd_primes.iter().all(|&p| p > 2 && p % 2 == 1),
            "All primes must be odd and > 2");

        let p_odd: i64 = odd_primes.iter().product();
        let p_two: i64 = 1 << k;
        let total = p_odd * p_two;

        // Verify pairwise coprimality
        for i in 0..odd_primes.len() {
            for j in (i+1)..odd_primes.len() {
                assert_eq!(gcd_i64(odd_primes[i], odd_primes[j]), 1,
                    "Primes must be pairwise coprime: {} and {}", odd_primes[i], odd_primes[j]);
            }
        }

        let inv2_mod_p_odd = mod_inv(2, p_odd);
        let inv_p_odd_mod_p2 = mod_inv(modd(p_odd, p_two), p_two);

        TorusConfig {
            odd_primes: odd_primes.to_vec(),
            p_odd, k, p_two, total,
            inv2_mod_p_odd, inv_p_odd_mod_p2,
        }
    }

    /// Standard (3,5,7) × 2^k torus
    pub fn standard_357(k: u32) -> Self {
        Self::new(&[3, 5, 7], k)
    }

    /// Safe Basis torus (3,5,7,11,13) × 2^k
    pub fn safe_basis(k: u32) -> Self {
        Self::new(&[3, 5, 7, 11, 13], k)
    }

    /// Exact-lane torus (primes > 11) × 2^k
    pub fn exact_lanes(primes: &[i64], k: u32) -> Self {
        assert!(primes.iter().all(|&p| p > 11),
            "Exact lanes require primes > 11");
        Self::new(primes, k)
    }

    /// K-Elimination lift: given (r_odd, r_2), compute winding number
    /// and canonical representative in [0, total).
    pub fn k_elim_lift(&self, pt: TorusPoint) -> KElimResult {
        k_eliminate(pt.r_odd, self.p_odd, pt.r_2, self.p_two)
    }

    /// Project an integer n onto the torus.
    pub fn project(&self, n: i64) -> TorusPoint {
        TorusPoint {
            r_odd: modd(n, self.p_odd),
            r_2: modd(n, self.p_two),
        }
    }

    /// Decompose r_odd into per-prime residues.
    pub fn decompose_odd(&self, r_odd: i64) -> Vec<(i64, i64)> {
        self.odd_primes.iter()
            .map(|&p| (modd(r_odd, p), p))
            .collect()
    }

    /// Carry pattern: for canonical lift n, compute floor(n/p) mod p per prime.
    pub fn carry_pattern(&self, n: i64) -> Vec<i64> {
        self.odd_primes.iter()
            .map(|&p| (n / p) % p)
            .collect()
    }

    /// Iterate over all torus points.
    pub fn all_points(&self) -> impl Iterator<Item = TorusPoint> + '_ {
        (0..self.p_odd).flat_map(move |r_odd| {
            (0..self.p_two).map(move |r_2| TorusPoint::new(r_odd, r_2))
        })
    }

    /// Total state count
    pub fn state_count(&self) -> i64 { self.total }

    /// Clone with different k (dynamic-k exploration)
    pub fn with_k(&self, new_k: u32) -> Self {
        Self::new(&self.odd_primes, new_k)
    }
}

impl std::fmt::Display for TorusConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let primes_str: Vec<String> = self.odd_primes.iter().map(|p| p.to_string()).collect();
        write!(f, "({})[{}] × 2^{} [{}]  total={}",
            primes_str.join("×"), self.p_odd, self.k, self.p_two, self.total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_357() {
        let t = TorusConfig::standard_357(4);
        assert_eq!(t.p_odd, 105);
        assert_eq!(t.p_two, 16);
        assert_eq!(t.total, 1680);
        assert_eq!(mulmod(t.inv2_mod_p_odd, 2, t.p_odd), 1);
    }

    #[test]
    fn test_kelim_lift() {
        let t = TorusConfig::standard_357(4);
        let r = t.k_elim_lift(TorusPoint::new(79, 10));
        assert_eq!(r.k, 3);
        assert_eq!(r.value, 394);
    }

    #[test]
    fn test_project_roundtrip() {
        let t = TorusConfig::standard_357(4);
        let pt = t.project(394);
        assert_eq!(pt.r_odd, 79);
        assert_eq!(pt.r_2, 10);
        let lifted = t.k_elim_lift(pt);
        assert_eq!(lifted.value, 394);
    }

    #[test]
    fn test_dynamic_k() {
        let t4 = TorusConfig::standard_357(4);
        let t8 = t4.with_k(8);
        assert_eq!(t8.p_odd, 105);
        assert_eq!(t8.p_two, 256);
        assert_eq!(t8.total, 105 * 256);
    }
}
