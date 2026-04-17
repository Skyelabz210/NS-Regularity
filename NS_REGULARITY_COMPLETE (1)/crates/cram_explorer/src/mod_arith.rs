//! # Modular Arithmetic Primitives
//!
//! Zero floating-point. All operations exact integer.
//! Foundation layer for the entire CRAM explorer toolbox.

/// Extended GCD: returns (gcd, x, y) where a*x + b*y = gcd
pub fn extended_gcd(a: i128, b: i128) -> (i128, i128, i128) {
    if a == 0 {
        return (b, 0, 1);
    }
    let (g, x1, y1) = extended_gcd(b % a, a);
    (g, y1 - (b / a) * x1, x1)
}

/// Modular inverse of a mod m. Panics if gcd(a,m) != 1.
pub fn mod_inv(a: i64, m: i64) -> i64 {
    let a_mod = ((a as i128 % m as i128) + m as i128) % m as i128;
    let (g, x, _) = extended_gcd(a_mod, m as i128);
    assert_eq!(g, 1, "mod_inv: gcd({}, {}) != 1", a, m);
    ((x % m as i128 + m as i128) % m as i128) as i64
}

/// Safe modular reduction (handles negatives)
#[inline(always)]
pub fn modd(a: i64, m: i64) -> i64 {
    ((a % m) + m) % m
}

/// Safe modular reduction for i128
#[inline(always)]
pub fn modd128(a: i128, m: i64) -> i64 {
    let m = m as i128;
    ((a % m + m) % m) as i64
}

/// Modular multiplication avoiding overflow via i128
#[inline(always)]
pub fn mulmod(a: i64, b: i64, m: i64) -> i64 {
    ((a as i128 * b as i128) % m as i128) as i64
}

/// Modular exponentiation
pub fn powmod(mut base: i64, mut exp: u64, m: i64) -> i64 {
    let mut result = 1i64;
    base = modd(base, m);
    while exp > 0 {
        if exp & 1 == 1 {
            result = mulmod(result, base, m);
        }
        exp >>= 1;
        base = mulmod(base, base, m);
    }
    result
}

/// GCD for u64
pub fn gcd_u64(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd_u64(b, a % b) }
}

/// GCD for i64 (absolute value)
pub fn gcd_i64(a: i64, b: i64) -> i64 {
    let (a, b) = (a.unsigned_abs(), b.unsigned_abs());
    gcd_u64(a, b) as i64
}

/// 2-adic valuation of n
pub fn v2(n: i64) -> u32 {
    if n == 0 { return 64; }
    n.unsigned_abs().trailing_zeros()
}

/// Exact rational number (a/b with b > 0, always reduced)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Frac {
    pub num: i64,
    pub den: i64,
}

impl Frac {
    pub fn new(n: i64, d: i64) -> Self {
        assert!(d != 0, "Frac: zero denominator");
        let sign = if d < 0 { -1 } else { 1 };
        let n = n * sign;
        let d = (d * sign).unsigned_abs() as i64;
        let g = gcd_i64(n.abs(), d);
        Frac { num: n / g, den: d / g }
    }

    pub fn zero() -> Self { Frac { num: 0, den: 1 } }
    pub fn one() -> Self { Frac { num: 1, den: 1 } }

    pub fn add(self, other: Self) -> Self {
        Frac::new(
            self.num * other.den + other.num * self.den,
            self.den * other.den,
        )
    }

    pub fn sub(self, other: Self) -> Self {
        Frac::new(
            self.num * other.den - other.num * self.den,
            self.den * other.den,
        )
    }

    pub fn mul(self, other: Self) -> Self {
        Frac::new(self.num * other.num, self.den * other.den)
    }
}

impl std::fmt::Display for Frac {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.den == 1 { write!(f, "{}", self.num) }
        else { write!(f, "{}/{}", self.num, self.den) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_inv() {
        assert_eq!(mulmod(mod_inv(2, 105), 2, 105), 1);
        assert_eq!(mulmod(mod_inv(105, 16), modd(105, 16), 16), 1);
        assert_eq!(mod_inv(3, 7), 5); // 3*5 = 15 ≡ 1 mod 7
    }

    #[test]
    fn test_frac() {
        let a = Frac::new(1264, 1680);
        assert_eq!(a.num, 79);
        assert_eq!(a.den, 105);
    }

    #[test]
    fn test_powmod() {
        assert_eq!(powmod(3, 4, 100), 81);
        assert_eq!(powmod(2, 10, 1000), 24);
    }
}
