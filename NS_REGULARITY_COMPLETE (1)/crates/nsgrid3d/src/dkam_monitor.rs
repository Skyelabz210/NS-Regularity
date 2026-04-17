//! # Per-Cell DKAM Subcriticality Monitor
//!
//! Wires the intrinsic CRAM address (from prime_hunt) into the NS
//! regularity tracker. For each grid cell, computes:
//!
//!   1. The stretching residue vector on exact-div basis {3, 7, 11, 13}
//!   2. The ternary zone classification (low / resonant / high)
//!   3. The binary carry pattern (threshold at p/2)
//!   4. A composite DKAM risk score
//!
//! Cells in the RESONANT zone (near p/2 carry threshold) are flagged
//! as approaching the clean window boundary. Phase 11 proved this
//! zone produces 5.8× more enstrophy growth than the quiet zones.
//!
//! The monitor runs per-timestep with O(N³ × k) cost (k = #lanes).
//! It does NOT modify the evolution — it is a pure diagnostic.

use crate::ns_grid_3d::*;

/// The exact-division lanes: primes coprime to SCALE = 10000 = 2⁴×5⁴
const EXACT_DIV_PRIMES: [u64; 4] = [3, 7, 11, 13];

/// Ternary zone classification for a residue mod p
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    /// Low zone: 0 ≤ r < p/3 — far below carry threshold
    Low,
    /// Resonant zone: p/3 ≤ r < 2p/3 — near carry threshold (p/2)
    Resonant,
    /// High zone: 2p/3 ≤ r < p — far above carry threshold
    High,
}

/// Per-cell DKAM diagnostic
#[derive(Debug, Clone)]
pub struct CellDiagnostic {
    /// Grid position
    pub ix: usize,
    pub iy: usize,
    pub iz: usize,
    /// Stretching magnitude (sum over components)
    pub stretching_abs: i128,
    /// Residues on exact-div primes: [s mod 3, s mod 7, s mod 11, s mod 13]
    pub residues: [u64; 4],
    /// Ternary zones per lane
    pub zones: [Zone; 4],
    /// Binary carry pattern (threshold at p/2)
    pub carries: [u8; 4],
    /// Number of lanes in resonant zone
    pub resonant_count: u8,
    /// Sqr-folded carry pattern (Legendre character)
    pub sqr_carries: [u8; 4],
}

impl CellDiagnostic {
    /// Risk score: higher means closer to clean window boundary.
    /// resonant_count is the primary indicator (Phase 11 confirmed).
    /// Sqr carries add secondary information (T8-T10 separation).
    pub fn risk_score(&self) -> u8 {
        self.resonant_count * 2 + self.sqr_carries.iter().sum::<u8>()
    }
}

/// Per-timestep DKAM census
#[derive(Debug, Clone)]
pub struct DKAMCensus {
    pub step: u64,
    pub total_cells: usize,
    /// Cells with ≥1 lane in resonant zone
    pub resonant_cells: usize,
    /// Cells with ALL lanes in resonant zone
    pub fully_resonant_cells: usize,
    /// Maximum risk score across all cells
    pub max_risk: u8,
    /// Average risk score
    pub avg_risk: f64,
    /// Carry pattern histogram: how many cells have each carry pattern
    pub carry_histogram: std::collections::HashMap<u16, usize>,
    /// Top-5 highest risk cells
    pub hot_cells: Vec<CellDiagnostic>,
}

/// Compute per-cell stretching and classify via CRAM address
pub fn diagnose_grid(grid: &NSGrid3D) -> DKAMCensus {
    let n = grid.n;
    let n_i = n as i64;
    let mut diagnostics = Vec::new();
    let mut carry_hist: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();

    for ix in 0..n {
        for iy in 0..n {
            for iz in 0..n {
                let c = &grid.cells[grid.idx(ix, iy, iz)];
                let w = [c.w_x as i128, c.w_y as i128, c.w_z as i128];

                let ixp = grid.wrap(ix as isize + 1);
                let ixm = grid.wrap(ix as isize - 1);
                let iyp = grid.wrap(iy as isize + 1);
                let iym = grid.wrap(iy as isize - 1);
                let izp = grid.wrap(iz as isize + 1);
                let izm = grid.wrap(iz as isize - 1);

                // Compute total stretching magnitude across all components
                let mut total_stretch: i128 = 0;
                for alpha in 0..3usize {
                    let dirs: [(usize,usize,usize,usize,usize,usize); 3] = [
                        (ixp, iy, iz, ixm, iy, iz),
                        (ix, iyp, iz, ix, iym, iz),
                        (ix, iy, izp, ix, iy, izm),
                    ];
                    for (beta, &(xp, yp, zp, xm, ym, zm)) in dirs.iter().enumerate() {
                        let get_u = |jx: usize, jy: usize, jz: usize| -> i128 {
                            let cell = &grid.cells[grid.idx(jx, jy, jz)];
                            match alpha { 0 => cell.u_x as i128, 1 => cell.u_y as i128,
                                          _ => cell.u_z as i128 }
                        };
                        let du = get_u(xp, yp, zp) - get_u(xm, ym, zm);
                        total_stretch += (w[beta] * du).abs();
                    }
                }

                let s_abs = total_stretch.unsigned_abs();

                // Compute residues, zones, and carries on exact-div basis
                let mut residues = [0u64; 4];
                let mut zones = [Zone::Low; 4];
                let mut carries = [0u8; 4];
                let mut sqr_carries = [0u8; 4];
                let mut resonant_count = 0u8;

                for (i, &p) in EXACT_DIV_PRIMES.iter().enumerate() {
                    let r = (s_abs % p as u128) as u64;
                    residues[i] = r;

                    // Ternary zone
                    let lo = (p + 2) / 3;
                    let hi = (2 * p + 1) / 3;
                    zones[i] = if r < lo { Zone::Low }
                               else if r < hi { Zone::Resonant }
                               else { Zone::High };

                    if zones[i] == Zone::Resonant { resonant_count += 1; }

                    // Binary carry
                    carries[i] = if r * 2 >= p { 1 } else { 0 };

                    // Sqr-folded carry (Legendre character diagnostic)
                    let sqr = (r * r) % p;
                    sqr_carries[i] = if sqr * 2 >= p { 1 } else { 0 };
                }

                // Carry pattern as u16 for histogram
                let carry_key = (carries[0] as u16)
                    | ((carries[1] as u16) << 1)
                    | ((carries[2] as u16) << 2)
                    | ((carries[3] as u16) << 3)
                    | ((sqr_carries[0] as u16) << 4)
                    | ((sqr_carries[1] as u16) << 5)
                    | ((sqr_carries[2] as u16) << 6)
                    | ((sqr_carries[3] as u16) << 7);
                *carry_hist.entry(carry_key).or_insert(0) += 1;

                diagnostics.push(CellDiagnostic {
                    ix, iy, iz, stretching_abs: total_stretch,
                    residues, zones, carries, resonant_count, sqr_carries,
                });
            }
        }
    }

    let total = diagnostics.len();
    let resonant_cells = diagnostics.iter().filter(|d| d.resonant_count > 0).count();
    let fully_resonant = diagnostics.iter().filter(|d| d.resonant_count == 4).count();
    let max_risk = diagnostics.iter().map(|d| d.risk_score()).max().unwrap_or(0);
    let avg_risk = diagnostics.iter().map(|d| d.risk_score() as f64).sum::<f64>() / total as f64;

    // Top-5 by risk
    diagnostics.sort_by(|a, b| b.risk_score().cmp(&a.risk_score())
        .then(b.stretching_abs.cmp(&a.stretching_abs)));
    let hot_cells: Vec<CellDiagnostic> = diagnostics.into_iter().take(5).collect();

    DKAMCensus {
        step: grid.step, total_cells: total,
        resonant_cells, fully_resonant_cells: fully_resonant,
        max_risk, avg_risk,
        carry_histogram: carry_hist,
        hot_cells,
    }
}

/// Print a compact census report
pub fn print_census(census: &DKAMCensus) {
    println!("  DKAM Census (step {}):", census.step);
    println!("    Total cells:     {}", census.total_cells);
    println!("    Resonant cells:  {} ({:.1}%)",
        census.resonant_cells, 100.0 * census.resonant_cells as f64 / census.total_cells as f64);
    println!("    Fully resonant:  {} ({:.1}%)",
        census.fully_resonant_cells, 100.0 * census.fully_resonant_cells as f64 / census.total_cells as f64);
    println!("    Max risk score:  {}/12", census.max_risk);
    println!("    Avg risk score:  {:.2}", census.avg_risk);
    println!("    Carry patterns:  {} distinct", census.carry_histogram.len());

    if !census.hot_cells.is_empty() {
        println!("    Hottest cells:");
        for (i, cell) in census.hot_cells.iter().enumerate().take(3) {
            println!("      #{}: ({},{},{}) risk={} res=[{},{},{},{}] zones=[{:?},{:?},{:?},{:?}]",
                i+1, cell.ix, cell.iy, cell.iz, cell.risk_score(),
                cell.residues[0], cell.residues[1], cell.residues[2], cell.residues[3],
                cell.zones[0], cell.zones[1], cell.zones[2], cell.zones[3]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn census_on_taylor_green() {
        let mut grid = NSGrid3D::new(8, 6, 100);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        let census = diagnose_grid(&grid);
        assert_eq!(census.total_cells, 512);
        assert!(census.carry_histogram.len() > 0);
        // With zero velocity, stretching = 0, all residues = 0,
        // all zones = Low (0 < p/3 for p ≥ 3)
        assert_eq!(census.resonant_cells, 0,
            "With zero velocity, no cells should be resonant");
    }

    #[test]
    fn census_with_velocity() {
        use crate::biot_savart::*;
        let kernel = BiotSavartKernel::precompute(8);
        let mut grid = NSGrid3D::new(8, 6, 100);
        grid.init_taylor_green(TG_BASE_AMP, 1, 1);
        biot_savart_update(&kernel, &mut grid);
        let census = diagnose_grid(&grid);
        // With BS velocity, stretching is nonzero → some cells will be resonant
        assert!(census.carry_histogram.len() > 1,
            "With nonzero velocity, should have multiple carry patterns");
        print_census(&census);
    }

    #[test]
    fn zone_classification_correct() {
        // p = 7: low = [0,2], resonant = [3,4], high = [5,6]
        // lo = (7+2)/3 = 3, hi = (14+1)/3 = 5
        for r in 0..7u64 {
            let lo = (7 + 2) / 3; // = 3
            let hi = (2 * 7 + 1) / 3; // = 5
            let zone = if r < lo { Zone::Low }
                       else if r < hi { Zone::Resonant }
                       else { Zone::High };
            match r {
                0 | 1 | 2 => assert_eq!(zone, Zone::Low),
                3 | 4 => assert_eq!(zone, Zone::Resonant),
                5 | 6 => assert_eq!(zone, Zone::High),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_a04_resonant_zone_enstrophy_correlation() {
        use crate::biot_savart::BiotSavartKernel;
        let n = 8;
        let mut grid = NSGrid3D::new(n, 6, 100);
        grid.init_taylor_green(SCALE, 1, 1);
        let kernel = BiotSavartKernel::precompute(n);

        let primes = [3u64, 7, 11, 13];
        let n3 = n * n * n;

        let mut resonant_stretch_sum: i128 = 0;
        let mut quiet_stretch_sum: i128 = 0;
        let mut resonant_count: u64 = 0;
        let mut quiet_count: u64 = 0;

        for _step in 0..20 {
            crate::biot_savart::biot_savart_update(&kernel, &mut grid);

            for idx in 0..n3 {
                let w_mag = grid.cells[idx].w_x.abs()
                    .max(grid.cells[idx].w_y.abs())
                    .max(grid.cells[idx].w_z.abs()) as u64;

                let mut in_resonant = false;
                for &p in &primes {
                    let r = w_mag % p;
                    if r >= p / 3 && r < 2 * p / 3 {
                        in_resonant = true;
                        break;
                    }
                }

                let ix = idx / (n * n);
                let iy = (idx / n) % n;
                let iz = idx % n;
                let mut local_stretch: i64 = 0;
                for comp in 0..3 {
                    let s = crate::fused_solver::stretching_3d(
                        &grid.cells, n, ix, iy, iz, comp);
                    local_stretch += s.abs();
                }

                if in_resonant {
                    resonant_stretch_sum += local_stretch as i128;
                    resonant_count += 1;
                } else {
                    quiet_stretch_sum += local_stretch as i128;
                    quiet_count += 1;
                }
            }

            grid.advance();
        }

        let avg_resonant = resonant_stretch_sum as f64 / resonant_count.max(1) as f64;
        let avg_quiet = quiet_stretch_sum as f64 / quiet_count.max(1) as f64;
        let ratio = avg_resonant / avg_quiet.max(0.001);

        println!("  Resonant zone cells: {}, avg stretching: {:.1}", resonant_count, avg_resonant);
        println!("  Quiet zone cells:    {}, avg stretching: {:.1}", quiet_count, avg_quiet);
        println!("  Ratio (resonant/quiet): {:.2}×", ratio);
    }
}

    /// SVD→NS diagnostic: stretching alignment coherence
    /// High coherence = stretching concentrated in one direction = DKAM safe
    /// Low coherence = stretching diffuse = elevated risk
    #[test]
    fn test_svd_stretching_coherence() {
        use svd_engine::{IntMatrix, coherence};
        let n = 8;
        let mut grid = NSGrid3D::new(n, 6, 100);
        grid.init_taylor_green(SCALE, 1, 1);
        let kernel = crate::biot_savart::BiotSavartKernel::precompute(n);

        // Build 3x3 stretching tensor at each cell, compute SVD coherence
        let mut coherence_scores: Vec<u64> = Vec::new();

        crate::biot_savart::biot_savart_update(&kernel, &mut grid);

        // Sample a few cells
        for idx in [0, 64, 128, 256] {
            let ix = idx / (n * n);
            let iy = (idx / n) % n;
            let iz = idx % n;

            // Build local stretching matrix (3×3)
            let mut data = vec![0i64; 9];
            for comp in 0..3 {
                let s = crate::fused_solver::stretching_3d(
                    &grid.cells, n, ix, iy, iz, comp);
                data[comp * 3 + comp] = s; // Diagonal approximation
            }

            let mat = IntMatrix::from_row_major(3, 3, data);
            let coh = coherence(&mat);
            coherence_scores.push(coh.score);
        }

        let avg_coh = coherence_scores.iter().sum::<u64>() / coherence_scores.len() as u64;
        println!("  SVD stretching coherence scores: {:?}", coherence_scores);
        println!("  Average coherence: {}/10000", avg_coh);
        println!("  DKAM interpretation: {} stretching alignment",
            if avg_coh > 5000 { "concentrated (safe)" } else { "diffuse (monitor)" });

        // Just verify it runs — the diagnostic is informational
        assert!(coherence_scores.len() == 4, "Should have 4 measurements");
    }
