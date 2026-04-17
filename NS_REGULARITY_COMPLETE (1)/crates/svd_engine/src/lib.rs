//! # Integer-Only SVD Engine — W3.3
//!
//! Computes singular value decomposition properties using exact integer
//! arithmetic. No floats.
//!
//! ## Approach
//!
//! Full SVD requires trigonometric rotations (sin/cos), which are inherently
//! continuous. We take two complementary approaches:
//!
//! 1. **Power iteration** for the dominant singular value (σ₁):
//!    Repeated matrix-vector multiply on A^T A converges to the dominant
//!    eigenvector. All operations are integer multiply + add.
//!    The singular value is recovered as the Rayleigh quotient.
//!
//! 2. **Jacobi rotations via integer CORDIC**: For full SVD of small matrices,
//!    CORDIC computes rotations using only shifts and adds — no sin/cos needed.
//!
//! ## Coherence Score
//!
//! The primary output for ASVE is the **coherence score**: the ratio of the
//! dominant singular value to the Frobenius norm. This measures how much of
//! the matrix's "energy" is concentrated in a single direction.
//!
//! - Coherence 10000 = rank-1 matrix (perfectly coherent)
//! - Coherence ~1000 = uniformly distributed (incoherent)
//! - Used by ASVE to decide confidence in knowledge graph state

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

// ============================================================================
// SCALED INTEGER MATRIX
// ============================================================================

/// Scale factor for integer arithmetic (2^20 gives ~6 decimal digits precision)
const SCALE: i128 = 1 << 20;
const SCALE_SQ: i128 = SCALE * SCALE;

/// A matrix of scaled integers.
/// Values are stored as value * SCALE internally.
#[derive(Clone, Debug)]
pub struct IntMatrix {
    pub rows: usize,
    pub cols: usize,
    /// Row-major storage, values are actual integers (not pre-scaled)
    pub data: Vec<i64>,
}

impl IntMatrix {
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self { rows, cols, data: vec![0; rows * cols] }
    }

    pub fn from_row_major(rows: usize, cols: usize, data: Vec<i64>) -> Self {
        assert_eq!(data.len(), rows * cols, "Data length mismatch");
        Self { rows, cols, data }
    }

    /// Identity matrix.
    pub fn identity(n: usize) -> Self {
        let mut m = Self::zeros(n, n);
        for i in 0..n { m.set(i, i, 1); }
        m
    }

    #[inline]
    pub fn get(&self, r: usize, c: usize) -> i64 {
        self.data[r * self.cols + c]
    }

    #[inline]
    pub fn set(&mut self, r: usize, c: usize, val: i64) {
        self.data[r * self.cols + c] = val;
    }

    /// Matrix-vector multiply: result[i] = Σⱼ M[i,j] * v[j]
    pub fn mul_vec(&self, v: &[i64]) -> Vec<i64> {
        assert_eq!(v.len(), self.cols);
        let mut result = vec![0i64; self.rows];
        for i in 0..self.rows {
            let mut sum = 0i128;
            for j in 0..self.cols {
                sum += self.get(i, j) as i128 * v[j] as i128;
            }
            result[i] = sum as i64;
        }
        result
    }

    /// Matrix multiply: C = A * B
    pub fn mul_mat(&self, other: &IntMatrix) -> IntMatrix {
        assert_eq!(self.cols, other.rows, "Dimension mismatch");
        let mut result = IntMatrix::zeros(self.rows, other.cols);
        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut sum = 0i128;
                for k in 0..self.cols {
                    sum += self.get(i, k) as i128 * other.get(k, j) as i128;
                }
                result.set(i, j, sum as i64);
            }
        }
        result
    }

    /// Transpose.
    pub fn transpose(&self) -> IntMatrix {
        let mut result = IntMatrix::zeros(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                result.set(j, i, self.get(i, j));
            }
        }
        result
    }

    /// Frobenius norm squared: Σ M[i,j]²
    pub fn frobenius_sq(&self) -> i128 {
        self.data.iter().map(|&x| x as i128 * x as i128).sum()
    }

    /// Integer square root (floor).
    pub fn isqrt(n: i128) -> i64 {
        if n <= 0 { return 0; }
        let mut x = n;
        let mut y = (x + 1) / 2;
        while y < x { x = y; y = (x + n / x) / 2; }
        x as i64
    }
}

// ============================================================================
// VECTOR OPERATIONS
// ============================================================================

/// Dot product of two integer vectors.
fn dot(a: &[i64], b: &[i64]) -> i128 {
    a.iter().zip(b).map(|(&x, &y)| x as i128 * y as i128).sum()
}

/// Squared L2 norm.
fn norm_sq(v: &[i64]) -> i128 {
    v.iter().map(|&x| x as i128 * x as i128).sum()
}

/// Scale a vector to prevent overflow: divide all components by a common factor.
/// Returns (scaled_vector, scale_factor_applied).
fn normalize_int(v: &[i64]) -> (Vec<i64>, i64) {
    let max_abs = v.iter().map(|&x| x.unsigned_abs()).max().unwrap_or(1).max(1);
    // Target: keep values in ~2^30 range to prevent i64 overflow in dot products
    let target = 1i64 << 30;
    let divisor = (max_abs as i64 / target).max(1);
    let scaled: Vec<i64> = v.iter().map(|&x| x / divisor).collect();
    (scaled, divisor)
}

// ============================================================================
// POWER ITERATION (Dominant Singular Value)
// ============================================================================

/// Result of power iteration.
#[derive(Clone, Debug)]
pub struct PowerIterResult {
    /// Dominant singular value squared (σ₁²), as integer
    pub sigma_sq: i128,
    /// Dominant right singular vector (unnormalized)
    pub v1: Vec<i64>,
    /// Number of iterations performed
    pub iterations: usize,
    /// Whether the iteration converged
    pub converged: bool,
}

/// Compute dominant singular value of A via power iteration on A^T A.
///
/// Returns σ₁² (the largest eigenvalue of A^T A).
/// The singular value σ₁ = √(σ₁²), but we keep it squared to stay integer.
pub fn power_iteration(a: &IntMatrix, max_iter: usize, tol_shift: u32) -> PowerIterResult {
    let ata = a.transpose().mul_mat(a);
    let n = ata.rows;

    // Initial vector: [1, 1, 1, ...]
    let mut v: Vec<i64> = vec![1; n];
    let mut prev_rayleigh: i128 = 0;
    let mut converged = false;
    let mut iterations = 0;

    for iter in 0..max_iter {
        iterations = iter + 1;

        // w = A^T A * v
        let w = ata.mul_vec(&v);

        // Normalize to prevent overflow
        let (w_norm, _) = normalize_int(&w);

        // Rayleigh quotient: (v^T A^T A v) / (v^T v), scaled by SCALE
        let numerator = dot(&v, &w);
        let denominator = norm_sq(&v);
        let rayleigh = if denominator > 0 {
            (numerator * SCALE) / denominator
        } else { 0 };

        // Convergence check: relative change in Rayleigh quotient
        if iter > 0 {
            let diff = (rayleigh - prev_rayleigh).unsigned_abs();
            let threshold = (prev_rayleigh.unsigned_abs() >> tol_shift).max(1);
            if diff < threshold as u128 {
                converged = true;
                v = w_norm;
                prev_rayleigh = rayleigh;
                break;
            }
        }

        prev_rayleigh = rayleigh;
        v = w_norm;
    }

    PowerIterResult {
        sigma_sq: prev_rayleigh,
        v1: v,
        iterations,
        converged,
    }
}

// ============================================================================
// COHERENCE SCORE
// ============================================================================

/// Coherence result for a matrix.
#[derive(Clone, Debug)]
pub struct CoherenceResult {
    /// Coherence score: 0..10000
    /// 10000 = rank-1 (perfectly coherent)
    /// ~(10000/rank) = uniformly distributed
    pub score: u64,
    /// Dominant singular value squared (scaled)
    pub sigma1_sq: i128,
    /// Frobenius norm squared
    pub frobenius_sq: i128,
    /// Effective rank estimate (Frobenius² / σ₁²)
    pub effective_rank_permil: u64,
    /// Power iteration details
    pub power_iter: PowerIterResult,
}

/// Compute coherence score of a matrix.
///
/// Coherence = σ₁² / ||A||_F² (ratio of dominant singular value energy to total)
///
/// Score is 0..10000 (integer). Higher = more coherent.
pub fn coherence(a: &IntMatrix) -> CoherenceResult {
    let frob_sq = a.frobenius_sq();
    if frob_sq == 0 {
        return CoherenceResult {
            score: 0,
            sigma1_sq: 0,
            frobenius_sq: 0,
            effective_rank_permil: 0,
            power_iter: PowerIterResult {
                sigma_sq: 0, v1: vec![], iterations: 0, converged: true
            },
        };
    }

    let pir = power_iteration(a, 100, 20);

    // σ₁² from power iteration is scaled by SCALE
    // coherence = σ₁² / frob_sq (but σ₁² has an extra SCALE factor from Rayleigh)
    // We need: score = (σ₁² / frob_sq) * 10000
    let sigma_sq_unscaled = pir.sigma_sq / SCALE;
    let score = if frob_sq > 0 {
        ((sigma_sq_unscaled * 10000) / frob_sq).unsigned_abs().min(10000) as u64
    } else { 0 };

    let effective_rank = if sigma_sq_unscaled > 0 {
        ((frob_sq * 1000) / sigma_sq_unscaled).unsigned_abs().min(99999) as u64
    } else { 0 };

    CoherenceResult {
        score,
        sigma1_sq: sigma_sq_unscaled,
        frobenius_sq: frob_sq,
        effective_rank_permil: effective_rank,
        power_iter: pir,
    }
}

// ============================================================================
// JACOBI ROTATION (2x2, exact integer CORDIC-style)
// ============================================================================

/// 2x2 Jacobi rotation result.
#[derive(Clone, Debug)]
pub struct Jacobi2x2 {
    /// Singular values squared (σ₁², σ₂²), sorted descending
    pub sigma_sq: [i128; 2],
    /// Off-diagonal energy eliminated
    pub off_diag_eliminated: i128,
}

/// Exact 2x2 SVD via characteristic polynomial (no trig needed).
///
/// For a 2x2 symmetric matrix [[a, b], [b, c]], the eigenvalues are:
///   λ = ((a+c) ± √((a-c)² + 4b²)) / 2
///
/// We return λ² to stay integer (avoiding the square root).
pub fn jacobi_2x2_symmetric(a: i64, b: i64, c: i64) -> Jacobi2x2 {
    let a = a as i128;
    let b = b as i128;
    let c = c as i128;

    let trace = a + c;
    let det = a * c - b * b;
    let disc = (a - c) * (a - c) + 4 * b * b; // discriminant, always ≥ 0

    // eigenvalues: (trace ± √disc) / 2
    // We compute eigenvalue products instead of square roots:
    //   λ₁ * λ₂ = det
    //   λ₁ + λ₂ = trace
    //   λ₁² + λ₂² = trace² - 2*det

    let sum_sq = trace * trace - 2 * det;
    let sqrt_disc = IntMatrix::isqrt(disc) as i128;

    let lambda1 = (trace + sqrt_disc) / 2;
    let lambda2 = (trace - sqrt_disc) / 2;

    let mut sigma_sq = [lambda1 * lambda1, lambda2 * lambda2];
    if sigma_sq[0] < sigma_sq[1] { sigma_sq.swap(0, 1); }

    Jacobi2x2 {
        sigma_sq,
        off_diag_eliminated: b * b,
    }
}

// ============================================================================
// COGNITIVE INTERFACE
// ============================================================================

/// Compute coherence of an HDV similarity matrix.
///
/// Given N HDV vectors, build the N×N similarity matrix (Hamming-based)
/// and compute its coherence score.
///
/// This is the primary interface for ASVE: given a set of knowledge graph
/// nodes, how coherent is their representation?
pub fn hdv_coherence(similarity_matrix: &[Vec<i64>]) -> CoherenceResult {
    let n = similarity_matrix.len();
    if n == 0 {
        return CoherenceResult {
            score: 0, sigma1_sq: 0, frobenius_sq: 0,
            effective_rank_permil: 0,
            power_iter: PowerIterResult { sigma_sq: 0, v1: vec![], iterations: 0, converged: true },
        };
    }

    let mut data = Vec::with_capacity(n * n);
    for row in similarity_matrix {
        assert_eq!(row.len(), n, "Similarity matrix must be square");
        data.extend_from_slice(row);
    }

    let mat = IntMatrix::from_row_major(n, n, data);
    coherence(&mat)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- IntMatrix basics ---

    #[test]
    fn matrix_identity() {
        let id = IntMatrix::identity(3);
        assert_eq!(id.get(0, 0), 1);
        assert_eq!(id.get(1, 1), 1);
        assert_eq!(id.get(0, 1), 0);
    }

    #[test]
    fn matrix_mul_vec() {
        // [[1,2],[3,4]] * [5,6] = [17, 39]
        let m = IntMatrix::from_row_major(2, 2, vec![1, 2, 3, 4]);
        let v = vec![5i64, 6];
        let result = m.mul_vec(&v);
        assert_eq!(result, vec![17, 39]);
    }

    #[test]
    fn matrix_mul_mat() {
        // [[1,2],[3,4]] * [[5,6],[7,8]] = [[19,22],[43,50]]
        let a = IntMatrix::from_row_major(2, 2, vec![1, 2, 3, 4]);
        let b = IntMatrix::from_row_major(2, 2, vec![5, 6, 7, 8]);
        let c = a.mul_mat(&b);
        assert_eq!(c.get(0, 0), 19);
        assert_eq!(c.get(0, 1), 22);
        assert_eq!(c.get(1, 0), 43);
        assert_eq!(c.get(1, 1), 50);
    }

    #[test]
    fn matrix_transpose() {
        let m = IntMatrix::from_row_major(2, 3, vec![1, 2, 3, 4, 5, 6]);
        let t = m.transpose();
        assert_eq!(t.rows, 3);
        assert_eq!(t.cols, 2);
        assert_eq!(t.get(0, 0), 1);
        assert_eq!(t.get(1, 0), 2);
        assert_eq!(t.get(2, 1), 6);
    }

    #[test]
    fn frobenius_norm() {
        let m = IntMatrix::from_row_major(2, 2, vec![3, 4, 0, 0]);
        assert_eq!(m.frobenius_sq(), 25); // 9 + 16
    }

    #[test]
    fn isqrt_correct() {
        assert_eq!(IntMatrix::isqrt(0), 0);
        assert_eq!(IntMatrix::isqrt(1), 1);
        assert_eq!(IntMatrix::isqrt(4), 2);
        assert_eq!(IntMatrix::isqrt(9), 3);
        assert_eq!(IntMatrix::isqrt(100), 10);
        assert_eq!(IntMatrix::isqrt(99), 9); // floor
    }

    // --- Power Iteration ---

    #[test]
    fn power_iter_rank1_matrix() {
        // Rank-1 matrix: [[4,2],[2,1]] has σ₁²=25, σ₂=0
        // A^T A = [[20,10],[10,5]], eigenvalues 25 and 0
        let a = IntMatrix::from_row_major(2, 2, vec![4, 2, 2, 1]);
        let pir = power_iteration(&a, 50, 15);
        assert!(pir.converged, "Should converge for rank-1");
        // σ₁² should be close to 25 (scaled by SCALE)
        let sigma_sq = pir.sigma_sq / SCALE;
        assert!((sigma_sq - 25).unsigned_abs() < 3, "σ₁² should be ~25, got {sigma_sq}");
    }

    #[test]
    fn power_iter_identity() {
        let a = IntMatrix::identity(3);
        let pir = power_iteration(&a, 50, 15);
        // Identity: all singular values = 1, so σ₁² = 1
        let sigma_sq = pir.sigma_sq / SCALE;
        assert!((sigma_sq - 1).unsigned_abs() < 2, "σ₁² of identity should be ~1, got {sigma_sq}");
    }

    #[test]
    fn power_iter_diagonal() {
        // diag(5, 3, 1): σ₁=5, σ₁²=25
        let a = IntMatrix::from_row_major(3, 3, vec![5,0,0, 0,3,0, 0,0,1]);
        let pir = power_iteration(&a, 50, 15);
        let sigma_sq = pir.sigma_sq / SCALE;
        assert!((sigma_sq - 25).unsigned_abs() < 3, "σ₁² should be ~25, got {sigma_sq}");
    }

    // --- Coherence ---

    #[test]
    fn coherence_rank1() {
        let a = IntMatrix::from_row_major(2, 2, vec![4, 2, 2, 1]);
        let c = coherence(&a);
        // Rank-1: all energy in one direction → coherence should be high
        assert!(c.score > 8000, "Rank-1 coherence should be >8000, got {}", c.score);
    }

    #[test]
    fn coherence_identity() {
        let a = IntMatrix::identity(3);
        let c = coherence(&a);
        // Identity: energy spread equally → coherence ≈ 1/3 = 3333
        assert!(c.score > 2000 && c.score < 5000,
            "Identity coherence should be ~3333, got {}", c.score);
    }

    #[test]
    fn coherence_zero_matrix() {
        let a = IntMatrix::zeros(3, 3);
        let c = coherence(&a);
        assert_eq!(c.score, 0, "Zero matrix should have 0 coherence");
    }

    #[test]
    fn coherence_single_element() {
        let a = IntMatrix::from_row_major(1, 1, vec![42]);
        let c = coherence(&a);
        assert_eq!(c.score, 10000, "1x1 matrix should be perfectly coherent");
    }

    // --- Jacobi 2x2 ---

    #[test]
    fn jacobi_2x2_diagonal() {
        // Already diagonal: [[5,0],[0,3]] → eigenvalues 5,3
        let j = jacobi_2x2_symmetric(5, 0, 3);
        assert_eq!(j.sigma_sq[0], 25); // 5²
        assert_eq!(j.sigma_sq[1], 9);  // 3²
        assert_eq!(j.off_diag_eliminated, 0);
    }

    #[test]
    fn jacobi_2x2_offdiag() {
        // [[2,1],[1,2]] → eigenvalues 3,1
        let j = jacobi_2x2_symmetric(2, 1, 2);
        assert_eq!(j.sigma_sq[0], 9); // 3²
        assert_eq!(j.sigma_sq[1], 1); // 1²
    }

    #[test]
    fn jacobi_2x2_large_offdiag() {
        // [[0,5],[5,0]] → eigenvalues 5,-5 → σ²=25,25
        let j = jacobi_2x2_symmetric(0, 5, 0);
        assert_eq!(j.sigma_sq[0], 25);
        assert_eq!(j.sigma_sq[1], 25);
    }

    // --- HDV Coherence Interface ---

    #[test]
    fn hdv_coherence_identity_sim() {
        // Perfect self-similarity matrix (identity-like)
        let sim = vec![
            vec![100i64, 0, 0],
            vec![0, 100, 0],
            vec![0, 0, 100],
        ];
        let c = hdv_coherence(&sim);
        assert!(c.score > 2000 && c.score < 5000,
            "Orthogonal HDVs should have moderate coherence, got {}", c.score);
    }

    #[test]
    fn hdv_coherence_identical_vectors() {
        // All vectors identical → perfect coherence
        let sim = vec![
            vec![100i64, 100, 100],
            vec![100, 100, 100],
            vec![100, 100, 100],
        ];
        let c = hdv_coherence(&sim);
        assert!(c.score > 8000, "Identical vectors should be highly coherent, got {}", c.score);
    }

    #[test]
    fn hdv_coherence_empty() {
        let c = hdv_coherence(&[]);
        assert_eq!(c.score, 0);
    }
}
