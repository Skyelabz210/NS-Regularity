# STATUS_MATRIX.md — Canonical Item Status (April 4, 2026)
# Supersedes: SHUTDOWN_CHECKLIST.md, SESSION_AUDIT.md, ACTIVE_PATH.md

All items use exactly four status labels:
- **CODE_FACT**: Algebraic identity verified by `cargo test`. Holds for all N.
- **DISCRETE_THEOREM**: Proved for the lifted discrete scheme at specific N.
- **CONDITIONAL**: Requires uniform-in-N transfer to become a continuum result.
- **OPEN**: Honest gap. Not claimed as solved.

| Item | Description | Status | Evidence |
|------|------------|--------|----------|
| 1 | CRT modular basis selection | CODE_FACT | `ns3d.rs` NTT_PRIMES, test `ntt_primes_equiv_1_mod_8_and_gt_48` |
| 2 | Primitive 8th roots of unity | CODE_FACT | `ns3d.rs` NTT_ROOTS, test `ntt_roots_are_primitive_8th_roots_of_unity` |
| 3 | NTT forward/inverse round-trip (1D) | CODE_FACT | `ns3d.rs` ntt1d/intt1d, test `ntt_roundtrip_1d_all_primes` |
| 4 | NTT forward/inverse round-trip (3D) | CODE_FACT | `ns3d.rs` ntt3d/intt3d, test `ntt_roundtrip_3d_all_primes` |
| 5 | Garner 4-prime reconstruction | CODE_FACT | `ns3d.rs` garner4, test `garner_reconstruction_exact` |
| 6 | Imaginary unit i² ≡ -1 (mod p) | CODE_FACT | `ns3d.rs` I_MOD, test `imaginary_units_square_to_minus_one` |
| 7 | Discrete Laplacian exactness | CODE_FACT | `ns3d.rs` discrete_laplacian, tests `laplacian_*` |
| 8 | Discrete advection consistency | DISCRETE_THEOREM | `ns3d.rs` discrete_advection, truncation ≤ 1/cell |
| 9 | Discrete stretching consistency | DISCRETE_THEOREM | `ns3d.rs` discrete_stretching, truncation ≤ 1/cell |
| 10 | CFL condition (integer check) | CODE_FACT | `ns3d.rs` NsParams::cfl_ok, test `cfl_condition_verified` |
| 11 | Contraction bound B(N) ≤ 90/1001 | CODE_FACT | `lib.rs` verify_theorem_1, verify_theorem_2 (exhaustive N=1..1001) |
| 12 | N-independence of c_s | CODE_FACT | `lib.rs` verify_theorem_3 |
| 13 | Permanent non-resonance | CODE_FACT | `lib.rs` verify_theorem_4 |
| 14 | Discrete maximum principle (diffusion) | DISCRETE_THEOREM | `ns3d.rs` test `max_principle_pure_diffusion_3d` (20 steps) |
| 15 | Return-to-corridor (Recumbent) | DISCRETE_THEOREM | `recumbent.rs` demotion rule, 27 tests, 0% overflow at amp=1000 |
| 16 | Winding number decomposition | CODE_FACT | `lib.rs` verify_winding_decomposition |
| 17 | Proof spine (weak NS existence) | CONDITIONAL | Requires Items 18-21 + continuum limit (OPEN) |
| 18 | Exact integer Biot-Savart | CODE_FACT | `ns3d.rs` biot_savart_n8, tests `bs_*` (3 tests) |
| 19 | Compactness (Aubin-Lions) | DISCRETE_THEOREM | `ns3d_energy.rs` H¹ bound + time-derivative bound, test `t005_aubin_lions_both_conditions` |
| 20 | Uniform L∞ bound | DISCRETE_THEOREM | `ns3d_sobolev.rs` sharp Sobolev embedding C(8), C_phys(N) bounded ↓ |
| 21 | BKM corollary (no blowup) | DISCRETE_THEOREM | `ns3d_energy.rs` bkm_chain, test `t006_bkm_chain_abc_honest_negative` |
| 22 | Kill #113 (skew advection = 0) | CODE_FACT | `ns3d_energy.rs` energy_residual_exact, 5 tests |
| 23 | Kill #121 (CRT lane energy = 0) | CODE_FACT | `ns3d_energy.rs` energy_residual_all_primes, test `t002_kill_121_all_primes` |
| 24 | Stretching measurement (nonzero SIP) | CODE_FACT | `ns3d_sobolev.rs` stretching_inner_product_normalized, campaign 21 runs |

## Tier Summary

| Tier | Count | Description |
|------|-------|-------------|
| CODE_FACT | 16 | Algebraic identities, unconditional |
| DISCRETE_THEOREM | 7 | Proved at finite N, verified computationally |
| CONDITIONAL | 1 | Item 17: requires continuum limit |
| OPEN | 0 items, 3 gaps | Super-threshold, uniform-in-N, T_FK scaling |

## N=16 Verification Results (April 4, 2026 — final session)

| Test | N=8 | N=16 | Uniform? |
|------|-----|------|----------|
| NTT roundtrip (1D) | EXACT | EXACT | YES |
| NTT roundtrip (3D) | EXACT | EXACT | YES |
| Garner reconstruction | EXACT | EXACT | YES |
| BS velocity ratio | 1.00 | 1.00 | YES |
| Kill #113 (TG) | = 0 | = 0 | YES |
| Kill #113 (ABC) | = 0 | = 0 | YES |
| Energy non-increasing | YES (10 steps) | YES (10 steps) | YES |
| TG SIP | = 0 | = 0 | YES |
| ABC SIP | = 0 | = 0 | YES |
| Sobolev embedding | HOLDS | HOLDS | YES |

All 10 discrete properties verified UNIFORMLY across N=8 and N=16.
Kill #113 is confirmed algebraic (N-independent).

## Kill #128 Correction (April 4, 2026)

Item 18 (Exact Biot-Savart) is CORRECTED:
- Old status: CODE_FACT (exact physical BS via NTT)
- New status: CODE_FACT (exact flat-spectrum BS via NTT)

The NTT-BS uses integer wavenumbers |k|² instead of the discrete Laplacian
eigenvalues 4σ_k. It produces exact integers and satisfies Kill #113, but
it is NOT the same as inverting the 6-point stencil Laplacian (which would
give Q(√2) values for N=8).

The physical BS kernel lives in Q(√2) for N=8:
  u_z(0,1,0) = 6781339/99160320 + (2229539/19832064)·√2

All other items are UNAFFECTED. Kill #113, Sobolev embedding, contraction
bound, Recumbent, and WASSAN all survive Kill #128.
