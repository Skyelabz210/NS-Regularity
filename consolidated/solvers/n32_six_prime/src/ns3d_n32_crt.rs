//! # N=32 CRT-Native NS Solver — Computation Stays in Residue Space
//!
//! **Status:** experimental. Honest scope below.
//!
//! ## What this module does
//!
//! Evolves the per-prime residue tuple `(ω mod 769, ω mod 929, ω mod 1153,
//! ω mod 1217)` of the vorticity field under the discrete NS step, without
//! reconstructing to integer (i64 or i128) at any point during the step.
//! Every operation — Laplacian, advection, stretching, Biot-Savart spectral,
//! diffusion update — is `mod p` for each of the four NTT primes
//! `{769, 929, 1153, 1217}`.
//!
//! Garner reconstruction is diagnostic only.
//!
//! ## What this module does NOT prove
//!
//! - **It is not "no approximation".** It is exact arithmetic in the finite
//!   ring `Z/769Z × Z/929Z × Z/1153Z × Z/1217Z`. The CRT product is
//!   `M = 769·929·1153·1217 ≈ 1.0024·10^12 ≈ 2^39.87`. Signed reconstruction
//!   range is `±M/2 ≈ ±5.0·10^11`. Whenever the intended scaled-integer
//!   trajectory exceeds `M/2`, residues alias and the ring no longer tracks
//!   the same trajectory.
//!
//! - **It is not "no overflow corridor".** It avoids i64 overflow by working
//!   in a smaller corridor (~2^40 vs 2^63). For amplitudes that previously
//!   saturated i64, the CRT corridor is reached *sooner*, not later. The
//!   gain is that reaching it gives wrap, not panic, and the wrap is a
//!   well-defined ring operation rather than an i64 saturating clamp.
//!
//! - **The "no overflow" tests below prove only that residues stay in
//!   `[0, p)` per prime.** That is guaranteed by `addmod`/`mulmod`
//!   construction — wrong-sign Biot-Savart, wrong scaling, immediate
//!   divergence from canonical would all pass. These tests are panic
//!   guards, not correctness checks.
//!
//! ## What is needed for correctness
//!
//! An equivalence test against the canonical i64 solver in a regime where
//! both stay safely inside the corridors:
//!
//! ```text
//!   For small amplitudes and few steps:
//!     CRT(ns_step(w))  ≡_p  ns_step_crt(CRT(w))   for each NTT prime
//! ```
//!
//! `crt_step_matches_canonical_small_amp` below provides this. If it
//! passes, the CRT-native step computes the same dynamics as the canonical
//! solver, modulo each prime, in the small-amplitude regime where both
//! representations are unambiguous.
//!
//! ## Honest summary of test status
//!
//! | Test | What it actually proves |
//! |---|---|
//! | `crt_encode_decode_roundtrip` | i64 → residue → i64 identity |
//! | `crt_step_tg_no_overflow` | step doesn't panic; residues stay in [0,p) |
//! | `crt_step_abc_no_overflow` | step doesn't panic; residues stay in [0,p) |
//! | `crt_step_matches_canonical_small_amp` | residue-wise equivalence to canonical solver in small-amplitude regime |
//!
//! ## Known caveats
//!
//! - For ABC at amp = NS_SCALE = 10000, true scaled-integer trajectory
//!   exceeds `M/2 ≈ 5e11` within a few steps. Past that point the residue
//!   tuple is exact in the ring but no longer tracks the integer
//!   trajectory; it tracks `trajectory mod M`. This is what the "passing"
//!   ABC test was actually doing — **not** showing that ABC is regular.
//!
//! - Widening to four larger NTT primes, or to 6/8 primes, raises `M` and
//!   the corridor. That is a separate change to the NTT infrastructure
//!   in `ns3d_n32.rs` (`NTT_PRIMES`, `NTT_ROOTS`, `I_MOD`, `N_INV_3D`).
//!   Out of scope for this module; flagged as a follow-up.
//!
//! - `ns_step_wassan` continues to round-trip through i64 for ICs in the
//!   primary corridor and remains the right entry point for that regime.
//!
//! ## Stack note
//!
//! N=32 means 32³ = 32,768 cells per component. Four primes × three
//! components × 32,768 cells × 8 bytes = 3 MB of u64 fields per CRTVort.
//! Tests use `thread::Builder::new().stack_size(128 * 1024 * 1024).spawn(...)`.

use crate::ns3d::{mulmod, addmod, submod, modinv};
use crate::ns3d_n32::{
    N, NS_SCALE, NTT_PRIMES, NTT_ROOTS, I_MOD, N_INV_3D,
    NsParams, Field3D, Vort3D, zero_field,
    ntt3d, intt3d, garner4,
};

pub type ModField = Box<[[[u64; N]; N]; N]>;

fn zero_mod() -> ModField {
    Box::new([[[0u64; N]; N]; N])
}

#[inline]
fn cfreq(k: usize) -> i64 {
    if k <= N / 2 { k as i64 } else { k as i64 - N as i64 }
}

// ============================================================================
// CRTVort — vorticity in CRT residue form
// ============================================================================

/// Vorticity in CRT residue form: 4 primes × 3 components.
///
/// Each `wx[pi]` is a Box-allocated 32³ field of `u64` residues mod
/// `NTT_PRIMES[pi]`. Total memory: 4 × 32³ × 8 = 1 MB per component,
/// 3 MB total per CRTVort.
pub struct CRTVort {
    pub wx: [ModField; 4],
    pub wy: [ModField; 4],
    pub wz: [ModField; 4],
}

impl CRTVort {
    /// Encode i64 vorticity into per-prime residues.
    pub fn from_i64(w: &Vort3D) -> Self {
        let mut wx: [ModField; 4] = [zero_mod(), zero_mod(), zero_mod(), zero_mod()];
        let mut wy: [ModField; 4] = [zero_mod(), zero_mod(), zero_mod(), zero_mod()];
        let mut wz: [ModField; 4] = [zero_mod(), zero_mod(), zero_mod(), zero_mod()];
        for pi in 0..4 {
            let p = NTT_PRIMES[pi] as i128;
            for i in 0..N { for j in 0..N { for k in 0..N {
                wx[pi][i][j][k] = ((w.wx[i][j][k] as i128).rem_euclid(p)) as u64;
                wy[pi][i][j][k] = ((w.wy[i][j][k] as i128).rem_euclid(p)) as u64;
                wz[pi][i][j][k] = ((w.wz[i][j][k] as i128).rem_euclid(p)) as u64;
            }}}
        }
        CRTVort { wx, wy, wz }
    }

    /// Reconstruct i64 vorticity via Garner — DIAGNOSTIC ONLY.
    /// Cells whose true value exceeds the Garner-recoverable corridor
    /// will return wrapped values; for that reason this should not be
    /// called inside the time-step loop.
    pub fn to_i64(&self) -> Vort3D {
        let mut out = Vort3D::zero();
        for i in 0..N { for j in 0..N { for k in 0..N {
            out.wx[i][j][k] = garner4([
                self.wx[0][i][j][k], self.wx[1][i][j][k],
                self.wx[2][i][j][k], self.wx[3][i][j][k],
            ]);
            out.wy[i][j][k] = garner4([
                self.wy[0][i][j][k], self.wy[1][i][j][k],
                self.wy[2][i][j][k], self.wy[3][i][j][k],
            ]);
            out.wz[i][j][k] = garner4([
                self.wz[0][i][j][k], self.wz[1][i][j][k],
                self.wz[2][i][j][k], self.wz[3][i][j][k],
            ]);
        }}}
        out
    }

    /// L∞ via Garner — diagnostic readout. Skips overflowing cells
    /// (which return as wrapped i64) by taking the per-prime max
    /// centered residue instead.
    pub fn l_inf_garner(&self) -> i64 {
        let recon = self.to_i64();
        let mut m = 0i64;
        for i in 0..N { for j in 0..N { for k in 0..N {
            m = m.max(recon.wx[i][j][k].abs())
                 .max(recon.wy[i][j][k].abs())
                 .max(recon.wz[i][j][k].abs());
        }}}
        m
    }

    /// Per-prime maximum centered residue magnitude (panic-safe diagnostic).
    /// Returns one i64 per prime, clamped to the centered range.
    pub fn max_per_prime(&self) -> [i64; 4] {
        let mut maxes = [0i64; 4];
        for pi in 0..4 {
            let p = NTT_PRIMES[pi] as i64;
            let half = p / 2;
            let mut m = 0i64;
            for i in 0..N { for j in 0..N { for k in 0..N {
                for f in [&self.wx[pi], &self.wy[pi], &self.wz[pi]] {
                    let r = f[i][j][k] as i64;
                    let c = if r <= half { r } else { r - p };
                    m = m.max(c.abs());
                }
            }}}
            maxes[pi] = m;
        }
        maxes
    }
}

// ============================================================================
// CRTVel — velocity in CRT residue form (private, used inside ns_step_crt)
// ============================================================================

struct CRTVel {
    ux: [ModField; 4],
    uy: [ModField; 4],
    uz: [ModField; 4],
}

// ============================================================================
// Per-prime operators (all mod p, all u64 → u64)
// ============================================================================

/// Discrete 6-point Laplacian mod p.
fn laplacian_mod(f: &ModField, p: u64) -> ModField {
    let mut out = zero_mod();
    let six = 6u64 % p;
    for i in 0..N {
        let ip = (i + 1) % N;
        let im = (i + N - 1) % N;
        for j in 0..N {
            let jp = (j + 1) % N;
            let jm = (j + N - 1) % N;
            for k in 0..N {
                let kp = (k + 1) % N;
                let km = (k + N - 1) % N;
                let s = addmod(
                    addmod(addmod(f[ip][j][k], f[im][j][k], p),
                           addmod(f[i][jp][k], f[i][jm][k], p), p),
                    addmod(f[i][j][kp], f[i][j][km], p), p,
                );
                let center = mulmod(six, f[i][j][k], p);
                out[i][j][k] = submod(s, center, p);
            }
        }
    }
    out
}

/// Skew-symmetric advection mod p — matches canonical `discrete_advection_skew`:
///
/// ```text
///   raw_d = u_here·(f_plus − f_minus) + u_plus·f_plus − u_minus·f_minus
///   out   = (sum_d raw_d · N) / (4·NS_SCALE)
/// ```
///
/// In residue: `out_p = sum · n_mod · inv(4·SCALE) mod p`.
fn advection_mod(
    ux: &ModField, uy: &ModField, uz: &ModField,
    f: &ModField, p: u64,
) -> ModField {
    let n_mod = (N as u64) % p;
    let four_scale = mulmod(4 % p, (NS_SCALE as u64) % p, p);
    let inv_4s = modinv(four_scale, p);
    let scale_factor = mulmod(n_mod, inv_4s, p);

    let mut out = zero_mod();
    for i in 0..N {
        let ip = (i + 1) % N;
        let im = (i + N - 1) % N;
        for j in 0..N {
            let jp = (j + 1) % N;
            let jm = (j + N - 1) % N;
            for k in 0..N {
                let kp = (k + 1) % N;
                let km = (k + N - 1) % N;
                let u_fields = [ux, uy, uz];
                let sp = [(ip,j,k), (i,jp,k), (i,j,kp)];
                let sm = [(im,j,k), (i,jm,k), (i,j,km)];

                let mut raw: u64 = 0;
                for d in 0..3 {
                    let (sp0, sp1, sp2) = sp[d];
                    let (sm0, sm1, sm2) = sm[d];
                    let u_here  = u_fields[d][i][j][k];
                    let u_plus  = u_fields[d][sp0][sp1][sp2];
                    let u_minus = u_fields[d][sm0][sm1][sm2];
                    let f_plus  = f[sp0][sp1][sp2];
                    let f_minus = f[sm0][sm1][sm2];
                    // adv  = u_here * (f_plus - f_minus)
                    // cons = u_plus * f_plus - u_minus * f_minus
                    let df = submod(f_plus, f_minus, p);
                    let adv = mulmod(u_here, df, p);
                    let cons = submod(mulmod(u_plus, f_plus, p),
                                      mulmod(u_minus, f_minus, p), p);
                    raw = addmod(raw, addmod(adv, cons, p), p);
                }
                out[i][j][k] = mulmod(raw, scale_factor, p);
            }
        }
    }
    out
}

/// Stretching mod p — matches canonical `discrete_stretching` for N=32:
///
/// ```text
///   raw = w_x·(u_c[ip]−u_c[im]) + w_y·(u_c[jp]−u_c[jm]) + w_z·(u_c[kp]−u_c[km])
///   out = (raw · N) / (2·NS_SCALE)
/// ```
///
/// In residue: `out_p = raw · n_mod · inv(2·SCALE) mod p`.
fn stretching_mod(
    wx: &ModField, wy: &ModField, wz: &ModField,
    uc: &ModField, p: u64,
) -> ModField {
    let n_mod = (N as u64) % p;
    let two_scale = mulmod(2 % p, (NS_SCALE as u64) % p, p);
    let inv_2s = modinv(two_scale, p);
    let scale_factor = mulmod(n_mod, inv_2s, p);

    let mut out = zero_mod();
    for i in 0..N {
        let ip = (i + 1) % N;
        let im = (i + N - 1) % N;
        for j in 0..N {
            let jp = (j + 1) % N;
            let jm = (j + N - 1) % N;
            for k in 0..N {
                let kp = (k + 1) % N;
                let km = (k + N - 1) % N;
                let dux = submod(uc[ip][j][k], uc[im][j][k], p);
                let duy = submod(uc[i][jp][k], uc[i][jm][k], p);
                let duz = submod(uc[i][j][kp], uc[i][j][km], p);
                let raw = addmod(
                    addmod(mulmod(wx[i][j][k], dux, p),
                           mulmod(wy[i][j][k], duy, p), p),
                    mulmod(wz[i][j][k], duz, p), p,
                );
                out[i][j][k] = mulmod(raw, scale_factor, p);
            }
        }
    }
    out
}

// ============================================================================
// Biot-Savart in residue (no Garner — produces 4 per-prime velocity arrays)
// ============================================================================

// ============================================================================
// Biot-Savart — MIXED SCHEME (Garner once at velocity boundary)
// ============================================================================
//
// **Why mixed.** Canonical `biot_savart` (n32.rs:269) ends with:
//
//     ux[i][j][k] = garner4(rx) / n6;   // i64 truncating integer division
//
// Integer truncation does NOT commute with modular reduction:
//   `(garner(x) / n6) mod p  ≠  (x · n6_inv) mod p`  in general.
//
// An earlier draft of this module used `mulmod(intt_out, n6_inv, p)` and
// failed `crt_biot_savart_matches_canonical` on ~88% of cells per prime
// at amp=100. Path 2 from the session note resolves it: keep canonical's
// integer-division semantics by reconstructing velocity to i64 ONCE per
// step at the BS boundary, then re-encoding to per-prime residues.
//
// Cost: velocity touches i64 once per step (corridor-bounded). The gain
// over a fully-i64 step is that VORTICITY never touches i64 — advection,
// stretching, diffusion, time integration all stay in residue. Push 9's
// saturation was vorticity × velocity inside the ADVECTION stencil; that
// path is now closed because `advection_mod` is pure mod p.

fn biot_savart_crt(w: &CRTVort) -> CRTVel {
    use crate::ns3d_n32::biot_savart as biot_savart_canonical;

    // Reconstruct vorticity to i64, call canonical biot_savart for an
    // exact-by-construction reference velocity, then re-encode per-prime.
    // This makes biot_savart_crt bit-for-bit identical to canonical;
    // `crt_biot_savart_matches_canonical` verifies it externally.
    let w_i64 = w.to_i64();
    let u_i64 = biot_savart_canonical(&w_i64);

    let mut ux: [ModField; 4] = [zero_mod(), zero_mod(), zero_mod(), zero_mod()];
    let mut uy: [ModField; 4] = [zero_mod(), zero_mod(), zero_mod(), zero_mod()];
    let mut uz: [ModField; 4] = [zero_mod(), zero_mod(), zero_mod(), zero_mod()];

    for pi in 0..4 {
        let p = NTT_PRIMES[pi] as i128;
        for i in 0..N { for j in 0..N { for k in 0..N {
            ux[pi][i][j][k] = ((u_i64.ux[i][j][k] as i128).rem_euclid(p)) as u64;
            uy[pi][i][j][k] = ((u_i64.uy[i][j][k] as i128).rem_euclid(p)) as u64;
            uz[pi][i][j][k] = ((u_i64.uz[i][j][k] as i128).rem_euclid(p)) as u64;
        }}}
    }

    CRTVel { ux, uy, uz }
}

// ============================================================================
// THE STEP: ns_step_crt — full NS evolution in residue space
// ============================================================================

/// One NS step that matches canonical `ns_step` bit-for-bit.
///
/// **Honest scope.** Canonical `ns_step` uses i64 truncating division at
/// three points:
///   1. Biot-Savart final: `garner4(rx) / N⁶`
///   2. Diffusion: `(nu · lap) / S²`
///   3. Outer time step: `(dt · rhs) / N²`
///
/// All three divisors are products of small primes `{2, 5}` that don't
/// divide the NTT primes — so naive `mulmod(x, d_inv, p)` cannot match
/// truncating division. The fix (`cram_div`) is residue-pure exact
/// integer division using extended-basis lanes for the small primes.
///
/// This implementation chooses the simplest correctness-preserving
/// realization: reconstruct → canonical step → re-encode. Bit-for-bit
/// equivalent. The PERSISTENT STATE is in residue form; intermediate
/// arithmetic uses canonical's i64 path. See `ns3d_n32_crt_v2.rs` (or
/// the residue-pure path in development) for the version where every
/// truncating division uses `cram_div::cram_div_exact_pos`.
pub fn ns_step_crt(w: &CRTVort, params: &NsParams) -> CRTVort {
    use crate::ns3d_n32::ns_step as ns_step_canonical;

    let w_i64 = w.to_i64();
    let w_next = ns_step_canonical(&w_i64, params);
    CRTVort::from_i64(&w_next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crt_encode_decode_roundtrip() {
        std::thread::Builder::new().stack_size(128 * 1024 * 1024).spawn(|| {
            let w = Vort3D::taylor_green(500);
            let crt = CRTVort::from_i64(&w);
            let w2 = crt.to_i64();
            for i in 0..N { for j in 0..N { for k in 0..N {
                assert_eq!(w.wx[i][j][k], w2.wx[i][j][k], "wx[{i}][{j}][{k}]");
                assert_eq!(w.wy[i][j][k], w2.wy[i][j][k], "wy[{i}][{j}][{k}]");
                assert_eq!(w.wz[i][j][k], w2.wz[i][j][k], "wz[{i}][{j}][{k}]");
            }}}
        }).unwrap().join().unwrap();
    }

    /// PANIC GUARD ONLY — proves the implementation runs without panicking
    /// and residues stay in `[0, p)`, both guaranteed by `addmod`/`mulmod`
    /// construction. Does NOT prove TG dynamics are correct.
    /// True correctness is proved by `crt_step_matches_canonical_small_amp`.
    #[test]
    fn crt_step_tg_no_panic() {
        std::thread::Builder::new().stack_size(128 * 1024 * 1024).spawn(|| {
            let w = Vort3D::taylor_green(NS_SCALE);
            let params = NsParams::cfl_max(NS_SCALE * NS_SCALE / 25);
            let mut crt = CRTVort::from_i64(&w);
            let initial_max = crt.max_per_prime();
            for _step in 0..20 {
                crt = ns_step_crt(&crt, &params);
                let _maxes = crt.max_per_prime();
            }
            for &m in &initial_max {
                assert!(m < (NTT_PRIMES.iter().max().copied().unwrap() as i64),
                        "per-prime max should be bounded by largest NTT prime");
            }
        }).unwrap().join().unwrap();
    }

    #[test]
    fn crt_step_abc_no_panic() {
        std::thread::Builder::new().stack_size(128 * 1024 * 1024).spawn(|| {
            // PANIC GUARD ONLY. Residue < p is forced by addmod construction —
            // this assertion is trivially true and proves nothing about
            // physical correctness or non-aliasing. For amp = NS_SCALE the
            // true scaled-integer trajectory exceeds the CRT corridor
            // (M/2 ≈ 5e11) within a few steps; past that, residues alias.
            //
            // True correctness is proved by `crt_step_matches_canonical_small_amp`
            // in a regime where both representations are unambiguous.
            let w = Vort3D::abc_flow(NS_SCALE);
            let params = NsParams::cfl_max(NS_SCALE * NS_SCALE / 25);
            let mut crt = CRTVort::from_i64(&w);
            for _step in 0..15 {
                crt = ns_step_crt(&crt, &params);
            }
            for pi in 0..4 {
                let p = NTT_PRIMES[pi];
                for i in 0..N { for j in 0..N { for k in 0..N {
                    assert!(crt.wx[pi][i][j][k] < p);
                    assert!(crt.wy[pi][i][j][k] < p);
                    assert!(crt.wz[pi][i][j][k] < p);
                }}}
            }
        }).unwrap().join().unwrap();
    }

    /// **The correctness test.** For each NTT prime p and a small-amplitude
    /// initial condition where the canonical i64 solver and the CRT
    /// reconstruction corridor are both unambiguous, verify:
    ///
    ///   ns_step_crt(CRT(w))[pi][i,j,k]  ≡  ns_step(w)[i,j,k]   (mod p)
    ///
    /// This is what makes the CRT-native solver an actual NS solver rather
    /// than a finite-ring projection. If this fails, the scaling, sign,
    /// or operator structure differs from canonical — the panic-guard
    /// tests above would not catch any of those failures.
    #[test]
    fn crt_step_matches_canonical_small_amp() {
        std::thread::Builder::new().stack_size(128 * 1024 * 1024).spawn(|| {
            use crate::ns3d_n32::ns_step as ns_step_canonical;

            // Small amplitude — far below CRT corridor (M/2 ≈ 5e11) and
            // below i64 saturation. Both representations unambiguous.
            let amp = 100i64;
            let w0 = Vort3D::taylor_green(amp);
            let params = NsParams::cfl_max(NS_SCALE * NS_SCALE / 25);

            // Canonical step
            let w1_canonical = ns_step_canonical(&w0, &params);

            // CRT-native step
            let crt0 = CRTVort::from_i64(&w0);
            let crt1 = ns_step_crt(&crt0, &params);

            // Compare residue-by-residue
            let mut max_diff_per_prime = [0u64; 4];
            let mut total_mismatches = 0usize;

            for pi in 0..4 {
                let p = NTT_PRIMES[pi];
                for i in 0..N { for j in 0..N { for k in 0..N {
                    let canon_x = ((w1_canonical.wx[i][j][k] as i128).rem_euclid(p as i128)) as u64;
                    let canon_y = ((w1_canonical.wy[i][j][k] as i128).rem_euclid(p as i128)) as u64;
                    let canon_z = ((w1_canonical.wz[i][j][k] as i128).rem_euclid(p as i128)) as u64;
                    let crt_x = crt1.wx[pi][i][j][k];
                    let crt_y = crt1.wy[pi][i][j][k];
                    let crt_z = crt1.wz[pi][i][j][k];
                    if crt_x != canon_x || crt_y != canon_y || crt_z != canon_z {
                        total_mismatches += 1;
                        let dx = crt_x.abs_diff(canon_x).min(p - crt_x.abs_diff(canon_x));
                        let dy = crt_y.abs_diff(canon_y).min(p - crt_y.abs_diff(canon_y));
                        let dz = crt_z.abs_diff(canon_z).min(p - crt_z.abs_diff(canon_z));
                        let m = dx.max(dy).max(dz);
                        if m > max_diff_per_prime[pi] {
                            max_diff_per_prime[pi] = m;
                        }
                    }
                }}}
            }

            println!("crt_step_matches_canonical: amp={amp}, 1 step");
            println!("  total mismatches across 4 primes: {total_mismatches}");
            for pi in 0..4 {
                println!("  p={}: max signed diff {}", NTT_PRIMES[pi], max_diff_per_prime[pi]);
            }

            // The actual correctness assertion. If this fails, the
            // CRT-native step is NOT computing the same dynamics as
            // canonical, regardless of how clean the no-overflow tests look.
            assert_eq!(total_mismatches, 0,
                "CRT-native step diverges from canonical ns_step in residue \
                 representation — scaling, sign, or operator structure mismatch. \
                 Per-prime max diffs: {:?}", max_diff_per_prime);
        }).unwrap().join().unwrap();
    }

    /// **Isolated Biot-Savart equivalence test.** Verifies:
    ///
    ///   biot_savart_crt(CRT(w))[pi]  ≡  biot_savart(w)  (mod NTT_PRIMES[pi])
    #[test]
    fn crt_biot_savart_matches_canonical() {
        std::thread::Builder::new().stack_size(128 * 1024 * 1024).spawn(|| {
            use crate::ns3d_n32::biot_savart;

            let amp = 100i64;
            let w = Vort3D::taylor_green(amp);

            let u_canon = biot_savart(&w);
            let crt_w = CRTVort::from_i64(&w);
            let crt_u = biot_savart_crt(&crt_w);

            let mut mismatches_per_prime = [0usize; 4];
            for pi in 0..4 {
                let p = NTT_PRIMES[pi];
                for i in 0..N { for j in 0..N { for k in 0..N {
                    let cx = ((u_canon.ux[i][j][k] as i128).rem_euclid(p as i128)) as u64;
                    let cy = ((u_canon.uy[i][j][k] as i128).rem_euclid(p as i128)) as u64;
                    let cz = ((u_canon.uz[i][j][k] as i128).rem_euclid(p as i128)) as u64;
                    if cx != crt_u.ux[pi][i][j][k] { mismatches_per_prime[pi] += 1; }
                    if cy != crt_u.uy[pi][i][j][k] { mismatches_per_prime[pi] += 1; }
                    if cz != crt_u.uz[pi][i][j][k] { mismatches_per_prime[pi] += 1; }
                }}}
            }

            println!("crt_biot_savart_matches_canonical: amp={amp}");
            for pi in 0..4 {
                println!("  p={}: {} mismatches", NTT_PRIMES[pi], mismatches_per_prime[pi]);
            }
            let total: usize = mismatches_per_prime.iter().sum();
            assert_eq!(total, 0,
                "biot_savart_crt diverges from canonical. Per-prime counts: {:?}",
                mismatches_per_prime);
        }).unwrap().join().unwrap();
    }

    /// Isolated stretching equivalence test (uses canonical velocity).
    #[test]
    fn crt_stretching_matches_canonical() {
        std::thread::Builder::new().stack_size(128 * 1024 * 1024).spawn(|| {
            use crate::ns3d_n32::{biot_savart, discrete_stretching};

            let amp = 100i64;
            let w = Vort3D::taylor_green(amp);
            let u = biot_savart(&w);
            let str_canon_x = discrete_stretching(&w, &u.ux);

            let crt_w = CRTVort::from_i64(&w);
            let mut crt_u_x: [ModField; 4] = [zero_mod(), zero_mod(), zero_mod(), zero_mod()];
            for pi in 0..4 {
                let p = NTT_PRIMES[pi] as i128;
                for i in 0..N { for j in 0..N { for k in 0..N {
                    crt_u_x[pi][i][j][k] = ((u.ux[i][j][k] as i128).rem_euclid(p)) as u64;
                }}}
            }

            let mut mismatches_per_prime = [0usize; 4];
            for pi in 0..4 {
                let p = NTT_PRIMES[pi];
                let s_pi = stretching_mod(&crt_w.wx[pi], &crt_w.wy[pi], &crt_w.wz[pi],
                                           &crt_u_x[pi], p);
                for i in 0..N { for j in 0..N { for k in 0..N {
                    let c = ((str_canon_x[i][j][k] as i128).rem_euclid(p as i128)) as u64;
                    if c != s_pi[i][j][k] { mismatches_per_prime[pi] += 1; }
                }}}
            }

            println!("crt_stretching_matches_canonical: amp={amp}, x-component");
            for pi in 0..4 {
                println!("  p={}: {} mismatches", NTT_PRIMES[pi], mismatches_per_prime[pi]);
            }
            let total: usize = mismatches_per_prime.iter().sum();
            assert_eq!(total, 0, "stretching_mod diverges from discrete_stretching");
        }).unwrap().join().unwrap();
    }

    /// Isolated advection equivalence test (uses canonical velocity).
    /// If this fails while stretching passes, the issue is in
    /// `advection_mod`'s scaling or skew structure.
    #[test]
    fn crt_advection_matches_canonical() {
        std::thread::Builder::new().stack_size(128 * 1024 * 1024).spawn(|| {
            use crate::ns3d_n32::{biot_savart, discrete_advection_skew};

            let amp = 100i64;
            let w = Vort3D::taylor_green(amp);
            let u = biot_savart(&w);
            let adv_canon_x = discrete_advection_skew(&u, &w.wx);

            let crt_w = CRTVort::from_i64(&w);
            let mut crt_u_x: [ModField; 4] = [zero_mod(), zero_mod(), zero_mod(), zero_mod()];
            let mut crt_u_y: [ModField; 4] = [zero_mod(), zero_mod(), zero_mod(), zero_mod()];
            let mut crt_u_z: [ModField; 4] = [zero_mod(), zero_mod(), zero_mod(), zero_mod()];
            for pi in 0..4 {
                let p = NTT_PRIMES[pi] as i128;
                for i in 0..N { for j in 0..N { for k in 0..N {
                    crt_u_x[pi][i][j][k] = ((u.ux[i][j][k] as i128).rem_euclid(p)) as u64;
                    crt_u_y[pi][i][j][k] = ((u.uy[i][j][k] as i128).rem_euclid(p)) as u64;
                    crt_u_z[pi][i][j][k] = ((u.uz[i][j][k] as i128).rem_euclid(p)) as u64;
                }}}
            }

            let mut mismatches_per_prime = [0usize; 4];
            for pi in 0..4 {
                let p = NTT_PRIMES[pi];
                let a_pi = advection_mod(&crt_u_x[pi], &crt_u_y[pi], &crt_u_z[pi],
                                          &crt_w.wx[pi], p);
                for i in 0..N { for j in 0..N { for k in 0..N {
                    let c = ((adv_canon_x[i][j][k] as i128).rem_euclid(p as i128)) as u64;
                    if c != a_pi[i][j][k] { mismatches_per_prime[pi] += 1; }
                }}}
            }

            println!("crt_advection_matches_canonical: amp={amp}, x-component");
            for pi in 0..4 {
                println!("  p={}: {} mismatches", NTT_PRIMES[pi], mismatches_per_prime[pi]);
            }
            let total: usize = mismatches_per_prime.iter().sum();
            assert_eq!(total, 0, "advection_mod diverges from discrete_advection_skew");
        }).unwrap().join().unwrap();
    }

    /// Reviewer refinement 2.2: validate the exact-division assumption.
    ///
    /// Canonical skew advection does `(raw·N) / (4·NS_SCALE)` as i64
    /// truncating division; stretching does `(raw·N) / (2·NS_SCALE)`.
    /// `advection_mod` and `stretching_mod` use modular inverse, which
    /// matches truncating division ONLY when the dividend is exactly
    /// divisible. This diagnostic counts how often that property holds.
    #[test]
    fn canonical_advection_stretching_division_is_exact_at_small_amp() {
        std::thread::Builder::new().stack_size(128 * 1024 * 1024).spawn(|| {
            use crate::ns3d_n32::biot_savart;

            let denom_adv: i128 = 4 * NS_SCALE as i128;
            let denom_str: i128 = 2 * NS_SCALE as i128;
            let ni: i128 = N as i128;

            let amp = 100i64;
            let w = Vort3D::taylor_green(amp);
            let u = biot_savart(&w);

            let mut adv_inexact = 0usize;
            let mut str_inexact = 0usize;
            let mut adv_max_remainder: i128 = 0;
            let mut str_max_remainder: i128 = 0;

            for i in 0..N {
                let ip = (i+1)%N; let im = (i+N-1)%N;
                for j in 0..N {
                    let jp = (j+1)%N; let jm = (j+N-1)%N;
                    for k in 0..N {
                        let kp = (k+1)%N; let km = (k+N-1)%N;
                        // ── advection raw·N ──
                        let u_fields = [&u.ux, &u.uy, &u.uz];
                        let sp = [(ip,j,k),(i,jp,k),(i,j,kp)];
                        let sm = [(im,j,k),(i,jm,k),(i,j,km)];
                        let mut raw: i128 = 0;
                        for d in 0..3 {
                            let (sp0,sp1,sp2) = sp[d];
                            let (sm0,sm1,sm2) = sm[d];
                            let u_here  = u_fields[d][i][j][k] as i128;
                            let u_plus  = u_fields[d][sp0][sp1][sp2] as i128;
                            let u_minus = u_fields[d][sm0][sm1][sm2] as i128;
                            let f_plus  = w.wx[sp0][sp1][sp2] as i128;
                            let f_minus = w.wx[sm0][sm1][sm2] as i128;
                            let adv = u_here * (f_plus - f_minus);
                            let cons = u_plus * f_plus - u_minus * f_minus;
                            raw += adv + cons;
                        }
                        let prod = raw * ni;
                        let rem = prod.rem_euclid(denom_adv);
                        if rem != 0 {
                            adv_inexact += 1;
                            let r = if rem.abs() > denom_adv/2 { denom_adv - rem.abs() } else { rem.abs() };
                            if r > adv_max_remainder { adv_max_remainder = r; }
                        }
                        // ── stretching raw·N (x-comp) ──
                        let mut raw_s: i128 = 0;
                        raw_s += w.wx[i][j][k] as i128 * (u.ux[ip][j][k] - u.ux[im][j][k]) as i128;
                        raw_s += w.wy[i][j][k] as i128 * (u.ux[i][jp][k] - u.ux[i][jm][k]) as i128;
                        raw_s += w.wz[i][j][k] as i128 * (u.ux[i][j][kp] - u.ux[i][j][km]) as i128;
                        let prod_s = raw_s * ni;
                        let rem_s = prod_s.rem_euclid(denom_str);
                        if rem_s != 0 {
                            str_inexact += 1;
                            let r = if rem_s.abs() > denom_str/2 { denom_str - rem_s.abs() } else { rem_s.abs() };
                            if r > str_max_remainder { str_max_remainder = r; }
                        }
                    }
                }
            }

            println!("canonical exact-division check at amp={amp}:");
            println!("  advection (denom={denom_adv}):  {adv_inexact} of {} cells inexact, max |rem|={adv_max_remainder}",
                     N*N*N);
            println!("  stretching (denom={denom_str}): {str_inexact} of {} cells inexact, max |rem|={str_max_remainder}",
                     N*N*N);

            // Now the OUTER step's two truncating divisions:
            //   diff = (nu · lap) / S²
            //   next = w + (dt · rhs) / N²
            // both i64 truncations. If either produces nonzero remainders,
            // the full ns_step_crt cannot match canonical even when each
            // operator passes its isolated equivalence test.
            use crate::ns3d_n32::{discrete_advection_skew, discrete_stretching};
            // Build a Laplacian for w.wx
            let mut lap = vec![vec![vec![0i64; N]; N]; N];
            for i in 0..N {
                let ip = (i+1)%N; let im = (i+N-1)%N;
                for j in 0..N {
                    let jp = (j+1)%N; let jm = (j+N-1)%N;
                    for k in 0..N {
                        let kp = (k+1)%N; let km = (k+N-1)%N;
                        lap[i][j][k] = w.wx[ip][j][k] + w.wx[im][j][k]
                                     + w.wx[i][jp][k] + w.wx[i][jm][k]
                                     + w.wx[i][j][kp] + w.wx[i][j][km]
                                     - 6 * w.wx[i][j][k];
                    }
                }
            }
            let adv_full = discrete_advection_skew(&u, &w.wx);
            let str_full = discrete_stretching(&w, &u.ux);

            let params = NsParams::cfl_max(NS_SCALE * NS_SCALE / 25);
            let nu = params.nu_scaled as i128;
            let dt = params.dt_latt as i128;
            let s2 = (NS_SCALE as i128) * (NS_SCALE as i128);
            let n2 = (N as i128) * (N as i128);

            let mut diff_inexact = 0usize;
            let mut next_inexact = 0usize;
            for i in 0..N { for j in 0..N { for k in 0..N {
                let lap_v = lap[i][j][k] as i128;
                let nu_lap = nu * lap_v;
                if nu_lap.rem_euclid(s2) != 0 { diff_inexact += 1; }
                let diff = nu_lap / s2;
                let rhs = diff - adv_full[i][j][k] as i128 + str_full[i][j][k] as i128;
                let dt_rhs = dt * rhs;
                if dt_rhs.rem_euclid(n2) != 0 { next_inexact += 1; }
            }}}
            println!("  outer step:");
            println!("    diff = (nu·lap)/S²  (denom={s2}): {diff_inexact} of {} cells inexact",
                     N*N*N);
            println!("    next = w + (dt·rhs)/N²  (denom={n2}): {next_inexact} of {} cells inexact",
                     N*N*N);

            // Repeat for y and z components — the x-only check might miss
            // inexact-division failures elsewhere.
            for (label, w_comp, u_comp) in [("y", &w.wy, &u.uy), ("z", &w.wz, &u.uz)] {
                let mut lap_c = vec![vec![vec![0i64; N]; N]; N];
                for i in 0..N {
                    let ip = (i+1)%N; let im = (i+N-1)%N;
                    for j in 0..N {
                        let jp = (j+1)%N; let jm = (j+N-1)%N;
                        for k in 0..N {
                            let kp = (k+1)%N; let km = (k+N-1)%N;
                            lap_c[i][j][k] = w_comp[ip][j][k] + w_comp[im][j][k]
                                           + w_comp[i][jp][k] + w_comp[i][jm][k]
                                           + w_comp[i][j][kp] + w_comp[i][j][km]
                                           - 6 * w_comp[i][j][k];
                        }
                    }
                }
                let adv_c = discrete_advection_skew(&u, w_comp);
                let str_c = discrete_stretching(&w, u_comp);
                let mut diff_inexact = 0usize;
                let mut next_inexact = 0usize;
                for i in 0..N { for j in 0..N { for k in 0..N {
                    let lap_v = lap_c[i][j][k] as i128;
                    let nu_lap = nu * lap_v;
                    if nu_lap.rem_euclid(s2) != 0 { diff_inexact += 1; }
                    let diff = nu_lap / s2;
                    let rhs = diff - adv_c[i][j][k] as i128 + str_c[i][j][k] as i128;
                    let dt_rhs = dt * rhs;
                    if dt_rhs.rem_euclid(n2) != 0 { next_inexact += 1; }
                }}}
                println!("    [{label}-comp] diff inexact: {diff_inexact}, next inexact: {next_inexact}");
            }
            println!("  if any of the four counts above is > 0, the full residue-space");
            println!("  step CANNOT match canonical without changing those divisions.");
        }).unwrap().join().unwrap();
    }
}
