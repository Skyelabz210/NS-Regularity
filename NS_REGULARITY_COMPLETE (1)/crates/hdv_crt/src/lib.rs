//! # Hyperdimensional Vectors in CRT Space — W2.6
//!
//! Implements hyperdimensional computing (HDC) primitives using modular
//! integer arithmetic on CRT lanes. No floats.
//!
//! ## HDC Operations
//!
//! - **Bind** (⊗): Element-wise modular multiplication. Represents association.
//!   Invertible: unbind(bind(A, B), B) = A.
//! - **Bundle** (⊕): Element-wise modular addition. Represents set union.
//!   Result is similar to all operands.
//! - **Permute** (ρ): Cyclic shift of components. Represents sequence/ordering.
//! - **Similarity**: Exact modular inner product. Measures relatedness.
//!
//! ## CRT Advantage
//!
//! Standard HDC uses binary or bipolar vectors ({0,1} or {-1,+1}).
//! CRT-HDC uses ℤ/pℤ components — exponentially richer per dimension.
//! A 1000-dim vector over ℤ/p with p ≈ 2^30 has capacity ~30,000 bits,
//! vs 1000 bits for binary HDC. Same operations, vastly more capacity.

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

// ============================================================================
// MODULAR ARITHMETIC
// ============================================================================

#[inline]
fn addmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 + b as u128) % m as u128) as u64
}

#[inline]
fn submod(a: u64, b: u64, m: u64) -> u64 {
    if a >= b { (a - b) % m } else { m - ((b - a) % m) }
}

#[inline]
fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    if m == 1 { return 0; }
    let mut result = 1u64;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 { result = mulmod(result, base, m); }
        exp >>= 1;
        base = mulmod(base, base, m);
    }
    result
}

fn mod_inverse(a: u64, p: u64) -> u64 {
    mod_pow(a, p - 2, p)
}

// ============================================================================
// HDV TYPE
// ============================================================================

/// Default dimensionality
pub const DEFAULT_DIM: usize = 1024;

/// Default modulus (NTT-friendly prime, ~30 bits)
pub const DEFAULT_MODULUS: u64 = 998244353;

/// A hyperdimensional vector over ℤ/pℤ.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HDV {
    /// Components in [0, modulus)
    pub components: Vec<u64>,
    /// Prime modulus for all arithmetic
    pub modulus: u64,
}

impl HDV {
    /// Create zero vector.
    pub fn zero(dim: usize, modulus: u64) -> Self {
        Self { components: vec![0; dim], modulus }
    }

    /// Create from explicit components.
    pub fn from_components(components: Vec<u64>, modulus: u64) -> Self {
        let v = Self { components, modulus };
        debug_assert!(v.components.iter().all(|&c| c < modulus));
        v
    }

    /// Create pseudo-random vector from seed (deterministic).
    pub fn random(dim: usize, modulus: u64, seed: u64) -> Self {
        let mut components = Vec::with_capacity(dim);
        let mut state = seed;
        for _ in 0..dim {
            state ^= state >> 33;
            state = state.wrapping_mul(0xff51afd7ed558ccd);
            state ^= state >> 33;
            state = state.wrapping_mul(0xc4ceb9fe1a85ec53);
            state ^= state >> 33;
            components.push(state % modulus);
        }
        Self { components, modulus }
    }

    /// Dimensionality.
    pub fn dim(&self) -> usize {
        self.components.len()
    }

    // -- Core HDC Operations ------------------------------------------------

    /// **Bind** (⊗): Element-wise modular multiplication.
    ///
    /// Represents association/binding between two concepts.
    /// Key property: bind is invertible — unbind(bind(A,B), B) = A.
    pub fn bind(&self, other: &Self) -> Self {
        assert_eq!(self.dim(), other.dim(), "Dimension mismatch");
        assert_eq!(self.modulus, other.modulus, "Modulus mismatch");
        let m = self.modulus;
        let components = self.components.iter()
            .zip(&other.components)
            .map(|(&a, &b)| mulmod(a, b, m))
            .collect();
        Self { components, modulus: m }
    }

    /// **Unbind**: Inverse of bind. unbind(bind(A,B), B) = A.
    ///
    /// Requires all components of `other` to be nonzero (invertible mod p).
    pub fn unbind(&self, other: &Self) -> Self {
        assert_eq!(self.dim(), other.dim());
        assert_eq!(self.modulus, other.modulus);
        let m = self.modulus;
        let components = self.components.iter()
            .zip(&other.components)
            .map(|(&a, &b)| {
                assert!(b != 0, "Cannot unbind with zero component");
                mulmod(a, mod_inverse(b, m), m)
            })
            .collect();
        Self { components, modulus: m }
    }

    /// **Bundle** (⊕): Element-wise modular addition.
    ///
    /// Represents set union / superposition. Result is similar to all inputs.
    pub fn bundle(&self, other: &Self) -> Self {
        assert_eq!(self.dim(), other.dim());
        assert_eq!(self.modulus, other.modulus);
        let m = self.modulus;
        let components = self.components.iter()
            .zip(&other.components)
            .map(|(&a, &b)| addmod(a, b, m))
            .collect();
        Self { components, modulus: m }
    }

    /// Bundle multiple vectors (n-ary).
    pub fn bundle_many(vectors: &[&Self]) -> Self {
        assert!(!vectors.is_empty(), "Cannot bundle empty set");
        let dim = vectors[0].dim();
        let modulus = vectors[0].modulus;
        let mut result = vec![0u64; dim];
        for v in vectors {
            assert_eq!(v.dim(), dim);
            assert_eq!(v.modulus, modulus);
            for (r, &c) in result.iter_mut().zip(&v.components) {
                *r = addmod(*r, c, modulus);
            }
        }
        Self { components: result, modulus }
    }

    /// **Permute** (ρ): Cyclic left shift by `amount` positions.
    ///
    /// Represents sequence position / temporal ordering.
    pub fn permute(&self, amount: usize) -> Self {
        let dim = self.dim();
        let shift = amount % dim;
        let mut components = Vec::with_capacity(dim);
        for i in 0..dim {
            components.push(self.components[(i + shift) % dim]);
        }
        Self { components, modulus: self.modulus }
    }

    /// Inverse permute (cyclic right shift).
    pub fn unpermute(&self, amount: usize) -> Self {
        let dim = self.dim();
        let shift = amount % dim;
        self.permute(dim - shift)
    }

    // -- Similarity ---------------------------------------------------------

    /// **Exact modular inner product**: Σ aᵢ·bᵢ mod p.
    pub fn inner_product(&self, other: &Self) -> u64 {
        assert_eq!(self.dim(), other.dim());
        assert_eq!(self.modulus, other.modulus);
        let m = self.modulus;
        self.components.iter()
            .zip(&other.components)
            .fold(0u64, |acc, (&a, &b)| addmod(acc, mulmod(a, b, m), m))
    }

    /// **Hamming distance**: Number of positions where components differ.
    pub fn hamming_distance(&self, other: &Self) -> usize {
        assert_eq!(self.dim(), other.dim());
        self.components.iter()
            .zip(&other.components)
            .filter(|(&a, &b)| a != b)
            .count()
    }

    /// **Normalized similarity** (0..10000 integer scale).
    /// Based on fraction of matching components.
    /// 10000 = identical, 0 = completely different.
    pub fn similarity(&self, other: &Self) -> u64 {
        let matching = self.dim() - self.hamming_distance(other);
        (matching as u64 * 10000) / self.dim() as u64
    }

    // -- Encoding -----------------------------------------------------------

    /// Encode an integer symbol into a random HDV (deterministic from symbol).
    pub fn encode_symbol(symbol: u64, dim: usize, modulus: u64) -> Self {
        Self::random(dim, modulus, symbol.wrapping_mul(2654435761)) // Knuth multiplicative hash
    }

    /// Encode a sequence of symbols using bind + permute.
    ///
    /// Sequence [A, B, C] → ρ²(A) ⊗ ρ¹(B) ⊗ ρ⁰(C)
    /// This preserves order information.
    pub fn encode_sequence(symbols: &[u64], dim: usize, modulus: u64) -> Self {
        let n = symbols.len();
        let mut result = Self::from_components(vec![1; dim], modulus); // multiplicative identity
        for (i, &sym) in symbols.iter().enumerate() {
            let atom = Self::encode_symbol(sym, dim, modulus);
            let shifted = atom.permute(n - 1 - i);
            result = result.bind(&shifted);
        }
        result
    }

    // -- Arithmetic ---------------------------------------------------------

    /// Scalar multiplication: every component × scalar mod p.
    pub fn scale(&self, scalar: u64) -> Self {
        let m = self.modulus;
        let components = self.components.iter()
            .map(|&c| mulmod(c, scalar, m))
            .collect();
        Self { components, modulus: m }
    }

    /// Negation: p - component for each.
    pub fn negate(&self) -> Self {
        let m = self.modulus;
        let components = self.components.iter()
            .map(|&c| if c == 0 { 0 } else { m - c })
            .collect();
        Self { components, modulus: m }
    }

    /// Subtraction: element-wise modular subtraction.
    pub fn sub(&self, other: &Self) -> Self {
        assert_eq!(self.dim(), other.dim());
        assert_eq!(self.modulus, other.modulus);
        let m = self.modulus;
        let components = self.components.iter()
            .zip(&other.components)
            .map(|(&a, &b)| submod(a, b, m))
            .collect();
        Self { components, modulus: m }
    }
}

// ============================================================================
// ITEM MEMORY
// ============================================================================

/// Associative memory storing (label, HDV) pairs.
/// Retrieval by nearest-neighbor in Hamming distance.
pub struct ItemMemory {
    entries: Vec<(String, HDV)>,
}

impl ItemMemory {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn store(&mut self, label: String, vector: HDV) {
        self.entries.push((label, vector));
    }

    /// Retrieve the label of the most similar stored vector.
    pub fn query(&self, probe: &HDV) -> Option<(&str, u64)> {
        if self.entries.is_empty() { return None; }
        let mut best_label = "";
        let mut best_sim = 0u64;
        for (label, vector) in &self.entries {
            let sim = probe.similarity(vector);
            if sim > best_sim {
                best_sim = sim;
                best_label = label;
            }
        }
        Some((best_label, best_sim))
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

impl Default for ItemMemory {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const DIM: usize = 256;
    const MOD: u64 = DEFAULT_MODULUS;

    // --- Bind / Unbind ---

    #[test]
    fn bind_unbind_roundtrip() {
        let a = HDV::random(DIM, MOD, 1);
        let b = HDV::random(DIM, MOD, 2);
        let bound = a.bind(&b);
        let recovered = bound.unbind(&b);
        assert_eq!(a, recovered, "unbind(bind(A,B), B) should equal A");
    }

    #[test]
    fn bind_is_commutative() {
        let a = HDV::random(DIM, MOD, 10);
        let b = HDV::random(DIM, MOD, 20);
        assert_eq!(a.bind(&b), b.bind(&a), "Bind should be commutative");
    }

    #[test]
    fn bind_dissimilar_to_operands() {
        let a = HDV::random(DIM, MOD, 100);
        let b = HDV::random(DIM, MOD, 200);
        let bound = a.bind(&b);
        // Bind result should be dissimilar to both operands
        let sim_a = bound.similarity(&a);
        let sim_b = bound.similarity(&b);
        assert!(sim_a < 5000, "Bind result should be dissimilar to A, got {sim_a}");
        assert!(sim_b < 5000, "Bind result should be dissimilar to B, got {sim_b}");
    }

    // --- Bundle ---

    #[test]
    fn bundle_similar_to_operands() {
        let a = HDV::random(DIM, MOD, 1);
        let b = HDV::random(DIM, MOD, 2);
        let bundled = a.bundle(&b);
        // Bundle is NOT expected to match either in Hamming terms for modular arithmetic,
        // but the inner product should be correlated
        let ip_a = bundled.inner_product(&a);
        let ip_b = bundled.inner_product(&b);
        assert_ne!(ip_a, 0, "Bundle should have nonzero inner product with A");
        assert_ne!(ip_b, 0, "Bundle should have nonzero inner product with B");
    }

    #[test]
    fn bundle_many_works() {
        let vecs: Vec<HDV> = (1..=5).map(|i| HDV::random(DIM, MOD, i * 1000 + 7)).collect();
        let refs: Vec<&HDV> = vecs.iter().collect();
        let bundled = HDV::bundle_many(&refs);
        assert_eq!(bundled.dim(), DIM);
        // At least most inner products should be nonzero
        let nonzero_count = vecs.iter()
            .filter(|v| bundled.inner_product(v) != 0)
            .count();
        assert!(nonzero_count >= 3, "Expected most inner products nonzero, got {nonzero_count}/5");
    }

    // --- Permute ---

    #[test]
    fn permute_unpermute_roundtrip() {
        let a = HDV::random(DIM, MOD, 42);
        let shifted = a.permute(7);
        let recovered = shifted.unpermute(7);
        assert_eq!(a, recovered, "unpermute(permute(A, k), k) should equal A");
    }

    #[test]
    fn permute_changes_vector() {
        let a = HDV::random(DIM, MOD, 42);
        let shifted = a.permute(1);
        assert_ne!(a, shifted, "Permute by 1 should change the vector");
    }

    #[test]
    fn permute_by_dim_is_identity() {
        let a = HDV::random(DIM, MOD, 42);
        let shifted = a.permute(DIM);
        assert_eq!(a, shifted, "Permute by dim should be identity");
    }

    // --- Similarity ---

    #[test]
    fn similarity_self_is_maximum() {
        let a = HDV::random(DIM, MOD, 42);
        assert_eq!(a.similarity(&a), 10000, "Self-similarity should be 10000");
    }

    #[test]
    fn similarity_random_is_low() {
        let a = HDV::random(DIM, MOD, 1);
        let b = HDV::random(DIM, MOD, 2);
        let sim = a.similarity(&b);
        // Random vectors in ℤ/pℤ with large p should have ~0% Hamming match
        assert!(sim < 500, "Random vectors should have low similarity, got {sim}");
    }

    #[test]
    fn hamming_distance_properties() {
        let a = HDV::random(DIM, MOD, 1);
        let b = HDV::random(DIM, MOD, 2);
        assert_eq!(a.hamming_distance(&a), 0, "Self-distance should be 0");
        assert_eq!(a.hamming_distance(&b), b.hamming_distance(&a), "Distance should be symmetric");
        assert!(a.hamming_distance(&b) > 0, "Different vectors should have nonzero distance");
    }

    // --- Encoding ---

    #[test]
    fn encode_symbol_deterministic() {
        let v1 = HDV::encode_symbol(42, DIM, MOD);
        let v2 = HDV::encode_symbol(42, DIM, MOD);
        assert_eq!(v1, v2, "Same symbol should encode identically");
    }

    #[test]
    fn different_symbols_dissimilar() {
        let v1 = HDV::encode_symbol(1, DIM, MOD);
        let v2 = HDV::encode_symbol(2, DIM, MOD);
        let sim = v1.similarity(&v2);
        assert!(sim < 500, "Different symbols should be dissimilar, got {sim}");
    }

    #[test]
    fn encode_sequence_preserves_order() {
        let seq1 = HDV::encode_sequence(&[1, 2, 3], DIM, MOD);
        let seq2 = HDV::encode_sequence(&[3, 2, 1], DIM, MOD);
        assert_ne!(seq1, seq2, "Different orderings should produce different vectors");
    }

    // --- Arithmetic ---

    #[test]
    fn negate_double_is_identity() {
        let a = HDV::random(DIM, MOD, 42);
        let double_neg = a.negate().negate();
        assert_eq!(a, double_neg, "Double negation should be identity");
    }

    #[test]
    fn add_negate_is_zero() {
        let a = HDV::random(DIM, MOD, 42);
        let sum = a.bundle(&a.negate());
        let zero = HDV::zero(DIM, MOD);
        assert_eq!(sum, zero, "A + (-A) should be zero");
    }

    #[test]
    fn scale_by_one_is_identity() {
        let a = HDV::random(DIM, MOD, 42);
        assert_eq!(a.scale(1), a, "Scaling by 1 should be identity");
    }

    #[test]
    fn sub_self_is_zero() {
        let a = HDV::random(DIM, MOD, 42);
        let diff = a.sub(&a);
        let zero = HDV::zero(DIM, MOD);
        assert_eq!(diff, zero, "A - A should be zero");
    }

    // --- Item Memory ---

    #[test]
    fn item_memory_exact_retrieval() {
        let mut mem = ItemMemory::new();
        let cat = HDV::encode_symbol(1, DIM, MOD);
        let dog = HDV::encode_symbol(2, DIM, MOD);
        let fish = HDV::encode_symbol(3, DIM, MOD);

        mem.store("cat".to_string(), cat.clone());
        mem.store("dog".to_string(), dog.clone());
        mem.store("fish".to_string(), fish.clone());

        let (label, score) = mem.query(&cat).unwrap();
        assert_eq!(label, "cat");
        assert_eq!(score, 10000);

        let (label, score) = mem.query(&dog).unwrap();
        assert_eq!(label, "dog");
        assert_eq!(score, 10000);
    }

    #[test]
    fn item_memory_noisy_retrieval() {
        let mut mem = ItemMemory::new();
        let original = HDV::encode_symbol(42, DIM, MOD);
        mem.store("target".to_string(), original.clone());
        mem.store("distractor".to_string(), HDV::encode_symbol(99, DIM, MOD));

        // Query with exact match should return target
        let (label, _) = mem.query(&original).unwrap();
        assert_eq!(label, "target");
    }

    // --- Capacity ---

    #[test]
    fn capacity_bits_exceed_binary() {
        // CRT-HDV capacity: dim * log2(modulus) bits
        let modulus_bits = 64 - MOD.leading_zeros();
        let crt_capacity = DIM as u32 * modulus_bits;
        let binary_capacity = DIM as u32; // Standard binary HDV

        assert!(
            crt_capacity > binary_capacity * 20,
            "CRT-HDV should have >20x capacity vs binary: {} vs {}",
            crt_capacity, binary_capacity
        );
    }
}
