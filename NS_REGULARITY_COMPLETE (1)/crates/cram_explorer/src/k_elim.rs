//! # K-Elimination Engine
//!
//! The fundamental CRAM bridge operation. Given value X living in two
//! coprime lanes (mod m) and (mod a), extracts the exact winding number:
//!
//! ```text
//!     k ≡ (s − r) · m⁻¹ (mod a)
//!     X = r + k·m     (unique for 0 ≤ X < m·a)
//! ```
//!
//! The winding number k is ALWAYS an exact integer — a topological
//! invariant counting how many times X wraps the smaller modulus.
//!
//! Works across: exact↔exact, exact↔approx, approx↔approx.
//! The bridge between scales in multi-resolution analysis.

use crate::mod_arith::*;

/// Result of a K-Elimination operation.
#[derive(Clone, Debug)]
pub struct KElimResult {
    /// The winding number (always exact integer)
    pub k: i64,
    /// Reconstructed value in [0, m*a)
    pub value: i64,
    /// Modulus of the first lane
    pub m: i64,
    /// Modulus of the second lane
    pub a: i64,
}

/// Perform K-Elimination between two coprime lanes.
///
/// Given: X ≡ r (mod m), X ≡ s (mod a), gcd(m,a) = 1
/// Returns: k such that X = r + k·m, with k ∈ [0, a)
pub fn k_eliminate(r: i64, m: i64, s: i64, a: i64) -> KElimResult {
    debug_assert_eq!(gcd_i64(m, a), 1, "K-Elim requires coprime moduli");
    let m_inv_a = mod_inv(m, a);
    let k = modd128((s as i128 - r as i128) * m_inv_a as i128, a);
    let value = r + k * m;
    KElimResult { k, value, m, a }
}

/// Multi-lane CRT reconstruction via iterated K-Elimination.
/// Given residues r_i mod m_i (all pairwise coprime), reconstructs
/// the unique X in [0, product(m_i)).
///
/// This is Garner's algorithm expressed as iterated K-Elimination.
pub fn garner_reconstruct(residues: &[(i64, i64)]) -> i64 {
    assert!(!residues.is_empty());
    let (mut value, mut modulus) = residues[0];
    value = modd(value, modulus);

    for &(r_i, m_i) in &residues[1..] {
        let result = k_eliminate(value, modulus, r_i, m_i);
        value = result.value;
        modulus *= m_i;
    }
    value
}

/// Batch K-Elimination: compute winding numbers between a reference
/// lane and all other lanes. Returns vec of (lane_modulus, winding_k).
pub fn k_elim_survey(ref_residue: i64, ref_mod: i64, lanes: &[(i64, i64)]) -> Vec<(i64, i64)> {
    lanes.iter()
        .map(|&(residue, modulus)| {
            let result = k_eliminate(ref_residue, ref_mod, residue, modulus);
            (modulus, result.k)
        })
        .collect()
}

/// Cross-lane K-Elimination matrix: compute k between every pair.
/// Returns lower-triangular matrix of winding numbers.
pub fn k_elim_matrix(lanes: &[(i64, i64)]) -> Vec<Vec<i64>> {
    let n = lanes.len();
    let mut matrix = vec![vec![0i64; n]; n];
    for i in 0..n {
        for j in (i+1)..n {
            let (r_i, m_i) = lanes[i];
            let (r_j, m_j) = lanes[j];
            if gcd_i64(m_i, m_j) == 1 {
                let result = k_eliminate(r_i, m_i, r_j, m_j);
                matrix[i][j] = result.k;
                matrix[j][i] = k_eliminate(r_j, m_j, r_i, m_i).k;
            }
        }
    }
    matrix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_kelim() {
        // X = 42: 42 mod 7 = 0, 42 mod 11 = 9
        let r = k_eliminate(0, 7, 9, 11);
        assert_eq!(r.k, 6);  // k = (9-0)*7^{-1} mod 11 = 9*8 mod 11 = 72 mod 11 = 6
        assert_eq!(r.value, 42);
    }

    #[test]
    fn test_garner() {
        // 394 = 79 mod 105, 10 mod 16
        let val = garner_reconstruct(&[(79, 105), (10, 16)]);
        assert_eq!(val, 394);
    }

    #[test]
    fn test_garner_3way() {
        // 42 mod 3 = 0, 42 mod 5 = 2, 42 mod 7 = 0
        let val = garner_reconstruct(&[(0, 3), (2, 5), (0, 7)]);
        assert_eq!(val, 42);
    }
}
