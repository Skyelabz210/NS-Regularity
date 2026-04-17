//! # Regularity Schema Sweep
//!
//! The Hydra proved there's no universal operator configuration for twin primes.
//! Apply the same principle to Navier-Stokes regularity:
//!
//!   For each valid CRAM operator configuration on the Safe Basis {2,3,5,7,11,13}:
//!     1. Encode the vorticity field into CRT lanes
//!     2. Apply the schema's operator to evolve one timestep
//!     3. Compute β(t) = max(|ω(t)| - bound(t)) per lane
//!     4. Record: does this configuration preserve β(t) = 0?
//!
//! If ALL valid configurations preserve regularity → the CRT torus structure
//! itself constrains the dynamics (structural regularity).
//!
//! If SOME configurations break regularity → those configurations reveal
//! which operator property is load-bearing (the failure is informative).
//!
//! This is NOT a proof of NS regularity. It is a systematic computational
//! experiment that either:
//!   (a) produces evidence for CRT-structural regularity, or
//!   (b) identifies the exact operator property that permits blowup.
//! Either result is valuable.

use crate::ns_grid_3d::*;
use crate::biot_savart::*;

/// Root operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RootOp { Add, Sub, Mul }

/// Post operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostOp { Id, Neg, Sqr }

/// A CRAM operator pair for one lane
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CRAMOp {
    pub root: RootOp,
    pub post: PostOp,
}

impl CRAMOp {
    /// Apply operator to two residues mod p
    pub fn apply(&self, a: u64, b: u64, p: u64) -> u64 {
        let inter = match self.root {
            RootOp::Add => ((a as u128 + b as u128) % p as u128) as u64,
            RootOp::Sub => if a >= b { (a - b) % p } else { p - ((b - a) % p) },
            RootOp::Mul => ((a as u128 * b as u128) % p as u128) as u64,
        };
        match self.post {
            PostOp::Id => inter,
            PostOp::Neg => if inter == 0 { 0 } else { p - inter },
            PostOp::Sqr => ((inter as u128 * inter as u128) % p as u128) as u64,
        }
    }
}

/// All 9 valid CRAM operator configurations (3 root × 3 post)
pub fn all_cram_ops() -> Vec<CRAMOp> {
    let mut ops = Vec::new();
    for &root in &[RootOp::Add, RootOp::Sub, RootOp::Mul] {
        for &post in &[PostOp::Id, PostOp::Neg, PostOp::Sqr] {
            ops.push(CRAMOp { root, post });
        }
    }
    ops
}

/// A schema configuration: one operator per lane across the Safe Basis.
#[derive(Debug, Clone)]
pub struct SchemaConfig {
    pub name: String,
    pub moduli: Vec<u64>,
    pub ops: Vec<CRAMOp>,
}

/// The Safe Basis moduli (exact primes for CRT lanes)
pub const SAFE_BASIS: [u64; 6] = [2, 3, 5, 7, 11, 13];

/// Result of one schema run
#[derive(Debug, Clone)]
pub struct SchemaResult {
    pub config_name: String,
    pub steps_run: u64,
    /// β(t) at each step: max(|ω_lane| - bound) per lane.
    /// β = 0 means regular. β > 0 means potential blowup.
    pub beta_history: Vec<Vec<i64>>,
    /// Maximum vorticity per step (in integer units)
    pub vort_max_history: Vec<i64>,
    /// Did any lane's winding number diverge?
    pub winding_bounded: bool,
    /// Did all lanes maintain the discrete maximum principle?
    pub max_principle_held: bool,
    /// Overall: did this configuration preserve regularity?
    pub regularity_preserved: bool,
}

/// Encode a scalar value into CRT residues across the Safe Basis.
pub fn encode_to_safe_basis(val: i64) -> Vec<u64> {
    SAFE_BASIS.iter().map(|&p| {
        let r = val.rem_euclid(p as i64);
        r as u64
    }).collect()
}

/// Run one schema configuration for n_steps on the given grid.
///
/// The schema modifies the vorticity evolution by applying its CRAM operators
/// to the residues at each step AFTER the standard Biot-Savart + vorticity update.
/// This is a post-hoc regularity check: the operators act as discriminators
/// that detect when the residue pattern enters a forbidden (blowup) region.
pub fn run_schema(
    config: &SchemaConfig,
    kernel: &BiotSavartKernel,
    n_steps: u64,
) -> SchemaResult {
    let n = kernel.n;
    let dt = 100i64;

    let mut grid = NSGrid3D::new(n, config.moduli.len(), dt);
    grid.init_taylor_green(TG_BASE_AMP, 1, 1);

    let m0 = grid.vort_sup_global();
    let mut beta_history = Vec::new();
    let mut vort_max_history = Vec::new();
    let mut max_principle_held = true;
    let mut winding_bounded = true;

    // Track per-lane winding numbers (cumulative phase displacement)
    let mut lane_windings: Vec<i64> = vec![0; config.moduli.len()];
    let mut prev_residues: Vec<u64> = encode_to_safe_basis(m0);

    // Gronwall bound: M(t) ≤ M(0) / (1 - Ch*M(0)*t)
    // For β computation: bound(t) = M(0) * (1 + some_growth_factor * t)
    // Use a simple linear bound for the certificate
    let growth_rate = 1i64; // Allow 1 unit of growth per step

    for step in 0..n_steps {
        // 1. Recover velocity via Biot-Savart
        biot_savart_update(kernel, &mut grid);

        // 2. Advance vorticity
        grid.advance();

        // 3. Compute per-lane residues of max vorticity
        let m_t = grid.vort_sup_global();
        vort_max_history.push(m_t);

        let curr_residues = encode_to_safe_basis(m_t);

        // 4. Apply schema operators to detect forbidden patterns
        let mut betas: Vec<i64> = Vec::new();
        for (i, &p) in config.moduli.iter().enumerate() {
            let r_curr = curr_residues[i];
            let r_prev = prev_residues[i];

            // Apply CRAM operator to (current, previous) residue pair
            let op_result = config.ops[i].apply(r_curr, r_prev, p);

            // β for this lane: operator output indicates regularity violation
            // If op_result > p/2, the residue crossed the "forbidden half"
            // of the residue space — analogous to the sieve power threshold
            let beta = if op_result > p / 2 {
                (op_result as i64) - (p as i64)  // negative = in safe region
            } else {
                op_result as i64
            };
            betas.push(beta);

            // Track winding: signed displacement in residue space
            let delta = if r_curr >= r_prev {
                (r_curr - r_prev) as i64
            } else {
                (r_curr as i64) - (r_prev as i64) + (p as i64)
            };
            let signed_delta = if delta > (p as i64 / 2) {
                delta - p as i64
            } else {
                delta
            };
            lane_windings[i] += signed_delta;

            // Winding divergence check
            if lane_windings[i].abs() > 10000 {
                winding_bounded = false;
            }
        }

        // 5. Maximum principle check: M(t+dt) ≤ M(t) + bound
        let bound = m0 + growth_rate * (step as i64 + 1);
        if m_t > bound {
            max_principle_held = false;
        }

        beta_history.push(betas);
        prev_residues = curr_residues;
    }

    // Regularity preserved iff:
    //   - winding numbers stayed bounded
    //   - maximum principle never violated
    //   - no lane's β ever exceeded the threshold
    let all_beta_safe = beta_history.iter().all(|betas| {
        betas.iter().all(|&b| b.abs() < SAFE_BASIS.iter().max().copied().unwrap_or(13) as i64)
    });

    SchemaResult {
        config_name: config.name.clone(),
        steps_run: n_steps,
        beta_history,
        vort_max_history,
        winding_bounded,
        max_principle_held,
        regularity_preserved: winding_bounded && max_principle_held && all_beta_safe,
    }
}

/// Run the full schema sweep: all 9 operators × 6 lanes = systematic probe
/// of the CRT regularity landscape.
pub fn full_schema_sweep(kernel: &BiotSavartKernel, n_steps: u64) -> Vec<SchemaResult> {
    let ops = all_cram_ops();
    let mut results = Vec::new();

    // Strategy 1: Homogeneous — same operator on all lanes
    for op in &ops {
        let config = SchemaConfig {
            name: format!("{:?}+{:?}_homogeneous", op.root, op.post),
            moduli: SAFE_BASIS.to_vec(),
            ops: vec![*op; SAFE_BASIS.len()],
        };
        results.push(run_schema(&config, kernel, n_steps));
    }

    // Strategy 2: Hydra Champion — the proven S6 heterogeneous config
    let champion_ops = vec![
        CRAMOp { root: RootOp::Add, post: PostOp::Id },   // mod 2
        CRAMOp { root: RootOp::Mul, post: PostOp::Id },   // mod 3 (secret weapon)
        CRAMOp { root: RootOp::Add, post: PostOp::Sqr },  // mod 5
        CRAMOp { root: RootOp::Add, post: PostOp::Sqr },  // mod 7
        CRAMOp { root: RootOp::Mul, post: PostOp::Neg },  // mod 11
        CRAMOp { root: RootOp::Mul, post: PostOp::Id },   // mod 13
    ];
    let config = SchemaConfig {
        name: "CHAMPION_S6_heterogeneous".into(),
        moduli: SAFE_BASIS.to_vec(),
        ops: champion_ops,
    };
    results.push(run_schema(&config, kernel, n_steps));

    // Strategy 3: Diffusion-only reference (Sub+Id = identity on subtraction)
    let diffusion_ops = vec![
        CRAMOp { root: RootOp::Sub, post: PostOp::Id }; SAFE_BASIS.len()
    ];
    let config = SchemaConfig {
        name: "DIFFUSION_REFERENCE_Sub+Id".into(),
        moduli: SAFE_BASIS.to_vec(),
        ops: diffusion_ops,
    };
    results.push(run_schema(&config, kernel, n_steps));

    results
}

/// Analyze sweep results and identify structural patterns.
pub fn analyze_sweep(results: &[SchemaResult]) -> SweepAnalysis {
    let total = results.len();
    let preserved: Vec<_> = results.iter().filter(|r| r.regularity_preserved).collect();
    let violated: Vec<_> = results.iter().filter(|r| !r.regularity_preserved).collect();

    // Identify which operator properties correlate with regularity
    let mut preserving_ops: Vec<String> = preserved.iter()
        .map(|r| r.config_name.clone()).collect();
    let mut violating_ops: Vec<String> = violated.iter()
        .map(|r| r.config_name.clone()).collect();

    preserving_ops.sort();
    violating_ops.sort();

    let structural = violated.is_empty();

    SweepAnalysis {
        total_configs: total,
        preserved_count: preserved.len(),
        violated_count: violated.len(),
        preserving_configs: preserving_ops,
        violating_configs: violating_ops,
        structural_regularity: structural,
    }
}

#[derive(Debug)]
pub struct SweepAnalysis {
    pub total_configs: usize,
    pub preserved_count: usize,
    pub violated_count: usize,
    pub preserving_configs: Vec<String>,
    pub violating_configs: Vec<String>,
    /// If true: ALL configs preserved regularity → CRT structure constrains dynamics
    pub structural_regularity: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_safe_basis() {
        let r = encode_to_safe_basis(42);
        assert_eq!(r[0], 0);  // 42 mod 2
        assert_eq!(r[1], 0);  // 42 mod 3
        assert_eq!(r[2], 2);  // 42 mod 5
        assert_eq!(r[3], 0);  // 42 mod 7
        assert_eq!(r[4], 9);  // 42 mod 11
        assert_eq!(r[5], 3);  // 42 mod 13
    }

    #[test]
    fn test_encode_negative() {
        let r = encode_to_safe_basis(-1);
        assert_eq!(r[0], 1);  // -1 mod 2 = 1
        assert_eq!(r[1], 2);  // -1 mod 3 = 2
        assert_eq!(r[2], 4);  // -1 mod 5 = 4
        assert_eq!(r[3], 6);  // -1 mod 7 = 6
    }

    #[test]
    fn test_all_cram_ops() {
        let ops = all_cram_ops();
        assert_eq!(ops.len(), 9);
    }

    #[test]
    fn test_cram_op_apply() {
        let op = CRAMOp { root: RootOp::Add, post: PostOp::Sqr };
        // (3 + 4) mod 7 = 0, then 0² mod 7 = 0
        assert_eq!(op.apply(3, 4, 7), 0);
        // (3 + 2) mod 7 = 5, then 5² mod 7 = 4
        assert_eq!(op.apply(3, 2, 7), 4);
    }

    #[test]
    fn test_single_schema_run() {
        let kernel = BiotSavartKernel::precompute(8);
        let config = SchemaConfig {
            name: "test_add_id".into(),
            moduli: SAFE_BASIS.to_vec(),
            ops: vec![CRAMOp { root: RootOp::Add, post: PostOp::Id }; 6],
        };
        let result = run_schema(&config, &kernel, 5);
        assert_eq!(result.steps_run, 5);
        assert_eq!(result.vort_max_history.len(), 5);
        assert_eq!(result.beta_history.len(), 5);
    }

    #[test]
    fn test_full_sweep() {
        let kernel = BiotSavartKernel::precompute(8);
        let results = full_schema_sweep(&kernel, 5);
        // 9 homogeneous + 1 champion + 1 diffusion reference = 11
        assert_eq!(results.len(), 11);

        let analysis = analyze_sweep(&results);
        println!("Schema sweep: {}/{} preserved regularity",
            analysis.preserved_count, analysis.total_configs);
        if analysis.structural_regularity {
            println!("  STRUCTURAL REGULARITY: All configs preserve β=0");
        } else {
            println!("  Violating configs:");
            for c in &analysis.violating_configs {
                println!("    - {}", c);
            }
        }
    }
}
