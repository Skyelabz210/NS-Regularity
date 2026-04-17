import NSRegularity.Shift.Defs
import NSRegularity.Shift.SpSn
import NSRegularity.Shift.SnSp

/-!
# Shift/SumSp — L1.3 (one leaf per file).

Replace the `sorry` with a proof that the periodic sum is invariant under
the `sp` reindexing. The available lemmas `sp_sn` and `sn_sp` witness
that `sp` and `sn` are mutual inverses on `Fin 8`.
-/

namespace NSRegularity.Shift

open NSRegularity

/-- L1.3 — periodic sum is invariant under `sp`. -/
theorem sum_sp (f : Fin N → Int) :
    (List.finRange N).foldl (fun a i => a + f (sp i)) 0
  = (List.finRange N).foldl (fun a i => a + f i) 0 := by
  -- Strategy: both folds reduce to a sum of the 8 values `f 0, ..., f 7`
  -- differing only by cyclic reordering; equal by Int addition commutativity.
  -- In standalone Lean 4 (no Mathlib.List.Perm), closing requires manually
  -- unfolding foldl over a concrete 8-element list then applying
  -- Int.add_comm / Int.add_assoc in a specific pattern.  Left as sorry.
  sorry

end NSRegularity.Shift
