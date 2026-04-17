//! # Commutativity Projector + Dynamic-k Explorer
//!
//! The projector Π: R → {0,1} accepts state (r_odd, r_2) iff
//! project(true_map(lift(state))) == torus_map(state).
//!
//! The dynamic-k explorer sweeps k from k_min to k_max,
//! measuring consistent fraction and faithful fraction at each depth.
//! This reveals whether the faithful basin grows with 2-adic resolution.

use crate::mod_arith::*;
use crate::torus::*;
use crate::maps::*;
use crate::k_elim::*;
use std::collections::HashSet;

/// Result of projector analysis for a single state.
#[derive(Clone, Debug)]
pub struct ProjectorResult {
    pub point: TorusPoint,
    pub consistent: bool,
    pub winding_k: i64,
    pub canonical_lift: i64,
    pub failure_detail: Option<String>,
}

/// Apply the commutativity projector to a single state.
pub fn check_consistency(
    pt: TorusPoint,
    map: &dyn TorusMap,
    cfg: &TorusConfig,
) -> ProjectorResult {
    let lifted = cfg.k_elim_lift(pt);
    let n = lifted.value;
    let next_n = map.true_step(n);
    let projected = cfg.project(next_n);
    let torus_next = map.torus_step(pt, cfg);
    let consistent = projected == torus_next;

    let failure_detail = if !consistent {
        Some(format!(
            "lift={}→map→{}→proj{}  vs  torus{}",
            n, next_n, projected, torus_next
        ))
    } else {
        None
    };

    ProjectorResult {
        point: pt,
        consistent,
        winding_k: lifted.k,
        canonical_lift: n,
        failure_detail,
    }
}

/// Full projector sweep: enumerate all states, classify each.
#[derive(Clone, Debug)]
pub struct ProjectorSweep {
    pub config: TorusConfig,
    pub map_name: String,
    pub consistent_count: u64,
    pub inconsistent_count: u64,
    pub consistent_fraction: Frac,
    pub k_distribution: Vec<u64>,     // consistent count per winding k
    pub consistent_states: Vec<ProjectorResult>,
}

/// Run the projector over the entire torus.
pub fn projector_sweep(
    map: &dyn TorusMap,
    cfg: &TorusConfig,
) -> ProjectorSweep {
    let mut consistent_count = 0u64;
    let mut inconsistent_count = 0u64;
    let mut k_dist = vec![0u64; cfg.p_two as usize];
    let mut consistent_states = Vec::new();

    for pt in cfg.all_points() {
        let result = check_consistency(pt, map, cfg);
        if result.consistent {
            consistent_count += 1;
            k_dist[result.winding_k as usize] += 1;
            consistent_states.push(result);
        } else {
            inconsistent_count += 1;
        }
    }

    let total = consistent_count + inconsistent_count;
    let g = gcd_u64(consistent_count, total);
    let frac = Frac::new(
        (consistent_count / g) as i64,
        (total / g) as i64,
    );

    ProjectorSweep {
        config: cfg.clone(),
        map_name: map.name().to_string(),
        consistent_count,
        inconsistent_count,
        consistent_fraction: frac,
        k_distribution: k_dist,
        consistent_states,
    }
}

/// Dynamic-k exploration: sweep k from k_min to k_max.
/// For each k, compute consistent count and faithful count.
#[derive(Clone, Debug)]
pub struct DynamicKResult {
    pub k: u32,
    pub p_two: i64,
    pub total_states: i64,
    pub consistent: u64,
    pub consistent_fraction: Frac,
    pub faithful: u64,
    pub faithful_over_consistent: Frac,
}

/// Run dynamic-k sweep.
pub fn dynamic_k_sweep(
    map: &dyn TorusMap,
    base_primes: &[i64],
    target_cycle: &[TorusPoint],
    k_min: u32,
    k_max: u32,
    max_orbit_steps: usize,
) -> Vec<DynamicKResult> {
    let mut results = Vec::new();

    for k in k_min..=k_max {
        let cfg = TorusConfig::new(base_primes, k);

        // Adjust target cycle for new k (the cycle points need valid r_2 values)
        // For Collatz 1-2-4: these are small enough to fit any k ≥ 3
        let cycle_set: HashSet<TorusPoint> = target_cycle.iter().cloned().collect();

        let mut consistent = 0u64;
        let mut faithful = 0u64;

        for pt in cfg.all_points() {
            let lifted = cfg.k_elim_lift(pt);
            let n = lifted.value;
            let next_n = map.true_step(n);
            let projected = cfg.project(next_n);
            let torus_next = map.torus_step(pt, &cfg);

            if projected == torus_next {
                consistent += 1;
                // Check reachability to target cycle
                if reaches_cycle(pt, target_cycle, map, &cfg, max_orbit_steps).is_some() {
                    faithful += 1;
                }
            }
        }

        let total = cfg.total as u64;
        let gc = gcd_u64(consistent, total);
        let gf = if faithful > 0 && consistent > 0 {
            gcd_u64(faithful, consistent)
        } else { 1 };

        results.push(DynamicKResult {
            k,
            p_two: cfg.p_two,
            total_states: cfg.total,
            consistent,
            consistent_fraction: Frac::new(
                (consistent / gc) as i64,
                (total / gc) as i64,
            ),
            faithful,
            faithful_over_consistent: Frac::new(
                (faithful / gf) as i64,
                (consistent / gf) as i64,
            ),
        });
    }

    results
}

/// Multi-step consistency: check how many steps a state stays consistent.
pub fn consistency_depth(
    start: TorusPoint,
    map: &dyn TorusMap,
    cfg: &TorusConfig,
    max_steps: usize,
) -> (usize, Option<String>) {
    let mut state = start;
    for step in 0..max_steps {
        let result = check_consistency(state, map, cfg);
        if !result.consistent {
            return (step, result.failure_detail);
        }
        state = map.torus_step(state, cfg);
    }
    (max_steps, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projector_357_k4() {
        let cfg = TorusConfig::standard_357(4);
        let sweep = projector_sweep(&CollatzMap, &cfg);
        assert_eq!(sweep.consistent_count, 1264);
        assert_eq!(sweep.consistent_fraction.num, 79);
        assert_eq!(sweep.consistent_fraction.den, 105);
    }
}
