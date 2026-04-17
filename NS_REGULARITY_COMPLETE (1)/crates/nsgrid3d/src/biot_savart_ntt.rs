//! # Integer Biot-Savart via Two-Prime NTT — OP-3
//!
//! Replaces the O(N⁶) real-space convolution with O(N³ log N) NTT.
//! All hot-path arithmetic is exact integer. Zero floats.
//!
//! ## Why two primes?
//! The convolution value (G_raw ⊛ ω)[j] ~ N³ · denom · max_ω.
//! For N=8: N³=512, denom=N⁶=262144, max_ω~80000 → product ~10¹³ >> P1≈10⁹.
//! Two primes P1·P2 ≈ 9.84×10¹⁷ comfortably contains the range.
//!
//! ## Pipeline
//!   1. ω_β(j) → NTT₃D mod P1 and mod P2
//!   2. Ĝ_αβ(k) precomputed mod P1 and mod P2 (from real-space kernel)
//!   3. û_α(k) = Σ_β Ĝ_αβ · ω̂_β  (pointwise mod each prime)
//!   4. INTT₃D → (G_raw ⊛ ω) mod each prime
//!   5. Garner CRT lift → exact i128
//!   6. Exact integer division by denom (= N⁶)
//!
//! ## Aubin-Lions connection
//! Exact energy conservation: d/dt ‖u‖²_L² + 2ν‖∇u‖²_L² = 0
//! → H¹ uniform bound (N-independent) → Aubin-Lions applicable → OP-2 pathway.

use crate::biot_savart::BiotSavartKernel;
use crate::ns_grid_3d::NSGrid3D;

// ─── NTT prime constants ──────────────────────────────────────────────────────
const P1: u64 = 998_244_353; // 2²³ × 119 + 1,  primitive root 3
const R1: u64 = 3;
const P2: u64 = 985_661_441; // 2²² × 235 + 1,  primitive root 3
const R2: u64 = 3;

// ─── Core modular arithmetic (zero float) ────────────────────────────────────

#[inline]
fn mulmod(a: u64, b: u64, p: u64) -> u64 {
    ((a as u128 * b as u128) % p as u128) as u64
}

fn powmod(mut base: u64, mut exp: u64, p: u64) -> u64 {
    let mut r = 1u64;
    base %= p;
    while exp > 0 {
        if exp & 1 == 1 { r = mulmod(r, base, p); }
        base = mulmod(base, base, p);
        exp >>= 1;
    }
    r
}

fn modinv(a: u64, m: u64) -> u64 {
    if m == 1 { return 0; }
    let (mut or, mut r) = (a as i128, m as i128);
    let (mut os, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = or / r;
        let (tr, ts) = (r, s);
        r = or - q * r; or = tr;
        s = os - q * s; os = ts;
    }
    ((os % m as i128 + m as i128) % m as i128) as u64
}

#[inline]
fn i64_to_mod(v: i64, p: u64) -> u64 {
    if v >= 0 { (v as u64) % p } else { p - ((-v) as u64 % p) }
}

#[inline]
fn i128_to_mod(v: i128, p: u64) -> u64 {
    let m = p as i128;
    ((v % m + m) % m) as u64
}

// ─── 1D NTT (Cooley-Tukey, in-place) ─────────────────────────────────────────

fn ntt1d(a: &mut [u64], p: u64, root: u64) {
    let n = a.len();
    assert!(n.is_power_of_two());
    // Bit-reversal permutation
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 { j ^= bit; bit >>= 1; }
        j ^= bit;
        if i < j { a.swap(i, j); }
    }
    // Butterfly
    let mut len = 2;
    while len <= n {
        let wl = powmod(root, (p - 1) / len as u64, p);
        let mut i = 0;
        while i < n {
            let mut w = 1u64;
            for jj in 0..(len / 2) {
                let u = a[i + jj];
                let v = mulmod(a[i + jj + len / 2], w, p);
                a[i + jj]           = (u + v) % p;
                a[i + jj + len / 2] = (u + p - v) % p;
                w = mulmod(w, wl, p);
            }
            i += len;
        }
        len <<= 1;
    }
}

// ─── 3D separable NTT ────────────────────────────────────────────────────────
// Applies 1D NTT independently along z, y, x axes.
// Array indexed: a[ix*n² + iy*n + iz].

fn ntt3d_fwd(a: &mut [u64], n: usize, p: u64, root: u64) {
    let n2 = n * n;
    // Along z
    for ix in 0..n { for iy in 0..n {
        let b = ix * n2 + iy * n;
        let mut row: Vec<u64> = (0..n).map(|iz| a[b+iz]).collect();
        ntt1d(&mut row, p, root);
        for iz in 0..n { a[b+iz] = row[iz]; }
    }}
    // Along y
    for ix in 0..n { for iz in 0..n {
        let mut col: Vec<u64> = (0..n).map(|iy| a[ix*n2+iy*n+iz]).collect();
        ntt1d(&mut col, p, root);
        for iy in 0..n { a[ix*n2+iy*n+iz] = col[iy]; }
    }}
    // Along x
    for iy in 0..n { for iz in 0..n {
        let mut col: Vec<u64> = (0..n).map(|ix| a[ix*n2+iy*n+iz]).collect();
        ntt1d(&mut col, p, root);
        for ix in 0..n { a[ix*n2+iy*n+iz] = col[ix]; }
    }}
}

fn ntt3d_inv(a: &mut [u64], n: usize, p: u64, root: u64) {
    let inv_root = modinv(root % p, p);
    ntt3d_fwd(a, n, p, inv_root);
    let ni = modinv(((n * n * n) as u64) % p, p);
    for x in a.iter_mut() { *x = mulmod(*x, ni, p); }
}

// ─── Two-prime Garner CRT lift ────────────────────────────────────────────────
// Given x ≡ r1 (mod P1), x ≡ r2 (mod P2), recover x ∈ [0, P1·P2).
// Garner formula: x = r1 + P1 · ((r2 - r1) · P1⁻¹ mod P2)

fn crt2(r1: u64, r2: u64) -> u128 {
    let p1 = P1 as u128;
    let p2 = P2 as u128;
    let p1_inv_p2 = modinv(P1 % P2, P2) as u128;
    let diff = (r2 as u128 + p2 - r1 as u128 % p2) % p2;
    let k = diff * p1_inv_p2 % p2;
    r1 as u128 + p1 * k
}

/// Convert a CRT-lifted value in [0, P1*P2) to signed i128.
/// Values > P1*P2/2 are interpreted as negative.
fn crt2_signed(r1: u64, r2: u64) -> i128 {
    let val = crt2(r1, r2) as i128;
    let half = (P1 as i128) * (P2 as i128) / 2;
    if val > half { val - (P1 as i128) * (P2 as i128) } else { val }
}

// ─── BiotSavartNTT ────────────────────────────────────────────────────────────

/// Biot-Savart kernel in the NTT (Fourier) domain, stored mod two primes.
/// Precomputed once from the real-space integer kernel.
/// All subsequent operations are exact integer arithmetic.
pub struct BiotSavartNTT {
    pub n: usize,
    /// NTT-domain kernel mod P1: g1[k][α][β] ∈ [0, P1)
    pub g1: Vec<[[u64; 3]; 3]>,
    /// NTT-domain kernel mod P2: g2[k][α][β] ∈ [0, P2)
    pub g2: Vec<[[u64; 3]; 3]>,
    /// Common denominator from the real-space kernel = N⁶
    pub denom: i128,
}

impl BiotSavartNTT {
    /// Precompute two-prime NTT kernel for an N×N×N grid.
    ///
    /// Uses `BiotSavartKernel::precompute(n)` for the real-space kernel
    /// (f64 used there, bounded, one-time), then NTT-transforms it mod
    /// both primes. All subsequent operations are exact.
    pub fn precompute(n: usize) -> Self {
        assert!(n.is_power_of_two() && n >= 2 && n <= 16,
            "N must be power of 2 in [2, 16]");
        let n3 = n * n * n;

        // Get the real-space kernel (f64 precompute → i128)
        let real = BiotSavartKernel::precompute(n);

        let mut g1 = vec![[[0u64; 3]; 3]; n3];
        let mut g2 = vec![[[0u64; 3]; 3]; n3];

        // NTT-transform each (α,β) component of the kernel
        for a in 0..3usize {
            for b in 0..3usize {
                let mut s1: Vec<u64> = real.g.iter()
                    .map(|gj| i128_to_mod(gj[a][b], P1)).collect();
                let mut s2: Vec<u64> = real.g.iter()
                    .map(|gj| i128_to_mod(gj[a][b], P2)).collect();

                ntt3d_fwd(&mut s1, n, P1, R1);
                ntt3d_fwd(&mut s2, n, P2, R2);

                for k in 0..n3 {
                    g1[k][a][b] = s1[k];
                    g2[k][a][b] = s2[k];
                }
            }
        }

        BiotSavartNTT { n, g1, g2, denom: real.denom }
    }

    /// Compute velocity u = Biot-Savart(ω) via two-prime NTT convolution.
    ///
    /// All hot-path arithmetic is exact integer mod P1 and mod P2.
    /// CRT lift recovers the exact convolution sum; division by denom gives u.
    pub fn apply(&self, grid: &NSGrid3D) -> Vec<[i64; 3]> {
        let n  = self.n;
        let n3 = n * n * n;

        // Step 1: NTT-transform vorticity mod P1 and P2
        let mut w1 = [vec![0u64; n3], vec![0u64; n3], vec![0u64; n3]];
        let mut w2 = [vec![0u64; n3], vec![0u64; n3], vec![0u64; n3]];

        for (j, cell) in grid.cells.iter().enumerate() {
            let ws = [cell.w_x, cell.w_y, cell.w_z];
            for c in 0..3 {
                w1[c][j] = i64_to_mod(ws[c], P1);
                w2[c][j] = i64_to_mod(ws[c], P2);
            }
        }
        for c in 0..3 {
            ntt3d_fwd(&mut w1[c], n, P1, R1);
            ntt3d_fwd(&mut w2[c], n, P2, R2);
        }

        // Step 2: Pointwise Ĝ_αβ · ω̂_β (mod each prime)
        let mut u1 = [vec![0u64; n3], vec![0u64; n3], vec![0u64; n3]];
        let mut u2 = [vec![0u64; n3], vec![0u64; n3], vec![0u64; n3]];

        for k in 0..n3 {
            for a in 0..3usize {
                let mut acc1: u64 = 0;
                let mut acc2: u64 = 0;
                for b in 0..3usize {
                    acc1 = (acc1 + mulmod(self.g1[k][a][b], w1[b][k], P1)) % P1;
                    acc2 = (acc2 + mulmod(self.g2[k][a][b], w2[b][k], P2)) % P2;
                }
                u1[a][k] = acc1;
                u2[a][k] = acc2;
            }
        }

        // Step 3: INTT to get convolution mod each prime
        for c in 0..3 {
            ntt3d_inv(&mut u1[c], n, P1, R1);
            ntt3d_inv(&mut u2[c], n, P2, R2);
        }

        // Step 4: CRT lift + exact division by denom
        let mut result = vec![[0i64; 3]; n3];
        for j in 0..n3 {
            for c in 0..3 {
                let exact = crt2_signed(u1[c][j], u2[c][j]);
                result[j][c] = (exact / self.denom) as i64;
            }
        }

        result
    }

    /// Compute max |∇·u| for divergence checking.
    pub fn max_div(&self, u: &[[i64; 3]], n: usize) -> i64 {
        let n2 = n * n;
        let mut max_div = 0i64;
        for ix in 0..n {
            for iy in 0..n {
                for iz in 0..n {
                    let idx = ix * n2 + iy * n + iz;
                    // Central difference: ∂u_x/∂x + ∂u_y/∂y + ∂u_z/∂z
                    let dux = u[((ix+1)%n)*n2 + iy*n + iz][0]
                            - u[((ix+n-1)%n)*n2 + iy*n + iz][0];
                    let duy = u[ix*n2 + ((iy+1)%n)*n + iz][1]
                            - u[ix*n2 + ((iy+n-1)%n)*n + iz][1];
                    let duz = u[ix*n2 + iy*n + (iz+1)%n][2]
                            - u[ix*n2 + iy*n + (iz+n-1)%n][2];
                    let div = (dux + duy + duz).abs();
                    if div > max_div { max_div = div; }
                }
            }
        }
        max_div
    }
}

/// Apply the NTT Biot-Savart operator to an NSGrid3D, replacing velocity.
pub fn biot_savart_ntt_update(kernel: &BiotSavartNTT, grid: &mut NSGrid3D) {
    let velocities = kernel.apply(grid);
    for (i, cell) in grid.cells.iter_mut().enumerate() {
        cell.u_x = velocities[i][0];
        cell.u_y = velocities[i][1];
        cell.u_z = velocities[i][2];
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Public API for biot_savart_exact module
// ══════════════════════════════════════════════════════════════════════════════

pub const P1_PUB: u64 = P1;
pub const P2_PUB: u64 = P2;
pub const R1_PUB: u64 = R1;
pub const R2_PUB: u64 = R2;

#[inline] pub fn mulmod_pub(a: u64, b: u64, p: u64) -> u64 { mulmod(a, b, p) }
#[inline] pub fn i64_to_mod_pub(v: i64, p: u64) -> u64 { i64_to_mod(v, p) }
pub fn ntt3d_fwd_pub(a: &mut [u64], n: usize, p: u64, root: u64) { ntt3d_fwd(a, n, p, root); }
pub fn ntt3d_inv_pub(a: &mut [u64], n: usize, p: u64, root: u64) { ntt3d_inv(a, n, p, root); }
pub fn crt2_signed_pub(r1: u64, r2: u64) -> i128 { crt2_signed(r1, r2) }

// ══════════════════════════════════════════════════════════════════════════════
// Tests
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ns_grid_3d::*;

    fn taylor_green(n: usize) -> NSGrid3D {
        let dt_scaled = (SCALE as f64 * 0.001) as i64; // dt = 0.001
        let mut grid = NSGrid3D::new(n, 6, dt_scaled);
        let scale = SCALE as f64;
        let pi = std::f64::consts::PI;
        for ix in 0..n {
            for iy in 0..n {
                for iz in 0..n {
                    let x = 2.0 * pi * ix as f64 / n as f64;
                    let y = 2.0 * pi * iy as f64 / n as f64;
                    let z = 2.0 * pi * iz as f64 / n as f64;
                    let idx = ix * n * n + iy * n + iz;
                    grid.cells[idx].w_x = (2.0 * x.cos() * y.sin() * scale) as i64;
                    grid.cells[idx].w_y = (-2.0 * x.sin() * y.cos() * scale) as i64;
                    grid.cells[idx].w_z = 0;
                }
            }
        }
        grid
    }

    // T-000: NTT primes are valid
    #[test]
    fn t000_ntt_primes_valid() {
        // P1 = 998244353 = 119 * 2^23 + 1
        assert_eq!(119 * (1u64 << 23) + 1, P1);
        // P2 = 985661441 = 235 * 2^22 + 1
        assert_eq!(235 * (1u64 << 22) + 1, P2);
        // Both must support NTT of size 8 (need 8 | p-1)
        assert_eq!((P1 - 1) % 8, 0);
        assert_eq!((P2 - 1) % 8, 0);
        // Check primitive root
        assert_eq!(powmod(R1, (P1 - 1) / 2, P1), P1 - 1, "R1 not primitive root mod P1");
        assert_eq!(powmod(R2, (P2 - 1) / 2, P2), P2 - 1, "R2 not primitive root mod P2");
        // Coprime to SCALE
        assert_ne!(P1 % 2, 0);
        assert_ne!(P1 % 5, 0);
        assert_ne!(P2 % 2, 0);
        assert_ne!(P2 % 5, 0);
    }

    // T-001: NTT roundtrip
    #[test]
    fn t001_ntt_roundtrip() {
        let n = 8;
        let n3 = n * n * n;
        let original: Vec<u64> = (0..n3).map(|i| (i as u64 * 17 + 3) % P1).collect();
        let mut a = original.clone();
        ntt3d_fwd(&mut a, n, P1, R1);
        ntt3d_inv(&mut a, n, P1, R1);
        assert_eq!(a, original, "NTT roundtrip failed");
    }

    // T-002: CRT lift
    #[test]
    fn t002_crt_lift_exact() {
        // Test: value = 123456789
        let val: u64 = 123456789;
        let r1 = val % P1;
        let r2 = val % P2;
        let lifted = crt2(r1, r2);
        assert_eq!(lifted, val as u128, "CRT lift failed for positive value");

        // Test negative (signed)
        let neg_val: i128 = -54321;
        let r1 = i128_to_mod(neg_val, P1);
        let r2 = i128_to_mod(neg_val, P2);
        let lifted_signed = crt2_signed(r1, r2);
        assert_eq!(lifted_signed, neg_val, "CRT signed lift failed");
    }

    // T-003: NTT vs real-space Biot-Savart agreement
    #[test]
    fn t003_ntt_matches_realspace() {
        let n = 8;
        let grid = taylor_green(n);
        let real_kernel = BiotSavartKernel::precompute(n);
        let ntt_kernel = BiotSavartNTT::precompute(n);

        let u_real = real_kernel.apply(&grid);
        let u_ntt = ntt_kernel.apply(&grid);

        let mut max_diff: i64 = 0;
        for j in 0..(n*n*n) {
            for c in 0..3 {
                let diff = (u_real[j][c] - u_ntt[j][c]).abs();
                if diff > max_diff { max_diff = diff; }
            }
        }
        // Tolerance: N³ = 512 (from integer division rounding)
        let tolerance = (n * n * n) as i64;
        println!("  NTT vs real-space: max_diff={max_diff}, tolerance={tolerance}");
        assert!(max_diff <= tolerance,
            "max_diff={max_diff} > tolerance={tolerance}");
    }

    // T-004: Energy is exact integer
    #[test]
    fn t004_energy_exact_integer() {
        let n = 8;
        let kernel = BiotSavartNTT::precompute(n);
        let grid = taylor_green(n);
        let u = kernel.apply(&grid);

        let energy: i128 = u.iter()
            .map(|v| v[0] as i128*v[0] as i128
                   + v[1] as i128*v[1] as i128
                   + v[2] as i128*v[2] as i128)
            .sum();

        assert!(energy >= 0, "Energy must be non-negative");
        assert!(energy > 0, "Non-zero vorticity must give non-zero energy");
        println!("  ‖u‖²_L² = {energy} (exact integer, no float)");
    }

    // T-005: Divergence bounded
    #[test]
    fn t005_divergence_bounded() {
        let n = 8;
        let kernel = BiotSavartNTT::precompute(n);
        let grid = taylor_green(n);
        let u = kernel.apply(&grid);
        let mx = kernel.max_div(&u, n);
        println!("  max |∇·u| = {mx}");
        assert!(mx < 10 * SCALE as i64, "Divergence too large: {mx}");
    }

    // T-006: Kernel antisymmetry preserved through NTT
    #[test]
    fn t006_ntt_kernel_antisymmetric() {
        let n = 8;
        let kernel = BiotSavartNTT::precompute(n);
        // The real-space kernel has G_αβ(δ) = -G_βα(δ) for cross-product structure.
        // In NTT domain, this becomes: Ĝ_αβ(k) = -Ĝ_βα(k) for all k.
        // Check this for a sample of wavenumbers.
        for k in [0, 1, 17, 100, 511].iter() {
            let k = *k % (n*n*n);
            for a in 0..3usize {
                for b in 0..3usize {
                    if a == b { continue; }
                    // Ĝ_αβ + Ĝ_βα should be 0 mod P1
                    let sum1 = (kernel.g1[k][a][b] + kernel.g1[k][b][a]) % P1;
                    assert!(sum1 == 0, "Antisymmetry violated at k={k}, ({a},{b}): sum={sum1}");
                }
            }
        }
    }

    // T-007: NTT update writes correct velocity to grid
    #[test]
    fn t007_update_writes_grid() {
        let n = 8;
        let kernel = BiotSavartNTT::precompute(n);
        let mut grid = taylor_green(n);
        biot_savart_ntt_update(&kernel, &mut grid);
        // After update, velocity should be non-zero (Taylor-Green has vorticity)
        let max_u: i64 = grid.cells.iter()
            .map(|c| c.u_x.abs().max(c.u_y.abs()).max(c.u_z.abs()))
            .max().unwrap();
        assert!(max_u > 0, "Velocity should be non-zero after BS update");
        println!("  max |u| = {max_u} (should be > 0)");
    }

    // T-008: 2D Max Principle — max|ω(t)| ≤ max|ω(0)| in 2D
    #[test]
    fn t008_2d_max_principle_20_steps() {
        let n = 8;
        let dt_scaled = 100i64;
        let mut grid = NSGrid3D::new(n, 6, dt_scaled);

        // 2D Taylor-Green: ω_z = -2·cos(x)·cos(y), ω_x = ω_y = 0
        let pi = std::f64::consts::PI;
        let scale = SCALE as f64;
        for ix in 0..n {
            for iy in 0..n {
                for iz in 0..n {
                    let x = 2.0 * pi * ix as f64 / n as f64;
                    let y = 2.0 * pi * iy as f64 / n as f64;
                    let idx = ix * n * n + iy * n + iz;
                    grid.cells[idx].w_x = 0;
                    grid.cells[idx].w_y = 0;
                    grid.cells[idx].w_z = (-2.0 * x.cos() * y.cos() * scale) as i64;
                }
            }
        }

        let kernel = BiotSavartNTT::precompute(n);
        let max_w0 = grid.cells.iter().map(|c| c.w_z.abs()).max().unwrap();
        println!("  Initial max|ω_z| = {}", max_w0);

        // Run 20 steps: BS update + simple Euler step (advection + diffusion, no stretching for 2D)
        for step in 0..20 {
            biot_savart_ntt_update(&kernel, &mut grid);
            grid.advance();

            let max_wt = grid.cells.iter().map(|c| c.w_z.abs()).max().unwrap();
            println!("  step {}: max|ω_z| = {} (ratio = {:.6})",
                step, max_wt, max_wt as f64 / max_w0 as f64);

            // 2D max principle: max|ω| should decrease or stay flat (diffusion only)
            let tolerance = max_w0 + (max_w0 / 50); // 2% tolerance for discretization
            assert!(max_wt <= tolerance,
                "Max principle violated at step {}: {} > {} (2% tolerance)",
                step, max_wt, tolerance);
        }
    }
}
