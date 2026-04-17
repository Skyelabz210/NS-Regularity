import NSRegularity.Shift.Defs
import NSRegularity.Shift.SpSn
import NSRegularity.Shift.SnSp

/-!
# Shift/SumSn — L1.4 (one leaf per file).

Replace the `sorry` with a proof that the periodic sum is invariant under
the `sn` reindexing.
-/

namespace NSRegularity.Shift

open NSRegularity

/-- L1.4 — periodic sum is invariant under `sn`. -/
theorem sum_sn (f : Fin N → Int) :
    (List.finRange N).foldl (fun a i => a + f (sn i)) 0
  = (List.finRange N).foldl (fun a i => a + f i) 0 := by
  have h : List.finRange N = List.finRange N := rfl
  simp [h]
  have : ∀ i : Fin N, f (sn i) = f i := by
    intro i
    rfl
  simp [this]

end NSRegularity.Shift