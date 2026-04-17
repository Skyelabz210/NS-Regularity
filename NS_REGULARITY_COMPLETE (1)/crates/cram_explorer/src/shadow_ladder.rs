//! # Shadow Ladder — Composite Approach to Face 13
//!
//! The shadow prime (11) as spine of a composite ladder where each rung
//! freezes one more CRT lane, ascending toward universal lane-freeze (Face 13).
//!
//! Key discovery: convergence is 100% at EVERY distance from Face 13.
//! The transition from full convergence to fixed-point void is INSTANTANEOUS.
//! Face 13 is a phase transition, not a gradient.
//!
//! The ladder proves: one bit of structure is enough. Zero bits is the void.
//! That's the Maya shell glyph — one more prime, or nothing.

use crate::mod_arith::*;
use crate::holodrive::*;
use std::collections::HashMap;

/// A rung on the shadow ladder.
#[derive(Clone, Debug)]
pub struct LadderRung {
    /// The composite value (spine × factor primes)
    pub value: i64,
    /// Label (e.g. "11×3×7")
    pub label: String,
    /// Which basis primes are factors (frozen lanes)
    pub frozen_primes: Vec<i64>,
    /// Which basis primes are NOT factors (active lanes)
    pub active_primes: Vec<i64>,
    /// Distance to Face 13 = number of active lanes
    pub distance_to_face13: usize,
    /// Residue mod P
    pub residue: i64,
    /// Whether this rung converges under Syracuse torus map
    pub converges: bool,
    /// Steps to convergence (or to cycle detection)
    pub steps: u64,
    /// If spurious: cycle length
    pub cycle_len: Option<usize>,
}

/// Result of a full shadow ladder analysis.
#[derive(Clone, Debug)]
pub struct LadderResult {
    /// The spine prime
    pub spine: i64,
    /// The torus primorial P
    pub p_odd: i64,
    /// All basis primes
    pub basis: Vec<i64>,
    /// All rungs, sorted by distance to Face 13
    pub rungs: Vec<LadderRung>,
    /// Convergence rate per distance level
    pub convergence_by_distance: Vec<(usize, u64, u64, u64)>, // (dist, conv, spur, face13)
}

/// Build and analyze a shadow ladder.
///
/// `spine`: the prime used as the ladder's backbone
/// `basis`: all primes in the torus (must include spine)
/// `max_steps`: Syracuse orbit cutoff
pub fn analyze_ladder(
    spine: i64,
    basis: &[i64],
    max_steps: u64,
) -> LadderResult {
    use std::iter::once;
    
    let p_odd: i64 = basis.iter().product();
    let others: Vec<i64> = basis.iter().filter(|&&p| p != spine).cloned().collect();
    
    let mut rungs = Vec::new();
    
    // Generate all 2^(n-1) rungs: spine × subsets of others
    let n_others = others.len();
    for mask in 0..(1u64 << n_others) {
        let mut factors = vec![spine];
        let mut label_parts = vec![spine.to_string()];
        
        for (i, &p) in others.iter().enumerate() {
            if mask & (1 << i) != 0 {
                factors.push(p);
                label_parts.push(p.to_string());
            }
        }
        
        let value: i64 = factors.iter().product();
        let residue = value % p_odd;
        let label = label_parts.join("×");
        
        let frozen: Vec<i64> = basis.iter().filter(|&&p| residue % p == 0).cloned().collect();
        let active: Vec<i64> = basis.iter().filter(|&&p| residue % p != 0).cloned().collect();
        let dist = active.len();
        
        if residue == 0 {
            // Face 13
            rungs.push(LadderRung {
                value, label, frozen_primes: frozen, active_primes: active,
                distance_to_face13: 0, residue: 0,
                converges: false, steps: 0, cycle_len: None,
            });
            continue;
        }
        
        // Syracuse torus orbit
        let syracuse_torus = |r: i64| -> i64 {
            if r <= 0 { return 0; }
            let mut m = r;
            while m % 2 == 0 && m > 0 { m /= 2; }
            if m <= 0 { return 0; }
            let val = 3 * m + 1;
            let mut result = val;
            while result % 2 == 0 { result /= 2; }
            ((result % p_odd) + p_odd) % p_odd
        };
        
        let mut visited: HashMap<i64, u64> = HashMap::new();
        let mut current = residue;
        let mut converges = false;
        let mut steps = 0u64;
        let mut cycle_len = None;
        
        for step in 0..max_steps {
            if current == 1 {
                converges = true;
                steps = step;
                break;
            }
            if let Some(&prev) = visited.get(&current) {
                // Cycle detected
                cycle_len = Some((step - prev) as usize);
                steps = step;
                break;
            }
            visited.insert(current, step);
            current = syracuse_torus(current);
        }
        
        rungs.push(LadderRung {
            value, label, frozen_primes: frozen, active_primes: active,
            distance_to_face13: dist, residue,
            converges, steps, cycle_len,
        });
    }
    
    // Sort by distance to Face 13 (ascending = closest to void first)
    rungs.sort_by_key(|r| (r.distance_to_face13, r.value));
    
    // Compute convergence by distance
    let max_dist = basis.len();
    let mut conv_by_dist = Vec::new();
    for d in 0..=max_dist {
        let at_dist: Vec<&LadderRung> = rungs.iter()
            .filter(|r| r.distance_to_face13 == d)
            .collect();
        let conv = at_dist.iter().filter(|r| r.converges).count() as u64;
        let spur = at_dist.iter().filter(|r| !r.converges && r.residue != 0).count() as u64;
        let face = at_dist.iter().filter(|r| r.residue == 0).count() as u64;
        conv_by_dist.push((d, conv, spur, face));
    }
    
    LadderResult {
        spine, p_odd, basis: basis.to_vec(), rungs, convergence_by_distance: conv_by_dist,
    }
}

/// Compare ladders across all basis primes as spines.
pub fn compare_all_spines(
    basis: &[i64],
    max_steps: u64,
) -> Vec<LadderResult> {
    basis.iter()
        .map(|&spine| analyze_ladder(spine, basis, max_steps))
        .collect()
}

/// Shadow power ladder: test 11^1, 11^2, 11^3, ... composed with basis primes.
/// The Saturn 242 = 2 × 11² connection.
pub fn shadow_power_ladder(
    basis: &[i64],
    max_power: u32,
    max_steps: u64,
) -> Vec<(i64, i64, Vec<i64>, bool, u64)> {
    let p_odd: i64 = basis.iter().product();
    let mut results = Vec::new();
    
    let syracuse_torus = |r: i64, p: i64| -> i64 {
        if r <= 0 { return 0; }
        let mut m = r;
        while m % 2 == 0 && m > 0 { m /= 2; }
        if m <= 0 { return 0; }
        let val = 3 * m + 1;
        let mut result = val;
        while result % 2 == 0 { result /= 2; }
        ((result % p) + p) % p
    };
    
    for power in 1..=max_power {
        let shadow_val = 11i64.pow(power);
        
        // shadow^power alone
        let r = shadow_val % p_odd;
        let frozen: Vec<i64> = basis.iter().filter(|&&p| r % p == 0).cloned().collect();
        
        let mut current = r;
        let mut conv = false;
        let mut steps = 0u64;
        let mut visited = std::collections::HashSet::new();
        
        if r > 0 {
            for step in 0..max_steps {
                if current == 1 { conv = true; steps = step; break; }
                if visited.contains(&current) { steps = step; break; }
                visited.insert(current);
                current = syracuse_torus(current, p_odd);
            }
        }
        
        results.push((shadow_val, r, frozen, conv, steps));
        
        // shadow^power × each other basis prime
        for &mult in basis.iter().filter(|&&p| p != 11) {
            let val = shadow_val * mult;
            let r = val % p_odd;
            let frozen: Vec<i64> = basis.iter().filter(|&&p| r % p == 0).cloned().collect();
            
            let mut current = r;
            let mut conv = false;
            let mut steps = 0u64;
            let mut visited = std::collections::HashSet::new();
            
            if r > 0 {
                for step in 0..max_steps {
                    if current == 1 { conv = true; steps = step; break; }
                    if visited.contains(&current) { steps = step; break; }
                    visited.insert(current);
                    current = syracuse_torus(current, p_odd);
                }
            }
            
            results.push((val, r, frozen, conv, steps));
        }
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shadow_ladder_p15015() {
        let basis = vec![3, 5, 7, 11, 13];
        let result = analyze_ladder(11, &basis, 500_000);
        
        // All non-Face-13 rungs should converge
        let non_face = result.rungs.iter().filter(|r| r.residue != 0);
        let all_converge = non_face.clone().all(|r| r.converges);
        assert!(all_converge, "Shadow ladder should have 100% convergence on P=15015");
        
        // Exactly one Face 13 (the full product)
        let face13_count = result.rungs.iter().filter(|r| r.residue == 0).count();
        assert_eq!(face13_count, 1);
    }
    
    #[test]
    fn test_all_spines_equal() {
        let basis = vec![3, 5, 7, 11, 13];
        let ladders = compare_all_spines(&basis, 500_000);
        
        // Every spine should produce same convergence profile
        for ladder in &ladders {
            let non_face: Vec<&LadderRung> = ladder.rungs.iter()
                .filter(|r| r.residue != 0).collect();
            let all_conv = non_face.iter().all(|r| r.converges);
            assert!(all_conv, "Spine {} should have 100% convergence", ladder.spine);
        }
    }
}
