//! # NTT 6-Prime Extension — Phase 5A Basis Widening
//!
//! ## Why
//!
//! The 4-prime basis (769, 929, 1153, 1217) gives M_NTT = 1,002,448,197,601 ≈ 10^12
//! and a Biot-Savart raw corridor of M_NTT/2 ≈ 5.01×10^11.
//!
//! corridor_probe_v3 shows that ABC at amp=100 consumes **99.999%** of this
//! corridor in 5 steps. The 4-prime basis is saturated at realistic amplitudes.
//!
//! Adding p5=1409 and p6=1601:
//!
//! ```text
//!   M_NTT_6 = 769·929·1153·1217·1409·1601 = 2,261,331,666,182,114,209
//!   M_NTT_6 / 2                            ≈ 1.13 × 10^18
//! ```
//!
//! This is 2.25 million× larger than the current limit. It extends the BS wall
//! for moderate trajectories, but it does not eliminate the wall in general.
//! Push 10 v4 observed TG amp=1000 reaching state magnitude ~1.3e20 at step 15;
//! that regime cannot be handled by treating arbitrary large primes as neutral
//! capacity. Higher/chaos-regime primes may be used as probes only after a
//! composite layer is built over them and K-Elim on the winding number identifies
//! the resulting object. The CRAM-consistent widening path preserves the Safe
//! Basis ontology: 11 as the shadow link, 2 as recumbent overflow, 13 as stable
//! boundary, 19 as edge-of-chaos, plus composites such as 3*5*7 or 7*p^k where
//! the prime-power role is explicit.
//!
//! ## Prime selection criteria
//!
//! Both p5 and p6 satisfy:
//!   1. `p ≡ 1 mod 64` → primitive 32nd root of unity exists (since 32 | p-1)
//!   2. Primitive 32nd root verified: `r^32 ≡ 1` and `r^16 ≢ 1` mod p
//!   3. `i = sqrt(-1)` exists (= r^8): `i^2 ≡ -1` mod p — required for Biot-Savart cross-product
//!   4. Coprime to 2 and 5 (automatic for primes > 5)
//!   5. Forward/inverse NTT round-trip verified in tests
//!
//! ## Non-breaking design
//!
//! This module is **additive**: the existing 4-prime arrays (NTT_PRIMES, NTT_ROOTS,
//! I_MOD, N_INV_3D, NTT_MAT) are unchanged. New arrays with `_6` suffix provide
//! the extended basis. Functions `ntt3d_ext` / `intt3d_ext` use the extended matrix
//! for prime index in 0..6; `garner6_centered_i128` reconstructs over all 6 primes.
//!
//! ## D-012 compliance
//!
//! All Garner loops are ascending (j = 0..n). Descending order is silent
//! corruption — this module hardcodes ascending.

use crate::ns3d::{modinv, mulmod, addmod, submod};
use crate::ns3d_n32::N;

// ============================================================================
// 6-Prime NTT constants
// ============================================================================

/// All 6 NTT primes — first 4 are the existing basis, p5/p6 are new.
pub const NTT_PRIMES_6: [u64; 6] = [769, 929, 1153, 1217, 1409, 1601];

/// Primitive 32nd roots of unity for each of the 6 primes.
/// Verified: roots32[i]^32 ≡ 1 (mod p) and roots32[i]^16 ≢ 1 (mod p).
pub const NTT_ROOTS_6: [u64; 6] = [251, 701, 286, 540, 10, 109];

/// sqrt(-1) mod p for each prime — i = r^8 where r is the primitive 32nd root.
/// Verified: i_p^2 ≡ -1 (mod p). Required for Biot-Savart spectral cross-product.
pub const I_MOD_6: [u64; 6] = [707, 605, 140, 78, 452, 1561];

/// N^{-3} mod p for each prime — used in 3D INTT normalisation.
/// N=32, N^3=32768. Verified: (32768 mod p) × n_inv_3d ≡ 1 (mod p).
pub const N_INV_3D_6: [u64; 6] = [18, 694, 617, 1110, 765, 1479];

/// Garner mixed-radix coefficient for prime pi (ascending j):
///   G_C_6[pi] = (p0·p1·...·p{pi-1})^{-1} mod p_pi
pub const G_C_6: [u64; 6] = [
    1,    // p0: degenerate (used as 1 in Garner)
    749,  // G_C01:   769^{-1} mod 929
    839,  // G_C012:  (769·929)^{-1} mod 1153
    720,  // G_C0123: (769·929·1153)^{-1} mod 1217
    755,  // (769·929·1153·1217)^{-1} mod 1409
    347,  // (769·929·1153·1217·1409)^{-1} mod 1601
];

/// M_NTT_6 = product of all 6 NTT primes.
pub const M_NTT_6: u128 = 2_261_331_666_182_114_209;

/// BS raw corridor limit for 6-prime basis: |raw| must be ≤ M_NTT_6/2.
pub const M_NTT_6_HALF: u128 = M_NTT_6 / 2; // 1_130_665_833_091_057_104

// ============================================================================
// 6-Prime NTT matrix — NTT_MAT_6[pi][k][j] = omega^{kj mod N} mod p_pi
// ============================================================================

const fn build_ntt_mat_6() -> [[[u64; N]; N]; 6] {
    let mut mat = [[[0u64; N]; N]; 6];
    let mut pi = 0;
    while pi < 6 {
        let p = NTT_PRIMES_6[pi];
        let omega = NTT_ROOTS_6[pi];
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

static NTT_MAT_6: [[[u64; N]; N]; 6] = build_ntt_mat_6();

// ============================================================================
// Extended NTT functions (p_idx in 0..6)
// ============================================================================

pub fn ntt1d_ext(f: &[u64; N], p_idx: usize) -> [u64; N] {
    let p = NTT_PRIMES_6[p_idx];
    let mut out = [0u64; N];
    for k in 0..N {
        let mut acc = 0u64;
        for j in 0..N {
            acc = addmod(acc, mulmod(f[j], NTT_MAT_6[p_idx][k][j], p), p);
        }
        out[k] = acc;
    }
    out
}

pub fn intt1d_unnorm_ext(fhat: &[u64; N], p_idx: usize) -> [u64; N] {
    let p = NTT_PRIMES_6[p_idx];
    let mut out = [0u64; N];
    for j in 0..N {
        let mut acc = 0u64;
        for k in 0..N {
            let inv_idx = (N - k) % N;
            acc = addmod(acc, mulmod(fhat[k], NTT_MAT_6[p_idx][inv_idx][j], p), p);
        }
        out[j] = acc;
    }
    out
}

/// 3D NTT for prime index in 0..6 (backward compatible with existing 4-prime loops).
pub fn ntt3d_ext(f: &[[[i64; N]; N]; N], p_idx: usize) -> Box<[[[u64; N]; N]; N]> {
    let p = NTT_PRIMES_6[p_idx];
    // Reduce to [0, p)
    let mut buf: Box<[[[u64; N]; N]; N]> = Box::new([[[0u64; N]; N]; N]);
    for i in 0..N { for j in 0..N { for k in 0..N {
        buf[i][j][k] = ((f[i][j][k] % p as i64 + p as i64) as u64) % p;
    }}}
    // Axis 2 (k)
    for i in 0..N { for j in 0..N {
        let mut row = [0u64; N];
        for k in 0..N { row[k] = buf[i][j][k]; }
        let tr = ntt1d_ext(&row, p_idx);
        for k in 0..N { buf[i][j][k] = tr[k]; }
    }}
    // Axis 1 (j)
    for i in 0..N { for k in 0..N {
        let mut row = [0u64; N];
        for j in 0..N { row[j] = buf[i][j][k]; }
        let tr = ntt1d_ext(&row, p_idx);
        for j in 0..N { buf[i][j][k] = tr[j]; }
    }}
    // Axis 0 (i)
    for j in 0..N { for k in 0..N {
        let mut row = [0u64; N];
        for i in 0..N { row[i] = buf[i][j][k]; }
        let tr = ntt1d_ext(&row, p_idx);
        for i in 0..N { buf[i][j][k] = tr[i]; }
    }}
    buf
}

/// 3D INTT for prime index in 0..6.
pub fn intt3d_ext(fhat: &[[[u64; N]; N]; N], p_idx: usize) -> Box<[[[u64; N]; N]; N]> {
    let p = NTT_PRIMES_6[p_idx];
    let mut buf: Box<[[[u64; N]; N]; N]> = Box::new(*fhat);
    for i in 0..N { for j in 0..N {
        let mut row = [0u64; N]; for k in 0..N { row[k] = buf[i][j][k]; }
        let tr = intt1d_unnorm_ext(&row, p_idx);
        for k in 0..N { buf[i][j][k] = tr[k]; }
    }}
    for i in 0..N { for k in 0..N {
        let mut row = [0u64; N]; for j in 0..N { row[j] = buf[i][j][k]; }
        let tr = intt1d_unnorm_ext(&row, p_idx);
        for j in 0..N { buf[i][j][k] = tr[j]; }
    }}
    for j in 0..N { for k in 0..N {
        let mut row = [0u64; N]; for i in 0..N { row[i] = buf[i][j][k]; }
        let tr = intt1d_unnorm_ext(&row, p_idx);
        for i in 0..N { buf[i][j][k] = tr[i]; }
    }}
    let ninv = N_INV_3D_6[p_idx];
    for i in 0..N { for j in 0..N { for k in 0..N {
        buf[i][j][k] = mulmod(buf[i][j][k], ninv, p);
    }}}
    buf
}

// ============================================================================
// 6-prime Garner reconstruction
// ============================================================================

/// Garner reconstruction over all 6 NTT primes.
///
/// Returns the centred representative in (-M_NTT_6/2, M_NTT_6/2] as i128.
/// Valid range: any integer with |x| < M_NTT_6/2 ≈ 1.13×10^18.
///
/// **D-012 compliance:** j-loop is ASCENDING. This is hardcoded — never change
/// to descending.
pub fn garner6_centered_i128(r: [u64; 6]) -> i128 {
    let p = NTT_PRIMES_6;

    // Ascending Garner (D-012 compliant)
    let a0 = r[0] as u128;
    // a1 = (r1 - a0) * (p0)^{-1} mod p1
    let a1 = ((r[1] as i128 - a0 as i128).rem_euclid(p[1] as i128) as u64 * G_C_6[1] % p[1]) as u128;
    // t2 = (a0 + p0*a1) mod p2
    let t2 = (a0 + p[0] as u128 * a1) % p[2] as u128;
    let a2 = ((r[2] as i128 - t2 as i128).rem_euclid(p[2] as i128) as u64 * G_C_6[2] % p[2]) as u128;
    // t3 = (a0 + p0*a1 + p0*p1*a2) mod p3
    let p01 = p[0] as u128 * p[1] as u128;
    let t3 = (a0 + p[0] as u128 * a1 + p01 * a2) % p[3] as u128;
    let a3 = ((r[3] as i128 - t3 as i128).rem_euclid(p[3] as i128) as u64 * G_C_6[3] % p[3]) as u128;
    // t4 = (a0 + p0*a1 + p01*a2 + p012*a3) mod p4
    let p012 = p01 * p[2] as u128;
    let t4 = (a0 + p[0] as u128 * a1 + p01 * a2 + p012 * a3) % p[4] as u128;
    let a4 = ((r[4] as i128 - t4 as i128).rem_euclid(p[4] as i128) as u64 * G_C_6[4] % p[4]) as u128;
    // t5 = (a0 + p0*a1 + p01*a2 + p012*a3 + p0123*a4) mod p5
    let p0123 = p012 * p[3] as u128;
    let t5 = (a0 + p[0] as u128 * a1 + p01 * a2 + p012 * a3 + p0123 * a4) % p[5] as u128;
    let a5 = ((r[5] as i128 - t5 as i128).rem_euclid(p[5] as i128) as u64 * G_C_6[5] % p[5]) as u128;

    // x = a0 + p0*a1 + p01*a2 + p012*a3 + p0123*a4 + p01234*a5
    let p01234 = p0123 * p[4] as u128;
    // This can exceed u128 for the last term — but M_NTT_6 ≈ 2.26e18 << 2^64,
    // so x fits in u64 when computed mod M_NTT_6.
    // Compute x mod M_NTT_6 to stay in range.
    let m6 = M_NTT_6;
    let x = {
        let mut acc: u128 = a0 % m6;
        acc = (acc + p[0] as u128 * a1 % m6) % m6;
        acc = (acc + p01 % m6 * a2 % m6) % m6;
        acc = (acc + p012 % m6 * a3 % m6) % m6;
        acc = (acc + p0123 % m6 * a4 % m6) % m6;
        acc = (acc + p01234 % m6 * a5 % m6) % m6;
        acc
    };

    // Centre: x in [0, M_NTT_6), return signed centred representative
    if x <= m6 / 2 {
        x as i128
    } else {
        x as i128 - m6 as i128
    }
}

// ============================================================================
// TESTS — Phase 5A validation
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ns3d_n32::{NTT_PRIMES, NTT_ROOTS, I_MOD, N_INV_3D, zero_field};
    use crate::ns3d::modinv;

    /// Verify the first 4 entries of 6-prime arrays match the existing 4-prime arrays.
    #[test]
    fn ntt6_first4_match_existing() {
        for i in 0..4 {
            assert_eq!(NTT_PRIMES_6[i], NTT_PRIMES[i], "prime mismatch at i={}", i);
            assert_eq!(NTT_ROOTS_6[i], NTT_ROOTS[i], "root mismatch at i={}", i);
            assert_eq!(I_MOD_6[i], I_MOD[i], "I_MOD mismatch at i={}", i);
            assert_eq!(N_INV_3D_6[i], N_INV_3D[i], "N_INV_3D mismatch at i={}", i);
        }
    }

    /// Verify p5=1409 and p6=1601 have valid 32nd roots and sqrt(-1).
    #[test]
    fn new_primes_roots_valid() {
        for pi in 4..6 {
            let p = NTT_PRIMES_6[pi];
            let r = NTT_ROOTS_6[pi];
            let i_p = I_MOD_6[pi];

            // r^32 ≡ 1 mod p
            let r32: u64 = (0..32).fold(1u128, |acc, _| acc * r as u128 % p as u128) as u64;
            assert_eq!(r32, 1, "p={}: r^32 = {} ≠ 1", p, r32);

            // r^16 ≢ 1 mod p (primitive)
            let r16: u64 = (0..16).fold(1u128, |acc, _| acc * r as u128 % p as u128) as u64;
            assert_ne!(r16, 1, "p={}: r^16 = 1 (not primitive 32nd root)", p);

            // i^2 ≡ -1 mod p
            let i2 = (i_p as u128 * i_p as u128 % p as u128) as u64;
            assert_eq!(i2, p - 1, "p={}: i^2 = {} ≠ p-1={}", p, i2, p-1);

            // I_MOD_6[pi] = r^8
            let r8: u64 = (0..8).fold(1u128, |acc, _| acc * r as u128 % p as u128) as u64;
            assert_eq!(i_p, r8, "p={}: I_MOD = {} but r^8 = {}", p, i_p, r8);

            // N_INV_3D: (N^3 mod p) * N_INV_3D ≡ 1 mod p
            let n3_mod = ((N as u64).pow(3)) % p;
            let check = (n3_mod as u128 * N_INV_3D_6[pi] as u128 % p as u128) as u64;
            assert_eq!(check, 1, "p={}: N^3*N_INV_3D check={} ≠ 1", p, check);
        }
    }

    /// Garner coefficient table is correct.
    #[test]
    fn garner_coefficients_correct() {
        for pi in 1..6 {
            let p = NTT_PRIMES_6[pi];
            // prefix = p0 * p1 * ... * p{pi-1} mod p_pi
            let prefix_mod: u64 = NTT_PRIMES_6[..pi]
                .iter()
                .fold(1u128, |acc, &q| acc * q as u128 % p as u128) as u64;
            let check = (prefix_mod as u128 * G_C_6[pi] as u128 % p as u128) as u64;
            assert_eq!(check, 1, "G_C_6[{}] invalid: prefix_mod={} * G_C={} mod {}={} ≠ 1",
                pi, prefix_mod, G_C_6[pi], p, check);
        }
    }

    /// NTT round-trip for new primes p5 and p6.
    #[test]
    fn ntt_roundtrip_new_primes() {
        std::thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(|| {
            // A simple field: identity at (0,0,0), zeros elsewhere
            let mut f = zero_field();
            f[0][0][0] = 100;
            f[1][0][0] = -50;
            f[0][1][0] = 75;
            f[0][0][1] = 33;

            for pi in 4..6 {
                let p = NTT_PRIMES_6[pi];
                // Forward NTT
                let fhat = ntt3d_ext(&f, pi);
                // Inverse NTT
                let f_recon = intt3d_ext(&*fhat, pi);

                // Compare: f_recon[i][j][k] should equal f[i][j][k] mod p
                for i in 0..N { for j in 0..N { for k in 0..N {
                    let expected = ((f[i][j][k] % p as i64 + p as i64) as u64) % p;
                    assert_eq!(
                        f_recon[i][j][k], expected,
                        "pi={} p={}: roundtrip fail at ({},{},{}): got {} expected {}",
                        pi, p, i, j, k, f_recon[i][j][k], expected
                    );
                }}}
                println!("  NTT roundtrip pi={} p={}: PASS", pi, p);
            }
        }).unwrap().join().unwrap();
    }

    /// garner6_centered_i128 matches garner4 on the first 4 residues.
    #[test]
    fn garner6_matches_garner4() {
        use crate::ns3d_n32::garner4;

        let m_ntt4: u128 = NTT_PRIMES[..4].iter().map(|&p| p as u128).product();
        let mut h: u64 = 0xFEEDBEEF;
        for _ in 0..500 {
            h = h.wrapping_mul(6364136223846793005).wrapping_add(1);
            // x in (-M_NTT4/2, M_NTT4/2) — valid for both 4-prime and 6-prime
            let x = ((h as u128) % m_ntt4) as i128;
            let x_cent = if x <= (m_ntt4 / 2) as i128 { x } else { x - m_ntt4 as i128 };

            let r4: [u64; 4] = NTT_PRIMES[..4].iter().map(|&p| x_cent.rem_euclid(p as i128) as u64).collect::<Vec<_>>().try_into().unwrap();
            let r5 = x_cent.rem_euclid(NTT_PRIMES_6[4] as i128) as u64;
            let r6 = x_cent.rem_euclid(NTT_PRIMES_6[5] as i128) as u64;
            let r6_full = [r4[0], r4[1], r4[2], r4[3], r5, r6];

            let g4 = garner4(r4) as i128;
            let g6 = garner6_centered_i128(r6_full);

            assert_eq!(g4, g6,
                "garner6 mismatch at x={}: garner4={} garner6={}", x_cent, g4, g6);
        }
    }

    /// M_NTT_6 constant is the product of all 6 primes.
    #[test]
    fn m_ntt6_constant_correct() {
        let expected: u128 = NTT_PRIMES_6.iter().map(|&p| p as u128).product();
        assert_eq!(M_NTT_6, expected, "M_NTT_6 mismatch: got {} expected {}", M_NTT_6, expected);
        assert_eq!(M_NTT_6_HALF, expected / 2, "M_NTT_6_HALF mismatch");
    }

    /// BS raw margin is adequate: corridor_probe_v3 showed max|raw|≈5.01×10^11
    /// for ABC at NS_SCALE. New limit is M_NTT_6/2 ≈ 1.13×10^18.
    #[test]
    fn new_corridor_margin_adequate() {
        // From corridor_probe_v3 empirical data (ABC NS_SCALE, 3 steps):
        let max_raw_observed: u128 = 501_201_527_460;
        // Old limit (4 primes):
        let old_limit: u128 = 501_224_098_800; // M_NTT_4/2
        // New limit (6 primes):
        let new_limit: u128 = M_NTT_6_HALF;

        // Old margin was 22,571,340 — dangerously thin
        let old_margin = old_limit as i128 - max_raw_observed as i128;
        // New margin is enormous
        let new_margin = new_limit as i128 - max_raw_observed as i128;

        println!("Old margin (4-prime): {}", old_margin);
        println!("New margin (6-prime): {}", new_margin);
        println!("Improvement factor: {}×", new_margin / old_margin.max(1));

        assert!(old_margin > 0, "Old corridor already violated — data inconsistency");
        assert!(new_margin > 0, "New corridor violated — impossible (M_NTT_6 > observed)");
        // New margin should be millions of times larger
        assert!(new_margin > old_margin * 1_000_000,
            "Expected new margin >> old: new={} old={}", new_margin, old_margin);
    }
}

// ============================================================================
// P5B-9A prerequisite tests
// ============================================================================

#[cfg(test)]
mod p5b9a_prereqs {
    use super::*;
    use crate::ns3d_n32::{NTT_PRIMES, Vort3D, garner4};
    use crate::ns3d::{modinv, mulmod, addmod, submod};

    const N: usize = 32;

    fn cfreq(k: usize) -> i64 {
        if k <= N/2 { k as i64 } else { k as i64 - N as i64 }
    }

    /// garner6 correctly handles values near and beyond the 4p corridor edge.
    #[test]
    fn garner6_near_corridor_edge() {
        let m_ntt4: u128 = NTT_PRIMES[..4].iter().map(|&p| p as u128).product();
        let m_ntt4_half = (m_ntt4 / 2) as i128;
        let m_ntt6_half = (M_NTT_6 / 2) as i128;
        let test_vals: Vec<i128> = vec![
            m_ntt4_half - 1, m_ntt4_half, m_ntt4_half + 1,
            m_ntt4_half + 1_000_000, m_ntt6_half - 1, -m_ntt4_half, -m_ntt4_half - 1,
        ];
        for x in test_vals {
            if x.unsigned_abs() as i128 > m_ntt6_half { continue; }
            let r6: [u64; 6] = NTT_PRIMES_6.map(|p| x.rem_euclid(p as i128) as u64);
            let recon = garner6_centered_i128(r6);
            assert_eq!(recon, x, "garner6 corridor-edge fail at x={}: got {}", x, recon);
        }
    }

    /// Per-prime spectral residues are CRT-consistent.
    /// If this passes: P5B-9A (true 6p spectral) is mathematically valid.
    /// Each r_i comes from independent NTT — NOT from Garner4 seeding.
    #[test]
    fn per_prime_spectral_residues_crt_consistent() {
        std::thread::Builder::new().stack_size(128*1024*1024).spawn(|| {
            let w0 = Vort3D::taylor_green(100);

            let mut ux_rs: [Box<[[[u64;N];N];N]>; 6] = [
                Box::new([[[0u64;N];N];N]), Box::new([[[0u64;N];N];N]),
                Box::new([[[0u64;N];N];N]), Box::new([[[0u64;N];N];N]),
                Box::new([[[0u64;N];N];N]), Box::new([[[0u64;N];N];N]),
            ];

            for pi in 0..6 {
                let p = NTT_PRIMES_6[pi];
                let i_p = I_MOD_6[pi];
                let mut wx_f = [[[0i64;N];N];N];
                let mut wy_f = [[[0i64;N];N];N];
                let mut wz_f = [[[0i64;N];N];N];
                for i in 0..N { for j in 0..N { for k in 0..N {
                    wx_f[i][j][k] = (w0.wx[i][j][k] % p as i64 + p as i64) % p as i64;
                    wy_f[i][j][k] = (w0.wy[i][j][k] % p as i64 + p as i64) % p as i64;
                    wz_f[i][j][k] = (w0.wz[i][j][k] % p as i64 + p as i64) % p as i64;
                }}}
                let wx_hat = ntt3d_ext(&wx_f, pi);
                let wy_hat = ntt3d_ext(&wy_f, pi);
                let wz_hat = ntt3d_ext(&wz_f, pi);
                let mut ux_hat = Box::new([[[0u64;N];N];N]);
                let s = |kv: i64, x: u64| -> u64 {
                    if kv >= 0 { mulmod((kv as u64) % p, x, p) }
                    else { let v = mulmod((-kv as u64) % p, x, p); submod(0, v, p) }
                };
                for i0 in 0..N { for i1 in 0..N { for i2 in 0..N {
                    let kx = cfreq(i0); let ky = cfreq(i1); let kz = cfreq(i2);
                    let ksq = (kx*kx + ky*ky + kz*kz) as u64;
                    if ksq == 0 { continue; }
                    let inv_ksq = modinv(ksq % p, p);
                    let cx = submod(s(ky, mulmod(wz_hat[i0][i1][i2], inv_ksq, p)),
                                     s(kz, mulmod(wy_hat[i0][i1][i2], inv_ksq, p)), p);
                    ux_hat[i0][i1][i2] = mulmod(i_p, cx, p);
                }}}
                let ux_r = intt3d_ext(&*ux_hat, pi);
                for i in 0..N { for j in 0..N { for k in 0..N {
                    ux_rs[pi][i][j][k] = ux_r[i][j][k];
                }}}
            }

            let m_ntt4_half = (769u128*929*1153*1217)/2;
            let mut mismatches = 0usize;
            let mut max_diff: i128 = 0;

            for i in 0..N { for j in 0..N { for k in 0..N {
                let r4: [u64; 4] = std::array::from_fn(|pi| ux_rs[pi][i][j][k]);
                let r6: [u64; 6] = std::array::from_fn(|pi| ux_rs[pi][i][j][k]);
                let g4 = garner4(r4) as i128;
                if g4.unsigned_abs() as u128 >= m_ntt4_half { continue; }
                let g6 = garner6_centered_i128(r6);
                if g4 != g6 {
                    mismatches += 1;
                    let d = (g4 - g6).abs();
                    if d > max_diff { max_diff = d; }
                    if mismatches <= 2 {
                        println!("Mismatch ({},{},{}): g4={} g6={}", i,j,k,g4,g6);
                    }
                }
            }}}

            println!("CRT consistency: {} mismatches, max_diff={}", mismatches, max_diff);
            assert_eq!(mismatches, 0,
                "Per-prime spectral NOT CRT-consistent: {} mismatches. \
                 P5B-9A requires all 6 primes to give consistent spectral residues.",
                mismatches);
            println!("✓ P5B-9A GATE PASS: per-prime spectral residues are CRT-consistent.");
        }).unwrap().join().unwrap();
    }

    /// Architecture guard: seeding p5/p6 from Garner4 does NOT widen the wall.
    #[test]
    fn p5_p6_seeding_from_garner4_forbidden() {
        let m_ntt4: i128 = 769*929*1153*1217;
        let m_ntt4_half: i128 = m_ntt4 / 2;
        let true_raw: i128 = m_ntt4_half + 1_000_000;  // beyond 4p wall
        let aliased_raw4 = true_raw - m_ntt4;           // what Garner4 returns
        let r4_seeded = aliased_raw4.rem_euclid(1409) as u64;
        let r4_true   = true_raw.rem_euclid(1409) as u64;
        assert_ne!(r4_seeded, r4_true);
        println!("GUARD: seeding p5 from Garner4 gives r4={} (wrong), true r4={}", r4_seeded, r4_true);
        println!("       P5B-9A: r4/r5 must be independent spectral residues.");
    }
}
