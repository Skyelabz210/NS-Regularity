//! # N3-STRUCTB — Stretching-pairing identities + Frobenius pointwise certificate
//!
//! Exact-integer Rust port of `transfinite/instrument/structb.py` and
//! `transfinite/instrument/structb_rigor.py` (R2 Wave 1, ledger #123–#125).
//! All arithmetic is integer (`i64` / `i128`) — ZERO float (A1 zero-float).
//!
//! The factor `2h` from the central difference is absorbed; every identity
//! holds at the raw integer level on `(ℤ/Nℤ)³` because the `1/(2h)` factors
//! cancel identically on both sides. Identity exactness is independent of
//! grid scaling.
//!
//! ## Operators
//!
//! - `central_diff(f, d) = f[x+e_d] − f[x−e_d]`   (factor `2h` absorbed)
//! - `forward_diff(f, d) = f[x+e_d] − f[x]`        (factor `h`  absorbed)
//! - `curl(ψ)`             — divergence-free for central differences
//! - `stretching(ω, u)_c = Σ_d ω_d · d_d u_c`    (raw integer form;
//!     distinct from [`crate::ns3d::discrete_stretching`], which applies
//!     physical-units scaling `(raw · N) / (2 · NS_SCALE)`)
//!
//! ## Identity 1 — strain form
//!
//! For any fields ω, u on `(ℤ/Nℤ)³` and `G_{cd} = d_d u_c`:
//!
//! ```text
//! ⟨S(ω,u), ω⟩ = Σ_x ω^T G ω
//! sym  := Σ_x ω^T (G + G^T) ω  =  2 · ⟨S(ω,u), ω⟩
//! anti := Σ_x ω^T (G − G^T) ω  ≡  0   (integer algebra; antisymmetric form annihilates)
//! ```
//!
//! ## Identity 2 — flux / cocycle form
//!
//! For solenoidal ω (`div_h ω = 0`):
//!
//! ```text
//! ⟨S,ω⟩ + ⟨(ω·∇_h)ω, u⟩ + Σ_x u_c · L_d(ω_c, ω_d) + Σ_x (u·ω) · div_h ω  =  0
//! L_d(f, g)[x] := (Dp_d f)[x]·(Dp_d g)[x] − (Dp_d f)[x−e_d]·(Dp_d g)[x−e_d]
//! ```
//!
//! `L_d` is the discrete Leibniz defect (backward difference of the forward-
//! difference product). It telescopes to zero on the torus; the residual is
//! exactly zero in `i128`.
//!
//! ## Frobenius pointwise certificate
//!
//! At every cell, with `q = ω^T (G + G^T) ω`, `F² = Σ (G_{cd} + G_{dc})²`,
//! `w² = |ω|²`:
//!
//! ```text
//! q² ≤ F² · w⁴        (Cauchy–Schwarz, exact integers)
//! ```
//!
//! [`frobenius_certificate`] returns the tightest observed `q² / (F² · w⁴)`
//! as an [`ExactRational`] and the maximum `F²` — the per-step Lyapunov-rate
//! observable that R2 N4-NSWEEP carries alongside `{ℓ(t), γ(t), ratio,
//! enstrophy}` on the live trajectory.

use crate::ns3d::{Field3D, N, Vel3D, Vort3D};
use qmnf_crtbigint::ExactRational;

/// Vector lattice field — three [`Field3D`] components.
pub type VectorField = [Field3D; 3];

#[inline]
fn zero_scalar() -> Field3D { [[[0i64; N]; N]; N] }
#[inline]
fn zero_vector() -> VectorField { [zero_scalar(); 3] }

/// Periodic shift of a scalar field by `s` cells along `axis`.
pub fn shift(f: &Field3D, axis: usize, s: isize) -> Field3D {
    let n = N as isize;
    let mut g = zero_scalar();
    for i in 0..N {
        for j in 0..N {
            for k in 0..N {
                let mut idx = [i as isize, j as isize, k as isize];
                idx[axis] = (idx[axis] + s).rem_euclid(n);
                g[i][j][k] = f[idx[0] as usize][idx[1] as usize][idx[2] as usize];
            }
        }
    }
    g
}

/// Central difference `d_axis f = f[x+e] − f[x−e]` (factor `2h` absorbed).
pub fn central_diff(f: &Field3D, axis: usize) -> Field3D {
    let fp = shift(f, axis, 1);
    let fm = shift(f, axis, -1);
    let mut g = zero_scalar();
    for i in 0..N { for j in 0..N { for k in 0..N {
        g[i][j][k] = fp[i][j][k] - fm[i][j][k];
    }}}
    g
}

/// Forward difference `Dp_axis f = f[x+e] − f[x]` (factor `h` absorbed).
pub fn forward_diff(f: &Field3D, axis: usize) -> Field3D {
    let fp = shift(f, axis, 1);
    let mut g = zero_scalar();
    for i in 0..N { for j in 0..N { for k in 0..N {
        g[i][j][k] = fp[i][j][k] - f[i][j][k];
    }}}
    g
}

/// Discrete curl `ω = ∇ × ψ` — divergence-free for central differences.
pub fn curl(psi: &VectorField) -> VectorField {
    let dpz_y = central_diff(&psi[2], 1);
    let dpy_z = central_diff(&psi[1], 2);
    let dpx_z = central_diff(&psi[0], 2);
    let dpz_x = central_diff(&psi[2], 0);
    let dpy_x = central_diff(&psi[1], 0);
    let dpx_y = central_diff(&psi[0], 1);

    let mut w = zero_vector();
    for i in 0..N { for j in 0..N { for k in 0..N {
        w[0][i][j][k] = dpz_y[i][j][k] - dpy_z[i][j][k];
        w[1][i][j][k] = dpx_z[i][j][k] - dpz_x[i][j][k];
        w[2][i][j][k] = dpy_x[i][j][k] - dpx_y[i][j][k];
    }}}
    w
}

/// Discrete divergence `div_h w = Σ_d d_d w_d`.
pub fn divergence(w: &VectorField) -> Field3D {
    let mut g = zero_scalar();
    for d in 0..3 {
        let dwd = central_diff(&w[d], d);
        for i in 0..N { for j in 0..N { for k in 0..N {
            g[i][j][k] += dwd[i][j][k];
        }}}
    }
    g
}

/// Discrete stretching `S_c = Σ_d ω_d · d_d u_c` (raw integer form).
///
/// Distinct from [`crate::ns3d::discrete_stretching`], which applies the
/// physical-units scaling `(raw · N) / (2 · NS_SCALE)`. The bare integer form
/// is the one the identities below are stated against.
pub fn stretching(w: &VectorField, u: &VectorField) -> VectorField {
    let mut s = zero_vector();
    for c in 0..3 {
        for d in 0..3 {
            let duc = central_diff(&u[c], d);
            for i in 0..N { for j in 0..N { for k in 0..N {
                s[c][i][j][k] += w[d][i][j][k] * duc[i][j][k];
            }}}
        }
    }
    s
}

/// Inner product `⟨a, b⟩ = Σ_x Σ_c a_c · b_c`, exact `i128`.
pub fn pair_dot(a: &VectorField, b: &VectorField) -> i128 {
    let mut t: i128 = 0;
    for c in 0..3 {
        for i in 0..N { for j in 0..N { for k in 0..N {
            t += (a[c][i][j][k] as i128) * (b[c][i][j][k] as i128);
        }}}
    }
    t
}

/// Identity 1 — strain quadratic decomposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrainQuadratic {
    /// `Σ ω_c ω_d G_{cd}` — equal to `⟨S(ω,u), ω⟩`.
    pub full: i128,
    /// `Σ ω_c ω_d (G_{cd} + G_{dc}) = 2 · full`.
    pub sym: i128,
    /// `Σ ω_c ω_d (G_{cd} − G_{dc}) ≡ 0`.
    pub anti: i128,
}

/// Strain quadratic decomposition of `⟨S(ω,u), ω⟩`.
pub fn strain_quadratic(w: &VectorField, u: &VectorField) -> StrainQuadratic {
    let g: [[Field3D; 3]; 3] =
        core::array::from_fn(|c| core::array::from_fn(|d| central_diff(&u[c], d)));

    let mut full: i128 = 0;
    let mut sym:  i128 = 0;
    let mut anti: i128 = 0;
    for i in 0..N { for j in 0..N { for k in 0..N {
        for c in 0..3 {
            for d in 0..3 {
                let wc  = w[c][i][j][k] as i128;
                let wd  = w[d][i][j][k] as i128;
                let gcd = g[c][d][i][j][k] as i128;
                let gdc = g[d][c][i][j][k] as i128;
                full += wc * wd * gcd;
                sym  += wc * wd * (gcd + gdc);
                anti += wc * wd * (gcd - gdc);
            }
        }
    }}}
    StrainQuadratic { full, sym, anti }
}

/// Identity 2 — flux/cocycle components.
#[derive(Debug, Clone, Copy)]
pub struct FluxIdentity {
    /// `⟨S(ω, u), ω⟩`.
    pub stretching_ip: i128,
    /// `⟨(ω · ∇_h) ω, u⟩ = Σ u_c · ω_d · d_d ω_c`.
    pub transport: i128,
    /// `Σ u_c · L_d(ω_c, ω_d)` — Leibniz defect cocycle (cubic flux).
    pub leibniz_flux: i128,
    /// `Σ (u · ω) · div_h ω` — vanishes exactly when ω is solenoidal.
    pub div_dot: i128,
    /// `stretching_ip + transport + leibniz_flux + div_dot` — Identity 2 says this is `0`.
    pub residual: i128,
}

fn transport_pair(w: &VectorField, u: &VectorField) -> i128 {
    let mut t: i128 = 0;
    for c in 0..3 {
        for d in 0..3 {
            let dwc = central_diff(&w[c], d);
            for i in 0..N { for j in 0..N { for k in 0..N {
                t += (u[c][i][j][k] as i128)
                    * (w[d][i][j][k] as i128)
                    * (dwc[i][j][k] as i128);
            }}}
        }
    }
    t
}

fn defect_local(w: &VectorField, u: &VectorField) -> i128 {
    let mut t3: i128 = 0;
    for c in 0..3 {
        for d in 0..3 {
            let wc  = &w[c];
            let wd  = &w[d];
            let wcp = shift(wc, d, 1);
            let wcm = shift(wc, d, -1);
            let wdp = shift(wd, d, 1);
            let wdm = shift(wd, d, -1);
            for i in 0..N { for j in 0..N { for k in 0..N {
                let a  = (wcp[i][j][k] - wc[i][j][k])  as i128;
                let b  = (wdp[i][j][k] - wd[i][j][k])  as i128;
                let ap = (wc[i][j][k]  - wcm[i][j][k]) as i128;
                let bp = (wd[i][j][k]  - wdm[i][j][k]) as i128;
                t3 += (u[c][i][j][k] as i128) * (a * b - ap * bp);
            }}}
        }
    }
    t3
}

fn div_dot_term(w: &VectorField, u: &VectorField) -> i128 {
    let dv = divergence(w);
    let mut t: i128 = 0;
    for i in 0..N { for j in 0..N { for k in 0..N {
        let s: i128 = (0..3)
            .map(|c| (u[c][i][j][k] as i128) * (w[c][i][j][k] as i128))
            .sum();
        t += s * (dv[i][j][k] as i128);
    }}}
    t
}

/// Compute all four Identity 2 components plus their residual (must be exactly `0`).
pub fn flux_identity(w: &VectorField, u: &VectorField) -> FluxIdentity {
    let stretching_ip = pair_dot(&stretching(w, u), w);
    let transport     = transport_pair(w, u);
    let leibniz_flux  = defect_local(w, u);
    let div_dot       = div_dot_term(w, u);
    let residual      = stretching_ip + transport + leibniz_flux + div_dot;
    FluxIdentity { stretching_ip, transport, leibniz_flux, div_dot, residual }
}

/// Frobenius pointwise certificate report.
#[derive(Debug, Clone)]
pub struct FrobeniusReport {
    /// `true` iff `q² ≤ F² · w⁴` holds at every cell.
    pub cert_holds_everywhere: bool,
    /// Maximum observed ratio `q² / (F² · w⁴) ≤ 1` across all cells, exact rational.
    pub tightest_ratio: ExactRational,
    /// `max_x ‖E_h(x)‖²_F` as exact `i128` — the per-step Lyapunov-rate
    /// observable. R2 N4-NSWEEP carries this alongside
    /// `{ℓ(t), γ(t), ratio, enstrophy}`.
    pub max_frob_sq: i128,
}

/// Pointwise Frobenius certificate `q² ≤ F² · w⁴` with `q = ω^T (G+G^T) ω`,
/// `F² = Σ (G_{cd}+G_{dc})²`, `w² = |ω|²`, plus the maximum Frobenius square
/// (Lyapunov-rate observable).
pub fn frobenius_certificate(w: &VectorField, u: &VectorField) -> FrobeniusReport {
    let g: [[Field3D; 3]; 3] =
        core::array::from_fn(|c| core::array::from_fn(|d| central_diff(&u[c], d)));

    let mut cert_holds = true;
    let mut worst = ExactRational::new(0, 1);
    let mut max_frob: i128 = 0;

    for i in 0..N { for j in 0..N { for k in 0..N {
        let mut q:       i128 = 0;
        let mut frob_sq: i128 = 0;
        let mut w_sq:    i128 = 0;
        for c in 0..3 {
            let wc = w[c][i][j][k] as i128;
            w_sq += wc * wc;
            for d in 0..3 {
                let gcd = g[c][d][i][j][k] as i128;
                let gdc = g[d][c][i][j][k] as i128;
                let wd  = w[d][i][j][k]   as i128;
                let s   = gcd + gdc;
                q       += wc * wd * s;
                frob_sq += s * s;
            }
        }
        if frob_sq > max_frob { max_frob = frob_sq; }

        let lhs = q * q;                       // q²  (nonneg)
        let rhs = frob_sq * w_sq * w_sq;       // F² · w⁴
        if lhs > rhs { cert_holds = false; }

        if rhs > 0 {
            // q² ≥ 0 always; safe cast.
            let candidate = ExactRational::new(lhs as u128, rhs as u128);
            if worst.lt(&candidate) { worst = candidate; }
        }
    }}}

    FrobeniusReport {
        cert_holds_everywhere: cert_holds,
        tightest_ratio: worst,
        max_frob_sq: max_frob,
    }
}

/// Standalone `max_x ‖E_h(x)‖²_F` — the per-step Lyapunov-rate observable.
///
/// Carry this alongside `{ℓ(t), γ(t), ratio, enstrophy}` on the live
/// trajectory (R2 N4-NSWEEP target — direct integer telemetry).
pub fn frobenius_max_sq(u: &VectorField) -> i128 {
    let g: [[Field3D; 3]; 3] =
        core::array::from_fn(|c| core::array::from_fn(|d| central_diff(&u[c], d)));
    let mut max_frob: i128 = 0;
    for i in 0..N { for j in 0..N { for k in 0..N {
        let mut frob_sq: i128 = 0;
        for c in 0..3 {
            for d in 0..3 {
                let s = (g[c][d][i][j][k] + g[d][c][i][j][k]) as i128;
                frob_sq += s * s;
            }
        }
        if frob_sq > max_frob { max_frob = frob_sq; }
    }}}
    max_frob
}

/// Adapter: build a [`VectorField`] from a [`Vort3D`] vorticity field.
pub fn from_vort3d(w: &Vort3D) -> VectorField {
    [w.wx, w.wy, w.wz]
}

/// Adapter: build a [`VectorField`] from a [`Vel3D`] velocity field.
pub fn from_vel3d(u: &Vel3D) -> VectorField {
    [u.ux, u.uy, u.uz]
}

// ============================================================================
// TESTS — exact-integer reproduction of the Python instrument's outputs
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- deterministic seed builders (mirror the Python instrument) ----------

    /// Integer "sine-ish" table from `structb.py` — exact, no float.
    /// `[0, 7, 10, 7, 0, -7, -10, -7]` (period-8).
    const TBL: [i64; N] = [0, 7, 10, 7, 0, -7, -10, -7];

    fn ctb_at(i: usize) -> i64 { TBL[(i + 2) % N] }

    /// TG-like ψ seed: `ψ_x[i,j,k] =  amp · TBL[i] · CTB[j]`,
    /// `ψ_y[i,j,k] = -amp · CTB[i] · TBL[j]`, `ψ_z = 0`.
    fn psi_tg_like(amp: i64) -> VectorField {
        let mut psi = zero_vector();
        for i in 0..N { for j in 0..N { for k in 0..N {
            psi[0][i][j][k] =  amp * TBL[i]    * ctb_at(j);
            psi[1][i][j][k] = -amp * ctb_at(i) * TBL[j];
            let _ = k;
        }}}
        psi
    }

    /// Checkerboard ψ: `ψ_c = amp · (-1)^(i+j+k)` (sign alternates per cell).
    fn psi_checkerboard(amp: i64) -> VectorField {
        let mut psi = zero_vector();
        for i in 0..N { for j in 0..N { for k in 0..N {
            let s: i64 = if (i + j + k) % 2 == 0 { 1 } else { -1 };
            psi[0][i][j][k] =  amp * s;
            psi[1][i][j][k] = -amp * s;
            psi[2][i][j][k] =  amp * s;
        }}}
        psi
    }

    /// Delta ψ: three nonzero cells, amplitude `amp`.
    fn psi_delta(amp: i64) -> VectorField {
        let mut psi = zero_vector();
        psi[0][3][3][3] =  amp;
        psi[1][4][3][3] = -amp;
        psi[2][3][4][3] =  amp;
        psi
    }

    /// Deterministic linear-congruential pseudorandom field for u-side tests.
    /// Uses Numerical Recipes constants (no float, no system RNG); reproducible
    /// across Rust toolchains.
    fn lcg_field(seed: u64, lo: i64, hi: i64) -> Field3D {
        let span = (hi - lo + 1) as u64;
        let mut state = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let mut f = zero_scalar();
        for i in 0..N { for j in 0..N { for k in 0..N {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            f[i][j][k] = lo + ((state >> 16) % span) as i64;
        }}}
        f
    }

    fn psi_random(seed: u64, lo: i64, hi: i64) -> VectorField {
        [lcg_field(seed,         lo, hi),
         lcg_field(seed ^ 0xA1,  lo, hi),
         lcg_field(seed ^ 0xB2,  lo, hi)]
    }

    // ---------- Identity 1: strain form ----------

    /// `anti ≡ 0` for any seeds (integer algebra — no smallness, no analysis).
    #[test]
    fn identity1_antisymmetric_part_vanishes() {
        let seeds: &[(&str, VectorField, VectorField)] = &[
            ("TG/TG-shift",     curl(&psi_tg_like(3)), curl(&psi_tg_like(2))),
            ("checker/TG",      curl(&psi_checkerboard(5)), curl(&psi_tg_like(2))),
            ("delta/TG",        curl(&psi_delta(100)), curl(&psi_tg_like(2))),
            ("rand/rand",       curl(&psi_random(0xC0FFEE, -9, 9)),
                                curl(&psi_random(0xDECAFB,  -20, 20))),
            ("rand-large/rand", curl(&psi_random(0xFEEDBEEF, -999, 999)),
                                curl(&psi_random(0xBADCAFE,   -50, 50))),
        ];
        for (label, w, u) in seeds {
            let sq = strain_quadratic(w, u);
            assert_eq!(sq.anti, 0,
                "antisymmetric part nonzero on seed '{}' (got {})", label, sq.anti);
            assert_eq!(sq.sym, 2 * sq.full,
                "sym ≠ 2·full on seed '{}'", label);
        }
    }

    /// `full ≡ ⟨S(ω,u), ω⟩` (strain decomposition matches the operator).
    #[test]
    fn identity1_full_matches_stretching_ip() {
        let w = curl(&psi_delta(100));
        let u = curl(&psi_tg_like(2));
        let sq = strain_quadratic(&w, &u);
        let ip = pair_dot(&stretching(&w, &u), &w);
        assert_eq!(sq.full, ip,
            "Identity 1: full ({}) ≠ ⟨S,ω⟩ ({})", sq.full, ip);
    }

    /// Structured seeds (TG, checkerboard, single modes) produce exact-zero
    /// production against ω = curl(ψ_self) — the discrete triad-interaction
    /// requirement at the integer level (ledger #127 finding).
    #[test]
    fn structured_seeds_self_produce_zero() {
        let cases: &[(&str, VectorField)] = &[
            ("TG amp=3",          curl(&psi_tg_like(3))),
            ("checker amp=5",     curl(&psi_checkerboard(5))),
        ];
        for (label, w) in cases {
            let u_self: VectorField = *w;
            let sq = strain_quadratic(w, &u_self);
            assert_eq!(sq.full, 0,
                "structured seed '{}' should self-produce zero (got full={})",
                label, sq.full);
        }
    }

    // ---------- Identity 2: flux/cocycle form ----------

    /// `⟨S,ω⟩ + ⟨(ω·∇)ω, u⟩ + Σ u·L + Σ (u·ω) div ω = 0` exactly, for solenoidal ω.
    #[test]
    fn identity2_residual_is_zero_for_solenoidal_seeds() {
        let seeds: &[(&str, VectorField, VectorField)] = &[
            ("TG/TG-shift",  curl(&psi_tg_like(3)), curl(&psi_tg_like(2))),
            ("checker/TG",   curl(&psi_checkerboard(5)), curl(&psi_tg_like(2))),
            ("delta/TG",     curl(&psi_delta(100)), curl(&psi_tg_like(2))),
            ("rand/rand",    curl(&psi_random(0xC0FFEE, -9, 9)),
                             curl(&psi_random(0xDECAFB,  -20, 20))),
            ("rand-large",   curl(&psi_random(0xFEEDBEEF, -999, 999)),
                             curl(&psi_random(0xBADCAFE,   -50, 50))),
        ];
        for (label, w, u) in seeds {
            let fi = flux_identity(w, u);
            assert_eq!(fi.residual, 0,
                "Identity 2 residual nonzero on seed '{}': {:?}", label, fi);
            assert_eq!(fi.div_dot, 0,
                "div_dot must vanish for solenoidal ω on seed '{}'", label);
        }
    }

    // ---------- Frobenius pointwise certificate ----------

    /// `q² ≤ F²·|ω|⁴` holds at every cell, across structured + random seeds.
    #[test]
    fn frobenius_cert_holds_pointwise() {
        let seeds: &[(&str, VectorField, VectorField)] = &[
            ("TG/TG",        curl(&psi_tg_like(3)),       curl(&psi_tg_like(2))),
            ("checker/TG",   curl(&psi_checkerboard(5)),  curl(&psi_tg_like(2))),
            ("delta/TG",     curl(&psi_delta(100)),       curl(&psi_tg_like(2))),
            ("rand-small",   curl(&psi_random(0xC0FFEE, -9, 9)),
                             curl(&psi_random(0xDECAFB, -20, 20))),
            ("rand-medium",  curl(&psi_random(0x11_2233, -99, 99)),
                             curl(&psi_random(0x44_5566, -50, 50))),
            ("rand-large",   curl(&psi_random(0xFEEDBEEF, -999, 999)),
                             curl(&psi_random(0xBADCAFE,   -50, 50))),
        ];
        let bound = ExactRational::new(1, 1);
        for (label, w, u) in seeds {
            let r = frobenius_certificate(w, u);
            assert!(r.cert_holds_everywhere,
                "Frobenius certificate violated on '{}'", label);
            assert!(r.tightest_ratio.le(&bound),
                "tightest ratio {} not ≤ 1 on '{}'", r.tightest_ratio, label);
            assert!(r.max_frob_sq >= 0,
                "max Frobenius square negative on '{}'", label);
        }
    }

    /// `frobenius_max_sq` agrees with the maximum from `frobenius_certificate`.
    #[test]
    fn frobenius_max_sq_matches_certificate_max() {
        let w = curl(&psi_random(0xC0FFEE, -9, 9));
        let u = curl(&psi_random(0xDECAFB, -20, 20));
        let r = frobenius_certificate(&w, &u);
        let m = frobenius_max_sq(&u);
        assert_eq!(r.max_frob_sq, m,
            "Lyapunov observable disagreement: cert.max_frob_sq={}, direct={}",
            r.max_frob_sq, m);
    }

    // ---------- Sanity ----------

    #[test]
    fn curl_is_solenoidal_to_integer_zero() {
        let w = curl(&psi_random(0xC0FFEE, -9, 9));
        let dv = divergence(&w);
        for i in 0..N { for j in 0..N { for k in 0..N {
            assert_eq!(dv[i][j][k], 0,
                "div curl ≠ 0 at ({}, {}, {})", i, j, k);
        }}}
    }

    #[test]
    fn vort3d_adapter_matches_array_form() {
        let psi = psi_tg_like(3);
        let w_arr = curl(&psi);
        let v3 = Vort3D { wx: w_arr[0], wy: w_arr[1], wz: w_arr[2] };
        let recovered = from_vort3d(&v3);
        assert_eq!(recovered, w_arr);
    }
}
