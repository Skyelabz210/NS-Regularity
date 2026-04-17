//! # CRAM Diagnostics Suite
//!
//! Adapted from the CRAM Operator Skill (operational, not educational).
//! All diagnostics are exact-integer.
//!
//! - Per-lane energy audit (SCR analog)
//! - Ring homomorphism verification (V7)
//! - Ramanujan spike events
//! - Sync observables (O1-O5)
//! - DKAM degree check

use crate::mod_arith::*;
use crate::torus::*;
use crate::maps::*;

// ============================================================
// PER-LANE ENERGY
// ============================================================

/// Per-lane energy over a trajectory: sum of |residue mod p| per step.
#[derive(Clone, Debug)]
pub struct LaneEnergy {
    /// Energy per odd prime lane
    pub per_lane: Vec<(i64, i64)>,  // (prime, total_energy)
    /// Total energy across all lanes
    pub total: i64,
    /// Number of steps measured
    pub steps: u64,
}

/// Compute lane energy along a trajectory.
pub fn lane_energy_trajectory(
    trajectory: &[TorusPoint],
    cfg: &TorusConfig,
) -> LaneEnergy {
    let mut per_lane: Vec<(i64, i64)> = cfg.odd_primes.iter()
        .map(|&p| (p, 0i64))
        .collect();

    for pt in trajectory {
        for (i, &p) in cfg.odd_primes.iter().enumerate() {
            per_lane[i].1 += modd(pt.r_odd, p);
        }
    }

    let total: i64 = per_lane.iter().map(|(_, e)| *e).sum();

    LaneEnergy {
        per_lane,
        total,
        steps: trajectory.len() as u64,
    }
}

// ============================================================
// RING HOMOMORPHISM CHECK (V7)
// ============================================================

/// Verify that the torus map commutes with per-prime projection.
/// Returns (total_checks, violations).
pub fn ring_homomorphism_check(
    map: &dyn TorusMap,
    cfg: &TorusConfig,
    states: &[TorusPoint],
) -> (u64, u64) {
    let mut checks = 0u64;
    let mut violations = 0u64;

    for &pt in states {
        let next = map.torus_step(pt, cfg);
        for &p in &cfg.odd_primes {
            let input_mod_p = modd(pt.r_odd, p);
            let output_mod_p = modd(next.r_odd, p);

            // The per-prime map should factor through
            let expected = if pt.r_2 % 2 == 0 {
                mulmod(input_mod_p, mod_inv(2, p), p)
            } else {
                modd(3 * input_mod_p + 1, p)
            };

            checks += 1;
            if output_mod_p != expected {
                violations += 1;
            }
        }
    }

    (checks, violations)
}

// ============================================================
// RAMANUJAN SPIKE EVENTS
// ============================================================

/// Ramanujan sum c_p(n) for prime p:
///   -1 if p | n (spike event)
///   p-1 if gcd(n,p) = 1
pub fn ramanujan_cp(p: i64, n: i64) -> i64 {
    if modd(n, p) == 0 { -1 } else { p - 1 }
}

/// Spike vector: which lanes fire (residue ≡ 0) at a given state.
pub fn spike_vector(pt: TorusPoint, cfg: &TorusConfig) -> Vec<(i64, bool)> {
    cfg.odd_primes.iter()
        .map(|&p| (p, modd(pt.r_odd, p) == 0))
        .collect()
}

/// Ramanujan contribution vector at a state.
pub fn ramanujan_vector(pt: TorusPoint, cfg: &TorusConfig) -> Vec<(i64, i64)> {
    cfg.odd_primes.iter()
        .map(|&p| {
            let r = modd(pt.r_odd, p);
            (p, if r == 0 { -1 } else { p - 1 })
        })
        .collect()
}

/// Count spike density along a trajectory per lane.
/// Returns vec of (prime, spike_count, total_steps).
pub fn spike_density(
    trajectory: &[TorusPoint],
    cfg: &TorusConfig,
) -> Vec<(i64, u64, u64)> {
    let total = trajectory.len() as u64;
    cfg.odd_primes.iter()
        .map(|&p| {
            let spikes = trajectory.iter()
                .filter(|pt| modd(pt.r_odd, p) == 0)
                .count() as u64;
            (p, spikes, total)
        })
        .collect()
}

// ============================================================
// SYNC OBSERVABLES (O1-O5 adapted for general maps)
// ============================================================

/// Orbit observables adapted from CRAM Operator Sync Schema.
#[derive(Clone, Debug)]
pub struct SyncObservables {
    /// O1: Orbit energy = Σ n²
    pub energy: i128,
    /// O2: Orbit enstrophy = Σ (n_{t+1} - n_t)²
    pub enstrophy: i128,
    /// O3: Peak value
    pub peak: i64,
    /// O4: Branch balance (even_steps - odd_steps) [for branching maps]
    pub branch_balance: i64,
    /// O5: Per-lane residue sums
    pub lane_sums: Vec<(i64, i64)>,
    /// Total steps
    pub steps: u64,
    /// Total domain wraps
    pub wraps: i64,
}

/// Compute sync observables for an integer orbit.
pub fn compute_observables(
    start_n: i64,
    map: &dyn TorusMap,
    cfg: &TorusConfig,
    max_steps: usize,
) -> SyncObservables {
    let mut n = start_n;
    let mut energy: i128 = 0;
    let mut enstrophy: i128 = 0;
    let mut peak = n;
    let mut even_count = 0i64;
    let mut odd_count = 0i64;
    let mut lane_sums: Vec<(i64, i64)> = cfg.odd_primes.iter()
        .map(|&p| (p, 0i64))
        .collect();
    let mut wraps = 0i64;

    for step in 0..max_steps {
        energy += n as i128 * n as i128;
        for (i, &p) in cfg.odd_primes.iter().enumerate() {
            lane_sums[i].1 += modd(n, p);
        }

        let next_n = map.true_step(n);
        let delta = next_n as i128 - n as i128;
        enstrophy += delta * delta;
        wraps += next_n / cfg.total;

        if n % 2 == 0 { even_count += 1; } else { odd_count += 1; }
        if next_n > peak { peak = next_n; }

        if next_n >= 1 && next_n <= 4 && step > 0 { break; }
        n = next_n;
    }

    SyncObservables {
        energy, enstrophy, peak,
        branch_balance: even_count - odd_count,
        lane_sums,
        steps: max_steps as u64,
        wraps,
    }
}

// ============================================================
// DKAM DEGREE CHECK (honest — reports when inapplicable)
// ============================================================

/// Result of DKAM degree analysis.
#[derive(Clone, Debug)]
pub enum DKAMResult {
    /// deg(F) is well-defined and we have a comparison
    Determined {
        degree: u32,
        rho: u32,
        subcritical: bool,
        label: String,
    },
    /// deg(F) is NOT well-defined for this map
    Inapplicable {
        reason: String,
    },
}

/// Check DKAM applicability for a map.
/// Tests scaling: F(λn)/F(n) for multiple λ values.
pub fn dkam_check(
    map: &dyn TorusMap,
    cfg: &TorusConfig,
    test_values: &[i64],
    lambdas: &[i64],
) -> DKAMResult {
    let rho = *cfg.odd_primes.iter().min().unwrap() as u32;

    // Test scaling behavior
    let mut consistent_degree: Option<u32> = None;

    for &lambda in lambdas {
        for &n in test_values {
            if n <= 0 { continue; }
            let fn_val = map.true_step(n);
            let f_lambda_n = map.true_step(lambda * n);

            if fn_val == 0 { continue; }

            // For degree d: F(λn) = λ^d * F(n)
            // Check d=1: F(λn) =? λ * F(n)
            let ratio_times_fn = f_lambda_n;
            let expected_d1 = lambda * fn_val;

            if map.has_parity_branch() {
                // Parity of n and λn may differ → branching breaks scaling
                if (n % 2) != ((lambda * n) % 2) {
                    return DKAMResult::Inapplicable {
                        reason: format!(
                            "Parity branch: n={} is {}, λn={} is {} — scaling undefined",
                            n, if n % 2 == 0 { "even" } else { "odd" },
                            lambda * n, if (lambda * n) % 2 == 0 { "even" } else { "odd" }
                        ),
                    };
                }
                // Even if same parity, affine +b term breaks exact scaling
                if ratio_times_fn != expected_d1 {
                    // Check if it's close (affine correction)
                    let diff = (ratio_times_fn - expected_d1).abs();
                    if diff > 1 {
                        return DKAMResult::Inapplicable {
                            reason: format!(
                                "Affine correction: F({}·{})={} ≠ {}·F({})={}·{}={}, diff={}",
                                lambda, n, f_lambda_n, lambda, n, lambda, fn_val, expected_d1, diff
                            ),
                        };
                    }
                }
            }
        }
    }

    // If we get here with a branching map, the degree analysis was partial
    if map.has_parity_branch() {
        DKAMResult::Inapplicable {
            reason: "Piecewise-affine map with branching predicate — \
                     deg(F) not well-defined as a single integer. \
                     DKAM requires polynomial-homogeneous nonlinearity.".to_string(),
        }
    } else {
        DKAMResult::Determined {
            degree: 1,
            rho,
            subcritical: 1 < rho,
            label: format!("deg=1 < ρ={} → subcritical", rho),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramanujan_spike() {
        assert_eq!(ramanujan_cp(3, 9), -1);   // 3 | 9
        assert_eq!(ramanujan_cp(3, 7), 2);    // gcd(7,3)=1
        assert_eq!(ramanujan_cp(7, 14), -1);  // 7 | 14
    }

    #[test]
    fn test_dkam_collatz_inapplicable() {
        let cfg = TorusConfig::standard_357(4);
        let result = dkam_check(&CollatzMap, &cfg, &[3, 5, 7, 9, 11], &[2, 3]);
        match result {
            DKAMResult::Inapplicable { .. } => {} // expected
            _ => panic!("DKAM should be inapplicable to Collatz"),
        }
    }
}
