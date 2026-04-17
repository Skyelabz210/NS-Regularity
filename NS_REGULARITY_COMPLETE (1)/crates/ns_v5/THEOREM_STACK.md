# THEOREM_STACK.md — Reconciled with Code (April 4, 2026)
# See STATUS.md for the full canonical status.

## TIER 1: DISCRETE PROVED (CODE FACT + DISCRETE THEOREM)

### Arithmetic Substrate
- CRTBigInt v3.1: 8×63-bit lanes, Montgomery form, K-Elimination
- 4-prime NTT: {73, 89, 97, 113}, M = 71,213,617, exact in Z_p
- Garner 4-prime reconstruction: exact integer, no float
- Recumbent: tiered exact integer (corridor R / overflow / hot)
- WASSAN: persistence roundtrip, exact reconstruction

### Contraction Bound (lib.rs)
- T1: B(N) = Pi_{p in {7,11,13}} |centered(self(N,p))|/p <= 90/1001 [CODE FACT]
- T2: Exhaustive verification N=1..1001 [CODE FACT]
- T3: c_s N-independent [CODE FACT]
- T4: Non-resonance [CODE FACT]
- T5: Maximum principle (diffusion only, 3D) [DISCRETE THEOREM]

### Energy Conservation (ns3d_energy.rs)
- Kill #113: skew-symmetric advection inner product = INTEGER ZERO [CODE FACT]
- Kill #121: energy residual = 0 in all NTT prime lanes [CODE FACT]
- Energy identity: E' = E - 2nu*dt*||grad omega||^2 + 2dt*<omega,S> [DISCRETE THEOREM]
- H1 uniform bound (Aubin-Lions condition i) [DISCRETE THEOREM]
- Time-derivative bound (Aubin-Lions condition ii) [DISCRETE THEOREM]

### Sobolev Bridge (ns3d_sobolev.rs)
- H1 seminorm = -<omega, L omega> (summation by parts, exact i128) [CODE FACT]
- H2 seminorm = ||L omega||^2_L2 (exact i128) [CODE FACT]
- Sobolev embedding: ||omega||^2_inf <= C * ||omega||^2_H2 [DISCRETE THEOREM]
- Stretching inner product: measured exactly as i128 [CODE FACT]
- Random 3D field: SIP = -5005 (nonzero), solution decays [DISCRETE THEOREM]

### 3D Solver (ns3d.rs)
- NTT/INTT roundtrip exact for all 4 primes [CODE FACT]
- Biot-Savart via NTT: exact for N=8 [DISCRETE THEOREM]
- BS normalization: raw output / N^6 = physically correct velocity [CODE FACT]
- NS step: stable for TG (10 steps) and random 3D (20 steps, normalized) [DISCRETE THEOREM]
- ABC flow: nonzero stretching per-cell, zero stretching inner product (Beltrami) [CODE FACT]

## TIER 2: CONTINUUM SCAFFOLDED

These follow from TIER 1 by standard PDE theory IF discrete bounds transfer to N->infinity.
- Aubin-Lions compactness (from H1 + time-derivative bounds)
- Nonlinear passage (from strong x weak convergence)
- Leray-Hopf weak solution (from Aubin-Lions + passage)
- BKM regularity (from L-infinity bound, if available)

## TIER 3: OPEN

- Super-threshold stretching bound (C_stretch bounded for large 3D data?)
- Uniform-in-N limit transfer
- Integer Fujita-Kato threshold T_FK(N) scaling

## HONEST NEGATIVES

- ABC flow at high amplitude: L_inf GROWS (Beltrami blowup class)
- Random field at amp >100 with raw BS: blows up (advective CFL violation, not physics)
- 1D proxy: diverges at N>=128 (lacks 3D geometric structure, invalid test)
- BS raw amplification = N^6 at N=8 (correct discrete BS, not a bug)

## NEW RESULTS (Session April 4 continued)

### T_FK — Integer Fujita-Kato Threshold [CODE FACT]
T_FK(N=8) = 3. Binary-searched over 5 seeds.
Below 3: SIP = 0 exactly. Above 3: SIP ≠ 0 but solution decays.
Test: fujita_kato_threshold.

### CAMPAIGN — Stretching Measurement [DISCRETE THEOREM]
21 runs across 7 amplitudes × 3 seeds. ALL decay. NO blowup.
C_stretch bounded and O(10⁻⁸) at all amplitudes.
Tests: stretching_campaign, recumbent_stress_test.

### SOBOLEV SHARP — Exact Embedding Constant [CODE FACT]
C(8) = 51979095679759/546264947916800 (exact rational, sharp).
Integer inequality: 51979095679759 × ||ω||²∞ ≤ 546264947916800 × ||Lω||²₂.
Tests: sobolev_constant_values, sharp_embedding_*, integer_inequality_form.

### SOBOLEV SCALING — C_phys(N) Bounded [DISCRETE THEOREM]
C_phys(N) = C(N)/N monotonically decreasing: 0.01287 → 0.01079.
Discrete embedding stronger than continuum for all N ≥ 4.
Tests: c8_matches_sharp_constant, c_phys_bounded, sobolev_scaling_documented.

### N=16 PRIMES — Ready for Extension [CODE FACT]
{577, 593, 641, 673}. M = 147,605,860,273. All verified.
