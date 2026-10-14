//! F_p² = F_p[√δ] — The Dual-Substrate Element
//!
//! a + b√δ where:
//!   a = the RESIDUE (physics lives here)
//!   b = the AMPLITUDE (continuum information lives here)
//!   norm = a² - δb² (the BRIDGE — always an integer in F_p)
//!
//! Frobenius σ(a+b√δ) = a-b√δ: flips amplitude, preserves residue.

fn mulmod(a: u64, b: u64, p: u64) -> u64 { ((a as u128 * b as u128) % p as u128) as u64 }
fn addmod(a: u64, b: u64, p: u64) -> u64 { ((a as u128 + b as u128) % p as u128) as u64 }
fn submod(a: u64, b: u64, p: u64) -> u64 { if a >= b { a - b } else { p - (b - a) } }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fp2 {
    pub a: u64,     // Residue: physics
    pub b: u64,     // Amplitude: continuum
    pub p: u64,     // Prime modulus
    pub delta: u64, // Quadratic non-residue
}

impl Fp2 {
    pub fn new(a: u64, b: u64, p: u64, delta: u64) -> Self {
        Fp2 { a: a % p, b: b % p, p, delta: delta % p }
    }

    pub fn from_int(x: u64, p: u64, delta: u64) -> Self {
        Fp2 { a: x % p, b: 0, p, delta: delta % p }
    }

    /// Norm: a² - δb² mod p. Always in F_p. The BRIDGE quantity.
    pub fn norm(&self) -> u64 {
        let a2 = mulmod(self.a, self.a, self.p);
        let db2 = mulmod(self.delta, mulmod(self.b, self.b, self.p), self.p);
        submod(a2, db2, self.p)
    }

    /// Frobenius: σ(a + b√δ) = a - b√δ. Flips b, preserves a.
    pub fn conjugate(&self) -> Fp2 {
        Fp2 { a: self.a, b: if self.b == 0 { 0 } else { self.p - self.b },
               p: self.p, delta: self.delta }
    }

    /// Addition: componentwise
    pub fn add(&self, other: &Fp2) -> Fp2 {
        debug_assert_eq!(self.p, other.p);
        Fp2::new(addmod(self.a, other.a, self.p),
                 addmod(self.b, other.b, self.p),
                 self.p, self.delta)
    }

    /// Subtraction: componentwise
    pub fn sub(&self, other: &Fp2) -> Fp2 {
        debug_assert_eq!(self.p, other.p);
        Fp2::new(submod(self.a, other.a, self.p),
                 submod(self.b, other.b, self.p),
                 self.p, self.delta)
    }

    /// Multiplication: (a+b√δ)(c+d√δ) = (ac+bdδ) + (ad+bc)√δ
    pub fn mul(&self, other: &Fp2) -> Fp2 {
        debug_assert_eq!(self.p, other.p);
        let p = self.p;
        let re = addmod(mulmod(self.a, other.a, p),
                        mulmod(mulmod(self.delta, self.b, p), other.b, p), p);
        let im = addmod(mulmod(self.a, other.b, p), mulmod(self.b, other.a, p), p);
        Fp2 { a: re, b: im, p, delta: self.delta }
    }

    /// Square: (a+b√δ)² = (a²+δb²) + (2ab)√δ
    pub fn sqr(&self) -> Fp2 {
        let p = self.p;
        let re = addmod(mulmod(self.a, self.a, p),
                        mulmod(self.delta, mulmod(self.b, self.b, p), p), p);
        let im = mulmod(2 % p, mulmod(self.a, self.b, p), p);
        Fp2 { a: re, b: im, p, delta: self.delta }
    }

    /// Inverse: 1/(a+b√δ) = (a-b√δ)/(a²-δb²)
    pub fn inv(&self) -> Fp2 {
        let n = self.norm();
        assert_ne!(n, 0, "Cannot invert zero-norm element");
        let n_inv = Self::modpow(n, self.p - 2, self.p);
        let conj = self.conjugate();
        Fp2::new(mulmod(conj.a, n_inv, self.p),
                 mulmod(conj.b, n_inv, self.p),
                 self.p, self.delta)
    }

    /// Modular exponentiation
    fn modpow(mut base: u64, mut exp: u64, m: u64) -> u64 {
        let mut result = 1u64;
        base %= m;
        while exp > 0 {
            if exp & 1 == 1 { result = mulmod(result, base, m); }
            exp >>= 1;
            base = mulmod(base, base, m);
        }
        result
    }

    /// Tonelli-Shanks: find r such that r² ≡ x (mod p), or None.
    pub fn sqrt_mod(x: u64, p: u64) -> Option<u64> {
        let x = x % p;
        if x == 0 { return Some(0); }
        if Self::modpow(x, (p - 1) / 2, p) != 1 { return None; }
        
        let mut q = p - 1;
        let mut s = 0u32;
        while q % 2 == 0 { q /= 2; s += 1; }
        
        if s == 1 { return Some(Self::modpow(x, (p + 1) / 4, p)); }
        
        let mut z = 2u64;
        while Self::modpow(z, (p - 1) / 2, p) != p - 1 { z += 1; }
        
        let mut m = s;
        let mut c = Self::modpow(z, q, p);
        let mut t = Self::modpow(x, q, p);
        let mut r = Self::modpow(x, (q + 1) / 2, p);
        
        loop {
            if t == 1 { return Some(r); }
            let mut i = 1u32;
            let mut temp = mulmod(t, t, p);
            while temp != 1 { temp = mulmod(temp, temp, p); i += 1; }
            let b = Self::modpow(c, 1u64 << (m - i - 1), p);
            m = i;
            c = mulmod(b, b, p);
            t = mulmod(t, c, p);
            r = mulmod(r, b, p);
        }
    }

    /// Find smallest quadratic non-residue mod p
    pub fn find_qnr(p: u64) -> u64 {
        for d in 2..p {
            if Self::modpow(d, (p - 1) / 2, p) == p - 1 { return d; }
        }
        panic!("No QNR found for p={}", p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fp2_basic_arithmetic() {
        for p in [7, 11, 13] {
            let delta = Fp2::find_qnr(p);
            let one = Fp2::new(1, 0, p, delta);
            let prod = one.mul(&one);
            assert_eq!(prod.a, 1);
            assert_eq!(prod.b, 0);

            // (√δ)² = δ
            let sqrt_d = Fp2::new(0, 1, p, delta);
            let sq = sqrt_d.sqr();
            assert_eq!(sq.a, delta % p, "p={p}: (√δ)² should be δ");
            assert_eq!(sq.b, 0);

            // x * x⁻¹ = 1
            let x = Fp2::new(3 % p, 2 % p, p, delta);
            if x.norm() != 0 {
                let inv = x.inv();
                let prod = x.mul(&inv);
                assert_eq!(prod.a, 1, "p={p}: x*x⁻¹ should be 1");
                assert_eq!(prod.b, 0);
            }

            // norm = x * conjugate(x)
            let conj = x.conjugate();
            let norm_prod = x.mul(&conj);
            assert_eq!(norm_prod.a, x.norm(), "p={p}: x*σ(x) should be norm");
            assert_eq!(norm_prod.b, 0);
        }
    }

    #[test]
    fn fp2_ring_homomorphism() {
        for p in [7u64, 11, 13] {
            let delta = Fp2::find_qnr(p);
            let (x_val, y_val) = (100u64, 42u64);
            let fx = Fp2::from_int(x_val, p, delta);
            let fy = Fp2::from_int(y_val, p, delta);

            let fsum = fx.add(&fy);
            assert_eq!(fsum.a, (x_val + y_val) % p);
            assert_eq!(fsum.b, 0);

            let fprod = fx.mul(&fy);
            assert_eq!(fprod.a, (x_val * y_val) % p);
            assert_eq!(fprod.b, 0);
        }
    }

    #[test]
    fn nested_radicals_exist() {
        let ntt_primes = [769u64, 929, 1153, 1217];
        for p in ntt_primes {
            let s2 = Fp2::sqrt_mod(2, p).expect("√2 should exist");
            assert_eq!((s2 * s2) % p, 2 % p, "p={p}: (√2)² should be 2");

            let inner = (2 + s2) % p;
            let s_n16 = Fp2::sqrt_mod(inner, p).expect("√(2+√2) should exist");
            assert_eq!((s_n16 * s_n16) % p, inner, "p={p}: nested N=16");

            let inner2 = (2 + s_n16) % p;
            let s_n32 = Fp2::sqrt_mod(inner2, p).expect("√(2+√(2+√2)) should exist");
            assert_eq!((s_n32 * s_n32) % p, inner2, "p={p}: nested N=32");
        }
    }
}
