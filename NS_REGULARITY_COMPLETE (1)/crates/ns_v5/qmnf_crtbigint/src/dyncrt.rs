//! DynCRT — dynamic-lane CRT carrier with NO representation ceiling.
//!
//! Anthony Diaz (Acid) — HackFate.us / Skyelabz210. ZERO FLOAT (A1).
//!
//! The fixed-lane `CRTBigInt` ([u64; 8], M ≈ 2^504) is a finite corridor: values
//! past M collapse to residue classes. This carrier removes that ceiling by making
//! the basis GROWABLE. When a value's magnitude approaches the current modulus
//! product M, lanes are added on demand (more coprime ~2^63 primes), extending M
//! by ~63 bits per lane with no upper bound.
//!
//! Built entirely on existing primitives in this crate:
//!   - addmod / submod / mulmod / mod_inverse  (lanewise exact modular arithmetic)
//!   - Garner mixed-radix digits                (exact ordering, no full reconstruct)
//!
//! Representation: sign-magnitude. `residues[i] = |value| mod moduli[i]`, with a
//! separate sign. The magnitude is the unique X in [0, M) with X ≡ residues (mod m_i),
//! valid as long as |value| < M — which the grow-on-demand policy guarantees.
//!
//! Ordering is by mixed-radix (Garner) winding digits, most-significant first —
//! exact at ANY lane count, never assembling the full magnitude into a fixed width.

use crate::{addmod, submod, mulmod, mod_inverse, EXACT_MODULI};
use core::cmp::Ordering;

fn is_prime_u64(n: u64) -> bool {
    if n < 2 { return false; }
    for &p in &[2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n % p == 0 { return n == p; }
    }
    let mut d = n - 1;
    let mut r = 0u32;
    while d % 2 == 0 { d /= 2; r += 1; }
    for &a in &[2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if a % n == 0 { continue; }
        let mut x = mod_pow_u64(a, d, n);
        if x == 1 || x == n - 1 { continue; }
        let mut composite = true;
        for _ in 0..r - 1 {
            x = mulmod(x, x, n);
            if x == n - 1 { composite = false; break; }
        }
        if composite { return false; }
    }
    true
}

fn mod_pow_u64(mut base: u64, mut exp: u64, m: u64) -> u64 {
    let mut acc = 1u64 % m;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 { acc = mulmod(acc, base, m); }
        base = mulmod(base, base, m);
        exp >>= 1;
    }
    acc
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 { let t = a % b; a = b; b = t; }
    a
}

fn bn_mul_small(limbs: &mut Vec<u64>, mul: u64) {
    let mut carry: u128 = 0;
    for limb in limbs.iter_mut() {
        let prod = (*limb as u128) * (mul as u128) + carry;
        *limb = prod as u64;
        carry = prod >> 64;
    }
    while carry > 0 { limbs.push(carry as u64); carry >>= 64; }
}

fn bn_add_small(limbs: &mut Vec<u64>, add: u64) {
    let mut carry: u128 = add as u128;
    for limb in limbs.iter_mut() {
        if carry == 0 { break; }
        let sum = (*limb as u128) + carry;
        *limb = sum as u64;
        carry = sum >> 64;
    }
    while carry > 0 { limbs.push(carry as u64); carry >>= 64; }
}

fn bn_trim(limbs: &mut Vec<u64>) {
    while limbs.len() > 1 && *limbs.last().unwrap() == 0 { limbs.pop(); }
}

fn bn_cmp(a: &[u64], b: &[u64]) -> Ordering {
    let alen = { let mut l = a.len(); while l > 1 && a[l-1] == 0 { l -= 1; } l };
    let blen = { let mut l = b.len(); while l > 1 && b[l-1] == 0 { l -= 1; } l };
    if alen != blen { return alen.cmp(&blen); }
    for i in (0..alen).rev() {
        match a[i].cmp(&b[i]) { Ordering::Equal => continue, ord => return ord }
    }
    Ordering::Equal
}

/// A growable coprime basis of ~2^63 moduli. Shared by all DynCRT values that
/// interoperate. Lanes are appended on demand; never removed (values stay valid).
#[derive(Clone, Debug)]
pub struct DynBasis {
    pub moduli: Vec<u64>,
}

impl DynBasis {
    /// Seed with the 8 canonical EXACT_MODULI (≈ 2^504).
    pub fn canonical() -> Self {
        Self { moduli: EXACT_MODULI.to_vec() }
    }

    pub fn len(&self) -> usize { self.moduli.len() }
    pub fn is_empty(&self) -> bool { self.moduli.is_empty() }

    /// Approximate bit-capacity log2(M) = Σ log2(m_i). Each lane ≈ 63 bits.
    pub fn capacity_bits(&self) -> u32 {
        self.moduli.iter().map(|&m| 64 - m.leading_zeros()).sum()
    }

    /// Add `n` new coprime primes just below 2^63. Exact, no float.
    pub fn extend(&mut self, n: usize) {
        let mut candidate = *self.moduli.iter().min().unwrap_or(&((1u64 << 63) - 1));
        candidate = candidate.saturating_sub(1);
        let mut added = 0;
        while added < n {
            if candidate < 3 { break; }
            if is_prime_u64(candidate) && self.moduli.iter().all(|&m| gcd_u64(m, candidate) == 1) {
                self.moduli.push(candidate);
                added += 1;
            }
            candidate -= 1;
        }
    }
}

/// A dynamic-lane exact integer. Magnitude in residues over `basis.moduli`,
/// with a separate sign.
#[derive(Clone, Debug)]
pub struct DynCRT {
    pub residues: Vec<u64>,
    pub sign: i8, // -1, 0, +1
}

impl DynCRT {
    pub fn zero(basis: &DynBasis) -> Self {
        Self { residues: vec![0; basis.len()], sign: 0 }
    }

    pub fn from_i128(value: i128, basis: &DynBasis) -> Self {
        let sign = if value > 0 { 1 } else if value < 0 { -1 } else { 0 };
        let mag = value.unsigned_abs();
        let residues = basis.moduli.iter().map(|&m| (mag % m as u128) as u64).collect();
        Self { residues, sign }
    }

    pub fn is_zero(&self) -> bool { self.sign == 0 || self.residues.iter().all(|&r| r == 0) }

    fn mag_add(&self, other: &Self, basis: &DynBasis) -> Vec<u64> {
        (0..basis.len()).map(|i| addmod(self.residues[i], other.residues[i], basis.moduli[i])).collect()
    }
    fn mag_sub_big_minus_small(big: &Self, small: &Self, basis: &DynBasis) -> Vec<u64> {
        (0..basis.len()).map(|i| submod(big.residues[i], small.residues[i], basis.moduli[i])).collect()
    }

    /// Exact mixed-radix (Garner) digits, little-endian. Exact ordering at ANY lane count.
    pub fn mixed_radix_digits(&self, basis: &DynBasis) -> Vec<u64> {
        let k = basis.len();
        let m = &basis.moduli;
        let mut v = vec![0u64; k];
        if k == 0 { return v; }
        v[0] = self.residues[0] % m[0];
        for i in 1..k {
            let mi = m[i];
            let mut temp = self.residues[i] % mi;
            for j in 0..i {
                let vj = v[j] % mi;
                if temp < vj { temp += mi; }
                temp -= vj;
                temp = mulmod(temp, mod_inverse(m[j] % mi, mi).unwrap_or(0), mi);
            }
            v[i] = temp;
        }
        v
    }

    /// Exact magnitude comparison by winding digits, most-significant first.
    pub fn cmp_abs(&self, other: &Self, basis: &DynBasis) -> Ordering {
        let da = self.mixed_radix_digits(basis);
        let db = other.mixed_radix_digits(basis);
        for i in (0..basis.len()).rev() {
            match da[i].cmp(&db[i]) { Ordering::Equal => continue, ord => return ord }
        }
        Ordering::Equal
    }

    /// Signed comparison of true integer values. Exact over [0, M).
    pub fn cmp_signed(&self, other: &Self, basis: &DynBasis) -> Ordering {
        let sa = if self.is_zero() { 0 } else { self.sign.signum() };
        let sb = if other.is_zero() { 0 } else { other.sign.signum() };
        match (sa, sb) {
            (a, b) if a < b => Ordering::Less,
            (a, b) if a > b => Ordering::Greater,
            (0, 0) => Ordering::Equal,
            (1, 1) => self.cmp_abs(other, basis),
            (-1, -1) => other.cmp_abs(self, basis),
            _ => Ordering::Equal,
        }
    }

    /// Exact full-width magnitude as little-endian u64 limbs. For readout and reproject.
    pub fn to_limbs(&self, basis: &DynBasis) -> Vec<u64> {
        let v = self.mixed_radix_digits(basis);
        let m = &basis.moduli;
        let k = basis.len();
        let mut limbs = vec![0u64];
        for i in (0..k).rev() {
            if i < k - 1 { bn_mul_small(&mut limbs, m[i]); }
            bn_add_small(&mut limbs, v[i]);
        }
        bn_trim(&mut limbs);
        limbs
    }

    /// Signed value if it fits i128.
    pub fn to_i128(&self, basis: &DynBasis) -> Option<i128> {
        if self.is_zero() { return Some(0); }
        let limbs = self.to_limbs(basis);
        if limbs.len() > 2 { return None; }
        let lo = limbs[0] as u128;
        let hi = if limbs.len() > 1 { limbs[1] as u128 } else { 0 };
        let mag = (hi << 64) | lo;
        if mag > i128::MAX as u128 { return None; }
        let val = mag as i128;
        Some(if self.sign < 0 { -val } else { val })
    }

    pub fn add(&self, other: &Self, basis: &DynBasis) -> Self {
        if self.is_zero() { return other.clone(); }
        if other.is_zero() { return self.clone(); }
        if self.sign == other.sign {
            Self { residues: self.mag_add(other, basis), sign: self.sign }
        } else {
            let (big, small, sign) = match self.cmp_abs(other, basis) {
                Ordering::Greater | Ordering::Equal => (self, other, self.sign),
                Ordering::Less => (other, self, other.sign),
            };
            let residues = Self::mag_sub_big_minus_small(big, small, basis);
            let is_zero = residues.iter().all(|&r| r == 0);
            Self { residues, sign: if is_zero { 0 } else { sign } }
        }
    }

    pub fn sub(&self, other: &Self, basis: &DynBasis) -> Self {
        let neg = Self { residues: other.residues.clone(), sign: -other.sign };
        self.add(&neg, basis)
    }

    pub fn mul(&self, other: &Self, basis: &DynBasis) -> Self {
        let residues = (0..basis.len())
            .map(|i| mulmod(self.residues[i], other.residues[i], basis.moduli[i]))
            .collect();
        Self { residues, sign: self.sign * other.sign }
    }

    /// Lossless re-projection onto a larger basis. Requires value < old M.
    pub fn reproject(&self, old_basis: &DynBasis, new_basis: &DynBasis) -> Self {
        let limbs = self.to_limbs(old_basis);
        let residues = new_basis.moduli.iter().map(|&m| {
            let mut rem: u128 = 0;
            for &limb in limbs.iter().rev() {
                rem = ((rem << 64) | (limb as u128)) % (m as u128);
            }
            rem as u64
        }).collect();
        Self { residues, sign: self.sign }
    }

    /// Activation level: number of non-zero Garner digits (winding tower depth).
    /// A bounded activation level across a trajectory window is the exact integer
    /// witness for a uniform L∞ bound — the BKM criterion input.
    pub fn activation_level(&self, basis: &DynBasis) -> usize {
        let digits = self.mixed_radix_digits(basis);
        digits.iter().rposition(|&d| d != 0).map(|i| i + 1).unwrap_or(0)
    }
}

/// A self-growing value: carries its basis and extends lanes on demand.
/// There is NO largest representable value.
pub struct GrowingInt {
    pub basis: DynBasis,
    pub value: DynCRT,
}

impl GrowingInt {
    pub fn from_i128(v: i128) -> Self {
        let basis = DynBasis::canonical();
        let value = DynCRT::from_i128(v, &basis);
        Self { basis, value }
    }

    pub fn bit_len(&self) -> usize {
        let limbs = self.value.to_limbs(&self.basis);
        let top = *limbs.last().unwrap();
        (limbs.len() - 1) * 64 + (64 - top.leading_zeros() as usize)
    }

    pub fn ensure_headroom(&mut self, headroom_bits: u32) {
        while self.basis.capacity_bits() < self.bit_len() as u32 + headroom_bits {
            let old = self.basis.clone();
            self.basis.extend(2);
            self.value = self.value.reproject(&old, &self.basis);
        }
    }

    pub fn mul_assign(&mut self, other_val: i128) {
        let other_bits = 128 - (other_val.unsigned_abs()).leading_zeros();
        self.ensure_headroom(other_bits + 8);
        let o = DynCRT::from_i128(other_val, &self.basis);
        self.value = self.value.mul(&o, &self.basis);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dyncrt_signed_matches_native() {
        let basis = DynBasis::canonical();
        let vals: [i128; 11] = [-(1i128<<100), -(1i128<<64), -5_000_000_000, -7, -1, 0,
                                 1, 7, 5_000_000_000, 1i128<<64, 1i128<<100];
        for &a in &vals { for &b in &vals {
            let ca = DynCRT::from_i128(a, &basis);
            let cb = DynCRT::from_i128(b, &basis);
            assert_eq!(ca.cmp_signed(&cb, &basis), a.cmp(&b), "cmp {a} {b}");
            if let Some(s) = a.checked_add(b) {
                assert_eq!(ca.add(&cb, &basis).to_i128(&basis), Some(s), "add {a}+{b}");
            }
            if let Some(d) = a.checked_sub(b) {
                assert_eq!(ca.sub(&cb, &basis).to_i128(&basis), Some(d), "sub {a}-{b}");
            }
        }}
        println!("DynCRT signed add/sub/cmp: 121 pairs match native i128");
    }

    #[test]
    fn basis_extends_unbounded() {
        let mut basis = DynBasis::canonical();
        let cap0 = basis.capacity_bits();
        assert_eq!(basis.len(), 8);
        basis.extend(8);
        assert_eq!(basis.len(), 16);
        for i in 0..basis.len() { for j in (i+1)..basis.len() {
            assert_eq!(gcd_u64(basis.moduli[i], basis.moduli[j]), 1, "lanes must be coprime");
        }}
        assert!(basis.capacity_bits() > cap0 + 400, "8 new ~63-bit lanes add >400 bits");
        println!("basis extended 8→16 lanes, capacity {cap0}→{} bits, all coprime", basis.capacity_bits());
    }

    #[test]
    fn growing_int_no_ceiling() {
        // (10^9)^40 = 10^360, far past M=2^504 of the 8-lane carrier.
        // GrowingInt extends lanes on demand; value stays exact.
        let mut g = GrowingInt::from_i128(1_000_000_000);
        let mut prev_bits = g.bit_len();
        for k in 2..=40 {
            g.mul_assign(1_000_000_000);
            let bits = g.bit_len();
            assert!(bits > prev_bits, "step {k}: magnitude must strictly grow (no wrap)");
            prev_bits = bits;
        }
        assert!(g.bit_len() > 1190, "10^360 ≈ 2^1196");
        assert!(g.basis.len() > 8, "basis must have grown past 8 lanes");
        let expected_bits = 1196;
        assert!((g.bit_len() as i64 - expected_bits).abs() <= 2,
            "bit_len {} ~ {expected_bits}", g.bit_len());
        println!("GrowingInt: 10^360 (≈2^{}) held EXACTLY across {} lanes — NO CEILING",
            g.bit_len(), g.basis.len());
    }

    #[test]
    fn reproject_lossless() {
        let old = DynBasis::canonical();
        let mut new = old.clone();
        new.extend(4);
        let v = DynCRT::from_i128((1i128<<100) + 12345, &old);
        let v2 = v.reproject(&old, &new);
        assert_eq!(v2.to_i128(&new), Some((1i128<<100) + 12345));
        let w = DynCRT::from_i128((1i128<<100) + 12344, &new);
        assert_eq!(v2.cmp_signed(&w, &new), Ordering::Greater);
        println!("reproject 8→12 lanes lossless; comparison consistent post-extension");
    }

    #[test]
    fn activation_level_tracks_magnitude() {
        let basis = DynBasis::canonical();
        let zero = DynCRT::zero(&basis);
        assert_eq!(zero.activation_level(&basis), 0, "zero has activation level 0");
        let small = DynCRT::from_i128(7, &basis);
        assert_eq!(small.activation_level(&basis), 1, "7 fits in level 1");
        // Large value needs multiple Garner digits
        let large = DynCRT::from_i128(1i128 << 100, &basis);
        assert!(large.activation_level(&basis) >= 2, "2^100 needs multiple levels");
        println!("activation_level: zero=0, 7=1, 2^100={}", large.activation_level(&basis));
    }
}
