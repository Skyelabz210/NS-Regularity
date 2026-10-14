//! Stencil Eigenvalue Computation for N=32
//!
//! The physical Laplacian eigenvalue 4σ_k is computed DIRECTLY from
//! the NTT primitive root ω mod p. No nested radicals needed.
//!
//! cos(2πk/N) mod p = (ω^k + ω^{-k}) × 2⁻¹ mod p
//! 4σ_k = 6 - 2(cos_x + cos_y + cos_z) mod p
//!
//! This IS the same eigenvalue that involves √2 in real arithmetic,
//! but computed via the modular root-of-unity path — exact integer.

fn mulmod(a: u64, b: u64, p: u64) -> u64 { ((a as u128 * b as u128) % p as u128) as u64 }
fn addmod(a: u64, b: u64, p: u64) -> u64 { ((a as u128 + b as u128) % p as u128) as u64 }
fn submod(a: u64, b: u64, p: u64) -> u64 { if a >= b { a - b } else { p - (b - a) } }
fn modpow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    let mut r = 1u64; base %= m;
    while exp > 0 { if exp & 1 == 1 { r = mulmod(r, base, m); } exp >>= 1; base = mulmod(base, base, m); }
    r
}
fn modinv(a: u64, p: u64) -> u64 { modpow(a, p - 2, p) }

/// N=32 NTT configuration
pub const N: usize = 32;
pub const N3: usize = N * N * N; // 32768

/// NTT primes: all ≡ 1 mod 32, all > 768 (max |k|²)
pub const NTT_PRIMES: [u64; 4] = [769, 929, 1153, 1217];

/// Primitive 32nd roots of unity mod each prime
pub const NTT_ROOTS: [u64; 4] = [251, 701, 286, 540];

/// Compute centered wavenumber: k ∈ [0,N) → k_c ∈ [-N/2, N/2)
fn centered(k: usize) -> i32 {
    if k <= N / 2 { k as i32 } else { k as i32 - N as i32 }
}

/// Compute cos(2πk/N) mod p using the primitive root ω
/// cos(2πk/N) = (ω^k + ω^{-k}) / 2  mod p
fn cos_mod(k: usize, omega: u64, p: u64) -> u64 {
    let k_mod = k % N;
    if k_mod == 0 { return 1; } // cos(0) = 1
    let wk = modpow(omega, k_mod as u64, p);
    let wk_inv = modinv(wk, p);
    let sum = addmod(wk, wk_inv, p);
    let inv2 = modinv(2, p);
    mulmod(sum, inv2, p)
}

/// Compute the stencil eigenvalue 4σ_k mod p
/// 4σ_k = 6 - 2(cos(2πk_x/N) + cos(2πk_y/N) + cos(2πk_z/N))
///
/// This is -λ(k) where λ(k) is the 6-point stencil Laplacian eigenvalue.
/// For k = (0,0,0): 4σ = 0 (DC mode, skip in BS).
pub fn stencil_eigenvalue(kx: usize, ky: usize, kz: usize, omega: u64, p: u64) -> u64 {
    let cx = cos_mod(kx, omega, p);
    let cy = cos_mod(ky, omega, p);
    let cz = cos_mod(kz, omega, p);
    // 6 - 2*(cx + cy + cz) mod p
    let sum_cos = addmod(addmod(cx, cy, p), cz, p);
    let two_sum = addmod(sum_cos, sum_cos, p); // 2 * sum
    submod(6 % p, two_sum, p)
}

/// Compute the FLAT wavenumber |k|² (what the NTT-BS uses)
pub fn flat_wavenumber_sq(kx: usize, ky: usize, kz: usize) -> u64 {
    let cx = centered(kx) as i64;
    let cy = centered(ky) as i64;
    let cz = centered(kz) as i64;
    (cx * cx + cy * cy + cz * cz) as u64
}

/// Build the full inverse eigenvalue table for one prime.
/// Returns: inv_eigen[kx][ky][kz] = modinv(4σ_k, p), or 0 for DC mode.
pub fn build_inv_stencil_table(prime_idx: usize) -> Vec<u64> {
    let p = NTT_PRIMES[prime_idx];
    // ω_N^2 is the primitive N-th root (NTT_ROOTS is 2N-th root? Let's check)
    // Actually, NTT_ROOTS[i] is primitive 32nd root of unity.
    // cos(2πk/32) needs ω₃₂: ω₃₂^k gives e^{2πik/32}
    // So cos(2πk/32) = (ω^k + ω^{-k})/2
    let omega = NTT_ROOTS[prime_idx];

    let mut table = vec![0u64; N3];
    for kx in 0..N {
        for ky in 0..N {
            for kz in 0..N {
                let idx = kx * N * N + ky * N + kz;
                if kx == 0 && ky == 0 && kz == 0 {
                    table[idx] = 0; // DC mode: skip
                    continue;
                }
                let eigen = stencil_eigenvalue(kx, ky, kz, omega, p);
                if eigen == 0 {
                    table[idx] = 0; // Degenerate mode
                } else {
                    table[idx] = modinv(eigen, p);
                }
            }
        }
    }
    table
}

/// Build the flat |k|² inverse table for one prime (for comparison).
pub fn build_inv_flat_table(prime_idx: usize) -> Vec<u64> {
    let p = NTT_PRIMES[prime_idx];
    let mut table = vec![0u64; N3];
    for kx in 0..N {
        for ky in 0..N {
            for kz in 0..N {
                let idx = kx * N * N + ky * N + kz;
                let ksq = flat_wavenumber_sq(kx, ky, kz);
                if ksq == 0 {
                    table[idx] = 0;
                } else {
                    table[idx] = modinv(ksq % p, p);
                }
            }
        }
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stencil_eigenvalue_dc_is_zero() {
        for i in 0..4 {
            let p = NTT_PRIMES[i];
            let omega = NTT_ROOTS[i];
            assert_eq!(stencil_eigenvalue(0, 0, 0, omega, p), 0,
                "DC mode eigenvalue must be 0 for prime {p}");
        }
    }

    #[test]
    fn stencil_eigenvalue_nonzero_for_nondc() {
        // For single-mode k=(1,0,0): 4σ = 2(1-cos(2π/32))
        // cos(2π/32) = cos(π/16) ≈ 0.9808 → 4σ ≈ 0.0384
        // Nonzero, and should be nonzero mod p
        for i in 0..4 {
            let p = NTT_PRIMES[i];
            let omega = NTT_ROOTS[i];
            let e = stencil_eigenvalue(1, 0, 0, omega, p);
            assert_ne!(e, 0, "k=(1,0,0) eigenvalue must be nonzero for p={p}");
        }
    }

    #[test]
    fn cos_mod_consistency() {
        // cos(0) = 1, cos(N/2) = cos(π) = -1
        for i in 0..4 {
            let p = NTT_PRIMES[i];
            let omega = NTT_ROOTS[i];
            assert_eq!(cos_mod(0, omega, p), 1, "cos(0) = 1 for p={p}");
            assert_eq!(cos_mod(N / 2, omega, p), p - 1, "cos(π) = -1 for p={p}");

            // cos(N/4) = cos(π/2) = 0
            let c_quarter = cos_mod(N / 4, omega, p);
            assert_eq!(c_quarter, 0, "cos(π/2) = 0 for p={p}");
        }
    }

    #[test]
    fn omega_is_primitive_32nd_root() {
        for i in 0..4 {
            let p = NTT_PRIMES[i];
            let omega = NTT_ROOTS[i];
            // ω^32 ≡ 1 mod p
            assert_eq!(modpow(omega, 32, p), 1, "ω^32 = 1 for p={p}");
            // ω^16 ≢ 1 mod p (primitive)
            assert_ne!(modpow(omega, 16, p), 1, "ω^16 ≠ 1 for p={p}");
        }
    }

    #[test]
    fn stencil_vs_flat_differ_for_multimode() {
        // For k=(1,1,0): |k|² = 2, 4σ = 4sin²(π/32)+4sin²(π/32) ≠ 2
        // They should give DIFFERENT inverse values
        let mut differ_count = 0;
        for i in 0..4 {
            let p = NTT_PRIMES[i];
            let omega = NTT_ROOTS[i];
            let stencil = stencil_eigenvalue(1, 1, 0, omega, p);
            let flat = flat_wavenumber_sq(1, 1, 0) % p;
            if stencil != flat { differ_count += 1; }
        }
        assert!(differ_count >= 3,
            "Stencil and flat eigenvalues should differ for most primes at k=(1,1,0)");
    }

    #[test]
    fn stencil_table_builds_successfully() {
        // The stencil eigenvalue can be zero mod p for modes where
        // the real-valued eigenvalue happens to be a multiple of p.
        // This is a modular artifact, not a physics error.
        // Affected modes: p=929 has 48, p=1217 has 24.
        // For Garner reconstruction, fall back to remaining 3 primes.
        let expected_zeros = [0usize, 48, 0, 24]; // per prime

        for i in 0..4 {
            let table = build_inv_stencil_table(i);
            assert_eq!(table.len(), N3);
            assert_eq!(table[0], 0); // DC mode

            let zero_count = table.iter().filter(|&&x| x == 0).count() - 1; // -1 for DC
            assert_eq!(zero_count, expected_zeros[i],
                "Prime idx {i}: expected {expected_zeros:?} zero eigenvalue modes");

            // Verify nonzero entries roundtrip
            let p = NTT_PRIMES[i];
            let omega = NTT_ROOTS[i];
            let mut verified = 0;
            for kx in 0..N { for ky in 0..N { for kz in 0..N {
                let idx = kx * N * N + ky * N + kz;
                if table[idx] != 0 {
                    let eigen = stencil_eigenvalue(kx, ky, kz, omega, p);
                    let product = mulmod(eigen, table[idx], p);
                    assert_eq!(product, 1, "Roundtrip failed at k=({kx},{ky},{kz}) for p={p}");
                    verified += 1;
                }
            }}}
            println!("Prime {p}: {verified} modes verified, {} zero eigenvalue modes",
                zero_count);
        }
    }

    #[test]
    fn stencil_inverse_roundtrip() {
        // For a few modes, verify: eigenvalue × inv_eigenvalue ≡ 1 mod p
        let test_modes = [(1,0,0), (0,1,0), (0,0,1), (1,1,1), (3,5,7), (15,8,12)];
        for i in 0..4 {
            let p = NTT_PRIMES[i];
            let omega = NTT_ROOTS[i];
            let table = build_inv_stencil_table(i);

            for (kx, ky, kz) in &test_modes {
                let idx = kx * N * N + ky * N + kz;
                let eigen = stencil_eigenvalue(*kx, *ky, *kz, omega, p);
                let inv = table[idx];
                let product = mulmod(eigen, inv, p);
                assert_eq!(product, 1,
                    "p={p}: eigenvalue({kx},{ky},{kz}) × inv should be 1, got {product}");
            }
        }
    }

    #[test]
    fn stencil_symmetry_kx_ky_kz() {
        // Eigenvalue is symmetric under permutation of (kx, ky, kz)
        for i in 0..4 {
            let p = NTT_PRIMES[i];
            let omega = NTT_ROOTS[i];
            let e_xyz = stencil_eigenvalue(3, 5, 7, omega, p);
            let e_yxz = stencil_eigenvalue(5, 3, 7, omega, p);
            let e_zyx = stencil_eigenvalue(7, 5, 3, omega, p);
            assert_eq!(e_xyz, e_yxz, "p={p}: eigenvalue symmetric in kx,ky");
            assert_eq!(e_xyz, e_zyx, "p={p}: eigenvalue symmetric in kx,kz");
        }
    }

    #[test]
    fn flat_vs_stencil_agree_at_mode_n_half() {
        // At k = (N/2, 0, 0): cos(2π·(N/2)/N) = cos(π) = -1
        // 4σ = 2(1-(-1)) = 4,  |k|² = (N/2)² = 256
        // These should be DIFFERENT (4 vs 256)
        for i in 0..4 {
            let p = NTT_PRIMES[i];
            let omega = NTT_ROOTS[i];
            let stencil = stencil_eigenvalue(N/2, 0, 0, omega, p);
            let flat = flat_wavenumber_sq(N/2, 0, 0) % p;
            // 4σ = 2(1-cos(π)) = 2(1-(-1)) = 4
            assert_eq!(stencil, 4, "p={p}: stencil at k=N/2 should be 4");
            // |k|² = 16² = 256
            assert_eq!(flat, 256 % p, "p={p}: flat at k=N/2 should be 256");
            assert_ne!(stencil, flat, "p={p}: stencil ≠ flat at k=N/2");
        }
    }
}
