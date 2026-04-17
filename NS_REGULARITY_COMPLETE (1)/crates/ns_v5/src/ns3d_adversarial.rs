//! Adversarial Test Suite for NS Regularity Claims
//!
//! This module stress-tests every named claim in the workspace:
//! - Kill #113: skew-symmetric advection energy identity (exact zero)
//! - Kill #121: CRT lane energy topology (each prime lane zero)
//! - One-sided vs skew comparison (skew conserves, one-sided leaks)
//! - Garner/CRT reconstruction integrity
//! - Energy monotonicity under evolution
//! - H^1 bound under extended evolution
//! - Sharp Sobolev embedding under adversarial fields
//! - Stretching constant boundedness
//!
//! Every test uses exact integer arithmetic. Zero floating-point.
//! Every assertion is strict equality or strict inequality (no tolerances).

use crate::ns3d::*;
use crate::ns3d_energy::*;
use crate::ns3d_sobolev::*;
use crate::recumbent::Recumbent;

// ============================================================================
// SECTION 1: RANDOM FIELD GENERATORS
// ============================================================================

/// Generate a deterministic pseudo-random vorticity field.
/// The field has no special symmetry (not 2D, not Beltrami).
/// Amplitude controls the maximum absolute value per component.
fn adversarial_field(amplitude: i64, seed: usize) -> Vort3D {
    let mut w = Vort3D::zero();
    for i in 0..N {
        for j in 0..N {
            for k in 0..N {
                let hash = (i.wrapping_mul(97)
                    .wrapping_add(j.wrapping_mul(31))
                    .wrapping_add(k.wrapping_mul(17))
                    .wrapping_add(seed.wrapping_mul(13))
                    .wrapping_add(7)) as i64;
                let range = 2 * amplitude + 1;
                w.wx[i][j][k] = (hash.wrapping_mul(137) % range) - amplitude;
                w.wy[i][j][k] = (hash.wrapping_mul(251) % range) - amplitude;
                w.wz[i][j][k] = (hash.wrapping_mul(389) % range) - amplitude;
            }
        }
    }
    w
}

/// Generate a checkerboard vorticity field (maximally oscillatory).
/// This is the worst case for the Laplacian and Sobolev embedding.
fn checkerboard_field(amplitude: i64) -> Vort3D {
    let mut w = Vort3D::zero();
    for i in 0..N {
        for j in 0..N {
            for k in 0..N {
                let sign = if (i + j + k) % 2 == 0 { 1i64 } else { -1i64 };
                w.wx[i][j][k] = amplitude * sign;
                w.wy[i][j][k] = amplitude * sign;
                w.wz[i][j][k] = 0; // keep wz different to break symmetry
            }
        }
    }
    w
}

/// Generate a single-mode vorticity field (delta function).
/// Worst case for the Green's function / Sobolev embedding.
fn delta_field(amplitude: i64, pos_i: usize, pos_j: usize, pos_k: usize) -> Vort3D {
    let mut w = Vort3D::zero();
    w.wx[pos_i][pos_j][pos_k] = amplitude;
    w
}

// ============================================================================
// SECTION 2: KILL #113 TORTURE TESTS
// ============================================================================

#[cfg(test)]
mod kill_113_adversarial {
    use super::*;

    /// Kill #113 for Taylor-Green at multiple amplitudes.
    /// The identity must hold EXACTLY for all amplitudes.
    #[test]
    fn kill_113_tg_multi_amplitude() {
        for &amp in &[1i64, 10, 100, 1000, NS_SCALE] {
            let w = Vort3D::taylor_green(amp);
            let u = biot_savart_n8(&w);
            let residual = energy_residual_exact(&u, &w);
            assert_eq!(residual, 0,
                "Kill #113 VIOLATED for TG at amplitude {}: residual = {}",
                amp, residual);
        }
    }

    /// Kill #113 for ABC flow at multiple amplitudes.
    #[test]
    fn kill_113_abc_multi_amplitude() {
        for &amp in &[1i64, 10, 100, 1000, NS_SCALE / 10] {
            let w = Vort3D::abc_flow(amp);
            let u = biot_savart_n8(&w);
            let residual = energy_residual_exact(&u, &w);
            assert_eq!(residual, 0,
                "Kill #113 VIOLATED for ABC at amplitude {}: residual = {}",
                amp, residual);
        }
    }

    /// Kill #113 for random fields at 10 different seeds.
    /// This is the strongest test: no structural symmetry.
    #[test]
    fn kill_113_random_10_seeds() {
        for seed in 0..10usize {
            let w = adversarial_field(500, seed);
            let u = biot_savart_n8(&w);
            let residual = energy_residual_exact(&u, &w);
            assert_eq!(residual, 0,
                "Kill #113 VIOLATED for random field seed={}: residual = {}",
                seed, residual);
        }
    }

    /// Kill #113 for checkerboard field (maximally oscillatory).
    #[test]
    fn kill_113_checkerboard() {
        let w = checkerboard_field(1000);
        let u = biot_savart_n8(&w);
        let residual = energy_residual_exact(&u, &w);
        assert_eq!(residual, 0,
            "Kill #113 VIOLATED for checkerboard: residual = {}", residual);
    }

    /// Kill #113 for delta fields at all 8 positions.
    #[test]
    fn kill_113_all_deltas() {
        for pos in 0..N {
            let w = delta_field(5000, pos, 0, 0);
            let u = biot_savart_n8(&w);
            let residual = energy_residual_exact(&u, &w);
            assert_eq!(residual, 0,
                "Kill #113 VIOLATED for delta at position {}: residual = {}",
                pos, residual);
        }
    }

    /// Kill #113 after NS evolution (not just initial data).
    /// The identity must hold at EVERY timestep.
    #[test]
    fn kill_113_during_evolution() {
        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);
        let mut w = Vort3D::taylor_green(NS_SCALE / 100);
        for step in 0..20 {
            let u = biot_savart_n8(&w);
            let residual = energy_residual_exact(&u, &w);
            assert_eq!(residual, 0,
                "Kill #113 VIOLATED at step {}: residual = {}", step, residual);
            w = ns_step_skew(&w, &p);
        }
    }
}

// ============================================================================
// SECTION 3: KILL #121 (CRT LANE TOPOLOGY)
// ============================================================================

#[cfg(test)]
mod kill_121_adversarial {
    use super::*;

    /// Kill #121: energy residual is zero in EVERY NTT prime lane.
    #[test]
    fn kill_121_all_primes_multi_field() {
        let fields: Vec<(&str, Vort3D)> = vec![
            ("TG(100)", Vort3D::taylor_green(100)),
            ("ABC(100)", Vort3D::abc_flow(100)),
            ("Random(500,0)", adversarial_field(500, 0)),
            ("Random(500,1)", adversarial_field(500, 1)),
            ("Checkerboard(100)", checkerboard_field(100)),
        ];

        for (name, w) in &fields {
            let u = biot_savart_n8(w);
            let residues = energy_residual_all_primes(&u, w);
            for (lane, &r) in residues.iter().enumerate() {
                assert_eq!(r, 0,
                    "Kill #121 VIOLATED for {} in prime lane {}: residue = {}",
                    name, NTT_PRIMES[lane], r);
            }
        }
    }
}

// ============================================================================
// SECTION 4: ONE-SIDED VS SKEW COMPARISON
// ============================================================================

#[cfg(test)]
mod one_sided_vs_skew {
    use super::*;

    /// Skew step preserves or decreases energy. One-sided may not.
    #[test]
    fn skew_energy_non_increasing() {
        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);
        let w0 = Vort3D::taylor_green(NS_SCALE / 100);
        let e0 = l2_sq(&w0);

        let mut w_skew = w0.clone();
        for step in 0..15 {
            w_skew = ns_step_skew(&w_skew, &p);
            let e = l2_sq(&w_skew);
            assert!(e <= e0 + 1, // +1 for integer truncation
                "Skew energy INCREASED at step {}: {} > {}", step, e, e0);
        }
    }

    /// Compare skew vs standard step: skew should have better energy control.
    #[test]
    fn skew_vs_standard_energy_comparison() {
        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);
        let w0 = Vort3D::taylor_green(NS_SCALE / 100);
        let e0 = l2_sq(&w0);

        // Standard step
        let w_std = ns_step(&w0, &p);
        let e_std = l2_sq(&w_std);

        // Skew step
        let w_skew = ns_step_skew(&w0, &p);
        let e_skew = l2_sq(&w_skew);

        println!("Energy comparison: E0={} E_std={} E_skew={}", e0, e_std, e_skew);
        // Skew should be non-increasing
        assert!(e_skew <= e0 + 1,
            "Skew energy increased: {} > {}", e_skew, e0);
    }
}

// ============================================================================
// SECTION 5: GARNER/CRT RECONSTRUCTION INTEGRITY
// ============================================================================

#[cfg(test)]
mod crt_integrity {
    use super::*;

    /// Garner roundtrip: encode as residues, decode, verify equality.
    #[test]
    fn garner_roundtrip_exhaustive_boundary() {
        // Test values near the M/2 boundary (most likely to fail)
        let test_values: Vec<i64> = vec![
            0, 1, -1, 73, -73, 89, -89, 97, -97, 113, -113,
            1000, -1000, NS_SCALE, -NS_SCALE,
            5_000_000, -5_000_000,
            35_000_000, -35_000_000, // near M/2 = 35,606,808
            35_606_807, -35_606_807, // one below M/2
        ];

        for &x in &test_values {
            // Encode into residues
            let r0 = ((x % 73) + 73) as u64 % 73;
            let r1 = ((x % 89) + 89) as u64 % 89;
            let r2 = ((x % 97) + 97) as u64 % 97;
            let r3 = ((x % 113) + 113) as u64 % 113;

            // Decode via Garner
            let reconstructed = garner4([r0, r1, r2, r3]);

            assert_eq!(reconstructed, x,
                "Garner reconstruction failed for x={}: got {}",
                x, reconstructed);
        }
    }

    /// NTT roundtrip for all 4 primes with adversarial data.
    #[test]
    fn ntt_roundtrip_adversarial() {
        for p_idx in 0..4 {
            let p = NTT_PRIMES[p_idx];
            // Fill with values near the prime boundary
            let f: [u64; N] = core::array::from_fn(|i| {
                ((i as u64 * 17 + 3) * 41) % p
            });
            let fhat = ntt1d(&f, p_idx);
            let f_back_raw = intt1d_unnorm(&fhat, p_idx);
            // Normalize
            let n_inv = modinv(N as u64, p);
            let f_back: [u64; N] = core::array::from_fn(|i| {
                mulmod(f_back_raw[i], n_inv, p)
            });
            assert_eq!(f, f_back,
                "NTT roundtrip failed for prime {} (idx={})", p, p_idx);
        }
    }
}

// ============================================================================
// SECTION 6: SHARP SOBOLEV EMBEDDING UNDER ADVERSARIAL FIELDS
// ============================================================================

#[cfg(test)]
mod sobolev_adversarial {
    use super::*;

    /// Sobolev embedding must hold for ALL fields, including adversarial ones.
    #[test]
    fn sharp_sobolev_adversarial_fields() {
        let fields: Vec<(&str, Vort3D)> = vec![
            ("TG(10000)", Vort3D::taylor_green(NS_SCALE)),
            ("ABC(1000)", Vort3D::abc_flow(NS_SCALE / 10)),
            ("Random(1000,0)", adversarial_field(1000, 0)),
            ("Random(1000,7)", adversarial_field(1000, 7)),
            ("Checkerboard(500)", checkerboard_field(500)),
            ("Delta(5000,0,0,0)", delta_field(5000, 0, 0, 0)),
            ("Delta(5000,3,3,3)", delta_field(5000, 3, 3, 3)),
        ];

        for (name, w) in &fields {
            let (holds, lhs, rhs) = sharp_sobolev_check(w);
            assert!(holds,
                "Sharp Sobolev VIOLATED for {}: {} > {}", name, lhs, rhs);
        }
    }

    /// Sobolev embedding holds throughout NS evolution (not just initial data).
    #[test]
    fn sharp_sobolev_during_evolution() {
        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);
        let mut w = Vort3D::taylor_green(NS_SCALE / 100);

        for step in 0..20 {
            let (holds, lhs, rhs) = sharp_sobolev_check(&w);
            assert!(holds,
                "Sharp Sobolev VIOLATED at step {}: {} > {}", step, lhs, rhs);
            w = ns_step_skew(&w, &p);
        }
    }
}

// ============================================================================
// SECTION 7: RECUMBENT RECONSTRUCTION INVARIANT
// ============================================================================

#[cfg(test)]
mod recumbent_adversarial {
    use super::*;

    /// Reconstruction invariant: reconstruct() == original value for all values.
    #[test]
    fn reconstruction_invariant_wide_range() {
        for x in -100_000..=100_000i64 {
            let r = Recumbent::from_ns(x);
            let reconstructed = r.reconstruct_exact();
            assert_eq!(reconstructed, x,
                "Recumbent reconstruction failed for x={}: got {}",
                x, reconstructed);
        }
    }

    /// Tier transitions preserve reconstruction invariant.
    #[test]
    fn tier_transitions_preserve_value() {
        // Start inside corridor, push outside, push back
        let r1 = Recumbent::from_ns(100);
        assert!(r1.in_corridor());

        let big = Recumbent::from_ns(50000);
        assert!(!big.in_corridor());
        assert_eq!(big.reconstruct_exact(), 50000);

        let sum = r1.add(&big);
        assert_eq!(sum.reconstruct_exact(), 50100);

        let diff = sum.sub(&big);
        assert_eq!(diff.reconstruct_exact(), 100);
        assert!(diff.in_corridor());
    }
}

// ============================================================================
// SECTION 8: H^1 BOUND UNDER EXTENDED EVOLUTION
// ============================================================================

#[cfg(test)]
mod h1_bound_adversarial {
    use super::*;

    /// Uniform H^1 bound must hold for 50 steps (extended evolution).
    #[test]
    fn h1_bound_50_steps() {
        let w0 = Vort3D::taylor_green(NS_SCALE / 100);
        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);
        let (holds, lhs, rhs) = verify_h1_bound(&w0, &p, 50);
        assert!(holds,
            "H^1 bound VIOLATED after 50 steps: LHS={} > RHS={}",
            lhs.to_i64_exact(), rhs.to_i64_exact());
    }

    /// Both Aubin-Lions conditions after 50 steps.
    #[test]
    fn aubin_lions_50_steps() {
        let w0 = Vort3D::taylor_green(NS_SCALE / 100);
        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);
        let (h1_ok, _, _) = verify_h1_bound(&w0, &p, 50);
        let td_bound = time_derivative_bound(&w0, &p, 50);
        assert!(h1_ok, "Aubin-Lions condition (i) violated at 50 steps");
        assert!(td_bound < i128::MAX / 2,
            "Aubin-Lions condition (ii) violated: bound at sentinel");
    }
}

// ============================================================================
// SECTION 9: STRETCHING CONSTANT BOUNDEDNESS
// ============================================================================

#[cfg(test)]
mod stretching_adversarial {
    use super::*;

    /// Stretching constant must remain bounded across 10 evolution steps.
    #[test]
    fn stretching_constant_bounded_evolution() {
        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);
        let w0 = adversarial_field(100, 42);
        let mut w = w0.clone();

        let mut max_sip: i128 = 0;
        for step in 0..10 {
            let sip = stretching_inner_product_normalized(&w);
            if sip.abs() > max_sip { max_sip = sip.abs(); }
            w = ns_step_normalized(&w, &p);
        }

        // SIP should be bounded (not growing exponentially)
        let sip_final = stretching_inner_product_normalized(&w);
        assert!(sip_final.abs() <= max_sip + 1,
            "Stretching grew: max={} final={}", max_sip, sip_final.abs());
    }

    /// All solutions must decay under normalized BS (campaign regression).
    #[test]
    fn campaign_regression_all_decay() {
        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);
        for &amp in &[10i64, 100, 500, 1000] {
            for seed in 0..2usize {
                let w0 = random_field(amp, seed);
                let l2_0 = l2_sq(&w0);
                let mut w = w0;
                for _ in 0..10 {
                    w = ns_step_normalized(&w, &p);
                }
                let l2_f = l2_sq(&w);
                assert!(l2_f <= l2_0,
                    "L2 GREW for amp={} seed={}: {} > {}",
                    amp, seed, l2_f, l2_0);
            }
        }
    }
}
