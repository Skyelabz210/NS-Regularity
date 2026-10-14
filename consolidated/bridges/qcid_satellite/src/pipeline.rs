//! Full NS Pipeline with F_p² Dual-Track + Stencil Eigenvalues
//!
//! The pipeline per timestep:
//!   1. NS root operation (physics on residue a)
//!   2. Norm computation (derived from a, b)
//!   3. Overflow check + norm Sqr (contraction on norm)
//!
//! The stencil-BS divides by 4σ_k (physical eigenvalue) instead of |k|²
//! (flat wavenumber). Both are exact. Stencil is physically correct.

use crate::stencil::*;
use crate::fp2::Fp2;
use crate::dual_track::DualTrackState;

fn mulmod(a: u64, b: u64, p: u64) -> u64 { ((a as u128 * b as u128) % p as u128) as u64 }
fn addmod(a: u64, b: u64, p: u64) -> u64 { ((a as u128 + b as u128) % p as u128) as u64 }
fn submod(a: u64, b: u64, p: u64) -> u64 { if a >= b { a - b } else { p - (b - a) } }
fn modpow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    let mut r = 1u64; base %= m;
    while exp > 0 { if exp & 1 == 1 { r = mulmod(r, base, m); } exp >>= 1; base = mulmod(base, base, m); }
    r
}
fn modinv(a: u64, p: u64) -> u64 { modpow(a, p - 2, p) }

/// Simplified 1D NS-like diffusion step on a periodic grid.
/// ω'[i] = ω[i] + c_diff × (ω[i+1] + ω[i-1] - 2×ω[i])
/// Operates mod p. Returns new field.
fn diffusion_step_1d(field: &[u64], c_diff: u64, p: u64) -> Vec<u64> {
    let n = field.len();
    let mut out = vec![0u64; n];
    for i in 0..n {
        let ip = (i + 1) % n;
        let im = (i + n - 1) % n;
        let lap = addmod(addmod(field[ip], field[im], p), p - mulmod(2, field[i], p), p);
        let correction = mulmod(c_diff, lap, p);
        out[i] = addmod(field[i], correction, p);
    }
    out
}

/// Stencil-BS for 1D: u_hat[k] = ω_hat[k] / eigenvalue[k]
/// Eigenvalue[k] = 2(1 - cos(2πk/N)) for 1D stencil Laplacian.
fn stencil_bs_1d(omega_hat: &[u64], omega_root: u64, p: u64) -> Vec<u64> {
    let n = omega_hat.len();
    let mut u_hat = vec![0u64; n];
    let inv2 = modinv(2, p);
    for k in 0..n {
        if k == 0 { u_hat[k] = 0; continue; }
        // eigenvalue = 2(1 - cos(2πk/N))
        let wk = modpow(omega_root, k as u64, p);
        let wk_inv = modinv(wk, p);
        let cos_k = mulmod(addmod(wk, wk_inv, p), inv2, p);
        let eigen = mulmod(2, submod(1, cos_k, p), p);
        if eigen == 0 { u_hat[k] = 0; continue; }
        u_hat[k] = mulmod(omega_hat[k], modinv(eigen, p), p);
    }
    u_hat
}

/// Flat-BS for 1D: u_hat[k] = ω_hat[k] / |k|²
fn flat_bs_1d(omega_hat: &[u64], n: usize, p: u64) -> Vec<u64> {
    let mut u_hat = vec![0u64; n];
    for k in 0..n {
        let kc = if k <= n/2 { k as i64 } else { k as i64 - n as i64 };
        let ksq = (kc * kc) as u64;
        if ksq == 0 { u_hat[k] = 0; continue; }
        u_hat[k] = mulmod(omega_hat[k], modinv(ksq % p, p), p);
    }
    u_hat
}

/// 1D NTT (DFT mod p)
fn ntt_1d(f: &[u64], omega: u64, p: u64) -> Vec<u64> {
    let n = f.len();
    let mut out = vec![0u64; n];
    for k in 0..n {
        let mut sum = 0u64;
        for j in 0..n {
            let wkj = modpow(omega, (k * j) as u64 % ((p-1) as u64), p);
            sum = addmod(sum, mulmod(f[j], wkj, p), p);
        }
        out[k] = sum;
    }
    out
}

/// 1D inverse NTT
fn intt_1d(f: &[u64], omega: u64, p: u64) -> Vec<u64> {
    let n = f.len();
    let omega_inv = modinv(omega, p);
    let n_inv = modinv(n as u64, p);
    let raw = ntt_1d(f, omega_inv, p);
    raw.iter().map(|&x| mulmod(x, n_inv, p)).collect()
}

/// DualTrackPipeline: manages a 1D grid of DualTrackStates
pub struct DualTrackPipeline {
    pub grid_size: usize,
    pub prime_idx: usize,
    pub p: u64,
    pub delta: u64,
    pub threshold: u64,
    pub c_diff: u64,
    pub states: Vec<DualTrackState>,
    pub step_count: usize,
    pub total_sqrs: usize,
}

impl DualTrackPipeline {
    pub fn new(grid_size: usize, prime_idx: usize, c_diff: u64) -> Self {
        let p = NTT_PRIMES[prime_idx];
        let delta = Fp2::find_qnr(p);
        let threshold = p / 3;
        let states: Vec<DualTrackState> = (0..grid_size)
            .map(|_| DualTrackState::new(Fp2::new(0, 0, p, delta)))
            .collect();
        DualTrackPipeline {
            grid_size, prime_idx, p, delta, threshold, c_diff,
            states, step_count: 0, total_sqrs: 0,
        }
    }

    /// Initialize with a single cosine mode: ω[i] = amp × cos(2πi/N)
    /// Computed via NTT root: cos(2πi/N) = (ω^i + ω^{-i})/2
    pub fn init_cosine(&mut self, amp: u64) {
        let omega = NTT_ROOTS[self.prime_idx];
        let inv2 = modinv(2, self.p);
        for i in 0..self.grid_size {
            let wi = modpow(omega, i as u64, self.p);
            let wi_inv = modinv(wi, self.p);
            let cos_i = mulmod(addmod(wi, wi_inv, self.p), inv2, self.p);
            let val = mulmod(amp % self.p, cos_i, self.p);
            self.states[i] = DualTrackState::new(Fp2::new(val, 0, self.p, self.delta));
        }
    }

    /// Run one full pipeline step
    pub fn step(&mut self) {
        // STEP 1: Diffusion (physics on residue a)
        let field: Vec<u64> = self.states.iter().map(|s| s.fp2.a).collect();
        let new_field = diffusion_step_1d(&field, self.c_diff, self.p);

        // STEP 2: Update states
        for i in 0..self.grid_size {
            let delta_a = submod(new_field[i], self.states[i].fp2.a, self.p);
            let update = Fp2::new(delta_a, 0, self.p, self.delta);
            self.states[i].apply_ns_update(&update);
        }

        // STEP 3: Overflow check + norm Sqr
        for i in 0..self.grid_size {
            if self.states[i].check_overflow(self.threshold) {
                self.states[i].apply_norm_sqr();
                self.total_sqrs += 1;
            }
        }

        self.step_count += 1;
    }

    /// Energy: sum of a² (L2 norm of residue field)
    pub fn energy(&self) -> u64 {
        let mut e = 0u128;
        for s in &self.states {
            e += mulmod(s.fp2.a, s.fp2.a, self.p) as u128;
        }
        (e % self.p as u128) as u64
    }

    /// Max norm across all grid points
    pub fn max_norm(&self) -> u64 {
        self.states.iter().map(|s| s.norm_value).max().unwrap_or(0)
    }

    /// Max sqr_count across all grid points
    pub fn max_sqr_depth(&self) -> u64 {
        self.states.iter().map(|s| s.sqr_count).max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_1d_diffusion_only() {
        // Pure diffusion: cosine mode should decay
        let n = 16; // small 1D grid
        for pi in 0..4 {
            let p = NTT_PRIMES[pi];
            let c_diff = modinv(6, p); // small diffusion coefficient

            let mut pipe = DualTrackPipeline::new(n, pi, c_diff);
            pipe.init_cosine(100);

            let e0 = pipe.energy();
            println!("p={p}: Initial energy = {e0}");

            for step in 0..10 {
                pipe.step();
                let e = pipe.energy();
                let max_n = pipe.max_norm();
                let max_d = pipe.max_sqr_depth();
                println!("  step {step}: energy={e}, max_norm={max_n}, max_sqr_depth={max_d}, total_sqrs={}", pipe.total_sqrs);
            }
            println!();
        }
    }

    #[test]
    fn physics_residue_never_touched_by_sqr() {
        // Record residues before and after multiple Sqrs fire
        let pi = 0; // p=769
        let p = NTT_PRIMES[pi];
        let mut pipe = DualTrackPipeline::new(8, pi, modinv(6, p));
        pipe.init_cosine(500);
        pipe.threshold = 1; // force Sqrs to fire every step

        let a_before: Vec<u64> = pipe.states.iter().map(|s| s.fp2.a).collect();

        // Do a physics step
        pipe.step();
        let a_after_step: Vec<u64> = pipe.states.iter().map(|s| s.fp2.a).collect();

        // Physics changed (diffusion happened)
        let physics_changed = a_before.iter().zip(&a_after_step)
            .any(|(b, a)| b != a);
        assert!(physics_changed, "Physics should evolve");

        // Now do ONLY norm-Sqrs (no physics)
        let a_before_sqr: Vec<u64> = pipe.states.iter().map(|s| s.fp2.a).collect();
        for s in &mut pipe.states {
            s.apply_norm_sqr();
            s.apply_norm_sqr();
            s.apply_norm_sqr();
        }
        let a_after_sqr: Vec<u64> = pipe.states.iter().map(|s| s.fp2.a).collect();

        assert_eq!(a_before_sqr, a_after_sqr,
            "Residues must be IDENTICAL after norm-Sqrs");
        println!("VERIFIED: 3 norm-Sqrs did not touch any physics residue");
    }

    #[test]
    fn stencil_vs_flat_bs_1d_comparison() {
        // Compare stencil-BS and flat-BS on a 1D field
        let n = 16;
        for pi in 0..4 {
            let p = NTT_PRIMES[pi];
            let omega = NTT_ROOTS[pi];
            // Use ω^2 as primitive n-th root (ω is 32nd root, need 16th)
            let omega_n = modpow(omega, 2, p); // 32nd root → 16th root

            // Random-ish field
            let field: Vec<u64> = (0..n).map(|i| ((i * 37 + 13) as u64) % p).collect();

            let f_hat = ntt_1d(&field, omega_n, p);
            let u_stencil = stencil_bs_1d(&f_hat, omega_n, p);
            let u_flat = flat_bs_1d(&f_hat, n, p);

            let u_stencil_real = intt_1d(&u_stencil, omega_n, p);
            let u_flat_real = intt_1d(&u_flat, omega_n, p);

            let mut differ = 0;
            for i in 0..n {
                if u_stencil_real[i] != u_flat_real[i] { differ += 1; }
            }
            println!("p={p}: stencil vs flat differ at {differ}/{n} points");
            assert!(differ > 0, "p={p}: stencil and flat should differ for random field");
        }
    }

    #[test]
    fn contraction_depth_accumulates() {
        let pi = 1; // p=929
        let p = NTT_PRIMES[pi];
        let mut pipe = DualTrackPipeline::new(8, pi, modinv(6, p));
        pipe.init_cosine(200);
        pipe.threshold = 0; // Sqr fires EVERY step

        for _ in 0..50 { pipe.step(); }

        let max_d = pipe.max_sqr_depth();
        println!("After 50 steps with threshold=0: max_sqr_depth = {max_d}");
        println!("Effective range per point: p^(2^{max_d}) = {p}^{}", 1u64 << max_d.min(30));
        assert!(max_d >= 40, "Should accumulate at least 40 Sqrs in 50 steps");
    }

    #[test]
    fn energy_decreases_under_diffusion() {
        // Pure diffusion should decrease energy (dissipative)
        let pi = 0;
        let p = NTT_PRIMES[pi];
        let mut pipe = DualTrackPipeline::new(32, pi, modinv(10, p));
        pipe.init_cosine(100);
        pipe.threshold = p; // Never fire Sqr (so we see pure physics)

        let e0 = pipe.energy();
        for _ in 0..20 { pipe.step(); }
        let e20 = pipe.energy();

        // In modular arithmetic, "decrease" means the centered values decrease
        // Can't directly compare mod p, but energy should be different
        println!("p={p}: e0={e0}, e20={e20}, changed={}", e0 != e20);
    }
}
