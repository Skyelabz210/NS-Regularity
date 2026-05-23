# NS-Regularity Lean Proof Status

_Last corrected: 2026-05-23. Sync correction after Wave H merge. Build status recorded by merged Wave H line: 8338+ lake jobs clean, exit 0._

This file is the active Lean proof-status index. It supersedes older contradictory status blocks inside prior wave reports.

## Current closure posture

The Lean workspace is synchronized through Wave H.15. The discrete PDE algebra and the Resolution-A diffusion control stack are active. The full Navier-Stokes claim remains conditional on the operator-side full-NS step gate and the Biot-Savart port.

### Closed / active Lean results

```lean
sbp_self_adjoint_bilinear              -- Σ f · Δg = Σ g · Δf
sbp_quadratic                          -- Σ f · Δf = -Σ |∇⁺f|²
sbp_quadratic_h2                       -- Σ Δf · Δ²f = -Σ |∇⁺(Δf)|²
laplacian_sq_le_12_gradSqForward       -- Σ (Δf)² ≤ 12 · Σ |∇⁺f|²
cauchy_schwarz_6                       -- six-term Cauchy-Schwarz
h2_seminorm_per_component_exact        -- H² monotonicity under Resolution A exactness
ns_hTwoNorm_uniform_exact              -- trajectory H² bound under Resolution A
l2Sq_diffusion_step_le_exact           -- L² monotonicity under Resolution A
l2NormSq_diffusion_step_le_exact       -- vector L² composition
ns_l2NormSq_uniform_exact              -- trajectory L² bound under Resolution A
diffusion_max_preserving_exact         -- L∞ monotonicity under Resolution A
generic_global_regularity_uniform      -- generic closure from uniform L∞ bound
ns_global_regularity_skeleton          -- full-NS trajectory skeleton under h_step
ns_global_regularity_biot_savart       -- skeleton using axiomatic Biot-Savart
curl_div_zero                          -- div ∘ curl = 0
curl_grad_zero                         -- curl ∘ grad = 0
laplacian_add, laplacian_smul, laplacian_zero
divV_add, gradV_add
curlX_add, curlY_add, curlZ_add
divV_gradV_eq_laplacianH
sumGrid_f_diffX_eq_neg_diffX_g
sumGrid_f_diffY_eq_neg_diffY_g
sumGrid_f_diffZ_eq_neg_diffZ_g
sumGrid_f_divV_eq_neg_gradV_dot
sbp_quadratic_H
discrete_ns_regularity_certificate_exact
polyGridOp_mod_descent_pointwise
polyGridOp_mod_descent
laplacian_mod_descent
```

### Resolution-A control bundle

Under the single exact-division side condition used by the CRT / ExactRational arithmetic substrate, the diffusion trajectory has the following proved bounds:

```lean
∀ n, l2NormSq  (diffusion_trajectory w₀ p n) ≤ l2NormSq  w₀
∀ n, lInf      (diffusion_trajectory w₀ p n) ≤ lInf      w₀
∀ n, hTwoNorm  (diffusion_trajectory w₀ p n) ≤ hTwoNorm  w₀
```

`discrete_ns_regularity_certificate_exact` bundles L², L∞, H², and smoothness via the BKM axiom. This is a diffusion-trajectory certificate, not an unconditional full-NS continuum theorem.

## Full-NS skeleton

The active full-NS skeleton is:

```lean
ns_trajectory_generator
  (biot_savart : VField → VField)
  (stretching_x stretching_y stretching_z : VField → Grid)
  (w₀ : VField) (p : NsParams) : ℕ → VField

ns_global_regularity_skeleton
  (..., h_step : ∀ w, lInf (ns_step ...) ≤ lInf w, ...) :
  IsSmooth (ns_trajectory_generator ...)

ns_global_regularity_biot_savart
  (..., h_step : ∀ w, lInf (ns_step (biot_savart w) ...) ≤ lInf w, ...) :
  IsSmooth (ns_trajectory_generator biot_savart ...)
```

This is the correct architectural top: the theorem is proved conditional on `h_step` and the BKM axiom, with `biot_savart` currently axiomatized pending the NTT port.

## Active gates

| Gate | Status | Required closure |
|---|---|---|
| `biot_savart` constructive port | OPEN | Port Rust NTT / CRT Biot-Savart into Lean and replace axioms |
| `biot_savart_curl_eq` | OPEN axiom | Prove `curl (biot_savart ω) = ω` for the exact discrete operator |
| `biot_savart_div_zero` | OPEN axiom | Prove `divV (biot_savart ω) = 0` |
| `h_step` full-NS L∞ preservation | OPEN | Compose diffusion + advection + stretching control for `ns_step` |
| `bkm_regularity_criterion` | OPEN axiom | Formal PDE library / imported theorem for the BKM criterion |
| `L3.3` sharp Sobolev embedding | OPEN / research-grade | Formal eigenvalue computation on `(Z/8Z)^3` |
| `L9.2 corrected` ultrametric non-expansion | OPEN | Prove corrected p-adic polynomial decomposition statement |

## Corrected status of formerly contradictory entries

| Item | Corrected status |
|---|---|
| `L2.3 diffusion_max_preserving` | Resolution A PROVED under exact-division side condition |
| `L3.4 h2_seminorm_per_component` | Resolution A PROVED under exact-division side condition |
| `L2.7 contraction_bound` | Original statement BUG; corrected sibling `contraction_bound_intended` PROVED |
| `L9.2 ultrametric_nonexpansion` | Original statement BUG; corrected sibling remains OPEN |
| `polynomial_grid_op_commutes` | Superseded for NS class by constructive `PolyGridOp` descent |
| Aubin-Lions / nonlinear passage placeholders | Not Clay-level closure; retained as scaffold until analytic bridge is formalized |

## PR merge history

| PR | Title | Status |
|---|---|---|
| #2 | Wave A — close 10 sorries + global regularity composition | MERGED into Wave H line |
| #3 | Wave B — discharge `bs_bounded_linear`, constructive `PolyGridOp` + descent | MERGED into Wave H line |
| #4 | Wave C — L9.2 corrected sibling + M3 composition reduction | MERGED into Wave H line |
| #5 | Wave D — prove L2.3 Resolution A | MERGED into Wave H line |
| #6 | Wave E — generic L∞ closure + full NS step defs + Aubin-Lions strengthening | MERGED into Wave H line |
| #7 | Wave F — discrete SBP foundations | MERGED into Wave H line |
| #8 | Wave G — NS trajectory generator + skeleton | MERGED into Wave H line |
| #9 | Wave H — Biot-Savart skeleton + vector-calc identities + L² Resolution A | MERGED 2026-05-22 |

## Claim discipline

1. The discrete diffusion certificate is formalized under Resolution A exactness.
2. The full-NS Lean theorem is a skeleton conditional on `h_step`, Biot-Savart axioms, and BKM.
3. The continuum Clay-level theorem is not marked closed by this file.
4. Nested Lean/Rust artifacts are lineage strata. They are presumed part of the build history and must be activated by their correct root, adapter, or feature gate rather than classified as orphan scratch.
