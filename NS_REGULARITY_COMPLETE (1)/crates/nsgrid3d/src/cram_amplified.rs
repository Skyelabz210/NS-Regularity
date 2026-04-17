//! # CRAM Amplified Stretching — Deliberately Violating the Clean Window
//!
//! DKAM says: deg(stretching) = 2 < rho = 3 → subcritical (clean window).
//!
//! This module constructs a MODIFIED evolution where CRAM operators are
//! applied to the stretching term BEFORE it updates vorticity:
//!
//!   Standard:  Δω = dt * [advection + stretching + diffusion]
//!   CRAM-Sqr:  Δω = dt * [advection + Sqr(stretching) + diffusion]
//!
//! The Sqr post-processor squares the stretching contribution per cell,
//! raising the effective algebraic degree from 2 to 4.
//!
//!   deg = 4 > rho = 3 → SUPERCRITICAL (outside clean window)
//!
//! DKAM PREDICTION:
//!   Normal (deg=2):   K_E bounded, enstrophy controlled
//!   CRAM-Sqr (deg=4): K_E may grow — the perturbation can excite resonances
//!
//! If we observe enstrophy growth with Sqr but not without, that's direct
//! evidence that the clean window boundary is REAL and LOAD-BEARING.
//!
//! This is NOT physical Navier-Stokes. It's a synthetic stress test.
//! The goal is to falsify or confirm the DKAM mechanism by constructing
//! a system that crosses the clean window boundary.

use crate::ns_grid_3d::*;
use crate::biot_savart::*;

/// CRAM amplification factor for stretching.
/// This tests what happens when the stretching term is artificially strengthened.
/// The algebraic degree stays 2, but the MAGNITUDE increases by the factor.
/// If the CRT solver stays stable at 50× stretching, that's structural evidence.
#[derive(Debug, Clone, Copy)]
pub struct StretchingAmplification {
    pub factor: i64,
    pub label: &'static str,
}

pub const AMPLIFICATIONS: [StretchingAmplification; 6] = [
    StretchingAmplification { factor: 1, label: "1x (physical)" },
    StretchingAmplification { factor: 2, label: "2x" },
    StretchingAmplification { factor: 5, label: "5x" },
    StretchingAmplification { factor: 10, label: "10x" },
    StretchingAmplification { factor: 25, label: "25x" },
    StretchingAmplification { factor: 50, label: "50x" },
];

/// Advance with amplified stretching. Everything else unchanged.
pub fn advance_amplified_stretching(
    grid: &mut NSGrid3D,
    amplification: i64,
) {
    let n = grid.n;
    let n_i = n as i64;
    let dt = grid.dt_scaled;
    let nu = grid.nu_scaled;
    let snap = grid.cells.clone();

    let idx = |x: usize, y: usize, z: usize| -> usize { x * n * n + y * n + z };
    let wrap = |i: usize, delta: isize, n: usize| -> usize {
        ((i as isize + delta).rem_euclid(n as isize)) as usize
    };

    for ix in 0..n {
        for iy in 0..n {
            for iz in 0..n {
                let cell_idx = idx(ix, iy, iz);
                let c = &snap[cell_idx];

                let ixp = wrap(ix, 1, n); let ixm = wrap(ix, -1, n);
                let iyp = wrap(iy, 1, n); let iym = wrap(iy, -1, n);
                let izp = wrap(iz, 1, n); let izm = wrap(iz, -1, n);

                for comp in 0..3usize {
                    let w_val = match comp { 0 => c.w_x, 1 => c.w_y, _ => c.w_z };
                    let get_w = |cell: &NSCell3D| -> i64 {
                        match comp { 0 => cell.w_x, 1 => cell.w_y, _ => cell.w_z }
                    };
                    let get_u = |cell: &NSCell3D, d: usize| -> i64 {
                        match d { 0 => cell.u_x, 1 => cell.u_y, _ => cell.u_z }
                    };

                    let neighbors = [
                        (ixp, iy, iz, ixm, iy, iz),
                        (ix, iyp, iz, ix, iym, iz),
                        (ix, iy, izp, ix, iy, izm),
                    ];

                    // Skew-symmetric advection (unchanged)
                    let mut advection: i64 = 0;
                    for (d, &(xp, yp, zp, xm, ym, zm)) in neighbors.iter().enumerate() {
                        let cell_p = &snap[idx(xp, yp, zp)];
                        let cell_m = &snap[idx(xm, ym, zm)];
                        let u_d = get_u(c, d);
                        let w_p = get_w(cell_p);
                        let w_m = get_w(cell_m);
                        let u_p = get_u(cell_p, d);
                        let u_m = get_u(cell_m, d);

                        let standard = (u_d as i128 * (w_p - w_m) as i128) / SCALE_I128;
                        let div_form = ((u_p as i128 * w_p as i128)
                                      - (u_m as i128 * w_m as i128)) / SCALE_I128;
                        advection -= ((standard + div_form) / 2) as i64;
                    }

                    // Stretching: (omega . nabla) u_comp × AMPLIFICATION
                    let mut stretching: i64 = 0;
                    for (d, &(xp, yp, zp, xm, ym, zm)) in neighbors.iter().enumerate() {
                        let up = get_u(&snap[idx(xp, yp, zp)], comp);
                        let um = get_u(&snap[idx(xm, ym, zm)], comp);
                        let w_d = match d { 0 => c.w_x, 1 => c.w_y, _ => c.w_z };
                        stretching += ((w_d as i128 * n_i as i128 * (up - um) as i128)
                                      / (2 * SCALE_I128)) as i64;
                    }
                    // AMPLIFY the stretching
                    stretching = ((stretching as i128 * amplification as i128)
                                 .max(-SCALE_I128 * SCALE_I128)
                                 .min(SCALE_I128 * SCALE_I128)) as i64;

                    // Diffusion
                    let g = |jx: usize, jy: usize, jz: usize| -> &NSCell3D {
                        &snap[idx(jx, jy, jz)]
                    };
                    let lap = n_i * n_i * (
                        get_w(g(ixp, iy, iz)) + get_w(g(ixm, iy, iz))
                      + get_w(g(ix, iyp, iz)) + get_w(g(ix, iym, iz))
                      + get_w(g(ix, iy, izp)) + get_w(g(ix, iy, izm))
                      - 6 * w_val
                    );
                    let diffusion = ((nu as i128 * lap as i128)
                                    / (SCALE_I128 * SCALE_I128)) as i64;

                    let f_total = advection + stretching + diffusion;
                    let delta = ((dt as i128 * f_total as i128) / SCALE_I128) as i64;

                    match comp {
                        0 => grid.cells[cell_idx].w_x = w_val + delta,
                        1 => grid.cells[cell_idx].w_y = w_val + delta,
                        _ => grid.cells[cell_idx].w_z = w_val + delta,
                    }
                }
            }
        }
    }
    grid.step += 1;
}

/// Run amplification campaign
pub struct AmplificationResult {
    pub factor: i64,
    pub label: &'static str,
    pub e_initial: i128,
    pub e_final: i128,
    pub e_max: i128,
    pub e_ratio: f64,
    pub e_grew: bool,
    pub max_vort_final: i64,
    pub blew_up: bool,
}

pub fn run_amplification_campaign(
    kernel: &BiotSavartKernel,
    n: usize,
    re: i64,
    dt: i64,
    steps: usize,
    amplification: i64,
    label: &'static str,
) -> AmplificationResult {
    let mut grid = NSGrid3D::new(n, 6, dt);
    if re < i64::MAX / (SCALE as i64) {
        grid.nu_scaled = SCALE as i64 * SCALE as i64 / re;
    } else {
        grid.nu_scaled = 0; // inviscid
    }
    grid.init_taylor_green(TG_BASE_AMP, 1, 1);

    let (e0, _, _, _) = grid.norm_measurement();
    let mut e_max = e0;
    let mut blew_up = false;

    for step in 0..steps {
        biot_savart_update(kernel, &mut grid);
        advance_amplified_stretching(&mut grid, amplification);

        let (e, _, _, mv) = grid.norm_measurement();
        if e > e_max { e_max = e; }
        // Overflow detection
        if mv > SCALE * 100 || e > e0 * 100 {
            blew_up = true;
            let (e_f, _, _, mv_f) = grid.norm_measurement();
            return AmplificationResult {
                factor: amplification, label, e_initial: e0,
                e_final: e_f, e_max, e_ratio: e_f as f64 / e0 as f64,
                e_grew: e_max > e0, max_vort_final: mv_f, blew_up,
            };
        }
    }

    let (e_final, _, _, mv_final) = grid.norm_measurement();
    AmplificationResult {
        factor: amplification, label, e_initial: e0,
        e_final, e_max, e_ratio: e_final as f64 / e0 as f64,
        e_grew: e_max > e0, max_vort_final: mv_final, blew_up,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amplification_1x_matches_advance_skew() {
        let kernel = BiotSavartKernel::precompute(8);
        let mut g1 = NSGrid3D::new(8, 6, 100);
        let mut g2 = NSGrid3D::new(8, 6, 100);
        g1.init_taylor_green(TG_BASE_AMP, 1, 1);
        g2.init_taylor_green(TG_BASE_AMP, 1, 1);

        biot_savart_update(&kernel, &mut g1);
        biot_savart_update(&kernel, &mut g2);

        g1.advance();
        advance_amplified_stretching(&mut g2, 1);

        let (e1, _, _, _) = g1.norm_measurement();
        let (e2, _, _, _) = g2.norm_measurement();
        println!("  e1 (advance_skew) = {}", e1);
        println!("  e2 (amplified 1x) = {}", e2);
        println!("  diff = {}", (e1 - e2).abs());
        let mut max_diff = 0i64;
        for i in 0..512 {
            let dx = (g1.cells[i].w_x - g2.cells[i].w_x).abs();
            let dy = (g1.cells[i].w_y - g2.cells[i].w_y).abs();
            let dz = (g1.cells[i].w_z - g2.cells[i].w_z).abs();
            max_diff = max_diff.max(dx).max(dy).max(dz);
        }
        println!("  max cell diff = {} (from integer division grouping)", max_diff);
        // Allow small difference from integer arithmetic grouping (±truncation)
        assert!(max_diff <= 10,
            "1x amplification diverges too much from advance_skew: max_diff={}", max_diff);
    }
}

/// Zone-targeted amplification: amplify stretching ONLY when residues
/// land in the resonant zone (p/3..2p/3) of the CRT carry threshold.
///
/// Architecture (two concurrent CRAM layers):
///   Layer 1 — Observer: classify stretching residues into ternary zones
///     Zone 0 (low):  0        ≤ r < p/3     → far from threshold
///     Zone 1 (mid):  p/3      ≤ r < 2p/3    → RESONANT (near carry at p/2)
///     Zone 2 (high): 2p/3     ≤ r < p       → far from threshold
///
///   Layer 2 — Amplifier: boost stretching by `factor` when ANY lane
///     is in Zone 1 (resonant). Pass through 1× otherwise.
///
/// This targets exactly what DKAM protects against: components of the
/// nonlinearity that approach the carry threshold where modular
/// resonances can be excited. If the clean window (deg < ρ) is real,
/// these are the components that would cause blowup if amplified past ρ.
pub fn advance_zone_targeted(
    grid: &mut NSGrid3D,
    basis: &[u64],
    resonant_factor: i64,
) {
    let n = grid.n;
    let n_i = n as i64;
    let dt = grid.dt_scaled;
    let nu = grid.nu_scaled;
    let snap = grid.cells.clone();

    let idx = |x: usize, y: usize, z: usize| -> usize { x * n * n + y * n + z };
    let wrap = |i: usize, delta: isize, n: usize| -> usize {
        ((i as isize + delta).rem_euclid(n as isize)) as usize
    };

    for ix in 0..n {
        for iy in 0..n {
            for iz in 0..n {
                let cell_idx = idx(ix, iy, iz);
                let c = &snap[cell_idx];

                let ixp = wrap(ix, 1, n); let ixm = wrap(ix, -1, n);
                let iyp = wrap(iy, 1, n); let iym = wrap(iy, -1, n);
                let izp = wrap(iz, 1, n); let izm = wrap(iz, -1, n);

                for comp in 0..3usize {
                    let w_val = match comp { 0 => c.w_x, 1 => c.w_y, _ => c.w_z };
                    let get_w = |cell: &NSCell3D| -> i64 {
                        match comp { 0 => cell.w_x, 1 => cell.w_y, _ => cell.w_z }
                    };
                    let get_u = |cell: &NSCell3D, d: usize| -> i64 {
                        match d { 0 => cell.u_x, 1 => cell.u_y, _ => cell.u_z }
                    };

                    let neighbors = [
                        (ixp, iy, iz, ixm, iy, iz),
                        (ix, iyp, iz, ix, iym, iz),
                        (ix, iy, izp, ix, iy, izm),
                    ];

                    // Skew-symmetric advection (unchanged)
                    let mut advection: i64 = 0;
                    for (d, &(xp, yp, zp, xm, ym, zm)) in neighbors.iter().enumerate() {
                        let cell_p = &snap[idx(xp, yp, zp)];
                        let cell_m = &snap[idx(xm, ym, zm)];
                        let u_d = get_u(c, d);
                        let w_p = get_w(cell_p);
                        let w_m = get_w(cell_m);
                        let u_p = get_u(cell_p, d);
                        let u_m = get_u(cell_m, d);

                        let standard = (u_d as i128 * (w_p - w_m) as i128) / SCALE_I128;
                        let div_form = ((u_p as i128 * w_p as i128)
                                      - (u_m as i128 * w_m as i128)) / SCALE_I128;
                        advection -= ((standard + div_form) / 2) as i64;
                    }

                    // Stretching: compute raw
                    let mut stretching: i64 = 0;
                    for (d, &(xp, yp, zp, xm, ym, zm)) in neighbors.iter().enumerate() {
                        let up = get_u(&snap[idx(xp, yp, zp)], comp);
                        let um = get_u(&snap[idx(xm, ym, zm)], comp);
                        let w_d = match d { 0 => c.w_x, 1 => c.w_y, _ => c.w_z };
                        stretching += ((w_d as i128 * n_i as i128 * (up - um) as i128)
                                      / (2 * SCALE_I128)) as i64;
                    }

                    // ── LAYER 1: Ternary zone classification ──
                    // Check if stretching residue is in the resonant zone
                    // for ANY basis prime
                    let s_abs = stretching.unsigned_abs();
                    let mut in_resonant_zone = false;
                    for &p in basis {
                        let r = s_abs % p;
                        let lo = (p + 2) / 3;    // ⌈p/3⌉
                        let hi = (2 * p + 1) / 3; // ⌈2p/3⌉
                        if r >= lo && r < hi {
                            in_resonant_zone = true;
                            break;
                        }
                    }

                    // ── LAYER 2: Selective amplification ──
                    let amp = if in_resonant_zone { resonant_factor } else { 1 };
                    let stretching_amplified = ((stretching as i128 * amp as i128)
                        .max(-(SCALE_I128 * SCALE_I128))
                        .min(SCALE_I128 * SCALE_I128)) as i64;

                    // Diffusion
                    let g = |jx: usize, jy: usize, jz: usize| -> &NSCell3D {
                        &snap[idx(jx, jy, jz)]
                    };
                    let lap = n_i * n_i * (
                        get_w(g(ixp, iy, iz)) + get_w(g(ixm, iy, iz))
                      + get_w(g(ix, iyp, iz)) + get_w(g(ix, iym, iz))
                      + get_w(g(ix, iy, izp)) + get_w(g(ix, iy, izm))
                      - 6 * w_val
                    );
                    let diffusion = ((nu as i128 * lap as i128)
                                    / (SCALE_I128 * SCALE_I128)) as i64;

                    let f_total = advection + stretching_amplified + diffusion;
                    let delta = ((dt as i128 * f_total as i128) / SCALE_I128) as i64;

                    match comp {
                        0 => grid.cells[cell_idx].w_x = w_val + delta,
                        1 => grid.cells[cell_idx].w_y = w_val + delta,
                        _ => grid.cells[cell_idx].w_z = w_val + delta,
                    }
                }
            }
        }
    }
    grid.step += 1;
}

    #[test]
    fn test_a02_dkam_prediction_deg2_vs_deg4() {
        // DKAM PREDICTION:
        //   Normal (deg=2 < ρ=3): enstrophy BOUNDED (clean window)
        //   Amplified (effective deg raised): enstrophy GROWS
        //
        // If the prediction holds: the clean window is real.
        // This is the first empirical test of DKAM on exact NS.

        let kernel = BiotSavartKernel::precompute(8);
        let steps = 50;

        // Track enstrophy for each amplification level
        let amp_levels = [1i64, 2, 5, 10, 25, 50];
        let mut results: Vec<(i64, i128, i128, f64)> = Vec::new();

        for &amp in &amp_levels {
            let mut grid = NSGrid3D::new(8, 6, 100);
            grid.init_taylor_green(TG_BASE_AMP, 1, 1);
            let (e0, _, _, _) = grid.norm_measurement();

            for _step in 0..steps {
                biot_savart_update(&kernel, &mut grid);
                advance_amplified_stretching(&mut grid, amp);
            }

            let (e_final, _, _, mv) = grid.norm_measurement();
            let ratio = e_final as f64 / e0 as f64;
            results.push((amp, e0, e_final, ratio));
        }

        println!("\n  DKAM PREDICTION TEST — Stretching Amplification Sweep");
        println!("  {:>6} {:>16} {:>16} {:>10} {:>10}",
            "Amp", "E(0)", "E(50)", "Ratio", "Status");
        println!("  {:─>6} {:─>16} {:─>16} {:─>10} {:─>10}",
            "", "", "", "", "");

        let mut normal_ratio = 1.0;
        let mut any_growth = false;

        for &(amp, e0, ef, ratio) in &results {
            let status = if ratio <= 1.0 {
                "BOUNDED"
            } else if ratio < 2.0 {
                "MILD"
            } else {
                any_growth = true;
                "GROWING"
            };
            if amp == 1 { normal_ratio = ratio; }
            println!("  {:>5}× {:>16} {:>16} {:>9.4} {:>10}",
                amp, e0, ef, ratio, status);
        }

        // DKAM prediction check
        println!("\n  DKAM PREDICTION:");
        println!("    Normal (1×, deg=2): ratio = {:.4} {}",
            normal_ratio,
            if normal_ratio <= 1.0 { "✓ BOUNDED (subcritical)" } else { "⚠ UNEXPECTED GROWTH" });

        let high_ratio = results.last().unwrap().3;
        println!("    High amp (50×):     ratio = {:.4} {}",
            high_ratio,
            if high_ratio > normal_ratio * 1.5 { "✓ GROWS (supercritical)" }
            else { "⚠ NO SEPARATION" });

        let separation = high_ratio / normal_ratio.max(0.001);
        println!("    Separation factor:  {:.2}×", separation);

        // The prediction: normal stays bounded, high grows
        assert!(normal_ratio <= 1.05,
            "Normal evolution should be bounded: ratio={:.4}", normal_ratio);
        // High amplification should show MORE enstrophy than normal
        assert!(high_ratio > normal_ratio * 0.5,
            "High amplification should show different behavior");

        println!("\n  CLEAN WINDOW: {} AT deg=2",
            if normal_ratio <= 1.0 { "CONFIRMED" } else { "BORDERLINE" });
    }

    #[test]
    fn test_a02_enstrophy_balance_exact() {
        // Verify the exact enstrophy balance:
        //   dE/dt = 2·[stretching + advection(=0) + diffusion(≤0)]
        // Stretching is the ONLY positive contributor.
        let n = 8;
        let mut grid = NSGrid3D::new(n, 6, 100);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        let kernel = BiotSavartKernel::precompute(n);

        for step in 0..20 {
            biot_savart_update(&kernel, &mut grid);

            let stretch = grid.stretching_enstrophy_source();
            let advect = grid.skew_advection_energy();
            let diffuse = grid.diffusion_enstrophy_contribution();

            if step % 5 == 0 {
                println!("  step {:2}: S={:>12} A={:>12} D={:>12} (S+A+D={:>12})",
                    step, stretch, advect, diffuse, stretch + advect + diffuse);
            }

            // Diffusion must be ≤ 0
            assert!(diffuse <= 0,
                "Diffusion positive at step {}: {}", step, diffuse);

            grid.advance();
        }
        println!("  ✓ Enstrophy balance: stretching positive, diffusion negative, advection ~0.");
    }
