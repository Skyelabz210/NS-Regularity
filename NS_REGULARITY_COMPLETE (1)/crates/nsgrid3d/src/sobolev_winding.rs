//! # Sobolev Winding Tracker — K_E and K_H via K-Elimination
//!
//! Extends L^inf winding (K_vort) to L^2 (K_E) and H^1 (K_H).
//! All exact integer. Zero float.

use crate::ns_grid_3d::*;

/// Safe basis product for exact-division lanes {3,7,11,13}
pub const M_SAFE: u64 = 3 * 7 * 11 * 13; // = 3003

/// Mersenne anchor prime
pub const P_ANCHOR: u64 = (1u64 << 31) - 1; // = 2147483647

fn mod_inverse(a: u64, m: u64) -> u64 {
    if m == 1 { return 0; }
    let (mut old_r, mut r) = (a as i128, m as i128);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = old_r / r;
        let tmp = r; r = old_r - q * r; old_r = tmp;
        let tmp = s; s = old_s - q * s; old_s = tmp;
    }
    assert_eq!(old_r, 1);
    ((old_s % m as i128 + m as i128) % m as i128) as u64
}

fn k_eliminate(r_safe: u64, r_anchor: u64, m_safe: u64, p_anchor: u64) -> u64 {
    let inv = mod_inverse(m_safe % p_anchor, p_anchor);
    let diff = if r_anchor >= r_safe % p_anchor {
        r_anchor - r_safe % p_anchor
    } else {
        p_anchor - (r_safe % p_anchor - r_anchor)
    };
    ((diff as u128 * inv as u128) % p_anchor as u128) as u64
}

/// Complete Sobolev state at one timestep
#[derive(Debug, Clone)]
pub struct SobolevState {
    pub step: u64,
    pub enstrophy: i128,
    pub palinstrophy: i128,
    pub max_vort: i64,
    pub stretching_source: i128,
    pub advection_energy: i128,
    pub diffusion_contribution: i128,
    pub k_vort: u64,
    pub k_enstrophy: u64,
    pub k_palinstrophy: u64,
    pub w_composite: u64,
}

/// Sobolev winding tracker with history
pub struct SobolevTracker {
    pub history: Vec<SobolevState>,
}

impl SobolevTracker {
    pub fn new() -> Self { Self { history: Vec::new() } }

    /// Record a measurement from the grid and compute winding numbers.
    pub fn record(&mut self, grid: &NSGrid3D) -> SobolevState {
        let (enstrophy, palinstrophy, _, max_vort) = grid.norm_measurement();
        let stretching_source = grid.stretching_enstrophy_source();
        let advection_energy = grid.skew_advection_energy();
        let diffusion_contribution = grid.diffusion_enstrophy_contribution();

        let e_abs = enstrophy.unsigned_abs();
        let k_e = k_eliminate(
            (e_abs % M_SAFE as u128) as u64,
            (e_abs % P_ANCHOR as u128) as u64,
            M_SAFE, P_ANCHOR,
        );

        let h_abs = palinstrophy.unsigned_abs();
        let k_h = k_eliminate(
            (h_abs % M_SAFE as u128) as u64,
            (h_abs % P_ANCHOR as u128) as u64,
            M_SAFE, P_ANCHOR,
        );

        let k_vort = max_vort.unsigned_abs() / M_SAFE as u64;

        let state = SobolevState {
            step: grid.step,
            enstrophy, palinstrophy, max_vort,
            stretching_source, advection_energy, diffusion_contribution,
            k_vort, k_enstrophy: k_e, k_palinstrophy: k_h,
            w_composite: k_vort + k_e + k_h,
        };
        self.history.push(state.clone());
        state
    }

    /// C1_E norm equivalence constant
    pub fn c1_enstrophy(&self, idx: usize) -> f64 {
        let s = &self.history[idx];
        if s.enstrophy <= 0 { return 0.0; }
        s.k_enstrophy as f64 * M_SAFE as f64 / s.enstrophy as f64
    }

    /// Summary certificate
    pub fn certificate_summary(&self) -> (bool, bool, bool, bool) {
        let adv_zero = self.history.iter().all(|s| s.advection_energy == 0);
        let diff_nonpos = self.history.iter().all(|s| s.diffusion_contribution <= 0);
        let k_bounded = self.history.windows(2)
            .all(|w| w[1].k_enstrophy <= w[0].k_enstrophy + 1);
        let mono = self.history.windows(2)
            .all(|w| w[1].enstrophy <= w[0].enstrophy);
        (adv_zero, diff_nonpos, k_bounded, mono)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k_enstrophy_equals_floor() {
        for &n in &[4usize, 8] {
            let mut grid = NSGrid3D::new(n, 6, 100);
            grid.init_taylor_green(TG_BASE_AMP, 1, 1);
            let (e, _, _, _) = grid.norm_measurement();
            let e_abs = e.unsigned_abs();
            let k_floor = (e_abs / M_SAFE as u128) as u64;
            let k_elim = k_eliminate(
                (e_abs % M_SAFE as u128) as u64,
                (e_abs % P_ANCHOR as u128) as u64,
                M_SAFE, P_ANCHOR,
            );
            assert_eq!(k_floor, k_elim, "N={}: floor={}, K-Elim={}", n, k_floor, k_elim);
        }
    }

    #[test]
    fn enstrophy_positive() {
        let mut grid = NSGrid3D::new(8, 6, 100);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        let (e, h, _, mv) = grid.norm_measurement();
        assert!(e > 0); assert!(h > 0); assert!(mv > 0);
    }

    #[test]
    fn diffusion_nonpositive() {
        let mut grid = NSGrid3D::new(8, 6, 100);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        let d = grid.diffusion_enstrophy_contribution();
        assert!(d <= 0, "diffusion contribution = {} must be <= 0", d);
    }

    #[test]
    fn advection_zero_static() {
        // With u=0 everywhere, advection energy should be zero
        let mut grid = NSGrid3D::new(8, 6, 100);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        // u is zero (no Biot-Savart applied)
        let ae = grid.skew_advection_energy();
        assert_eq!(ae, 0, "advection energy with zero velocity = {}", ae);
    }

    #[test]
    fn enstrophy_scales_n_cubed() {
        let mut g8 = NSGrid3D::new(8, 6, 100);
        let mut g16 = NSGrid3D::new(16, 6, 100);
        g8.init_taylor_green(TG_BASE_AMP, 1, 1);
        g16.init_taylor_green(TG_BASE_AMP, 1, 1);
        let (e8, _, _, _) = g8.norm_measurement();
        let (e16, _, _, _) = g16.norm_measurement();
        let ratio = e16 as f64 / e8 as f64;
        assert!(ratio > 6.0 && ratio < 10.0, "E(16)/E(8) = {:.2}", ratio);
    }
}
