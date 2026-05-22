import NSRegularity.Shift.SumSp
import NSRegularity.Shift.SumSn

/-!
# Shift/SumGrid — 3D shift-invariance of `sumGrid`

Lifts the 1D reindex lemmas `sum_sp` / `sum_sn` to the 3D `sumGrid` triple
sum, in each of the three directions. These are the foundational lemmas
needed for discrete summation-by-parts in 3D (Σ f · Δg identities).

For an arbitrary integer-valued grid function `f`,
  `sumGrid (f ∘ sp_in_x) = sumGrid f`
  `sumGrid (f ∘ sn_in_x) = sumGrid f`
and similarly for y- and z-directions.
-/

namespace NSRegularity.Shift

open NSRegularity

/-- 3D shift-invariance in x via `sp`. -/
theorem sumGrid_sp_x (f : Grid) :
    sumGrid (fun i j k => f (sp i) j k) = sumGrid f := by
  unfold sumGrid
  exact sum_sp (fun i => ∑ j : Fin N, ∑ k : Fin N, f i j k)

/-- 3D shift-invariance in x via `sn`. -/
theorem sumGrid_sn_x (f : Grid) :
    sumGrid (fun i j k => f (sn i) j k) = sumGrid f := by
  unfold sumGrid
  exact sum_sn (fun i => ∑ j : Fin N, ∑ k : Fin N, f i j k)

/-- 3D shift-invariance in y via `sp`. -/
theorem sumGrid_sp_y (f : Grid) :
    sumGrid (fun i j k => f i (sp j) k) = sumGrid f := by
  unfold sumGrid
  apply Finset.sum_congr rfl
  intros i _
  exact sum_sp (fun j => ∑ k : Fin N, f i j k)

/-- 3D shift-invariance in y via `sn`. -/
theorem sumGrid_sn_y (f : Grid) :
    sumGrid (fun i j k => f i (sn j) k) = sumGrid f := by
  unfold sumGrid
  apply Finset.sum_congr rfl
  intros i _
  exact sum_sn (fun j => ∑ k : Fin N, f i j k)

/-- 3D shift-invariance in z via `sp`. -/
theorem sumGrid_sp_z (f : Grid) :
    sumGrid (fun i j k => f i j (sp k)) = sumGrid f := by
  unfold sumGrid
  apply Finset.sum_congr rfl
  intros i _
  apply Finset.sum_congr rfl
  intros j _
  exact sum_sp (fun k => f i j k)

/-- 3D shift-invariance in z via `sn`. -/
theorem sumGrid_sn_z (f : Grid) :
    sumGrid (fun i j k => f i j (sn k)) = sumGrid f := by
  unfold sumGrid
  apply Finset.sum_congr rfl
  intros i _
  apply Finset.sum_congr rfl
  intros j _
  exact sum_sn (fun k => f i j k)

end NSRegularity.Shift
