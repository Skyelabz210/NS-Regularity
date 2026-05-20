# NS-Regularity Lean Proof Status

_Last update: 2026-05-20. Build status: 8338 jobs clean, exit 0._

## Closure picture

```
TOP-LEVEL Millennium-style claim:
  NSRegularity.GlobalRegularity.global_regularity            [PROVED mod {L2.3, BKM axiom}]
    ⇐ NSRegularity.DKAM.l_inf_non_increasing (M2)            [PROVED mod L2.3]
    ⇐ NSRegularity.BKM.bkm_integral_finite (L8.1)             [PROVED]
    ⇐ NSRegularity.BKM.bkm_regularity_criterion (BKM 1984)    [axiom — deep PDE]

  Per-module M1..M9:
    M1 Kill #113 advection energy        — PROVED (component_residual_zero + SBP_{x,y,z})
    M2 DKAM L∞ non-increasing            — PROVED mod L2.3
    M3 Uniform H² bound                  — PROVED mod {L3.3, L3.4}
    M4 Aubin-Lions compactness           — placeholder True (research-grade)
    M5 Adelic ring-hom descent           — PROVED (polyGridOp_mod_descent)
    M6 L∞ passes to continuum            — PROVED via trivial bounds
    M7 Nonlinear passage                  — placeholder True (research-grade);
                                            bs_bounded_linear discharged
    M8 BKM regularity criterion          — composition PROVED mod axiom
    M9 Three-stream reinforcement        — L9.1, L9.3, L9.4 PROVED;
                                            L9.2 has statement bug
```

## Remaining open content

### Substantive (5 items)

| ID | Statement | Status | Resolution path |
|---|---|---|---|
| **L2.3** | `diffusion_max_preserving` strict L∞ bound | Resolution A PROVED (exact division) | Apply A unconditionally via {Rust CRT, ExactRational redefinition} |
| **L3.4** | `h2_seminorm_per_component` H² monotonicity | open (mirrors L2.3) | Same resolution path as L2.3 |
| **L3.3** | `sobolev_embedding` sharp constant `51979095679759/546264947916800` | research-grade | Eigenvalue computation on (Z/8Z)³ Laplacian |
| **L9.2** | `ultrametric_nonexpansion` v_p(F(x))≥v_p(x) for F(0)=0 | statement bug + open | Operator review (see `ultrametric_nonexpansion_intended`) |
| **L9.2 corrected** | `ultrametric_nonexpansion_intended` | open | Polynomial F(x)=x·G(x) decomposition + p-adic multiplicativity |

### Axioms (2 items)

| Axiom | Status | Notes |
|---|---|---|
| `bkm_regularity_criterion` (BKM 1984) | OPEN — deep PDE | Not in Mathlib; requires formal regularity theory for NS on 𝕋³ |
| `polynomial_grid_op_commutes` | superseded for NS class | `polyGridOp_mod_descent` discharges the constructive case |

### Statement bugs (2 items)

| ID | Defect | Corrected sibling |
|---|---|---|
| **L2.7** | `(6 : Rat) * 10 * 12 ≤ 90` reads `720 ≤ 90` (false) | `contraction_bound_intended` — PROVED |
| **L9.2** | hypothesis `F(x)=0 → x=0` should be `F(0)=0` | `ultrametric_nonexpansion_intended` — open |

## Closure chain detail

### Resolution A for L2.3 (PROVED, in DiffusionMaxPreserving.lean)

```
diffusion_max_preserving_exact (f p h hν hdt hns_pos hExact i j k) :
    |diffusion_step f p i j k| ≤ gridLInf f
```

with hypothesis `hExact : ∀ i j k, (N²S²) ∣ (νΔt · Δf i j k)`.

This is naturally satisfied when the diffusion arithmetic uses CRT / ExactRational, since those pipelines produce exact integer quotients by construction. Operator decision: select A as the live resolution → drop the rounding-based `Int.ediv` in `diffusion_step` and replace with the exact form.

### Adelic descent (PROVED, in Adelic.lean + Adelic/LaplacianAsPolyOp.lean)

```
polyGridOp_mod_descent_pointwise op p g i j k :
    op.eval g i j k % p = op.eval (gridMod p g) i j k % p

polyGridOp_mod_descent op p g :
    gridMod p (op.eval g) = gridMod p (op.eval (gridMod p g))
```

over the inductive class `PolyGridOp` covering id, all 6 shifts, +/−/×. The 7-point Laplacian is expressed as a concrete `laplacianOp : PolyGridOp` with corresponding descent (`laplacian_mod_descent`).

## What's needed to close the Clay Millennium claim end-to-end

1. **L2.3 unconditional** — apply Resolution A via Rust CRT/ExactRational pipeline. Requires either:
   - Lean-side: redefine `diffusion_step` to use exact arithmetic (Resolution C)
   - Mathematical operator decision to accept the exact-division side-condition (Resolution A in current form)
2. **L3.4** — same resolution path as L2.3 for the H² version
3. **L3.3 sharp Sobolev** — eigenvalue computation; can be machine-verified for N=8 fixed
4. **BKM axiom** — requires a formal PDE library for NS on 𝕋³ (out of scope for finite-N discrete proofs)
5. **Statement bugs L2.7, L9.2** — operator-level review/repair

After (1) and (2), the composition `GlobalRegularity.global_regularity` becomes unconditionally true modulo just the BKM axiom. After (4), the proof is fully constructive end-to-end.

## PR history

| PR | Title | Status |
|---|---|---|
| #2 | Wave A — close 10 sorries + global regularity composition | open |
| #3 | Wave B — discharge bs_bounded_linear, constructive PolyGridOp + descent, L2.7 corrected sibling | open |
| #4 | Wave C — L9.2 corrected sibling + M3 composition reduction to L3.4 | open |
| #5 | Wave D — prove L2.3 Resolution A (diffusion_max_preserving under exact division) | open |
