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
  have h : List.finRange N = [0,1,2,3,4,5,6,7] := by decide
  simp [h]
  show f (sp 0) + f (sp 1) + f (sp 2) + f (sp 3) + f (sp 4) + f (sp 5) + f (sp 6) + f (sp 7) =
       f 0 + f 1 + f 2 + f 3 + f 4 + f 5 + f 6 + f 7
  unfold sp
  decide

end NSRegularity.Shift