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

/-! ## Bilinear self-adjointness — composition of the 6 reindex lemmas -/

/-- **SBP / self-adjointness (bilinear).**
    `Σ f · Δg = Σ g · Δf`.

    Proved by:
    1. Expanding both sides via `laplacian` definition,
    2. Splitting each sum-of-products into 7 component sumGrids
       (6 cross-terms + diagonal),
    3. Reindexing each cross-term on LHS to match RHS via
       the `cross_*` lemmas + `mul_comm`. -/
theorem sbp_self_adjoint_bilinear (f g : Grid) :
    sumGrid (fun i j k => f i j k * laplacian g i j k) =
    sumGrid (fun i j k => g i j k * laplacian f i j k) := by
  -- Both sides decompose to: cross-x-sp + cross-x-sn + cross-y-sp + cross-y-sn
  --                        + cross-z-sp + cross-z-sn - 6 · diag
  -- Helper: distribute the sumGrid over the 7-way decomposition.
  have hsplit : ∀ (h₁ h₂ : Grid),
      sumGrid (fun i j k => h₁ i j k * (h₂ (sp i) j k + h₂ (sn i) j k
                                       + h₂ i (sp j) k + h₂ i (sn j) k
                                       + h₂ i j (sp k) + h₂ i j (sn k)
                                       - 6 * h₂ i j k)) =
        sumGrid (fun i j k => h₁ i j k * h₂ (sp i) j k) +
        sumGrid (fun i j k => h₁ i j k * h₂ (sn i) j k) +
        sumGrid (fun i j k => h₁ i j k * h₂ i (sp j) k) +
        sumGrid (fun i j k => h₁ i j k * h₂ i (sn j) k) +
        sumGrid (fun i j k => h₁ i j k * h₂ i j (sp k)) +
        sumGrid (fun i j k => h₁ i j k * h₂ i j (sn k)) -
        6 * sumGrid (fun i j k => h₁ i j k * h₂ i j k) := by
    intros h₁ h₂
    unfold sumGrid
    -- Step 1: pointwise distribute the multiplication
    have hdistribute : ∀ i j k : Fin N,
        h₁ i j k * (h₂ (sp i) j k + h₂ (sn i) j k
                  + h₂ i (sp j) k + h₂ i (sn j) k
                  + h₂ i j (sp k) + h₂ i j (sn k)
                  - 6 * h₂ i j k) =
        h₁ i j k * h₂ (sp i) j k + h₁ i j k * h₂ (sn i) j k
      + h₁ i j k * h₂ i (sp j) k + h₁ i j k * h₂ i (sn j) k
      + h₁ i j k * h₂ i j (sp k) + h₁ i j k * h₂ i j (sn k)
      - 6 * (h₁ i j k * h₂ i j k) := by
      intros i j k; ring
    simp only [hdistribute, Finset.sum_add_distrib, Finset.sum_sub_distrib,
               ← Finset.mul_sum]
  -- Apply hsplit to LHS (with h₁ = f, h₂ = g) and to RHS (with h₁ = g, h₂ = f)
  have hLHS_eq : sumGrid (fun i j k => f i j k * laplacian g i j k) =
        sumGrid (fun i j k => f i j k * g (sp i) j k) +
        sumGrid (fun i j k => f i j k * g (sn i) j k) +
        sumGrid (fun i j k => f i j k * g i (sp j) k) +
        sumGrid (fun i j k => f i j k * g i (sn j) k) +
        sumGrid (fun i j k => f i j k * g i j (sp k)) +
        sumGrid (fun i j k => f i j k * g i j (sn k)) -
        6 * sumGrid (fun i j k => f i j k * g i j k) := by
    unfold laplacian; exact hsplit f g
  have hRHS_eq : sumGrid (fun i j k => g i j k * laplacian f i j k) =
        sumGrid (fun i j k => g i j k * f (sp i) j k) +
        sumGrid (fun i j k => g i j k * f (sn i) j k) +
        sumGrid (fun i j k => g i j k * f i (sp j) k) +
        sumGrid (fun i j k => g i j k * f i (sn j) k) +
        sumGrid (fun i j k => g i j k * f i j (sp k)) +
        sumGrid (fun i j k => g i j k * f i j (sn k)) -
        6 * sumGrid (fun i j k => g i j k * f i j k) := by
    unfold laplacian; exact hsplit g f
  rw [hLHS_eq, hRHS_eq]
  -- Now use cross-term reindex (LHS sum) = (RHS sum, mul_commuted)
  -- Each pairing:
  --   LHS cross_x_sp: Σ f·g(sp_x) = Σ f(sn_x)·g  by cross_x_sp
  --   RHS cross_x_sn: Σ g·f(sn_x) = Σ f(sn_x)·g  by mul_comm
  -- So LHS_term_x_sp = RHS_term_x_sn (swapped).
  have e1 : sumGrid (fun i j k => f i j k * g (sp i) j k) =
            sumGrid (fun i j k => g i j k * f (sn i) j k) := by
    rw [cross_x_sp]
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    ring
  have e2 : sumGrid (fun i j k => f i j k * g (sn i) j k) =
            sumGrid (fun i j k => g i j k * f (sp i) j k) := by
    rw [cross_x_sn]
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    ring
  have e3 : sumGrid (fun i j k => f i j k * g i (sp j) k) =
            sumGrid (fun i j k => g i j k * f i (sn j) k) := by
    rw [cross_y_sp]
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    ring
  have e4 : sumGrid (fun i j k => f i j k * g i (sn j) k) =
            sumGrid (fun i j k => g i j k * f i (sp j) k) := by
    rw [cross_y_sn]
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    ring
  have e5 : sumGrid (fun i j k => f i j k * g i j (sp k)) =
            sumGrid (fun i j k => g i j k * f i j (sn k)) := by
    rw [cross_z_sp]
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    ring
  have e6 : sumGrid (fun i j k => f i j k * g i j (sn k)) =
            sumGrid (fun i j k => g i j k * f i j (sp k)) := by
    rw [cross_z_sn]
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    ring
  have e7 : sumGrid (fun i j k => f i j k * g i j k) =
            sumGrid (fun i j k => g i j k * f i j k) := by
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    ring
  rw [e1, e2, e3, e4, e5, e6, e7]
  ring

/-! ## H¹ quadratic form identity -/

/-- Forward-gradient energy: Σ |∇⁺f|² summed over the 3 spatial directions. -/
def gradSqForward (f : Grid) : Int :=
  ∑ i : Fin N, ∑ j : Fin N, ∑ k : Fin N,
    ((f (sp i) j k - f i j k) ^ 2
   + (f i (sp j) k - f i j k) ^ 2
   + (f i j (sp k) - f i j k) ^ 2)

/-- **SBP quadratic identity.**
    `Σ f · Δf = -Σ |∇⁺f|²`.

    The discrete H¹ analogue of the integration-by-parts identity
    ⟨f, Δf⟩ = −‖∇f‖². Specialization of `sbp_self_adjoint_bilinear`
    to f = g, combined with the algebraic identity
    (a−b)² = a² − 2ab + b² and sumGrid shift-invariance. -/
theorem sbp_quadratic (f : Grid) :
    sumGrid (fun i j k => f i j k * laplacian f i j k) = -gradSqForward f := by
  -- 1. Decompose Σ f · Δf into 7 sums (6 cross + 1 diagonal):
  have hLHS_split : sumGrid (fun i j k => f i j k * laplacian f i j k) =
        sumGrid (fun i j k => f i j k * f (sp i) j k) +
        sumGrid (fun i j k => f i j k * f (sn i) j k) +
        sumGrid (fun i j k => f i j k * f i (sp j) k) +
        sumGrid (fun i j k => f i j k * f i (sn j) k) +
        sumGrid (fun i j k => f i j k * f i j (sp k)) +
        sumGrid (fun i j k => f i j k * f i j (sn k)) -
        6 * sumGrid (fun i j k => f i j k * f i j k) := by
    unfold sumGrid laplacian
    have hdistribute : ∀ i j k : Fin N,
        f i j k * (f (sp i) j k + f (sn i) j k
                 + f i (sp j) k + f i (sn j) k
                 + f i j (sp k) + f i j (sn k)
                 - 6 * f i j k) =
        f i j k * f (sp i) j k + f i j k * f (sn i) j k
      + f i j k * f i (sp j) k + f i j k * f i (sn j) k
      + f i j k * f i j (sp k) + f i j k * f i j (sn k)
      - 6 * (f i j k * f i j k) := by
      intros i j k; ring
    simp only [hdistribute, Finset.sum_add_distrib, Finset.sum_sub_distrib,
               ← Finset.mul_sum]
  -- 2. The "sn" cross terms equal the "sp" cross terms by reindexing + mul_comm:
  have sn_eq_sp_x : sumGrid (fun i j k => f i j k * f (sn i) j k) =
                    sumGrid (fun i j k => f i j k * f (sp i) j k) := by
    rw [cross_x_sn f f]
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    ring
  have sn_eq_sp_y : sumGrid (fun i j k => f i j k * f i (sn j) k) =
                    sumGrid (fun i j k => f i j k * f i (sp j) k) := by
    rw [cross_y_sn f f]
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    ring
  have sn_eq_sp_z : sumGrid (fun i j k => f i j k * f i j (sn k)) =
                    sumGrid (fun i j k => f i j k * f i j (sp k)) := by
    rw [cross_z_sn f f]
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    ring
  -- 3. Σ f(sp_d)² = Σ f² for each direction (sumGrid shift-invariance applied to f²):
  have sq_inv_x : sumGrid (fun i j k => (f (sp i) j k) ^ 2) =
                  sumGrid (fun i j k => (f i j k) ^ 2) :=
    sumGrid_sp_x (fun i j k => (f i j k) ^ 2)
  have sq_inv_y : sumGrid (fun i j k => (f i (sp j) k) ^ 2) =
                  sumGrid (fun i j k => (f i j k) ^ 2) :=
    sumGrid_sp_y (fun i j k => (f i j k) ^ 2)
  have sq_inv_z : sumGrid (fun i j k => (f i j (sp k)) ^ 2) =
                  sumGrid (fun i j k => (f i j k) ^ 2) :=
    sumGrid_sp_z (fun i j k => (f i j k) ^ 2)
  -- 4. Expand gradSqForward into 3 directional terms, each (sp - center)²:
  have hRHS_split : gradSqForward f =
        (sumGrid (fun i j k => (f (sp i) j k) ^ 2)
         - 2 * sumGrid (fun i j k => f i j k * f (sp i) j k)
         + sumGrid (fun i j k => (f i j k) ^ 2)) +
        (sumGrid (fun i j k => (f i (sp j) k) ^ 2)
         - 2 * sumGrid (fun i j k => f i j k * f i (sp j) k)
         + sumGrid (fun i j k => (f i j k) ^ 2)) +
        (sumGrid (fun i j k => (f i j (sp k)) ^ 2)
         - 2 * sumGrid (fun i j k => f i j k * f i j (sp k)) +
        sumGrid (fun i j k => (f i j k) ^ 2)) := by
    unfold gradSqForward sumGrid
    have hexpand : ∀ i j k : Fin N,
        (f (sp i) j k - f i j k) ^ 2
       + (f i (sp j) k - f i j k) ^ 2
       + (f i j (sp k) - f i j k) ^ 2 =
        ((f (sp i) j k) ^ 2 - 2 * (f i j k * f (sp i) j k) + (f i j k) ^ 2)
      + ((f i (sp j) k) ^ 2 - 2 * (f i j k * f i (sp j) k) + (f i j k) ^ 2)
      + ((f i j (sp k)) ^ 2 - 2 * (f i j k * f i j (sp k)) + (f i j k) ^ 2) := by
      intros i j k; ring
    simp only [hexpand, Finset.sum_add_distrib, Finset.sum_sub_distrib,
               ← Finset.mul_sum]
  -- 5. Combine everything: substitute sq_inv_* and sn_eq_sp_* and ring
  rw [hLHS_split, hRHS_split]
  rw [sq_inv_x, sq_inv_y, sq_inv_z, sn_eq_sp_x, sn_eq_sp_y, sn_eq_sp_z]
  -- Now both sides are polynomials in the same three quantities
  -- (sumGrid f*f(sp_d) for d ∈ {x,y,z}) plus diagonal sumGrid f²
  -- Note: sumGrid (fun i j k => f i j k * f i j k) = sumGrid (fun i j k => (f i j k)^2)
  have hsq_eq : sumGrid (fun i j k => f i j k * f i j k) =
                sumGrid (fun i j k => (f i j k) ^ 2) := by
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    apply Finset.sum_congr rfl; intros _ _
    ring
  rw [hsq_eq]
  ring

end NSRegularity.HSquared
