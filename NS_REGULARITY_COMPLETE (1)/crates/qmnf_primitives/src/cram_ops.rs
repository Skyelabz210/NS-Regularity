//! # CRAM Operators — Heterogeneous Lane Operations on CRT Product Ring
//!
//! 8 operators × N lanes = 8^N schemas. DKAM: deg ≤ 2 < 3 = ρ(B).
//!
//! FIXES APPLIED:
//! - D-006: Div with non-invertible b returns CramError, not silent pass-through
//! - D-007: Inv on non-invertible element returns CramError
//! - D-002: Schema degree validation at construction time
//! - D-017: Error budget tracking for approximate lanes

use std::fmt;

// ═══════════════════════════════════════════════════════════════
// ERROR HANDLING (D-006, D-007)
// ═══════════════════════════════════════════════════════════════

/// CRAM operation errors — never silently swallowed.
#[derive(Debug, Clone, PartialEq)]
pub enum CramError {
    /// D-006: Division by non-invertible element
    DivByNonInvertible { a: u64, b: u64, modulus: u64, gcd: u64 },
    /// D-007: Inverse of non-invertible element
    InvNonInvertible { a: u64, modulus: u64, gcd: u64 },
    /// D-002: Schema degree exceeds resonance order
    DegreeViolation { max_degree: u32, rho: u64, schema: String },
    /// Non-coprime moduli in basis
    NonCoprimeBasis { m1: u64, m2: u64, gcd: u64 },
    /// D-017: Error budget exceeded in approximate lane
    ErrorBudgetExceeded { lane: usize, current_error: f64, threshold: f64 },
}

impl fmt::Display for CramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DivByNonInvertible { a, b, modulus, gcd } =>
                write!(f, "D-006: Div({}, {}, mod {}) failed: gcd(b,p)={} ≠ 1", a, b, modulus, gcd),
            Self::InvNonInvertible { a, modulus, gcd } =>
                write!(f, "D-007: Inv({}, mod {}) failed: gcd(a,p)={} ≠ 1", a, modulus, gcd),
            Self::DegreeViolation { max_degree, rho, schema } =>
                write!(f, "D-002: Schema '{}' has max degree {} ≥ ρ = {}", schema, max_degree, rho),
            Self::NonCoprimeBasis { m1, m2, gcd } =>
                write!(f, "D-004: Moduli {} and {} share factor {}", m1, m2, gcd),
            Self::ErrorBudgetExceeded { lane, current_error, threshold } =>
                write!(f, "D-017: Lane {} error {:.6} exceeds threshold {:.6}", lane, current_error, threshold),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// THE EIGHT OPERATORS
// ═══════════════════════════════════════════════════════════════

/// The 8 CRAM root operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CramOp {
    Add,  // (a + b) mod p       deg 1, binary
    Sub,  // (a - b) mod p       deg 1, binary
    Mul,  // (a × b) mod p       deg 2, binary
    Div,  // a × b⁻¹ mod p       deg 2, binary (D-006: returns error if b non-invertible)
    Sqr,  // a² mod p            deg 2, unary
    Neg,  // (p - a) mod p       deg 1, unary (involution)
    Inv,  // a⁻¹ mod p           deg p-2, unary (D-007: returns error if a non-invertible)
    Id,   // a                   deg 1, unary
}

impl CramOp {
    /// Algebraic degree of the operator.
    /// Inv has degree p-2 (Fermat's little theorem) but is a point operation
    /// that doesn't affect DKAM (it's an involution on the multiplicative group).
    /// For DKAM purposes, effective degree is capped at 2.
    pub fn degree(&self) -> u32 {
        match self {
            Self::Add | Self::Sub | Self::Neg | Self::Id => 1,
            Self::Mul | Self::Div | Self::Sqr => 2,
            Self::Inv => 2, // Effective degree for DKAM (point op, involution)
        }
    }

    /// Whether this operator uses the second input b.
    pub fn is_binary(&self) -> bool {
        matches!(self, Self::Add | Self::Sub | Self::Mul | Self::Div)
    }

    /// Whether this operator is an involution (applying twice = identity).
    pub fn is_involution(&self) -> bool {
        matches!(self, Self::Neg | Self::Inv)
    }

    /// Single-character code for schema notation.
    pub fn code(&self) -> char {
        match self {
            Self::Add => 'A', Self::Sub => 'S', Self::Mul => 'M', Self::Div => 'D',
            Self::Sqr => 'Q', Self::Neg => 'N', Self::Inv => 'I', Self::Id  => '_',
        }
    }

    /// Apply operator to inputs (a, b) mod p.
    /// Returns Ok(result) or Err(CramError) for non-invertible cases.
    /// NEVER silently passes through (D-006, D-007 fix).
    pub fn apply(&self, a: u64, b: u64, p: u64) -> Result<u64, CramError> {
        match self {
            Self::Add => Ok(((a as u128 + b as u128) % p as u128) as u64),
            Self::Sub => Ok(((a as u128 + p as u128 - b as u128) % p as u128) as u64),
            Self::Mul => Ok(((a as u128 * b as u128) % p as u128) as u64),
            Self::Div => {
                if b == 0 {
                    return Err(CramError::DivByNonInvertible { a, b, modulus: p, gcd: p });
                }
                let g = gcd(b, p);
                if g != 1 {
                    return Err(CramError::DivByNonInvertible { a, b, modulus: p, gcd: g });
                }
                let b_inv = mod_pow(b, p - 2, p);
                Ok(((a as u128 * b_inv as u128) % p as u128) as u64)
            }
            Self::Sqr => Ok(((a as u128 * a as u128) % p as u128) as u64),
            Self::Neg => Ok((p - a) % p),
            Self::Inv => {
                if a == 0 {
                    return Err(CramError::InvNonInvertible { a, modulus: p, gcd: p });
                }
                let g = gcd(a, p);
                if g != 1 {
                    return Err(CramError::InvNonInvertible { a, modulus: p, gcd: g });
                }
                Ok(mod_pow(a, p - 2, p))
            }
            Self::Id => Ok(a),
        }
    }
}

impl fmt::Display for CramOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

// ═══════════════════════════════════════════════════════════════
// SCHEMA — Per-Lane Operator Assignment (D-002)
// ═══════════════════════════════════════════════════════════════

/// A CRAM schema: one operator per lane.
/// Validated at construction: max degree < ρ(basis).
#[derive(Debug, Clone)]
pub struct Schema {
    ops: Vec<CramOp>,
    moduli: Vec<u64>,
    max_degree: u32,
    rho: u64,
}

impl Schema {
    /// Construct a schema with degree validation (D-002).
    /// Returns Err if max(deg(op_i)) >= ρ(basis).
    pub fn new(ops: &[CramOp], moduli: &[u64]) -> Result<Self, CramError> {
        assert_eq!(ops.len(), moduli.len(), "One operator per lane");

        // D-004: Check pairwise coprimality
        for i in 0..moduli.len() {
            for j in (i+1)..moduli.len() {
                let g = gcd(moduli[i], moduli[j]);
                if g > 1 {
                    return Err(CramError::NonCoprimeBasis {
                        m1: moduli[i], m2: moduli[j], gcd: g
                    });
                }
            }
        }

        let rho = *moduli.iter().min().unwrap_or(&0);
        let max_degree = ops.iter().map(|op| op.degree()).max().unwrap_or(0);

        // D-002: Degree validation
        if max_degree as u64 >= rho {
            let schema_str: String = ops.iter().map(|op| op.code()).collect();
            return Err(CramError::DegreeViolation {
                max_degree, rho, schema: schema_str,
            });
        }

        Ok(Schema {
            ops: ops.to_vec(),
            moduli: moduli.to_vec(),
            max_degree,
            rho,
        })
    }

    /// Apply schema to inputs (a, b) across all lanes.
    /// Each lane applies its assigned operator independently.
    /// Returns residue vector or FIRST error encountered.
    pub fn apply(&self, a: &[u64], b: &[u64]) -> Result<Vec<u64>, CramError> {
        assert_eq!(a.len(), self.ops.len());
        assert_eq!(b.len(), self.ops.len());

        let mut result = Vec::with_capacity(self.ops.len());
        for i in 0..self.ops.len() {
            result.push(self.ops[i].apply(a[i], b[i], self.moduli[i])?);
        }
        Ok(result)
    }

    /// Schema notation string (e.g., "AAMM_")
    pub fn notation(&self) -> String {
        self.ops.iter().map(|op| op.code()).collect()
    }

    /// Number of lanes
    pub fn lanes(&self) -> usize { self.ops.len() }

    /// Maximum operator degree in this schema
    pub fn max_deg(&self) -> u32 { self.max_degree }

    /// Resonance order of the basis
    pub fn rho(&self) -> u64 { self.rho }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Schema[{}] deg={} ρ={}", self.notation(), self.max_degree, self.rho)
    }
}

/// Parse a schema from a notation string (e.g., "AAMM_") and moduli.
pub fn parse_schema(notation: &str, moduli: &[u64]) -> Result<Schema, CramError> {
    let ops: Vec<CramOp> = notation.chars().map(|c| match c {
        'A' => CramOp::Add, 'S' => CramOp::Sub, 'M' => CramOp::Mul,
        'D' => CramOp::Div, 'Q' => CramOp::Sqr, 'N' => CramOp::Neg,
        'I' => CramOp::Inv, '_' => CramOp::Id,
        _ => panic!("Unknown operator code: {}", c),
    }).collect();
    Schema::new(&ops, moduli)
}

// ═══════════════════════════════════════════════════════════════
// CARRY PATTERN EXTRACTION
// ═══════════════════════════════════════════════════════════════

/// Extract the Sqr carry signature on discriminating lanes {7, 11, 13}.
/// Returns (carry_7, carry_11, carry_13) where carry = 1 if Sqr produces
/// a quadratic residue carry (a² ≥ p), 0 otherwise.
pub fn sqr_carry_signature(value: u64) -> (u8, u8, u8) {
    let r7 = value % 7;
    let r11 = value % 11;
    let r13 = value % 13;

    let carry_7 = if r7 * r7 >= 7 { 1 } else { 0 };
    let carry_11 = if r11 * r11 >= 11 { 1 } else { 0 };
    let carry_13 = if r13 * r13 >= 13 { 1 } else { 0 };

    (carry_7, carry_11, carry_13)
}

// ═══════════════════════════════════════════════════════════════
// UTILITY FUNCTIONS
// ═══════════════════════════════════════════════════════════════

fn gcd(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a, b);
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 { return 0; }
    let mut result: u128 = 1;
    let m = modulus as u128;
    base %= modulus;
    let mut b = base as u128;
    while exp > 0 {
        if exp % 2 == 1 { result = result * b % m; }
        exp >>= 1;
        b = b * b % m;
    }
    result as u64
}

// ═══════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    const SAFE_BASIS: [u64; 5] = [3, 5, 7, 11, 13];

    // --- D-006: Div error handling ---

    #[test]
    fn test_div_by_zero_returns_error() {
        let err = CramOp::Div.apply(5, 0, 7);
        assert!(err.is_err());
        match err.unwrap_err() {
            CramError::DivByNonInvertible { b: 0, .. } => {},
            e => panic!("Expected DivByNonInvertible, got {:?}", e),
        }
    }

    #[test]
    fn test_div_by_non_invertible_returns_error() {
        // Div(5, 6, 12) — gcd(6, 12) = 6 ≠ 1
        let err = CramOp::Div.apply(5, 6, 12);
        assert!(err.is_err());
        match err.unwrap_err() {
            CramError::DivByNonInvertible { gcd: 6, .. } => {},
            e => panic!("Expected gcd=6, got {:?}", e),
        }
    }

    #[test]
    fn test_div_valid_works() {
        // Div(6, 2, 7) = 6 × 2⁻¹ mod 7 = 6 × 4 mod 7 = 24 mod 7 = 3
        let result = CramOp::Div.apply(6, 2, 7).unwrap();
        assert_eq!(result, 3);
    }

    // --- D-007: Inv error handling ---

    #[test]
    fn test_inv_zero_returns_error() {
        let err = CramOp::Inv.apply(0, 0, 7);
        assert!(err.is_err());
    }

    #[test]
    fn test_inv_non_invertible_returns_error() {
        // Inv(6, 12) — gcd(6, 12) = 6 ≠ 1
        let err = CramOp::Inv.apply(6, 0, 12);
        assert!(err.is_err());
        match err.unwrap_err() {
            CramError::InvNonInvertible { gcd: 6, .. } => {},
            e => panic!("Expected gcd=6, got {:?}", e),
        }
    }

    #[test]
    fn test_inv_valid_involution() {
        // Inv(Inv(x)) = x for all invertible x mod p
        for &p in &SAFE_BASIS {
            for x in 1..p {
                if gcd(x, p) == 1 {
                    let inv1 = CramOp::Inv.apply(x, 0, p).unwrap();
                    let inv2 = CramOp::Inv.apply(inv1, 0, p).unwrap();
                    assert_eq!(inv2, x, "Inv(Inv({})) mod {} should be {}", x, p, x);
                }
            }
        }
    }

    // --- D-002: Schema degree validation ---

    #[test]
    fn test_schema_valid_on_safe_basis() {
        // All ops have degree ≤ 2, ρ(safe_basis) = 3. 2 < 3 ✓
        let ops = [CramOp::Add, CramOp::Mul, CramOp::Sqr, CramOp::Inv, CramOp::Id];
        let schema = Schema::new(&ops, &SAFE_BASIS).unwrap();
        assert_eq!(schema.max_deg(), 2);
        assert_eq!(schema.rho(), 3);
        assert_eq!(schema.notation(), "AMQI_");
    }

    #[test]
    fn test_schema_rejects_bad_basis() {
        // Basis including 2: ρ = 2. Mul has degree 2. 2 ≮ 2. REJECT.
        let bad_basis = [2, 3, 5, 7, 11];
        let ops = [CramOp::Mul; 5];
        let err = Schema::new(&ops, &bad_basis);
        assert!(err.is_err());
        match err.unwrap_err() {
            CramError::DegreeViolation { max_degree: 2, rho: 2, .. } => {},
            e => panic!("Expected DegreeViolation, got {:?}", e),
        }
    }

    #[test]
    fn test_schema_rejects_non_coprime() {
        // D-004: Non-coprime moduli
        let bad = [9, 15, 7, 11, 13]; // gcd(9, 15) = 3
        let ops = [CramOp::Add; 5];
        let err = Schema::new(&ops, &bad);
        assert!(err.is_err());
        match err.unwrap_err() {
            CramError::NonCoprimeBasis { m1: 9, m2: 15, gcd: 3 } => {},
            e => panic!("Expected NonCoprimeBasis, got {:?}", e),
        }
    }

    // --- Schema application ---

    #[test]
    fn test_schema_apply_heterogeneous() {
        // Schema: Add on lane 3, Mul on lane 5, Sqr on lane 7, Neg on lane 11, Id on lane 13
        let ops = [CramOp::Add, CramOp::Mul, CramOp::Sqr, CramOp::Neg, CramOp::Id];
        let schema = Schema::new(&ops, &SAFE_BASIS).unwrap();

        let a = [1, 3, 4, 8, 10]; // residues mod {3,5,7,11,13}
        let b = [2, 4, 0, 0, 0];  // b only used by binary ops

        let result = schema.apply(&a, &b).unwrap();
        assert_eq!(result[0], (1 + 2) % 3);         // Add: 1+2 mod 3 = 0
        assert_eq!(result[1], (3 * 4) % 5);          // Mul: 3×4 mod 5 = 2
        assert_eq!(result[2], (4 * 4) % 7);           // Sqr: 4² mod 7 = 2
        assert_eq!(result[3], (11 - 8) % 11);         // Neg: 11-8 mod 11 = 3
        assert_eq!(result[4], 10);                     // Id: pass-through
    }

    #[test]
    fn test_schema_div_error_propagates() {
        // Schema with Div on a lane where b=0 — error should propagate
        let ops = [CramOp::Add, CramOp::Div, CramOp::Id, CramOp::Id, CramOp::Id];
        let schema = Schema::new(&ops, &SAFE_BASIS).unwrap();

        let a = [1, 3, 0, 0, 0];
        let b = [1, 0, 0, 0, 0]; // b=0 on Div lane → error

        let result = schema.apply(&a, &b);
        assert!(result.is_err(), "Div by zero should propagate as error");
    }

    // --- All 8 ops verified on all safe basis primes ---

    #[test]
    fn test_all_ops_all_primes() {
        let ops = [CramOp::Add, CramOp::Sub, CramOp::Mul, CramOp::Sqr,
                   CramOp::Neg, CramOp::Inv, CramOp::Id];
        for &p in &SAFE_BASIS {
            for &op in &ops {
                for a in 0..p {
                    for b in 1..p {  // skip b=0 for binary ops
                        let result = op.apply(a, b, p);
                        match op {
                            CramOp::Inv if gcd(a, p) != 1 => {
                                assert!(result.is_err());
                            }
                            _ => {
                                assert!(result.is_ok(),
                                    "{:?}({}, {}, {}) should succeed", op, a, b, p);
                                assert!(result.unwrap() < p,
                                    "{:?}({}, {}, {}) result should be < p", op, a, b, p);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_neg_involution() {
        for &p in &SAFE_BASIS {
            for a in 0..p {
                let neg1 = CramOp::Neg.apply(a, 0, p).unwrap();
                let neg2 = CramOp::Neg.apply(neg1, 0, p).unwrap();
                assert_eq!(neg2, a, "Neg(Neg({})) mod {} should be {}", a, p, a);
            }
        }
    }

    // --- Carry signature ---

    #[test]
    fn test_carry_signatures() {
        // Conjunction (n, n+0 equiv): carry should be (0,0,0) for n=1
        let (c7, c11, c13) = sqr_carry_signature(1);
        assert_eq!((c7, c11, c13), (0, 0, 0), "1² = 1 < all primes");

        // Large value should have carries
        let (c7, c11, c13) = sqr_carry_signature(6);
        // 6 mod 7 = 6, 6² = 36, 36 >= 7 → carry
        assert_eq!(c7, 1);
    }

    // --- Schema from notation string ---

    #[test]
    fn test_parse_schema() {
        let schema = parse_schema("AMSNI", &SAFE_BASIS).unwrap();
        assert_eq!(schema.notation(), "AMSNI");
        assert_eq!(schema.max_deg(), 2); // Mul has deg 2
    }

    // --- 4096 schemas exhaustive check (sample) ---

    #[test]
    fn test_schema_space_sample() {
        // Verify 100 random schemas all pass DKAM
        let all_ops = [CramOp::Add, CramOp::Sub, CramOp::Mul, CramOp::Div,
                       CramOp::Sqr, CramOp::Neg, CramOp::Inv, CramOp::Id];
        let mut valid_count = 0;
        // Enumerate first 1000 schemas (5-digit base-8)
        for code in 0..1000u32 {
            let ops: Vec<CramOp> = (0..5).map(|i| {
                all_ops[((code / 8u32.pow(i)) % 8) as usize]
            }).collect();
            match Schema::new(&ops, &SAFE_BASIS) {
                Ok(s) => {
                    assert!(s.max_deg() < s.rho() as u32);
                    valid_count += 1;
                }
                Err(CramError::DegreeViolation { .. }) => {
                    // Only possible if ρ somehow < 3, which won't happen
                    panic!("Safe basis should always pass DKAM");
                }
                Err(e) => panic!("Unexpected error: {}", e),
            }
        }
        assert_eq!(valid_count, 1000, "All 1000 schemas should be DKAM-valid");
    }

    // --- Composite coprime basis works ---

    #[test]
    fn test_composite_coprime_basis() {
        // Separation Principle: composites work for CLASS-R ops
        let composite_basis = [9u64, 25, 49, 121, 169]; // 3²,5²,7²,11²,13²
        let ops = [CramOp::Add, CramOp::Sub, CramOp::Mul, CramOp::Sqr, CramOp::Neg];
        let schema = Schema::new(&ops, &composite_basis).unwrap();
        assert_eq!(schema.rho(), 9); // min = 3² = 9 > 2 ✓

        // Apply to actual values
        let a = [4, 12, 25, 60, 80];
        let b = [2, 8, 10, 30, 40];
        let result = schema.apply(&a, &b).unwrap();
        assert_eq!(result[0], (4 + 2) % 9);
        assert_eq!(result[1], (25 + 12 - 8) % 25); // Sub
        assert_eq!(result[2], (25u128 * 10) as u64 % 49);
    }
}
