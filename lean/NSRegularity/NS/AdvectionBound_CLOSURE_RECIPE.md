# Full-step L∞ gate — closure recipe

Last updated: 2026-06-12 session.
Routed from `lean/PROOF_STATUS.md` ("non-trivial full-step L∞ closure").

This document captures the structural decomposition of the open
full-step L∞ gate after the work on PR #11. It is a **routing**
document — it does **not** close the gate. It records exactly what
remains to be proved, in the form a future attempt can pick up directly.

## Context

The Lean parametric NS skeleton (`NSRegularity.NS.Trajectory`)
takes a hypothesis `h_step : ∀ w, lInf (ns_step ...) ≤ lInf w` and
returns `IsSmooth (ns_trajectory_generator ...)`. Discharging `h_step`
for non-trivial Biot-Savart and non-trivial stretching is the open
piece. The work in `NSRegularity.NS.AdvectionBound` factors this
problem into named arithmetic chunks; the chunks are all proved; the
final inequality reducing them to L∞ preservation is left as a
hypothesis the caller supplies.

## Reduction chain

The full closure of the per-component step would compose:

```
ns_step_component_perturbation_le        (proved, this PR)
  |ns_step_component u uc_grad f p (i,j,k) − diffusion_step f p (i,j,k)|
      ≤ localPressure u uc_grad f p (i,j,k)

lInf_preserved_of_capacity_aux           (proved, this PR — arithmetic core)
  habs ∧ hperturb ∧ hcap_ijk ⟹ |A| ≤ G

localPressure_uniform_le_aux             (proved, this PR — Piece A core)
  bound localPressure pointwise by Δt·(6·G·M_u + M_uc)

localCapacity_uniform_ge_aux             (proved, this PR — Piece B core)
  capacity_uniform ≤ capacity_local      [via Res-A diffusion contraction]

capacity_vs_pressure_sufficient_aux      (proved, this PR — composition)
  Piece A + Piece B + CFL ⟹ pointwise capacity-vs-pressure inequality

small_data_sufficient_aux                (proved, this PR — explicit threshold)
  data bound K + CFL refinement |Δt|·K ≤ G − D_max
      ⟹ Δt·(6·G·M_u + M_uc) ≤ G − D_max
```

What is **NOT** proved (this is the residual open content):

```
Piece A — DATA BOUND          (open, structural)
  ∃ K : Int, 6 · gridLInf f · max(gridLInf u.x, …, .z) + gridLInf uc_grad ≤ K

  Trivially exists for any specific input (take K equal to the LHS).
  The open content is whether K can be chosen *uniformly along the
  trajectory* — i.e., that the velocity / stretching budget does not
  blow up faster than the diffusion contraction can absorb it. For
  3D NS in general this is exactly Beale-Kato-Majda territory.

Piece B — CFL REFINEMENT      (open, quantitative)
  ∀ trajectory state w, |p.dt_latt| · K ≤ gridLInf w.{x|y|z}
                                       − D_max(w.{x|y|z}, p)

  Asks for a step-size constraint relating |Δt| to the spare diffusion
  contraction capacity. Standard parabolic CFL gives |Δt| ≤ C·h²/ν for
  the diffusion sub-step alone; this Piece would need the analogous
  bound for the full operator-split step. Proving it requires either
  (a) a small-data regime with explicit smallness conditions, or
  (b) a fine analysis of the laplacian's spare contraction capacity at
  every grid point, not just at the sup.
```

## Closure target

A full Lean theorem closing the gate would have shape

```lean
theorem ns_step_component_preserves_gridLInf
    (u : VField) (uc_grad : Grid) (f : Grid) (p : NsParams)
    (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hns_pos : 0 < p.ns_scale)
    (hExact : … Res-A exactness …)
    (h_pieceA : ∃ K, 0 ≤ K ∧ data-bound condition)
    (h_pieceB : ∃ D_max, diffusion-step bound D_max ∧ |Δt|·K ≤ gridLInf f − D_max)
    : gridLInf (ns_step_component u uc_grad f p) ≤ gridLInf f := by
  …compose via the aux helpers listed above…
```

Each `∃` is the substantive open content. The compositional step
between them — once both are present as hypotheses — is one `linarith`
call through `lInf_preserved_of_capacity_aux`.

## Recipe for future attacks

1. Pick a regime (small-data, axisymmetric, periodic + symmetric
   initial data, …) in which Piece A is tractable.
2. Bound `gridLInf u` along the trajectory using Kill #113 + the
   Sobolev embedding `C_phys(N)` from `STATUS_MATRIX.md`.
3. Bound `gridLInf uc_grad` similarly.
4. Combine to obtain a uniform `K` via `small_data_sufficient_aux`.
5. Tune the integer CFL in `NsParams.cfl_max` to satisfy
   `|Δt|·K ≤ gridLInf f − D_max`. (`cfl_max` currently uses
   `(NS_SCALE² · n2) / (6 · nu_scaled)`; the tightening replaces 6 by
   an N-dependent constant absorbing K.)
6. Apply `capacity_vs_pressure_sufficient_aux` then
   `lInf_preserved_of_capacity_aux` componentwise to obtain
   `gridLInf (ns_step ...) ≤ gridLInf w`.
7. Lift to `lInf (ns_step ...) ≤ lInf w` (the `h_step` hypothesis of
   `ns_global_regularity_skeleton`).
8. Invoke `ns_global_regularity_skeleton` to obtain `IsSmooth`.

Steps 1–4 are the operator data analysis (Piece A); steps 5–6 are
the CFL refinement (Piece B); steps 7–8 are the existing skeleton
plumbing.

## What is **not** part of this gate

The four open items of `lean/PROOF_STATUS.md`:

| Open item | This gate | Closed here? |
|---|---|---|
| Constructive Biot-Savart NTT port | independent | No |
| Non-trivial full-step L∞ closure  | **THIS gate** | partially — operator-side factored, both Pieces open |
| Analytical N→∞ transfer           | independent | No |
| Continuum C^∞ via Mathlib PDE     | independent | No |

Closing this gate (both Pieces) is one of four pieces of the full
Clay-level continuum theorem. The other three remain entirely
out-of-scope for this PR and the surrounding session.
