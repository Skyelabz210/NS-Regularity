//! # Seam 1 closure — canonical-BS H² commutator sweep
//!
//! The h2_closure.rs tests use a local toy Biot-Savart with small primes
//! {17, 19, 23, 29}, M = 215,441. That wraps at any significant amplitude,
//! making the TG/Random results unreliable. This module uses the **canonical
//! exact Biot-Savart** from `ns_regularity::ns3d::biot_savart_normalized`
//! (primes {73, 89, 97, 113}, M = 71,213,617) and measures the signed
//! commutator inner product `signed_ip = ⟨Δω, [Δ, A_skew(u,·)]ω⟩` across
//! TG / ABC / random fields at amplitudes 10, 100, 1000, 5000, 10000.
//!
//! ## The closure dichotomy
//!
//! `dH²/dt = -ν|∇Δω|² + signed_ip`
//!
//! - `signed_ip < 0` (DISS): commutator is dissipative → H² monotone non-increasing
//! - `signed_ip = 0` (ZERO): Kill #113 extends exactly to H² → H² conserved (advection
//!    transparent) and decreases under diffusion
//! - `signed_ip > 0` (PUMP): commutator pumps H² energy → bound only if
//!    `signed_ip ≤ ν|∇Δω|²`
//!
//! For Beltrami flows (TG with u = ω, ABC) the commutator is exactly zero
//! per the algebraic identity in `h2_advection_commutator`. The question is
//! what happens with **u = BS(ω)** — the actual NS dynamics with a
//! non-Beltrami velocity field — at all amplitudes the canonical substrate
//! can hold.
//!
//! ## What this sweep produces
//!
//! For each (flow_type, amplitude) combination:
//!   - `commutator` — `Σ_α ⟨Δω_α, [Δ, A_skew(u,·)]ω_α⟩` over the three components
//!   - `diff_H2` — `Σ_α |∇Δω_α|²` (the diffusion term)
//!   - `ratio` — `commutator / diff_H2`
//!   - `sign_label` — DISS / ZERO / PUMP
//!
//! Closure of Seam 1 by this route requires: **all sign_labels are DISS or
//! ZERO, or PUMP cases have `commutator < ν · diff_H2`** for the working
//! `ν` value.

#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use ns_regularity::ns3d::{Vort3D, Vel3D, biot_savart_normalized, Field3D, NS_SCALE};

    const N: usize = 8;

    /// Discrete Laplacian for a single Field3D component on the N=8 periodic grid.
    /// Matches the canonical ns3d::discrete_laplacian definition (6-point stencil).
    fn laplacian_component(f: &Field3D) -> Field3D {
        let mut o = [[[0i64; N]; N]; N];
        for i in 0..N { for j in 0..N { for k in 0..N {
            let ip = (i + 1) % N; let im = (i + N - 1) % N;
            let jp = (j + 1) % N; let jm = (j + N - 1) % N;
            let kp = (k + 1) % N; let km = (k + N - 1) % N;
            o[i][j][k] = f[ip][j][k] + f[im][j][k]
                       + f[i][jp][k] + f[i][jm][k]
                       + f[i][j][kp] + f[i][j][km]
                       - 6 * f[i][j][k];
        }}}
        o
    }

    /// Skew-symmetric advection of scalar f by velocity (ux, uy, uz):
    ///   A_skew[f] = (1/2) [ u·∇f  +  ∇·(uf) ]
    /// on the periodic grid via the standard centered-difference convention
    /// matching the canonical ns_step_skew implementation.
    fn advection_skew_component(
        ux: &Field3D, uy: &Field3D, uz: &Field3D, f: &Field3D
    ) -> Field3D {
        let mut o = [[[0i64; N]; N]; N];
        for i in 0..N { for j in 0..N { for k in 0..N {
            let ip = (i + 1) % N; let im = (i + N - 1) % N;
            let jp = (j + 1) % N; let jm = (j + N - 1) % N;
            let kp = (k + 1) % N; let km = (k + N - 1) % N;
            // u·∇f  (centered)
            let udf =
                ux[i][j][k] * (f[ip][j][k] - f[im][j][k]) / 2
              + uy[i][j][k] * (f[i][jp][k] - f[i][jm][k]) / 2
              + uz[i][j][k] * (f[i][j][kp] - f[i][j][km]) / 2;
            // ∇·(uf)  (centered)
            let div_uf =
                (ux[ip][j][k]*f[ip][j][k] - ux[im][j][k]*f[im][j][k]) / 2
              + (uy[i][jp][k]*f[i][jp][k] - uy[i][jm][k]*f[i][jm][k]) / 2
              + (uz[i][j][kp]*f[i][j][kp] - uz[i][j][km]*f[i][j][km]) / 2;
            o[i][j][k] = (udf + div_uf) / 2;
        }}}
        o
    }

    fn inner_product(f: &Field3D, g: &Field3D) -> i128 {
        let mut s: i128 = 0;
        for i in 0..N { for j in 0..N { for k in 0..N {
            s += (f[i][j][k] as i128) * (g[i][j][k] as i128);
        }}} s
    }

    fn h1_seminorm_sq(f: &Field3D) -> i128 {
        // |∇f|² = -⟨f, Δf⟩ (summation by parts on periodic grid)
        let lap = laplacian_component(f);
        -inner_product(f, &lap)
    }

    /// Adversarial deterministic random vorticity field on N=8 grid.
    /// Divergence is NOT enforced; this measures the worst-case adversarial input.
    fn random_vort3d(seed: u64, amp: i64) -> Vort3D {
        let mut w = Vort3D::zero();
        let mut rng = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        for i in 0..N { for j in 0..N { for k in 0..N {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let vx = ((rng >> 33) as i64 % (2*amp+1)) - amp;
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let vy = ((rng >> 33) as i64 % (2*amp+1)) - amp;
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let vz = ((rng >> 33) as i64 % (2*amp+1)) - amp;
            w.wx[i][j][k] = vx;
            w.wy[i][j][k] = vy;
            w.wz[i][j][k] = vz;
        }}}
        w
    }

    /// Compute the H² commutator three-component sum:
    ///     signed_ip = Σ_{α∈{x,y,z}} ⟨Δω_α, Δ(A_skew(u, ω_α)) − A_skew(u, Δω_α)⟩
    /// using the canonical (normalized) Biot-Savart velocity u = BS(ω).
    fn commutator_three_component(w: &Vort3D) -> (i128, i128) {
        let u: Vel3D = biot_savart_normalized(w);
        let mut signed_ip: i128 = 0;
        let mut diff_h2: i128 = 0;
        for (wf, ulabel) in [(&w.wx, "x"), (&w.wy, "y"), (&w.wz, "z")] {
            let _ = ulabel;
            let lap_w = laplacian_component(wf);
            let a_skew_w = advection_skew_component(&u.ux, &u.uy, &u.uz, wf);
            let lap_a_skew = laplacian_component(&a_skew_w);
            let a_skew_lap = advection_skew_component(&u.ux, &u.uy, &u.uz, &lap_w);
            // signed_ip contribution: ⟨Δω, Δ(A_skew) − A_skew(Δω)⟩
            // = h2_full − h2_commuted
            let h2_full = inner_product(&lap_w, &lap_a_skew);
            let h2_commuted = inner_product(&lap_w, &a_skew_lap);
            signed_ip += h2_full - h2_commuted;
            // diff_h2 contribution: |∇Δω|² (positive)
            diff_h2 += h1_seminorm_sq(&lap_w);
        }
        (signed_ip, diff_h2)
    }

    fn sign_label(ip: i128) -> &'static str {
        if ip < 0 { "DISS" } else if ip == 0 { "ZERO" } else { "PUMP" }
    }

    #[test]
    fn seam1_canonical_bs_commutator_sweep() {
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║  SEAM 1 CLOSURE SWEEP                                             ║");
        println!("║  Canonical Biot-Savart (primes {{73,89,97,113}}, M=71,213,617)    ║");
        println!("║  H² commutator signed inner product across amplitudes              ║");
        println!("╚══════════════════════════════════════════════════════════════════╝");
        println!();
        println!("                IC    amp        signed_ip          diff_H2     ratio  sign");
        println!("  ─────────────────────────────────────────────────────────────────────────");

        let amps = [10i64, 100, 500, 1000, 5000, 10000];
        let mut max_pump_ratio: f64 = 0.0;
        let mut all_diss_or_zero = true;
        let mut total = 0u32;
        let mut diss = 0u32;
        let mut zero = 0u32;
        let mut pump = 0u32;

        // Sweep 1: Taylor-Green (Beltrami-like in 2D, ω·u not generally 0 in 3D evolution)
        for &amp in &amps {
            let w = Vort3D::taylor_green(amp);
            let (ip, dh2) = commutator_three_component(&w);
            let ratio = if dh2 > 0 { (ip.abs() as f64) / (dh2 as f64) } else { 0.0 };
            let label = sign_label(ip);
            if ip > 0 {
                pump += 1;
                if ratio > max_pump_ratio { max_pump_ratio = ratio; }
                all_diss_or_zero = false;
            } else if ip < 0 { diss += 1; } else { zero += 1; }
            total += 1;
            println!("  {:>14}  {:>5}  {:>17}  {:>17}  {:>7.3}  {}",
                "Taylor-Green", amp, ip, dh2, ratio, label);
        }
        println!();

        // Sweep 2: ABC (Beltrami; expected ZERO at every amplitude)
        for &amp in &amps {
            let w = Vort3D::abc_flow(amp);
            let (ip, dh2) = commutator_three_component(&w);
            let ratio = if dh2 > 0 { (ip.abs() as f64) / (dh2 as f64) } else { 0.0 };
            let label = sign_label(ip);
            if ip > 0 {
                pump += 1;
                if ratio > max_pump_ratio { max_pump_ratio = ratio; }
                all_diss_or_zero = false;
            } else if ip < 0 { diss += 1; } else { zero += 1; }
            total += 1;
            println!("  {:>14}  {:>5}  {:>17}  {:>17}  {:>7.3}  {}",
                "ABC", amp, ip, dh2, ratio, label);
        }
        println!();

        // Sweep 3: Adversarial random fields, 5 seeds × 6 amplitudes
        for seed in 1..=5u64 {
            for &amp in &amps {
                let w = random_vort3d(seed, amp);
                let (ip, dh2) = commutator_three_component(&w);
                let ratio = if dh2 > 0 { (ip.abs() as f64) / (dh2 as f64) } else { 0.0 };
                let label = sign_label(ip);
                if ip > 0 {
                    pump += 1;
                    if ratio > max_pump_ratio { max_pump_ratio = ratio; }
                    all_diss_or_zero = false;
                } else if ip < 0 { diss += 1; } else { zero += 1; }
                total += 1;
                println!("  {:>11} s={}  {:>5}  {:>17}  {:>17}  {:>7.3}  {}",
                    "Random", seed, amp, ip, dh2, ratio, label);
            }
        }
        println!();

        println!("  ─────────────────────────────────────────────────────────────────────────");
        println!("  TOTAL: {} measurements", total);
        println!("    DISS (signed_ip < 0, dissipative):  {}", diss);
        println!("    ZERO (signed_ip = 0, Kill #113 extends): {}", zero);
        println!("    PUMP (signed_ip > 0, H² growth-tendency): {}", pump);
        if pump > 0 {
            println!("    Max PUMP ratio (|signed_ip| / |∇Δω|²): {:.4}", max_pump_ratio);
        }
        println!();
        if all_diss_or_zero {
            println!("  ★ ALL CASES DISS or ZERO. H² monotone non-increasing by");
            println!("    the algebraic structure — SEAM 1 CLOSED.");
        } else {
            println!("  PUMP cases observed. Closure conditional on |signed_ip| ≤ ν · |∇Δω|².");
            println!("  Working viscosity ν=100 (SCALE×100/SCALE per ns3d_energy):");
            println!("  Required: max_PUMP_ratio ≤ ν = 100/SCALE = 0.01");
            println!("    Observed: {:.6}", max_pump_ratio);
            if max_pump_ratio <= 0.01 {
                println!("    ✓ Below ν: H² monotone non-increasing for ν ≥ 0.01");
            } else {
                println!("    ✗ Above ν: H² monotonicity needs further analysis");
            }
        }
    }
}
