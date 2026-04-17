//! # NSGrid3D: 3D Navier-Stokes on Exact Integer Substrate
//!
//! Implements the semi-discrete vorticity equation on the periodic torus T_N^3:
//!
//!   d/dt Ω^N = -(u^N · ∇_h) Ω^N + ν Δ_h Ω^N + (Ω^N · ∇_h) u^N
//!
//! where u^N = B_h(Ω^N / SCALE) is recovered at each timestep via the discrete
//! Biot-Savart operator (Section 5 of THREE_MISSING_LEMMAS).
//!
//! All arithmetic is exact integer. Zero floating-point in the solver.

/// Scale factor for all physical quantities.
/// physical_value = integer_value / SCALE
pub const SCALE: i64 = 10_000;
pub const SCALE_I128: i128 = SCALE as i128;

/// Base amplitude for Taylor-Green initial data.
pub const TG_BASE_AMP: i64 = SCALE; // amplitude = 1.0 in physical units

/// A single grid cell storing vorticity (primary) and velocity (derived).
#[derive(Debug, Clone)]
pub struct NSCell3D {
    /// Vorticity components (scaled by SCALE)
    pub w_x: i64,
    pub w_y: i64,
    pub w_z: i64,
    /// Velocity components (scaled by SCALE, derived from Biot-Savart)
    pub u_x: i64,
    pub u_y: i64,
    pub u_z: i64,
}

impl NSCell3D {
    pub fn zero() -> Self {
        NSCell3D { w_x: 0, w_y: 0, w_z: 0, u_x: 0, u_y: 0, u_z: 0 }
    }
}

/// 3D Navier-Stokes grid on the periodic torus T_N^3.
#[derive(Debug, Clone)]
pub struct NSGrid3D {
    /// Grid points per axis (N). Must be even, ≥ 2.
    pub n: usize,
    /// Number of CRT lanes (for metadata; actual CRT is in future integration).
    pub safe_basis_count: usize,
    /// Timestep (scaled by SCALE): physical dt = dt_scaled / SCALE.
    pub dt_scaled: i64,
    /// Viscosity (scaled by SCALE²): physical ν = nu_scaled / SCALE².
    pub nu_scaled: i64,
    /// Grid cells, indexed as cells[ix * n * n + iy * n + iz].
    pub cells: Vec<NSCell3D>,
    /// Current timestep number.
    pub step: u64,
}

impl NSGrid3D {
    /// Create a new grid with N points per axis.
    ///
    /// # Arguments
    /// * `n` - Grid points per axis (must be even, ≥ 2, ≤ 16)
    /// * `safe_basis_count` - Number of CRT lanes (metadata)
    /// * `dt_scaled` - Timestep × SCALE
    pub fn new(n: usize, safe_basis_count: usize, dt_scaled: i64) -> Self {
        assert!(n >= 2 && n <= 16, "N must be in [2, 16]");
        assert!(n % 2 == 0, "N must be even");
        let n3 = n * n * n;
        NSGrid3D {
            n,
            safe_basis_count,
            dt_scaled,
            // ν = 1/Re. For Re=100 on unit domain: ν = 0.01 = 100/SCALE²
            // nu_scaled = 100 means ν = 100/10000² = 10⁻⁶ ... too small.
            // Actually: nu_scaled is ν * SCALE² so that ν * Δ Ω has units of SCALE.
            // For ν = 0.01 (Re=100): nu_scaled = 0.01 * SCALE² = 1,000,000
            nu_scaled: SCALE as i64 * SCALE as i64 / 100, // ν = 1/100
            cells: vec![NSCell3D::zero(); n3],
            step: 0,
        }
    }

    /// Periodic index wrapping.
    #[inline]
    pub fn wrap(&self, i: isize) -> usize {
        ((i % self.n as isize) + self.n as isize) as usize % self.n
    }

    /// Flat index from 3D coordinates.
    #[inline]
    pub fn idx(&self, ix: usize, iy: usize, iz: usize) -> usize {
        ix * self.n * self.n + iy * self.n + iz
    }

    /// Initialize with 3D Taylor-Green vortex.
    ///
    /// ω_x =  A cos(kx·x) sin(ky·y) sin(kz·z)
    /// ω_y = -A sin(kx·x) cos(ky·y) sin(kz·z)  (not used here: simplified)
    /// ω_z = 0
    ///
    /// Uses integer sine/cosine lookup table for exact values on the grid.
    pub fn init_taylor_green(&mut self, amp: i64, kx: usize, ky: usize) {
        let n = self.n;

        // Precompute integer sin/cos table: sin(2πk·m/N) * SCALE
        // For small N, these are exact (up to SCALE quantization).
        let sin_tab: Vec<i64> = (0..n).map(|m| {
            let angle = 2.0 * std::f64::consts::PI * m as f64 / n as f64;
            (angle.sin() * SCALE as f64).round() as i64
        }).collect();

        let cos_tab: Vec<i64> = (0..n).map(|m| {
            let angle = 2.0 * std::f64::consts::PI * m as f64 / n as f64;
            (angle.cos() * SCALE as f64).round() as i64
        }).collect();

        for ix in 0..n {
            for iy in 0..n {
                for iz in 0..n {
                    let idx = self.idx(ix, iy, iz);

                    // sin(kx * 2π ix / N) etc.
                    let sx = sin_tab[(kx * ix) % n];
                    let cx = cos_tab[(kx * ix) % n];
                    let sy = sin_tab[(ky * iy) % n];
                    let cy = cos_tab[(ky * iy) % n];
                    let sz = sin_tab[(1 * iz) % n]; // kz = 1
                    let _cz = cos_tab[(1 * iz) % n];

                    // ω_x = A * cos(kx·x) * sin(ky·y) * sin(kz·z)
                    // All products divided by SCALE to keep units consistent
                    let wx = ((amp as i128 * cx as i128 / SCALE_I128)
                              * sy as i128 / SCALE_I128
                              * sz as i128 / SCALE_I128) as i64;

                    // ω_y = -A * sin(kx·x) * cos(ky·y) * sin(kz·z)
                    let wy = -(((amp as i128 * sx as i128 / SCALE_I128)
                                * cy as i128 / SCALE_I128
                                * sz as i128 / SCALE_I128) as i64);

                    // ω_z = 0 (divergence-free condition)
                    self.cells[idx].w_x = wx;
                    self.cells[idx].w_y = wy;
                    self.cells[idx].w_z = 0;
                }
            }
        }
    }

    /// Global sup-norm of vorticity: M(t) = max_{j,α} |Ω^α_j(t)|.
    pub fn vort_sup_global(&self) -> i64 {
        self.cells.iter().map(|c| {
            c.w_x.abs().max(c.w_y.abs()).max(c.w_z.abs())
        }).max().unwrap_or(0)
    }

    /// Per-component sup-norm of vorticity: comp 0=w_x, 1=w_y, 2=w_z
    pub fn vort_sup(&self, comp: usize) -> i64 {
        self.cells.iter().map(|c| match comp {
            0 => c.w_x.abs(),
            1 => c.w_y.abs(),
            _ => c.w_z.abs(),
        }).max().unwrap_or(0)
    }

    /// Global sup-norm of velocity.
    pub fn vel_sup_global(&self) -> i64 {
        self.cells.iter().map(|c| {
            c.u_x.abs().max(c.u_y.abs()).max(c.u_z.abs())
        }).max().unwrap_or(0)
    }

    /// Advance one timestep using the corrected algorithm from Section 5.3:
    ///
    /// 1. u^N(t) = B_h(Ω^N(t)) — done externally via biot_savart_update()
    /// 2. F = -(u·∇)Ω + ν·ΔΩ + (Ω·∇)u
    /// 3. Ω(t+dt) = Ω(t) + dt·F/SCALE
    ///
    /// NOTE: Caller MUST call biot_savart_update() before advance() to set
    /// the velocity field. If Biot-Savart is not yet integrated, this uses
    /// the existing velocity field (for testing infrastructure).
    pub fn advance(&mut self) {
        let n = self.n;
        let n_i = n as i64;
        let dt = self.dt_scaled;
        let nu = self.nu_scaled;

        // Snapshot current state for stencil reads
        let snap = self.cells.clone();

        for ix in 0..n {
            for iy in 0..n {
                for iz in 0..n {
                    let idx = self.idx(ix, iy, iz);
                    let c = &snap[idx];

                    // Neighbor indices (periodic)
                    let ixp = self.wrap(ix as isize + 1);
                    let ixm = self.wrap(ix as isize - 1);
                    let iyp = self.wrap(iy as isize + 1);
                    let iym = self.wrap(iy as isize - 1);
                    let izp = self.wrap(iz as isize + 1);
                    let izm = self.wrap(iz as isize - 1);

                    // Helper: get cell at grid point
                    let g = |jx: usize, jy: usize, jz: usize| -> &NSCell3D {
                        &snap[jx * n * n + jy * n + jz]
                    };

                    // For each vorticity component α:
                    for comp in 0..3usize {
                        let w_val = match comp {
                            0 => c.w_x,
                            1 => c.w_y,
                            _ => c.w_z,
                        };
                        let get_w = |cell: &NSCell3D| -> i64 {
                            match comp { 0 => cell.w_x, 1 => cell.w_y, _ => cell.w_z }
                        };
                        let get_u = |cell: &NSCell3D, d: usize| -> i64 {
                            match d { 0 => cell.u_x, 1 => cell.u_y, _ => cell.u_z }
                        };

                        // --- Advection: -(u · ∇_h) ω_α ---
                        // Central difference: ∂f/∂x_d ≈ N/2 · (f[+e_d] - f[-e_d])
                        let neighbors = [
                            (ixp, iy, iz, ixm, iy, iz),   // d=0 (x)
                            (ix, iyp, iz, ix, iym, iz),    // d=1 (y)
                            (ix, iy, izp, ix, iy, izm),    // d=2 (z)
                        ];

                        let mut advection: i64 = 0;
                        for (d, &(xp, yp, zp, xm, ym, zm)) in neighbors.iter().enumerate() {
                            let wp = get_w(g(xp, yp, zp));
                            let wm = get_w(g(xm, ym, zm));
                            let u_d = get_u(c, d);
                            // -(u_d * N/2 * (w+ - w-)) / SCALE
                            advection += -((u_d as i128 * n_i as i128 * (wp - wm) as i128)
                                          / (2 * SCALE_I128)) as i64;
                        }

                        // --- Stretching: (ω · ∇_h) u_α ---
                        let mut stretching: i64 = 0;
                        for (d, &(xp, yp, zp, xm, ym, zm)) in neighbors.iter().enumerate() {
                            let up = get_u(g(xp, yp, zp), comp);
                            let um = get_u(g(xm, ym, zm), comp);
                            let w_d = match d { 0 => c.w_x, 1 => c.w_y, _ => c.w_z };
                            // (w_d * N/2 * (u+ - u-)) / SCALE
                            stretching += ((w_d as i128 * n_i as i128 * (up - um) as i128)
                                          / (2 * SCALE_I128)) as i64;
                        }

                        // --- Diffusion: ν Δ_h ω_α ---
                        // 7-point Laplacian: N² Σ_d (f[+e_d] + f[-e_d] - 2f)
                        let lap = n_i * n_i * (
                            get_w(g(ixp, iy, iz)) + get_w(g(ixm, iy, iz))
                          + get_w(g(ix, iyp, iz)) + get_w(g(ix, iym, iz))
                          + get_w(g(ix, iy, izp)) + get_w(g(ix, iy, izm))
                          - 6 * w_val
                        );
                        // ν * Δ ω: nu_scaled * lap / SCALE²
                        let diffusion = ((nu as i128 * lap as i128) / (SCALE_I128 * SCALE_I128)) as i64;

                        // --- Total RHS ---
                        let f_total = advection + stretching + diffusion;

                        // --- Update: Ω(t+dt) = Ω(t) + dt * F / SCALE ---
                        let delta = ((dt as i128 * f_total as i128) / SCALE_I128) as i64;
                        let new_val = w_val + delta;

                        match comp {
                            0 => self.cells[idx].w_x = new_val,
                            1 => self.cells[idx].w_y = new_val,
                            _ => self.cells[idx].w_z = new_val,
                        }
                    }
                }
            }
        }

        self.step += 1;
    }

    /// Compute total enstrophy: Σ_j |ω_j|² (in i128 to avoid overflow).
    pub fn total_enstrophy(&self) -> i128 {
        self.cells.iter().map(|c| {
            c.w_x as i128 * c.w_x as i128
          + c.w_y as i128 * c.w_y as i128
          + c.w_z as i128 * c.w_z as i128
        }).sum()
    }

    /// Compute total kinetic energy: Σ_j |u_j|² / 2 (in i128).
    pub fn total_kinetic_energy(&self) -> i128 {
        self.cells.iter().map(|c| {
            (c.u_x as i128 * c.u_x as i128
           + c.u_y as i128 * c.u_y as i128
           + c.u_z as i128 * c.u_z as i128) / 2
        }).sum()
    }

    /// H¹ seminorm squared: Σ_j |∇ω_j|² (via finite differences).
    /// Returns (numerator, denominator) — denominator = 1 for grid-scale norms.
    pub fn h1_seminorm_sq(&self) -> (i128, i128) {
        let n = self.n;
        let mut total: i128 = 0;
        for ix in 0..n {
            for iy in 0..n {
                for iz in 0..n {
                    for comp in 0..3usize {
                        let idx = ix * n * n + iy * n + iz;
                        let w_c = match comp {
                            0 => self.cells[idx].w_x,
                            1 => self.cells[idx].w_y,
                            _ => self.cells[idx].w_z,
                        };
                        // Forward difference in each spatial direction
                        let ixp = (ix + 1) % n;
                        let iyp = (iy + 1) % n;
                        let izp = (iz + 1) % n;
                        let w_xp = match comp {
                            0 => self.cells[ixp * n * n + iy * n + iz].w_x,
                            1 => self.cells[ixp * n * n + iy * n + iz].w_y,
                            _ => self.cells[ixp * n * n + iy * n + iz].w_z,
                        };
                        let w_yp = match comp {
                            0 => self.cells[ix * n * n + iyp * n + iz].w_x,
                            1 => self.cells[ix * n * n + iyp * n + iz].w_y,
                            _ => self.cells[ix * n * n + iyp * n + iz].w_z,
                        };
                        let w_zp = match comp {
                            0 => self.cells[ix * n * n + iy * n + izp].w_x,
                            1 => self.cells[ix * n * n + iy * n + izp].w_y,
                            _ => self.cells[ix * n * n + iy * n + izp].w_z,
                        };
                        let dx = (w_xp - w_c) as i128;
                        let dy = (w_yp - w_c) as i128;
                        let dz = (w_zp - w_c) as i128;
                        total += dx * dx + dy * dy + dz * dz;
                    }
                }
            }
        }
        (total, 1)
    }

    /// Combined norm measurement: (enstrophy, H¹_num, H¹_den, max_vort)
    pub fn norm_measurement(&self) -> (i128, i128, i128, i64) {
        let l2_sq = self.total_enstrophy();
        let (h1_num, h1_den) = self.h1_seminorm_sq();
        let max_vort = [0,1,2].iter().map(|&c| self.vort_sup(c)).max().unwrap_or(0);
        (l2_sq, h1_num, h1_den, max_vort)
    }

    /// Stretching enstrophy source: Σ_{j,c} ω_c · (ω·∇)u_c
    pub fn stretching_enstrophy_source(&self) -> i128 {
        let n = self.n;
        let mut total: i128 = 0;
        for ix in 0..n {
            for iy in 0..n {
                for iz in 0..n {
                    for comp in 0..3 {
                        let idx = ix * n * n + iy * n + iz;
                        let w_c = match comp {
                            0 => self.cells[idx].w_x,
                            1 => self.cells[idx].w_y,
                            _ => self.cells[idx].w_z,
                        };
                        let s = crate::fused_solver::stretching_3d(
                            &self.cells, n, ix, iy, iz, comp);
                        total += w_c as i128 * s as i128;
                    }
                }
            }
        }
        total
    }

    /// Skew-symmetric advection energy: Σ_{j,c} ω_c · A^skew_c
    pub fn skew_advection_energy(&self) -> i128 {
        let n = self.n;
        let mut total: i128 = 0;
        for ix in 0..n {
            for iy in 0..n {
                for iz in 0..n {
                    for comp in 0..3 {
                        let idx = ix * n * n + iy * n + iz;
                        let w_c = match comp {
                            0 => self.cells[idx].w_x,
                            1 => self.cells[idx].w_y,
                            _ => self.cells[idx].w_z,
                        };
                        let a = crate::fused_solver::skew_advection_3d(
                            &self.cells, n, ix, iy, iz, comp);
                        total += w_c as i128 * a as i128;
                    }
                }
            }
        }
        total
    }

    /// Diffusion enstrophy contribution: Σ_{j,c} ω_c · ν·Δω_c
    pub fn diffusion_enstrophy_contribution(&self) -> i128 {
        let n = self.n;
        let nu = self.nu_scaled;
        let mut total: i128 = 0;
        for ix in 0..n {
            for iy in 0..n {
                for iz in 0..n {
                    for comp in 0..3 {
                        let idx = ix * n * n + iy * n + iz;
                        let w_c = match comp {
                            0 => self.cells[idx].w_x,
                            1 => self.cells[idx].w_y,
                            _ => self.cells[idx].w_z,
                        };
                        let d = crate::fused_solver::diffusion_3d(
                            &self.cells, n, ix, iy, iz, comp, nu);
                        total += w_c as i128 * d as i128;
                    }
                }
            }
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = NSGrid3D::new(8, 6, 100);
        assert_eq!(grid.cells.len(), 512);
        assert_eq!(grid.n, 8);
    }

    #[test]
    fn test_wrap() {
        let grid = NSGrid3D::new(8, 6, 100);
        assert_eq!(grid.wrap(8), 0);
        assert_eq!(grid.wrap(-1), 7);
        assert_eq!(grid.wrap(0), 0);
        assert_eq!(grid.wrap(7), 7);
    }

    #[test]
    fn test_taylor_green_init() {
        let mut grid = NSGrid3D::new(8, 6, 100);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        let m = grid.vort_sup_global();
        assert!(m > 0, "Taylor-Green should produce nonzero vorticity");
        assert!(m <= SCALE, "Vorticity should be bounded by amplitude");
    }

    #[test]
    fn test_divergence_free_initial_vorticity() {
        // ∇·ω = ∂ω_x/∂x + ∂ω_y/∂y + ∂ω_z/∂z should be 0
        let mut grid = NSGrid3D::new(8, 6, 100);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        let n = grid.n;
        let n_i = n as i64;
        let mut max_div: i64 = 0;

        for ix in 0..n {
            for iy in 0..n {
                for iz in 0..n {
                    let ixp = grid.wrap(ix as isize + 1);
                    let ixm = grid.wrap(ix as isize - 1);
                    let iyp = grid.wrap(iy as isize + 1);
                    let iym = grid.wrap(iy as isize - 1);
                    let izp = grid.wrap(iz as isize + 1);
                    let izm = grid.wrap(iz as isize - 1);

                    let div = n_i * (grid.cells[grid.idx(ixp,iy,iz)].w_x
                                   - grid.cells[grid.idx(ixm,iy,iz)].w_x) / 2
                            + n_i * (grid.cells[grid.idx(ix,iyp,iz)].w_y
                                   - grid.cells[grid.idx(ix,iym,iz)].w_y) / 2
                            + n_i * (grid.cells[grid.idx(ix,iy,izp)].w_z
                                   - grid.cells[grid.idx(ix,iy,izm)].w_z) / 2;

                    max_div = max_div.max(div.abs());
                }
            }
        }
        // Should be small (≤ integer rounding error from sin/cos tables)
        assert!(max_div < SCALE / 10,
            "Divergence of initial vorticity too large: {}", max_div);
    }

    #[test]
    fn test_advance_no_blowup() {
        let mut grid = NSGrid3D::new(8, 6, 100);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        let m0 = grid.vort_sup_global();

        for _ in 0..20 {
            grid.advance();
        }

        let m20 = grid.vort_sup_global();
        // Without Biot-Savart, velocity = 0, so only diffusion acts.
        // Vorticity should decrease (diffusion is dissipative).
        assert!(m20 <= m0 + 1, "Vorticity should not grow with diffusion only: {} -> {}", m0, m20);
    }

    #[test]
    fn test_enstrophy_positive() {
        let mut grid = NSGrid3D::new(8, 6, 100);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        let e = grid.total_enstrophy();
        assert!(e > 0, "Enstrophy should be positive for nontrivial data");
    }
}
