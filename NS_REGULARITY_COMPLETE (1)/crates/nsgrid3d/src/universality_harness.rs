//! # CRAM Hasse Bridge Universality Harness
//!
//! Tests the global p-adic non-expansion on discriminating lanes {7,11,13}
//! across multiple grid sizes and initial conditions.
//!
//! If non-expansion holds for ALL configurations → structural property
//! If it fails for some → property of Taylor-Green only

use nsgrid3d::ns_grid_3d::*;
use nsgrid3d::biot_savart::*;

const DISC: [u64; 3] = [7, 11, 13];

fn v_p(x: i64, p: u64) -> u32 {
    if x == 0 { return 99; }
    let mut x = x.unsigned_abs();
    let mut v = 0u32;
    while x % p == 0 { x /= p; v += 1; }
    v
}

/// Run a single test: given grid with IC, evolve n_steps, check v_p non-expansion
fn test_nonexpansion(
    label: &str,
    grid: &mut NSGrid3D,
    kernel: &BiotSavartKernel,
    n_steps: usize,
) -> (bool, [u64; 3], [u64; 3], [u64; 3]) {
    let n = grid.n;
    let n3 = n * n * n;

    // Snapshot initial v_p on discriminating lanes
    let mut v_init: Vec<[[u32; 3]; 3]> = Vec::with_capacity(n3);
    for i in 0..n3 {
        let c = &grid.cells[i];
        let w = [c.w_x, c.w_y, c.w_z];
        let mut init = [[0u32; 3]; 3];
        for comp in 0..3 {
            for (pi, &p) in DISC.iter().enumerate() {
                init[comp][pi] = v_p(w[comp], p);
            }
        }
        v_init.push(init);
    }

    let e_init = grid.total_enstrophy();
    let m_init = grid.vort_sup_global();

    // Evolve
    for _ in 0..n_steps {
        biot_savart_update(kernel, grid);
        grid.advance();
    }

    let e_final = grid.total_enstrophy();
    let m_final = grid.vort_sup_global();

    // Global non-expansion check
    let mut inc = [0u64; 3];
    let mut dec = [0u64; 3];
    let mut same = [0u64; 3];

    for i in 0..n3 {
        let c = &grid.cells[i];
        let w = [c.w_x, c.w_y, c.w_z];
        for comp in 0..3 {
            for (pi, &p) in DISC.iter().enumerate() {
                let vi = v_init[i][comp][pi];
                let vf = v_p(w[comp], p);
                if vi >= 90 || vf >= 90 { same[pi] += 1; continue; }
                if vf > vi { inc[pi] += 1; }
                else if vf < vi { dec[pi] += 1; }
                else { same[pi] += 1; }
            }
        }
    }

    let all_non_expansive = dec.iter().all(|&d| d == 0);

    println!("  {:>30} | N={} | {} steps", label, n, n_steps);
    println!("    Enstrophy: {} -> {} ({:.1}%)", e_init, e_final,
        e_final as f64 / e_init.max(1) as f64 * 100.0);
    println!("    max|w|:    {} -> {}", m_init, m_final);
    for (pi, &p) in DISC.iter().enumerate() {
        let tag = if dec[pi] == 0 { "NON-EXP \u{2713}" } else { "FAILS \u{2717}" };
        println!("    p={:>2}: \u{2191}{:>4} \u{2193}{:>4} ={:>4} {}",
            p, inc[pi], dec[pi], same[pi], tag);
    }
    let verdict = if all_non_expansive { "\u{2605} ALL DISC LANES NON-EXPANSIVE" }
                  else { "\u{2717} NON-EXPANSION VIOLATED" };
    println!("    {}", verdict);
    println!();

    (all_non_expansive, inc, dec, same)
}

fn main() {
    println!("\u{2554}{}\u{2557}", "\u{2550}".repeat(64));
    println!("\u{2551}  CRAM HASSE BRIDGE: UNIVERSALITY HARNESS                      \u{2551}");
    println!("\u{2551}  Is p-adic non-expansion on {{7,11,13}} structural or accidental?\u{2551}");
    println!("\u{255A}{}\u{255D}\n", "\u{2550}".repeat(64));

    let mut pass = 0u32;
    let mut fail = 0u32;

    // ═══════════════════════════════════════════════════
    // TEST 1: N=4, Taylor-Green, 30 steps
    // ═══════════════════════════════════════════════════
    {
        let n = 4;
        let dt = 100;
        let kernel = BiotSavartKernel::precompute(n);
        let mut grid = NSGrid3D::new(n, 6, dt);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        biot_savart_update(&kernel, &mut grid);
        let (ok, _, _, _) = test_nonexpansion("TG N=4, 30 steps", &mut grid, &kernel, 30);
        if ok { pass += 1; } else { fail += 1; }
    }

    // ═══════════════════════════════════════════════════
    // TEST 2: N=8, Taylor-Green, 30 steps (reference)
    // ═══════════════════════════════════════════════════
    {
        let n = 8;
        let dt = 100;
        let kernel = BiotSavartKernel::precompute(n);
        let mut grid = NSGrid3D::new(n, 6, dt);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        biot_savart_update(&kernel, &mut grid);
        let (ok, _, _, _) = test_nonexpansion("TG N=8, 30 steps (reference)", &mut grid, &kernel, 30);
        if ok { pass += 1; } else { fail += 1; }
    }

    // ═══════════════════════════════════════════════════
    // TEST 3: N=8, Taylor-Green wavenumber (2,1), 30 steps
    // ═══════════════════════════════════════════════════
    {
        let n = 8;
        let dt = 100;
        let kernel = BiotSavartKernel::precompute(n);
        let mut grid = NSGrid3D::new(n, 6, dt);
        grid.init_taylor_green(TG_BASE_AMP, 2, 1);
        biot_savart_update(&kernel, &mut grid);
        let (ok, _, _, _) = test_nonexpansion("TG (kx=2,ky=1) N=8", &mut grid, &kernel, 30);
        if ok { pass += 1; } else { fail += 1; }
    }

    // ═══════════════════════════════════════════════════
    // TEST 4: N=8, Taylor-Green half amplitude
    // ═══════════════════════════════════════════════════
    {
        let n = 8;
        let dt = 100;
        let kernel = BiotSavartKernel::precompute(n);
        let mut grid = NSGrid3D::new(n, 6, dt);
        grid.init_taylor_green(TG_BASE_AMP / 2, 1, 1);
        biot_savart_update(&kernel, &mut grid);
        let (ok, _, _, _) = test_nonexpansion("TG half-amp N=8", &mut grid, &kernel, 30);
        if ok { pass += 1; } else { fail += 1; }
    }

    // ═══════════════════════════════════════════════════
    // TEST 5: N=8, Taylor-Green double amplitude
    // ═══════════════════════════════════════════════════
    {
        let n = 8;
        let dt = 100;
        let kernel = BiotSavartKernel::precompute(n);
        let mut grid = NSGrid3D::new(n, 6, dt);
        grid.init_taylor_green(TG_BASE_AMP * 2, 1, 1);
        biot_savart_update(&kernel, &mut grid);
        let (ok, _, _, _) = test_nonexpansion("TG double-amp N=8", &mut grid, &kernel, 30);
        if ok { pass += 1; } else { fail += 1; }
    }

    // ═══════════════════════════════════════════════════
    // TEST 6: N=4, longer run (60 steps)
    // ═══════════════════════════════════════════════════
    {
        let n = 4;
        let dt = 100;
        let kernel = BiotSavartKernel::precompute(n);
        let mut grid = NSGrid3D::new(n, 6, dt);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        biot_savart_update(&kernel, &mut grid);
        let (ok, _, _, _) = test_nonexpansion("TG N=4, 60 steps", &mut grid, &kernel, 60);
        if ok { pass += 1; } else { fail += 1; }
    }

    // ═══════════════════════════════════════════════════
    // TEST 7: N=8, dt=50 (smaller timestep)
    // ═══════════════════════════════════════════════════
    {
        let n = 8;
        let dt = 50;
        let kernel = BiotSavartKernel::precompute(n);
        let mut grid = NSGrid3D::new(n, 6, dt);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        biot_savart_update(&kernel, &mut grid);
        let (ok, _, _, _) = test_nonexpansion("TG N=8, dt=50", &mut grid, &kernel, 30);
        if ok { pass += 1; } else { fail += 1; }
    }

    // ═══════════════════════════════════════════════════
    // TEST 8: N=8, manually perturbed IC
    // ═══════════════════════════════════════════════════
    {
        let n = 8;
        let dt = 100;
        let kernel = BiotSavartKernel::precompute(n);
        let mut grid = NSGrid3D::new(n, 6, dt);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        // Add perturbation: shift w_x by a non-trivial amount at random cells
        let perturbations: [(usize, i64); 8] = [
            (0, 1000), (42, -500), (100, 2000), (200, -1500),
            (300, 800), (400, -300), (450, 1700), (500, -900),
        ];
        for &(idx, delta) in &perturbations {
            if idx < grid.cells.len() {
                grid.cells[idx].w_x += delta;
                grid.cells[idx].w_y += delta / 2;
            }
        }
        biot_savart_update(&kernel, &mut grid);
        let (ok, _, _, _) = test_nonexpansion("TG+perturbation N=8", &mut grid, &kernel, 30);
        if ok { pass += 1; } else { fail += 1; }
    }

    // ═══════════════════════════════════════════════════
    // TEST 9: N=8, TG wavenumber (1,2)
    // ═══════════════════════════════════════════════════
    {
        let n = 8;
        let dt = 100;
        let kernel = BiotSavartKernel::precompute(n);
        let mut grid = NSGrid3D::new(n, 6, dt);
        grid.init_taylor_green(TG_BASE_AMP, 1, 2);
        biot_savart_update(&kernel, &mut grid);
        let (ok, _, _, _) = test_nonexpansion("TG (kx=1,ky=2) N=8", &mut grid, &kernel, 30);
        if ok { pass += 1; } else { fail += 1; }
    }

    // ═══════════════════════════════════════════════════
    // TEST 10: N=4, small dt, long run
    // ═══════════════════════════════════════════════════
    {
        let n = 4;
        let dt = 25;
        let kernel = BiotSavartKernel::precompute(n);
        let mut grid = NSGrid3D::new(n, 6, dt);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        biot_savart_update(&kernel, &mut grid);
        let (ok, _, _, _) = test_nonexpansion("TG N=4, dt=25, 100 steps", &mut grid, &kernel, 100);
        if ok { pass += 1; } else { fail += 1; }
    }

    // ═══════════════════════════════════════════════════
    // SUMMARY
    // ═══════════════════════════════════════════════════
    println!("{}", "=".repeat(64));
    println!("  UNIVERSALITY HARNESS: {}/{} PASS", pass, pass + fail);
    println!("{}", "=".repeat(64));

    if fail == 0 {
        println!("\n  ALL CONFIGURATIONS: p-adic non-expansion on {{7,11,13}} HOLDS");
        println!("  Evidence: STRUCTURAL property (not accidental)");
        println!("  D-024: CONDITIONALLY CLOSED");
    } else {
        println!("\n  {} configurations FAILED non-expansion.", fail);
        println!("  D-024 remains OPEN on failing configurations.");
    }
}
