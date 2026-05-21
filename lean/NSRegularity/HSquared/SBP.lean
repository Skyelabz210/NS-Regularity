import NSRegularity.NS.Defs
import NSRegularity.Shift.SumGrid

/-!
# HSquared/SBP — Discrete summation-by-parts cross-term reindex lemmas

Six reindex lemmas, one per (direction, shift) pair, that handle the cross
terms in the expansion of `Σ f · Δg`:

```
Σ f · g(sp_x) = Σ f(sn_x) · g    (and analogously for sn shift, y, z dirs)
```

These are the foundational lemmas for the discrete Laplacian's self-adjointness
identity `Σ f · Δg = Σ g · Δf` and the H¹ quadratic-form identity
`Σ f · Δf = -Σ |∇⁺f|²`. The composition is straightforward bookkeeping
on top of these reindex lemmas.

## Future work (deferred)

The bilinear self-adjointness `sbp_self_adjoint_bilinear` follows from these
six reindex lemmas: expand each side via the Laplacian definition, then
reindex each cross term. The proof requires careful `Finset.sum_add_distrib`
+ `Finset.sum_congr` tactic engineering; deferred. This file provides the
load-bearing reindex lemmas; the bilinear identity itself is a mechanical
composition.
-/

namespace NSRegularity.HSquared

open NSRegularity NSRegularity.NS NSRegularity.Shift

/-! ## Cross-term reindex lemmas — six (direction, shift) pairs -/

/-- Reindex: Σ f · g(sp_x) = Σ f(sn_x) · g. -/
theorem cross_x_sp (f g : Grid) :
    sumGrid (fun i j k => f i j k * g (sp i) j k) =
    sumGrid (fun i j k => f (sn i) j k * g i j k) := by
  rw [← sumGrid_sn_x (fun i j k => f i j k * g (sp i) j k)]
  apply Finset.sum_congr rfl; intros i _
  apply Finset.sum_congr rfl; intros j _
  apply Finset.sum_congr rfl; intros k _
  simp only [sp_sn]

/-- Reindex: Σ f · g(sn_x) = Σ f(sp_x) · g. -/
theorem cross_x_sn (f g : Grid) :
    sumGrid (fun i j k => f i j k * g (sn i) j k) =
    sumGrid (fun i j k => f (sp i) j k * g i j k) := by
  rw [← sumGrid_sp_x (fun i j k => f i j k * g (sn i) j k)]
  apply Finset.sum_congr rfl; intros i _
  apply Finset.sum_congr rfl; intros j _
  apply Finset.sum_congr rfl; intros k _
  simp only [sn_sp]

/-- Reindex: Σ f · g(sp_y) = Σ f(sn_y) · g. -/
theorem cross_y_sp (f g : Grid) :
    sumGrid (fun i j k => f i j k * g i (sp j) k) =
    sumGrid (fun i j k => f i (sn j) k * g i j k) := by
  rw [← sumGrid_sn_y (fun i j k => f i j k * g i (sp j) k)]
  apply Finset.sum_congr rfl; intros i _
  apply Finset.sum_congr rfl; intros j _
  apply Finset.sum_congr rfl; intros k _
  simp only [sp_sn]

/-- Reindex: Σ f · g(sn_y) = Σ f(sp_y) · g. -/
theorem cross_y_sn (f g : Grid) :
    sumGrid (fun i j k => f i j k * g i (sn j) k) =
    sumGrid (fun i j k => f i (sp j) k * g i j k) := by
  rw [← sumGrid_sp_y (fun i j k => f i j k * g i (sn j) k)]
  apply Finset.sum_congr rfl; intros i _
  apply Finset.sum_congr rfl; intros j _
  apply Finset.sum_congr rfl; intros k _
  simp only [sn_sp]

/-- Reindex: Σ f · g(sp_z) = Σ f(sn_z) · g. -/
theorem cross_z_sp (f g : Grid) :
    sumGrid (fun i j k => f i j k * g i j (sp k)) =
    sumGrid (fun i j k => f i j (sn k) * g i j k) := by
  rw [← sumGrid_sn_z (fun i j k => f i j k * g i j (sp k))]
  apply Finset.sum_congr rfl; intros i _
  apply Finset.sum_congr rfl; intros j _
  apply Finset.sum_congr rfl; intros k _
  simp only [sp_sn]

/-- Reindex: Σ f · g(sn_z) = Σ f(sp_z) · g. -/
theorem cross_z_sn (f g : Grid) :
    sumGrid (fun i j k => f i j k * g i j (sn k)) =
    sumGrid (fun i j k => f i j (sp k) * g i j k) := by
  rw [← sumGrid_sp_z (fun i j k => f i j k * g i j (sn k))]
  apply Finset.sum_congr rfl; intros i _
  apply Finset.sum_congr rfl; intros j _
  apply Finset.sum_congr rfl; intros k _
  simp only [sn_sp]

end NSRegularity.HSquared
