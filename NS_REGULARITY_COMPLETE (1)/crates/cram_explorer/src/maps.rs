//! # Dynamical Maps on the CRT Torus
//!
//! Trait-based architecture for pluggable maps. Each map defines:
//! - The "true" integer operation (on Z)
//! - The torus approximation (on Z/P_odd × Z/2^k)
//! - A human-readable name
//!
//! This separation is what makes the commutativity projector work:
//! we check whether true_map composed with project equals
//! torus_map composed with project.

use crate::mod_arith::*;
use crate::torus::*;

/// A dynamical map operating on both integers and the CRT torus.
pub trait TorusMap: std::fmt::Debug {
    /// Name of the map
    fn name(&self) -> &str;

    /// The true integer operation: Z → Z
    fn true_step(&self, n: i64) -> i64;

    /// The torus approximation: R → R
    fn torus_step(&self, pt: TorusPoint, cfg: &TorusConfig) -> TorusPoint;

    /// Whether the map has a parity branch (needed for DKAM analysis)
    fn has_parity_branch(&self) -> bool { false }
}

// ============================================================
// COLLATZ MAP
// ============================================================

/// Standard Collatz: even → n/2, odd → 3n+1
#[derive(Debug)]
pub struct CollatzMap;

impl TorusMap for CollatzMap {
    fn name(&self) -> &str { "Collatz (3n+1)" }

    fn true_step(&self, n: i64) -> i64 {
        if n <= 0 { 0 }
        else if n % 2 == 0 { n / 2 }
        else { 3 * n + 1 }
    }

    fn torus_step(&self, pt: TorusPoint, cfg: &TorusConfig) -> TorusPoint {
        if pt.r_2 % 2 == 0 {
            // Even branch: ×2^{-1} on odd lanes, right-shift on 2-lane
            TorusPoint::new(
                mulmod(pt.r_odd, cfg.inv2_mod_p_odd, cfg.p_odd),
                pt.r_2 / 2,
            )
        } else {
            // Odd branch: 3r+1 on all lanes
            TorusPoint::new(
                modd(3 * pt.r_odd + 1, cfg.p_odd),
                modd(3 * pt.r_2 + 1, cfg.p_two),
            )
        }
    }

    fn has_parity_branch(&self) -> bool { true }
}

// ============================================================
// GENERALIZED AFFINE MAP: even → n/d, odd → a*n + b
// ============================================================

/// Generalized affine map: even → n/d, odd → a*n + b
/// Collatz is the special case d=2, a=3, b=1.
#[derive(Debug)]
pub struct AffineMap {
    pub name: String,
    pub d: i64,    // divisor for even branch
    pub a: i64,    // multiplier for odd branch
    pub b: i64,    // addend for odd branch
}

impl AffineMap {
    pub fn new(name: &str, d: i64, a: i64, b: i64) -> Self {
        assert!(d > 0 && gcd_i64(d, 2) > 0, "divisor must be even-related");
        AffineMap { name: name.to_string(), d, a, b }
    }

    /// 5n+1 variant
    pub fn five_n_plus_one() -> Self {
        Self::new("5n+1", 2, 5, 1)
    }

    /// 3n+5 variant
    pub fn three_n_plus_five() -> Self {
        Self::new("3n+5", 2, 3, 5)
    }
}

impl TorusMap for AffineMap {
    fn name(&self) -> &str { &self.name }

    fn true_step(&self, n: i64) -> i64 {
        if n <= 0 { 0 }
        else if n % self.d == 0 { n / self.d }
        else { self.a * n + self.b }
    }

    fn torus_step(&self, pt: TorusPoint, cfg: &TorusConfig) -> TorusPoint {
        if pt.r_2 % self.d == 0 {
            let inv_d = mod_inv(self.d, cfg.p_odd);
            TorusPoint::new(
                mulmod(pt.r_odd, inv_d, cfg.p_odd),
                pt.r_2 / self.d,
            )
        } else {
            TorusPoint::new(
                modd(self.a * pt.r_odd + self.b, cfg.p_odd),
                modd(self.a * pt.r_2 + self.b, cfg.p_two),
            )
        }
    }

    fn has_parity_branch(&self) -> bool { true }
}

// ============================================================
// PURE TORUS MAP (no integer lift — for studying torus dynamics)
// ============================================================

/// A map defined only on the torus (no integer lift).
/// Useful for studying abstract CRT dynamics without requiring
/// a commutativity diagram.
#[derive(Debug)]
pub struct PureTorusMap {
    pub name: String,
    /// Per-lane multipliers for even branch
    pub even_mults: Vec<i64>,
    /// Per-lane addends for even branch
    pub even_adds: Vec<i64>,
    /// Per-lane multipliers for odd branch
    pub odd_mults: Vec<i64>,
    /// Per-lane addends for odd branch
    pub odd_adds: Vec<i64>,
}

impl PureTorusMap {
    /// Create from per-lane specifications.
    /// `even_mults` and `odd_mults` have length = number of odd primes + 1 (for 2-lane)
    pub fn new(name: &str,
        even_mults: Vec<i64>, even_adds: Vec<i64>,
        odd_mults: Vec<i64>, odd_adds: Vec<i64>) -> Self
    {
        PureTorusMap {
            name: name.to_string(),
            even_mults, even_adds, odd_mults, odd_adds,
        }
    }
}

impl TorusMap for PureTorusMap {
    fn name(&self) -> &str { &self.name }

    fn true_step(&self, _n: i64) -> i64 {
        panic!("PureTorusMap has no integer lift — use torus_step only")
    }

    fn torus_step(&self, pt: TorusPoint, cfg: &TorusConfig) -> TorusPoint {
        if pt.r_2 % 2 == 0 {
            // Even branch — apply per-lane even operations
            // Last entry in mults/adds is for the 2-power lane
            let n_primes = cfg.odd_primes.len();
            let new_r_odd = if n_primes <= self.even_mults.len() {
                let mut r = pt.r_odd;
                // Apply the product-level operation
                let m = self.even_mults.get(0).copied().unwrap_or(1);
                let a = self.even_adds.get(0).copied().unwrap_or(0);
                r = modd(m * r + a, cfg.p_odd);
                r
            } else {
                pt.r_odd
            };
            let m2 = self.even_mults.last().copied().unwrap_or(1);
            let a2 = self.even_adds.last().copied().unwrap_or(0);
            let new_r_2 = modd(m2 * pt.r_2 + a2, cfg.p_two);
            TorusPoint::new(new_r_odd, new_r_2)
        } else {
            let n_primes = cfg.odd_primes.len();
            let new_r_odd = if n_primes <= self.odd_mults.len() {
                let m = self.odd_mults.get(0).copied().unwrap_or(1);
                let a = self.odd_adds.get(0).copied().unwrap_or(0);
                modd(m * pt.r_odd + a, cfg.p_odd)
            } else {
                pt.r_odd
            };
            let m2 = self.odd_mults.last().copied().unwrap_or(1);
            let a2 = self.odd_adds.last().copied().unwrap_or(0);
            let new_r_2 = modd(m2 * pt.r_2 + a2, cfg.p_two);
            TorusPoint::new(new_r_odd, new_r_2)
        }
    }
}

// ============================================================
// ORBIT MACHINERY
// ============================================================

/// Orbit result from forward simulation
#[derive(Clone, Debug)]
pub struct OrbitResult {
    pub start: TorusPoint,
    pub trajectory: Vec<TorusPoint>,
    pub cycle_start: Option<usize>,
    pub cycle_len: Option<usize>,
    pub steps: usize,
}

/// Forward simulate a map on the torus with cycle detection.
pub fn simulate_orbit(
    start: TorusPoint,
    map: &dyn TorusMap,
    cfg: &TorusConfig,
    max_steps: usize,
) -> OrbitResult {
    let mut visited: std::collections::HashMap<TorusPoint, usize> = std::collections::HashMap::new();
    let mut trajectory = Vec::new();
    let mut state = start;

    for step in 0..max_steps {
        if let Some(&prev_step) = visited.get(&state) {
            return OrbitResult {
                start,
                trajectory,
                cycle_start: Some(prev_step),
                cycle_len: Some(step - prev_step),
                steps: step,
            };
        }
        visited.insert(state, step);
        trajectory.push(state);
        state = map.torus_step(state, cfg);
    }

    OrbitResult {
        start, trajectory,
        cycle_start: None, cycle_len: None, steps: max_steps,
    }
}

/// Check if a torus point is in a specific cycle.
pub fn reaches_cycle(
    start: TorusPoint,
    target_cycle: &[TorusPoint],
    map: &dyn TorusMap,
    cfg: &TorusConfig,
    max_steps: usize,
) -> Option<usize> {
    let target_set: std::collections::HashSet<TorusPoint> =
        target_cycle.iter().cloned().collect();
    let mut state = start;
    for step in 0..max_steps {
        if target_set.contains(&state) {
            return Some(step);
        }
        state = map.torus_step(state, cfg);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collatz_cycle() {
        let cfg = TorusConfig::standard_357(4);
        let map = CollatzMap;
        let cycle = [
            TorusPoint::new(1, 1),
            TorusPoint::new(4, 4),
            TorusPoint::new(2, 2),
        ];
        // State (1,1) should be in the cycle at step 0
        assert_eq!(reaches_cycle(TorusPoint::new(1, 1), &cycle, &map, &cfg, 100), Some(0));
        // State (8,8) should reach it in 1 step
        assert_eq!(reaches_cycle(TorusPoint::new(8, 8), &cycle, &map, &cfg, 100), Some(1));
    }
}
