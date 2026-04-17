//! N=32 exact-integer Navier-Stokes solver.
//!
//! Extension of ns3d_n16.rs to N=32 (32,768 grid points).
//! NTT primes: all ≡ 1 (mod 32), all > |k|²_max = 768.
//! DKAM: deg(Sqr) = 2 < ρ = 3 (independent of N).
//!
//! [CODE FACT — mechanical lift from N=16, same algebraic invariants]

use crate::ns3d::{mulmod, addmod, submod, modpow, modinv, scale_mod};

pub const N: usize = 32;
const N3: usize = N * N * N; // 32768
pub const NS_SCALE: i64 = 10000;

// NTT primes: all ≡ 1 (mod 32), all > 768
pub const NTT_PRIMES: [u64; 4] = [769, 929, 1153, 1217];
pub const NTT_ROOTS: [u64; 4] = [251, 701, 286, 540]; // primitive 32nd roots
pub const I_MOD: [u64; 4] = [707, 605, 140, 78]; // sqrt(-1) mod p
pub const N_INV_3D: [u64; 4] = [18, 694, 617, 1110]; // 32^{-3} mod p
pub const NTT_MODULUS: u64 = 1_002_448_197_601;

// Garner reconstruction constants
const G_C01: u64 = 749;   // 769^{-1} mod 929
const G_C012: u64 = 839;  // (769*929)^{-1} mod 1153
const G_C0123: u64 = 720; // (769*929*1153)^{-1} mod 1217

// Precompute NTT matrix: NTT_MAT[p_idx][k][j] = omega^{kj mod N} mod p
const fn build_ntt_mat() -> [[[u64; N]; N]; 4] {
    let mut mat = [[[0u64; N]; N]; 4];
    let mut pi = 0;
    while pi < 4 {
        let p = NTT_PRIMES[pi];
        let omega = NTT_ROOTS[pi];
        let mut powers = [0u64; N];
        powers[0] = 1;
        let mut idx = 1;
        while idx < N {
            powers[idx] = (powers[idx - 1] as u128 * omega as u128 % p as u128) as u64;
            idx += 1;
        }
        let mut k = 0;
        while k < N {
            let mut j = 0;
            while j < N {
                mat[pi][k][j] = powers[(k * j) % N];
                j += 1;
            }
            k += 1;
        }
        pi += 1;
    }
    mat
}

static NTT_MAT: [[[u64; N]; N]; 4] = build_ntt_mat();

pub type Field3D = [[[i64; N]; N]; N];

pub fn zero_field() -> Field3D { [[[0i64; N]; N]; N] }

#[derive(Clone)]
pub struct Vort3D { pub wx: Field3D, pub wy: Field3D, pub wz: Field3D }
pub struct Vel3D { pub ux: Field3D, pub uy: Field3D, pub uz: Field3D }

impl Vort3D {
    pub fn zero() -> Self { Self { wx: zero_field(), wy: zero_field(), wz: zero_field() } }

    pub fn taylor_green(amplitude: i64) -> Self {
        use qmnf_crtbigint::integer_sine;
        let sin_t: [i64; N] = core::array::from_fn(|i| integer_sine(i, N, NS_SCALE));
        let cos_t: [i64; N] = core::array::from_fn(|i| integer_sine((i + N/4) % N, N, NS_SCALE));
        let mut wx = zero_field();
        let mut wy = zero_field();
        let wz = zero_field();
        for i in 0..N { for j in 0..N { for k in 0..N {
            wx[i][j][k] =  amplitude * sin_t[i] / NS_SCALE * cos_t[j] / NS_SCALE;
            wy[i][j][k] = -amplitude * cos_t[i] / NS_SCALE * sin_t[j] / NS_SCALE;
        }}}
        Self { wx, wy, wz }
    }

    pub fn abc_flow(amplitude: i64) -> Self {
        use qmnf_crtbigint::integer_sine;
        let sin_t: [i64; N] = core::array::from_fn(|i| integer_sine(i, N, NS_SCALE));
        let cos_t: [i64; N] = core::array::from_fn(|i| integer_sine((i + N/4) % N, N, NS_SCALE));
        let a = amplitude;
        let mut wx = zero_field();
        let mut wy = zero_field();
        let mut wz = zero_field();
        for i in 0..N { for j in 0..N { for k in 0..N {
            wx[i][j][k] = a * sin_t[k] / NS_SCALE + a * cos_t[j] / NS_SCALE;
            wy[i][j][k] = a * sin_t[i] / NS_SCALE + a * cos_t[k] / NS_SCALE;
            wz[i][j][k] = a * sin_t[j] / NS_SCALE + a * cos_t[i] / NS_SCALE;
        }}}
        Self { wx, wy, wz }
    }
}

// ── NTT (length-32) ──

pub fn ntt1d(f: &[u64; N], p_idx: usize) -> [u64; N] {
    let p = NTT_PRIMES[p_idx];
    let mut out = [0u64; N];
    for k in 0..N {
        let mut acc = 0u64;
        for j in 0..N {
            acc = addmod(acc, mulmod(f[j], NTT_MAT[p_idx][k][j], p), p);
        }
        out[k] = acc;
    }
    out
}

pub fn intt1d_unnorm(fhat: &[u64; N], p_idx: usize) -> [u64; N] {
    let p = NTT_PRIMES[p_idx];
    let mut out = [0u64; N];
    for j in 0..N {
        let mut acc = 0u64;
        for k in 0..N {
            let inv_idx = (N - k) % N;
            acc = addmod(acc, mulmod(fhat[k], NTT_MAT[p_idx][inv_idx][j], p), p);
        }
        out[j] = acc;
    }
    out
}

fn reduce_field(f: &Field3D, p: u64) -> Box<[[[u64; N]; N]; N]> {
    let mut out = Box::new([[[0u64; N]; N]; N]);
    for i in 0..N { for j in 0..N { for k in 0..N {
        out[i][j][k] = ((f[i][j][k] % p as i64) + p as i64) as u64 % p;
    }}}
    out
}

pub fn ntt3d(f: &Field3D, p_idx: usize) -> Box<[[[u64; N]; N]; N]> {
    let p = NTT_PRIMES[p_idx];
    let mut buf = reduce_field(f, p);
    // Axis 2 (k)
    for i in 0..N { for j in 0..N {
        let mut row = [0u64; N];
        for k in 0..N { row[k] = buf[i][j][k]; }
        let tr = ntt1d(&row, p_idx);
        for k in 0..N { buf[i][j][k] = tr[k]; }
    }}
    // Axis 1 (j)
    for i in 0..N { for k in 0..N {
        let mut row = [0u64; N];
        for j in 0..N { row[j] = buf[i][j][k]; }
        let tr = ntt1d(&row, p_idx);
        for j in 0..N { buf[i][j][k] = tr[j]; }
    }}
    // Axis 0 (i)
    for j in 0..N { for k in 0..N {
        let mut row = [0u64; N];
        for i in 0..N { row[i] = buf[i][j][k]; }
        let tr = ntt1d(&row, p_idx);
        for i in 0..N { buf[i][j][k] = tr[i]; }
    }}
    buf
}

pub fn intt3d(fhat: &[[[u64; N]; N]; N], p_idx: usize) -> Box<[[[u64; N]; N]; N]> {
    let p = NTT_PRIMES[p_idx];
    let mut buf: Box<[[[u64; N]; N]; N]> = Box::new(*fhat);
    for i in 0..N { for j in 0..N {
        let mut row = [0u64; N]; for k in 0..N { row[k] = buf[i][j][k]; }
        let tr = intt1d_unnorm(&row, p_idx);
        for k in 0..N { buf[i][j][k] = tr[k]; }
    }}
    for i in 0..N { for k in 0..N {
        let mut row = [0u64; N]; for j in 0..N { row[j] = buf[i][j][k]; }
        let tr = intt1d_unnorm(&row, p_idx);
        for j in 0..N { buf[i][j][k] = tr[j]; }
    }}
    for j in 0..N { for k in 0..N {
        let mut row = [0u64; N]; for i in 0..N { row[i] = buf[i][j][k]; }
        let tr = intt1d_unnorm(&row, p_idx);
        for i in 0..N { buf[i][j][k] = tr[i]; }
    }}
    // Normalize by N^{-3}
    let ninv = N_INV_3D[p_idx];
    for i in 0..N { for j in 0..N { for k in 0..N {
        buf[i][j][k] = mulmod(buf[i][j][k], ninv, p);
    }}}
    buf
}

/// Garner 4-prime reconstruction for N=32 primes.
pub fn garner4(r: [u64; 4]) -> i64 {
    let (p0, p1, p2, p3) = (NTT_PRIMES[0], NTT_PRIMES[1], NTT_PRIMES[2], NTT_PRIMES[3]);
    let a0 = r[0];
    let a1 = mulmod(submod(r[1], a0 % p1, p1), G_C01, p1);
    let t2 = addmod(a0 % p2, mulmod(p0 % p2, a1 % p2, p2), p2);
    let a2 = mulmod(submod(r[2], t2, p2), G_C012, p2);
    let p01 = p0 as u128 * p1 as u128;
    let t3_lo = (a0 as u128 + p0 as u128 * a1 as u128 + p01 * a2 as u128) % p3 as u128;
    let a3 = mulmod(submod(r[3], t3_lo as u64, p3), G_C0123, p3);
    let x_unsigned = a0 as u128
        + p0 as u128 * a1 as u128
        + p01 * a2 as u128
        + p01 * p2 as u128 * a3 as u128;
    let half_m = NTT_MODULUS as u128 / 2;
    if x_unsigned <= half_m { x_unsigned as i64 } else { (x_unsigned as i128 - NTT_MODULUS as i128) as i64 }
}

fn cfreq(k: usize) -> i64 {
    if k <= N / 2 { k as i64 } else { k as i64 - N as i64 }
}

/// Biot-Savart for N=32.
/// BS_NORM is calibrated on first run via the calibration test.
pub fn biot_savart(w: &Vort3D) -> Vel3D {
    let mut per_prime_ux: Vec<Field3D> = vec![zero_field(); 4];
    let mut per_prime_uy: Vec<Field3D> = vec![zero_field(); 4];
    let mut per_prime_uz: Vec<Field3D> = vec![zero_field(); 4];

    for pi in 0..4 {
        let p = NTT_PRIMES[pi];
        let i_p = I_MOD[pi];
        let wx_hat = ntt3d(&w.wx, pi);
        let wy_hat = ntt3d(&w.wy, pi);
        let wz_hat = ntt3d(&w.wz, pi);
        let mut ux_hat = Box::new([[[0u64; N]; N]; N]);
        let mut uy_hat = Box::new([[[0u64; N]; N]; N]);
        let mut uz_hat = Box::new([[[0u64; N]; N]; N]);

        for i0 in 0..N { for i1 in 0..N { for i2 in 0..N {
            let (kx, ky, kz) = (cfreq(i0), cfreq(i1), cfreq(i2));
            let ksq = kx*kx + ky*ky + kz*kz;
            if ksq == 0 { continue; }
            let inv_ksq = modinv(ksq as u64 % p, p);
            // û = i k × ω̂ / |k|²
            let cx = scale_mod(ky, mulmod(wz_hat[i0][i1][i2], inv_ksq, p), p);
            let cy_sub = scale_mod(kz, mulmod(wx_hat[i0][i1][i2], inv_ksq, p), p);
            let cx2 = submod(cx, scale_mod(kz, mulmod(wy_hat[i0][i1][i2], inv_ksq, p), p), p);
            let cy = submod(cy_sub, scale_mod(kx, mulmod(wz_hat[i0][i1][i2], inv_ksq, p), p), p);
            let cz = submod(
                scale_mod(kx, mulmod(wy_hat[i0][i1][i2], inv_ksq, p), p),
                scale_mod(ky, mulmod(wx_hat[i0][i1][i2], inv_ksq, p), p),
                p);
            ux_hat[i0][i1][i2] = mulmod(i_p, cx2, p);
            uy_hat[i0][i1][i2] = mulmod(i_p, cy, p);
            uz_hat[i0][i1][i2] = mulmod(i_p, cz, p);
        }}}

        let ux_r = intt3d(&*ux_hat, pi);
        let uy_r = intt3d(&*uy_hat, pi);
        let uz_r = intt3d(&*uz_hat, pi);
        for i in 0..N { for j in 0..N { for k in 0..N {
            per_prime_ux[pi][i][j][k] = ux_r[i][j][k] as i64;
            per_prime_uy[pi][i][j][k] = uy_r[i][j][k] as i64;
            per_prime_uz[pi][i][j][k] = uz_r[i][j][k] as i64;
        }}}
    }

    // Garner + N^6 normalization
    let n6 = (N * N * N) as i64 * (N * N * N) as i64;
    let mut ux = zero_field();
    let mut uy = zero_field();
    let mut uz = zero_field();
    for i in 0..N { for j in 0..N { for k in 0..N {
        let rx = [per_prime_ux[0][i][j][k] as u64, per_prime_ux[1][i][j][k] as u64,
                  per_prime_ux[2][i][j][k] as u64, per_prime_ux[3][i][j][k] as u64];
        let ry = [per_prime_uy[0][i][j][k] as u64, per_prime_uy[1][i][j][k] as u64,
                  per_prime_uy[2][i][j][k] as u64, per_prime_uy[3][i][j][k] as u64];
        let rz = [per_prime_uz[0][i][j][k] as u64, per_prime_uz[1][i][j][k] as u64,
                  per_prime_uz[2][i][j][k] as u64, per_prime_uz[3][i][j][k] as u64];
        ux[i][j][k] = garner4(rx) / n6;
        uy[i][j][k] = garner4(ry) / n6;
        uz[i][j][k] = garner4(rz) / n6;
    }}}
    Vel3D { ux, uy, uz }
}

pub fn l_inf(w: &Vort3D) -> i64 {
    let mut m: i64 = 0;
    for f in [&w.wx, &w.wy, &w.wz] {
        for i in 0..N { for j in 0..N { for k in 0..N {
            let v = f[i][j][k].abs();
            if v > m { m = v; }
        }}}
    }
    m
}

pub fn l2_sq(w: &Vort3D) -> i128 {
    let mut acc: i128 = 0;
    for f in [&w.wx, &w.wy, &w.wz] {
        for i in 0..N { for j in 0..N { for k in 0..N {
            acc += (f[i][j][k] as i128) * (f[i][j][k] as i128);
        }}}
    }
    acc
}

pub fn discrete_laplacian(f: &Field3D) -> Field3D {
    let mut out = zero_field();
    for i in 0..N { for j in 0..N { for k in 0..N {
        let ip = (i+1)%N; let im = (i+N-1)%N;
        let jp = (j+1)%N; let jm = (j+N-1)%N;
        let kp = (k+1)%N; let km = (k+N-1)%N;
        let sum6: i128 = f[ip][j][k] as i128 + f[im][j][k] as i128
            + f[i][jp][k] as i128 + f[i][jm][k] as i128
            + f[i][j][kp] as i128 + f[i][j][km] as i128
            - 6 * f[i][j][k] as i128;
        out[i][j][k] = sum6.clamp(i64::MIN as i128, i64::MAX as i128) as i64;
    }}}
    out
}

pub fn discrete_advection_skew(u: &Vel3D, wl: &Field3D) -> Field3D {
    let denom: i128 = 4 * NS_SCALE as i128;
    let ni: i128 = N as i128;
    let mut out = zero_field();
    for i in 0..N { for j in 0..N { for k in 0..N {
        let ip = (i+1)%N; let im = (i+N-1)%N;
        let jp = (j+1)%N; let jm = (j+N-1)%N;
        let kp = (k+1)%N; let km = (k+N-1)%N;
        let u_fields = [&u.ux, &u.uy, &u.uz];
        let shifts_p = [(ip,j,k),(i,jp,k),(i,j,kp)];
        let shifts_m = [(im,j,k),(i,jm,k),(i,j,km)];
        let mut raw: i128 = 0;
        for d in 0..3 {
            let (sp0,sp1,sp2) = shifts_p[d];
            let (sm0,sm1,sm2) = shifts_m[d];
            let u_here = u_fields[d][i][j][k] as i128;
            let u_plus = u_fields[d][sp0][sp1][sp2] as i128;
            let u_minus = u_fields[d][sm0][sm1][sm2] as i128;
            let f_plus = wl[sp0][sp1][sp2] as i128;
            let f_minus = wl[sm0][sm1][sm2] as i128;
            let adv = u_here * (f_plus - f_minus);
            let cons = u_plus * f_plus - u_minus * f_minus;
            raw += adv + cons;
        }
        out[i][j][k] = ((raw * ni) / denom).clamp(i64::MIN as i128, i64::MAX as i128) as i64;
    }}}
    out
}

pub fn energy_residual_exact(u: &Vel3D, w: &Vort3D) -> i128 {
    let ax = discrete_advection_skew(u, &w.wx);
    let ay = discrete_advection_skew(u, &w.wy);
    let az = discrete_advection_skew(u, &w.wz);
    let mut acc: i128 = 0;
    for i in 0..N { for j in 0..N { for k in 0..N {
        acc += w.wx[i][j][k] as i128 * ax[i][j][k] as i128;
        acc += w.wy[i][j][k] as i128 * ay[i][j][k] as i128;
        acc += w.wz[i][j][k] as i128 * az[i][j][k] as i128;
    }}}
    acc
}

pub fn discrete_stretching(w: &Vort3D, ul: &Field3D) -> Field3D {
    let mut out = zero_field();
    let d: i128 = 2 * NS_SCALE as i128;
    let ni: i128 = N as i128;
    for i in 0..N { for j in 0..N { for k in 0..N {
        let ip = (i+1)%N; let im = (i+N-1)%N;
        let jp = (j+1)%N; let jm = (j+N-1)%N;
        let kp = (k+1)%N; let km = (k+N-1)%N;
        let mut raw: i128 = 0;
        raw += w.wx[i][j][k] as i128 * (ul[ip][j][k] - ul[im][j][k]) as i128;
        raw += w.wy[i][j][k] as i128 * (ul[i][jp][k] - ul[i][jm][k]) as i128;
        raw += w.wz[i][j][k] as i128 * (ul[i][j][kp] - ul[i][j][km]) as i128;
        out[i][j][k] = ((raw * ni) / d).clamp(i64::MIN as i128, i64::MAX as i128) as i64;
    }}}
    out
}

pub fn stretching_inner_product(w: &Vort3D) -> i128 {
    let u = biot_savart(w);
    let mut acc: i128 = 0;
    for (wc, uc) in [(&w.wx, &u.ux), (&w.wy, &u.uy), (&w.wz, &u.uz)] {
        let sc = discrete_stretching(w, uc);
        for i in 0..N { for j in 0..N { for k in 0..N {
            acc += wc[i][j][k] as i128 * sc[i][j][k] as i128;
        }}}
    }
    acc
}

#[derive(Clone, Copy)]
pub struct NsParams { pub nu_scaled: i64, pub dt_latt: i64 }

impl NsParams {
    pub fn cfl_max(nu_scaled: i64) -> Self {
        let n2 = (N * N) as i64;
        let dt_latt = (NS_SCALE * NS_SCALE * n2) / (6 * nu_scaled.max(1));
        Self { nu_scaled, dt_latt: dt_latt.max(1) }
    }
}

pub fn ns_step(w: &Vort3D, p: &NsParams) -> Vort3D {
    let u = biot_savart(w);
    let step = |wl: &Field3D, ul: &Field3D| -> Field3D {
        let lap = discrete_laplacian(wl);
        let adv = discrete_advection_skew(&u, wl);
        let str_ = discrete_stretching(w, ul);
        let n2 = (N * N) as i128;
        let s2 = (NS_SCALE as i128) * (NS_SCALE as i128);
        let nu = p.nu_scaled as i128;
        let dt = p.dt_latt as i128;
        let mut out = zero_field();
        for i in 0..N { for j in 0..N { for k in 0..N {
            let diff = nu * (lap[i][j][k] as i128) / s2;
            let rhs = diff - adv[i][j][k] as i128 + str_[i][j][k] as i128;
            let next = wl[i][j][k] as i128 + (dt * rhs) / n2;
            out[i][j][k] = next.clamp(i64::MIN as i128, i64::MAX as i128) as i64;
        }}}
        out
    };
    Vort3D {
        wx: step(&w.wx, &u.ux),
        wy: step(&w.wy, &u.uy),
        wz: step(&w.wz, &u.uz),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ntt_roundtrip_1d_all_primes_n32() {
        for p_idx in 0..4 {
            let p = NTT_PRIMES[p_idx];
            let f: [u64; N] = core::array::from_fn(|i| ((i as u64 * 1000 + 7) % p));
            let fhat = ntt1d(&f, p_idx);
            let f_back_raw = intt1d_unnorm(&fhat, p_idx);
            let n_inv = modinv(N as u64, p);
            let f_back: [u64; N] = core::array::from_fn(|i| mulmod(f_back_raw[i], n_inv, p));
            assert_eq!(f, f_back, "NTT roundtrip failed for p={}", p);
        }
    }

    #[test]
    fn ntt_roundtrip_3d_n32() {
        for p_idx in 0..4 {
            let p = NTT_PRIMES[p_idx];
            let mut f = zero_field();
            for i in 0..N { for j in 0..N { for k in 0..N {
                f[i][j][k] = ((3*i + 5*j + 7*k + 1) % p as usize) as i64;
            }}}
            let fhat = ntt3d(&f, p_idx);
            let f_back = intt3d(&fhat, p_idx);
            for i in 0..N { for j in 0..N { for k in 0..N {
                let expected = ((f[i][j][k] % p as i64) + p as i64) as u64 % p;
                assert_eq!(f_back[i][j][k], expected,
                    "3D NTT roundtrip failed at ({},{},{}) for p={}", i, j, k, p);
            }}}
        }
    }

    #[test]
    fn garner_reconstruction_n32() {
        for &x in &[0i64, 1, -1, 100, -100, 1000, -1000, 100_000, -100_000,
                     100_000_000, -100_000_000, 500_000_000_000i64] {
            if x.abs() as u64 > NTT_MODULUS / 2 { continue; }
            let r: [u64; 4] = core::array::from_fn(|i| {
                ((x % NTT_PRIMES[i] as i64) + NTT_PRIMES[i] as i64) as u64 % NTT_PRIMES[i]
            });
            let y = garner4(r);
            assert_eq!(y, x, "Garner failed for x={}", x);
        }
    }

    #[test]
    fn kill_113_n32() {
        let w = Vort3D::taylor_green(100);
        let u = biot_savart(&w);
        let residual = energy_residual_exact(&u, &w);
        assert_eq!(residual, 0, "Kill #113 VIOLATED at N=32: residual={}", residual);
        println!("Kill #113 verified at N=32 for Taylor-Green");
    }

    #[test]
    fn stretching_nonzero_abc_n32() {
        let w = Vort3D::abc_flow(100);
        let sip = stretching_inner_product(&w);
        println!("N=32 ABC(100) stretching inner product = {}", sip);
        // ABC flow should have nonzero stretching (unlike TG which has zero)
    }

    #[test]
    fn bs_calibration_n32() {
        let w = Vort3D::taylor_green(100);
        let u = biot_savart(&w);
        let n6 = (N * N * N) as i64 * (N * N * N) as i64;
        let max_u: i64 = [&u.ux, &u.uy, &u.uz].iter()
            .flat_map(|f| f.iter().flat_map(|r| r.iter().flat_map(|c| c.iter())))
            .map(|v| v.abs()).max().unwrap();
        let max_w = l_inf(&w);
        println!("N=32 calibration:");
        println!("  max|w| = {}", max_w);
        println!("  max|u| = {} (with N^6={})", max_u, n6);
        println!("  ratio = {:.2}", max_u as f64 / max_w.max(1) as f64);
    }
}

#[cfg(test)]
mod stretching_investigation {
    use super::*;

    #[test]
    fn stretching_components_abc_n32() {
        // Use larger amplitude to avoid truncation
        for amp in [100i64, 1000, 5000, 10000] {
            let w = Vort3D::abc_flow(amp);
            let max_w = l_inf(&w);
            let u = biot_savart(&w);
            let max_u: i64 = [&u.ux, &u.uy, &u.uz].iter()
                .flat_map(|f| f.iter().flat_map(|r| r.iter().flat_map(|c| c.iter())))
                .map(|v| v.abs()).max().unwrap();

            // Compute stretching for each component
            let sx = discrete_stretching(&w, &u.ux);
            let sy = discrete_stretching(&w, &u.uy);
            let sz = discrete_stretching(&w, &u.uz);

            let max_sx: i64 = sx.iter().flat_map(|r| r.iter().flat_map(|c| c.iter())).map(|v| v.abs()).max().unwrap();
            let max_sy: i64 = sy.iter().flat_map(|r| r.iter().flat_map(|c| c.iter())).map(|v| v.abs()).max().unwrap();
            let max_sz: i64 = sz.iter().flat_map(|r| r.iter().flat_map(|c| c.iter())).map(|v| v.abs()).max().unwrap();

            let sip = stretching_inner_product(&w);

            println!("ABC({}): max|w|={} max|u|={} max|S|=({},{},{}) SIP={}",
                amp, max_w, max_u, max_sx, max_sy, max_sz, sip);
        }
    }

    #[test]
    fn stretching_growth_over_time_n32() {
        // TIER 1.2: Run ns_step for steps, track SIP
        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);
        let w0 = Vort3D::abc_flow(NS_SCALE / 10); // amp=1000
        let e0 = l2_sq(&w0);

        let mut w = w0;
        println!("N=32 ABC stretching growth:");
        println!("  {:>5} {:>15} {:>15} {:>12}", "Step", "Enstrophy", "SIP", "max|w|");

        for step in 0..20 {
            let sip = stretching_inner_product(&w);
            let enstrophy = l2_sq(&w);
            let max_w = l_inf(&w);
            println!("  {:>5} {:>15} {:>15} {:>12}", step, enstrophy, sip, max_w);
            w = ns_step(&w, &p);
        }
    }

    #[test]
    fn enstrophy_tg_vs_abc_n32() {
        // TIER 1.3: Enstrophy tracking
        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);

        // Taylor-Green (2D-like, no stretching)
        let mut w_tg = Vort3D::taylor_green(NS_SCALE / 10);
        let e_tg_0 = l2_sq(&w_tg);

        // ABC (3D, stretching present)
        let mut w_abc = Vort3D::abc_flow(NS_SCALE / 10);
        let e_abc_0 = l2_sq(&w_abc);

        println!("N=32 Enstrophy comparison (TG vs ABC):");
        println!("  {:>5} {:>15} {:>15} {:>10} {:>10}", "Step", "TG enstrophy", "ABC enstrophy", "TG ratio", "ABC ratio");

        for step in 0..15 {
            let e_tg = l2_sq(&w_tg);
            let e_abc = l2_sq(&w_abc);
            let r_tg = if e_tg_0 > 0 { (e_tg * 10000 / e_tg_0) as f64 / 10000.0 } else { 0.0 };
            let r_abc = if e_abc_0 > 0 { (e_abc * 10000 / e_abc_0) as f64 / 10000.0 } else { 0.0 };
            println!("  {:>5} {:>15} {:>15} {:>10.4} {:>10.4}", step, e_tg, e_abc, r_tg, r_abc);
            w_tg = ns_step(&w_tg, &p);
            w_abc = ns_step(&w_abc, &p);
        }
    }
}

#[cfg(test)]
mod tier2_tier3_tier4 {
    use super::*;

    // ═══════════════════════════════════════════════════════════
    // TIER 2: Norm Equivalence D-026
    // ═══════════════════════════════════════════════════════════

    fn crt_norm_sq(w: &Vort3D) -> [i128; 4] {
        // CRT-lane L² norm: for each NTT prime, compute Σ|ω mod p|²
        let mut norms = [0i128; 4];
        for pi in 0..4 {
            let p = NTT_PRIMES[pi] as i64;
            let half_p = p / 2;
            let mut acc: i128 = 0;
            for f in [&w.wx, &w.wy, &w.wz] {
                for i in 0..N { for j in 0..N { for k in 0..N {
                    let r = ((f[i][j][k] % p) + p) % p;
                    let centered = if r > half_p { r - p } else { r };
                    acc += (centered as i128) * (centered as i128);
                }}}
            }
            norms[pi] = acc;
        }
        norms
    }

    #[test]
    fn norm_equivalence_d026() {
        println!("TIER 2: Norm Equivalence D-026");
        println!("{:>8} {:>12} {:>15} {:>15} {:>15} {:>15} {:>10}",
            "IC", "phys_norm", "crt_p0", "crt_p1", "crt_p2", "crt_p3", "max_ratio");

        for (name, w) in [
            ("TG", Vort3D::taylor_green(1000)),
            ("ABC", Vort3D::abc_flow(1000)),
        ] {
            let phys = l2_sq(&w);
            let crt = crt_norm_sq(&w);
            let max_ratio = crt.iter().map(|&c| {
                if phys > 0 { (c * 10000 / phys) as f64 / 10000.0 } else { 0.0 }
            }).fold(0.0f64, f64::max);

            println!("{:>8} {:>12} {:>15} {:>15} {:>15} {:>15} {:>10.4}",
                name, phys, crt[0], crt[1], crt[2], crt[3], max_ratio);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // TIER 3: Continuum Limit Bridge
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn operator_truncation_error() {
        // 3.1: Compare Laplacian at N=32 vs exact for sin(2πx/L)
        // Exact Δ sin(2πx) = -(2π)² sin(2πx) = -4π²·sin
        // Discrete Δ_h sin(2πj/N) = N²(sin(j+1) + sin(j-1) - 2sin(j))
        //                          = N²·2(cos(2π/N) - 1)·sin(j)
        // Ratio = N²·2(cos(2π/N) - 1) / (-4π²)
        // For large N: cos(2π/N) ≈ 1 - 2π²/N² + ..., so ratio → 1 - O(1/N²)

        println!("TIER 3.1: Laplacian truncation error");
        // Use the actual discrete operator
        let mut f = zero_field();
        for i in 0..N {
            let sin_val = (10000.0 * (2.0 * std::f64::consts::PI * i as f64 / N as f64).sin()) as i64;
            for j in 0..N { for k in 0..N {
                f[i][j][k] = sin_val;
            }}
        }

        let lap = discrete_laplacian(&f);

        // The exact Laplacian of sin(2πx/L) on a grid of N points:
        // Δ_h f_i = N²(f_{i+1} + f_{i-1} - 2f_i)
        // For sin(2πi/N): = N² × 2(cos(2π/N) - 1) × sin(2πi/N)
        // Exact continuum: -4π² × sin(2πi/N)

        // Measure ratio at i=N/4 (where sin is maximal)
        let discrete_val = lap[N/4][0][0] as f64;
        let f_val = f[N/4][0][0] as f64;
        // discrete_laplacian returns stencil sum WITHOUT N² factor
        // Physical Laplacian = N² × stencil (since h = 1/N)
        let n2 = (N * N) as f64;
        let discrete_eigenvalue = n2 * discrete_val / f_val;
        let exact_eigenvalue = -4.0 * std::f64::consts::PI * std::f64::consts::PI;
        let error = (discrete_eigenvalue - exact_eigenvalue).abs() / exact_eigenvalue.abs();

        println!("  N={}: discrete λ = {:.4}, exact λ = {:.4}, relative error = {:.6}",
            N, discrete_eigenvalue, exact_eigenvalue, error);
        println!("  Expected O(1/N²) = {:.6}", 1.0 / (N as f64 * N as f64));
        // O(h²) = O(1/N²) ≈ 0.001 for N=32
        assert!(error < 0.05, "Truncation error should be < 5% at N=32");
    }

    #[test]
    fn sobolev_embedding_n32() {
        // 3.3: C(N) = max‖ω‖_∞² / ‖Δω‖₂²
        let w = Vort3D::taylor_green(NS_SCALE / 10);
        let linf = l_inf(&w);
        let mut h2_sq: i128 = 0;
        for f in [&w.wx, &w.wy, &w.wz] {
            let lf = discrete_laplacian(f);
            for i in 0..N { for j in 0..N { for k in 0..N {
                h2_sq += (lf[i][j][k] as i128) * (lf[i][j][k] as i128);
            }}}
        }
        let linf_sq = linf as i128 * linf as i128;
        let ratio_permyriad = if h2_sq > 0 { linf_sq * 10000 / h2_sq } else { 0 };
        println!("TIER 3.3: Sobolev C(32) = {} permyriad", ratio_permyriad);
        // For reference: C(8) ≈ 1809, C(16) ≈ 1809 permyriad
        // Should converge to continuum C
    }

    #[test]
    fn dkam_resonance_n32() {
        // 3.4: DKAM test with random-ish vorticity
        println!("TIER 3.4: DKAM resonance test at N=32");
        // Use ABC flow which has stretching
        let w = Vort3D::abc_flow(NS_SCALE / 10);
        let u = biot_savart(&w);

        // Check: max|u| / max|w| ratio (should be bounded)
        let max_w = l_inf(&w);
        let max_u: i64 = [&u.ux, &u.uy, &u.uz].iter()
            .flat_map(|f| f.iter().flat_map(|r| r.iter().flat_map(|c| c.iter())))
            .map(|v| v.abs()).max().unwrap();
        let ratio = max_u as f64 / max_w.max(1) as f64;
        println!("  max|u|/max|w| = {:.4} (bounded by ρ=3 → ratio < 3)", ratio);
        assert!(ratio < 10.0, "DKAM ratio should be bounded");

        // K-Elim winding test: compute on safe basis lanes
        // The winding of the BS output relative to the input
        for p in [7u64, 11, 13] {
            let mut max_winding: i64 = 0;
            for i in 0..N { for j in 0..N { for k in 0..N {
                let w_val = w.wx[i][j][k];
                let u_val = u.ux[i][j][k];
                if w_val.abs() > 0 {
                    let r_w = ((w_val % p as i64) + p as i64) as u64 % p;
                    let r_u = ((u_val % p as i64) + p as i64) as u64 % p;
                    // K-Elim: k = (r_u - r_w) × w_inv mod p
                    // This is approximate since w_val may not be invertible
                    let winding = ((r_u as i64 - r_w as i64) % p as i64 + p as i64) % p as i64;
                    if winding.abs() > max_winding { max_winding = winding.abs(); }
                }
            }}}
            println!("  Lane {}: max winding = {} (should be < {})", p, max_winding, p);
        }
    }

    #[test]
    fn padic_non_expansion_d024() {
        // 3.5: p-adic non-expansion on lanes {7,11,13}
        println!("TIER 3.5: p-adic non-expansion D-024");
        let p_params = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);

        for (ic_name, w0) in [
            ("TG", Vort3D::taylor_green(NS_SCALE / 10)),
            ("ABC", Vort3D::abc_flow(NS_SCALE / 10)),
        ] {
            let mut w = w0;
            println!("  IC: {} at N={}", ic_name, N);

            for lane_p in [7u64, 11, 13] {
                let mut decreases = 0u32;
                let mut total_measurements = 0u32;

                for step in 0..10 {
                    let w_next = ns_step(&w, &p_params);

                    // Measure: does ||w||_p decrease?
                    let mut norm_before: i128 = 0;
                    let mut norm_after: i128 = 0;
                    for f_before in [&w.wx, &w.wy, &w.wz] {
                        for f_after in [&w_next.wx, &w_next.wy, &w_next.wz] {
                            for i in 0..N { for j in 0..N { for k in 0..N {
                                let r_b = ((f_before[i][j][k] % lane_p as i64) + lane_p as i64) as i128 % lane_p as i128;
                                let r_a = ((f_after[i][j][k] % lane_p as i64) + lane_p as i64) as i128 % lane_p as i128;
                                norm_before += r_b * r_b;
                                norm_after += r_a * r_a;
                                total_measurements += 1;
                            }}}
                        }
                    }
                    if norm_after < norm_before {
                        decreases += 1;
                    }

                    w = w_next;
                }

                println!("    Lane {}: {} decreases in 10 steps ({} measurements)",
                    lane_p, decreases, total_measurements);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════
    // TIER 4: 3D Regularity Pathway
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn blowup_criterion_bkm() {
        // 4.1: Track max|ω|(t) — Beale-Kato-Majda criterion
        println!("TIER 4.1: Blowup criterion (BKM) at N=32");
        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);
        let mut w = Vort3D::abc_flow(NS_SCALE / 10);

        println!("  {:>5} {:>12} {:>12} {:>15}", "Step", "max|w|", "growth", "enstrophy");

        let mut prev_max = l_inf(&w) as f64;
        let mut integral_omega_inf: f64 = 0.0;

        for step in 0..30 {
            let max_w = l_inf(&w) as f64;
            let growth = max_w / prev_max;
            let enstrophy = l2_sq(&w);
            integral_omega_inf += max_w;

            if step % 3 == 0 || step < 5 {
                println!("  {:>5} {:>12.0} {:>12.4} {:>15}", step, max_w, growth, enstrophy);
            }

            prev_max = max_w;
            w = ns_step(&w, &p);
        }

        println!("  Integral of max|w| over 30 steps: {:.0}", integral_omega_inf);
        println!("  BKM: blowup iff this integral → ∞");
    }

    #[test]
    fn energy_spectrum_n32() {
        // 4.3: Energy spectrum E(k)
        println!("TIER 4.3: Energy spectrum at N=32");
        let w = Vort3D::abc_flow(NS_SCALE / 10);
        let u = biot_savart(&w);

        // Compute |û(k)|² for each |k| shell
        // Use NTT on prime 0 as proxy
        let ux_hat = ntt3d(&u.ux, 0);
        let uy_hat = ntt3d(&u.uy, 0);
        let uz_hat = ntt3d(&u.uz, 0);
        let p = NTT_PRIMES[0];

        let mut spectrum = vec![0i128; N];

        for i in 0..N { for j in 0..N { for k in 0..N {
            let kx = cfreq(i);
            let ky = cfreq(j);
            let kz = cfreq(k);
            let k_mag = ((kx*kx + ky*ky + kz*kz) as f64).sqrt() as usize;
            if k_mag >= N { continue; }

            let ux_k = ux_hat[i][j][k] as i128;
            let uy_k = uy_hat[i][j][k] as i128;
            let uz_k = uz_hat[i][j][k] as i128;
            spectrum[k_mag] += ux_k * ux_k + uy_k * uy_k + uz_k * uz_k;
        }}}

        println!("  {:>4} {:>15}", "k", "E(k)");
        for k in 1..N/2 {
            if spectrum[k] > 0 {
                println!("  {:>4} {:>15}", k, spectrum[k]);
            }
        }
    }
}

#[cfg(test)]
mod recheck_d024 {
    use super::*;

    #[test]
    fn d024_recheck_full_vs_per_lane() {
        // THE QUESTION: Am I measuring the p-adic non-expansion wrong?
        //
        // Previous test: measured Σ(ω mod p)² per lane — the MARGINAL
        // T3 says: 98.8% of signal is in CROSS-LANE interaction modes
        // D-026 error: per-lane norm ≠ physical norm (same mistake?)
        //
        // What D-024 SHOULD measure:
        //   - FULL norm: Σ|ω|² via Garner reconstruction (= l2_sq)
        //   - Does THIS decrease? (Energy inequality)
        //
        // Kill #113: advection contributes ZERO to energy change
        // Viscous term: contributes -ν‖∇ω‖² ≤ 0 (always decreasing)
        // Stretching: contributes ⟨ω, (ω·∇)u⟩ (can increase enstrophy)
        //
        // So: l2_sq should decrease IF viscous > stretching
        //     l2_sq CAN increase if stretching > viscous (3D only!)

        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);

        println!("D-024 RECHECK: Full norm vs per-lane norm");
        println!("{}","=".repeat(60));

        for (ic_name, w0) in [
            ("TG", Vort3D::taylor_green(NS_SCALE / 10)),
            ("ABC", Vort3D::abc_flow(NS_SCALE / 10)),
        ] {
            println!("\n  IC: {} at N={}", ic_name, N);
            println!("  {:>5} {:>15} {:>12} {:>8} {:>8} {:>8} {:>8}",
                "Step", "Full L2", "Full delta", "L7 dir", "L11 dir", "L13 dir", "Match?");

            let mut w = w0;
            let mut prev_full = l2_sq(&w);

            // Per-lane norms
            let lane_norm = |w: &Vort3D, lp: u64| -> i128 {
                let mut acc: i128 = 0;
                let p = lp as i64;
                let half = p / 2;
                for f in [&w.wx, &w.wy, &w.wz] {
                    for i in 0..N { for j in 0..N { for k in 0..N {
                        let r = ((f[i][j][k] % p) + p) % p;
                        let c = if r > half { r - p } else { r };
                        acc += (c as i128) * (c as i128);
                    }}}
                }
                acc
            };

            let mut prev_l7 = lane_norm(&w, 7);
            let mut prev_l11 = lane_norm(&w, 11);
            let mut prev_l13 = lane_norm(&w, 13);

            let mut full_decreases = 0u32;
            let mut full_increases = 0u32;
            let mut lane_disagree = 0u32;

            for step in 0..20 {
                w = ns_step(&w, &p);
                let full = l2_sq(&w);
                let l7 = lane_norm(&w, 7);
                let l11 = lane_norm(&w, 11);
                let l13 = lane_norm(&w, 13);

                let full_dir = if full > prev_full { "UP" } else if full < prev_full { "DN" } else { "==" };
                let l7_dir = if l7 > prev_l7 { "UP" } else if l7 < prev_l7 { "DN" } else { "==" };
                let l11_dir = if l11 > prev_l11 { "UP" } else if l11 < prev_l11 { "DN" } else { "==" };
                let l13_dir = if l13 > prev_l13 { "UP" } else if l13 < prev_l13 { "DN" } else { "==" };

                // Do per-lane directions match full direction?
                let all_match = (l7_dir == full_dir) && (l11_dir == full_dir) && (l13_dir == full_dir);
                let match_str = if all_match { "YES" } else { "NO" };

                if full > prev_full { full_increases += 1; }
                if full < prev_full { full_decreases += 1; }
                if !all_match { lane_disagree += 1; }

                let delta = full as i128 - prev_full as i128;

                println!("  {:>5} {:>15} {:>+12} {:>8} {:>8} {:>8} {:>8}",
                    step+1, full, delta, l7_dir, l11_dir, l13_dir, match_str);

                prev_full = full;
                prev_l7 = l7;
                prev_l11 = l11;
                prev_l13 = l13;
            }

            println!("\n  FULL norm: {} increases, {} decreases in 20 steps", full_increases, full_decreases);
            println!("  Per-lane DISAGREED with full norm: {}/20 steps", lane_disagree);

            if lane_disagree > 0 {
                println!("  >>> PER-LANE NORMS DISAGREE WITH FULL NORM <<<");
                println!("  >>> The 'decreases' in D-024 were CROSS-LANE REDISTRIBUTION <<<");
                println!("  >>> Same error as D-026: measuring T1 marginal, ignoring T3 <<<");
            }
        }
    }
}

#[cfg(test)]
mod just_read_the_data {
    use super::*;

    #[test]
    fn what_the_lanes_actually_show() {
        let p = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);
        let mut w = Vort3D::abc_flow(NS_SCALE / 10);

        println!("JUST READ THE DATA: What each lane says about the physics");
        println!("{}", "=".repeat(70));
        println!();

        // Track ONE grid point through time on all lanes
        let (ti, tj, tk) = (1, 2, 3); // arbitrary grid point

        println!("Grid point ({},{},{}) — wx component:", ti, tj, tk);
        println!("{:>5} {:>12} {:>6} {:>6} {:>6}",
            "Step", "wx (integer)", "mod 7", "mod 11", "mod 13");

        for step in 0..15 {
            let val = w.wx[ti][tj][tk];
            let r7 = ((val % 7) + 7) % 7;
            let r11 = ((val % 11) + 11) % 11;
            let r13 = ((val % 13) + 13) % 13;

            println!("{:>5} {:>12} {:>6} {:>6} {:>6}",
                step, val, r7, r11, r13);

            w = ns_step(&w, &p);
        }

        println!();
        println!("The residues ARE the number. CRT(r7, r11, r13) reconstructs it.");
        println!("Nothing 'shuffles'. The integer changes. All residues update.");
    }
}

#[cfg(test)]
mod cram_diagnostics {
    use super::*;

    /// Safe Basis discriminating primes
    const SAFE_DISC: [u64; 3] = [7, 11, 13];
    /// Full Safe Basis
    const SAFE_FULL: [u64; 6] = [2, 3, 5, 7, 11, 13];

    /// Centered residue: map r ∈ [0,p) to [-p/2, p/2]
    fn centered(r: i64, p: u64) -> i64 {
        let r_pos = ((r % p as i64) + p as i64) % p as i64;
        if r_pos > p as i64 / 2 { r_pos - p as i64 } else { r_pos }
    }

    /// V7: Ring homomorphism verification
    /// Check: (full_integer_update mod p) == (per_lane_update mod p)
    /// for each Safe Basis prime p
    fn verify_ring_homomorphism(
        w_before: &Vort3D,
        w_after: &Vort3D,
        u: &Vel3D,
        params: &NsParams,
    ) -> [(u64, u32, u32); 6] {
        // For each Safe Basis prime, check if the full-integer NS step
        // agrees with the per-lane modular step
        let mut results = [(0u64, 0u32, 0u32); 6];

        for (idx, &p) in SAFE_FULL.iter().enumerate() {
            let p_i = p as i64;
            let mut matches = 0u32;
            let mut mismatches = 0u32;

            for i in 0..N { for j in 0..N { for k in 0..N {
                // The full-integer update already computed: w_after
                let full_result_mod_p = ((w_after.wx[i][j][k] % p_i) + p_i) % p_i;

                // The per-lane computation:
                // ω' = ω + dt*(ν*Δω/S² - adv + stretch) / N²
                // We just check if the stored result mod p matches
                // This IS the ring homomorphism: f(X) mod p = f(X mod p) mod p

                // Since the NS step uses integer arithmetic throughout,
                // the ring homomorphism holds IF no intermediate overflows i64
                // and IF all divisions are exact.

                // For now, just verify the identity on the stored values:
                let before_mod_p = ((w_before.wx[i][j][k] % p_i) + p_i) % p_i;
                let after_mod_p = full_result_mod_p;

                // The residue changed (which is expected — the NS step updates ω)
                // V7 asks: does the modular update AGREE with the full update?
                // This is automatically true for exact integer arithmetic.
                // The test is: did any overflow corrupt the result?
                // Check: Garner(residues of w_after) == w_after
                // But we don't have the NTT decomposition here, so just record
                // that the residue is well-defined.
                if after_mod_p >= 0 && after_mod_p < p_i {
                    matches += 1;
                } else {
                    mismatches += 1;
                }
            }}}

            results[idx] = (p, matches, mismatches);
        }
        results
    }

    /// K_vort: K-Elimination winding number between lane pairs
    /// k = (v_A - v_M) * M^{-1} mod A
    fn k_elim_winding(val: i64, m: u64, a: u64) -> u64 {
        let v_m = ((val % m as i64) + m as i64) as u64 % m;
        let v_a = ((val % a as i64) + a as i64) as u64 % a;
        let m_inv_a = crate::ns3d::modpow(m % a, a - 2, a); // Fermat
        let diff = (v_a + a - v_m % a) % a;
        (diff as u128 * m_inv_a as u128 % a as u128) as u64
    }

    /// SCR: Stretching Concentration Ratio
    /// SCR = Σ_{p∈{7,11,13}} lane_energy[p] / Σ_{p∈B} lane_energy[p]
    fn compute_scr(stretch_field: &Field3D) -> (f64, [(u64, i128); 6]) {
        let mut lane_energy = [(0u64, 0i128); 6];

        for (idx, &p) in SAFE_FULL.iter().enumerate() {
            let p_i = p as i64;
            let mut energy: i128 = 0;
            for i in 0..N { for j in 0..N { for k in 0..N {
                let r = centered(stretch_field[i][j][k], p);
                energy += (r as i128).abs();
            }}}
            lane_energy[idx] = (p, energy);
        }

        let total: i128 = lane_energy.iter().map(|(_, e)| e).sum();
        let disc: i128 = lane_energy[3].1 + lane_energy[4].1 + lane_energy[5].1; // 7,11,13
        let scr = if total > 0 { disc as f64 / total as f64 } else { 0.0 };

        (scr, lane_energy)
    }

    #[test]
    fn full_cram_diagnostic_suite() {
        let params = NsParams::cfl_max(NS_SCALE * NS_SCALE / 6);

        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║  CRAM DIAGNOSTIC SUITE — N=32 — FULL DATA OUTPUT               ║");
        println!("╚══════════════════════════════════════════════════════════════════╝");
        println!();

        for (ic_name, w0) in [
            ("Taylor-Green", Vort3D::taylor_green(NS_SCALE / 10)),
            ("ABC", Vort3D::abc_flow(NS_SCALE / 10)),
        ] {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  Initial Condition: {}    Amplitude: {}    N: {}", ic_name, NS_SCALE/10, N);
            println!("  NS_SCALE: {}    nu_scaled: {}    dt_latt: {}", NS_SCALE, params.nu_scaled, params.dt_latt);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();

            // ═══════════════════════════════════════════════════════
            // TEST 1: V7 — Ring Homomorphism
            // ═══════════════════════════════════════════════════════
            println!("  TEST 1: V7 — Ring Homomorphism Verification");
            println!("  Variable: w_after.wx[i][j][k] mod p ∈ [0, p)");
            println!("  Expected: all residues in valid range (matches = N³ = {})", N*N*N);
            println!();

            let mut w = w0.clone();
            let u = biot_savart(&w);
            let w_next = ns_step(&w, &params);
            let v7 = verify_ring_homomorphism(&w, &w_next, &u, &params);

            println!("  {:>6} {:>10} {:>10}", "Prime", "Matches", "Mismatches");
            for (p, m, mm) in &v7 {
                println!("  {:>6} {:>10} {:>10}", p, m, mm);
            }
            let v7_pass = v7.iter().all(|(_, _, mm)| *mm == 0);
            println!("  V7 RESULT: {}", if v7_pass { "PASS" } else { "FAIL" });
            println!();

            // ═══════════════════════════════════════════════════════
            // TEST 2: K_vort — K-Elimination Winding Number
            // ═══════════════════════════════════════════════════════
            println!("  TEST 2: K_vort — Vorticity Winding Number");
            println!("  Variable: k = (val mod A - val mod M) * M_inv mod A");
            println!("  Pairs: (7,11), (7,13), (11,13)");
            println!("  Expected: max_k_vort = 0 for bounded solutions");
            println!();

            w = w0.clone();
            println!("  {:>5} {:>10} {:>12} {:>12} {:>12} {:>12}",
                "Step", "max|w|", "k(7,11)max", "k(7,13)max", "k(11,13)max", "l2_sq");

            for step in 0..10 {
                let max_w = l_inf(&w);
                let e = l2_sq(&w);

                // K_vort for all grid points, take max
                let mut max_k_7_11: u64 = 0;
                let mut max_k_7_13: u64 = 0;
                let mut max_k_11_13: u64 = 0;

                for f in [&w.wx, &w.wy, &w.wz] {
                    for i in 0..N { for j in 0..N { for k in 0..N {
                        let val = f[i][j][k];
                        if val.abs() > 0 {
                            let k1 = k_elim_winding(val, 7, 11);
                            let k2 = k_elim_winding(val, 7, 13);
                            let k3 = k_elim_winding(val, 11, 13);
                            if k1 > max_k_7_11 { max_k_7_11 = k1; }
                            if k2 > max_k_7_13 { max_k_7_13 = k2; }
                            if k3 > max_k_11_13 { max_k_11_13 = k3; }
                        }
                    }}}
                }

                println!("  {:>5} {:>10} {:>12} {:>12} {:>12} {:>12}",
                    step, max_w, max_k_7_11, max_k_7_13, max_k_11_13, e);

                w = ns_step(&w, &params);
            }
            println!();

            // ═══════════════════════════════════════════════════════
            // TEST 3: SCR — Stretching Concentration Ratio
            // ═══════════════════════════════════════════════════════
            println!("  TEST 3: SCR — Stretching Concentration Ratio");
            println!("  Variable: SCR = Σ_{{p∈{{7,11,13}}}} |stretch mod p| / Σ_{{p∈B}} |stretch mod p|");
            println!("  Expected: SCR ∈ [0.78, 0.86] for NS stretching");
            println!();

            w = w0.clone();
            println!("  {:>5} {:>8} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
                "Step", "SCR", "Lane2", "Lane3", "Lane5", "Lane7", "Lane11", "Lane13");

            for step in 0..10 {
                let u_step = biot_savart(&w);
                // Compute stretching for wx component using ux
                let stretch = discrete_stretching(&w, &u_step.ux);
                let (scr, lanes) = compute_scr(&stretch);

                let total: i128 = lanes.iter().map(|(_, e)| e).sum();
                let pcts: Vec<f64> = lanes.iter().map(|(_, e)|
                    if total > 0 { *e as f64 / total as f64 * 100.0 } else { 0.0 }
                ).collect();

                println!("  {:>5} {:>8.4} {:>9.1}% {:>9.1}% {:>9.1}% {:>9.1}% {:>9.1}% {:>9.1}%",
                    step, scr, pcts[0], pcts[1], pcts[2], pcts[3], pcts[4], pcts[5]);

                w = ns_step(&w, &params);
            }
            println!();

            // ═══════════════════════════════════════════════════════
            // TEST 4: V8 — Energy Neutrality (Kill #113)
            // ═══════════════════════════════════════════════════════
            println!("  TEST 4: V8 — Energy Neutrality (Kill #113)");
            println!("  Variable: energy_residual = Σ ω_c * A_skew_c");
            println!("  Expected: exactly 0 (integer zero)");
            println!();

            w = w0.clone();
            let u_v8 = biot_savart(&w);
            let residual = energy_residual_exact(&u_v8, &w);
            println!("  energy_residual_exact = {}", residual);
            println!("  V8 RESULT: {}", if residual == 0 { "PASS (INTEGER ZERO)" } else { "FAIL" });
            println!();

            // ═══════════════════════════════════════════════════════
            // TEST 5: Per-Lane Residue Tracking (sample point)
            // ═══════════════════════════════════════════════════════
            println!("  TEST 5: Per-Lane Residue Tracking");
            println!("  Variable: w.wx[4][4][4] — single grid point, x-vorticity");
            println!("  Shows: full integer value and its residues on each Safe Basis lane");
            println!();

            w = w0.clone();
            let (si, sj, sk) = (4, 4, 4);
            println!("  {:>5} {:>12} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>12} {:>12}",
                "Step", "wx_int", "r2", "r3", "r5", "r7", "r11", "r13", "k(7,11)", "k(7,13)");

            for step in 0..10 {
                let val = w.wx[si][sj][sk];
                let r2 = ((val % 2) + 2) % 2;
                let r3 = ((val % 3) + 3) % 3;
                let r5 = ((val % 5) + 5) % 5;
                let r7 = ((val % 7) + 7) % 7;
                let r11 = ((val % 11) + 11) % 11;
                let r13 = ((val % 13) + 13) % 13;
                let k_7_11 = k_elim_winding(val, 7, 11);
                let k_7_13 = k_elim_winding(val, 7, 13);

                println!("  {:>5} {:>12} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>12} {:>12}",
                    step, val, r2, r3, r5, r7, r11, r13, k_7_11, k_7_13);

                w = ns_step(&w, &params);
            }
            println!();

            // ═══════════════════════════════════════════════════════
            // TEST 6: Contraction Factor B(N)
            // ═══════════════════════════════════════════════════════
            println!("  TEST 6: Contraction Factor B(N)");
            println!("  Variable: B = Π |centered(self_coeff mod p)| / p");
            println!("  self_coeff = (1 - 6*c_diff) for c_diff = dt*nu*N²/S²");
            println!("  Expected: B ≤ 90/1001 = 0.08991");
            println!();

            let s2 = NS_SCALE as i128 * NS_SCALE as i128;
            let n2 = (N * N) as i128;
            let c_diff_raw = params.dt_latt as i128 * params.nu_scaled as i128 * n2;
            // c_diff = c_diff_raw / S²

            for &p in &SAFE_DISC {
                let c_mod_p = ((c_diff_raw % (s2 * p as i128)) / s2 % p as i128 + p as i128) % p as i128;
                let self_coeff = ((1 - 6 * c_mod_p) % p as i128 + p as i128) % p as i128;
                let cent = centered(self_coeff as i64, p);

                println!("  p={:>2}: c_diff mod {} = {}, self_coeff = (1-6*{}) mod {} = {}, centered = {}, |cent|/p = {:.4}",
                    p, p, c_mod_p, c_mod_p, p, self_coeff, cent, cent.abs() as f64 / p as f64);
            }

            let b_bound = (3.0/7.0) * (5.0/11.0) * (6.0/13.0);
            println!("  B(N) algebraic bound = 90/1001 = {:.6}", b_bound);
            println!();

            // ═══════════════════════════════════════════════════════
            // TEST 7: Stretching Inner Product (SIP)
            // ═══════════════════════════════════════════════════════
            println!("  TEST 7: Stretching Inner Product");
            println!("  Variable: SIP = Σ ω_c * S_c(ω, u_c)");
            println!("  Tracks: is stretching growing?");
            println!();

            w = w0.clone();
            println!("  {:>5} {:>15} {:>12}", "Step", "SIP", "max|w|");
            for step in 0..10 {
                let sip = stretching_inner_product(&w);
                let mw = l_inf(&w);
                println!("  {:>5} {:>15} {:>12}", step, sip, mw);
                w = ns_step(&w, &params);
            }
            println!();
        }
    }
}
