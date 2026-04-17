//! # Schema Program: 9-Slot NS Regularity Search
//!
//! Every CRT-NS configuration is a tuple:
//!   Σ = (L, P, D, S, B, K, C, M, T)
//! where each slot is independently swappable under the homogeneity invariant.
//!
//! The search objective:
//!   J(Σ) = λ₁·sup_N R(N) + λ₂·sup_N B_N(T) + λ₃·sup_N Ĉ_h(N)
//!        + λ₄·sup_N D_N(T) + λ₅·sup_N W_N(T)/capacity
//!
//! Minimize J(Σ) over admissible schemas Σ ∈ H.

use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 1: OPERATOR ALPHABET
// ═══════════════════════════════════════════════════════════════════════════════

/// Primitive operators in the chain alphabet.
/// A chain is a word over this alphabet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    P, // Projection / Biot-Savart velocity recovery
    T, // Transport operator (advection)
    D, // Diffusion operator
    S, // Stretching operator (the dangerous one)
    K, // K-Elimination safety / winding monitor
    H, // Hydra gating / interaction filter
    A, // Anchor pullback / orbit step
    R, // Reconstruction / rebalance
    E, // E8 / geometric probe
    M, // Measurement / logging step
}

impl Op {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'P' => Some(Op::P), 'T' => Some(Op::T), 'D' => Some(Op::D),
            'S' => Some(Op::S), 'K' => Some(Op::K), 'H' => Some(Op::H),
            'A' => Some(Op::A), 'R' => Some(Op::R), 'E' => Some(Op::E),
            'M' => Some(Op::M), _ => None,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Op::P => 'P', Op::T => 'T', Op::D => 'D', Op::S => 'S',
            Op::K => 'K', Op::H => 'H', Op::A => 'A', Op::R => 'R',
            Op::E => 'E', Op::M => 'M',
        }
    }
}

/// Parse a chain string like "PTDSPKM" into operator sequence.
pub fn parse_chain(s: &str) -> Vec<Op> {
    s.chars().filter_map(Op::from_char).collect()
}

/// Format a chain as a string.
pub fn format_chain(chain: &[Op]) -> String {
    chain.iter().map(|o| o.to_char()).collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 2: SLOT DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Slot L: Lane schema — modulus architecture.
#[derive(Debug, Clone)]
pub struct LaneSchema {
    pub lane_type: LaneType,
    /// Layers of moduli (e.g., [[2,3,5],[7,11,13],[17,19,23]])
    pub layers: Vec<Vec<u64>>,
    /// Anchor moduli for K-Elimination bridges
    pub anchors: Vec<u64>,
    /// Band mode: how composite coprimality is exploited
    pub band_mode: BandMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneType {
    HPrime,       // Homogeneous prime lanes only
    HCompBand,    // Homogeneous composite coprime bands
    HTriad,       // Three-layer honeycomb, identical template per component
    HAnchorPull,  // Homogeneous lanes with anchor progression pullback
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandMode {
    PrimeOnly,          // No composite exploitation
    CompositeOptional,  // Composite bands available but not required
    CompositeFused,     // Composite bands mandatory and fused
}

/// Slot P: Projection / velocity recovery schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionSchema {
    SpectralBiotSavart,           // Fourier-space exact Biot-Savart
    ExactPseudoSpectral,          // Exact integer pseudo-spectral
    LocalStencil,                 // Stencil-based approximation
    HybridWithExactPostCorrect,   // Hybrid with exact post-correction
}

/// Slot D: Derivative stencil schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivativeSchema {
    Central2,               // 2nd-order central difference
    SkewSymmetricTransport, // Skew-symmetric transport stencil
    ConservativeVorticity,  // Conservative vorticity form
    MonotoneDiffusive,      // Monotone-diffusive hybrid (3D)
}

/// Slot S: Stretching operator schema — WHERE THE PROBLEM LIVES.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StretchingSchema {
    Direct,              // Direct componentwise stretching
    Projected,           // Projected stretching
    BoundedBand,         // Bounded-band stretching
    HydraGated,          // Hydra-gated stretching (interaction-filtered)
    CompBandDecoupled,   // Composite-band decoupled stretching
    E8ProbeWeighted,     // E8-probe-weighted stretching monitor
}

/// Slot B: Band/coupling schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandSchema {
    IndependentPrime,     // Independent prime lanes
    FusedComposite,       // Fused composite bands
    TriadBanding,         // Triad banding
    AdaptiveRepartition,  // Adaptive band repartition
    DelayGraphCoupled,    // Delay-graph-coupled bands
}

/// Slot K: K-Elimination / overflow control schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KSchema {
    PassiveOverflowDetect,  // Pure overflow detection
    WindingStressMonitor,   // Winding stress monitor
    SoftRescale,            // Soft rescale before saturation
    AnchorWindingBarrier,   // Anchor-driven winding barrier
    BandLocalCertificate,   // Band-local overflow certificates
}

/// Slot T: Time integrator schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSchema {
    ExplicitEuler,
    SSPRK2,
    SSPRK3,
    SplitSemiImplicit,
    AdaptiveExactInteger,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 3: MEASUREMENT SCHEMA (Slot M)
// ═══════════════════════════════════════════════════════════════════════════════

/// Slot M: Which measurements to track.
#[derive(Debug, Clone)]
pub struct MeasurementSchema {
    pub track_rn: bool,          // R(N) = max|Ω|/capacity
    pub track_bkm_sum: bool,     // BKM discrete sum
    pub track_c_eff: bool,       // Effective C_h estimate
    pub track_winding: bool,     // Winding stress
    pub track_divergence: bool,  // Divergence defect
    pub track_lock_score: bool,  // Cross-band lock score
    pub track_occupancy: bool,   // Lane occupancy histogram
}

impl MeasurementSchema {
    pub fn full() -> Self {
        Self {
            track_rn: true, track_bkm_sum: true, track_c_eff: true,
            track_winding: true, track_divergence: true,
            track_lock_score: true, track_occupancy: true,
        }
    }
}

/// Runtime measurements collected at each timestep. ALL INTEGER.
/// Scale convention: values marked _scaled are × ORACLE_SCALE (10,000).
#[derive(Debug, Clone, Default)]
pub struct StepMeasurements {
    pub m_t: i64,                  // M(t) = max|Ω| (raw integer)
    pub rn_scaled: i64,            // R(N) × ORACLE_SCALE = |ω| * SCALE / capacity
    pub bkm_increment_scaled: i64, // ||ω||_∞ · dt × ORACLE_SCALE
    pub c_eff_scaled: i64,         // Ĉ_h × ORACLE_SCALE² (negative = decaying)
    pub winding_stress: i64,       // max |κ| across all bridges
    pub div_defect: i64,           // max |∇·u|
    pub lock_score_scaled: i64,    // cross-band lock metric × ORACLE_SCALE
}

/// Accumulated measurements over a full run. All integer.
#[derive(Debug, Clone)]
pub struct RunMeasurements {
    pub schema_name: String,
    pub grid_n: usize,
    pub steps: Vec<StepMeasurements>,
    // Suprema (all scaled integers)
    pub sup_rn_scaled: i64,
    pub bkm_sum_scaled: i64,
    pub sup_c_eff_scaled: i64,
    pub sup_winding: i64,
    pub sup_div_defect: i64,
    pub sup_lock_score_scaled: i64,
}

impl RunMeasurements {
    pub fn new(name: &str, n: usize) -> Self {
        Self {
            schema_name: name.to_string(), grid_n: n,
            steps: Vec::new(),
            sup_rn_scaled: 0, bkm_sum_scaled: 0, sup_c_eff_scaled: 0,
            sup_winding: 0, sup_div_defect: 0, sup_lock_score_scaled: 0,
        }
    }

    pub fn record(&mut self, s: StepMeasurements) {
        if s.rn_scaled > self.sup_rn_scaled { self.sup_rn_scaled = s.rn_scaled; }
        self.bkm_sum_scaled += s.bkm_increment_scaled;
        if s.c_eff_scaled > self.sup_c_eff_scaled { self.sup_c_eff_scaled = s.c_eff_scaled; }
        if s.winding_stress > self.sup_winding { self.sup_winding = s.winding_stress; }
        if s.div_defect > self.sup_div_defect { self.sup_div_defect = s.div_defect; }
        if s.lock_score_scaled > self.sup_lock_score_scaled { self.sup_lock_score_scaled = s.lock_score_scaled; }
        self.steps.push(s);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 4: THE COMPLETE SCHEMA TUPLE Σ
// ═══════════════════════════════════════════════════════════════════════════════

/// The full 9-slot schema: Σ = (L, P, D, S, B, K, C, M, T)
#[derive(Debug, Clone)]
pub struct Schema {
    pub name: String,
    pub lane: LaneSchema,
    pub projection: ProjectionSchema,
    pub derivative: DerivativeSchema,
    pub stretching: StretchingSchema,
    pub band: BandSchema,
    pub k_schema: KSchema,
    pub chain: Vec<Op>,
    pub measurement: MeasurementSchema,
    pub time: TimeSchema,
}

impl Schema {
    /// Check homogeneous admissibility: Σ ∈ H
    /// All componentwise arithmetic operators must be lane-template identical.
    pub fn is_homogeneous(&self) -> bool {
        // Check: all layers use the same reconstruction law
        // Check: all layers have same derivative/projection coefficients
        // Check: same anchor semantics across components
        // For now: lane schema type must be H-prefixed
        matches!(self.lane.lane_type,
            LaneType::HPrime | LaneType::HCompBand |
            LaneType::HTriad | LaneType::HAnchorPull)
    }

    pub fn chain_string(&self) -> String {
        format_chain(&self.chain)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 5: THE FIVE FAMILIES
// ═══════════════════════════════════════════════════════════════════════════════

fn triad_lane_schema() -> LaneSchema {
    LaneSchema {
        lane_type: LaneType::HTriad,
        layers: vec![vec![2, 3, 5], vec![7, 11, 13], vec![17, 19, 23]],
        anchors: vec![29, 31],
        band_mode: BandMode::PrimeOnly,
    }
}

fn compband_lane_schema() -> LaneSchema {
    LaneSchema {
        lane_type: LaneType::HCompBand,
        layers: vec![vec![2, 3, 5], vec![7, 11, 13], vec![17, 19, 23]],
        anchors: vec![29, 31],
        band_mode: BandMode::CompositeOptional,
    }
}

fn anchor_pull_lane_schema() -> LaneSchema {
    LaneSchema {
        lane_type: LaneType::HAnchorPull,
        layers: vec![vec![2, 3, 5], vec![7, 11, 13], vec![17, 19, 23]],
        anchors: vec![29, 31, 37, 41],
        band_mode: BandMode::CompositeOptional,
    }
}

/// H0: Theorem baseline — clean reference model.
pub fn h0_baseline_triad() -> Schema {
    Schema {
        name: "H0_baseline_triad".into(),
        lane: triad_lane_schema(),
        projection: ProjectionSchema::SpectralBiotSavart,
        derivative: DerivativeSchema::Central2,
        stretching: StretchingSchema::Direct,
        band: BandSchema::IndependentPrime,
        k_schema: KSchema::PassiveOverflowDetect,
        chain: parse_chain("PTDSPKM"),
        measurement: MeasurementSchema::full(),
        time: TimeSchema::ExplicitEuler,
    }
}

/// H1: Band-separable — tests composite coprimality advantage.
pub fn h1_compband_triad() -> Schema {
    Schema {
        name: "H1_compband_triad".into(),
        lane: compband_lane_schema(),
        projection: ProjectionSchema::SpectralBiotSavart,
        derivative: DerivativeSchema::Central2,
        stretching: StretchingSchema::CompBandDecoupled,
        band: BandSchema::FusedComposite,
        k_schema: KSchema::PassiveOverflowDetect,
        chain: parse_chain("PTSDPKM"),
        measurement: MeasurementSchema::full(),
        time: TimeSchema::ExplicitEuler,
    }
}

/// H2: Anchor-pullback — reorder the torus walk.
pub fn h2_anchor_pullback_triad() -> Schema {
    Schema {
        name: "H2_anchor_pullback_triad".into(),
        lane: anchor_pull_lane_schema(),
        projection: ProjectionSchema::SpectralBiotSavart,
        derivative: DerivativeSchema::Central2,
        stretching: StretchingSchema::CompBandDecoupled,
        band: BandSchema::FusedComposite,
        k_schema: KSchema::AnchorWindingBarrier,
        chain: parse_chain("APTRSPDKM"),
        measurement: MeasurementSchema::full(),
        time: TimeSchema::ExplicitEuler,
    }
}

/// H3: Hydra-gated — selective suppression of dangerous interactions.
pub fn h3_hydra_gated_triad() -> Schema {
    Schema {
        name: "H3_hydra_gated_triad".into(),
        lane: triad_lane_schema(),
        projection: ProjectionSchema::SpectralBiotSavart,
        derivative: DerivativeSchema::Central2,
        stretching: StretchingSchema::HydraGated,
        band: BandSchema::DelayGraphCoupled,
        k_schema: KSchema::WindingStressMonitor,
        chain: parse_chain("PTHSPDKM"),
        measurement: MeasurementSchema::full(),
        time: TimeSchema::ExplicitEuler,
    }
}

/// H4: Geometric probe — exact diagnostics, no theorem overclaim.
pub fn h4_probe_triad() -> Schema {
    Schema {
        name: "H4_probe_triad".into(),
        lane: triad_lane_schema(),
        projection: ProjectionSchema::SpectralBiotSavart,
        derivative: DerivativeSchema::Central2,
        stretching: StretchingSchema::E8ProbeWeighted,
        band: BandSchema::IndependentPrime,
        k_schema: KSchema::WindingStressMonitor,
        chain: parse_chain("PTSDEKM"),
        measurement: MeasurementSchema::full(),
        time: TimeSchema::ExplicitEuler,
    }
}

/// All five families.
pub fn all_families() -> Vec<Schema> {
    vec![
        h0_baseline_triad(),
        h1_compband_triad(),
        h2_anchor_pullback_triad(),
        h3_hydra_gated_triad(),
        h4_probe_triad(),
    ]
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 6: OBJECTIVE FUNCTION J(Σ)
// ═══════════════════════════════════════════════════════════════════════════════

/// Weights for the objective function. All integer (relative).
#[derive(Debug, Clone)]
pub struct ObjectiveWeights {
    pub lambda_rn: i64,
    pub lambda_bkm: i64,
    pub lambda_c_eff: i64,
    pub lambda_div: i64,
    pub lambda_winding: i64,
}

impl ObjectiveWeights {
    pub fn default_equal() -> Self {
        Self { lambda_rn: 1, lambda_bkm: 1, lambda_c_eff: 1, lambda_div: 1, lambda_winding: 1 }
    }

    pub fn theorem_facing() -> Self {
        Self { lambda_rn: 1, lambda_bkm: 3, lambda_c_eff: 5, lambda_div: 2, lambda_winding: 1 }
    }
}

/// Evaluate J(Σ) from accumulated measurements. All integer.
/// Returns J × ORACLE_SCALE (scaled integer).
pub fn evaluate_objective(
    runs: &[RunMeasurements],
    weights: &ObjectiveWeights,
    capacity_scaled: i64,
) -> i64 {
    let sup_rn = runs.iter().map(|r| r.sup_rn_scaled).max().unwrap_or(0);
    let sup_bkm = runs.iter().map(|r| r.bkm_sum_scaled).max().unwrap_or(0);
    let sup_c_eff = runs.iter().map(|r| r.sup_c_eff_scaled).max().unwrap_or(0);
    let sup_div = runs.iter().map(|r| r.sup_div_defect).max().unwrap_or(0);
    let sup_winding = runs.iter().map(|r| r.sup_winding).max().unwrap_or(0);

    weights.lambda_rn * sup_rn
    + weights.lambda_bkm * sup_bkm
    + weights.lambda_c_eff * sup_c_eff
    + weights.lambda_div * sup_div
    + weights.lambda_winding * sup_winding
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 7: SCHEMA RUNNER — Execute a chain on the solver
// ═══════════════════════════════════════════════════════════════════════════════

use crate::ns_grid_3d::*;
use crate::biot_savart::*;
use crate::dual_tray::*;

/// Execute one schema on a grid for n_steps, collecting all measurements.
pub fn execute_schema(
    schema: &Schema,
    grid_n: usize,
    n_steps: u64,
) -> RunMeasurements {
    let mut measurements = RunMeasurements::new(&schema.name, grid_n);

    // Set up grid + kernel
    let dt_scaled = 100i64;
    let mut grid = NSGrid3D::new(grid_n, 8, dt_scaled);
    grid.init_taylor_green(TG_BASE_AMP, 1, 1);
    let kernel = BiotSavartKernel::precompute(grid_n);

    // Dual-tray for cross-bridge monitoring
    let sliding_config = match schema.stretching {
        StretchingSchema::HydraGated => champion_s8_config(),
        StretchingSchema::E8ProbeWeighted => regularity_probe_config(),
        _ => diffusion_reference_config(),
    };
    let mut dual = DualTrayCRT::with_sliding_config(sliding_config);

    // CRT capacity (product of first 8 primary primes ≈ 2^504)
    // For measurement purposes, use the single-lane capacity
    let capacity_lane = PRIMARY_PRIMES[0] as i128;
    let dt_scaled = 100i64;

    let mut prev_m: i64 = grid.vort_sup_global();

    for step in 0..n_steps {
        // Execute the chain word
        for &op in &schema.chain {
            match op {
                Op::P => {
                    // Projection: Biot-Savart velocity recovery
                    biot_savart_update(&kernel, &mut grid);
                }
                Op::T | Op::D | Op::S => {
                    // Transport + Diffusion + Stretching all happen in advance()
                    // The schema distinction is in HOW they're combined,
                    // but the underlying exact-integer stencil is the same.
                    // We only call advance() once per step (after all T/D/S in chain).
                }
                Op::K => {
                    // K-Elimination monitoring
                    let m_t = grid.vort_sup_global();
                    dual.encode(m_t);
                    dual.merge_sliding_to_primary();
                }
                Op::H => {
                    // Hydra gating: check interaction pattern
                    // The sliding tray with champion config acts as the filter
                    let m_t = grid.vort_sup_global();
                    dual.encode(m_t);
                    let scan = dual.cross_bridge_scan();
                    // If bridges are diverging, the Hydra is flagging danger
                    if !scan.all_bounded {
                        // In a full implementation, this would gate the stretching
                        // For now: record the divergence
                    }
                }
                Op::A => {
                    // Anchor pullback: noop in current implementation
                    // Would reparameterize the torus walk
                }
                Op::R => {
                    // Reconstruction / rebalance: noop for now
                }
                Op::E => {
                    // E8 geometric probe: noop for now
                }
                Op::M => {
                    // Measurement: handled below after all ops
                }
            }
        }

        // Advance vorticity (combines T+D+S in the exact-integer stencil)
        grid.advance();

        // --- Collect measurements (ALL INTEGER) ---
        let m_t = grid.vort_sup_global();

        // R(N) × ORACLE_SCALE = m_t * ORACLE_SCALE / capacity
        // capacity is lane prime ~ 4.6e18, so m_t * 10000 won't overflow i64 for m_t < 9.2e14
        let rn_scaled = ((m_t as i128 * 10_000) / capacity_lane as i128) as i64;

        // BKM increment × ORACLE_SCALE: ||ω||_∞ * dt / SCALE, all scaled
        // = m_t * dt_scaled / (SCALE * SCALE) * ORACLE_SCALE
        // = m_t * dt_scaled * ORACLE_SCALE / SCALE²
        let bkm_inc_scaled = ((m_t as i128 * dt_scaled as i128 * 10_000)
                             / (SCALE as i128 * SCALE as i128)) as i64;

        // Ĉ_h × ORACLE_SCALE²: (M(t+1) - M(t)) * SCALE² / (dt * M(t)²)
        // But we're computing AFTER advance, so m_t IS M(t+1). prev_m is M(t).
        let c_eff_scaled = if prev_m > 0 {
            let delta_m = m_t - prev_m;
            // c_eff = delta_m / (dt_phys * prev_m²)
            //       = delta_m * SCALE / (dt_scaled * prev_m²) * ORACLE_SCALE²
            // To keep in i128: delta_m * SCALE * ORACLE_SCALE² / (dt_scaled * prev_m²)
            let num = delta_m as i128 * SCALE as i128 * 10_000 * 10_000;
            let den = dt_scaled as i128 * prev_m as i128 * prev_m as i128;
            if den != 0 { (num / den) as i64 } else { 0 }
        } else { 0 };

        // Winding stress
        dual.encode(m_t);
        let scan = dual.cross_bridge_scan();
        let winding_stress = scan.max_kappa;

        // Divergence defect
        let u_vecs = kernel.apply(&grid);
        let n = grid_n;
        let n_i = n as i64;
        let mut max_div: i64 = 0;
        for ix in 0..n {
            for iy in 0..n {
                for iz in 0..n {
                    let ixp = (ix+1)%n; let ixm = (ix+n-1)%n;
                    let iyp = (iy+1)%n; let iym = (iy+n-1)%n;
                    let izp = (iz+1)%n; let izm = (iz+n-1)%n;
                    let d = n_i*(u_vecs[ixp*n*n+iy*n+iz][0]-u_vecs[ixm*n*n+iy*n+iz][0])/2
                          + n_i*(u_vecs[ix*n*n+iyp*n+iz][1]-u_vecs[ix*n*n+iym*n+iz][1])/2
                          + n_i*(u_vecs[ix*n*n+iy*n+izp][2]-u_vecs[ix*n*n+iy*n+izm][2])/2;
                    max_div = max_div.max(d.abs());
                }
            }
        }

        let step_m = StepMeasurements {
            m_t,
            rn_scaled,
            bkm_increment_scaled: bkm_inc_scaled,
            c_eff_scaled,
            winding_stress,
            div_defect: max_div,
            lock_score_scaled: if scan.all_bounded { 0 } else { 10_000 },
        };

        measurements.record(step_m);
        prev_m = m_t;
    }

    measurements
}

/// Run all five families at a given N, rank by J(Σ).
pub fn run_family_sweep(grid_n: usize, n_steps: u64) -> Vec<(Schema, RunMeasurements, i64)> {
    let families = all_families();
    let weights = ObjectiveWeights::theorem_facing();
    let capacity_lane = PRIMARY_PRIMES[0] as i64;

    let mut results: Vec<(Schema, RunMeasurements, i64)> = Vec::new();

    for schema in &families {
        if !schema.is_homogeneous() { continue; }
        let meas = execute_schema(schema, grid_n, n_steps);
        let j_val = evaluate_objective(&[meas.clone()], &weights, capacity_lane);
        results.push((schema.clone(), meas, j_val));
    }

    // Sort by J(Σ) ascending (lower = better)
    results.sort_by_key(|r| r.2);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chain() {
        let chain = parse_chain("PTDSPKM");
        assert_eq!(chain, vec![Op::P, Op::T, Op::D, Op::S, Op::P, Op::K, Op::M]);
    }

    #[test]
    fn test_format_chain() {
        let chain = vec![Op::P, Op::T, Op::H, Op::S, Op::P, Op::D, Op::K, Op::M];
        assert_eq!(format_chain(&chain), "PTHSPDKM");
    }

    #[test]
    fn test_all_families_homogeneous() {
        for schema in all_families() {
            assert!(schema.is_homogeneous(),
                "Schema {} must be homogeneous", schema.name);
        }
    }

    #[test]
    fn test_h0_baseline() {
        let h0 = h0_baseline_triad();
        assert_eq!(h0.chain_string(), "PTDSPKM");
        assert_eq!(h0.stretching, StretchingSchema::Direct);
    }

    #[test]
    fn test_execute_h0() {
        let h0 = h0_baseline_triad();
        let meas = execute_schema(&h0, 8, 5);
        assert_eq!(meas.steps.len(), 5);
        assert!(meas.sup_rn_scaled >= 0, "R(N) should be non-negative");
        assert!(meas.bkm_sum_scaled > 0, "BKM sum should be positive");
    }

    #[test]
    fn test_objective_ranking() {
        let results = run_family_sweep(8, 5);
        assert_eq!(results.len(), 5);
        for w in results.windows(2) {
            assert!(w[0].2 <= w[1].2 + 1,
                "Results not sorted: {} > {}", w[0].2, w[1].2);
        }
        println!("\nFamily ranking (J ascending = better):");
        for (schema, meas, j) in &results {
            println!("  J={:>10} | {:30} | chain={} | R={} | BKM={} | Ĉ_h={} | W={} | D={}",
                j, schema.name, schema.chain_string(),
                meas.sup_rn_scaled, meas.bkm_sum_scaled, meas.sup_c_eff_scaled,
                meas.sup_winding, meas.sup_div_defect);
        }
    }
}
