//! GearManifold: Cascaded CRT lanes with residue-native state.
//! SafeAnchor: O(1) K-vort via running residues.

use crate::crt::{gcd, k_eliminate};

/// Single magnitude lane: tracks residue and winding count for one modulus.
#[derive(Debug, Clone)]
pub struct MagnitudeLane {
    pub modulus: u128,
    pub residue: u128,
    pub magnitude: u128, // winding count (how many full wraps)
}

impl MagnitudeLane {
    pub fn new(modulus: u128) -> Self {
        Self { modulus, residue: 0, magnitude: 0 }
    }

    pub fn from_value(modulus: u128, value: u128) -> Self {
        Self {
            modulus,
            residue: value % modulus,
            magnitude: value / modulus,
        }
    }

    /// Add delta, return overflow (amount that caused wrap).
    pub fn add(&mut self, delta: i128) -> i128 {
        let old = self.residue as i128;
        let m = self.modulus as i128;
        let new_val = old + delta;

        if new_val >= 0 && new_val < m {
            self.residue = new_val as u128;
            0
        } else {
            self.residue = new_val.rem_euclid(m) as u128;
            let wraps = if delta >= 0 {
                (old + delta) / m
            } else {
                -((m - 1 - old - delta - 1).max(0) / m)
            };
            self.magnitude = (self.magnitude as i128 + wraps) as u128;
            wraps
        }
    }
}

/// GearManifold: multiple cascaded CRT lanes.
/// Reconstruction is OUTPUT ONLY — never on hot path.
#[derive(Debug, Clone)]
pub struct GearManifold {
    pub lanes: Vec<MagnitudeLane>,
    pub moduli: Vec<u128>,
    pub product: u128,
}

impl GearManifold {
    pub fn new(moduli: &[u128]) -> Self {
        // Verify pairwise coprimality
        for i in 0..moduli.len() {
            for j in (i + 1)..moduli.len() {
                assert_eq!(gcd(moduli[i], moduli[j]), 1,
                    "Moduli must be pairwise coprime: {} and {} share factor {}",
                    moduli[i], moduli[j], gcd(moduli[i], moduli[j]));
            }
        }
        let product = moduli.iter().product();
        Self {
            lanes: moduli.iter().map(|&m| MagnitudeLane::new(m)).collect(),
            moduli: moduli.to_vec(),
            product,
        }
    }

    pub fn from_value(moduli: &[u128], value: u128) -> Self {
        for i in 0..moduli.len() {
            for j in (i + 1)..moduli.len() {
                assert_eq!(gcd(moduli[i], moduli[j]), 1);
            }
        }
        let product = moduli.iter().product();
        Self {
            lanes: moduli.iter().map(|&m| MagnitudeLane::from_value(m, value)).collect(),
            moduli: moduli.to_vec(),
            product,
        }
    }

    /// Current residue tuple. O(k).
    pub fn residues(&self) -> Vec<u128> {
        self.lanes.iter().map(|l| l.residue).collect()
    }

    /// Add delta in residue space. O(k).
    pub fn add(&mut self, delta: i128) {
        for lane in &mut self.lanes {
            lane.add(delta);
        }
    }

    /// Primary winding number. Lane 0 magnitude.
    pub fn k_vort(&self) -> u128 {
        self.lanes[0].magnitude
    }

    /// Residue for a specific modulus (by index).
    pub fn r(&self, idx: usize) -> u128 {
        self.lanes[idx].residue
    }
}

/// SafeAnchor: K-Elimination engine with running residues.
/// Zero reconstruction on the hot path.
#[derive(Debug, Clone)]
pub struct SafeAnchor {
    pub inner: GearManifold,
    pub m_safe: u128,       // Product of basis moduli
    pub p_anchor: u128,     // Anchor modulus (coprime to m_safe)
    pub r_safe: u128,       // Running: X mod M_safe
    pub r_anchor: u128,     // Running: X mod P_anchor
}

impl SafeAnchor {
    pub fn new(basis: &[u128], p_anchor: u128) -> Self {
        let m_safe: u128 = basis.iter().product();
        assert_eq!(gcd(m_safe, p_anchor), 1,
            "Safe product {} and anchor {} must be coprime", m_safe, p_anchor);

        Self {
            inner: GearManifold::new(basis),
            m_safe,
            p_anchor,
            r_safe: 0,
            r_anchor: 0,
        }
    }

    pub fn from_value(basis: &[u128], p_anchor: u128, value: u128) -> Self {
        let m_safe: u128 = basis.iter().product();
        assert_eq!(gcd(m_safe, p_anchor), 1);

        Self {
            inner: GearManifold::from_value(basis, value),
            m_safe,
            p_anchor,
            r_safe: value % m_safe,
            r_anchor: value % p_anchor,
        }
    }

    /// Update all residues. O(1) per lane.
    pub fn add(&mut self, delta: i128) {
        self.inner.add(delta);
        let m = self.m_safe as i128;
        let a = self.p_anchor as i128;
        self.r_safe = ((self.r_safe as i128 + delta).rem_euclid(m)) as u128;
        self.r_anchor = ((self.r_anchor as i128 + delta).rem_euclid(a)) as u128;
    }

    /// Zero-reconstruction winding number via K-Elimination.
    pub fn k_vort_fast(&self) -> Option<u128> {
        k_eliminate(self.r_safe, self.m_safe, self.r_anchor, self.p_anchor)
    }

    /// Set to specific value.
    pub fn set_value(&mut self, value: u128) {
        self.inner = GearManifold::from_value(&self.inner.moduli, value);
        self.r_safe = value % self.m_safe;
        self.r_anchor = value % self.p_anchor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnitude_lane() {
        let mut lane = MagnitudeLane::from_value(7, 23);
        assert_eq!(lane.residue, 2);   // 23 % 7 = 2
        assert_eq!(lane.magnitude, 3); // 23 / 7 = 3
        lane.add(5);
        assert_eq!(lane.residue, 0);   // (23+5) % 7 = 0
        assert_eq!(lane.magnitude, 4); // wrapped once more
    }

    #[test]
    fn test_gear_manifold() {
        let gm = GearManifold::from_value(&[3, 5, 7], 23);
        assert_eq!(gm.residues(), vec![2, 3, 2]); // 23%3=2, 23%5=3, 23%7=2
        assert_eq!(gm.product, 105);
    }

    #[test]
    fn test_safe_anchor() {
        // Basis {2,3,5}, anchor 7
        let sa = SafeAnchor::from_value(&[2, 3, 5], 7, 17);
        assert_eq!(sa.r_safe, 17 % 30);  // 17
        assert_eq!(sa.r_anchor, 17 % 7); // 3
        let k = sa.k_vort_fast().unwrap();
        // x = k * M_safe + r_safe => 17 = k*30 + 17 => k = 0
        assert_eq!(k, 0);

        let sa2 = SafeAnchor::from_value(&[2, 3, 5], 7, 47);
        let k2 = sa2.k_vort_fast().unwrap();
        // 47 = k*30 + 17 => k = 1
        assert_eq!(k2, 1);
    }

    #[test]
    fn test_coprimality_enforcement() {
        // Should panic: 6 and 4 share factor 2
        let result = std::panic::catch_unwind(|| GearManifold::new(&[6, 4]));
        assert!(result.is_err());
    }
}
