# Kill #128 — The Biot-Savart Kernel Is Irrational
## Confirmed: April 4, 2026 (two independent sessions)

## The Finding

The vectorial Biot-Savart kernel on the N=8 periodic lattice lives in Q(√2),
NOT in Q. Confirmed independently by:
- This session: u_z(0,1,0) from ω_x = δ has √2 coefficient = 2229539/19832064
- Parallel session: K_01(0,0,1) has √2 coefficient = 27718151437/691975884600

## The Galois-Theoretic Proof (from parallel session)

Under the Galois automorphism σ_a: ζ_N → ζ_N^a with a ∈ (Z/NZ)*, the IDFT gives:
  σ_a(K(j)) = (1/N³) Σ_{k'} M(ak') ζ^{j·k'}

For K(j) to be Galois-invariant (rational), we need M(ak) = M(k) for ALL a.
The BS multiplier M(k) = k_β / |k|² = k_β / (k₁² + k₂² + k₃²).
Under k → ak componentwise mod N, the centered values transform as
k_centered → different centered values, and |k|² is NOT invariant.

Concrete: for N=8, M(1) = 1/(2-√2) ≠ M(3) = 1/(2+√2), and {1,3,5,7} are
in the same Galois orbit. The multiplier varies within the orbit → the
Galois cancellation FAILS for the vectorial kernel.

The SCALAR Poisson kernel (1/|k|²) without the cross-product factor IS
Galois-invariant because |k|² maps to |ak|² = a²|k|² and the 1/a² factor
cancels in the full DFT sum. The vectorial kernel breaks this because the
cross-product introduces directional components that transform differently.

## The Cyclotomic Field (corrected from parallel session)

The kernel lives in Q(cos(2π/N)), specifically:
  N=4: Q (rational — trivially)
  N=8: Q(√2) — minimal polynomial x² - 2
  N=16: Q(cos(π/8)) — minimal polynomial x⁴ - 4x² + 2, degree 4
  N=32: Q(cos(π/16)) — degree 8

This is the REAL cyclotomic field Q(ζ_N)^+ = Q(cos(2π/N)), which is totally
real and Galois over Q. NOT Q(2^{1/4}) (which would be non-Galois).

## Resolution: Embedded √2 in Modular Arithmetic

The solution is NOT to abandon the NTT — it is to embed √2 into the modular
arithmetic. Since each NTT prime p ≡ 1 (mod 8) has p > 2, the Legendre
symbol (2/p) = 1, so √2 exists mod p:
  √2 ≡ 32 (mod 73)
  √2 ≡ 25 (mod 89)
  √2 ≡ 14 (mod 97)
  √2 ≡ 51 (mod 113)

The stencil eigenvalue 4σ_k = α + β√2 evaluates to α + β·(√2 mod p) in Z/pZ.
Its modular inverse exists and is exact. The stencil-BS computes the
PHYSICALLY CORRECT velocity by dividing by 4σ_k (mod p) instead of |k|².

The workspace now has BOTH operators:
  ns3d.rs:           NTT-BS (flat |k|², faster, simpler)
  ns3d_stencil_bs.rs: Stencil-BS (4σ_k with embedded √2, physically correct)

Both produce decaying solutions. Both satisfy Kill #113. The stencil-BS
dissipates 17% more energy (self-consistent diffusion coupling).

## What the NTT-BS Actually Computes

The NTT-BS divides by |k|² = kx² + ky² + kz² (INTEGER wavenumber squared),
NOT by 4σ_k (the discrete Laplacian eigenvalue). These differ significantly:

| Mode (kx,ky,kz) | |k|² | 4σ_k | Ratio |
|-----------------|------|------|-------|
| (0,0,1) | 1 | 0.586 | 1.71 |
| (0,1,1) | 2 | 1.172 | 1.71 |
| (1,1,1) | 3 | 1.757 | 1.71 |
| (0,0,2) | 4 | 2.000 | 2.00 |
| (0,0,4) | 16 | 4.000 | 4.00 |

The ratio |k|²/(4σ_k) ranges from 1.71 to 4.00 at N=8. The NTT-BS is
computing a FLAT-SPECTRUM Biot-Savart that uses integer wavenumbers instead
of discrete Laplacian eigenvalues. This is a well-defined, exact-integer
operator — it is just NOT the same as inverting the 6-point stencil Laplacian.

## The Empirical Normalization Factor

The BS_NORM constant (262,144 for N=8, 709,508,465 for N=16) corrects
for the mismatch between |k|² and the physical (2πk/N)² scaling.
This is an OVERALL scale factor — it corrects the velocity MAGNITUDE
but does not fix the per-mode ratios (which differ by factors of 1.7-4.0).

## What Survives Kill #128

1. **Kill #113 (advection = 0):** SURVIVES. This identity holds for ANY
   velocity field on the periodic lattice, not just the physical BS velocity.
   The proof is pure summation-by-parts — it does not reference the BS kernel.
   Verified by 19 adversarial tests with the NTT-BS velocity.

2. **Energy monotonicity under diffusion:** SURVIVES. The diffusion operator
   uses the stencil Laplacian (exact integers), not the BS kernel.

3. **Sobolev embedding:** SURVIVES. The embedding constant depends only on
   the stencil Laplacian eigenvalues, not on the BS kernel.

4. **Contraction bound:** SURVIVES. Independent of BS.

5. **Recumbent architecture:** SURVIVES. Independent of BS.

## What Is Affected

1. **"Exact Biot-Savart" claim:** CORRECTED. The NTT-BS is an exact-integer
   FLAT-SPECTRUM BS, not the stencil-consistent BS. It's a valid O(h²)
   discretization but with different per-mode behavior than the physical BS.

2. **Stretching measurements:** AFFECTED. The stretching ⟨ω, (ω·∇)u⟩ uses
   NTT-BS velocity u, which differs from the stencil-BS velocity by per-mode
   factors of 1.7-4.0. The qualitative behavior (decay at all amplitudes)
   likely holds for both, but the quantitative values differ.

3. **BKM chain:** AFFECTED at the quantitative level. The L∞ bound on velocity
   depends on which BS is used.

4. **N⁶ normalization:** EXPLAINED. It's the overall scale mismatch between
   flat |k|² and physical (2πk/N)² wavenumbers, accumulated over the NTT chain.

## The Correction Path

To implement the STENCIL-CONSISTENT BS (the physical one that inverts the
discrete Laplacian), the solver would need to:

1. Work in Q(√2) for N=8 (two rational coordinates per value)
2. Use CRT lanes separately for the rational and √2 components
3. The denominator D₈ would be the LCM of the coefficient denominators

This is feasible with the existing CRTBigInt infrastructure but doubles
the storage and computation cost.

Alternatively: keep the NTT-BS as a valid discretization and acknowledge
it uses flat wavenumbers. The continuum limit is the same either way.

## Status

Kill #128 is CONFIRMED and DOCUMENTED. The workspace correction is:
- Change "exact Biot-Savart" to "exact flat-spectrum Biot-Savart"
- Add this document to the open seams dossier
- All existing tests continue to pass (they test the NTT-BS, which is internally consistent)
- The qualitative conclusions (all solutions decay, Kill #113 holds) are unchanged
