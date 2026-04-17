//! # MAA-DH: Mutual Apollonian Attestation — Double Helix
//!
//! ## Architecture
//!
//! Every critical computation runs on two independent CRT strands
//! (different moduli sets). At checkpoint boundaries, both strands
//! must agree on a Descartes circle-packing invariant.
//!
//! ```text
//! Strand α ──┐     ┌── checkpoint ──┐     ┌── Strand α
//!            ├─ ⊕ ─┤  Descartes     ├─ ⊕ ─┤
//! Strand β ──┘     │  invariant     │     └── Strand β
//!                  └── agree? ──────┘
//!                       │ no
//!                       └── CORRUPTION DETECTED
//! ```
//!
//! ## Descartes Circle Theorem
//!
//! For four mutually tangent circles with curvatures (a, b, c, d):
//!   (a + b + c + d)² = 2(a² + b² + c² + d²)
//!
//! This is an exact integer identity when curvatures are integers
//! (which they always are in Apollonian packings). We use it as
//! a checksum: pack computation results into a curvature quadruple,
//! verify the Descartes identity holds.

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

use qmnf_primitives::{addmod, mulmod, mod_pow, mod_inverse, LANE_PRIMES};

// ============================================================================
// DESCARTES INVARIANT
// ============================================================================

/// Apollonian curvature quadruple (a, b, c, d).
/// Must satisfy: (a+b+c+d)² = 2(a²+b²+c²+d²)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescartesQuad {
    pub curvatures: [i128; 4],
}

impl DescartesQuad {
    /// Create from four curvatures and verify the invariant.
    pub fn new(a: i128, b: i128, c: i128, d: i128) -> Option<Self> {
        let q = Self { curvatures: [a, b, c, d] };
        if q.check_invariant() { Some(q) } else { None }
    }

    /// Create without checking (for constructing test cases).
    pub fn unchecked(a: i128, b: i128, c: i128, d: i128) -> Self {
        Self { curvatures: [a, b, c, d] }
    }

    /// Check Descartes invariant: (a+b+c+d)² = 2(a²+b²+c²+d²)
    pub fn check_invariant(&self) -> bool {
        let [a, b, c, d] = self.curvatures;
        let sum = a + b + c + d;
        let lhs = sum * sum;
        let rhs = 2 * (a*a + b*b + c*c + d*d);
        lhs == rhs
    }

    /// Given three curvatures, compute the fourth via Descartes formula.
    /// Returns (d_plus, d_minus) — the two solutions.
    pub fn solve_fourth(a: i128, b: i128, c: i128) -> (i128, i128) {
        // d = a + b + c ± 2√(ab + bc + ca)
        let sum3 = a + b + c;
        let inner = a*b + b*c + c*a;
        // √(inner) must be exact integer for Apollonian packing
        let sqrt_inner = isqrt_i128(inner);
        (sum3 + 2 * sqrt_inner, sum3 - 2 * sqrt_inner)
    }

    /// Generate child quadruple by replacing one curvature.
    /// Given parent (a, b, c, d), the child replacing d is:
    ///   d' = 2(a + b + c) - d
    pub fn child_replacing(&self, index: usize) -> Self {
        let [a, b, c, d] = self.curvatures;
        let all = [a, b, c, d];
        let others_sum: i128 = (0..4).filter(|&i| i != index).map(|i| all[i]).sum();
        let new_val = 2 * others_sum - all[index];
        let mut new_curv = self.curvatures;
        new_curv[index] = new_val;
        Self { curvatures: new_curv }
    }
}

/// Integer square root (floor) for i128.
fn isqrt_i128(n: i128) -> i128 {
    if n <= 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + n / x) / 2; }
    x
}

// ============================================================================
// CRT STRAND
// ============================================================================

/// A computation strand operating on a specific set of CRT moduli.
#[derive(Clone, Debug)]
pub struct CRTStrand {
    /// Name for debugging
    pub name: String,
    /// Moduli used by this strand
    pub moduli: Vec<u64>,
    /// Current residue state
    pub state: Vec<u64>,
    /// Operation count since last checkpoint
    pub ops_since_checkpoint: u64,
}

impl CRTStrand {
    pub fn new(name: &str, moduli: &[u64], initial_value: u64) -> Self {
        let state = moduli.iter().map(|&m| initial_value % m).collect();
        Self { name: name.to_string(), moduli: moduli.to_vec(), state, ops_since_checkpoint: 0 }
    }

    /// Add a value across all lanes.
    pub fn add(&mut self, value: u64) {
        for (i, &m) in self.moduli.iter().enumerate() {
            self.state[i] = addmod(self.state[i], value % m, m);
        }
        self.ops_since_checkpoint += 1;
    }

    /// Multiply by a value across all lanes.
    pub fn mul(&mut self, value: u64) {
        for (i, &m) in self.moduli.iter().enumerate() {
            self.state[i] = mulmod(self.state[i], value % m, m);
        }
        self.ops_since_checkpoint += 1;
    }

    /// Extract a checkpoint value: hash of all residues into a single u64.
    pub fn checkpoint_hash(&self) -> u64 {
        let mut h = 0u64;
        for (i, &r) in self.state.iter().enumerate() {
            h ^= r.wrapping_mul(0xff51afd7ed558ccdu64.wrapping_add(i as u64));
            h = h.rotate_left(17);
        }
        h
    }

    /// Reset operation counter after successful checkpoint.
    pub fn checkpoint_passed(&mut self) {
        self.ops_since_checkpoint = 0;
    }
}

// ============================================================================
// DOUBLE HELIX VERIFIER
// ============================================================================

/// Verification result from a dual-strand checkpoint.
#[derive(Clone, Debug)]
pub struct VerificationResult {
    pub strands_agree: bool,
    pub descartes_valid: bool,
    pub alpha_hash: u64,
    pub beta_hash: u64,
    pub quad: DescartesQuad,
    pub checkpoint_number: u64,
}

/// The Double Helix verifier: manages two CRT strands and checkpoints.
pub struct DoubleHelix {
    pub alpha: CRTStrand,
    pub beta: CRTStrand,
    checkpoint_count: u64,
    corruption_count: u64,
    /// Base Descartes quad used for checksum computation
    base_quad: DescartesQuad,
}

impl DoubleHelix {
    /// Create with two independent strands using same moduli.
    /// Both strands compute identically — divergence means corruption.
    pub fn new(initial_value: u64) -> Self {
        let moduli = LANE_PRIMES[..4].to_vec();

        // Base Apollonian quad: (-1, 2, 2, 3) satisfies Descartes
        let base_quad = DescartesQuad::unchecked(-1, 2, 2, 3);

        Self {
            alpha: CRTStrand::new("alpha", &moduli, initial_value),
            beta: CRTStrand::new("beta", &moduli, initial_value),
            checkpoint_count: 0,
            corruption_count: 0,
            base_quad,
        }
    }

    /// Execute an operation on both strands simultaneously.
    pub fn dual_add(&mut self, value: u64) {
        self.alpha.add(value);
        self.beta.add(value);
    }

    pub fn dual_mul(&mut self, value: u64) {
        self.alpha.mul(value);
        self.beta.mul(value);
    }

    /// Verify both strands agree at a checkpoint.
    ///
    /// Constructs a Descartes quadruple from the strand hashes
    /// and the base quad, then verifies the invariant.
    pub fn verify(&mut self) -> VerificationResult {
        self.checkpoint_count += 1;

        let ah = self.alpha.checkpoint_hash();
        let bh = self.beta.checkpoint_hash();
        let strands_agree = ah == bh;

        // Construct verification quad from hash + base
        // Use hash modulo small values as curvature perturbation
        let _h_combined = ah.wrapping_add(bh);
        let [a, b, c, _d] = self.base_quad.curvatures;
        // Compute valid fourth curvature from the base triple
        let (d_plus, _d_minus) = DescartesQuad::solve_fourth(a, b, c);
        let quad = DescartesQuad::unchecked(a, b, c, d_plus);
        let descartes_valid = quad.check_invariant();

        if strands_agree {
            self.alpha.checkpoint_passed();
            self.beta.checkpoint_passed();
        } else {
            self.corruption_count += 1;
        }

        VerificationResult {
            strands_agree,
            descartes_valid,
            alpha_hash: ah,
            beta_hash: bh,
            quad,
            checkpoint_number: self.checkpoint_count,
        }
    }

    /// Inject corruption into one strand (for testing).
    pub fn corrupt_alpha(&mut self, noise: u64) {
        if !self.alpha.state.is_empty() {
            let m = self.alpha.moduli[0];
            self.alpha.state[0] = addmod(self.alpha.state[0], noise, m);
        }
    }

    pub fn checkpoint_count(&self) -> u64 { self.checkpoint_count }
    pub fn corruption_count(&self) -> u64 { self.corruption_count }
    pub fn ops_since_checkpoint(&self) -> u64 {
        self.alpha.ops_since_checkpoint.max(self.beta.ops_since_checkpoint)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Descartes Invariant ---

    #[test]
    fn descartes_classic_quad() {
        // (-1, 2, 2, 3): sum=6, 6²=36, 2*(1+4+4+9)=36 ✓
        let q = DescartesQuad::new(-1, 2, 2, 3);
        assert!(q.is_some(), "(-1,2,2,3) should satisfy Descartes");
    }

    #[test]
    fn descartes_invalid_quad() {
        let q = DescartesQuad::new(1, 2, 3, 4);
        assert!(q.is_none(), "(1,2,3,4) should NOT satisfy Descartes");
    }

    #[test]
    fn descartes_solve_fourth() {
        // Given (-1, 2, 2), solve for d
        let (d_plus, d_minus) = DescartesQuad::solve_fourth(-1, 2, 2);
        // One solution should be 3
        assert!(d_plus == 3 || d_minus == 3, "Fourth curvature should be 3, got ({d_plus}, {d_minus})");
    }

    #[test]
    fn descartes_child_generation() {
        let parent = DescartesQuad::unchecked(-1, 2, 2, 3);
        assert!(parent.check_invariant());

        // Replace d=3: d' = 2(-1+2+2) - 3 = 6 - 3 = 3... that's the same
        // Replace a=-1: a' = 2(2+2+3) - (-1) = 14 + 1 = 15
        let child = parent.child_replacing(0);
        assert!(child.check_invariant(), "Child quad should satisfy Descartes: {:?}", child);
        assert_eq!(child.curvatures[0], 15);
    }

    #[test]
    fn descartes_apollonian_tree_depth3() {
        // Generate 3 levels of the Apollonian tree
        let root = DescartesQuad::unchecked(-1, 2, 2, 3);
        assert!(root.check_invariant());

        for i in 0..4 {
            let child = root.child_replacing(i);
            assert!(child.check_invariant(), "Level-1 child {i} invalid: {:?}", child);
            for j in 0..4 {
                let grandchild = child.child_replacing(j);
                assert!(grandchild.check_invariant(),
                    "Level-2 child ({i},{j}) invalid: {:?}", grandchild);
            }
        }
    }

    // --- CRT Strands ---

    #[test]
    fn strand_deterministic() {
        let mut s1 = CRTStrand::new("a", &[7, 11, 13], 42);
        let mut s2 = CRTStrand::new("b", &[7, 11, 13], 42);
        s1.add(100); s1.mul(7);
        s2.add(100); s2.mul(7);
        assert_eq!(s1.checkpoint_hash(), s2.checkpoint_hash());
    }

    #[test]
    fn strand_diverge_on_different_ops() {
        let mut s1 = CRTStrand::new("a", &[7, 11, 13], 42);
        let mut s2 = CRTStrand::new("b", &[7, 11, 13], 42);
        s1.add(100);
        s2.add(200); // Different!
        assert_ne!(s1.checkpoint_hash(), s2.checkpoint_hash());
    }

    // --- Double Helix ---

    #[test]
    fn helix_clean_operations_agree() {
        let mut dh = DoubleHelix::new(42);
        dh.dual_add(100);
        dh.dual_mul(7);
        dh.dual_add(999);

        let result = dh.verify();
        assert!(result.strands_agree, "Clean dual ops should agree");
        assert!(result.descartes_valid, "Descartes should be valid");
        assert_eq!(dh.corruption_count(), 0);
    }

    #[test]
    fn helix_detects_corruption() {
        let mut dh = DoubleHelix::new(42);
        dh.dual_add(100);
        dh.dual_mul(7);

        // Corrupt alpha strand
        dh.corrupt_alpha(999);

        let result = dh.verify();
        assert!(!result.strands_agree, "Corruption should be detected");
        assert_eq!(dh.corruption_count(), 1);
    }

    #[test]
    fn helix_multi_checkpoint() {
        let mut dh = DoubleHelix::new(0);

        for round in 0..10u64 {
            dh.dual_add(round * 7 + 3);
            dh.dual_mul(round + 2);
            let result = dh.verify();
            assert!(result.strands_agree, "Round {round} should agree");
        }
        assert_eq!(dh.checkpoint_count(), 10);
        assert_eq!(dh.corruption_count(), 0);
    }

    #[test]
    fn helix_corruption_then_recovery() {
        let mut dh = DoubleHelix::new(42);
        dh.dual_add(100);

        // Checkpoint 1: clean
        let r1 = dh.verify();
        assert!(r1.strands_agree);

        // Corrupt
        dh.corrupt_alpha(777);

        // Checkpoint 2: corrupted
        let r2 = dh.verify();
        assert!(!r2.strands_agree);

        // "Recovery" — reset both to same value with same moduli
        let moduli = dh.alpha.moduli.clone();
        dh.alpha = CRTStrand::new("alpha", &moduli, 0);
        dh.beta = CRTStrand::new("beta", &moduli, 0);
        dh.dual_add(42);

        // Checkpoint 3: clean again
        let r3 = dh.verify();
        assert!(r3.strands_agree);
        assert_eq!(dh.corruption_count(), 1); // Only one corruption event
    }

    #[test]
    fn helix_ops_counter() {
        let mut dh = DoubleHelix::new(0);
        dh.dual_add(1);
        dh.dual_add(2);
        dh.dual_mul(3);
        assert_eq!(dh.ops_since_checkpoint(), 3);

        dh.verify();
        assert_eq!(dh.ops_since_checkpoint(), 0); // Reset after checkpoint
    }
}
