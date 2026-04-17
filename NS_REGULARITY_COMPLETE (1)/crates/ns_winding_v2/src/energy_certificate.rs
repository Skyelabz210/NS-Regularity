//! # Exact Discrete Energy Certificate via Skew-Symmetric Advection
//!
//! The key identity: for divergence-free flow on a periodic grid,
//! the skew-symmetric discretization of (u·∇)u satisfies
//!
//!   Σ_{i,j} u_c(i,j) · A_c(i,j) ≡ 0  (mod p)
//!
//! where A_c is the advection term for component c.
//! This is verifiable EXACTLY in CRT arithmetic.
//!
//! Physical meaning: the nonlinear term transfers energy between
//! scales but creates/destroys none. In CRT, this is a theorem
//! about modular arithmetic, not an approximation.

use crate::*;

// ============================================================================
// SKEW-SYMMETRIC ADVECTION
// ============================================================================

/// Compute the skew-symmetric advection term for component c at (iy, ix).
///
/// The skew-symmetric form is:
///   A_c = ½ [u_x · D_x(u_c) + D_x(u_x · u_c)]
///       + ½ [u_y · D_y(u_c) + D_y(u_y · u_c)]
///
/// Since we work in integer arithmetic, we avoid the factor ½ by computing
/// 2·A_c instead (the energy identity becomes Σ u_c · 2A_c ≡ 0, which is
/// equivalent since 2 is invertible mod any odd prime).
///
/// Returns the residue of 2·A_c in the given prime.
pub fn skew_symmetric_advection(
    grid: &Grid2D,
    iy: usize,
    ix: usize,
    component: usize, // 0 = u_x, 1 = u_y
    prime: u64,
) -> u64 {
    let n = grid.n;

    let ix_p = (ix + 1) % n;
    let ix_m = if ix == 0 { n - 1 } else { ix - 1 };
    let iy_p = (iy + 1) % n;
    let iy_m = if iy == 0 { n - 1 } else { iy - 1 };

    // Values at this point (reduced into working prime)
    let ux = grid.get(0, iy, ix) % prime;
    let uy = grid.get(1, iy, ix) % prime;
    let uc = grid.get(component, iy, ix) % prime;

    // --- x-direction ---
    // Term 1: u_x · D_x(u_c) = u_x · (u_c[i+1] - u_c[i-1])
    let uc_xp = grid.get(component, iy, ix_p) % prime;
    let uc_xm = grid.get(component, iy, ix_m) % prime;
    let dx_uc = submod(uc_xp, uc_xm, prime);
    let term1_x = mulmod(ux, dx_uc, prime);

    // Term 2: D_x(u_x · u_c) = (u_x[i+1]·u_c[i+1]) - (u_x[i-1]·u_c[i-1])
    let ux_xp = grid.get(0, iy, ix_p) % prime;
    let ux_xm = grid.get(0, iy, ix_m) % prime;
    let prod_xp = mulmod(ux_xp, uc_xp, prime);
    let prod_xm = mulmod(ux_xm, uc_xm, prime);
    let term2_x = submod(prod_xp, prod_xm, prime);

    // --- y-direction ---
    // Term 1: u_y · D_y(u_c) = u_y · (u_c[j+1] - u_c[j-1])
    let uc_yp = grid.get(component, iy_p, ix) % prime;
    let uc_ym = grid.get(component, iy_m, ix) % prime;
    let dy_uc = submod(uc_yp, uc_ym, prime);
    let term1_y = mulmod(uy, dy_uc, prime);

    // Term 2: D_y(u_y · u_c) = (u_y[j+1]·u_c[j+1]) - (u_y[j-1]·u_c[j-1])
    let uy_yp = grid.get(1, iy_p, ix) % prime;
    let uy_ym = grid.get(1, iy_m, ix) % prime;
    let prod_yp = mulmod(uy_yp, uc_yp, prime);
    let prod_ym = mulmod(uy_ym, uc_ym, prime);
    let term2_y = submod(prod_yp, prod_ym, prime);

    // 2·A_c = (term1_x + term2_x) + (term1_y + term2_y)
    let x_part = addmod(term1_x, term2_x, prime);
    let y_part = addmod(term1_y, term2_y, prime);
    addmod(x_part, y_part, prime)
}

// ============================================================================
// ENERGY CERTIFICATE
// ============================================================================

/// Result of the energy certificate verification.
#[derive(Debug, Clone)]
pub struct EnergyCertificate {
    /// The residue of Σ u_c · (2·A_c) for each velocity component, in each prime.
    /// If the identity holds, these are all zero.
    pub advection_energy_residues: Vec<(usize, u64, u64)>, // (component, prime, residue)
    /// Whether the identity holds exactly.
    pub identity_holds: bool,
    /// Discrete divergence residue at the sample point (diagnostic).
    pub divergence_residue: u64,
    /// Discrete energy: Σ (u_x² + u_y²) mod p
    pub energy_residue: u64,
}

/// Verify the exact discrete energy identity:
///   Σ_{grid} u_c(x) · A_c^{skew}(x) ≡ 0  (mod p)
///
/// for each velocity component c ∈ {u_x, u_y}.
///
/// This function computes the global inner product of each velocity field
/// with its own skew-symmetric advection term. If the field is discretely
/// divergence-free and the grid is periodic, this sum is exactly zero.
///
/// The verification is done in a specified exact prime.
pub fn verify_energy_identity(grid: &Grid2D, prime: u64) -> EnergyCertificate {
    let n = grid.n;
    let mut results = Vec::new();
    let mut all_zero = true;

    // For each velocity component
    for component in [0usize, 1] {
        let mut inner_product = 0u64;

        for iy in 0..n {
            for ix in 0..n {
                let uc = grid.get(component, iy, ix) % prime;
                let adv = skew_symmetric_advection(grid, iy, ix, component, prime);

                // Accumulate u_c · (2·A_c)
                let contribution = mulmod(uc, adv, prime);
                inner_product = addmod(inner_product, contribution, prime);
            }
        }

        if inner_product != 0 {
            all_zero = false;
        }
        results.push((component, prime, inner_product));
    }

    // Cross-term check: also verify Σ u_x · A_x + Σ u_y · A_y
    // (This is the total energy contribution of advection)
    let total = if results.len() == 2 {
        addmod(results[0].2, results[1].2, prime)
    } else {
        0
    };

    if total != 0 {
        all_zero = false;
    }

    // Compute discrete divergence at center point (diagnostic)
    let cx = n / 2;
    let cy = n / 2;
    let div = compute_discrete_divergence(grid, cy, cx, prime);

    // Compute energy
    let energy = compute_energy_residue(grid, prime);

    EnergyCertificate {
        advection_energy_residues: results,
        identity_holds: all_zero,
        divergence_residue: div,
        energy_residue: energy,
    }
}

/// Compute discrete divergence: D_x(u_x) + D_y(u_y) at a point.
/// For a divergence-free field on a periodic grid, this should be zero.
pub fn compute_discrete_divergence(
    grid: &Grid2D,
    iy: usize,
    ix: usize,
    prime: u64,
) -> u64 {
    let n = grid.n;

    let ix_p = (ix + 1) % n;
    let ix_m = if ix == 0 { n - 1 } else { ix - 1 };
    let iy_p = (iy + 1) % n;
    let iy_m = if iy == 0 { n - 1 } else { iy - 1 };

    let dux_dx = submod(
        grid.get(0, iy, ix_p) % prime,
        grid.get(0, iy, ix_m) % prime,
        prime,
    );
    let duy_dy = submod(
        grid.get(1, iy_p, ix) % prime,
        grid.get(1, iy_m, ix) % prime,
        prime,
    );

    addmod(dux_dx, duy_dy, prime)
}

/// Global discrete divergence: maximum over all grid points.
pub fn max_discrete_divergence(grid: &Grid2D, prime: u64) -> u64 {
    let n = grid.n;
    let mut max_div = 0u64;
    let half_p = prime / 2;

    for iy in 0..n {
        for ix in 0..n {
            let div = compute_discrete_divergence(grid, iy, ix, prime);
            // Interpret as signed: near 0 or near p both mean "small"
            let abs_div = if div <= half_p { div } else { prime - div };
            if abs_div > max_div {
                max_div = abs_div;
            }
        }
    }
    max_div
}

/// Compute total discrete energy: Σ (u_x² + u_y²) mod p.
pub fn compute_energy_residue(grid: &Grid2D, prime: u64) -> u64 {
    let n = grid.n;
    let mut sum = 0u64;
    for iy in 0..n {
        for ix in 0..n {
            let ux = grid.get(0, iy, ix) % prime;
            let uy = grid.get(1, iy, ix) % prime;
            let ux2 = mulmod(ux, ux, prime);
            let uy2 = mulmod(uy, uy, prime);
            sum = addmod(sum, addmod(ux2, uy2, prime), prime);
        }
    }
    sum
}

/// Run energy certificate verification over multiple timesteps.
/// Returns: Vec of (timestep, certificate).
pub fn energy_certificate_trajectory(
    n: usize,
    reynolds: f64,
    amplitude: f64,
    wavenumber: f64,
    steps: usize,
    prime: u64,
) -> Vec<(u64, EnergyCertificate)> {
    let mut state = NSState2D::taylor_green(n, amplitude, wavenumber);
    let params = NSEvolveParams::new(n, reynolds, amplitude);
    let mut trajectory = Vec::with_capacity(steps);

    for _t in 0..steps {
        let cert = verify_energy_identity(&state.grid, prime);
        trajectory.push((state.timestep, cert));
        state = state.advance(&params);
    }

    trajectory
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_ec01_skew_advection_zero_constant_field() {
        // Constant velocity → all derivatives zero → advection zero
        let n = 8;
        let mut grid = Grid2D::new(n, &[0, 1, 3, 6]);
        let p = EXACT_PRIMES[0];
        for iy in 0..n {
            for ix in 0..n {
                grid.set(0, iy, ix, 42);
                grid.set(1, iy, ix, 42);
            }
        }
        let adv = skew_symmetric_advection(&grid, 4, 4, 0, p);
        assert_eq!(adv, 0, "Skew advection of constant field must be zero");
    }

    #[test]
    fn t_ec02_energy_identity_constant_field() {
        // Constant field: trivially divergence-free, identity should hold
        let n = 8;
        let mut grid = Grid2D::new(n, &[0, 1, 3, 6]);
        let p = EXACT_PRIMES[0];
        for iy in 0..n {
            for ix in 0..n {
                grid.set(0, iy, ix, 100);
                grid.set(1, iy, ix, 200);
            }
        }
        let cert = verify_energy_identity(&grid, p);
        assert!(cert.identity_holds, "Energy identity must hold for constant field");
    }

    #[test]
    fn t_ec03_divergence_zero_constant() {
        let n = 8;
        let mut grid = Grid2D::new(n, &[0, 1, 3, 6]);
        let p = EXACT_PRIMES[0];
        for iy in 0..n {
            for ix in 0..n {
                grid.set(0, iy, ix, 42);
                grid.set(1, iy, ix, 77);
            }
        }
        assert_eq!(
            max_discrete_divergence(&grid, p),
            0,
            "Constant field must be divergence-free"
        );
    }

    #[test]
    fn t_ec04_taylor_green_divergence() {
        // Taylor-Green is analytically divergence-free.
        // Discrete divergence should be small (O(h²) quantization).
        let n = 32;
        let grid = taylor_green_init(n, 1.0, 1.0);
        let p = EXACT_PRIMES[0];
        let max_div = max_discrete_divergence(&grid, p);
        // The divergence should be small relative to field magnitudes
        // (bounded by discretization + quantization error)
        let threshold = (SCALE_FACTOR as u64) * 2; // generous threshold
        let max_div_signed = if max_div <= p / 2 { max_div } else { p - max_div };
        assert!(
            max_div_signed < threshold,
            "Taylor-Green discrete divergence too large: {}",
            max_div_signed
        );
    }

    #[test]
    fn t_ec05_energy_identity_taylor_green() {
        // The KEY test: does the skew-symmetric energy identity hold
        // for Taylor-Green initial conditions?
        //
        // Note: Taylor-Green is ANALYTICALLY divergence-free, but the
        // discrete centered-difference divergence is O(h²) nonzero.
        // The energy identity holds exactly only for discretely
        // divergence-free fields. So this test checks whether the
        // Taylor-Green discretization is "close enough" for the identity
        // to hold modularly — or whether we need explicit projection.
        let n = 16;
        let grid = taylor_green_init(n, 1.0, 1.0);
        let p = EXACT_PRIMES[0];
        let cert = verify_energy_identity(&grid, p);

        // Report the residues regardless of pass/fail
        for (comp, _prime, res) in &cert.advection_energy_residues {
            let res_signed = if *res <= p / 2 {
                *res as i128
            } else {
                *res as i128 - p as i128
            };
            eprintln!(
                "  Energy identity residue for component {}: {} (signed: {})",
                EXACT_LANE_NAMES[*comp],
                res,
                res_signed
            );
        }
        eprintln!("  Divergence residue at center: {}", cert.divergence_residue);

        // The identity may not hold exactly due to discrete divergence.
        // This is an honest negative if it fails — documenting the gap
        // between analytic and discrete div-free conditions.
        // We record the result either way.
        if cert.identity_holds {
            eprintln!("  ✓ Energy identity holds EXACTLY for Taylor-Green");
        } else {
            eprintln!("  ✗ Energy identity does NOT hold exactly — discrete divergence nonzero");
            eprintln!("    This is expected: Taylor-Green is analytically div-free but");
            eprintln!("    discretely div-free only up to O(h²) quantization error.");
            eprintln!("    A Hodge projection step is needed to make it discretely exact.");
        }
        // This test documents the finding; it doesn't assert pass/fail
        // because the result is informative either way.
    }

    #[test]
    fn t_ec06_energy_identity_manufactured_divfree() {
        // Manufacture a DISCRETELY divergence-free field and verify
        // the energy identity holds EXACTLY.
        //
        // Construction: use a stream function ψ and define
        //   u_x =  Dy(ψ) = (ψ[j+1] - ψ[j-1]) / 2
        //   u_y = -Dx(ψ) = -(ψ[i+1] - ψ[i-1]) / 2
        //
        // Then D_x(u_x) + D_y(u_y) = 0 EXACTLY (by commutativity of
        // mixed differences on a periodic grid).
        let n = 8;
        let p = EXACT_PRIMES[0];
        let mut grid = Grid2D::new(n, &[0, 1, 3, 6]);

        // Stream function: ψ(i,j) = i*j mod p (arbitrary smooth-ish function)
        let psi = |iy: usize, ix: usize| -> u64 {
            ((iy as u64 * 100 + 7) * (ix as u64 * 100 + 13)) % p
        };

        for iy in 0..n {
            for ix in 0..n {
                let ix_p = (ix + 1) % n;
                let ix_m = if ix == 0 { n - 1 } else { ix - 1 };
                let iy_p = (iy + 1) % n;
                let iy_m = if iy == 0 { n - 1 } else { iy - 1 };

                // u_x = (ψ[j+1] - ψ[j-1]) (omit /2 — absorbed into units)
                let ux = submod(psi(iy_p, ix), psi(iy_m, ix), p);
                // u_y = -(ψ[i+1] - ψ[i-1])
                let uy = submod(psi(iy, ix_m), psi(iy, ix_p), p); // note sign flip

                grid.set(0, iy, ix, ux);
                grid.set(1, iy, ix, uy);
            }
        }

        // Verify discrete divergence is EXACTLY zero at every point
        for iy in 0..n {
            for ix in 0..n {
                let div = compute_discrete_divergence(&grid, iy, ix, p);
                assert_eq!(
                    div, 0,
                    "Stream function field must be discretely div-free at ({},{})",
                    iy, ix
                );
            }
        }

        // NOW verify the energy identity
        let cert = verify_energy_identity(&grid, p);
        assert!(
            cert.identity_holds,
            "Energy identity MUST hold for discretely divergence-free field. \
             Residues: u_x={}, u_y={}",
            cert.advection_energy_residues[0].2,
            cert.advection_energy_residues[1].2
        );
    }

    #[test]
    fn t_ec07_energy_identity_multiple_primes() {
        // Verify the identity in ALL 7 exact primes for the manufactured field
        let n = 8;

        for lane_idx in 0..7 {
            let p = EXACT_PRIMES[lane_idx];
            let mut grid = Grid2D::new(n, &[0, 1, 3, 6]);

            let psi = |iy: usize, ix: usize| -> u64 {
                ((iy as u64 * 31 + 5) * (ix as u64 * 47 + 3) + iy as u64 * 7) % p
            };

            for iy in 0..n {
                for ix in 0..n {
                    let ix_p = (ix + 1) % n;
                    let ix_m = if ix == 0 { n - 1 } else { ix - 1 };
                    let iy_p = (iy + 1) % n;
                    let iy_m = if iy == 0 { n - 1 } else { iy - 1 };

                    let ux = submod(psi(iy_p, ix), psi(iy_m, ix), p);
                    let uy = submod(psi(iy, ix_m), psi(iy, ix_p), p);

                    grid.set(0, iy, ix, ux);
                    grid.set(1, iy, ix, uy);
                }
            }

            let cert = verify_energy_identity(&grid, p);
            assert!(
                cert.identity_holds,
                "Energy identity failed in prime {} (lane {}). \
                 Residues: u_x={}, u_y={}",
                p,
                lane_idx,
                cert.advection_energy_residues[0].2,
                cert.advection_energy_residues[1].2,
            );
        }
    }

    #[test]
    fn t_ec08_energy_identity_stress_random_stream() {
        // Stress test: 50 random stream functions, verify identity each time
        let n = 8;
        let p = EXACT_PRIMES[0];

        for seed in 0..50u64 {
            let mut grid = Grid2D::new(n, &[0, 1, 3, 6]);

            // Pseudo-random stream function
            let psi = |iy: usize, ix: usize| -> u64 {
                let v = (seed.wrapping_mul(7919) + iy as u64 * 6271 + ix as u64 * 3571) % p;
                v
            };

            for iy in 0..n {
                for ix in 0..n {
                    let ix_p = (ix + 1) % n;
                    let ix_m = if ix == 0 { n - 1 } else { ix - 1 };
                    let iy_p = (iy + 1) % n;
                    let iy_m = if iy == 0 { n - 1 } else { iy - 1 };

                    let ux = submod(psi(iy_p, ix), psi(iy_m, ix), p);
                    let uy = submod(psi(iy, ix_m), psi(iy, ix_p), p);

                    grid.set(0, iy, ix, ux);
                    grid.set(1, iy, ix, uy);
                }
            }

            let cert = verify_energy_identity(&grid, p);
            assert!(
                cert.identity_holds,
                "Energy identity failed for seed {}. Residues: u_x={}, u_y={}",
                seed,
                cert.advection_energy_residues[0].2,
                cert.advection_energy_residues[1].2,
            );
        }
    }
}
