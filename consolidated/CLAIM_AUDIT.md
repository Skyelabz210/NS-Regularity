# NS proof claim audit and dependency boundaries

Date: 2026-10-14. This is an evidence classification, not a deletion or rewriting of historical theorem manuscripts.

## Established algebraic facts or well-supported finite-discrete identities

- CRT/K-Elimination: quotient/winding recovery is exact under its modulus/coprimality/range assumptions. This establishes representation correctness, not a PDE bound.
- On a periodic finite grid, a correctly skew-symmetric advection operator has `<w,A_skew(w)> = 0`; current NS corpus calls this Kill #113. The identity is useful and requires verification against the *actual production operator*.
- A diffusion stencil with nonnegative weights summing to one obeys a discrete maximum principle, provided CFL and exact-division hypotheses hold.
- Residue representation is lossless only under the carrier range/reconstruction assumptions. A wider basis may remove overflow without changing the dynamics.
- The discrete `H²` energy identity must include the discrete-time positive correction `||Δδw||²`; the exact signed stretching/commutator inequality is a separate obligation.
- Finite NTT forward/inverse identities and finite-grid spectral kernels are arithmetically testable, including the `Q(√2)` correction of the N=8 stencil kernel.

## Formal Lean scope

The canonical `lean/PROOF_STATUS.md` documents an active, compiled finite-grid proof stack. It reports a conditional full-NS trajectory theorem requiring `h_step` (per-step L∞ control) and lists remaining Biot–Savart axioms. The `IsSmooth` predicate is defined for the discrete trajectory by existence of a uniform L∞ bound, and is not the standard analytic `C∞` regularity of continuum Navier–Stokes.

The exact axiom dependencies must be checked using `#print axioms` on the theorem itself, not by counting `sorry` strings. A proof of `P → Q` does not prove `Q` until `P` is established.

## Claims requiring corrected scope

1. **Prime-pair unit carry.** `floor((x+1)/M)-floor(x/M) ∈ {0,1}` is correct for an integer increment of one. An NS evolution step may change the integer field by much more than one and spatial concentration is not controlled by this digit-carry formula. A unit digit event does not imply an N-independent vorticity norm bound.
2. **Skew energy vs full vorticity dynamics.** Vanishing skew-advection inner product does not control positive vortex stretching. With explicit Euler, even a skew operator alone contributes the positive term `dt² ||A w||²` to next-step energy. Deduce a global inequality only from the complete step identity.
3. **D-030 unconditional L∞.** The uploaded complete closure ledger explicitly retires unconditional `||ω||∞ ≤ (p−1)+√E(0)` as a general-data conclusion. Reuse only the genuinely established conditional form with correctly scaled physical energy.
4. **N⁶ exhaustion.** Any assertion that arbitrary physical large data becomes small solely under mesh refinement requires a mesh-consistent norm comparison, discretization consistency, and a resolution-independent small-data threshold. Raw integer norm scalings cannot substitute for that transfer.
5. **D-024 / p-adic contraction.** Per-lane norm reductions may coexist with an increasing reconstructed global norm. The uploaded test output records TG and ABC counterexamples. Local non-expansion alone cannot be promoted to physical-space contraction.
6. **D-RCRT/adelic language.** An increasing CRT modulus can provide compatible arithmetic encodings, but identifying the CRT inverse limit with the physical `T³` requires a proved embedding, topology, norm and PDE operator convergence. The Pontryagin dual of `widehat Z` is `Q/Z`, not `Z`; earlier manuscripts treating these duals as identical need a corrected analytic map.
7. **Infinite-domain passage.** Viscous NS is parabolic and does not enjoy a finite propagation-speed domain-of-dependence principle. Periodization from R³ to T³ requires a separate localization/limit argument.
8. **Biot–Savart.** Discrete curl inversion and physically normalized `H²` / `L∞` estimates require the zero-mode/divergence hypotheses and consistent finite difference operators.
9. **Finite tests.** A cargo test that passes because it correctly detects a violating example is a successful test of the checker, not positive evidence for a universal bound.

## Current analytic endpoint

The outstanding general-data closure must produce a uniform-in-resolution, correctly physically normalized estimate strong enough to prevent finite-time blowup, with a consistent passage to the 3D continuum equations. Candidate routes are an a priori high-norm estimate, an integrable vorticity supremum in time, or a compatible Prodi–Serrin class. The needed theorem must account for nonlinear stretching for arbitrary smooth divergence-free data.

Record hypothesis, constants, N-dependence, arithmetic corridor and proof status for every new candidate result. Historical 'CLOSED' labels remain searchable as their original authors' claims but do not supersede this gate-by-gate audit.

## Additional axiom audit (2026-10-14): curl-range soundness

A second inconsistency was identified in `lean/NSRegularity/NS/BiotSavart.lean` after the previous divergence-precondition repair. For every periodic vector field `u`, `Σ curl(u)_x = 0` by summation by parts. The constant field `ω=(1,0,0)` satisfies `div ω=0` but `Σ ω_x=N³≠0`, contradicting the older `curl(BS ω)=ω` axiom. For even N a checkerboard field has zero mean and divergence but lies in the orthogonal complement of the centered curl's range. The axiom has therefore been restricted to fields admitting an actual divergence-free preimage. The independent regression probe `scripts/verify_biot_savart_domain.py` confirms the finite-grid consequences using exact integer arithmetic. Constructive NTT and trajectory-invariance obligations remain.
