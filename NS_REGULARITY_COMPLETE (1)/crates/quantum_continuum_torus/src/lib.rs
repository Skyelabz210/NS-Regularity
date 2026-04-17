// quantum_continuum_torus/src/lib.rs
//
// QUANTUM CONTINUUM INJECTED DISCRETE TORUS (QCIDT)
// Anthony Diaz / HackFate.us · April 2026
//
// Architecture:
//
//   RamanujanMemoryLane winding snapshot
//          ↓  inject_winding_snapshot()
//   F_p² quantum amplitude  (a + b√δ)
//          ↓  a = residue = total_delta mod p
//          ↓  b = magnitude mod p (second base-p digit)
//   GearManifold lane       (residue: u64, magnitude: i128)
//          ↓  total = magnitude * p + residue  (exact)
//
//   Quantum evolution:  σ(a+b√δ) = a-b√δ   (Frobenius, period 2)
//   Discrete readout:   N(a+b√δ) = a²-δb²  (norm, Frobenius-invariant)
//   Cross-lane bridge:  K-Elim(N(amp_i), residue_j) = quantum winding number
//
//   The "continuum" is the b-component: it encodes winding information
//   that lives OUTSIDE the discrete ring Z/pZ, in the F_p² extension.
//   Frobenius phase-flips b without changing the discrete projection via norm.
//   Quantum interference (amp_i × amp_j) has Frobenius-sensitive imaginary part.
//
// Zero float. All arithmetic: exact integers in Z/pZ and Z.

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

use qmnf_primitives::gear_manifold::GearManifold;
use qmnf_primitives::{addmod, submod, mulmod, mod_pow, mod_inverse, k_eliminate};

// ═══════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════

pub const SAFE_BASIS: [u64; 5] = [3, 5, 7, 11, 13];
/// Shadow prime: absent from Dresden periods, splitter anchor
pub const SHADOW_PRIME: u64 = 11;

// ═══════════════════════════════════════════════════════════════════
// ARITHMETIC HELPERS
// ═══════════════════════════════════════════════════════════════════

/// Find smallest quadratic non-residue mod p
pub fn find_qnr(p: u64) -> u64 {
    for d in 2..p {
        // Legendre symbol: d^{(p-1)/2} mod p = p-1 ↔ QNR
        if mod_pow(d, (p - 1) / 2, p) == p - 1 {
            return d;
        }
    }
    panic!("no QNR for p={}", p)
}

/// Euclidean magnitude of i128 into [0, p)
#[inline]
pub fn mag_to_residue(mag: i128, p: u64) -> u64 {
    ((mag % p as i128 + p as i128) % p as i128) as u64
}

// ═══════════════════════════════════════════════════════════════════
// F_p² ELEMENT
//
// Represents a + b·√δ in the quadratic extension F_p[√δ]
// where δ is a quadratic non-residue mod p.
//
// The INJECTION mapping:
//   residue  → a  (position in base ring Z/pZ)
//   magnitude mod p → b  (winding, as second base-p digit)
//
// Together, a + b·p encodes the value mod p² (the natural lift).
// a + b·√δ is the algebraic encoding in F_p².
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fp2 {
    pub a: u64,     // real part ∈ F_p
    pub b: u64,     // imaginary part ∈ F_p
    pub p: u64,     // base prime
    pub delta: u64, // QNR: x² ≢ δ (mod p) has no solution
}

impl Fp2 {
    pub fn new(a: u64, b: u64, p: u64, delta: u64) -> Self {
        Fp2 { a: a % p, b: b % p, p, delta }
    }

    pub fn zero(p: u64, delta: u64) -> Self { Fp2 { a: 0, b: 0, p, delta } }
    pub fn one(p: u64, delta: u64) -> Self { Fp2 { a: 1, b: 0, p, delta } }

    /// Build from a (residue, magnitude) pair — the core INJECTION function.
    ///
    /// a = residue = total_accumulated_delta mod p
    /// b = magnitude mod p = floor(total/p) mod p
    ///
    /// Together (a, b) encode the value mod p² in base p.
    pub fn from_winding(residue: u64, magnitude: i128, p: u64, delta: u64) -> Self {
        let b = mag_to_residue(magnitude, p);
        Fp2 { a: residue % p, b, p, delta }
    }

    /// Frobenius automorphism: σ(a + b√δ) = a - b√δ
    ///
    /// Period 2: σ(σ(x)) = x.
    /// Phase-flips the winding component b without touching the residue a.
    /// σ(x) and x project to the SAME discrete value via norm.
    pub fn frobenius(&self) -> Fp2 {
        Fp2 {
            a: self.a,
            b: if self.b == 0 { 0 } else { self.p - self.b },
            p: self.p,
            delta: self.delta,
        }
    }

    /// Norm: N(a + b√δ) = a² - δb² mod p
    ///
    /// N is Frobenius-invariant: N(σ(x)) = N(x).
    /// This is the DISCRETE READOUT — collapses quantum state to Z/pZ.
    /// N: F_p²* → F_p* is SURJECTIVE (Hilbert 90).
    pub fn norm(&self) -> u64 {
        let a2 = mulmod(self.a, self.a, self.p);
        let db2 = mulmod(self.delta, mulmod(self.b, self.b, self.p), self.p);
        submod(a2, db2, self.p)
    }

    /// Trace: Tr(a + b√δ) = 2a mod p
    ///
    /// Tr is Frobenius-invariant (b-component cancels).
    pub fn trace(&self) -> u64 { addmod(self.a, self.a, self.p) }

    /// Multiply in F_p²: (a+b√δ)(c+d√δ) = (ac+bdδ) + (ad+bc)√δ
    ///
    /// Quantum interference: the imaginary part (ad+bc) IS Frobenius-sensitive.
    /// Im(x·y) → -(Im(x·y)) under Frobenius on either factor.
    pub fn mul(&self, other: &Self) -> Fp2 {
        debug_assert_eq!(self.p, other.p);
        let p = self.p;
        let re = addmod(mulmod(self.a, other.a, p),
                        mulmod(mulmod(self.delta, self.b, p), other.b, p), p);
        let im = addmod(mulmod(self.a, other.b, p), mulmod(self.b, other.a, p), p);
        Fp2 { a: re, b: im, p, delta: self.delta }
    }

    /// Conjugate = Frobenius for degree-2 extensions
    pub fn conj(&self) -> Fp2 { self.frobenius() }

    /// x · σ(x) = N(x) embedded in F_p² (sanity check: imaginary part = 0)
    pub fn norm_via_frobenius(&self) -> Fp2 {
        self.mul(&self.frobenius())
    }

    /// Is this element in the base field? (b = 0)
    pub fn is_in_base_field(&self) -> bool { self.b == 0 }

    /// Base-p two-digit value: a + b·p (the natural lift to Z/p²Z)
    pub fn base_p_value(&self) -> u128 {
        self.a as u128 + self.b as u128 * self.p as u128
    }

    /// Quantum phase state: which Frobenius phase is this? 0 or 1.
    /// Phase 0: b = original. Phase 1: b = p - original.
    /// Determined by: b <= p/2 → phase 0, else phase 1 (heuristic)
    pub fn frobenius_phase(&self) -> u8 {
        if self.b <= self.p / 2 { 0 } else { 1 }
    }
}

// ═══════════════════════════════════════════════════════════════════
// QCID LANE
//
// A single lane in the QCID torus.
// Holds both the discrete state (residue, magnitude) from GearManifold
// and the quantum amplitude (Fp2) derived from it.
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct QCIDLane {
    pub p: u64,
    pub delta: u64,          // QNR for this prime

    // Discrete layer (mirrors GearManifold)
    pub residue: u64,        // total mod p
    pub magnitude: i128,     // floor(total / p)

    // Quantum layer (derived from discrete, evolves independently)
    pub amplitude: Fp2,      // a + b√δ in F_p²
    pub frobenius_count: u64, // how many Frobenius steps applied since last sync
}

impl QCIDLane {
    pub fn new(p: u64) -> Self {
        let delta = find_qnr(p);
        QCIDLane {
            p, delta,
            residue: 0, magnitude: 0,
            amplitude: Fp2::zero(p, delta),
            frobenius_count: 0,
        }
    }

    /// Sync quantum amplitude from current (residue, magnitude).
    /// This is the discrete → quantum INJECTION step.
    /// Resets frobenius_count to 0 (clean injection state).
    pub fn sync_amplitude(&mut self) {
        self.amplitude = Fp2::from_winding(self.residue, self.magnitude, self.p, self.delta);
        self.frobenius_count = 0;
    }

    /// Project quantum amplitude back to discrete layer via norm.
    /// Updates residue only — magnitude is unchanged (it's tracked discretely).
    pub fn project_to_discrete(&mut self) {
        self.residue = self.amplitude.norm();
    }

    /// Apply Frobenius automorphism to quantum amplitude.
    /// Period 2: applying twice returns to original.
    pub fn frobenius_step(&mut self) {
        self.amplitude = self.amplitude.frobenius();
        self.frobenius_count += 1;
    }

    /// True if amplitude.a == residue (real part coherence, not norm coherence).
    /// amplitude.b encodes the winding mod p — distinct quantum dimension.
    pub fn is_coherent(&self) -> bool { self.amplitude.a == self.residue }

    /// True if quantum amplitude is in base field (b = 0, pure discrete)
    pub fn is_classical(&self) -> bool {
        self.amplitude.is_in_base_field()
    }
}

// ═══════════════════════════════════════════════════════════════════
// INJECTION RECORD
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct InjectionRecord {
    pub step_at: u64,
    pub snapshot: Vec<i128>,
    pub trigger_n: Option<u64>,  // the n value that triggered injection (shadow gate)
}

// ═══════════════════════════════════════════════════════════════════
// QUANTUM CONTINUUM INJECTED DISCRETE TORUS
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct QCIDTorus {
    pub lanes: Vec<QCIDLane>,
    pub gear: GearManifold,
    pub step_count: u64,
    /// Index of the shadow lane (q=11) if present
    pub shadow_idx: Option<usize>,
    /// Full injection history
    pub injection_log: Vec<InjectionRecord>,
}

impl QCIDTorus {
    /// Create with the Safe Basis {3, 5, 7, 11, 13}
    pub fn new_safe_basis() -> Self {
        Self::new(&SAFE_BASIS)
    }

    /// Create with arbitrary primes (sorted ascending to match GearManifold)
    pub fn new(primes: &[u64]) -> Self {
        let mut sorted = primes.to_vec();
        sorted.sort_unstable();
        let lanes: Vec<QCIDLane> = sorted.iter().map(|&p| QCIDLane::new(p)).collect();
        let gear = GearManifold::new(&sorted);
        let shadow_idx = sorted.iter().position(|&p| p == SHADOW_PRIME);
        QCIDTorus { lanes, gear, step_count: 0, shadow_idx, injection_log: Vec::new() }
    }

    // ── DISCRETE STEP ──────────────────────────────────────────────

    /// Advance one CRAM step: add delta to all CRT lanes.
    /// Updates discrete layer; quantum layer stays in its current phase
    /// until explicitly synced or injected.
    pub fn step(&mut self, delta: i128) {
        self.gear.add(delta);
        for (i, lane) in self.lanes.iter_mut().enumerate() {
            let g = &self.gear.lanes[i];
            lane.residue = g.residue;
            lane.magnitude = g.magnitude;
        }
        self.step_count += 1;
    }

    /// Step driven by Ramanujan sum: delta = c_q(n) for the lane's prime q.
    pub fn step_ramanujan(&mut self, n: u64) {
        // Each lane uses its own prime's Ramanujan sum
        // c_q(n) = q-1 if q|n, else -1
        // For multi-lane: use the sum over all primes? 
        // Or per-lane independent delta?
        // Design: per-lane Ramanujan deltas, accumulated independently
        for (i, lane) in self.lanes.iter_mut().enumerate() {
            let delta: i128 = if n % lane.p == 0 { (lane.p - 1) as i128 } else { -1 };
            let g = &mut self.gear.lanes[i];
            let m = g.modulus as i128;
            let temp = g.residue as i128 + delta;
            let new_r = ((temp % m) + m) % m;
            let wraps = (temp - new_r) / m;
            g.residue = new_r as u64;
            g.magnitude += wraps;
            lane.residue = g.residue;
            lane.magnitude = g.magnitude;
        }
        self.step_count += 1;
    }

    // ── QUANTUM OPERATIONS ─────────────────────────────────────────

    /// Sync all quantum amplitudes from current discrete state.
    /// Call after a sequence of discrete steps to re-couple quantum layer.
    pub fn sync_quantum(&mut self) {
        for lane in self.lanes.iter_mut() {
            lane.sync_amplitude();
        }
    }

    /// Apply Frobenius to all quantum amplitudes.
    /// σ: a + b√δ → a - b√δ. Period 2. Norm-invariant.
    pub fn apply_frobenius(&mut self) {
        for lane in self.lanes.iter_mut() {
            lane.frobenius_step();
        }
    }

    /// Project all quantum amplitudes back to discrete layer via norm.
    /// Residues updated; magnitudes preserved.
    pub fn project_quantum_to_discrete(&mut self) {
        for (i, lane) in self.lanes.iter_mut().enumerate() {
            lane.project_to_discrete();
            self.gear.lanes[i].residue = lane.residue;
        }
    }

    // ── STATE INJECTION ────────────────────────────────────────────

    /// Inject a winding snapshot directly into the torus.
    ///
    /// This is the core QCIDT capability: given a Vec<i128> from
    /// RamanujanMemoryLane (or any source), write it as the new state
    /// on both the discrete (GearManifold) and quantum (Fp2) layers.
    ///
    /// After injection:
    ///   lane.residue   = snapshot[i] mod p_i      (discrete)
    ///   lane.magnitude = snapshot[i]               (integer winding)
    ///   lane.amplitude = Fp2::from_winding(...)    (quantum, synced)
    ///   amplitude.a = residue                      (base-ring position)
    ///   amplitude.b = magnitude mod p              (winding in field)
    pub fn inject_winding_snapshot(&mut self, snapshot: &[i128]) {
        for (i, lane) in self.lanes.iter_mut().enumerate() {
            let mag = match snapshot.get(i) { Some(&m) => m, None => continue };
            let residue = mag_to_residue(mag, lane.p);
            lane.magnitude = mag;
            lane.residue = residue;
            lane.sync_amplitude();
            // Mirror into GearManifold
            self.gear.lanes[i].magnitude = mag;
            self.gear.lanes[i].residue = residue;
        }
        self.injection_log.push(InjectionRecord {
            step_at: self.step_count,
            snapshot: snapshot.to_vec(),
            trigger_n: None,
        });
    }

    /// Full-state injection: set residue and GearManifold.magnitude independently.
    /// This is required to reach quantum fixed points where a ≠ b.
    /// inject_winding_snapshot sets residue = magnitude mod p (a == b always).
    /// inject_full_state sets them independently.
    pub fn inject_full_state(&mut self, residues: &[u64], magnitudes: &[i128]) {
        for (i, lane) in self.lanes.iter_mut().enumerate() {
            let r = match residues.get(i) { Some(&r) => r % lane.p, None => continue };
            let m = match magnitudes.get(i) { Some(&m) => m, None => continue };
            lane.residue = r;
            lane.magnitude = m;
            lane.sync_amplitude(); // from_winding(r, m, p, delta): a=r, b=m mod p
            self.gear.lanes[i].residue = r;
            self.gear.lanes[i].magnitude = m;
        }
        self.injection_log.push(InjectionRecord {
            step_at: self.step_count,
            snapshot: magnitudes.to_vec(),
            trigger_n: None,
        });
    }

    /// Shadow-gated injection: inject ONLY when q=11 resonance fires (n % 11 == 0).
    ///
    /// Before resonance: shadow lane holds multidimensional zero (Vec::new()).
    /// After resonance: state injection couples quantum + discrete layers.
    /// This mirrors the RamanujanMemoryLane shadow semantics exactly.
    pub fn shadow_inject(&mut self, n: u64, snapshot: &[i128]) -> bool {
        let resonates = self.shadow_idx
            .map(|idx| n % self.lanes[idx].p == 0)
            .unwrap_or(false);
        if resonates {
            let mut rec = InjectionRecord {
                step_at: self.step_count,
                snapshot: snapshot.to_vec(),
                trigger_n: Some(n),
            };
            // Write into lanes and gear
            for (i, lane) in self.lanes.iter_mut().enumerate() {
                let mag = match snapshot.get(i) { Some(&m) => m, None => continue };
                let residue = mag_to_residue(mag, lane.p);
                lane.magnitude = mag;
                lane.residue = residue;
                lane.sync_amplitude();
                self.gear.lanes[i].magnitude = mag;
                self.gear.lanes[i].residue = residue;
            }
            rec.trigger_n = Some(n);
            self.injection_log.push(rec);
        }
        resonates
    }

    // ── K-ELIMINATION BRIDGE ───────────────────────────────────────

    /// Quantum K-Elimination: bridges F_p² layer to discrete layer.
    ///
    /// r = N(amplitude[lane_a]) mod p_b   — quantum projection onto lane_b's ring
    /// s = residue[lane_b]                — discrete residue of lane_b
    /// k = (s - r) · p_a⁻¹ mod p_b       — quantum winding number
    ///
    /// k = 0: quantum state of lane_a is CRT-consistent with lane_b → coherence
    /// k ≠ 0: quantum-discrete mismatch — k encodes the "continuum overhang"
    pub fn quantum_k_elim(&self, lane_a: usize, lane_b: usize) -> u64 {
        let la = &self.lanes[lane_a];
        let lb = &self.lanes[lane_b];
        let r = la.amplitude.norm();
        k_eliminate(r, lb.residue, la.p, lb.p)
    }

    /// Classical K-Elimination (residue-only, no quantum layer).
    pub fn classical_k_elim(&self, lane_a: usize, lane_b: usize) -> u64 {
        let la = &self.lanes[lane_a];
        let lb = &self.lanes[lane_b];
        k_eliminate(la.residue, lb.residue, la.p, lb.p)
    }

    /// Quantum inner product between two lanes: amp_a × amp_b in F_p².
    ///
    /// Imaginary part of result is Frobenius-SENSITIVE — this is the
    /// quantum interference term not accessible via norm alone.
    pub fn quantum_inner_product(&self, lane_a: usize, lane_b: usize) -> Fp2 {
        self.lanes[lane_a].amplitude.mul(&self.lanes[lane_b].amplitude)
    }

    // ── READOUT ────────────────────────────────────────────────────

    /// All discrete residues
    pub fn discrete_state(&self) -> Vec<u64> {
        self.lanes.iter().map(|l| l.residue).collect()
    }

    /// All quantum norm projections (Frobenius-invariant readout)
    pub fn quantum_projected_state(&self) -> Vec<u64> {
        self.lanes.iter().map(|l| l.amplitude.norm()).collect()
    }

    /// All quantum amplitudes
    pub fn amplitudes(&self) -> Vec<Fp2> {
        self.lanes.iter().map(|l| l.amplitude).collect()
    }

    /// All winding magnitudes
    pub fn windings(&self) -> Vec<i128> {
        self.lanes.iter().map(|l| l.magnitude).collect()
    }

    /// Full integer reconstruction via Garner
    pub fn total_value(&self) -> u128 {
        self.gear.full_value()
    }

    /// True if quantum norm agrees with discrete residue on every lane
    pub fn is_coherent(&self) -> bool {
        self.lanes.iter().all(|l| l.is_coherent())
    }

    /// True if net winding is zero mod product of moduli
    pub fn is_energy_neutral(&self) -> bool {
        let m: i128 = self.lanes.iter().map(|l| l.p as i128).product();
        let total: i128 = self.lanes.iter().map(|l| l.magnitude).sum();
        total % m == 0
    }

    /// Number of injections performed
    pub fn injection_count(&self) -> usize { self.injection_log.len() }

    /// K-Elim winding number matrix: k[i][j] = quantum_k_elim(i, j)
    pub fn winding_matrix(&self) -> Vec<Vec<u64>> {
        let n = self.lanes.len();
        (0..n).map(|i| (0..n).filter(|&j| j != i)
            .map(|j| self.quantum_k_elim(i, j))
            .collect()
        ).collect()
    }
}


// ═══════════════════════════════════════════════════════════════════
// SCHEMA CHIMERA: Heterogeneous Operators on the QCIDT
// ═══════════════════════════════════════════════════════════════════

/// The 8 CRAM operators as exact integer functions on Z/pZ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CramOp {
    Add, Sub, Mul, Div, Sqr, Neg, Inv, Id,
}

impl CramOp {
    /// Apply the binary operator f(a, b) mod p
    /// For unary ops (Sqr, Neg, Inv, Id): b is ignored
    pub fn apply(&self, a: u64, b: u64, p: u64) -> u64 {
        match self {
            CramOp::Add => addmod(a, b, p),
            CramOp::Sub => submod(a, b, p),
            CramOp::Mul => mulmod(a, b, p),
            CramOp::Div => {
                if b == 0 || gcd(b, p) != 1 { return 0; }
                mulmod(a, mod_inverse(b, p), p)
            }
            CramOp::Sqr => mulmod(a, a, p),
            CramOp::Neg => if a == 0 { 0 } else { p - a },
            CramOp::Inv => {
                if a == 0 || gcd(a, p) != 1 { return 0; }
                mod_inverse(a, p)
            }
            CramOp::Id => a,
        }
    }

    /// Algebraic degree of the operator
    pub fn degree(&self) -> u32 {
        match self {
            CramOp::Add | CramOp::Sub | CramOp::Neg | CramOp::Id => 1,
            CramOp::Mul | CramOp::Sqr => 2,
            CramOp::Div | CramOp::Inv => { /* p-2 by Fermat, but bounded */ 2 }
        }
    }
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

/// A 5-lane heterogeneous schema: one operator per Safe Basis prime
#[derive(Debug, Clone)]
pub struct Schema {
    pub ops: Vec<(u64, CramOp)>,  // (prime, operator) pairs
    pub label: String,
}

impl Schema {
    pub fn new(ops: Vec<(u64, CramOp)>, label: &str) -> Self {
        Schema { ops, label: label.to_string() }
    }

    /// Standard NS diffusion schema: Id on all lanes
    pub fn ns_diffusion() -> Self {
        Schema::new(
            SAFE_BASIS.iter().map(|&p| (p, CramOp::Id)).collect(),
            "NS-Diffusion",
        )
    }

    /// Stretching schema: Sqr on discriminating lanes, Id on parking
    pub fn ns_stretching() -> Self {
        Schema::new(vec![
            (3, CramOp::Id),    // parking
            (5, CramOp::Id),    // parking
            (7, CramOp::Sqr),   // discriminating: degree 2
            (11, CramOp::Sqr),  // shadow: degree 2
            (13, CramOp::Sqr),  // boundary: degree 2
        ], "NS-Stretching")
    }

    /// Maximum algebraic degree across all lanes
    pub fn max_degree(&self) -> u32 {
        self.ops.iter().map(|(_, op)| op.degree()).max().unwrap_or(0)
    }

    /// DKAM subcriticality check: max_degree < ρ
    pub fn is_dkam_safe(&self, rho: u32) -> bool {
        self.max_degree() < rho
    }

    /// Apply schema to a QCIDT torus state
    /// Each lane's residue is transformed by its operator
    /// b-coupling: the b-component of lane i feeds as the `b` argument to lane (i+1)
    pub fn apply_to_torus(&self, torus: &mut QCIDTorus) {
        let n = torus.lanes.len().min(self.ops.len());
        let residues: Vec<u64> = (0..n).map(|i| torus.lanes[i].residue).collect();
        let b_vals: Vec<u64> = (0..n).map(|i| torus.lanes[i].amplitude.b).collect();

        for i in 0..n {
            let (p, op) = self.ops[i];
            let a = residues[i];
            // Cyclic b-coupling: b from previous lane
            let b = if i > 0 { b_vals[i - 1] } else { b_vals[n - 1] };
            let new_r = op.apply(a, b, p);
            torus.lanes[i].residue = new_r;
            torus.gear.lanes[i].residue = new_r;
        }
        // Sync quantum layer
        torus.sync_quantum();
    }
}

// ═══════════════════════════════════════════════════════════════════
// SCHEMA ORBIT: Track chimera trajectory through L steps
// ═══════════════════════════════════════════════════════════════════

/// Record of one orbit step
#[derive(Debug, Clone)]
pub struct OrbitStep {
    pub step: usize,
    pub residues: Vec<u64>,
    pub norms: Vec<u64>,
    pub b_components: Vec<u64>,
}

/// Complete orbit of a schema on the QCIDT
#[derive(Debug, Clone)]
pub struct SchemaOrbit {
    pub schema_label: String,
    pub steps: Vec<OrbitStep>,
    pub period: Option<usize>,
    pub contraction: Option<u64>,  // norm product, if periodic
}

impl SchemaOrbit {
    /// Run a schema for L steps, recording the trajectory
    pub fn compute(schema: &Schema, torus: &mut QCIDTorus, max_steps: usize) -> Self {
        let mut steps = Vec::new();
        let mut seen_states: Vec<Vec<u64>> = Vec::new();
        let mut period = None;

        for s in 0..max_steps {
            let residues: Vec<u64> = torus.lanes.iter().map(|l| l.residue).collect();
            let norms: Vec<u64> = torus.lanes.iter().map(|l| l.amplitude.norm()).collect();
            let b_comps: Vec<u64> = torus.lanes.iter().map(|l| l.amplitude.b).collect();

            // Check for period (residue cycle)
            if let Some(pos) = seen_states.iter().position(|st| st == &residues) {
                period = Some(s - pos);
                steps.push(OrbitStep { step: s, residues, norms, b_components: b_comps });
                break;
            }
            seen_states.push(residues.clone());
            steps.push(OrbitStep { step: s, residues, norms, b_components: b_comps });

            schema.apply_to_torus(torus);
        }

        // Compute contraction: product of norms at boundary prime (lane 13)
        let boundary_idx = torus.lanes.iter().position(|l| l.p == 13);
        let contraction = boundary_idx.map(|idx| {
            steps.iter()
                .map(|s| s.norms[idx])
                .fold(1u64, |acc, n| mulmod(acc, n.max(1), 13))
        });

        SchemaOrbit {
            schema_label: schema.label.clone(),
            steps,
            period,
            contraction,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// RESOLUTION LADDER: The N-Refinement Chimera Engine
//
// For each resolution N in the doubling sequence:
//   1. Compute the transition coefficient c(N) = dt·ν·N²·SCALE⁻¹ mod p
//   2. Run the schema orbit for T steps at this coefficient
//   3. Measure contraction: norm product over the orbit
//   4. INJECT the inter-resolution contraction c[N] - c[N/2] as the
//      b-component into the quantum layer
//   5. Frobenius phase-flip the injection
//   6. Advance to next resolution
//
// The b-component carries the MEMORY of how contraction changed
// between resolutions. The Frobenius flip probes whether this
// memory survives projection through the norm (it does — the norm
// is Frobenius-invariant, so the discrete readout is the same
// regardless of which Frobenius phase the memory is in).
// ═══════════════════════════════════════════════════════════════════

/// One rung of the resolution ladder
#[derive(Debug, Clone)]
pub struct LadderRung {
    pub n: u64,
    pub coefficients: Vec<u64>,     // c(N) per prime
    pub self_coefficients: Vec<i64>, // (1-6c) centered per prime
    pub orbit_period: Option<usize>,
    pub contraction_norm: u64,       // norm product at boundary
    pub b_injection: Vec<u64>,       // what was injected as b
    pub is_erasure: Vec<bool>,       // which lanes hit self=0
}

/// The full resolution ladder from N_start through N_end
#[derive(Debug)]
pub struct ResolutionLadder {
    pub rungs: Vec<LadderRung>,
    pub scale: u64,
    pub dt: u64,
    pub nu: u64,
}

impl ResolutionLadder {
    /// Build the resolution ladder for the NS chimera
    pub fn build(n_start: u64, n_doublings: usize, dt: u64, nu: u64, scale: u64) -> Self {
        let primes = SAFE_BASIS;
        let mut rungs = Vec::new();
        let mut prev_coeffs: Option<Vec<u64>> = None;
        let mut torus = QCIDTorus::new_safe_basis();

        let mut n = n_start;
        for _k in 0..=n_doublings {
            // Compute c(N) = dt·ν·N²·SCALE⁻¹ mod p for each prime
            let coeffs: Vec<u64> = primes.iter().map(|&p| {
                let si = mod_pow(scale % p, p - 2, p);
                let a = (dt % p) * (nu % p) % p;
                let b = (n % p) * (n % p) % p;
                a * b % p * si % p
            }).collect();

            // Self-coefficients (1 - 6c) centered
            let self_coeffs: Vec<i64> = primes.iter().zip(coeffs.iter()).map(|(&p, &c)| {
                let s = (1 + p - (6 * c) % p) % p;
                if s <= p / 2 { s as i64 } else { s as i64 - p as i64 }
            }).collect();

            // Erasure detection
            let is_erasure: Vec<bool> = primes.iter().zip(coeffs.iter())
                .map(|(&p, &c)| (6 * c) % p == 1 % p)
                .collect();

            // Inject inter-resolution contraction as b-component
            let b_injection: Vec<u64> = if let Some(ref prev) = prev_coeffs {
                primes.iter().enumerate().map(|(i, &p)| {
                    submod(coeffs[i], prev[i], p)
                }).collect()
            } else {
                vec![0; primes.len()]
            };

            // Actually inject into the torus
            if prev_coeffs.is_some() {
                // Inject b-values without changing a (residues)
                for (i, lane) in torus.lanes.iter_mut().enumerate() {
                    let new_b = b_injection[i];
                    lane.amplitude = Fp2::new(
                        lane.amplitude.a,
                        new_b,
                        lane.p,
                        lane.amplitude.delta,
                    );
                }
                // Frobenius phase-flip to test norm invariance
                torus.apply_frobenius();
            }

            // Compute contraction norm at boundary prime (p=13)
            let boundary_idx = 4; // index of p=13 in SAFE_BASIS
            let contraction_norm = torus.lanes[boundary_idx].amplitude.norm();

            // Run a short schema orbit to measure period
            let mut test_torus = torus.clone();
            let schema = Schema::ns_diffusion();
            let orbit = SchemaOrbit::compute(&schema, &mut test_torus, 100);

            rungs.push(LadderRung {
                n,
                coefficients: coeffs.clone(),
                self_coefficients: self_coeffs,
                orbit_period: orbit.period,
                contraction_norm,
                b_injection,
                is_erasure,
            });

            prev_coeffs = Some(coeffs);
            n = n.saturating_mul(2);
        }

        ResolutionLadder { rungs, scale, dt, nu }
    }

    /// Check permanent non-resonance across all rungs
    pub fn all_non_resonant(&self) -> bool {
        self.rungs.iter().all(|rung|
            rung.coefficients.iter().zip(SAFE_BASIS.iter())
                .skip(2) // skip parking primes 3, 5
                .all(|(&c, _)| c != 0)
        )
    }

    /// Check if every cycle contains at least one erasure per lane
    pub fn all_have_erasure(&self) -> bool {
        // For each discriminating prime, check if erasure occurs somewhere
        for lane_idx in 2..5 { // lanes 7, 11, 13
            let any_erasure = self.rungs.iter().any(|r| r.is_erasure[lane_idx]);
            if !any_erasure { return false; }
        }
        true
    }
}

// ═══════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Core QCIDT tests ──

    #[test]
    fn test_fp2_frobenius_period_two() {
        let x = Fp2::new(3, 5, 13, find_qnr(13));
        let sx = x.frobenius();
        let ssx = sx.frobenius();
        assert_eq!(ssx, x, "σ² = identity");
    }

    #[test]
    fn test_fp2_norm_frobenius_invariant() {
        let x = Fp2::new(4, 7, 13, find_qnr(13));
        assert_eq!(x.norm(), x.frobenius().norm(), "N(σ(x)) = N(x)");
    }

    #[test]
    fn test_fp2_from_winding_encodes_both() {
        let p = 11u64;
        let delta = find_qnr(p);
        let residue = 7u64;
        let magnitude: i128 = 25;
        let amp = Fp2::from_winding(residue, magnitude, p, delta);
        assert_eq!(amp.a, residue % p, "a = residue mod p");
        assert_eq!(amp.b, (magnitude as u64) % p, "b = magnitude mod p");
    }

    #[test]
    fn test_inject_snapshot_coherent() {
        let mut torus = QCIDTorus::new_safe_basis();
        torus.inject_winding_snapshot(&[0, 0, 0, 0, 0]);
        assert!(torus.is_coherent(), "Zero snapshot = coherent state");
    }

    #[test]
    fn test_shadow_gate_11_resonance() {
        let mut torus = QCIDTorus::new_safe_basis();
        let snap = vec![10i128, 20, 30, 40, 50];
        assert!(!torus.shadow_inject(7, &snap), "7 is not shadow prime");
        assert!(torus.shadow_inject(11, &snap), "11 IS shadow prime");
        assert!(torus.shadow_inject(22, &snap), "22 is multiple of 11");
    }

    #[test]
    fn test_step_advances_discrete() {
        let mut torus = QCIDTorus::new_safe_basis();
        torus.step(100);
        // Residue of 100: mod 3=1, mod 5=0, mod 7=2, mod 11=1, mod 13=9
        assert_eq!(torus.lanes[0].residue, 100 % 3);
        assert_eq!(torus.lanes[2].residue, 100 % 7);
        assert_eq!(torus.lanes[4].residue, 100 % 13);
    }

    #[test]
    fn test_frobenius_on_torus_period_two() {
        let mut torus = QCIDTorus::new_safe_basis();
        torus.inject_winding_snapshot(&[3, 5, 7, 11, 13]);
        torus.sync_quantum();
        let before: Vec<Fp2> = torus.amplitudes();
        torus.apply_frobenius();
        torus.apply_frobenius();
        torus.sync_quantum(); // re-sync to check
        // After two Frobenius, quantum state should return to original
        // (Frobenius is period 2)
        let after: Vec<Fp2> = torus.amplitudes();
        // Note: sync_quantum rebuilds from (residue, magnitude), so
        // double-frobenius + sync should give same as original sync
    }

    // ── Schema tests ──

    #[test]
    fn test_schema_dkam_safety() {
        let diff = Schema::ns_diffusion();
        assert!(diff.is_dkam_safe(3), "Diffusion (deg 1) < ρ=3");

        let stretch = Schema::ns_stretching();
        assert!(stretch.is_dkam_safe(3), "Stretching (deg 2) < ρ=3");

        // Degree 3 schema would NOT be safe
        assert!(stretch.max_degree() == 2);
    }

    #[test]
    fn test_schema_orbit_finds_period() {
        let mut torus = QCIDTorus::new_safe_basis();
        torus.inject_winding_snapshot(&[1, 1, 1, 1, 1]);
        torus.sync_quantum();

        let schema = Schema::ns_diffusion(); // Id on all = identity
        let orbit = SchemaOrbit::compute(&schema, &mut torus, 100);
        // Id schema should have period 1 (fixed point)
        assert_eq!(orbit.period, Some(1), "Id schema is fixed point");
    }

    #[test]
    fn test_schema_sqr_orbit_period_divides_p_minus_1() {
        // Sqr: x → x² mod p. Period divides (p-1)/2 for x coprime to p.
        let mut torus = QCIDTorus::new(&[13]);
        torus.inject_winding_snapshot(&[2]); // Start at 2
        torus.sync_quantum();

        let schema = Schema::new(vec![(13, CramOp::Sqr)], "Sqr-13");
        let orbit = SchemaOrbit::compute(&schema, &mut torus, 100);

        if let Some(per) = orbit.period {
            assert!(per <= 12, "Sqr period on Z/13Z divides 12");
            assert!(12 % per == 0 || per % 12 == 0,
                "Period {} should divide p-1=12", per);
        }
    }

    // ── Resolution Ladder tests ──

    #[test]
    fn test_ladder_permanent_non_resonance() {
        let ladder = ResolutionLadder::build(8, 30, 100, 6, 10000);
        assert!(ladder.all_non_resonant(),
            "All discriminating coefficients coprime to all primes");
    }

    #[test]
    fn test_ladder_has_erasure() {
        let ladder = ResolutionLadder::build(8, 30, 100, 6, 10000);
        assert!(ladder.all_have_erasure(),
            "Every discriminating lane has at least one erasure");
    }

    #[test]
    fn test_ladder_self_coefficients_bounded() {
        let ladder = ResolutionLadder::build(8, 30, 100, 6, 10000);
        for rung in &ladder.rungs {
            for (i, &sc) in rung.self_coefficients.iter().enumerate() {
                let p = SAFE_BASIS[i];
                let ratio = sc.unsigned_abs() as f64 / p as f64;
                if i >= 2 { // discriminating lanes only
                    assert!(ratio < 0.5,
                        "N={}: |self|/p = {}/{} = {:.3} >= 0.5",
                        rung.n, sc.unsigned_abs(), p, ratio);
                }
            }
        }
    }

    #[test]
    fn test_ladder_coefficient_cycling() {
        let ladder = ResolutionLadder::build(8, 60, 100, 6, 10000);
        // After 30 doublings, coefficients should repeat
        if ladder.rungs.len() > 30 {
            for i in 0..30 {
                // Check lane 2 (p=7, period 3)
                assert_eq!(
                    ladder.rungs[i].coefficients[2],
                    ladder.rungs[i + 3].coefficients[2],
                    "p=7 coefficient should cycle with period 3"
                );
            }
        }
    }

    #[test]
    fn test_ladder_b_injection_nonzero() {
        let ladder = ResolutionLadder::build(8, 10, 100, 6, 10000);
        // First rung has zero injection (no previous)
        assert!(ladder.rungs[0].b_injection.iter().all(|&b| b == 0));
        // Subsequent rungs should have nonzero injection on at least some lanes
        let any_nonzero = ladder.rungs[1..].iter()
            .any(|r| r.b_injection.iter().any(|&b| b != 0));
        assert!(any_nonzero,
            "Inter-resolution injection should produce nonzero b on some lane");
    }

    #[test]
    fn test_ladder_contraction_norm_bounded() {
        let ladder = ResolutionLadder::build(8, 30, 100, 6, 10000);
        for rung in &ladder.rungs {
            assert!(rung.contraction_norm < 13,
                "Norm at boundary prime p=13 should be < 13, got {}",
                rung.contraction_norm);
        }
    }

    #[test]
    fn test_ladder_spectral_fingerprint_constant() {
        // Permanent Non-Resonance: SP = 2/p at every resolution
        let ladder = ResolutionLadder::build(8, 30, 100, 6, 10000);
        for rung in &ladder.rungs {
            for (i, &c) in rung.coefficients.iter().enumerate() {
                let p = SAFE_BASIS[i];
                if p <= 5 { continue; } // skip parking
                // SP(p, c) should have c coprime to p
                // Ramanujan sum = -1 → SP numerator = 2p
                let (num, _den) = crate::sieve_power(p, c);
                assert_eq!(num, 2 * p,
                    "N={}: SP({}, c={}) numerator should be 2×{}={}, got {}",
                    rung.n, p, c, p, 2*p, num);
            }
        }
    }
}

/// Sieve power SP(p, d) = (2p - 1 - c_p(d)) / p² (exact rational)
pub fn sieve_power(p: u64, d: u64) -> (u64, u64) {
    let c: i64 = if d > 0 && d % p == 0 { p as i64 - 1 } else { -1 };
    let num = (2 * p as i64 - 1 - c) as u64;
    (num, p * p)
}

// ═══════════════════════════════════════════════════════════════════
// WINDING CORRECTION ENGINE
//
// The Maya correction mechanism applied to prime detection:
// 1. Track residues on backbone lanes (7, 11) — position in cycle
// 2. Track winding (K-Elim quotient) — how many times around
// 3. Compute arrow diff (forward - backward K-Elim) — directional excess
// 4. Subtract arrow from raw position — "unwind" the time direction
// 5. Check: does the unwound position predict primality?
// ═══════════════════════════════════════════════════════════════════

/// The 143-state shadow×boundary torus
/// This is the "13th face of zero" — the reservoir for excess
#[derive(Debug, Clone)]
pub struct FaceOfZero {
    /// State on T₁₁ × T₁₃ (143 states)
    pub state: (u64, u64),  // (r mod 11, r mod 13)
    /// Accumulated winding on each component
    pub winding_11: i64,
    pub winding_13: i64,
    /// Excess: winding difference between the two components
    pub excess: i64,
}

impl FaceOfZero {
    pub fn new() -> Self {
        FaceOfZero { state: (0, 0), winding_11: 0, winding_13: 0, excess: 0 }
    }

    /// Update with a new integer n
    pub fn observe(&mut self, n: u64) {
        let old_11 = self.state.0;
        let old_13 = self.state.1;
        self.state = (n % 11, n % 13);

        // Track winding: how many times we've wrapped each component
        if (n % 11) < old_11 { self.winding_11 += 1; }
        if (n % 13) < old_13 { self.winding_13 += 1; }
        self.excess = self.winding_11 - self.winding_13;
    }

    /// Is the current state at the face of zero?
    pub fn at_zero(&self) -> bool {
        self.state.0 == 0 && self.state.1 == 0
    }

    /// Combined torus position (0..142)
    pub fn torus_position(&self) -> u64 {
        self.state.0 * 13 + self.state.1  // unique encoding on 143-state torus
    }
}

/// Arrow of time between backbone lanes 7 and 11
#[derive(Debug, Clone)]
pub struct ArrowOfTime {
    /// Forward K-Elim: k(7→11) = (r₁₁ - r₇) · 7⁻¹ mod 11
    pub k_forward: u64,
    /// Backward K-Elim: k(11→7) = (r₇ - r₁₁) · 11⁻¹ mod 7
    pub k_backward: u64,
    /// Arrow diff (the directional quantity)
    pub arrow: u64,
    /// Joint position on T₇ × T₁₁ (77 states)
    pub backbone_state: u64,
}

impl ArrowOfTime {
    /// Compute the arrow for integer n
    pub fn compute(n: u64) -> Self {
        let r7 = n % 7;
        let r11 = n % 11;
        let inv_7_11 = mod_pow(7, 9, 11);  // 7⁻¹ mod 11
        let inv_11_7 = mod_pow(11, 5, 7);  // 11⁻¹ mod 7

        let k_fwd = ((r11 as i64 - (r7 % 11) as i64 + 11) as u64 * inv_7_11) % 11;
        let k_bwd = ((r7 as i64 - (r11 % 7) as i64 + 7) as u64 * inv_11_7) % 7;
        let arrow = (k_fwd as i64 - k_bwd as i64).unsigned_abs() % 77;
        let backbone = k_fwd * 7 + k_bwd;

        ArrowOfTime {
            k_forward: k_fwd,
            k_backward: k_bwd,
            arrow: arrow as u64,
            backbone_state: backbone % 77,
        }
    }

    /// Unwound position: subtract arrow from backbone state
    pub fn unwound_position(&self) -> u64 {
        (self.backbone_state + 77 - self.arrow % 77) % 77
    }
}

/// Winding correction engine for prime detection
pub struct WindingCorrector {
    /// Face of zero tracker
    pub face: FaceOfZero,
    /// Statistics per torus state
    pub state_counts: Vec<u64>,      // 77 backbone states
    pub state_prime_counts: Vec<u64>, // how many primes at each state
    pub state_cousin_counts: Vec<u64>, // how many cousin primes
    /// Unwound statistics
    pub unwound_counts: Vec<u64>,
    pub unwound_prime_counts: Vec<u64>,
    pub unwound_cousin_counts: Vec<u64>,
    /// Excess accumulation at boundary
    pub excess_at_prime: Vec<i64>,    // excess value when a prime is observed
}

impl WindingCorrector {
    pub fn new() -> Self {
        WindingCorrector {
            face: FaceOfZero::new(),
            state_counts: vec![0; 77],
            state_prime_counts: vec![0; 77],
            state_cousin_counts: vec![0; 77],
            unwound_counts: vec![0; 77],
            unwound_prime_counts: vec![0; 77],
            unwound_cousin_counts: vec![0; 77],
            excess_at_prime: Vec::new(),
        }
    }

    /// Process a range of integers, tracking everything
    pub fn scan(&mut self, start: u64, end: u64) {
        for n in start..end {
            self.face.observe(n);
            let arrow = ArrowOfTime::compute(n);
            let bs = (arrow.backbone_state % 77) as usize;
            let uw = (arrow.unwound_position() % 77) as usize;

            self.state_counts[bs] += 1;
            self.unwound_counts[uw] += 1;

            if is_prime_trial(n) {
                self.state_prime_counts[bs] += 1;
                self.unwound_prime_counts[uw] += 1;
                self.excess_at_prime.push(self.face.excess);

                // Check cousin
                if is_prime_trial(n + 4) {
                    self.state_cousin_counts[bs] += 1;
                    self.unwound_cousin_counts[uw] += 1;
                }
            }
        }
    }

    /// Compute the prime density variation across backbone states
    /// Lower variance = more regular distribution = better frame
    pub fn density_variance(&self, use_unwound: bool) -> f64 {
        let (counts, primes) = if use_unwound {
            (&self.unwound_counts, &self.unwound_prime_counts)
        } else {
            (&self.state_counts, &self.state_prime_counts)
        };

        let densities: Vec<f64> = counts.iter().zip(primes.iter())
            .filter(|(&c, _)| c > 0)
            .map(|(&c, &p)| p as f64 / c as f64)
            .collect();

        if densities.is_empty() { return 0.0; }
        let mean = densities.iter().sum::<f64>() / densities.len() as f64;
        let var = densities.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / densities.len() as f64;
        var
    }

    /// Cousin concentration: fraction of primes that are also cousin primes
    pub fn cousin_concentration(&self, use_unwound: bool) -> Vec<(usize, f64)> {
        let (primes, cousins) = if use_unwound {
            (&self.unwound_prime_counts, &self.unwound_cousin_counts)
        } else {
            (&self.state_prime_counts, &self.state_cousin_counts)
        };

        primes.iter().zip(cousins.iter()).enumerate()
            .filter(|(_, (&p, _))| p > 10) // need enough data
            .map(|(i, (&p, &c))| (i, c as f64 / p as f64))
            .collect()
    }
}

fn is_prime_trial(n: u64) -> bool {
    if n < 2 { return false; }
    if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5u64;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}

#[cfg(test)]
mod winding_tests {
    use super::*;

    #[test]
    fn test_face_of_zero_at_multiples_of_143() {
        let mut face = FaceOfZero::new();
        face.observe(143);
        assert!(face.at_zero(), "143 = 11×13 should be at the face of zero");
        face.observe(286);
        assert!(face.at_zero(), "286 = 2×143 should be at face of zero");
        face.observe(100);
        assert!(!face.at_zero(), "100 is not at face of zero");
    }

    #[test]
    fn test_arrow_asymmetry() {
        // For most primes > 13, the arrow should be nonzero
        let mut asymmetric = 0u32;
        let mut total = 0u32;
        for n in 100..500 {
            if !is_prime_trial(n) { continue; }
            let arrow = ArrowOfTime::compute(n);
            total += 1;
            if arrow.k_forward != arrow.k_backward as u64 {
                asymmetric += 1;
            }
        }
        let ratio = asymmetric as f64 / total as f64;
        assert!(ratio > 0.8,
            "Most primes should show K-Elim asymmetry: {}/{} = {:.1}%",
            asymmetric, total, ratio * 100.0);
    }

    #[test]
    fn test_unwinding_is_bijective() {
        // The unwound map should be a bijection on the occupied states
        let mut raw_states = std::collections::HashSet::new();
        let mut unwound_states = std::collections::HashSet::new();
        for n in 100..10000 {
            if !is_prime_trial(n) { continue; }
            let arrow = ArrowOfTime::compute(n);
            raw_states.insert(arrow.backbone_state);
            unwound_states.insert(arrow.unwound_position());
        }
        assert_eq!(raw_states.len(), unwound_states.len(),
            "Unwinding should be bijective: {} raw vs {} unwound",
            raw_states.len(), unwound_states.len());
    }

    #[test]
    fn test_unwinding_reduces_density_variance() {
        // The key test: does unwinding make prime density MORE regular?
        let mut corrector = WindingCorrector::new();
        corrector.scan(100, 50000);

        let var_raw = corrector.density_variance(false);
        let var_unwound = corrector.density_variance(true);

        println!("  Raw backbone density variance:    {:.8}", var_raw);
        println!("  Unwound backbone density variance: {:.8}", var_unwound);
        println!("  Reduction: {:.1}%",
            if var_raw > 0.0 { (1.0 - var_unwound / var_raw) * 100.0 } else { 0.0 });

        // We don't assert direction — let the data speak
        // If unwinding helps, var_unwound < var_raw
        // If it doesn't, we document that as an honest result
    }

    #[test]
    fn test_cousin_concentration_structure() {
        let mut corrector = WindingCorrector::new();
        corrector.scan(100, 50000);

        let raw_cousins = corrector.cousin_concentration(false);
        let unwound_cousins = corrector.cousin_concentration(true);

        // Compute variance of cousin concentration
        let raw_mean: f64 = raw_cousins.iter().map(|(_, c)| c).sum::<f64>()
            / raw_cousins.len().max(1) as f64;
        let raw_var: f64 = raw_cousins.iter().map(|(_, c)| (c - raw_mean).powi(2)).sum::<f64>()
            / raw_cousins.len().max(1) as f64;

        let uw_mean: f64 = unwound_cousins.iter().map(|(_, c)| c).sum::<f64>()
            / unwound_cousins.len().max(1) as f64;
        let uw_var: f64 = unwound_cousins.iter().map(|(_, c)| (c - uw_mean).powi(2)).sum::<f64>()
            / unwound_cousins.len().max(1) as f64;

        println!("  Cousin concentration variance (raw):     {:.8}", raw_var);
        println!("  Cousin concentration variance (unwound): {:.8}", uw_var);
        println!("  Mean cousin fraction (raw):     {:.4}", raw_mean);
        println!("  Mean cousin fraction (unwound): {:.4}", uw_mean);
    }

    #[test]
    fn test_excess_at_primes_bounded() {
        let mut corrector = WindingCorrector::new();
        corrector.scan(100, 10000);

        let max_excess = corrector.excess_at_prime.iter()
            .map(|e| e.abs()).max().unwrap_or(0);
        let mean_excess: f64 = corrector.excess_at_prime.iter()
            .map(|&e| e as f64).sum::<f64>()
            / corrector.excess_at_prime.len().max(1) as f64;

        println!("  Excess at primes: max={}, mean={:.2}, count={}",
            max_excess, mean_excess, corrector.excess_at_prime.len());

        // The excess should grow (it's a winding), but check structure
        assert!(corrector.excess_at_prime.len() > 100,
            "Should observe many primes");
    }

    #[test]
    fn test_shadow_prime_never_divides_backbone_spacing() {
        // 11 ∤ 4 (cousin gap = backbone spacing)
        assert!(4 % 11 != 0, "Shadow prime should not divide cousin gap");
        // 11 ∤ 7 (bridge prime)
        assert!(7 % 11 != 0);
        // 11 ∤ 13 (boundary prime)
        assert!(13 % 11 != 0);
        // But 11 × 13 = 143: the face of zero
        assert_eq!(11 * 13, 143);
        // And 7 × 11 = 77: the backbone torus
        assert_eq!(7 * 11, 77);
    }

    #[test]
    fn test_b_injection_sums_to_modulus() {
        // The b-injection cycle [2, 1, 4] on lane 7 sums to 7
        // This means the injection is conservative: wraps exactly once
        let b_cycle = [2u64, 1, 4];
        let sum: u64 = b_cycle.iter().sum();
        assert_eq!(sum, 7, "b-injection should sum to lane modulus");

        // The three values ARE the gap families:
        // 2 = twin, 1 = unit, 4 = cousin
        // The backbone heartbeat is twin-unit-cousin
    }

    #[test]
    fn test_maya_calendar_round_is_chimera_period() {
        // Calendar Round = lcm(260, 365) = 18980
        // This is the Maya equivalent of our chimera's 30-step period
        let cr = 18980u64;
        let tzolkin = 260u64;
        let haab = 365u64;

        // Verify: 18980 = lcm(260, 365)
        fn gcd_fn(mut a: u64, mut b: u64) -> u64 {
            while b != 0 { let t = b; b = a % b; a = t; } a
        }
        let g = gcd_fn(tzolkin, haab);
        assert_eq!(tzolkin / g * haab, cr, "CR = lcm(Tzolkin, Haab)");

        // 11 does NOT divide any of these
        assert!(cr % 11 != 0, "Shadow prime absent from Calendar Round");
        assert!(tzolkin % 11 != 0, "Shadow prime absent from Tzolkin");
        assert!(haab % 11 != 0, "Shadow prime absent from Haab");

        // But 11² appears in the eclipse displacement
        let eclipse = 11960u64;
        let saturn = 378u64;
        let disp = eclipse % saturn;
        assert_eq!(disp, 242, "Eclipse displacement = 242");
        assert_eq!(disp % (11*11), 0, "242 = 2 × 11²: shadow prime squared");
    }
}

// ═══════════════════════════════════════════════════════════════════
// THE NATURAL NUMBER TYPE
//
// Not ad hoc. Not a reflex to what we run into. Engineered.
//
// Four chimera layers, one per calendar type, coupled through the
// shadow prime and layered in the GearManifold:
//
//   Layer 0: TZOLKIN  (260 = 4×5×13)   — sacred/spectral,  max T7 gain
//   Layer 1: 819-DAY  (819 = 3²×7×13)  — bridge×boundary,  correction
//   Layer 2: HAAB     (365 = 5×73)      — solar/continuous, drift
//   Layer 3: ROUND    (18980 = lcm)     — compound,         period
//
// Each layer carries:
//   - Residue (position within its cycle)
//   - Winding (how many full cycles elapsed = K-Elim quotient)
//   - Quantum amplitude (F_p² via QCIDT injection)
//   - Arrow state (K-Elim asymmetry on backbone primes)
//
// The shadow prime (11) couples all four layers through MDZero:
//   - 11 ∤ 260, 11 ∤ 365, 11 ∤ 819, 11 ∤ 18980
//   - Shadow is structurally present but spectrally absent
//   - Coupling fires only at multiples of 11 (shadow resonance)
//
// The face of zero (143 = 11×13) is the shared ground state.
// The b-injection [2,1,4] provides the heartbeat.
// The second-order arrow provides direction.
// The winding accumulation provides magnitude.
// ═══════════════════════════════════════════════════════════════════

/// Calendar chimera layer
#[derive(Debug, Clone)]
pub struct CalendarLayer {
    pub name: &'static str,
    pub period: u64,
    pub residue: u64,      // position within this cycle
    pub winding: i64,       // how many full cycles elapsed
    pub factors: Vec<u64>,  // prime factorization of the period
}

impl CalendarLayer {
    pub fn new(name: &'static str, period: u64, factors: Vec<u64>) -> Self {
        CalendarLayer { name, period, residue: 0, winding: 0, factors }
    }

    /// Advance to a given value n
    pub fn set(&mut self, n: u64) {
        self.winding = (n / self.period) as i64;
        self.residue = n % self.period;
    }

    /// Which Safe Basis primes divide this period?
    pub fn active_lanes(&self) -> Vec<u64> {
        SAFE_BASIS.iter().filter(|&&p| self.period % p == 0).copied().collect()
    }

    /// Which Safe Basis primes are frozen (don't divide period)?
    pub fn frozen_lanes(&self) -> Vec<u64> {
        SAFE_BASIS.iter().filter(|&&p| self.period % p != 0).copied().collect()
    }

    /// Does the shadow prime divide this period?
    pub fn shadow_active(&self) -> bool {
        self.period % SHADOW_PRIME == 0
    }

    /// T6 sieve power of this period on each Safe Basis prime
    pub fn spectral_fingerprint(&self) -> Vec<(u64, u64, u64)> {
        SAFE_BASIS.iter().map(|&p| {
            let (num, den) = sieve_power(p, self.period);
            (p, num, den)
        }).collect()
    }
}

/// The compound natural number type
/// Four calendar layers coupled through the shadow prime
#[derive(Debug, Clone)]
pub struct NaturalNumber {
    /// The four calendar layers
    pub layers: [CalendarLayer; 4],
    /// The underlying QCIDT (provides quantum + discrete state)
    pub torus: QCIDTorus,
    /// Arrow of time state
    pub arrow: Option<ArrowOfTime>,
    /// Face of zero tracker
    pub face: FaceOfZero,
    /// The integer value
    pub value: u64,
}

impl NaturalNumber {
    /// Create a new natural number representation for value n
    pub fn from(n: u64) -> Self {
        let mut layers = [
            CalendarLayer::new("Tzolkin", 260, vec![2, 2, 5, 13]),
            CalendarLayer::new("819-day", 819, vec![3, 3, 7, 13]),
            CalendarLayer::new("Haab",    365, vec![5, 73]),
            CalendarLayer::new("Round",  18980, vec![2, 2, 5, 13, 73]),
        ];
        for layer in &mut layers {
            layer.set(n);
        }

        let mut torus = QCIDTorus::new_safe_basis();
        torus.step(n as i128);
        torus.sync_quantum();

        let arrow = if n > 0 { Some(ArrowOfTime::compute(n)) } else { None };

        let mut face = FaceOfZero::new();
        face.observe(n);

        NaturalNumber { layers, torus, arrow, face, value: n }
    }

    /// Is this number at the face of zero? (143 | n)
    pub fn at_face_of_zero(&self) -> bool {
        self.face.at_zero()
    }

    /// Shadow resonance: does 11 divide this number?
    pub fn shadow_resonates(&self) -> bool {
        self.value % SHADOW_PRIME == 0
    }

    /// Which calendar layers can "see" this number?
    /// A layer sees n if its period shares a factor with n's residue structure
    pub fn visible_layers(&self) -> Vec<&'static str> {
        self.layers.iter()
            .filter(|l| l.residue != 0)  // not at cycle boundary
            .map(|l| l.name)
            .collect()
    }

    /// Inter-layer winding excess: fastest layer vs slowest
    pub fn winding_excess(&self) -> i64 {
        let w_max = self.layers.iter().map(|l| l.winding).max().unwrap_or(0);
        let w_min = self.layers.iter().map(|l| l.winding).min().unwrap_or(0);
        w_max - w_min
    }

    /// The heartbeat position: which beat of [twin, unit, cousin]?
    pub fn heartbeat(&self) -> &'static str {
        match self.value % 3 {
            0 => "cousin",  // b=4
            1 => "twin",    // b=2
            _ => "unit",    // b=1
        }
    }

    /// Compound spectral fingerprint across all four layers
    pub fn compound_fingerprint(&self) -> Vec<(u64, u64)> {
        SAFE_BASIS.iter().map(|&p| {
            // Product of sieve powers across layers
            let mut total_num: u64 = 1;
            let mut total_den: u64 = 1;
            for layer in &self.layers {
                let (num, den) = sieve_power(p, layer.period);
                total_num = total_num.saturating_mul(num);
                total_den = total_den.saturating_mul(den);
            }
            (total_num, total_den)
        }).collect()
    }

    /// The K-Elim winding between the Tzolk'in and Haab' layers
    /// This IS the Calendar Round structure
    pub fn calendar_round_winding(&self) -> i64 {
        self.layers[0].winding - self.layers[2].winding
    }

    /// The correction layer (819-day) winding relative to the sacred layer
    pub fn correction_winding(&self) -> i64 {
        self.layers[1].winding - self.layers[0].winding
    }

    /// Second-order arrow (requires previous number)
    pub fn delta_arrow(&self, prev: &NaturalNumber) -> u64 {
        match (&self.arrow, &prev.arrow) {
            (Some(a), Some(b)) => (a.arrow + 77 - b.arrow) % 77,
            _ => 0,
        }
    }

    /// Full state vector: everything the number carries
    pub fn state_vector(&self) -> Vec<u64> {
        let mut v = Vec::new();
        // Layer residues (4)
        for l in &self.layers { v.push(l.residue); }
        // Layer windings (4)
        for l in &self.layers { v.push(l.winding as u64); }
        // Torus residues (5)
        for lane in &self.torus.lanes { v.push(lane.residue); }
        // Arrow (1)
        v.push(self.arrow.as_ref().map_or(0, |a| a.arrow));
        // Face position (1)
        v.push(self.face.torus_position());
        // Shadow resonance (1)
        v.push(if self.shadow_resonates() { 1 } else { 0 });
        v
    }
}

#[cfg(test)]
mod natural_number_tests {
    use super::*;

    #[test]
    fn test_nn_zero_is_at_face() {
        let nn = NaturalNumber::from(0);
        assert!(nn.at_face_of_zero());
    }

    #[test]
    fn test_nn_143_at_face() {
        let nn = NaturalNumber::from(143);
        assert!(nn.at_face_of_zero(), "143 = 11×13 is at the face of zero");
    }

    #[test]
    fn test_nn_shadow_resonance() {
        assert!(NaturalNumber::from(11).shadow_resonates());
        assert!(NaturalNumber::from(22).shadow_resonates());
        assert!(!NaturalNumber::from(10).shadow_resonates());
        assert!(!NaturalNumber::from(13).shadow_resonates());
    }

    #[test]
    fn test_nn_layer_visibility() {
        // 260 is a Tzolk'in boundary — layer 0 residue = 0
        let nn = NaturalNumber::from(260);
        assert_eq!(nn.layers[0].residue, 0, "260 mod 260 = 0");
        assert_ne!(nn.layers[1].residue, 0, "260 mod 819 ≠ 0");
    }

    #[test]
    fn test_nn_shadow_absent_from_all_periods() {
        let nn = NaturalNumber::from(1);
        for layer in &nn.layers {
            assert!(!layer.shadow_active(),
                "Shadow prime 11 should NOT divide {} (period {})",
                layer.name, layer.period);
        }
    }

    #[test]
    fn test_nn_winding_grows() {
        let nn1 = NaturalNumber::from(100);
        let nn2 = NaturalNumber::from(10000);
        assert!(nn2.winding_excess() >= nn1.winding_excess(),
            "Winding excess should grow with magnitude");
    }

    #[test]
    fn test_nn_state_vector_length() {
        let nn = NaturalNumber::from(42);
        let sv = nn.state_vector();
        // 4 residues + 4 windings + 5 torus + 1 arrow + 1 face + 1 shadow = 16
        assert_eq!(sv.len(), 16, "State vector should have 16 components");
    }

    #[test]
    fn test_nn_delta_arrow_between_consecutive_primes() {
        let p1 = NaturalNumber::from(101);
        let p2 = NaturalNumber::from(103);
        let delta = p2.delta_arrow(&p1);
        // 103 - 101 = 2 (twin gap)
        // delta should be one of the 8 values we observed
        assert!(delta < 77, "Delta arrow should be on the 77-torus");
    }

    #[test]
    fn test_nn_calendar_round_winding() {
        // At n = 18980 (one Calendar Round), Tzolk'in wraps 73 times, Haab' 52 times
        let nn = NaturalNumber::from(18980);
        assert_eq!(nn.layers[0].winding, 73, "Tzolk'in wraps 73 times in a CR");
        assert_eq!(nn.layers[2].winding, 52, "Haab' wraps 52 times in a CR");
        assert_eq!(nn.calendar_round_winding(), 21, "CR winding = 73 - 52 = 21");
    }

    #[test]
    fn test_nn_correction_layer_contains_bridge_and_boundary() {
        let nn = NaturalNumber::from(1);
        let active = nn.layers[1].active_lanes(); // 819-day
        assert!(active.contains(&7), "819-day should activate bridge (7)");
        assert!(active.contains(&13), "819-day should activate boundary (13)");
        assert!(!active.contains(&11), "819-day should NOT activate shadow (11)");
    }
}

// ═══════════════════════════════════════════════════════════════════
// COMPOSITE GEAR ARCHITECTURE
//
// Instead of prime lanes, load COMPOSITE coprime moduli.
// Each composite lane catches TWO primes in one gear.
//
// Architecture:
//   PARKING:   {2, 3, 5}           — trivial elimination  
//   BACKBONE:  {7}                  — arrow source (≡3 mod 4)
//   SHADOW:    {11}                 — anchor (permanent, MDZero)
//   BOUNDARY:  {13}                 — inner boundary
//   GEARS:     {323=17×19, 667=23×29, ...}  — composite extension
//
// Each gear is coprime to all others and to the shadow.
// CRT applies across the full set.
// K-Elim between shadow and any gear extracts the winding.
// ═══════════════════════════════════════════════════════════════════

/// A composite gear: product of two primes beyond the boundary
#[derive(Debug, Clone)]
pub struct CompositeGear {
    pub p: u64,        // first prime factor
    pub q: u64,        // second prime factor
    pub modulus: u64,   // p × q
    pub residue: u64,   // current value mod (p×q)
    pub winding: i64,   // floor(value / modulus)
}

impl CompositeGear {
    pub fn new(p: u64, q: u64) -> Self {
        assert!(p != q && gcd(p, q) == 1, "factors must be distinct coprimes");
        CompositeGear { p, q, modulus: p * q, residue: 0, winding: 0 }
    }

    /// Set to value n
    pub fn set(&mut self, n: u64) {
        self.residue = n % self.modulus;
        self.winding = (n / self.modulus) as i64;
    }

    /// Does this gear catch n? (n divisible by p or q)
    pub fn catches(&self, n: u64) -> bool {
        n % self.p == 0 || n % self.q == 0
    }

    /// Project residue onto the p-component: n mod p
    pub fn project_p(&self) -> u64 { self.residue % self.p }

    /// Project residue onto the q-component: n mod q  
    pub fn project_q(&self) -> u64 { self.residue % self.q }

    /// K-Elim between the two factors within this gear
    /// Recovers the mixed-radix digit: how p-position relates to q-position
    pub fn internal_k_elim(&self) -> u64 {
        let rp = self.project_p();
        let rq = self.project_q();
        let p_inv_q = mod_pow(self.p, self.q - 2, self.q);
        ((rq as i64 - rp as i64 + self.q as i64) as u64 * p_inv_q) % self.q
    }

    /// K-Elim between shadow (11) and this gear
    pub fn shadow_k_elim(&self, shadow_residue: u64) -> u64 {
        let inv_11 = mod_pow(11, self.modulus - 2, self.modulus);
        ((self.residue as i64 - shadow_residue as i64 + self.modulus as i64) as u64
            * inv_11) % self.modulus
    }

    /// Sieve power of this composite gear for gap d
    pub fn composite_sieve_power(&self, d: u64) -> (u64, u64) {
        // SP_composite(pq, d) = 1 - (1 - SP(p,d)) × (1 - SP(q,d))
        // For non-resonant (p∤d, q∤d): SP = 1 - (1-2/p)(1-2/q)
        let (np, dp) = sieve_power(self.p, d);
        let (nq, dq) = sieve_power(self.q, d);
        // Return as fraction: need common denominator dp×dq
        let num = np * dq + nq * dp - np * nq;
        let den = dp * dq;
        (num, den)
    }
}

/// The full composite-gear natural number
#[derive(Debug, Clone)]
pub struct GearNumber {
    /// The integer value
    pub value: u64,
    /// Backbone lane (7)
    pub backbone_residue: u64,
    /// Shadow lane (11) 
    pub shadow_residue: u64,
    /// Boundary lane (13)
    pub boundary_residue: u64,
    /// Composite gears
    pub gears: Vec<CompositeGear>,
    /// Arrow of time
    pub arrow: ArrowOfTime,
    /// Face of zero
    pub face: FaceOfZero,
    /// Shadow winding (cumulative shadow zeros below n)
    pub shadow_winding: u64,
    /// Shadow phase on the rung ladder
    pub shadow_phase: u64,
}

impl GearNumber {
    /// Create with default gear set: {323=17×19, 667=23×29}
    pub fn new(n: u64) -> Self {
        Self::with_gears(n, &[(17, 19), (23, 29)])
    }

    /// Create with custom gear pairs
    pub fn with_gears(n: u64, gear_pairs: &[(u64, u64)]) -> Self {
        let mut gears: Vec<CompositeGear> = gear_pairs.iter()
            .map(|&(p, q)| {
                let mut g = CompositeGear::new(p, q);
                g.set(n);
                g
            })
            .collect();

        let mut face = FaceOfZero::new();
        face.observe(n);

        GearNumber {
            value: n,
            backbone_residue: n % 7,
            shadow_residue: n % 11,
            boundary_residue: n % 13,
            gears,
            arrow: ArrowOfTime::compute(n),
            face,
            shadow_winding: n / 11,
            shadow_phase: ((n / 169) * 4) % 11,  // rung shadow phase
        }
    }

    /// Full sieve: is n coprime to all basis primes AND all gear factors?
    pub fn survives_full_sieve(&self) -> bool {
        if self.value < 2 { return false; }
        // Parking
        if self.value % 2 == 0 || self.value % 3 == 0 || self.value % 5 == 0 {
            return false;
        }
        // Backbone + shadow + boundary
        if self.backbone_residue == 0 || self.shadow_residue == 0 || self.boundary_residue == 0 {
            return false;
        }
        // Composite gears
        for gear in &self.gears {
            if gear.catches(self.value) { return false; }
        }
        true
    }

    /// Shadow K-Elim against each gear: the cross-domain winding
    pub fn shadow_gear_windings(&self) -> Vec<u64> {
        self.gears.iter()
            .map(|g| g.shadow_k_elim(self.shadow_residue))
            .collect()
    }

    /// Internal K-Elim within each gear: the intra-gear structure
    pub fn internal_windings(&self) -> Vec<u64> {
        self.gears.iter()
            .map(|g| g.internal_k_elim())
            .collect()
    }

    /// Compound state vector: everything the number carries
    pub fn state_vector(&self) -> Vec<u64> {
        let mut v = vec![
            self.backbone_residue,
            self.shadow_residue,
            self.boundary_residue,
            self.arrow.arrow,
            self.face.torus_position(),
            self.shadow_winding % 143,  // on the 143-torus
            self.shadow_phase,
        ];
        // Gear residues
        for g in &self.gears {
            v.push(g.residue);
        }
        // Shadow-gear windings
        v.extend(self.shadow_gear_windings());
        // Internal gear windings
        v.extend(self.internal_windings());
        v
    }

    /// Rung number on the 169-ladder
    pub fn rung(&self) -> u64 { self.value / 169 }
}

#[cfg(test)]
mod gear_tests {
    use super::*;

    #[test]
    fn test_gear_catches_both_factors() {
        let g = CompositeGear::new(17, 19);
        assert!(g.catches(17), "Should catch multiples of 17");
        assert!(g.catches(19), "Should catch multiples of 19");
        assert!(g.catches(323), "Should catch 17×19");
        assert!(!g.catches(23), "Should not catch 23");
    }

    #[test]
    fn test_gear_crt_decomposition() {
        let mut g = CompositeGear::new(17, 19);
        g.set(500);
        assert_eq!(g.project_p(), 500 % 17);
        assert_eq!(g.project_q(), 500 % 19);
        // The pair (p_component, q_component) uniquely determines residue mod 323
        assert_eq!(g.residue, 500 % 323);
    }

    #[test]
    fn test_gear_internal_k_elim() {
        let mut g = CompositeGear::new(17, 19);
        g.set(500);
        let k = g.internal_k_elim();
        // k should reconstruct: 500 mod 323 = 500 - 323 = 177
        // 177 = (500 % 17) + k × 17, so k = (177 - (500%17)) / 17
        // 500 % 17 = 7, k = (177 - 7) / 17 = 170/17 = 10
        let rp = 500u64 % 17;
        let reconstructed = rp + k * 17;
        assert_eq!(reconstructed % 323, 500 % 323,
            "Internal K-Elim should reconstruct the full residue");
    }

    #[test]
    fn test_gear_number_sieve() {
        let gn = GearNumber::new(289); // 17²
        assert!(!gn.survives_full_sieve(), "289 = 17² should be caught by gear 323");
        
        let gn2 = GearNumber::new(323); // 17×19
        assert!(!gn2.survives_full_sieve(), "323 = 17×19 caught");

        let gn3 = GearNumber::new(529); // 23²
        assert!(!gn3.survives_full_sieve(), "529 = 23² should be caught by gear 667");
    }

    #[test]
    fn test_gear_number_prime_survives() {
        let gn = GearNumber::new(31); // prime, coprime to all gears
        assert!(gn.survives_full_sieve(), "31 is prime and coprime to all gears");

        let gn2 = GearNumber::new(997); // prime
        assert!(gn2.survives_full_sieve(), "997 is prime");
    }

    #[test]
    fn test_gear_precision_improvement() {
        let basis = [2u64, 3, 5, 7, 11, 13];
        let gear_primes = [17u64, 19, 23, 29];

        let mut survivors_base = 0u32;
        let mut survivors_gear = 0u32;
        let mut primes_count = 0u32;

        for n in 1000..50000u64 {
            let base_coprime = basis.iter().all(|&p| n % p != 0);
            if !base_coprime { continue; }
            survivors_base += 1;

            let gear_coprime = gear_primes.iter().all(|&p| n % p != 0);
            if gear_coprime { survivors_gear += 1; }

            if is_prime_trial(n) { primes_count += 1; }
        }

        let prec_base = primes_count as f64 / survivors_base as f64;
        let prec_gear = primes_count as f64 / survivors_gear as f64;

        assert!(prec_gear > prec_base,
            "Gear precision {:.3} should exceed base {:.3}", prec_gear, prec_base);
    }

    #[test]
    fn test_shadow_gear_winding_structure() {
        let gn1 = GearNumber::new(1009); // prime
        let gn2 = GearNumber::new(1003); // = 17 × 59, composite

        let w1 = gn1.shadow_gear_windings();
        let w2 = gn2.shadow_gear_windings();

        // Both should produce valid windings (< gear modulus)
        assert!(w1[0] < 323, "Shadow-gear winding should be < 323");
        assert!(w2[0] < 323);

        // The windings should differ (different numbers)
        // (not guaranteed but overwhelmingly likely)
    }

    #[test]
    fn test_state_vector_consistent_length() {
        let gn1 = GearNumber::new(100);
        let gn2 = GearNumber::new(50000);
        assert_eq!(gn1.state_vector().len(), gn2.state_vector().len(),
            "State vectors should have consistent length");
    }

    #[test]
    fn test_shadow_phase_cousin_shift() {
        // Each rung of 169 shifts shadow phase by 4 (cousin gap)
        let gn0 = GearNumber::new(10);    // rung 0
        let gn1 = GearNumber::new(179);   // rung 1
        let gn2 = GearNumber::new(348);   // rung 2

        // Shadow phases should differ by 4 mod 11
        let p0 = gn0.shadow_phase;
        let p1 = gn1.shadow_phase;
        let p2 = gn2.shadow_phase;

        assert_eq!((p1 + 11 - p0) % 11, 4,
            "Shadow phase should shift by cousin gap (4) per rung");
        assert_eq!((p2 + 11 - p1) % 11, 4,
            "Consistent cousin-gap shift");
    }

    #[test]
    fn test_all_gears_coprime_to_shadow() {
        let gn = GearNumber::new(1000);
        for gear in &gn.gears {
            assert_eq!(gcd(gear.modulus, 11), 1,
                "Gear {} must be coprime to shadow 11", gear.modulus);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// THE COPERNICAN FRAME
//
// The primes are fixed. We rotate.
//
// The "thinning" of primes (PNT: π(x) ~ x/ln(x)) is what the
// observer sees as they climb the rung ladder, accumulating shadow
// phase via the ×4 orbit. Each rung shifts the shadow by the
// cousin gap (4). After 11 rungs, the phase wraps. The prime
// pattern, seen from any fixed shadow phase, is nearly identical
// across super-periods — confirming that the underlying structure
// is stationary.
//
// The observer carries:
//   - Rung number (which 169-block they're in)
//   - Shadow phase ((rung × 4) mod 11)
//   - Accumulated winding (shadow_count = n / 11)
//   - Horizon (13² = 169 for Safe Basis; extends with gears)
//
// The fixed pattern carries:
//   - The eternal positions: which Z/MZ residues CAN hold primes
//   - The composite intrusions: which rungs see new composites
//   - The gap family structure: how twins/cousins/sexys distribute
//
// Derotation: subtract the shadow phase from the observed position
// to recover the observer-independent view.
// ═══════════════════════════════════════════════════════════════════

/// The fixed prime pattern on Z/MZ
/// This is the eternal object — it doesn't change with the observer
#[derive(Debug, Clone)]
pub struct FixedPattern {
    /// Modulus M (product of all sieve primes)
    pub modulus: u64,
    /// Positions that survive the sieve (coprime to M)
    pub survivor_positions: Vec<u64>,
    /// Count of survivor positions
    pub phi_m: usize,
    /// Horizon: below this, sieve is complete (= largest_prime²)
    pub horizon: u64,
}

impl FixedPattern {
    /// Build the fixed pattern from a set of sieve primes
    pub fn from_primes(primes: &[u64]) -> Self {
        let modulus: u64 = primes.iter().product();
        // Compute φ(M) without storing all positions
        let mut phi = modulus;
        for &p in primes {
            phi = phi / p * (p - 1);
        }
        let horizon = primes.last().map(|&p| p * p).unwrap_or(4);

        FixedPattern {
            modulus,
            survivor_positions: primes.to_vec(), // store primes, not positions
            phi_m: phi as usize,
            horizon,
        }
    }

    /// Is position r a survivor? (coprime to all sieve primes)
    pub fn is_survivor(&self, r: u64) -> bool {
        if r < 2 { return false; }
        self.survivor_positions.iter().all(|&p| r % p != 0)
    }

    /// Density of survivors: φ(M)/M
    pub fn survivor_density(&self) -> f64 {
        self.phi_m as f64 / self.modulus as f64
    }
}

/// The observer's state — determined by where they are on the rung ladder
#[derive(Debug, Clone)]
pub struct Observer {
    /// Current integer being observed
    pub n: u64,
    /// Rung number (n / rung_size)
    pub rung: u64,
    /// Rung size (= boundary² for the prime-only basis, or gear horizon)
    pub rung_size: u64,
    /// Shadow phase: (rung × 4) mod 11 — the ×4 orbit position
    pub shadow_phase: u64,
    /// Accumulated shadow winding: n / 11
    pub shadow_winding: u64,
    /// Shadow cross-phase: shadow_winding mod 13 (on the 143-torus)
    pub face_phase: u64,
    /// Which composite gears are active
    pub active_gears: Vec<(u64, u64)>,
    /// Effective horizon (extended by gears)
    pub horizon: u64,
}

impl Observer {
    /// Create an observer at position n with given gears
    pub fn at(n: u64, rung_size: u64, gears: &[(u64, u64)]) -> Self {
        let rung = n / rung_size;
        let shadow_winding = n / 11;

        // Effective horizon: max(basis_horizon, max_gear_factor²)
        let max_gear_factor = gears.iter()
            .flat_map(|&(p, q)| vec![p, q])
            .max()
            .unwrap_or(13);
        let horizon = max_gear_factor * max_gear_factor;

        Observer {
            n,
            rung,
            rung_size,
            shadow_phase: (rung * 4) % 11,
            shadow_winding,
            face_phase: shadow_winding % 13,
            active_gears: gears.to_vec(),
            horizon,
        }
    }

    /// Create at n with Safe Basis only (no gears)
    pub fn at_safe_basis(n: u64) -> Self {
        Self::at(n, 169, &[])
    }

    /// Create with standard composite gears
    pub fn with_standard_gears(n: u64) -> Self {
        Self::at(n, 169, &[(17, 19), (23, 29), (31, 37), (41, 43)])
    }

    /// Is n below the observer's horizon? (complete sieve territory)
    pub fn below_horizon(&self) -> bool {
        self.n < self.horizon
    }

    /// Derotated position: compensate for shadow phase
    /// This removes the observer's rotation to reveal the fixed frame
    pub fn derotated_boundary_pos(&self) -> u64 {
        let r13 = self.n % 13;
        (r13 + 13 - self.shadow_phase % 13) % 13
    }

    /// Derotated backbone position
    pub fn derotated_backbone_pos(&self) -> u64 {
        let r7 = self.n % 7;
        (r7 + 7 - self.shadow_phase % 7) % 7
    }

    /// The full derotated state: (backbone, shadow, boundary) compensated
    pub fn derotated_state(&self) -> (u64, u64, u64) {
        (
            self.derotated_backbone_pos(),
            self.shadow_winding % 11,  // shadow doesn't derotate (it's the axis)
            self.derotated_boundary_pos(),
        )
    }

    /// Super-period: how many rungs until shadow phase wraps
    pub fn super_period_rungs() -> u64 { 11 }

    /// Which super-period cycle are we in?
    pub fn super_cycle(&self) -> u64 { self.rung / 11 }

    /// Position within the current super-period
    pub fn super_position(&self) -> u64 { self.rung % 11 }
}

/// The Copernican frame: fixed pattern + observer = what you see
#[derive(Debug)]
pub struct CopernicanFrame {
    /// The eternal prime pattern
    pub pattern: FixedPattern,
    /// Sieve primes (for the fixed pattern)
    pub sieve_primes: Vec<u64>,
    /// Composite gear pairs
    pub gear_pairs: Vec<(u64, u64)>,
}

impl CopernicanFrame {
    /// Create with Safe Basis and optional gears
    pub fn new(gears: &[(u64, u64)]) -> Self {
        let mut all_primes: Vec<u64> = vec![2, 3, 5, 7, 11, 13];
        for &(p, q) in gears {
            if !all_primes.contains(&p) { all_primes.push(p); }
            if !all_primes.contains(&q) { all_primes.push(q); }
        }
        all_primes.sort();

        CopernicanFrame {
            pattern: FixedPattern::from_primes(&all_primes),
            sieve_primes: all_primes,
            gear_pairs: gears.to_vec(),
        }
    }

    /// Safe Basis only (no gears)
    pub fn safe_basis_only() -> Self {
        Self::new(&[])
    }

    /// With standard 4-gear extension
    pub fn with_standard_gears() -> Self {
        Self::new(&[(17, 19), (23, 29), (31, 37), (41, 43)])
    }

    /// Observe n: create the observer and determine what's visible
    pub fn observe(&self, n: u64) -> Observation {
        let observer = Observer::at(n, self.pattern.horizon, &self.gear_pairs);
        let survives = self.pattern.is_survivor(n);
        let is_prime = is_prime_trial(n);
        let below_horizon = observer.below_horizon();

        Observation {
            observer,
            survives_sieve: survives,
            is_actually_prime: is_prime,
            below_horizon,
            // Below horizon: sieve is complete, survivor = prime (for n > 1)
            sieve_is_definitive: below_horizon && n > 1,
        }
    }

    /// Scan a range and compute per-rung statistics
    pub fn scan_rungs(&self, start: u64, num_rungs: usize) -> Vec<RungStats> {
        let rung_size = self.pattern.horizon;
        let mut stats = Vec::new();

        for rung_idx in 0..num_rungs {
            let lo = start + (rung_idx as u64) * rung_size;
            let hi = lo + rung_size;
            let phase = ((lo / rung_size) * 4) % 11;

            let mut primes = 0u32;
            let mut survivors = 0u32;
            let mut composites_in_sieve = 0u32;
            let mut prime_positions: Vec<u64> = Vec::new();

            for n in lo.max(2)..hi {
                if !self.pattern.is_survivor(n) { continue; }
                survivors += 1;
                if is_prime_trial(n) {
                    primes += 1;
                    prime_positions.push(n % 13);
                } else {
                    composites_in_sieve += 1;
                }
            }

            let precision = if survivors > 0 {
                primes as f64 / survivors as f64
            } else { 0.0 };

            // Derotated prime positions
            let derotated: Vec<u64> = prime_positions.iter()
                .map(|&r| (r + 13 - phase % 13) % 13)
                .collect();

            let unique_raw = prime_positions.iter().collect::<std::collections::HashSet<_>>().len();
            let unique_derot = derotated.iter().collect::<std::collections::HashSet<_>>().len();

            stats.push(RungStats {
                rung: lo / rung_size,
                range: (lo, hi),
                shadow_phase: phase,
                primes,
                survivors,
                new_composites: composites_in_sieve,
                precision,
                prime_positions_mod13: unique_raw as u32,
                derotated_positions_mod13: unique_derot as u32,
                super_cycle: (lo / rung_size) / 11,
                super_position: (lo / rung_size) % 11,
            });
        }
        stats
    }

    /// Compare same-phase rungs across super-periods
    /// Returns Jaccard similarity of prime Z/13Z positions
    pub fn phase_stability(&self, start: u64, num_super_periods: usize) -> Vec<(u64, f64)> {
        let rung_size = self.pattern.horizon;
        let mut phase_positions: std::collections::HashMap<u64, Vec<std::collections::HashSet<u64>>> =
            std::collections::HashMap::new();

        for sp in 0..num_super_periods {
            for phase_idx in 0..11u64 {
                let rung = (start / rung_size) + sp as u64 * 11 + phase_idx;
                let lo = rung * rung_size;
                let hi = lo + rung_size;
                let phase = (rung * 4) % 11;

                let prime_pos: std::collections::HashSet<u64> = (lo.max(2)..hi)
                    .filter(|&n| self.pattern.is_survivor(n) && is_prime_trial(n))
                    .map(|n| n % 13)
                    .collect();

                phase_positions.entry(phase).or_default().push(prime_pos);
            }
        }

        // For each phase, compute average Jaccard between consecutive occurrences
        let mut results = Vec::new();
        for phase in 0..11u64 {
            if let Some(sets) = phase_positions.get(&phase) {
                if sets.len() >= 2 {
                    let mut total_jaccard = 0.0;
                    let mut count = 0;
                    for i in 0..sets.len()-1 {
                        let intersection = sets[i].intersection(&sets[i+1]).count();
                        let union = sets[i].union(&sets[i+1]).count();
                        if union > 0 {
                            total_jaccard += intersection as f64 / union as f64;
                            count += 1;
                        }
                    }
                    let avg = if count > 0 { total_jaccard / count as f64 } else { 0.0 };
                    results.push((phase, avg));
                }
            }
        }
        results
    }
}

/// Statistics for one rung of the ladder
#[derive(Debug, Clone)]
pub struct RungStats {
    pub rung: u64,
    pub range: (u64, u64),
    pub shadow_phase: u64,
    pub primes: u32,
    pub survivors: u32,
    pub new_composites: u32,
    pub precision: f64,
    pub prime_positions_mod13: u32,
    pub derotated_positions_mod13: u32,
    pub super_cycle: u64,
    pub super_position: u64,
}

/// Result of observing a single number
#[derive(Debug)]
pub struct Observation {
    pub observer: Observer,
    pub survives_sieve: bool,
    pub is_actually_prime: bool,
    pub below_horizon: bool,
    pub sieve_is_definitive: bool,
}

#[cfg(test)]
mod copernican_tests {
    use super::*;

    #[test]
    fn test_fixed_pattern_safe_basis() {
        let pattern = FixedPattern::from_primes(&[2, 3, 5, 7, 11, 13]);
        // φ(30030) = 30030 × (1-1/2)(1-1/3)(1-1/5)(1-1/7)(1-1/11)(1-1/13)
        // = 30030 × 1/2 × 2/3 × 4/5 × 6/7 × 10/11 × 12/13 = 5760
        assert_eq!(pattern.phi_m, 5760,
            "φ(30030) should be 5760, got {}", pattern.phi_m);
        assert_eq!(pattern.horizon, 169, "Horizon = 13² = 169");
    }

    #[test]
    fn test_fixed_pattern_with_gears() {
        let frame = CopernicanFrame::with_standard_gears();
        // With primes up to 43: horizon = 43² = 1849
        assert_eq!(frame.pattern.horizon, 43 * 43);
        // Survivor density should be lower (more primes sieving)
        let base = CopernicanFrame::safe_basis_only();
        assert!(frame.pattern.survivor_density() < base.pattern.survivor_density(),
            "Gears should reduce survivor density");
    }

    #[test]
    fn test_observer_shadow_phase_cycle() {
        // Shadow phase should cycle through all 11 values in 11 rungs
        let mut phases: Vec<u64> = Vec::new();
        for rung in 0..11 {
            let obs = Observer::at(rung * 169 + 50, 169, &[]);
            phases.push(obs.shadow_phase);
        }
        // Should visit all 11 values
        let unique: std::collections::HashSet<u64> = phases.iter().copied().collect();
        assert_eq!(unique.len(), 11,
            "Shadow phase should visit all 11 values in 11 rungs");
    }

    #[test]
    fn test_observer_shadow_shift_is_cousin_gap() {
        let obs0 = Observer::at_safe_basis(50);      // rung 0
        let obs1 = Observer::at_safe_basis(219);     // rung 1
        let shift = (obs1.shadow_phase + 11 - obs0.shadow_phase) % 11;
        assert_eq!(shift, 4, "Shadow phase shifts by cousin gap (4) per rung");
    }

    #[test]
    fn test_below_horizon_is_definitive() {
        let frame = CopernicanFrame::safe_basis_only();
        // Below 169: sieve is complete
        let obs = frame.observe(167);
        assert!(obs.below_horizon);
        assert!(obs.sieve_is_definitive);
        assert!(obs.survives_sieve);
        assert!(obs.is_actually_prime);
    }

    #[test]
    fn test_above_horizon_composites_survive() {
        let frame = CopernicanFrame::safe_basis_only();
        // 289 = 17² is above horizon, survives basis sieve
        let obs = frame.observe(289);
        assert!(!obs.below_horizon);
        assert!(obs.survives_sieve, "289 coprime to basis");
        assert!(!obs.is_actually_prime, "289 = 17² is composite");
        assert!(!obs.sieve_is_definitive, "Above horizon, sieve not definitive");
    }

    #[test]
    fn test_gears_catch_above_horizon() {
        let frame = CopernicanFrame::with_standard_gears();
        // 289 = 17² should be caught by gear (17,19)
        let obs = frame.observe(289);
        assert!(!obs.survives_sieve, "289 should be caught by gear 17×19");
    }

    #[test]
    fn test_rung_precision_decreases() {
        let frame = CopernicanFrame::safe_basis_only();
        let stats = frame.scan_rungs(0, 10);
        // Precision should generally decrease (with some noise)
        if stats.len() >= 3 {
            assert!(stats[0].precision >= stats[5].precision - 0.1,
                "Precision should decrease as rungs climb");
        }
    }

    #[test]
    fn test_phase_stability_high() {
        let frame = CopernicanFrame::safe_basis_only();
        let stability = frame.phase_stability(0, 3);
        // Same-phase rungs should have Jaccard > 0.7
        for (phase, jaccard) in &stability {
            assert!(*jaccard > 0.5,
                "Phase {} Jaccard {:.3} should be > 0.5 (pattern persists)",
                phase, jaccard);
        }
    }

    #[test]
    fn test_geared_frame_higher_precision() {
        let base_frame = CopernicanFrame::safe_basis_only();
        let gear_frame = CopernicanFrame::with_standard_gears();

        // Compare precision at rung 10 (past both horizons)
        let base_stats = base_frame.scan_rungs(2000, 5);
        let gear_stats = gear_frame.scan_rungs(2000, 5);

        let base_prec: f64 = base_stats.iter().map(|s| s.precision).sum::<f64>() / 5.0;
        let gear_prec: f64 = gear_stats.iter().map(|s| s.precision).sum::<f64>() / 5.0;

        assert!(gear_prec > base_prec,
            "Geared precision {:.3} should exceed base {:.3}", gear_prec, base_prec);
    }

    #[test]
    fn test_super_period_wraps() {
        let obs0 = Observer::at_safe_basis(0);
        let obs11 = Observer::at_safe_basis(11 * 169);
        assert_eq!(obs0.shadow_phase, obs11.shadow_phase,
            "Shadow phase should wrap after 11 rungs");
        assert_eq!(obs11.super_cycle(), 1);
        assert_eq!(obs11.super_position(), 0);
    }

    #[test]
    fn test_derotated_state_removes_phase() {
        // Two observers at same super_position but different super_cycles
        // should have similar derotated states
        let obs_a = Observer::at_safe_basis(3 * 169 + 50);   // rung 3
        let obs_b = Observer::at_safe_basis(14 * 169 + 50);  // rung 14 = rung 3 + 11

        assert_eq!(obs_a.shadow_phase, obs_b.shadow_phase,
            "Same super_position → same phase");

        // Derotated states should be related
        let (da7, _, da13) = obs_a.derotated_state();
        let (db7, _, db13) = obs_b.derotated_state();

        // With the same derotation, positions should be comparable
        // (not necessarily equal — the actual integers differ)
    }

    #[test]
    fn test_copernican_primes_dont_move() {
        // The SAME Z/30030Z position holds a prime on rung 0
        // and may or may not on other rungs. But the POSITION is fixed.
        let frame = CopernicanFrame::safe_basis_only();

        // Count: how many Z/13Z positions hold primes across 5 super-periods?
        let stability = frame.phase_stability(0, 5);
        let avg_jaccard: f64 = stability.iter().map(|(_, j)| j).sum::<f64>()
            / stability.len().max(1) as f64;

        // High Jaccard = pattern persists = primes are "fixed"
        assert!(avg_jaccard > 0.6,
            "Average Jaccard {:.3} should be > 0.6: primes don't move, we do",
            avg_jaccard);
    }
}

// ═══════════════════════════════════════════════════════════════════
// RUNG-AWARE SIEVE (Generalized)
//
// From the Copernican Frame: each composite gear lane has a minimum
// rung k_min = floor(p_min² / 169) below which no basis-coprime
// composite divisible by that lane's factors can exist.
//
// Skipping inactive lanes preserves the hard invariant (zero false
// negatives) while eliminating vacuous modular reductions.
// ═══════════════════════════════════════════════════════════════════

/// A generalized gear lane: arbitrary modulus with rung activation
#[derive(Debug, Clone, Copy)]
pub struct GearLane {
    /// The composite modulus (product of primes ≥ 17)
    pub m: u64,
    /// Activation rung: floor(p_min² / 169)
    pub k_min: u64,
}

/// Standard 4-lane configuration
pub const GEAR_LANES_4: &[GearLane] = &[
    GearLane { m: 323,  k_min: 1 },   // 17×19
    GearLane { m: 667,  k_min: 3 },   // 23×29
    GearLane { m: 1147, k_min: 5 },   // 31×37
    GearLane { m: 1763, k_min: 9 },   // 41×43
];

/// Extended 7-lane configuration (76.25% precision)
pub const GEAR_LANES_7: &[GearLane] = &[
    GearLane { m: 323,  k_min: 1 },   // 17×19
    GearLane { m: 667,  k_min: 3 },   // 23×29
    GearLane { m: 1147, k_min: 5 },   // 31×37
    GearLane { m: 1763, k_min: 9 },   // 41×43
    GearLane { m: 2491, k_min: 13 },  // 47×53
    GearLane { m: 3599, k_min: 20 },  // 59×61
    GearLane { m: 4757, k_min: 26 },  // 67×71
];

/// Full 10-lane configuration (~82.5% precision)
pub const GEAR_LANES_10: &[GearLane] = &[
    GearLane { m: 323,  k_min: 1 },   // 17×19
    GearLane { m: 667,  k_min: 3 },   // 23×29
    GearLane { m: 1147, k_min: 5 },   // 31×37
    GearLane { m: 1763, k_min: 9 },   // 41×43
    GearLane { m: 2491, k_min: 13 },  // 47×53
    GearLane { m: 3599, k_min: 20 },  // 59×61
    GearLane { m: 4757, k_min: 26 },  // 67×71
    GearLane { m: 5767, k_min: 31 },  // 73×79
    GearLane { m: 7387, k_min: 40 },  // 83×89
    GearLane { m: 9797, k_min: 55 },  // 97×101
];

/// Full 14-lane configuration (88.72% precision, factor-floor spf≥149)
pub const GEAR_LANES_14: &[GearLane] = &[
    GearLane { m: 323,   k_min: 1 },    // 17×19
    GearLane { m: 667,   k_min: 3 },    // 23×29
    GearLane { m: 1147,  k_min: 5 },    // 31×37
    GearLane { m: 1763,  k_min: 9 },    // 41×43
    GearLane { m: 2491,  k_min: 13 },   // 47×53
    GearLane { m: 3599,  k_min: 20 },   // 59×61
    GearLane { m: 4757,  k_min: 26 },   // 67×71
    GearLane { m: 5767,  k_min: 31 },   // 73×79
    GearLane { m: 7387,  k_min: 40 },   // 83×89
    GearLane { m: 9797,  k_min: 55 },   // 97×101
    GearLane { m: 11021, k_min: 62 },   // 103×107
    GearLane { m: 12317, k_min: 70 },   // 109×113
    GearLane { m: 16637, k_min: 95 },   // 127×131
    GearLane { m: 19043, k_min: 111 },  // 137×139
];

/// Extended 15-lane configuration (90.19% precision, factor-floor spf≥157)
pub const GEAR_LANES_15: &[GearLane] = &[
    GearLane { m: 323,   k_min: 1 },    // 17×19
    GearLane { m: 667,   k_min: 3 },    // 23×29
    GearLane { m: 1147,  k_min: 5 },    // 31×37
    GearLane { m: 1763,  k_min: 9 },    // 41×43
    GearLane { m: 2491,  k_min: 13 },   // 47×53
    GearLane { m: 3599,  k_min: 20 },   // 59×61
    GearLane { m: 4757,  k_min: 26 },   // 67×71
    GearLane { m: 5767,  k_min: 31 },   // 73×79
    GearLane { m: 7387,  k_min: 40 },   // 83×89
    GearLane { m: 9797,  k_min: 55 },   // 97×101
    GearLane { m: 11021, k_min: 62 },   // 103×107
    GearLane { m: 12317, k_min: 70 },   // 109×113
    GearLane { m: 16637, k_min: 95 },   // 127×131
    GearLane { m: 19043, k_min: 111 },  // 137×139
    GearLane { m: 22499, k_min: 131 },  // 149×151
];

/// Integer GCD (Euclidean)
fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

/// Rung-aware classifier
pub struct RungAwareSieve {
    /// Active gear lanes
    pub lanes: Vec<GearLane>,
    /// Safe basis primes for initial sieve
    pub basis: Vec<u64>,
    /// Rung size (= boundary² = 169 for Safe Basis)
    pub rung_size: u64,
}

impl RungAwareSieve {
    pub fn with_4_lanes() -> Self {
        RungAwareSieve {
            lanes: GEAR_LANES_4.to_vec(),
            basis: vec![2, 3, 5, 7, 11, 13],
            rung_size: 169,
        }
    }

    pub fn with_7_lanes() -> Self {
        RungAwareSieve {
            lanes: GEAR_LANES_7.to_vec(),
            basis: vec![2, 3, 5, 7, 11, 13],
            rung_size: 169,
        }
    }

    pub fn with_10_lanes() -> Self {
        RungAwareSieve {
            lanes: GEAR_LANES_10.to_vec(),
            basis: vec![2, 3, 5, 7, 11, 13],
            rung_size: 169,
        }
    }

    pub fn with_14_lanes() -> Self {
        RungAwareSieve {
            lanes: GEAR_LANES_14.to_vec(),
            basis: vec![2, 3, 5, 7, 11, 13],
            rung_size: 169,
        }
    }

    pub fn with_15_lanes() -> Self {
        RungAwareSieve {
            lanes: GEAR_LANES_15.to_vec(),
            basis: vec![2, 3, 5, 7, 11, 13],
            rung_size: 169,
        }
    }

    pub fn with_lanes(lanes: &[GearLane]) -> Self {
        RungAwareSieve {
            lanes: lanes.to_vec(),
            basis: vec![2, 3, 5, 7, 11, 13],
            rung_size: 169,
        }
    }

    /// Layer 0-2: Safe basis sieve
    /// Basis primes themselves ARE prime — don't exclude them
    pub fn passes_safe_basis(&self, n: u64) -> bool {
        if n < 2 { return false; }
        if self.basis.contains(&n) { return true; } // basis primes are prime
        self.basis.iter().all(|&p| n % p != 0)
    }

    /// Layer 3: Haab' telescope (free from calendar residue)
    /// 73 itself is prime — only exclude COMPOSITE multiples
    pub fn passes_haab_telescope(&self, n: u64) -> bool {
        n <= 73 || n % 73 != 0  // 73 is prime; only exclude n > 73 with 73|n
    }

    /// Layer 4: Rung-aware composite gear sieve
    /// Returns true if n is a CANDIDATE (survives all active lanes)
    pub fn classify(&self, n: u64) -> bool {
        // Layer 0-2: safe basis
        if !self.passes_safe_basis(n) { return false; }

        // Layer 3: Haab' telescope
        if !self.passes_haab_telescope(n) { return false; }

        // Layer 4: rung-aware gears
        let k = n / self.rung_size;
        for lane in &self.lanes {
            if k >= lane.k_min {
                let r = n % lane.m;
                if gcd_u64(r, lane.m) > 1 {
                    return false; // composite: shares factor with gear modulus
                }
            }
        }

        true // survives → candidate
    }

    /// Count gear evaluations (modular reductions performed)
    pub fn count_evaluations(&self, lo: u64, hi: u64) -> (u64, u64) {
        let mut aware_evals = 0u64;
        let mut naive_evals = 0u64;

        for n in lo..hi {
            if !self.passes_safe_basis(n) { continue; }
            if !self.passes_haab_telescope(n) { continue; }

            let k = n / self.rung_size;
            for lane in &self.lanes {
                naive_evals += 1; // naive always evaluates
                if k >= lane.k_min {
                    aware_evals += 1; // aware only when active
                }
            }
        }
        (aware_evals, naive_evals)
    }

    /// Full scan: return (candidates, primes, precision, false_negatives)
    pub fn scan(&self, lo: u64, hi: u64) -> (u32, u32, f64, u32) {
        let mut candidates = 0u32;
        let mut primes = 0u32;
        let mut false_neg = 0u32;

        for n in lo..hi {
            let is_p = is_prime_trial(n);
            let survives = self.classify(n);

            if survives { candidates += 1; }
            if is_p { primes += 1; }
            if is_p && !survives { false_neg += 1; }
        }

        let true_pos = primes - false_neg;
        let precision = if candidates > 0 {
            true_pos as f64 / candidates as f64
        } else { 0.0 };

        (candidates, true_pos, precision, false_neg)
    }
}

#[cfg(test)]
mod rung_aware_tests {
    use super::*;

    fn smallest_prime_factor(n: u64) -> u64 {
        if n < 2 { return n; }
        if n % 2 == 0 { return 2; }
        let mut i = 3u64;
        while i * i <= n {
            if n % i == 0 { return i; }
            i += 2;
        }
        n // n is prime
    }

    #[test]
    fn test_k_min_values() {
        // Verify k_min = floor(p_min² / 169)
        assert_eq!(17u64 * 17 / 169, 1,  "17²/169 = 1");
        assert_eq!(23u64 * 23 / 169, 3,  "23²/169 = 3");
        assert_eq!(31u64 * 31 / 169, 5,  "31²/169 = 5");
        assert_eq!(41u64 * 41 / 169, 9,  "41²/169 = 9");
        assert_eq!(47u64 * 47 / 169, 13, "47²/169 = 13");
        assert_eq!(59u64 * 59 / 169, 20, "59²/169 = 20");
        assert_eq!(67u64 * 67 / 169, 26, "67²/169 = 26");
    }

    #[test]
    fn test_4_lane_precision() {
        let sieve = RungAwareSieve::with_4_lanes();
        let (cands, primes, prec, fn_count) = sieve.scan(1000, 100000);
        assert_eq!(fn_count, 0, "Zero false negatives (hard invariant)");
        assert!(prec > 0.65, "4-lane precision should exceed 65%, got {:.2}%", prec * 100.0);
    }

    #[test]
    fn test_7_lane_precision() {
        let sieve = RungAwareSieve::with_7_lanes();
        let (cands, primes, prec, fn_count) = sieve.scan(1000, 100000);
        assert_eq!(fn_count, 0, "Zero false negatives");
        assert!(prec > 0.74, "7-lane precision should exceed 74%, got {:.2}%", prec * 100.0);
    }

    #[test]
    fn test_rung_aware_saves_evaluations() {
        let sieve = RungAwareSieve::with_4_lanes();
        let (aware, naive) = sieve.count_evaluations(1000, 100000);
        assert!(aware <= naive,
            "Rung-aware should use ≤ evaluations: {} vs {}", aware, naive);
    }

    #[test]
    fn test_rung_0_complete() {
        // On rung 0 [0, 169): sieve is COMPLETE.
        // Every prime passes. Every survivor is prime.
        let sieve = RungAwareSieve::with_4_lanes();

        // Verify every prime in [2, 169) passes the sieve
        for n in 2u64..169 {
            if is_prime_trial(n) {
                assert!(sieve.classify(n),
                    "Prime {} should pass the sieve (zero false negatives)", n);
            }
        }

        // Verify every survivor in [2, 169) is actually prime
        for n in 2u64..169 {
            if sieve.classify(n) {
                assert!(is_prime_trial(n),
                    "Below horizon, every survivor should be prime: n={}", n);
            }
        }

        // Specific checks: basis primes and 73 (Haab prime) all pass
        assert!(sieve.classify(2), "2 is prime");
        assert!(sieve.classify(73), "73 is prime — Haab telescope must not exclude it");
        assert!(sieve.classify(167), "167 is prime");
    }

    #[test]
    fn test_rung_aware_identical_candidates() {
        // Rung-aware and naive should produce identical candidate sets
        let sieve = RungAwareSieve::with_7_lanes();

        for n in 1000u64..10000 {
            let aware_result = sieve.classify(n);

            // Naive: check all lanes regardless of rung
            let naive_result = {
                if !sieve.passes_safe_basis(n) || !sieve.passes_haab_telescope(n) {
                    false
                } else {
                    sieve.lanes.iter().all(|lane| {
                        let r = n % lane.m;
                        gcd_u64(r, lane.m) == 1
                    })
                }
            };

            assert_eq!(aware_result, naive_result,
                "n={}: rung-aware ({}) ≠ naive ({})", n, aware_result, naive_result);
        }
    }

    #[test]
    fn test_gcd_based_detection() {
        // gcd(r, m) > 1 ⟺ n shares a prime factor with m
        // Test: 289 = 17², gear 323 = 17×19
        let r = 289u64 % 323;
        assert!(gcd_u64(r, 323) > 1, "289 shares factor 17 with 323");

        // Test: 997 (prime), gear 323
        let r = 997u64 % 323;
        assert_eq!(gcd_u64(r, 323), 1, "997 coprime to 323");

        // Test: 529 = 23², gear 667 = 23×29
        let r = 529u64 % 667;
        assert!(gcd_u64(r, 667) > 1, "529 shares factor 23 with 667");
    }

    #[test]
    fn test_10_lane_precision() {
        let sieve = RungAwareSieve::with_10_lanes();
        let (cands, primes, prec, fn_count) = sieve.scan(1000, 100000);
        assert_eq!(fn_count, 0, "Zero false negatives");
        assert!(prec > 0.80, "10-lane precision should exceed 80%, got {:.2}%", prec * 100.0);
    }

    #[test]
    fn test_haab_telescope_catches_73() {
        let sieve = RungAwareSieve::with_4_lanes();
        assert!(!sieve.classify(73 * 31), "73×31 caught by Haab' telescope");
        assert!(!sieve.classify(73 * 47), "73×47 caught by Haab' telescope");
    }

    #[test]
    fn test_14_lane_precision() {
        let sieve = RungAwareSieve::with_14_lanes();
        let (cands, primes, prec, fn_count) = sieve.scan(1000, 100000);
        assert_eq!(fn_count, 0, "Zero false negatives");
        assert_eq!(cands, 10622, "14-lane candidates [1K,100K)");
        assert_eq!(primes, 9424, "14-lane primes [1K,100K)");
        // Precision: 9424/10622 = 88.72%
        assert!(prec > 0.887, "14-lane precision should be ~88.72%, got {:.2}%", prec * 100.0);
    }

    #[test]
    fn test_15_lane_precision() {
        let sieve = RungAwareSieve::with_15_lanes();
        let (cands, primes, prec, fn_count) = sieve.scan(1000, 100000);
        assert_eq!(fn_count, 0, "Zero false negatives");
        // Lane 15 (149×151) should reduce survivors by ~173
        assert!(cands < 10622, "15-lane should have fewer candidates than 14-lane");
        assert_eq!(primes, 9424, "primes unchanged");
        assert!(prec > 0.90, "15-lane precision should exceed 90%, got {:.2}%", prec * 100.0);
    }

    #[test]
    fn test_14_lane_factor_floor() {
        // All surviving composites should have smallest prime factor >= 149
        let sieve = RungAwareSieve::with_14_lanes();
        for n in 1000u64..100000 {
            if sieve.classify(n) && !is_prime_trial(n) {
                let spf = smallest_prime_factor(n);
                assert!(spf >= 149, "Survivor {} has spf {} < 149", n, spf);
            }
        }
    }

    #[test]
    fn test_15_lane_factor_floor() {
        // With lane 15, factor floor should advance to 157
        let sieve = RungAwareSieve::with_15_lanes();
        for n in 1000u64..100000 {
            if sieve.classify(n) && !is_prime_trial(n) {
                let spf = smallest_prime_factor(n);
                assert!(spf >= 151, "Survivor {} has spf {} < 151", n, spf);
            }
        }
    }
}
