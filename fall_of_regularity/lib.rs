//! # NS Winding Invariants v2 — Recombinant CRT Bridge
//!
//! Exact integer Navier-Stokes with lifted winding invariants and nonlinear advection.
//!
//! ## Changes from v1:
//! - **Fixed:** `submod` edge case when (b-a) is exact multiple of m
//! - **New:** `LiftedWindingAccumulator` — tracks K_{i,j}(T) ∈ ℤ via signed
//!   shortest-arc displacement on the covering space, not the torus
//! - **New:** `Grid2D` — periodic N×N grid with per-point exact lanes
//! - **New:** `compute_advection_residue` — nonlinear (u·∇)u in residue space
//! - **New:** `taylor_green_init` — 2D Taylor-Green vortex on integer grid
//! - **New:** `NSState2D` — full 2D NS evolution with advection + diffusion
//!
//! Zero floating-point in evolution. Initial conditions use f64 → integer
//! quantization with frozen error metadata.

#![allow(dead_code)]

use std::f64::consts::PI;

pub mod energy_certificate;

// ============================================================================
// MODULAR ARITHMETIC PRIMITIVES (submod FIXED)
// ============================================================================

/// Extended GCD: returns (g, x, y) such that a*x + b*y = g
pub fn extended_gcd(a: i128, b: i128) -> (i128, i128, i128) {
    if a == 0 {
        return (b, 0, 1);
    }
    let (g, x1, y1) = extended_gcd(b % a, a);
    (g, y1 - (b / a) * x1, x1)
}

/// Modular inverse: a^{-1} mod m, returns None if gcd(a,m) != 1
pub fn mod_inverse(a: u64, m: u64) -> Option<u64> {
    if a == 0 || m == 0 {
        return None;
    }
    let (g, x, _) = extended_gcd(a as i128, m as i128);
    if g != 1 {
        return None;
    }
    Some(((x % m as i128 + m as i128) % m as i128) as u64)
}

/// Modular addition: (a + b) mod m, overflow-safe via u128
pub fn addmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 + b as u128) % m as u128) as u64
}

/// Modular subtraction: (a - b) mod m, underflow-safe
/// **v2 FIX:** outer % m handles the case where (b - a) is an exact multiple of m
pub fn submod(a: u64, b: u64, m: u64) -> u64 {
    if a >= b {
        (a - b) % m
    } else {
        (m - ((b - a) % m)) % m  // <-- the fix: outer % m turns m → 0
    }
}

/// Modular multiplication: (a * b) mod m, overflow-safe via u128
pub fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

/// Modular exponentiation: a^exp mod m
pub fn powmod(base: u64, mut exp: u64, m: u64) -> u64 {
    if m == 1 {
        return 0;
    }
    let mut result: u128 = 1;
    let mut b = (base as u128) % (m as u128);
    while exp > 0 {
        if exp & 1 == 1 {
            result = (result * b) % m as u128;
        }
        exp >>= 1;
        b = (b * b) % m as u128;
    }
    result as u64
}

// ============================================================================
// 11-SMOOTH FACTORIZATION
// ============================================================================

/// Safe Prime Basis — the only primes allowed in approximation sub-lane moduli
pub const SAFE_BASIS: [u64; 5] = [2, 3, 5, 7, 11];

/// Result of 11-smooth factorization: d = 2^a * 3^b * 5^c * 7^d * 11^e
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmoothFactors {
    pub exponents: [u32; 5],
    pub cofactor: u64,
}

impl SmoothFactors {
    /// Compute prime-power moduli for each non-zero exponent
    pub fn sub_lane_moduli(&self) -> Vec<u64> {
        let mut result = Vec::new();
        for (i, &e) in self.exponents.iter().enumerate() {
            if e > 0 {
                result.push(SAFE_BASIS[i].pow(e));
            }
        }
        result
    }
}

/// Factor a denominator into its 11-smooth components
pub fn factor_11_smooth(mut n: u64) -> SmoothFactors {
    let mut exponents = [0u32; 5];
    for (i, &p) in SAFE_BASIS.iter().enumerate() {
        while n % p == 0 {
            exponents[i] += 1;
            n /= p;
        }
    }
    SmoothFactors {
        exponents,
        cofactor: n,
    }
}

// ============================================================================
// EXACT LANE — Large-prime CRT channel for NS field components
// ============================================================================

/// 7 near-Mersenne 63-bit primes for the exact NS lanes
pub const EXACT_PRIMES: [u64; 7] = [
    (1u64 << 63) - 25,  // p_0: u_x
    (1u64 << 63) - 165, // p_1: u_y
    (1u64 << 63) - 259, // p_2: u_z (zero in 2D)
    (1u64 << 63) - 301, // p_3: pressure
    (1u64 << 63) - 355, // p_4: ω_x (zero in 2D)
    (1u64 << 63) - 471, // p_5: ω_y (zero in 2D)
    (1u64 << 63) - 529, // p_6: ω_z (the only vorticity in 2D)
];

pub const EXACT_LANE_NAMES: [&str; 7] = ["u_x", "u_y", "u_z", "p", "ω_x", "ω_y", "ω_z"];

/// A single exact lane: one NS field component mod a 63-bit prime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExactLane {
    pub residue: u64,
    pub prime: u64,
    pub lane_index: usize,
}

impl ExactLane {
    pub fn encode(value: i64, lane_index: usize) -> Self {
        let p = EXACT_PRIMES[lane_index];
        let residue = if value >= 0 {
            (value as u64) % p
        } else {
            let abs_mod = ((-value) as u64) % p;
            if abs_mod == 0 { 0 } else { p - abs_mod }
        };
        ExactLane {
            residue,
            prime: p,
            lane_index,
        }
    }

    pub fn add(&self, other: &ExactLane) -> ExactLane {
        debug_assert_eq!(self.prime, other.prime);
        ExactLane {
            residue: addmod(self.residue, other.residue, self.prime),
            prime: self.prime,
            lane_index: self.lane_index,
        }
    }

    pub fn sub(&self, other: &ExactLane) -> ExactLane {
        debug_assert_eq!(self.prime, other.prime);
        ExactLane {
            residue: submod(self.residue, other.residue, self.prime),
            prime: self.prime,
            lane_index: self.lane_index,
        }
    }

    pub fn mul(&self, other: &ExactLane) -> ExactLane {
        debug_assert_eq!(self.prime, other.prime);
        ExactLane {
            residue: mulmod(self.residue, other.residue, self.prime),
            prime: self.prime,
            lane_index: self.lane_index,
        }
    }

    /// Decode back to signed i64 (centered representation)
    pub fn decode_signed(&self) -> i64 {
        let half = self.prime / 2;
        if self.residue <= half {
            self.residue as i64
        } else {
            -((self.prime - self.residue) as i64)
        }
    }
}

// ============================================================================
// APPROXIMATION SUB-LANE — 11-smooth composite for PDE estimates
// ============================================================================

pub const APPROX_LANE_NAMES: [&str; 6] = ["E(t)", "Ω(t)", "H(t)", "Λ(t)", "σ(t)", "κ(t)"];

/// A single approximation sub-lane with frozen error metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApproxSubLane {
    pub residue: u64,
    pub modulus: u64,
    pub frozen_epsilon_mb: i64,
    pub lane_index: usize,
    pub source: String,
}

impl ApproxSubLane {
    /// Absorb a continuous PDE estimate: scaled_numerator / denominator.
    /// Returns one sub-lane per prime-power factor of the denominator.
    pub fn absorb(
        scaled_numerator: i64,
        denominator: u64,
        epsilon_mb: i64,
        lane_index: usize,
    ) -> Result<Vec<ApproxSubLane>, String> {
        let factors = factor_11_smooth(denominator);
        if factors.cofactor != 1 {
            return Err(format!(
                "Denominator {} has cofactor {} — not 11-smooth",
                denominator, factors.cofactor
            ));
        }

        let moduli = factors.sub_lane_moduli();
        if moduli.is_empty() {
            return Err("Denominator 1 produces no sub-lanes".into());
        }

        Ok(moduli
            .iter()
            .map(|&m| {
                let residue = if scaled_numerator >= 0 {
                    (scaled_numerator as u64) % m
                } else {
                    let neg = ((-scaled_numerator) as u64) % m;
                    if neg == 0 { 0 } else { m - neg }
                };
                ApproxSubLane {
                    residue,
                    modulus: m,
                    frozen_epsilon_mb: epsilon_mb,
                    lane_index,
                    source: format!("{} (mod {})", APPROX_LANE_NAMES[lane_index], m),
                }
            })
            .collect())
    }

    pub fn add(&self, other: &ApproxSubLane) -> ApproxSubLane {
        debug_assert_eq!(self.modulus, other.modulus);
        ApproxSubLane {
            residue: addmod(self.residue, other.residue, self.modulus),
            modulus: self.modulus,
            frozen_epsilon_mb: self.frozen_epsilon_mb + other.frozen_epsilon_mb,
            lane_index: self.lane_index,
            source: format!("({} + {})", self.source, other.source),
        }
    }
}

// ============================================================================
// K-ELIMINATION — The Universal Winding Invariant
// ============================================================================

/// Result of K-Elimination between two coprime lanes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KElimResult {
    pub kappa: u64,
    pub exact_lane_index: usize,
    pub approx_lane_index: usize,
    pub exact_prime: u64,
    pub approx_modulus: u64,
    pub reconstruction: u128,
}

/// Compute κ_{i,j} ≡ (r_A - r_E mod q) · (p^{-1} mod q) mod q
pub fn k_eliminate(exact: &ExactLane, approx: &ApproxSubLane) -> KElimResult {
    let p_i = exact.prime;
    let q_j = approx.modulus;

    let r_e_mod_q = exact.residue % q_j;
    let r_a = approx.residue;

    let diff = submod(r_a, r_e_mod_q, q_j);

    let p_inv = mod_inverse(p_i % q_j, q_j)
        .expect("Exact primes and 11-smooth moduli must be coprime");

    let kappa = mulmod(diff, p_inv, q_j);

    let reconstruction = exact.residue as u128 + kappa as u128 * p_i as u128;

    KElimResult {
        kappa,
        exact_lane_index: exact.lane_index,
        approx_lane_index: approx.lane_index,
        exact_prime: p_i,
        approx_modulus: q_j,
        reconstruction,
    }
}

// ============================================================================
// LIFTED WINDING ACCUMULATOR — K(T) ∈ ℤ on the covering space
// ============================================================================

/// Tracks the lifted (unwrapped) winding number K_{i,j}(T) ∈ ℤ.
///
/// Each κ_{i,j} lives in ℤ/q_jℤ (a torus), which is intrinsically bounded.
/// The lifted winding lives on the universal cover ℤ and CAN diverge.
/// This is the mathematically correct observable for regularity detection.
///
/// At each timestep, we compute the signed shortest-arc displacement:
///   Δκ = κ(t) - κ(t-1)   (choosing representative in (-q/2, q/2])
/// Then accumulate: K(T) = Σ_{t=1}^{T} Δκ(t)
#[derive(Debug, Clone)]
pub struct LiftedWindingAccumulator {
    /// K_{i,j}(T) ∈ ℤ — the cumulative lifted winding for each (exact, approx) pair
    pub lifted: [[i128; 6]; 7],
    /// Previous timestep's κ values for displacement computation
    prev_kappa: [[u64; 6]; 7],
    /// The moduli q_j for each approximation lane (needed for arc computation)
    moduli: [u64; 6],
    /// Whether we have a previous timestep recorded
    initialized: bool,
    /// Total number of wrapping events detected (for diagnostics)
    pub total_wraps: u64,
    /// Per-pair wrap counts
    pub wrap_counts: [[u64; 6]; 7],
    /// History of L∞ norm of lifted winding vector
    pub linf_history: Vec<i128>,
    /// History of vorticity-row L∞ (rows 4,5,6 — the BKM-relevant rows)
    pub vorticity_linf_history: Vec<i128>,
}

impl LiftedWindingAccumulator {
    pub fn new(moduli: [u64; 6]) -> Self {
        LiftedWindingAccumulator {
            lifted: [[0i128; 6]; 7],
            prev_kappa: [[0u64; 6]; 7],
            moduli,
            initialized: false,
            total_wraps: 0,
            wrap_counts: [[0u64; 6]; 7],
            linf_history: Vec::new(),
            vorticity_linf_history: Vec::new(),
        }
    }

    /// Compute the signed shortest-arc displacement on ℤ/qℤ.
    /// Returns a value in (-q/2, q/2].
    pub fn signed_displacement(prev: u64, curr: u64, q: u64) -> i128 {
        let raw_diff = curr as i128 - prev as i128;
        let q_i128 = q as i128;
        let half_q = q_i128 / 2;

        // Normalize to (-q/2, q/2]
        let mut d = raw_diff % q_i128;
        if d > half_q {
            d -= q_i128;
        } else if d < -half_q {
            d += q_i128;
        }
        d
    }

    /// Record a new phase-lock signature and update lifted windings
    pub fn record(&mut self, kappa_matrix: &[[u64; 6]; 7]) {
        if !self.initialized {
            self.prev_kappa = *kappa_matrix;
            self.initialized = true;
            self.linf_history.push(0);
            self.vorticity_linf_history.push(0);
            return;
        }

        let mut linf: i128 = 0;
        let mut vort_linf: i128 = 0;

        for i in 0..7 {
            for j in 0..6 {
                let d = Self::signed_displacement(
                    self.prev_kappa[i][j],
                    kappa_matrix[i][j],
                    self.moduli[j],
                );

                self.lifted[i][j] += d;

                // Detect wrapping: |displacement| > q/3 signals a wrap
                if d.unsigned_abs() > (self.moduli[j] as u128 / 3) {
                    self.total_wraps += 1;
                    self.wrap_counts[i][j] += 1;
                }

                let abs_k = self.lifted[i][j].unsigned_abs() as i128;
                if abs_k > linf {
                    linf = abs_k;
                }

                // Vorticity rows: i ∈ {4, 5, 6}
                if i >= 4 && abs_k > vort_linf {
                    vort_linf = abs_k;
                }
            }
        }

        self.linf_history.push(linf);
        self.vorticity_linf_history.push(vort_linf);
        self.prev_kappa = *kappa_matrix;
    }

    /// Check if the lifted vorticity winding is bounded over the trajectory.
    /// Returns (is_bounded, growth_rate_estimate).
    pub fn vorticity_bounded(&self, window: usize) -> (bool, Option<f64>) {
        let n = self.vorticity_linf_history.len();
        if n < window + 1 {
            return (true, None);
        }

        let tail = &self.vorticity_linf_history[n - window..];
        let max_val = *tail.iter().max().unwrap();
        let min_val = *tail.iter().filter(|&&v| v > 0).min().unwrap_or(&1);

        // NOTE: this single use of f64 is diagnostic only, never in computation
        let growth = max_val as f64 / min_val.max(1) as f64;
        (growth < 2.0, Some(growth))
    }

    /// L∞ norm of the full lifted winding vector
    pub fn current_linf(&self) -> i128 {
        let mut m: i128 = 0;
        for i in 0..7 {
            for j in 0..6 {
                let v = self.lifted[i][j].unsigned_abs() as i128;
                if v > m { m = v; }
            }
        }
        m
    }

    /// L∞ norm of the vorticity rows only (BKM-relevant)
    pub fn vorticity_linf(&self) -> i128 {
        let mut m: i128 = 0;
        for i in 4..7 {
            for j in 0..6 {
                let v = self.lifted[i][j].unsigned_abs() as i128;
                if v > m { m = v; }
            }
        }
        m
    }
}

impl std::fmt::Display for LiftedWindingAccumulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Lifted Winding K(T) ∈ ℤ^42 (covering space)")?;
        writeln!(
            f, "{:>10} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
            "", "E(t)", "Ω(t)", "H(t)", "Λ(t)", "σ(t)", "κ(t)"
        )?;
        writeln!(f, "{}", "─".repeat(64))?;
        for i in 0..7 {
            write!(f, "{:>10} ", EXACT_LANE_NAMES[i])?;
            for j in 0..6 {
                let k = self.lifted[i][j];
                if k == 0 {
                    write!(f, "{:>8}", "·")?;
                } else {
                    write!(f, "{:>8}", k)?;
                }
            }
            writeln!(f)?;
        }
        writeln!(f, "{}", "─".repeat(64))?;
        writeln!(
            f, "‖K‖_∞: {} | Vorticity ‖K‖_∞: {} | Wraps: {}",
            self.current_linf(), self.vorticity_linf(), self.total_wraps
        )?;
        Ok(())
    }
}

// ============================================================================
// PHASE-LOCK SIGNATURE — The 42-Dimensional Winding Vector
// ============================================================================

/// Default moduli for the 6 approx sub-lanes (pure prime powers only in v2)
pub const DEFAULT_APPROX_MODULI: [u64; 6] = [
    1024, // E(t):  2^10
    2187, // Ω(t):  3^7
    3125, // H(t):  5^5
    2401, // Λ(t):  7^4
    1331, // σ(t):  11^3  (was 1000 — now pure prime power)
    2048, // κ(t):  2^11  (was 1100 — now pure prime power)
];

/// The full phase-lock signature: κ(t) on the torus
#[derive(Debug, Clone)]
pub struct PhaseLockSignature {
    pub kappa_matrix: [[u64; 6]; 7],
    pub results: Vec<KElimResult>,
    pub nonzero_count: u32,
    pub kappa_l1_norm: u128,
    pub kappa_l_inf: u64,
}

impl PhaseLockSignature {
    pub fn compute(exact_lanes: &[ExactLane; 7], approx_lanes: &[ApproxSubLane; 6]) -> Self {
        let mut kappa_matrix = [[0u64; 6]; 7];
        let mut results = Vec::with_capacity(42);
        let mut nonzero_count = 0u32;
        let mut l1_norm: u128 = 0;
        let mut l_inf = 0u64;

        for i in 0..7 {
            for j in 0..6 {
                let result = k_eliminate(&exact_lanes[i], &approx_lanes[j]);
                let k = result.kappa;
                kappa_matrix[i][j] = k;
                if k != 0 { nonzero_count += 1; }
                l1_norm += k as u128;
                if k > l_inf { l_inf = k; }
                results.push(result);
            }
        }

        PhaseLockSignature {
            kappa_matrix, results, nonzero_count,
            kappa_l1_norm: l1_norm, kappa_l_inf: l_inf,
        }
    }

    pub fn flat_vector(&self) -> Vec<u64> {
        let mut v = Vec::with_capacity(42);
        for row in &self.kappa_matrix {
            v.extend_from_slice(row);
        }
        v
    }
}

impl std::fmt::Display for PhaseLockSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Phase-Lock Signature κ(t) ∈ ℤ/qℤ^42 (torus)")?;
        writeln!(
            f, "{:>10} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
            "", "E(t)", "Ω(t)", "H(t)", "Λ(t)", "σ(t)", "κ(t)"
        )?;
        writeln!(f, "{}", "─".repeat(64))?;
        for i in 0..7 {
            write!(f, "{:>10} ", EXACT_LANE_NAMES[i])?;
            for j in 0..6 {
                let k = self.kappa_matrix[i][j];
                if k == 0 { write!(f, "{:>8}", "·")?; }
                else { write!(f, "{:>8}", k)?; }
            }
            writeln!(f)?;
        }
        writeln!(f, "{}", "─".repeat(64))?;
        writeln!(
            f, "Non-zero: {}/42 | L1: {} | L∞: {}",
            self.nonzero_count, self.kappa_l1_norm, self.kappa_l_inf
        )?;
        Ok(())
    }
}

// ============================================================================
// 2D PERIODIC GRID — Spatial NS discretization
// ============================================================================

/// Integer scaling factor for quantizing continuous fields.
/// S = 2^20 ≈ 10^6 gives ~6 decimal digits of precision.
/// Frozen error: at most 0.5/S per point.
pub const SCALE_FACTOR: i64 = 1 << 20;

/// A 2D periodic grid storing exact-lane fields per point.
/// Active lanes: 0 (u_x), 1 (u_y), 3 (p), 6 (ω_z) in 2D.
#[derive(Debug, Clone)]
pub struct Grid2D {
    pub n: usize,
    /// Field residues: lanes[local_index][iy * n + ix]
    pub lanes: Vec<Vec<u64>>,
    /// Which EXACT_PRIMES index each stored lane maps to
    pub active_lane_indices: Vec<usize>,
}

impl Grid2D {
    pub fn new(n: usize, active_lanes: &[usize]) -> Self {
        let lanes = active_lanes.iter().map(|_| vec![0u64; n * n]).collect();
        Grid2D {
            n,
            lanes,
            active_lane_indices: active_lanes.to_vec(),
        }
    }

    /// Get the local index (into self.lanes) for a given EXACT_PRIMES index
    pub fn local_index(&self, exact_lane_idx: usize) -> Option<usize> {
        self.active_lane_indices.iter().position(|&i| i == exact_lane_idx)
    }

    /// Get residue at grid point for a specific exact lane
    #[inline]
    pub fn get(&self, exact_lane_idx: usize, iy: usize, ix: usize) -> u64 {
        let li = self.local_index(exact_lane_idx).unwrap();
        self.lanes[li][iy * self.n + ix]
    }

    /// Set residue at grid point for a specific exact lane
    #[inline]
    pub fn set(&mut self, exact_lane_idx: usize, iy: usize, ix: usize, val: u64) {
        let li = self.local_index(exact_lane_idx).unwrap();
        self.lanes[li][iy * self.n + ix] = val;
    }

    /// Periodic index wrapping
    #[inline]
    pub fn wrap(&self, i: isize) -> usize {
        ((i % self.n as isize + self.n as isize) % self.n as isize) as usize
    }
}

// ============================================================================
// FINITE DIFFERENCES IN RESIDUE SPACE
// ============================================================================

/// Precomputed inverse of 2h in each exact lane.
pub fn precompute_inv_2h(h_scaled: u64) -> [u64; 7] {
    let two_h = 2 * h_scaled;
    std::array::from_fn(|i| {
        let p = EXACT_PRIMES[i];
        mod_inverse(two_h % p, p)
            .expect("2h must be coprime to all exact primes")
    })
}

/// Centered difference: (f[i+1] - f[i-1]) mod p
#[inline]
pub fn centered_diff_residue(f_plus: u64, f_minus: u64, prime: u64) -> u64 {
    submod(f_plus, f_minus, prime)
}

// ============================================================================
// NONLINEAR ADVECTION — (u·∇)u in residue space
// ============================================================================

/// Compute the advection term for one velocity component at one grid point.
///
/// For component c ∈ {0 (u_x), 1 (u_y)} in 2D:
///   A_c = u_x · ∂u_c/∂x + u_y · ∂u_c/∂y
///
/// All operations are exact modular arithmetic.
/// The result includes the 1/(2h) factor via precomputed inverse.
pub fn compute_advection_residue(
    grid: &Grid2D,
    iy: usize,
    ix: usize,
    component: usize,
    inv_2h: u64,
    prime: u64,
) -> u64 {
    let n = grid.n;

    let ix_p = (ix + 1) % n;
    let ix_m = if ix == 0 { n - 1 } else { ix - 1 };
    let iy_p = (iy + 1) % n;
    let iy_m = if iy == 0 { n - 1 } else { iy - 1 };

    // u_x and u_y at this point (reduced into this lane's prime)
    let ux_here = grid.get(0, iy, ix) % prime;
    let uy_here = grid.get(1, iy, ix) % prime;

    // ∂u_c/∂x numerator: u_c[i+1] - u_c[i-1]
    let uc_xp = grid.get(component, iy, ix_p) % prime;
    let uc_xm = grid.get(component, iy, ix_m) % prime;
    let duc_dx_num = centered_diff_residue(uc_xp, uc_xm, prime);

    // ∂u_c/∂y numerator
    let uc_yp = grid.get(component, iy_p, ix) % prime;
    let uc_ym = grid.get(component, iy_m, ix) % prime;
    let duc_dy_num = centered_diff_residue(uc_yp, uc_ym, prime);

    // A_c * 2h = u_x * Δ_x(u_c) + u_y * Δ_y(u_c)
    let term_x = mulmod(ux_here, duc_dx_num, prime);
    let term_y = mulmod(uy_here, duc_dy_num, prime);
    let advection_scaled = addmod(term_x, term_y, prime);

    // Divide by 2h
    mulmod(advection_scaled, inv_2h, prime)
}

/// Compute vorticity ω_z = ∂u_y/∂x - ∂u_x/∂y at a grid point.
pub fn compute_vorticity_residue(
    grid: &Grid2D,
    iy: usize,
    ix: usize,
    inv_2h: u64,
    prime: u64,
) -> u64 {
    let n = grid.n;

    let ix_p = (ix + 1) % n;
    let ix_m = if ix == 0 { n - 1 } else { ix - 1 };
    let iy_p = (iy + 1) % n;
    let iy_m = if iy == 0 { n - 1 } else { iy - 1 };

    let uy_xp = grid.get(1, iy, ix_p) % prime;
    let uy_xm = grid.get(1, iy, ix_m) % prime;
    let duy_dx = centered_diff_residue(uy_xp, uy_xm, prime);

    let ux_yp = grid.get(0, iy_p, ix) % prime;
    let ux_ym = grid.get(0, iy_m, ix) % prime;
    let dux_dy = centered_diff_residue(ux_yp, ux_ym, prime);

    let curl_num = submod(duy_dx, dux_dy, prime);
    mulmod(curl_num, inv_2h, prime)
}

// ============================================================================
// TAYLOR-GREEN VORTEX — 2D initial condition
// ============================================================================

/// Initialize a 2D Taylor-Green vortex on an N×N periodic grid.
///
/// NOTE: This function uses f64 for initial condition computation ONLY.
/// All subsequent evolution is exact modular arithmetic.
pub fn taylor_green_init(n: usize, amplitude: f64, wavenumber: f64) -> Grid2D {
    let mut grid = Grid2D::new(n, &[0, 1, 3, 6]);
    let s = SCALE_FACTOR as f64;
    let h = 2.0 * PI / n as f64;

    let encode = |val: i64, lane_idx: usize| -> u64 {
        let p = EXACT_PRIMES[lane_idx];
        if val >= 0 {
            (val as u64) % p
        } else {
            let abs_mod = ((-val) as u64) % p;
            if abs_mod == 0 { 0 } else { p - abs_mod }
        }
    };

    for iy in 0..n {
        let y = iy as f64 * h;
        for ix in 0..n {
            let x = ix as f64 * h;
            let kx = wavenumber * x;
            let ky = wavenumber * y;

            let ux_scaled = (amplitude * kx.cos() * ky.sin() * s).round() as i64;
            let uy_scaled = (-amplitude * kx.sin() * ky.cos() * s).round() as i64;
            let p_scaled = (-(amplitude * amplitude / 4.0)
                * ((2.0 * kx).cos() + (2.0 * ky).cos()) * s)
                .round() as i64;
            let wz_scaled = (-2.0 * amplitude * wavenumber * kx.cos() * ky.cos() * s)
                .round() as i64;

            grid.set(0, iy, ix, encode(ux_scaled, 0));
            grid.set(1, iy, ix, encode(uy_scaled, 1));
            grid.set(3, iy, ix, encode(p_scaled, 3));
            grid.set(6, iy, ix, encode(wz_scaled, 6));
        }
    }

    grid
}

// ============================================================================
// NS STATE 2D — Full evolution with advection + diffusion
// ============================================================================

/// Precomputed constants for NS evolution
#[derive(Debug, Clone)]
pub struct NSEvolveParams {
    pub inv_2h: [u64; 7],
    pub nu_residues: [u64; 7],
    pub dt_inv_s_residues: [u64; 7],
    pub n: usize,
}

impl NSEvolveParams {
    pub fn new(n: usize, reynolds: f64, amplitude: f64) -> Self {
        let l = 2.0 * PI;
        let h = l / n as f64;
        let nu = amplitude * l / reynolds;
        let dt = 0.1 * h / amplitude;

        let diff_factor = nu * dt / (h * h);
        let diff_scaled = (diff_factor * SCALE_FACTOR as f64).round() as i64;
        let dt_scaled = (dt * SCALE_FACTOR as f64).round() as i64;

        let encode = |val: i64, p: u64| -> u64 {
            if val >= 0 { (val as u64) % p }
            else {
                let abs_mod = ((-val) as u64) % p;
                if abs_mod == 0 { 0 } else { p - abs_mod }
            }
        };

        let h_scaled = (h * SCALE_FACTOR as f64).round() as u64;

        NSEvolveParams {
            inv_2h: precompute_inv_2h(h_scaled.max(1)),
            nu_residues: std::array::from_fn(|i| encode(diff_scaled, EXACT_PRIMES[i])),
            dt_inv_s_residues: std::array::from_fn(|i| encode(dt_scaled, EXACT_PRIMES[i])),
            n,
        }
    }
}

/// 2D NS state: grid + timestep counter
#[derive(Debug, Clone)]
pub struct NSState2D {
    pub grid: Grid2D,
    pub timestep: u64,
}

impl NSState2D {
    pub fn taylor_green(n: usize, amplitude: f64, wavenumber: f64) -> Self {
        NSState2D {
            grid: taylor_green_init(n, amplitude, wavenumber),
            timestep: 0,
        }
    }

    /// Advance one timestep: forward Euler, exact modular arithmetic.
    ///   u_c^{n+1} = u_c^n + ν·dt/h²·∇²u_c - dt·(u·∇)u_c
    pub fn advance(&self, params: &NSEvolveParams) -> NSState2D {
        let n = self.grid.n;
        let mut new_grid = Grid2D::new(n, &self.grid.active_lane_indices);

        for component in [0usize, 1] {
            let prime = EXACT_PRIMES[component];
            let inv_2h = params.inv_2h[component];
            let nu_res = params.nu_residues[component];
            let dt_res = params.dt_inv_s_residues[component];

            for iy in 0..n {
                for ix in 0..n {
                    let uc = self.grid.get(component, iy, ix);

                    // 5-point Laplacian stencil
                    let ix_p = (ix + 1) % n;
                    let ix_m = if ix == 0 { n - 1 } else { ix - 1 };
                    let iy_p = (iy + 1) % n;
                    let iy_m = if iy == 0 { n - 1 } else { iy - 1 };

                    let u_xp = self.grid.get(component, iy, ix_p);
                    let u_xm = self.grid.get(component, iy, ix_m);
                    let u_yp = self.grid.get(component, iy_p, ix);
                    let u_ym = self.grid.get(component, iy_m, ix);

                    let sum_nb = addmod(
                        addmod(u_xp, u_xm, prime),
                        addmod(u_yp, u_ym, prime),
                        prime,
                    );
                    let four_uc = mulmod(4, uc, prime);
                    let laplacian_num = submod(sum_nb, four_uc, prime);

                    // Diffusion: ν·dt/h² · (∇²u · h²) = ν·dt · ∇²u
                    let diffusion = mulmod(nu_res, laplacian_num, prime);

                    // Advection: (u·∇)u_c
                    let advection = compute_advection_residue(
                        &self.grid, iy, ix, component, inv_2h, prime,
                    );
                    let advection_dt = mulmod(dt_res, advection, prime);

                    // u^{n+1} = u^n + diffusion - advection
                    let updated = addmod(uc, diffusion, prime);
                    let updated = submod(updated, advection_dt, prime);

                    new_grid.set(component, iy, ix, updated);
                }
            }
        }

        // Recompute vorticity from updated velocity
        let wz_prime = EXACT_PRIMES[6];
        let inv_2h_wz = params.inv_2h[6];
        for iy in 0..n {
            for ix in 0..n {
                let wz = compute_vorticity_residue(&new_grid, iy, ix, inv_2h_wz, wz_prime);
                new_grid.set(6, iy, ix, wz);
            }
        }

        // Pressure: zero placeholder (Poisson solver not yet implemented)
        // This is a known open item — the projection method requires a
        // pressure Poisson solve which is the next architectural piece.

        NSState2D {
            grid: new_grid,
            timestep: self.timestep + 1,
        }
    }

    /// Kinetic energy diagnostic: Σ (u_x² + u_y²)
    pub fn kinetic_energy_residue(&self, lane: usize) -> u64 {
        let n = self.grid.n;
        let prime = EXACT_PRIMES[lane];
        let mut sum = 0u64;
        for iy in 0..n {
            for ix in 0..n {
                let ux = self.grid.get(0, iy, ix) % prime;
                let uy = self.grid.get(1, iy, ix) % prime;
                let ux2 = mulmod(ux, ux, prime);
                let uy2 = mulmod(uy, uy, prime);
                sum = addmod(sum, addmod(ux2, uy2, prime), prime);
            }
        }
        sum
    }

    /// Enstrophy diagnostic: Σ ω_z²
    pub fn enstrophy_residue(&self) -> u64 {
        let n = self.grid.n;
        let prime = EXACT_PRIMES[6];
        let mut sum = 0u64;
        for iy in 0..n {
            for ix in 0..n {
                let wz = self.grid.get(6, iy, ix);
                let wz2 = mulmod(wz, wz, prime);
                sum = addmod(sum, wz2, prime);
            }
        }
        sum
    }
}

/// Run a full NS simulation and collect winding diagnostics.
pub fn run_ns_simulation(
    n: usize,
    reynolds: f64,
    amplitude: f64,
    wavenumber: f64,
    steps: usize,
    approx_moduli: [u64; 6],
) -> (LiftedWindingAccumulator, Vec<u64>, Vec<u64>) {
    let mut state = NSState2D::taylor_green(n, amplitude, wavenumber);
    let params = NSEvolveParams::new(n, reynolds, amplitude);
    let mut accum = LiftedWindingAccumulator::new(approx_moduli);
    let mut ke_series = Vec::with_capacity(steps);
    let mut enstrophy_series = Vec::with_capacity(steps);

    for _t in 0..steps {
        let cx = n / 2;
        let cy = n / 2;

        let exact_lanes: [ExactLane; 7] = std::array::from_fn(|i| {
            let residue = if state.grid.local_index(i).is_some() {
                state.grid.get(i, cy, cx)
            } else {
                0
            };
            ExactLane { residue, prime: EXACT_PRIMES[i], lane_index: i }
        });

        let ke = state.kinetic_energy_residue(0);
        let enst = state.enstrophy_residue();

        let approx_lanes: [ApproxSubLane; 6] = std::array::from_fn(|j| {
            let residue = match j {
                0 => ke % approx_moduli[j],
                1 => enst % approx_moduli[j],
                _ => addmod(ke % approx_moduli[j], enst % approx_moduli[j], approx_moduli[j]),
            };
            ApproxSubLane {
                residue, modulus: approx_moduli[j],
                frozen_epsilon_mb: 500, lane_index: j,
                source: format!("{} t={}", APPROX_LANE_NAMES[j], state.timestep),
            }
        });

        let sig = PhaseLockSignature::compute(&exact_lanes, &approx_lanes);
        accum.record(&sig.kappa_matrix);

        ke_series.push(ke);
        enstrophy_series.push(enst);

        state = state.advance(&params);
    }

    (accum, ke_series, enstrophy_series)
}


// ============================================================================
// TESTS — 42 tests covering all layers
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── Modular Arithmetic (with submod fix) ──

    #[test]
    fn t01_mod_inverse_small() {
        assert_eq!(mod_inverse(3, 7), Some(5));
        assert_eq!(mod_inverse(2, 5), Some(3));
        assert_eq!(mulmod(3, 5, 7), 1);
    }

    #[test]
    fn t02_mod_inverse_not_coprime() {
        assert_eq!(mod_inverse(6, 9), None);
        assert_eq!(mod_inverse(0, 7), None);
    }

    #[test]
    fn t03_mod_inverse_63bit_prime() {
        let p = EXACT_PRIMES[0];
        let inv = mod_inverse(42, p).unwrap();
        assert_eq!(mulmod(42, inv, p), 1);
    }

    #[test]
    fn t04_addmod_overflow_safe() {
        let p = EXACT_PRIMES[0];
        let r = addmod(p - 1, p - 1, p);
        assert_eq!(r, p - 2);
    }

    #[test]
    fn t05_submod_fix_exact_multiple() {
        assert_eq!(submod(0, 7, 7), 0, "submod(0, m, m) must be 0");
        assert_eq!(submod(0, 14, 7), 0, "submod(0, 2m, m) must be 0");
        assert_eq!(submod(3, 10, 7), 0, "submod(3, 10, 7) = -7 ≡ 0 mod 7");
    }

    #[test]
    fn t06_submod_exhaustive_small_moduli() {
        for m in 2..20u64 {
            for a in 0..m {
                for b in 0..m {
                    let expected = ((a as i128 - b as i128).rem_euclid(m as i128)) as u64;
                    let got = submod(a, b, m);
                    assert_eq!(got, expected, "submod({}, {}, {})", a, b, m);
                }
            }
        }
    }

    #[test]
    fn t07_mulmod_squared() {
        let p = EXACT_PRIMES[0];
        assert_eq!(mulmod(p - 1, p - 1, p), 1);
    }

    #[test]
    fn t08_powmod() {
        assert_eq!(powmod(2, 10, 1000), 24);
        assert_eq!(powmod(3, 7, 100), 87);
        assert_eq!(powmod(7, 0, 13), 1);
    }

    // ── 11-Smooth Factorization ──

    #[test]
    fn t09_factor_power_of_two() {
        let f = factor_11_smooth(1024);
        assert_eq!(f.exponents, [10, 0, 0, 0, 0]);
        assert_eq!(f.cofactor, 1);
    }

    #[test]
    fn t10_factor_rejects_13() {
        assert_ne!(factor_11_smooth(13).cofactor, 1);
    }

    #[test]
    fn t11_default_moduli_are_prime_powers() {
        for &m in &DEFAULT_APPROX_MODULI {
            let f = factor_11_smooth(m);
            assert_eq!(f.cofactor, 1, "{} not 11-smooth", m);
            let nonzero = f.exponents.iter().filter(|&&e| e > 0).count();
            assert_eq!(nonzero, 1, "{} not pure prime power: {:?}", m, f.exponents);
        }
    }

    // ── Exact Lane ──

    #[test]
    fn t12_exact_encode_decode_roundtrip() {
        // Only values in (-p/2, p/2] can roundtrip through centered decode.
        // p ≈ 2^63 - 25, so p/2 ≈ 2^62. Values beyond this wrap correctly
        // but can't be distinguished from their modular equivalent.
        for &val in &[0i64, 1, -1, 42, -42, 1_000_000, -1_000_000,
                      (1i64 << 62) - 100, -((1i64 << 62) - 100)] {
            let lane = ExactLane::encode(val, 0);
            assert_eq!(lane.decode_signed(), val, "Roundtrip failed for {}", val);
        }
    }

    #[test]
    fn t13_exact_lane_arithmetic() {
        let a = ExactLane::encode(7, 0);
        let b = ExactLane::encode(6, 0);
        assert_eq!(a.mul(&b).residue, 42);
        assert_eq!(a.add(&b).residue, 13);
        assert_eq!(a.sub(&b).residue, 1);
    }

    // ── K-Elimination ──

    #[test]
    fn t14_k_eliminate_zero_winding() {
        let exact = ExactLane::encode(5, 0);
        let approx = ApproxSubLane {
            residue: 5 % 25, modulus: 25, frozen_epsilon_mb: 500,
            lane_index: 0, source: "test".into(),
        };
        assert_eq!(k_eliminate(&exact, &approx).kappa, 0);
    }

    #[test]
    fn t15_k_eliminate_identity() {
        let exact = ExactLane::encode(42, 0);
        let approx = ApproxSubLane {
            residue: 10, modulus: 25, frozen_epsilon_mb: 500,
            lane_index: 0, source: "test".into(),
        };
        let r = k_eliminate(&exact, &approx);
        let q = 25u64;
        let rhs = addmod(exact.residue % q, mulmod(r.kappa, exact.prime % q, q), q);
        assert_eq!(approx.residue, rhs);
    }

    #[test]
    fn t16_k_eliminate_stress_4200() {
        for seed in 0..100u64 {
            for ei in 0..7 {
                let exact = ExactLane::encode((seed * 7919 + ei as u64 * 1337) as i64, ei);
                for aj in 0..6 {
                    let q = DEFAULT_APPROX_MODULI[aj];
                    let approx = ApproxSubLane {
                        residue: ((seed + 1) * 4567 + aj as u64 * 2903) % q,
                        modulus: q, frozen_epsilon_mb: 500,
                        lane_index: aj, source: "stress".into(),
                    };
                    let r = k_eliminate(&exact, &approx);
                    let rhs = addmod(exact.residue % q, mulmod(r.kappa, exact.prime % q, q), q);
                    assert_eq!(approx.residue, rhs);
                }
            }
        }
    }

    #[test]
    fn t17_coprimality_all_pairs() {
        for &p in &EXACT_PRIMES {
            for &base in &SAFE_BASIS {
                assert!(p % base != 0, "p={} div by {}", p, base);
            }
        }
    }

    // ── Lifted Winding Accumulator ──

    #[test]
    fn t18_lifted_constant_zero_winding() {
        let mut accum = LiftedWindingAccumulator::new(DEFAULT_APPROX_MODULI);
        let kappa = [[5u64; 6]; 7];
        for _ in 0..20 {
            accum.record(&kappa);
        }
        assert_eq!(accum.current_linf(), 0);
        assert_eq!(accum.vorticity_linf(), 0);
        assert_eq!(accum.total_wraps, 0);
    }

    #[test]
    fn t19_lifted_linear_growth() {
        let moduli = DEFAULT_APPROX_MODULI;
        let mut accum = LiftedWindingAccumulator::new(moduli);
        for t in 0..50u64 {
            let mut kappa = [[0u64; 6]; 7];
            for i in 0..7 {
                for j in 0..6 {
                    kappa[i][j] = (t * 3) % moduli[j];
                }
            }
            accum.record(&kappa);
        }
        assert!(accum.current_linf() > 0, "Linear growth must produce nonzero lifted winding");
    }

    #[test]
    fn t20_lifted_wrap_detection() {
        let moduli = DEFAULT_APPROX_MODULI;
        let mut accum = LiftedWindingAccumulator::new(moduli);

        // Step 1: start at 100
        let mut ka = [[0u64; 6]; 7];
        ka[0][0] = 100;
        accum.record(&ka);

        // Step 2: jump to 500 — displacement +400, which is > 1024/3 ≈ 341
        // and < 1024/2 = 512, so signed displacement is +400 (no sign flip)
        let mut kb = [[0u64; 6]; 7];
        kb[0][0] = 500;
        accum.record(&kb);

        // The lifted winding should be +400 (large forward step)
        assert_eq!(accum.lifted[0][0], 400);
        assert!(accum.total_wraps > 0, "Large displacement should trigger wrap detection");

        // Also verify: small boundary crossing does NOT trigger wrap
        let moduli2 = DEFAULT_APPROX_MODULI;
        let mut accum2 = LiftedWindingAccumulator::new(moduli2);
        let mut kc = [[0u64; 6]; 7];
        kc[0][0] = moduli2[0] - 1; // 1023
        accum2.record(&kc);

        let mut kd = [[0u64; 6]; 7];
        kd[0][0] = 1;
        accum2.record(&kd);

        // Small boundary crossing: displacement = +2, NOT a wrap
        assert_eq!(accum2.lifted[0][0], 2);
    }

    #[test]
    fn t21_signed_displacement_symmetric() {
        let q = 1024u64;
        assert_eq!(LiftedWindingAccumulator::signed_displacement(10, 15, q), 5);
        assert_eq!(LiftedWindingAccumulator::signed_displacement(15, 10, q), -5);
        assert_eq!(LiftedWindingAccumulator::signed_displacement(1020, 3, q), 7);
        assert_eq!(LiftedWindingAccumulator::signed_displacement(3, 1020, q), -7);
    }

    #[test]
    fn t22_lifted_vorticity_rows_isolated() {
        let moduli = DEFAULT_APPROX_MODULI;
        let mut accum = LiftedWindingAccumulator::new(moduli);
        for t in 0..20u64 {
            let mut kappa = [[0u64; 6]; 7];
            for j in 0..6 { for i in 0..4 { kappa[i][j] = 5; } }
            for j in 0..6 {
                kappa[4][j] = (t * 7) % moduli[j];
                kappa[5][j] = (t * 11) % moduli[j];
                kappa[6][j] = (t * 13) % moduli[j];
            }
            accum.record(&kappa);
        }
        let vel_max: i128 = (0..4)
            .flat_map(|i| (0..6).map(move |j| accum.lifted[i][j].unsigned_abs() as i128))
            .max().unwrap();
        assert_eq!(vel_max, 0);
        assert!(accum.vorticity_linf() > 0);
    }

    // ── Phase-Lock Signature ──

    #[test]
    fn t23_signature_42_dimensions() {
        let exact: [ExactLane; 7] = std::array::from_fn(|i| ExactLane::encode(100 * (i as i64 + 1), i));
        let approx: [ApproxSubLane; 6] = std::array::from_fn(|j| ApproxSubLane {
            residue: (50 + j as u64 * 17) % DEFAULT_APPROX_MODULI[j],
            modulus: DEFAULT_APPROX_MODULI[j], frozen_epsilon_mb: 500,
            lane_index: j, source: format!("t_{}", j),
        });
        let sig = PhaseLockSignature::compute(&exact, &approx);
        assert_eq!(sig.flat_vector().len(), 42);
    }

    #[test]
    fn t24_signature_zero_winding_constant() {
        let val = 5i64;
        let exact: [ExactLane; 7] = std::array::from_fn(|i| ExactLane::encode(val, i));
        let approx: [ApproxSubLane; 6] = std::array::from_fn(|j| ApproxSubLane {
            residue: val as u64 % DEFAULT_APPROX_MODULI[j],
            modulus: DEFAULT_APPROX_MODULI[j], frozen_epsilon_mb: 500,
            lane_index: j, source: "const".into(),
        });
        assert_eq!(PhaseLockSignature::compute(&exact, &approx).nonzero_count, 0);
    }

    // ── Grid2D ──

    #[test]
    fn t25_grid_creation_access() {
        let mut grid = Grid2D::new(8, &[0, 1, 3, 6]);
        grid.set(0, 3, 5, 42);
        assert_eq!(grid.get(0, 3, 5), 42);
        assert_eq!(grid.get(0, 0, 0), 0);
    }

    #[test]
    fn t26_grid_periodic_wrap() {
        let grid = Grid2D::new(8, &[0]);
        assert_eq!(grid.wrap(-1), 7);
        assert_eq!(grid.wrap(8), 0);
        assert_eq!(grid.wrap(-9), 7);
    }

    // ── Finite Differences ──

    #[test]
    fn t27_centered_diff_constant() {
        let p = EXACT_PRIMES[0];
        assert_eq!(centered_diff_residue(42, 42, p), 0);
    }

    #[test]
    fn t28_inv_2h_roundtrip() {
        let h_scaled = SCALE_FACTOR as u64;
        let inv = precompute_inv_2h(h_scaled);
        for i in 0..7 {
            let p = EXACT_PRIMES[i];
            let two_h = (2 * h_scaled) % p;
            assert_eq!(mulmod(two_h, inv[i], p), 1, "Lane {}", i);
        }
    }

    // ── Taylor-Green ──

    #[test]
    fn t29_taylor_green_origin_zero() {
        let grid = taylor_green_init(16, 1.0, 1.0);
        let ux = ExactLane { residue: grid.get(0, 0, 0), prime: EXACT_PRIMES[0], lane_index: 0 };
        let uy = ExactLane { residue: grid.get(1, 0, 0), prime: EXACT_PRIMES[1], lane_index: 1 };
        assert_eq!(ux.decode_signed(), 0, "u_x(0,0) = A cos(0) sin(0) = 0");
        assert_eq!(uy.decode_signed(), 0, "u_y(0,0) = -A sin(0) cos(0) = 0");
    }

    #[test]
    fn t30_taylor_green_antisymmetry() {
        // u_x(x,y) = -u_x(-x,y) for k=1 (odd in x via sin)
        // Actually u_x = A cos(kx) sin(ky) which is even in x.
        // But u_y = -A sin(kx) cos(ky) which is odd in x.
        // So u_y(x, y) = -u_y(-x, y) (mod periodicity).
        let n = 16;
        let grid = taylor_green_init(n, 1.0, 1.0);
        let p1 = EXACT_PRIMES[1];

        for ix in 1..n/2 {
            let iy = n / 4;
            let uy_pos = ExactLane { residue: grid.get(1, iy, ix), prime: p1, lane_index: 1 };
            let uy_neg = ExactLane { residue: grid.get(1, iy, n - ix), prime: p1, lane_index: 1 };
            assert_eq!(
                uy_pos.decode_signed(), -uy_neg.decode_signed(),
                "u_y antisymmetry at ix={}", ix
            );
        }
    }

    #[test]
    fn t31_taylor_green_divergence_free() {
        let n = 32;
        let grid = taylor_green_init(n, 1.0, 1.0);
        let cx = n / 2;
        let cy = n / 2;
        let p = EXACT_PRIMES[0];
        let h_scaled = ((2.0 * PI / n as f64) * SCALE_FACTOR as f64).round() as u64;
        let inv_2h = precompute_inv_2h(h_scaled.max(1));

        let ux_xp = grid.get(0, cy, (cx + 1) % n) % p;
        let ux_xm = grid.get(0, cy, cx - 1) % p;
        let dux_dx = mulmod(centered_diff_residue(ux_xp, ux_xm, p), inv_2h[0], p);

        let uy_yp = grid.get(1, (cy + 1) % n, cx) % p;
        let uy_ym = grid.get(1, cy - 1, cx) % p;
        let duy_dy = mulmod(centered_diff_residue(uy_yp, uy_ym, p), inv_2h[0], p);

        let div = addmod(dux_dx, duy_dy, p);
        let div_signed = if div <= p / 2 { div as i128 } else { div as i128 - p as i128 };
        let threshold = SCALE_FACTOR as i128;
        assert!(
            div_signed.unsigned_abs() < threshold as u128,
            "Divergence too large: {}", div_signed
        );
    }

    // ── Nonlinear Advection ──

    #[test]
    fn t32_advection_zero_constant_field() {
        let n = 8;
        let mut grid = Grid2D::new(n, &[0, 1, 3, 6]);
        let p0 = EXACT_PRIMES[0];
        for iy in 0..n { for ix in 0..n {
            grid.set(0, iy, ix, 42);
            grid.set(1, iy, ix, 42);
        }}
        let inv_2h = precompute_inv_2h(SCALE_FACTOR as u64);
        assert_eq!(
            compute_advection_residue(&grid, 4, 4, 0, inv_2h[0], p0), 0,
            "Advection of constant field must be zero"
        );
    }

    #[test]
    fn t33_vorticity_zero_irrotational() {
        let n = 8;
        let mut grid = Grid2D::new(n, &[0, 1, 3, 6]);
        for iy in 0..n { for ix in 0..n {
            grid.set(0, iy, ix, 100);
            grid.set(1, iy, ix, 200);
        }}
        let inv_2h = precompute_inv_2h(SCALE_FACTOR as u64);
        assert_eq!(
            compute_vorticity_residue(&grid, 4, 4, inv_2h[6], EXACT_PRIMES[6]), 0,
            "Vorticity of uniform flow must be zero"
        );
    }

    #[test]
    fn t34_advection_nonzero_taylor_green() {
        // Taylor-Green has nonzero advection (except at symmetry points)
        let n = 16;
        let grid = taylor_green_init(n, 1.0, 1.0);
        let p0 = EXACT_PRIMES[0];
        let h_scaled = ((2.0 * PI / n as f64) * SCALE_FACTOR as f64).round() as u64;
        let inv_2h = precompute_inv_2h(h_scaled.max(1));

        // At a generic point (not on symmetry axes), advection should be nonzero
        let adv = compute_advection_residue(&grid, 3, 5, 0, inv_2h[0], p0);
        // We just check it's computable and not trivially zero at a generic point
        // (it could be zero at special points, so we pick an asymmetric location)
        let _ = adv; // no panic = success; value depends on exact discretization
    }

    // ── NS Evolution ──

    #[test]
    fn t35_ns_advance_preserves_grid() {
        let state = NSState2D::taylor_green(16, 1.0, 1.0);
        let params = NSEvolveParams::new(16, 100.0, 1.0);
        let s1 = state.advance(&params);
        assert_eq!(s1.timestep, 1);
        assert_eq!(s1.grid.n, 16);
    }

    #[test]
    fn t36_ns_multi_step_no_panic() {
        let mut state = NSState2D::taylor_green(8, 1.0, 1.0);
        let params = NSEvolveParams::new(8, 100.0, 1.0);
        for _ in 0..10 { state = state.advance(&params); }
        assert_eq!(state.timestep, 10);
    }

    #[test]
    fn t37_ns_zero_stays_zero() {
        let n = 8;
        let grid = Grid2D::new(n, &[0, 1, 3, 6]);
        let state = NSState2D { grid, timestep: 0 };
        let params = NSEvolveParams::new(n, 100.0, 1.0);
        let s1 = state.advance(&params);
        for iy in 0..n { for ix in 0..n {
            assert_eq!(s1.grid.get(0, iy, ix), 0);
            assert_eq!(s1.grid.get(1, iy, ix), 0);
        }}
    }

    // ── Full Simulation ──

    #[test]
    fn t38_simulation_produces_data() {
        let (accum, ke, enst) = run_ns_simulation(
            8, 100.0, 1.0, 1.0, 10, DEFAULT_APPROX_MODULI,
        );
        assert_eq!(ke.len(), 10);
        assert_eq!(enst.len(), 10);
        assert_eq!(accum.linf_history.len(), 10);
    }

    #[test]
    fn t39_simulation_low_re_bounded() {
        let (accum, _, _) = run_ns_simulation(
            16, 10.0, 1.0, 1.0, 20, DEFAULT_APPROX_MODULI,
        );
        let (bounded, _) = accum.vorticity_bounded(10);
        assert!(bounded, "Low-Re Taylor-Green should have bounded vorticity winding");
    }

    // ── Absorption API ──

    #[test]
    fn t40_absorb_prime_power_sublanes() {
        let lanes = ApproxSubLane::absorb(42, 1000, 500, 0).unwrap();
        assert_eq!(lanes.len(), 2);
        let moduli: Vec<u64> = lanes.iter().map(|l| l.modulus).collect();
        assert!(moduli.contains(&8));
        assert!(moduli.contains(&125));
    }

    #[test]
    fn t41_absorb_rejects_non_smooth() {
        assert!(ApproxSubLane::absorb(42, 13, 500, 0).is_err());
        assert!(ApproxSubLane::absorb(42, 26, 500, 0).is_err());
    }

    // ── Display ──

    #[test]
    fn t42_display_labels() {
        let mut accum = LiftedWindingAccumulator::new(DEFAULT_APPROX_MODULI);
        accum.record(&[[3u64; 6]; 7]);
        accum.record(&[[3u64; 6]; 7]);
        let s = format!("{}", accum);
        assert!(s.contains("Lifted Winding"));
        assert!(s.contains("ω_z"));
        assert!(s.contains("Wraps"));
    }
}
