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
  have h : sp ∘ sn = id := by
    ext i
    exact sp_sn i
  have h' : sn ∘ sp = id := by
    ext i
    exact sn_sp i
  have : Function.Bijective (sp : Fin N → Fin N) := by
    refine ⟨?_, ?_⟩
    · intro x y hxy
      have := congrArg sn hxy
      simp [h'] at this
      exact this
    · intro y
      refine ⟨sn y, ?_⟩
      simp [h]
  have : List.map sp (List.finRange N) = List.finRange N := by
    ext i
    simp [List.mem_finRange, Function.Bijective.surjective this]
  simp [List.foldl_map, this]

end NSRegularity.Shift