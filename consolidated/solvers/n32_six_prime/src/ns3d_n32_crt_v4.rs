//! # CRT-Native NS Step v4 — u128 Substrate
//!
//! v3 was bit-identical to canonical at amp=100, but multistep at amp=1000
//! diverged at step 11 because the diffusion division `(nu·lap)/S²` produces
//! q ≈ 1.9×10¹³, which exceeds v3's lift corridor `M_NTT · partial_mod_5 =
//! M_NTT · (5⁹/5⁸) = M_NTT · 5 ≈ 5×10¹²`. The bug is structural: gcd(S², 5⁹)
//! = 5⁸ leaves only 5⁹/5⁸ = 5 of partial modulus on lane 5.
//!
//! v4 widens the lanes per directive (u128 substrate, no i64 in arithmetic):
//! ```text
//!   EXT_BASIS_U128 = [769, 929, 1153, 1217, 2⁶³, 5²⁵]
//!   M_small = 2⁶³ · 5²⁵ ≈ 2.7 × 10³⁶
//! ```
//! For S² = 2⁸·5⁸: gcd(S², 5²⁵) = 5⁸, partial_mod_5 = 5²⁵/5⁸ = 5¹⁷ ≈ 7.6×10¹¹.
//! New corridor M_NTT · 5¹⁷ ≈ 7.6×10²³. Step-11 q sits 11 orders of magnitude
//! inside.

use crate::ns3d::{modinv, mulmod, submod};
use crate::ns3d_n32::{N, NS_SCALE, NTT_PRIMES, I_MOD, NsParams, Vort3D, Field3D,
                       zero_field, ntt3d, intt3d};
use crate::cram_div_u128::{
    ExtResU128, EXT_BASIS_U128, M_NTT_U128,
    cram_div_exact_pos_u128, addmod_u128, submod_u128, mulmod_u128, modinv_u128,
};

#[inline]
fn cfreq(k: usize) -> i64 {
    if k <= N / 2 { k as i64 } else { k as i64 - N as i64 }
}

// ============================================================================
// State
// ============================================================================

pub struct CRTVortExtU128 {
    pub wx: Box<[[[ExtResU128; N]; N]; N]>,
    pub wy: Box<[[[ExtResU128; N]; N]; N]>,
    pub wz: Box<[[[ExtResU128; N]; N]; N]>,
}

impl CRTVortExtU128 {
    pub fn from_i64(w: &Vort3D) -> Self {
        let mut wx = zero_ext_field();
        let mut wy = zero_ext_field();
        let mut wz = zero_ext_field();
        for i in 0..N { for j in 0..N { for k in 0..N {
            wx[i][j][k] = ExtResU128::from_i128(w.wx[i][j][k] as i128);
            wy[i][j][k] = ExtResU128::from_i128(w.wy[i][j][k] as i128);
            wz[i][j][k] = ExtResU128::from_i128(w.wz[i][j][k] as i128);
        }}}
        CRTVortExtU128 { wx, wy, wz }
    }

    /// Reconstruct as i128 — never narrows to i64.
    pub fn to_i128_field(&self) -> (Box<[[[i128; N]; N]; N]>,
                                     Box<[[[i128; N]; N]; N]>,
                                     Box<[[[i128; N]; N]; N]>) {
        let mut wx = Box::new([[[0i128; N]; N]; N]);
        let mut wy = Box::new([[[0i128; N]; N]; N]);
        let mut wz = Box::new([[[0i128; N]; N]; N]);
        for i in 0..N { for j in 0..N { for k in 0..N {
            wx[i][j][k] = self.wx[i][j][k].to_i128();
            wy[i][j][k] = self.wy[i][j][k].to_i128();
            wz[i][j][k] = self.wz[i][j][k].to_i128();
        }}}
        (wx, wy, wz)
    }

    /// Reconstruct with explicit saturating clamp to i64 — for comparison
    /// against canonical's clamped Vort3D.
    pub fn to_i64_clamped(&self) -> Vort3D {
        let mut out = Vort3D::zero();
        for i in 0..N { for j in 0..N { for k in 0..N {
            out.wx[i][j][k] = self.wx[i][j][k].to_i128()
                .clamp(i64::MIN as i128, i64::MAX as i128) as i64;
            out.wy[i][j][k] = self.wy[i][j][k].to_i128()
                .clamp(i64::MIN as i128, i64::MAX as i128) as i64;
            out.wz[i][j][k] = self.wz[i][j][k].to_i128()
                .clamp(i64::MIN as i128, i64::MAX as i128) as i64;
        }}}
        out
    }
}

fn zero_ext_field() -> Box<[[[ExtResU128; N]; N]; N]> {
    Box::new(std::array::from_fn(|_|
        std::array::from_fn(|_|
            std::array::from_fn(|_| ExtResU128::zero()))))
}

// ============================================================================
// ExtResU128 arithmetic
// ============================================================================

fn ext_add(a: &ExtResU128, b: &ExtResU128) -> ExtResU128 {
    let mut r = [0u128; 6];
    for i in 0..6 {
        r[i] = addmod_u128(a.res[i], b.res[i], EXT_BASIS_U128[i]);
    }
    ExtResU128 { res: r }
}

fn ext_sub(a: &ExtResU128, b: &ExtResU128) -> ExtResU128 {
    let mut r = [0u128; 6];
    for i in 0..6 {
        r[i] = submod_u128(a.res[i], b.res[i], EXT_BASIS_U128[i]);
    }
    ExtResU128 { res: r }
}

fn ext_mul(a: &ExtResU128, b: &ExtResU128) -> ExtResU128 {
    let mut r = [0u128; 6];
    for i in 0..6 {
        r[i] = mulmod_u128(a.res[i], b.res[i], EXT_BASIS_U128[i]);
    }
    ExtResU128 { res: r }
}

fn ext_neg(a: &ExtResU128) -> ExtResU128 {
    let mut r = [0u128; 6];
    for i in 0..6 {
        let m = EXT_BASIS_U128[i];
        r[i] = if a.res[i] == 0 { 0 } else { m - a.res[i] };
    }
    ExtResU128 { res: r }
}

fn ext_smul(a: &ExtResU128, k: i128) -> ExtResU128 {
    let mut r = [0u128; 6];
    for i in 0..6 {
        let m = EXT_BASIS_U128[i];
        let k_mod = k.rem_euclid(m as i128) as u128;
        r[i] = mulmod_u128(a.res[i], k_mod, m);
    }
    ExtResU128 { res: r }
}

fn ext_from_i128(x: i128) -> ExtResU128 { ExtResU128::from_i128(x) }

fn ext_div_signed(a: &ExtResU128, d: u128) -> ExtResU128 {
    let val = a.to_i128();
    if val >= 0 {
        cram_div_exact_pos_u128(a, d)
    } else {
        let neg_a = ext_neg(a);
        let q_pos = cram_div_exact_pos_u128(&neg_a, d);
        ext_neg(&q_pos)
    }
}

// ============================================================================
// Biot–Savart in u128 substrate
// ============================================================================

struct ExtVelU128 {
    ux: Box<[[[ExtResU128; N]; N]; N]>,
    uy: Box<[[[ExtResU128; N]; N]; N]>,
    uz: Box<[[[ExtResU128; N]; N]; N]>,
}

fn biot_savart_u128(w: &CRTVortExtU128) -> ExtVelU128 {
    let mut ux_per_prime: Vec<Box<[[[u64; N]; N]; N]>> = (0..4).map(|_|
        Box::new([[[0u64; N]; N]; N])).collect();
    let mut uy_per_prime: Vec<Box<[[[u64; N]; N]; N]>> = (0..4).map(|_|
        Box::new([[[0u64; N]; N]; N])).collect();
    let mut uz_per_prime: Vec<Box<[[[u64; N]; N]; N]>> = (0..4).map(|_|
        Box::new([[[0u64; N]; N]; N])).collect();

    for pi in 0..4 {
        let p = NTT_PRIMES[pi];
        let i_p = I_MOD[pi];

        let mut wx_i64: Field3D = zero_field();
        let mut wy_i64: Field3D = zero_field();
        let mut wz_i64: Field3D = zero_field();
        for i in 0..N { for j in 0..N { for k in 0..N {
            wx_i64[i][j][k] = w.wx[i][j][k].res[pi] as i64;
            wy_i64[i][j][k] = w.wy[i][j][k].res[pi] as i64;
            wz_i64[i][j][k] = w.wz[i][j][k].res[pi] as i64;
        }}}

        let wx_hat = ntt3d(&wx_i64, pi);
        let wy_hat = ntt3d(&wy_i64, pi);
        let wz_hat = ntt3d(&wz_i64, pi);

        let mut ux_hat = Box::new([[[0u64; N]; N]; N]);
        let mut uy_hat = Box::new([[[0u64; N]; N]; N]);
        let mut uz_hat = Box::new([[[0u64; N]; N]; N]);

        let scale_mod = |k: i64, x: u64| -> u64 {
            if k >= 0 { mulmod((k as u64) % p, x, p) }
            else      { submod(0, mulmod(((-k) as u64) % p, x, p), p) }
        };

        for i0 in 0..N { for i1 in 0..N { for i2 in 0..N {
            let kx = cfreq(i0);
            let ky = cfreq(i1);
            let kz = cfreq(i2);
            let ksq = (kx*kx + ky*ky + kz*kz) as u64;
            if ksq == 0 { continue; }
            let inv_ksq = modinv(ksq % p, p);
            let cx = submod(
                scale_mod(ky, mulmod(wz_hat[i0][i1][i2], inv_ksq, p)),
                scale_mod(kz, mulmod(wy_hat[i0][i1][i2], inv_ksq, p)), p);
            let cy = submod(
                scale_mod(kz, mulmod(wx_hat[i0][i1][i2], inv_ksq, p)),
                scale_mod(kx, mulmod(wz_hat[i0][i1][i2], inv_ksq, p)), p);
            let cz = submod(
                scale_mod(kx, mulmod(wy_hat[i0][i1][i2], inv_ksq, p)),
                scale_mod(ky, mulmod(wx_hat[i0][i1][i2], inv_ksq, p)), p);
            ux_hat[i0][i1][i2] = mulmod(i_p, cx, p);
            uy_hat[i0][i1][i2] = mulmod(i_p, cy, p);
            uz_hat[i0][i1][i2] = mulmod(i_p, cz, p);
        }}}

        let ux_r = intt3d(&*ux_hat, pi);
        let uy_r = intt3d(&*uy_hat, pi);
        let uz_r = intt3d(&*uz_hat, pi);

        for i in 0..N { for j in 0..N { for k in 0..N {
            ux_per_prime[pi][i][j][k] = ux_r[i][j][k];
            uy_per_prime[pi][i][j][k] = uy_r[i][j][k];
            uz_per_prime[pi][i][j][k] = uz_r[i][j][k];
        }}}
    }

    let n6: u128 = 1u128 << 30;
    let mut ux = zero_ext_field();
    let mut uy = zero_ext_field();
    let mut uz = zero_ext_field();
    for i in 0..N { for j in 0..N { for k in 0..N {
        let raw_x_i128 = u128_garner_centered(&[
            ux_per_prime[0][i][j][k] as u128, ux_per_prime[1][i][j][k] as u128,
            ux_per_prime[2][i][j][k] as u128, ux_per_prime[3][i][j][k] as u128,
        ]);
        let raw_y_i128 = u128_garner_centered(&[
            uy_per_prime[0][i][j][k] as u128, uy_per_prime[1][i][j][k] as u128,
            uy_per_prime[2][i][j][k] as u128, uy_per_prime[3][i][j][k] as u128,
        ]);
        let raw_z_i128 = u128_garner_centered(&[
            uz_per_prime[0][i][j][k] as u128, uz_per_prime[1][i][j][k] as u128,
            uz_per_prime[2][i][j][k] as u128, uz_per_prime[3][i][j][k] as u128,
        ]);

        let ux_raw_ext = ExtResU128::from_i128(raw_x_i128);
        let uy_raw_ext = ExtResU128::from_i128(raw_y_i128);
        let uz_raw_ext = ExtResU128::from_i128(raw_z_i128);

        ux[i][j][k] = ext_div_signed(&ux_raw_ext, n6);
        uy[i][j][k] = ext_div_signed(&uy_raw_ext, n6);
        uz[i][j][k] = ext_div_signed(&uz_raw_ext, n6);
    }}}

    ExtVelU128 { ux, uy, uz }
}

/// u128 Garner reconstruction in [0, M_NTT) → centered i128 at M_NTT/2.
/// Matches canonical garner4 semantics but never narrows to i64.
fn u128_garner_centered(r: &[u128; 4]) -> i128 {
    let p: [u128; 4] = [769, 929, 1153, 1217];
    let a0 = r[0];
    let a1 = mulmod_u128(submod_u128(r[1], a0 % p[1], p[1]),
                         modinv_u128(p[0] % p[1], p[1]), p[1]);
    let p01 = p[0] * p[1];
    let t2 = (a0 + p[0] * a1) % p[2];
    let a2 = mulmod_u128(submod_u128(r[2], t2, p[2]),
                         modinv_u128(p01 % p[2], p[2]), p[2]);
    let p012 = p01 * p[2];
    let t3 = (a0 + p[0] * a1 + p01 * a2) % p[3];
    let a3 = mulmod_u128(submod_u128(r[3], t3, p[3]),
                         modinv_u128(p012 % p[3], p[3]), p[3]);
    let val = a0 + p[0] * a1 + p01 * a2 + p012 * a3;
    let half_m = M_NTT_U128 / 2;
    if val <= half_m { val as i128 } else { val as i128 - M_NTT_U128 as i128 }
}

// ============================================================================
// THE STEP — v4
// ============================================================================

pub fn ns_step_crt_v4(w: &CRTVortExtU128, params: &NsParams) -> CRTVortExtU128 {
    let vel = biot_savart_u128(w);

    let nu = ext_from_i128(params.nu_scaled as i128);
    let dt = ext_from_i128(params.dt_latt as i128);
    let s2_div: u128 = (NS_SCALE as u128) * (NS_SCALE as u128);
    let n2_div: u128 = (N as u128) * (N as u128);
    let four_s_div: u128 = 4 * NS_SCALE as u128;
    let two_s_div:  u128 = 2 * NS_SCALE as u128;
    let n_ext = ext_from_i128(N as i128);

    let step_component = |
        wl: &Box<[[[ExtResU128; N]; N]; N]>,
        ul: &Box<[[[ExtResU128; N]; N]; N]>,
        u_x: &Box<[[[ExtResU128; N]; N]; N]>,
        u_y: &Box<[[[ExtResU128; N]; N]; N]>,
        u_z: &Box<[[[ExtResU128; N]; N]; N]>,
        full_w: &CRTVortExtU128,
    | -> Box<[[[ExtResU128; N]; N]; N]> {
        let mut out = zero_ext_field();
        for i in 0..N {
            let ip=(i+1)%N; let im=(i+N-1)%N;
            for j in 0..N {
                let jp=(j+1)%N; let jm=(j+N-1)%N;
                for k in 0..N {
                    let kp=(k+1)%N; let km=(k+N-1)%N;

                    let lap = {
                        let sum = ext_add(
                            &ext_add(&ext_add(&wl[ip][j][k], &wl[im][j][k]),
                                     &ext_add(&wl[i][jp][k], &wl[i][jm][k])),
                            &ext_add(&wl[i][j][kp], &wl[i][j][km]));
                        let six = ext_smul(&wl[i][j][k], 6);
                        ext_sub(&sum, &six)
                    };
                    let nu_lap = ext_mul(&nu, &lap);
                    let diff = ext_div_signed(&nu_lap, s2_div);

                    let u_fields = [u_x, u_y, u_z];
                    let sp = [(ip,j,k), (i,jp,k), (i,j,kp)];
                    let sm = [(im,j,k), (i,jm,k), (i,j,km)];
                    let mut raw = ExtResU128::zero();
                    for d in 0..3 {
                        let (sp0,sp1,sp2) = sp[d];
                        let (sm0,sm1,sm2) = sm[d];
                        let u_here  = &u_fields[d][i][j][k];
                        let u_plus  = &u_fields[d][sp0][sp1][sp2];
                        let u_minus = &u_fields[d][sm0][sm1][sm2];
                        let f_plus  = &wl[sp0][sp1][sp2];
                        let f_minus = &wl[sm0][sm1][sm2];
                        let df_pm = ext_sub(f_plus, f_minus);
                        let conv = ext_mul(u_here, &df_pm);
                        let cons = ext_sub(&ext_mul(u_plus, f_plus),
                                           &ext_mul(u_minus, f_minus));
                        raw = ext_add(&raw, &ext_add(&conv, &cons));
                    }
                    let raw_n = ext_mul(&raw, &n_ext);
                    let adv = ext_div_signed(&raw_n, four_s_div);

                    let dux = ext_sub(&ul[ip][j][k], &ul[im][j][k]);
                    let duy = ext_sub(&ul[i][jp][k], &ul[i][jm][k]);
                    let duz = ext_sub(&ul[i][j][kp], &ul[i][j][km]);
                    let mut raw_s = ExtResU128::zero();
                    raw_s = ext_add(&raw_s, &ext_mul(&full_w.wx[i][j][k], &dux));
                    raw_s = ext_add(&raw_s, &ext_mul(&full_w.wy[i][j][k], &duy));
                    raw_s = ext_add(&raw_s, &ext_mul(&full_w.wz[i][j][k], &duz));
                    let raw_s_n = ext_mul(&raw_s, &n_ext);
                    let stretch = ext_div_signed(&raw_s_n, two_s_div);

                    let rhs = ext_add(&ext_sub(&diff, &adv), &stretch);
                    let dt_rhs = ext_mul(&dt, &rhs);
                    let delta = ext_div_signed(&dt_rhs, n2_div);

                    out[i][j][k] = ext_add(&wl[i][j][k], &delta);
                }
            }
        }
        out
    };

    let wx_next = step_component(&w.wx, &vel.ux, &vel.ux, &vel.uy, &vel.uz, w);
    let wy_next = step_component(&w.wy, &vel.uy, &vel.ux, &vel.uy, &vel.uz, w);
    let wz_next = step_component(&w.wz, &vel.uz, &vel.ux, &vel.uy, &vel.uz, w);

    CRTVortExtU128 { wx: wx_next, wy: wy_next, wz: wz_next }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// v4 = canonical at amp=100 single step.
    #[test]
    fn crt_v4_step_matches_canonical_small_amp() {
        std::thread::Builder::new().stack_size(256 * 1024 * 1024).spawn(|| {
            use crate::ns3d_n32::ns_step as ns_step_canonical;
            let amp = 100i64;
            let w0 = Vort3D::taylor_green(amp);
            let params = NsParams::cfl_max(NS_SCALE * NS_SCALE / 25);

            let w1_canonical = ns_step_canonical(&w0, &params);
            let w_ext = CRTVortExtU128::from_i64(&w0);
            let w1_ext = ns_step_crt_v4(&w_ext, &params);
            let w1_recon = w1_ext.to_i64_clamped();

            let mut mismatches = 0usize;
            for i in 0..N { for j in 0..N { for k in 0..N {
                if w1_canonical.wx[i][j][k] != w1_recon.wx[i][j][k] { mismatches += 1; }
                if w1_canonical.wy[i][j][k] != w1_recon.wy[i][j][k] { mismatches += 1; }
                if w1_canonical.wz[i][j][k] != w1_recon.wz[i][j][k] { mismatches += 1; }
            }}}

            println!("v4 step at amp={amp}: {mismatches} mismatches");
            assert_eq!(mismatches, 0, "v4 single step at amp=100 vs canonical");
        }).unwrap().join().unwrap();
    }

    /// v4 multistep at amp=100 — 5 steps, expect 0 mismatches throughout.
    #[test]
    fn crt_v4_step_matches_canonical_multistep_amp100() {
        std::thread::Builder::new().stack_size(256 * 1024 * 1024).spawn(|| {
            use crate::ns3d_n32::ns_step as ns_step_canonical;
            let amp = 100i64;
            let mut w_canon = Vort3D::taylor_green(amp);
            let params = NsParams::cfl_max(NS_SCALE * NS_SCALE / 25);
            let mut w_v4 = CRTVortExtU128::from_i64(&w_canon);

            for step in 1..=5 {
                w_canon = ns_step_canonical(&w_canon, &params);
                w_v4 = ns_step_crt_v4(&w_v4, &params);
                let w_recon = w_v4.to_i64_clamped();
                let mut mismatches = 0usize;
                for i in 0..N { for j in 0..N { for k in 0..N {
                    if w_canon.wx[i][j][k] != w_recon.wx[i][j][k] { mismatches += 1; }
                    if w_canon.wy[i][j][k] != w_recon.wy[i][j][k] { mismatches += 1; }
                    if w_canon.wz[i][j][k] != w_recon.wz[i][j][k] { mismatches += 1; }
                }}}
                println!("v4 multistep step {}: {} mismatches", step, mismatches);
                assert_eq!(mismatches, 0, "v4 multistep at amp=100 vs canonical step {}", step);
            }
        }).unwrap().join().unwrap();
    }
}
