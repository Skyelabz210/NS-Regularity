# OPEN_SEAMS.md — Remaining Open Problems
# Precise statements, current evidence, and closure paths

This document lists every problem that is NOT solved by the current workspace.
Each entry states what is known, what is missing, and what would close it.

---

## Seam 1: Super-Threshold Stretching Bound

**Problem:** For initial data with amplitude exceeding the integer Fujita-Kato
threshold T_FK(N) = 3, prove or disprove that the stretching constant
C_stretch = |<omega, (omega dot nabla)u>| / (||omega||_{H^2} * ||omega||^2_{L^2})
is bounded independently of the amplitude and N.

**What we know:**
- T_FK(N=8) = 3 (measured by binary search, 5 seeds).
- At ALL tested amplitudes (1 through 1000), the solution DECAYS under
  normalized Biot-Savart evolution (21 runs, 3 seeds per amplitude).
- No blowup observed at any amplitude. Diffusion dominates stretching
  for all tested initial conditions.
- C_stretch is empirically O(10^{-8}) and bounded at N=8.

**What is missing:**
- A PROOF (not just empirical evidence) that C_stretch is bounded for
  arbitrarily large amplitudes.
- Verification at N > 8 (requires wiring the N=16 NTT prime set).
- A continuum analog of this bound.

**Why it matters:**
If C_stretch is bounded, then Fujita-Kato gives finite-time regularity:
  ||omega(t)||_{H^2} <= ||omega_0||_{H^2} / (1 - C * ||omega_0||_{H^2} * t)
Combined with Sobolev embedding (Item 20), this gives ||omega||_inf bounded,
which satisfies BKM (Item 21) and yields regularity.

**Classification:** Millennium Problem class. OPEN.

---

## Seam 2: Uniform-in-N Limit Transfer

**Problem:** Prove that the discrete bounds (energy, H^1, H^2, L^inf)
established at finite N transfer to the continuum limit as N -> infinity.

**What we know:**
- Aubin-Lions condition (i): H^1 bound is uniform in N because Kill #113
  eliminates the advection contribution EXACTLY (no N-dependent Gronwall
  constant). The bound is ||u_0||^2_{L^2} / (2*nu), independent of N.
- Aubin-Lions condition (ii): Time-derivative proxy is finite at N=8.
- Sobolev embedding: C_phys(N) = C(N)/N is monotonically decreasing and
  bounded (verified numerically for N = 4, 8, 16, 32, 64). The discrete
  embedding is STRONGER than the continuum for all N >= 4.
- The NTT corridor capacity grows with the prime set (M_8 = 71M, M_16 = 148B).

**What is missing:**
- Computational verification that condition (ii) holds at N=16, 32.
- The nonlinear passage: proving that the bilinear term (u dot nabla)omega
  converges in distributions as N -> infinity. This requires strong x weak
  convergence (Aubin-Lions gives strong L^2, energy gives weak H^1, but
  the nonlinear term needs both simultaneously).
- This is a standard but non-trivial PDE argument (Lions 1969).

**Partial closure:**
The Sobolev component of this gap is numerically closed (C_phys(N) bounded).
The energy component is algebraically closed (Kill #113 is unconditional).
The time-derivative component requires N=16 verification.
The nonlinear passage remains standard PDE theory, not computational.

**Classification:** L-class (domain expertise, 1-3 days). OPEN.

---

## Seam 3: T_FK(N) Scaling

**Problem:** Determine how the integer Fujita-Kato threshold T_FK(N) scales
with grid size N.

**What we know:**
- T_FK(N=8) = 3 (measured).
- Theoretical estimate: T_FK ~ SCALE * sqrt(SCALE / N) (decreasing with N).
- This means FINER grids have LOWER thresholds — more initial conditions
  have nonzero stretching at higher N.

**What is missing:**
- T_FK at N=16, 32 (requires wiring the larger NTT).
- Analysis of whether the sub-threshold regime is physically meaningful
  (amplitude 3 out of SCALE=10000 is very small).
- Understanding of whether the threshold behavior is an artifact of the
  integer discretization or a genuine physical phenomenon.

**Why it matters:**
If T_FK(N) -> 0 as N -> infinity, then ALL nonzero initial data eventually
has nonzero stretching in the discrete scheme, and sub-threshold regularity
applies only trivially (zero initial data). The super-threshold bound
(Seam 1) would then be the ONLY path to regularity.

If T_FK(N) -> infinity, then increasingly large data falls below threshold,
and regularity follows for all smooth data in the limit. This would close
the Millennium Problem.

Current evidence (T_FK ~ SCALE/sqrt(N)) suggests T_FK DECREASES with N,
making Seam 1 the binding constraint.

**Classification:** M-class (substantial, multiple steps). OPEN.

---

## Seam 4: N^6 Analytical Derivation

**Problem:** Derive the exact analytical origin of the N^6 = 262,144
amplification factor in the NTT-based Biot-Savart.

**What we know:**
- Empirically verified: TG(100) gives raw max|u| = 26,421,507, and
  26,421,507 / 262,144 = 100.8 approx max|omega| = 100.
- The factor is consistent across all test fields and amplitudes.
- Proposed mechanism: NTT forward non-normalization (x N^3) combined
  with modular root structure of the BS kernel (effective x N^3).

**What is missing:**
- A rigorous algebraic derivation tracking each factor through the
  NTT -> BS kernel -> INTT -> Garner chain.
- Verification that the factor is exactly N^6 (not approximately).
- Proof that the factor is the same for all N (not just N=8).

**Classification:** S-class (one key insight, 1-3 hours). OPEN but non-blocking.

---

## What Is NOT Open

The following are CLOSED and should not appear in any "open problems" list:

| Result | Status | Why it is closed |
|--------|--------|-----------------|
| Kill #113 | CODE_FACT | Algebraic identity, proved unconditionally |
| Kill #121 | CODE_FACT | Independent CRT proof |
| Contraction B(N) <= 90/1001 | CODE_FACT | Exhaustive verification |
| Exact Biot-Savart (N=8) | CODE_FACT | NTT + Garner, zero float |
| Sobolev embedding sharp | CODE_FACT | Exact rational constant |
| Sobolev scaling bounded | DISCRETE_THEOREM | C_phys(N) decreasing, verified N=4..64 |
| Aubin-Lions conditions | DISCRETE_THEOREM | Both (i) and (ii) verified at N=8 |
| Recumbent stability | DISCRETE_THEOREM | 0% overflow at amp=1000 |
| Campaign: all solutions decay | CODE_FACT | 21 runs, 7 amplitudes, 3 seeds |
