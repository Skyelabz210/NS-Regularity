# EXACT-INTEGER NAVIER-STOKES: COMPLETE TECHNICAL PACKET
## All Programs, Variables, Test Outputs, Linkages
### April 4, 2026 — 204 Tests, 0 Failures, 9,517 Lines Rust

---

## MODULE MAP

```
ns_v5/src/
├── lib.rs                 (686 lines)   — Theorem stack: T1-T12, contraction bound
├── ns3d.rs               (1326 lines)   — Core N=8 solver, NTT, BS, all operators
├── ns3d_n16.rs            (582 lines)   — N=16 extension, 4 NTT primes
├── ns3d_n32.rs            (848 lines)   — N=32 extension, 4 NTT primes
├── ns3d_energy.rs         (814 lines)   — Kill #113, energy chain, BKM
├── ns3d_sobolev.rs       (2240 lines)   — Sobolev embedding, sharp constants, campaigns
├── ns3d_adversarial.rs    (458 lines)   — Adversarial tests, random fields
├── ns3d_stencil_bs.rs     (653 lines)   — Q(√2) stencil BS (DIFFERENT OPERATOR)
├── ns3d_wassan.rs         (664 lines)   — WASSAN integration, CRT trajectory
├── recumbent.rs           (751 lines)   — Overflow geometry, tier system
├── wassan.rs              (326 lines)   — Structural persistence
└── cram_ns.rs             (169 lines)   — 1D CRAM regression harness
```

---

## 1. NTT PRIMES AND PARAMETERS

### N=8 (ns3d.rs)

```
Variable          Value                   Purpose
────────────────────────────────────────────────────────────
NTT_PRIMES        [17, 19, 23, 29]        ≡1 mod 8, all >12 (|k|²_max)
NTT_ROOTS         [2, 13, 10, 12]         Primitive 8th roots of unity
I_MOD             [13, 11, 22, 12]        √(-1) mod p
N_INV_3D          [15, 10, 10, 12]        8^{-3} mod p
NTT_MODULUS       214,319                 Product of 4 primes
NS_SCALE          10,000                  Integer scaling factor
```

### N=16 (ns3d_n16.rs)

```
Variable          Value                   Purpose
────────────────────────────────────────────────────────────
NTT_PRIMES        [577, 593, 641, 673]    ≡1 mod 16, all >192 (|k|²_max)
NTT_ROOTS         [171, 384, 601, 464]    Primitive 16th roots
I_MOD             [553, 77, 487, 58]      √(-1) mod p
N_INV_3D          [81, 345, 100, 615]     16^{-3} mod p
NTT_MODULUS       147,605,860,273         Product of 4 primes
BS_NORM           709,508,465             ⚠ EMPIRICALLY CALIBRATED
                                          (vs N^6 = 16,777,216 → 42× discrepancy)
```

**⚠ OPERATOR DISCREPANCY AT N=16:**
The `biot_savart()` function at N=16 divides by `BS_NORM = 709,508,465`.
The calibration test (`bs_calibration`) reports the correct factor is N^6 = 16,777,216.
These differ by 42×. The code uses the empirical value. The test says N^6 is correct.
This needs investigation — which normalization is physically correct?

At N=8: normalization = N^6 = 262,144 (consistent).
At N=32: normalization = N^6 = 1,073,741,824 (consistent).
Only N=16 has the anomalous BS_NORM constant.

### N=32 (ns3d_n32.rs)

```
Variable          Value                   Purpose
────────────────────────────────────────────────────────────
NTT_PRIMES        [769, 929, 1153, 1217]  ≡1 mod 32, all >768 (|k|²_max)
NTT_ROOTS         [251, 701, 286, 540]    Primitive 32nd roots
I_MOD             [707, 605, 140, 78]     √(-1) mod p
N_INV_3D          [18, 694, 617, 1110]    32^{-3} mod p
NTT_MODULUS       1,002,448,197,601       Product of 4 primes
Garner C01        749                     769^{-1} mod 929
Garner C012       839                     (769×929)^{-1} mod 1153
Garner C0123      720                     (769×929×1153)^{-1} mod 1217
```

---

## 2. DISCRETE OPERATORS

### Laplacian (all N)

```
Function:    discrete_laplacian(f: &Field3D) -> Field3D
Formula:     Δ_h f[i,j,k] = f[i+1] + f[i-1] + f[j+1] + f[j-1] + f[k+1] + f[k-1] - 6f[i,j,k]
Type:        6-point stencil, periodic BC
Arithmetic:  i128 (intermediate), i64 (output, clamped)
Eigenvalue:  μ_k = -4σ_k where σ_k = sin²(πkx/N) + sin²(πky/N) + sin²(πkz/N)
NOTE:        This is the STENCIL eigenvalue, NOT |k|² (see Section 5)
```

### Advection (all N)

```
Function:    discrete_advection_skew(u: &Vel3D, wl: &Field3D) -> Field3D
Formula:     A_skew = (u·∇f + ∇·(uf)) / 2  (skew-symmetric form)
Variables:   u_here, u_plus, u_minus, f_plus, f_minus (per direction d=0,1,2)
Arithmetic:  i128 intermediate, divide by denom = 4 * NS_SCALE, multiply by N
Property:    Σ ω · A_skew(u, ω) = 0 (INTEGER ZERO — Kill #113)
```

### Stretching (3D only)

```
Function:    discrete_stretching(w: &Vort3D, ul: &Field3D) -> Field3D
Formula:     S_c = Σ_d ω_d × (u_c[x+d] - u_c[x-d]) / (2h)
Variables:   w.wx, w.wy, w.wz (vorticity), ul (velocity component)
Arithmetic:  i128 intermediate, divide by d = 2 * NS_SCALE, multiply by N
Degree:      2 (quadratic in solution — verified: F(λω)/F(ω) = λ² ± 0.1%)
```

### Biot-Savart (spectral, per N)

```
Function:    biot_savart(w: &Vort3D) -> Vel3D
Method:      For each NTT prime p:
               1. wx_hat = NTT3D(wx) mod p
               2. ux_hat = i_p × (ky×wz_hat - kz×wy_hat) × modinv(|k|², p)
               3. ux = INTT3D(ux_hat) mod p
             Garner reconstruction → exact integer
Division:    modinv(|k|², p) — FLAT |k|², not stencil eigenvalue 4σ_k
Output:      Exact integer velocity (divided by normalization factor)
```

### NS Step (all N)

```
Function:    ns_step(w: &Vort3D, p: &NsParams) -> Vort3D
Formula:     ω' = ω + dt × (ν×Δω/S² - A_skew(u,ω) + S(ω,u)) / N²
Variables:
  p.nu_scaled    viscosity × SCALE²
  p.dt_latt      timestep (CFL-limited)
  NS_SCALE       10,000
  N              grid size (8, 16, 32)
Arithmetic:  i128 intermediate, i64 output (clamped)
```

---

## 3. STENCIL BS vs NTT BS (THE TWO OPERATORS)

```
                    NTT BS (ns3d.rs)           Stencil BS (ns3d_stencil_bs.rs)
────────────────────────────────────────────────────────────────────────────────
Divides by:         |k|²                       4σ_k = 4(sin²+sin²+sin²)
Field:              ℤ (exact integer)           ℚ(√2) for N=8
Output type:        i64                         QVel3D = (rat: Vel3D, sqrt2: Vel3D)
Denominator:        none (modinv in ℤ/pℤ)       D₈ = 3,098,760
Irrational:         NO                          YES (√2 coefficients)
Energy neutral:     YES (Kill #113)             YES (Kill #113, independent proof)
O(h²) consistent:   YES                         YES
Difference:         Per-mode velocity scaling differs by factors 1.7×–4.0× at N=8
```

**This is the operator discovery:** Two valid O(h²) discretizations of the same BS law
that produce DIFFERENT velocity fields. The NTT BS uses flat |k|² (the Fourier
wavenumber). The stencil BS uses 4σ_k (the discrete Laplacian eigenvalue). They
converge to the same continuum limit but differ at finite N.

Kill #113 holds for BOTH operators independently.

---

## 4. CRAM DIAGNOSTICS (from this session, N=32)

### V7 — Ring Homomorphism

```
All 32,768 grid points × 6 Safe Basis primes = PASS
Zero mismatches for both Taylor-Green and ABC initial conditions
```

### V8 — Energy Neutrality (Kill #113)

```
energy_residual_exact = 0 (integer zero)
For both TG and ABC at N=32
```

### K_vort — Vorticity Winding Number

```
Taylor-Green (amplitude 1000, 10 steps):
  k(7,11)_max = 10    k(7,13)_max = 12    k(11,13)_max = 12
  CONSTANT across all steps even as max|ω| grows 1000 → 7549

ABC (amplitude 1000, 10 steps):
  k(7,11)_max = 10    k(7,13)_max = 12    k(11,13)_max = 12
  CONSTANT across all steps even as max|ω| grows 2000 → 118417
```

### SCR — Stretching Concentration Ratio

```
Taylor-Green:  SCR = 0.75–0.78 (slightly below expected 0.78–0.86)
ABC:           SCR = 0.764 ± 0.001 (remarkably stable)

Per-lane distribution (ABC, stable across all steps):
  Lane 2:   5.0%   (parking)
  Lane 3:   6.6%   (fabric)
  Lane 5:  11.9%   (measurement)
  Lane 7:  17.1%   (bridge)      ← EXPECTED: 15-22% ✓
  Lane 11: 27.2%   (shadow)      ← EXPECTED: 19-36% ✓
  Lane 13: 32.2%   (boundary)    ← EXPECTED: 28-43% ✓
```

### B(N) — Contraction Factor

```
p=7:  c_diff mod 7 = 0, self_coeff = 1, centered = 1, |cent|/p = 0.1429
p=11: c_diff mod 11 = 5, self_coeff = 4, centered = 4, |cent|/p = 0.3636
p=13: c_diff mod 13 = 3, self_coeff = 9, centered = -4, |cent|/p = 0.3077
B(N=32) = 0.143 × 0.364 × 0.308 = 0.016  (≤ 90/1001 = 0.090 ✓)
```

### SIP — Stretching Inner Product

```
Taylor-Green: SIP = 0 at t=0 (structural — 2D flow, no stretching)
              SIP oscillates ±34B over 10 steps

ABC:          SIP = 0 at t=0 (Beltrami symmetry)
              SIP nonzero at t=1 (symmetry breaks)
              |SIP| grows to 55B by step 9
              max|ω| grows from 2000 → 118417
```

### Per-Lane Residue Tracking (ABC, grid point [4,4,4])

```
Step    wx_int    r2  r3  r5  r7  r11  r13   k(7,11)  k(7,13)
  0      1410      0   0   0   3    2    6       3        6
  1      1402      0   1   2   2    5   11       2        5
  2      2500      0   1   0   1    3    4       5        6
  3      1853      1   2   3   5    5    7       0        4
  4      3824      0   2   4   2    7    2       7        0
  5     12569      1   2   4   4    7   11       2        1
  6      2422      0   1   2   0    2    4       5        8
  7    -13535      1   1   0   3    6   11       2        3
  8    -58590      0   0   0   0    7    1       1        2
  9     76311      1   0   1   4    4    1       0        7
```

---

## 5. COMPLETE TEST LIST (204 tests, 0 failures)

### lib.rs (14 tests) — Theorem stack
```
theorem_1_algebraic_bound          T1: 90/1001 identity
theorem_1_exact_rational           T1: exact rational arithmetic
theorem_1_numerical                T1: 90 < 1001
theorem_2_exhaustive               T2: N=1..1001 all bounded
theorem_3_stretching_constant      T3: c_s constant across N
theorem_4_non_resonance            T4: c_s ≠ 6⁻¹ on all lanes
theorem_5_constant_field           T5: identity on constants
theorem_5_max_principle_preserves  T5: convex combination
resolution_ladder_all_contractive  B(N) < 1 for 15 resolutions
cycle_product_bounded              Product = 2.58e-113
proved_steps_regularity            Chain verification
self_coefficient_has_zero_step     Nilpotent detection
test_decompose_roundtrip           CRT decompose/reconstruct
test_integer_maximum_principle     Discrete max principle
test_stretching_bounded            Stretching equilibrium < M
test_winding_maximum_principle     Winding max principle
overshoot_scaling                  Overshoot parameter test
```

### ns3d.rs (14 tests) — Core N=8 solver
```
ntt_primes_equiv_1_mod_8_and_gt_48
ntt_roots_are_primitive_8th_roots_of_unity
ntt_matrix_entries_match_omega_powers
ntt_roundtrip_1d_all_primes
ntt_roundtrip_3d_all_primes
imaginary_units_square_to_minus_one
n_inv_3d_is_correct
garner_reconstruction_exact
garner_constants_verified
bs_zero_gives_zero
bs_all_primes_consistent
bs_corridor_holds_for_taylor_green
cfl_condition_verified
claim_boundary_documented
laplacian_constant_field_is_zero
laplacian_checkerboard_gives_known_result
enstrophy_non_increasing_pure_diffusion
max_principle_pure_diffusion_3d
ns_abc_flow_has_nonzero_stretching
ns_full_step_taylor_green_10_steps
```

### ns3d_n16.rs (9 tests)
```
ntt_roundtrip_1d_all_primes
ntt_roundtrip_3d
garner_reconstruction
kill_113_n16
bs_tg_velocity_normalized
energy_non_increasing_n16
stretching_ip_n16
sobolev_c16_check
bs_calibration                    ← Reports BS_NORM discrepancy
```

### ns3d_n32.rs (22 tests)
```
ntt_roundtrip_1d_all_primes_n32
ntt_roundtrip_3d_n32
garner_reconstruction_n32
kill_113_n32
stretching_nonzero_abc_n32
bs_calibration_n32
stretching_components_abc_n32
stretching_growth_over_time_n32
enstrophy_tg_vs_abc_n32
norm_equivalence_d026
operator_truncation_error
sobolev_embedding_n32
dkam_resonance_n32
padic_non_expansion_d024
blowup_criterion_bkm
energy_spectrum_n32
d024_recheck_full_vs_per_lane
what_the_lanes_actually_show
full_cram_diagnostic_suite        ← V7, K_vort, SCR, V8, B(N), SIP
```

### ns3d_energy.rs (15 tests) — Energy chain
```
t001_skew_adv_computes
t001_skew_adv_zero_velocity
t002_kill_113_taylor_green
t002_kill_113_abc_flow
t002_kill_113_zero
t002_kill_121_all_primes
t003_energy_non_increasing_skew
t003_ns_step_skew_finite
t004_uniform_h1_bound_verified
t005_aubin_lions_both_conditions
t005_time_derivative_bound_finite
t006_bkm_chain_taylor_green
t006_bkm_chain_abc_honest_negative
t007_final_24_24_report
t008_kill_113_cumulative_zero_skew_vs_onesided
```

### ns3d_sobolev.rs (30 tests) — Sobolev/regularity
```
t001_h1_seminorm_nonneg, t001_h1_seminorm_zero_field
t001_h1_seminorm_summation_by_parts_identity
t001_h2_seminorm_nonneg, t001_h2_seminorm_zero_field
t001_sobolev_hierarchy
t002_sobolev_ratio_nonneg, t002_sobolev_ratio_bounded_by_n3
t002_sobolev_embedding_linfty_leq_h2
t003_h2_non_increasing_taylor_green_20steps
t003_h2_non_increasing_abc_flow_20steps
t003_small_data_condition_satisfied
t004_corridor_closure_taylor_green
t004_corridor_closure_abc_flow
t005_k_elim_roundtrip, t005_winding_zero_field
t005_winding_ntt_vs_disc_bounded
t006_bkm_integral_finite, t006_l2t_h1_bound_finite
t006_closure_status_documented
bs_normalization_factor
seam_cfl_corrected, seam_final_measurement
seam_normalized_bs, seam_random_field_evolution
seam_smooth_3d_perturbation, seam_substep_random
diagnostic_passing_tg, diagnostic_velocity_magnitude
stretching_campaign, fujita_kato_threshold
recumbent_stress_test
sharp_sobolev: constant_values, taylor_green, abc, random, all_unit_vectors
integer_inequality_form
sobolev_constant_values, sobolev_scaling_documented
c8_matches_sharp_constant, c_phys_bounded
stretching_cell_level
```

### ns3d_adversarial.rs (14 tests)
```
kill_113: tg_multi_amplitude, abc_multi_amplitude, checkerboard,
          random_10_seeds, all_deltas, during_evolution
kill_121: all_primes_multi_field
h1_bound: h1_bound_50_steps, aubin_lions_50_steps
one_sided: skew_energy_non_increasing, skew_vs_standard_energy_comparison
stretching: campaign_regression_all_decay, stretching_constant_bounded_evolution
sobolev: sharp_sobolev_adversarial_fields, sharp_sobolev_during_evolution
recumbent: tier_transitions_preserve_value, reconstruction_invariant_wide_range
crt_integrity: garner_roundtrip_exhaustive_boundary, ntt_roundtrip_adversarial
```

### ns3d_stencil_bs.rs (7 tests) — The OTHER Biot-Savart
```
sqrt2_mod_primes
kill_113_stencil_bs
stencil_bs_tg_velocity
stencil_vs_ntt_comparison          ← Compares the TWO operators
stencil_vs_ntt_random_field
energy_comparison
ntt_vs_stencil_evolution
```

### ns3d_wassan.rs (14 tests)
```
t001_roundtrip: zero, taylor_green, abc
t002_wassan_step_matches_direct_step
t002_wassan_10step_trajectory_matches
t003_h2_crt_zero_field, t003_h2_norm_crt_matches_i128
t004_small_tg_stays_in_corridor
t004_overflow_steps: increments_correctly, zero_small_data_10steps
t005_checkpoint_restore_replay
t006_bkm_integral_crt_finite
t006_gronwall: h2_crt_positive_finite, terms_non_increasing
t007_run_trajectory_coherent
```

### recumbent.rs (24 tests), wassan.rs (7 tests), cram_ns.rs (4 tests)
```
(Overflow geometry, persistence, 1D regression — see source for full list)
```

---

## 6. OPEN QUESTIONS FROM THIS DATA

1. **BS_NORM at N=16:** Why is it 709,508,465 instead of N^6 = 16,777,216?
   The calibration test says N^6 is correct. The code uses the wrong value.
   Need to check: is the velocity at N=16 scaled correctly?

2. **K_vort saturation:** K_vort max = 10 on (7,11) and 12 on (7,13)/(11,13)
   for ALL steps. This is p-1 for each pair. Is this a ceiling or an artifact?

3. **SCR below expected range:** TG gives 0.75 vs expected 0.78. Is N=32
   shifting the concentration slightly vs the N=4..16 measurements?

4. **Stencil BS vs NTT BS:** Which operator is physically correct? The NTT BS
   uses flat |k|² (exact integers). The stencil BS uses 4σ_k (needs Q(√2)).
   Both are O(h²). They differ per-mode by factors 1.7×–4.0×. Which one
   should the regularity argument use?

---

## 7. BUILD & RUN

```bash
cd ns_v5
RUST_MIN_STACK=67108864 cargo test --release
# Expected: 204 passed, 0 failed, ~187 seconds
```

All arithmetic is exact integer. Zero floating point in production paths.
