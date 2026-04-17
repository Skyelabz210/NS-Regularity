//! # QMNF Primitives
//!
//! Single source of truth for modular arithmetic, NTT, K-Elimination,
//! Garner reconstruction, and Residue types. All HackFate crates depend
//! on this instead of duplicating primitives.

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

// ============================================================================
// MODULAR ARITHMETIC
// ============================================================================

#[inline] pub fn addmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 + b as u128) % m as u128) as u64 }
#[inline] pub fn submod(a: u64, b: u64, m: u64) -> u64 { if a >= b { (a-b)%m } else { (m-((b-a)%m))%m } }
#[inline] pub fn mulmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 * b as u128) % m as u128) as u64 }

pub fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    if m == 1 { return 0; }
    let mut r = 1u64; base %= m;
    while exp > 0 { if exp & 1 == 1 { r = mulmod(r, base, m); } exp >>= 1; base = mulmod(base, base, m); }
    r
}

/// Modular inverse via Fermat's little theorem. Requires p prime.
pub fn mod_inverse(a: u64, p: u64) -> u64 {
    assert!(a % p != 0, "Cannot invert zero mod {p}");
    mod_pow(a, p - 2, p)
}

/// Extended GCD modular inverse — works for ANY modulus (prime or composite).
/// Use this instead of mod_inverse when the modulus may be composite.
/// Required for non-coprime K-Elimination (gcd-factored), where
/// reduced moduli like 195 = 3×5×13 are composite.
pub fn mod_inverse_ext(a: u64, m: u64) -> u64 {
    let (mut old_r, mut r) = (a as i128, m as i128);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = old_r / r;
        let tmp = r; r = old_r - q * r; old_r = tmp;
        let tmp = s; s = old_s - q * s; old_s = tmp;
    }
    assert_eq!(old_r, 1, "gcd({},{}) = {} != 1, no inverse exists", a, m, old_r);
    ((old_s % m as i128 + m as i128) % m as i128) as u64
}

/// Non-coprime K-Elimination: recover winding for pairs with gcd > 1.
/// Standard k_eliminate requires gcd(m, a) = 1. This factors through the gcd.
/// Returns None if the residues are inconsistent (r_m % g != r_a % g).
pub fn k_eliminate_general(r_m: u64, r_a: u64, m: u64, a: u64) -> Option<u64> {
    let g = gcd(m, a);
    if g == 1 {
        return Some(k_eliminate(r_m, r_a, m, a));
    }
    if r_m % g != r_a % g {
        return None; // inconsistent
    }
    let m_red = m / g;
    let a_red = a / g;
    if gcd(m_red, a_red) > 1 {
        return k_eliminate_general(r_m % m_red, r_a % a_red, m_red, a_red);
    }
    let rm_mod = r_m % m_red;
    let ra_mod = r_a % a_red;
    Some(k_eliminate(rm_mod, ra_mod, m_red, a_red))
}

/// Non-coprime CRT reconstruction: X mod lcm(m, a) from residues.
/// Works for any m, a (coprime or not). Returns None if inconsistent.
pub fn reconstruct_non_coprime(r_m: u64, r_a: u64, m: u64, a: u64) -> Option<u128> {
    let g = gcd(m, a);
    if r_m % g != r_a % g {
        return None;
    }
    if g == 1 {
        let k = k_eliminate(r_m, r_a, m, a);
        return Some(r_m as u128 + k as u128 * m as u128);
    }
    let m_red = m / g;
    let a_red = a / g;
    let diff = ((r_a as u128 + a as u128) - (r_m as u128 % a as u128)) % a as u128;
    let diff_red = (diff / g as u128) as u64;
    let m_inv = mod_inverse_ext(m_red % a_red, a_red);
    let t = mulmod(diff_red % a_red, m_inv, a_red);
    Some(r_m as u128 + t as u128 * m as u128)
}

pub fn gcd(a: u64, b: u64) -> u64 { if b == 0 { a } else { gcd(b, a % b) } }

pub fn is_prime(n: u64) -> bool {
    if n < 2 { return false; } if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5u64; while i * i <= n { if n % i == 0 || n % (i+2) == 0 { return false; } i += 6; }
    true
}

// ============================================================================
// CONSTANTS
// ============================================================================

/// NTT-friendly primes: p ≡ 1 (mod 256), all verified prime.
pub const LANE_PRIMES: [u64; 7] = [
    998244353, 985661441, 754974721, 469762049, 167772161, 104857601, 69206017,
];

/// Safe Basis for approximation lanes.
pub const SAFE_BASIS: [u64; 6] = [2, 3, 5, 7, 11, 13];

/// NTT size (power of 2 ≥ 144 data bands).
pub const NTT_SIZE: usize = 256;

/// Number of data bands (F₁₂).
pub const NUM_DATA_BANDS: usize = 144;

/// Bytes per holographic coefficient (fits in smallest prime ~2^26).
pub const BYTES_PER_COEFF: usize = 3;

/// Total hologram data capacity.
pub const HOLOGRAM_CAPACITY: usize = NUM_DATA_BANDS * BYTES_PER_COEFF;

/// Crystal modulus (Mersenne prime M61).
pub const CRYSTAL_MODULUS: u64 = (1u64 << 61) - 1;

/// Fibonacci table (all 93 that fit in u64).
pub const FIB: [u64; 93] = {
    let mut f = [0u64; 93]; f[1] = 1;
    let mut i = 2; while i < 93 { f[i] = f[i-1] + f[i-2]; i += 1; } f
};

// ============================================================================
// K-ELIMINATION
// ============================================================================

/// K-Elimination: recover winding number k from two coprime residues.
/// k ≡ (s - r) * m⁻¹ (mod a), X = r + k*m (unique for 0 ≤ X < m*a).
pub fn k_eliminate(r: u64, s: u64, m: u64, a: u64) -> u64 {
    debug_assert_eq!(gcd(m, a), 1);
    let m_inv = mod_inverse(m % a, a);
    mulmod(submod(s % a, r % a, a), m_inv, a)
}

pub fn k_reconstruct(r: u64, s: u64, m: u64, a: u64) -> u128 {
    r as u128 + k_eliminate(r, s, m, a) as u128 * m as u128
}

// ============================================================================
// GARNER RECONSTRUCTION
// ============================================================================

pub fn garner_reconstruct(residues: &[u64], moduli: &[u64]) -> u128 {
    let k = residues.len();
    if k == 0 { return 0; } if k == 1 { return residues[0] as u128; }
    let mut v = vec![0u64; k];
    v[0] = residues[0];
    for i in 1..k {
        let mi = moduli[i];
        let mut temp = residues[i] % mi;
        for j in 0..i {
            let vj = v[j] % mi;
            if temp < vj { temp += mi; }
            temp -= vj;
            temp = mulmod(temp, mod_inverse(moduli[j] % mi, mi), mi);
        }
        v[i] = temp;
    }
    let mut result = v[0] as u128;
    let mut product = moduli[0] as u128;
    for i in 1..k { result += v[i] as u128 * product; product *= moduli[i] as u128; }
    result
}

// ============================================================================
// RESIDUES TYPE
// ============================================================================

/// A value represented as residues across CRT lanes.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Residues { pub values: Vec<u64> }

impl Residues {
    pub fn zero(n: usize) -> Self { Self { values: vec![0; n] } }
    pub fn from_u64(val: u64, primes: &[u64]) -> Self {
        Self { values: primes.iter().map(|&p| val % p).collect() }
    }
    pub fn add(&self, other: &Self, primes: &[u64]) -> Self {
        Self { values: self.values.iter().zip(&other.values).zip(primes)
            .map(|((&a, &b), &p)| addmod(a, b, p)).collect() }
    }
    pub fn mul(&self, other: &Self, primes: &[u64]) -> Self {
        Self { values: self.values.iter().zip(&other.values).zip(primes)
            .map(|((&a, &b), &p)| mulmod(a, b, p)).collect() }
    }
    pub fn reconstruct(&self, primes: &[u64]) -> u128 {
        garner_reconstruct(&self.values, primes)
    }
}

// ============================================================================
// NTT ENGINE
// ============================================================================

pub fn find_primitive_root(p: u64, order: usize) -> u64 {
    let n = order as u64; let exp = (p-1)/n;
    for g in 2..p {
        let w = mod_pow(g, exp, p);
        if w != 1 && mod_pow(w, n, p) == 1 {
            let mut ok = true; let mut d = 2u64;
            while d*d <= n { if n%d==0 && (mod_pow(w,d,p)==1 || mod_pow(w,n/d,p)==1) { ok=false; break; } d+=1; }
            if ok { return w; }
        }
    }
    panic!("No primitive {order}-th root mod {p}");
}

/// Per-lane NTT context with precomputed twiddle factors.
#[derive(Clone)]
pub struct LaneNtt {
    pub prime: u64,
    pub n_inv: u64,
    pub twiddles: Vec<u64>,
    pub twiddles_inv: Vec<u64>,
}

impl LaneNtt {
    pub fn new(prime: u64) -> Self {
        assert!(is_prime(prime));
        assert_eq!((prime - 1) % (NTT_SIZE as u64), 0);
        let omega = find_primitive_root(prime, NTT_SIZE);
        let omega_inv = mod_inverse(omega, prime);
        let n_inv = mod_inverse(NTT_SIZE as u64, prime);
        let mut tw = vec![0u64; NTT_SIZE]; let mut twi = vec![0u64; NTT_SIZE];
        tw[0] = 1; twi[0] = 1;
        for i in 1..NTT_SIZE { tw[i] = mulmod(tw[i-1], omega, prime); twi[i] = mulmod(twi[i-1], omega_inv, prime); }
        Self { prime, n_inv, twiddles: tw, twiddles_inv: twi }
    }

    pub fn transform(&self, data: &mut [u64; NTT_SIZE], inverse: bool) {
        let p = self.prime; let n = NTT_SIZE;
        let mut j = 0usize;
        for i in 1..n { let mut bit = n>>1; while j&bit!=0 { j^=bit; bit>>=1; } j^=bit; if i<j { data.swap(i,j); } }
        let tw = if inverse { &self.twiddles_inv } else { &self.twiddles };
        let mut len = 2; while len <= n { let h = len/2; let step = n/len;
            for s in (0..n).step_by(len) { for k in 0..h {
                let t = tw[k*step]; let u = data[s+k]; let v = mulmod(data[s+k+h], t, p);
                data[s+k] = addmod(u, v, p); data[s+k+h] = submod(u, v, p);
            }} len <<= 1; }
        if inverse { for v in data.iter_mut() { *v = mulmod(*v, self.n_inv, p); } }
    }
}

/// Multi-lane NTT context.
#[derive(Clone)]
pub struct NttCtx { pub lanes: Vec<LaneNtt> }
impl NttCtx {
    pub fn new(primes: &[u64]) -> Self { Self { lanes: primes.iter().map(|&p| LaneNtt::new(p)).collect() } }
    pub fn primes(&self) -> Vec<u64> { self.lanes.iter().map(|l| l.prime).collect() }
    pub fn num_lanes(&self) -> usize { self.lanes.len() }

    /// Forward NTT across all lanes: bands → sectors.
    pub fn forward(&self, bands: &[Residues]) -> Vec<Residues> {
        let nl = self.num_lanes();
        let mut bufs: Vec<[u64;NTT_SIZE]> = (0..nl).map(|l| {
            let mut b = [0u64;NTT_SIZE]; for (i, band) in bands.iter().enumerate() { b[i] = band.values[l]; } b
        }).collect();
        for (l, buf) in bufs.iter_mut().enumerate() { self.lanes[l].transform(buf, false); }
        (0..NTT_SIZE).map(|s| Residues { values: (0..nl).map(|l| bufs[l][s]).collect() }).collect()
    }

    /// Inverse NTT across all lanes: sectors → bands.
    pub fn inverse(&self, sectors: &[Residues]) -> Vec<Residues> {
        let nl = self.num_lanes();
        let mut bufs: Vec<[u64;NTT_SIZE]> = (0..nl).map(|l| {
            let mut b = [0u64;NTT_SIZE]; for (i, sec) in sectors.iter().enumerate() { b[i] = sec.values[l]; } b
        }).collect();
        for (l, buf) in bufs.iter_mut().enumerate() { self.lanes[l].transform(buf, true); }
        (0..NTT_SIZE).map(|b| Residues { values: (0..nl).map(|l| bufs[l][b]).collect() }).collect()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn modular_arithmetic() {
        assert_eq!(addmod(5, 7, 10), 2);
        assert_eq!(submod(3, 7, 10), 6);
        assert_eq!(mulmod(7, 8, 10), 6);
        assert_eq!(mod_pow(2, 10, 1000), 24);
    }

    #[test] fn mod_inverse_correct() {
        for &p in &[7u64, 11, 13, 998244353] {
            for a in 1..p.min(100) {
                let inv = mod_inverse(a, p);
                assert_eq!(mulmod(a, inv, p), 1, "inverse({a}, {p}) failed");
            }
        }
    }

    #[test] fn k_elimination_roundtrip() {
        let v = 35u64; let m = 7u64; let a = 11u64;
        assert_eq!(k_reconstruct(v % m, v % a, m, a), 35);
    }

    #[test] fn garner_small() {
        let primes = vec![7u64, 11, 13];
        for v in [0u64, 1, 42, 100, 1000] {
            let res: Vec<u64> = primes.iter().map(|&p| v % p).collect();
            assert_eq!(garner_reconstruct(&res, &primes), v as u128);
        }
    }

    #[test] fn garner_large() {
        let primes = LANE_PRIMES[..4].to_vec();
        for v in [0u64, 1, 42, 999999, u32::MAX as u64] {
            let res: Vec<u64> = primes.iter().map(|&p| v % p).collect();
            assert_eq!(garner_reconstruct(&res, &primes), v as u128);
        }
    }

    #[test] fn residues_roundtrip() {
        let primes = LANE_PRIMES[..4].to_vec();
        for v in [0u64, 42, 12345, 999999] {
            let r = Residues::from_u64(v, &primes);
            assert_eq!(r.reconstruct(&primes), v as u128);
        }
    }

    #[test] fn ntt_roundtrip() {
        let primes = LANE_PRIMES.iter().copied().filter(|p| is_prime(*p) && (*p-1) % NTT_SIZE as u64 == 0).take(4).collect::<Vec<_>>();
        let ctx = NttCtx::new(&primes);
        let mut bands: Vec<Residues> = (0..NTT_SIZE).map(|i| {
            if i < NUM_DATA_BANDS { Residues::from_u64((i as u64 + 1) * 7, &primes) }
            else { Residues::zero(primes.len()) }
        }).collect();
        let original = bands.clone();
        let sectors = ctx.forward(&bands);
        let recovered = ctx.inverse(&sectors);
        for i in 0..NTT_SIZE { assert_eq!(recovered[i], original[i], "NTT roundtrip failed at band {i}"); }
    }

    #[test] fn lane_primes_valid() {
        for &p in &LANE_PRIMES {
            assert!(is_prime(p), "{p} not prime");
            assert_eq!((p-1) % NTT_SIZE as u64, 0, "{p}-1 not divisible by {NTT_SIZE}");
        }
    }
}

#[cfg(test)]
mod benchmark_kelim;

pub mod cram3_grover;

pub mod nine65_mmbf;

pub mod sparse_toric_grover;

pub mod gear_manifold;

pub mod cram_ops;

pub mod cram_safety;

#[cfg(test)]
mod non_coprime_tests {
    use super::*;

    #[test]
    fn mod_inverse_ext_on_primes() {
        // Should match mod_inverse for prime moduli
        for &p in &[7u64, 11, 13, 17, 29, 97] {
            for a in 1..p {
                let inv_fermat = mod_inverse(a, p);
                let inv_ext = mod_inverse_ext(a, p);
                assert_eq!(inv_fermat, inv_ext,
                    "Mismatch for {}^-1 mod {}", a, p);
            }
        }
    }

    #[test]
    fn mod_inverse_ext_on_composites() {
        // These fail with mod_inverse (Fermat) but work with ext
        let composites = [(146u64, 195u64), (52, 73), (4, 9), (7, 15), (11, 26)];
        for &(a, m) in &composites {
            if gcd(a, m) == 1 {
                let inv = mod_inverse_ext(a, m);
                assert_eq!((a as u128 * inv as u128) % m as u128, 1,
                    "{}^-1 mod {} = {} but {}*{} mod {} != 1", a, m, inv, a, inv, m);
            }
        }
    }

    #[test]
    fn k_eliminate_general_coprime() {
        // For coprime pairs, should match standard k_eliminate
        for x in 0..500u64 {
            let r148 = x % 148;
            let r177 = x % 177;
            let k_std = k_eliminate(r148, r177, 148, 177);
            let k_gen = k_eliminate_general(r148, r177, 148, 177).unwrap();
            assert_eq!(k_std, k_gen);
        }
    }

    #[test]
    fn k_eliminate_general_non_coprime() {
        // Tzolkin/Haab: gcd = 5
        for x in [0u64, 1, 260, 365, 1000, 18979] {
            let r_tz = x % 260;
            let r_ha = x % 365;
            assert!(k_eliminate_general(r_tz, r_ha, 260, 365).is_some(),
                "K-Elim failed at x={}", x);
        }
    }

    #[test]
    fn reconstruct_non_coprime_tzolkin_haab() {
        // Full Calendar Round reconstruction
        for x in (0..18980u64).step_by(100) {
            let r_tz = x % 260;
            let r_ha = x % 365;
            let val = reconstruct_non_coprime(r_tz, r_ha, 260, 365);
            assert_eq!(val, Some(x as u128), "Failed at x={}", x);
        }
    }

    #[test]
    fn reconstruct_non_coprime_venus_mars() {
        // gcd(584, 780) = 4, lcm = 113880
        for x in (0..113880u64).step_by(500) {
            let r_v = x % 584;
            let r_m = x % 780;
            let val = reconstruct_non_coprime(r_v, r_m, 584, 780);
            assert_eq!(val, Some(x as u128), "Failed at x={}", x);
        }
    }

    #[test]
    fn reconstruct_inconsistent_returns_none() {
        // gcd(260, 365) = 5. Residues must agree mod 5.
        assert!(reconstruct_non_coprime(1, 2, 260, 365).is_none());
        assert!(reconstruct_non_coprime(3, 7, 260, 365).is_none());
    }
}
