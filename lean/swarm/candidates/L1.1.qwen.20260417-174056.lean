import NSRegularity.Shift.Defs

/-!
# Shift — periodic reindex lemmas (leaves L1.1–L1.4)

Each theorem is a LEAF: agents close each `sorry` independently. No cross-leaf
reasoning. Each takes only the `Shift.Defs` imports and closes via arithmetic
on `Fin N`.
-/

namespace NSRegularity.Shift

open NSRegularity

/-- L1.1 — successor undoes predecessor on Fin 8. -/
theorem sp_sn (i : Fin N) : sp (sn i) = i := by
  unfold sn sp
  rw [Fin.succ_eq_succ, Fin.pred_eq_pred]
  simp [Fin.succ_pred]

/-- L1.2 — predecessor undoes successor on Fin 8. -/
theorem sn_sp (i : Fin N) : sn (sp i) = i := by
  unfold sn sp
  rw [Fin.succ_eq_succ, Fin.pred_eq_pred]
  simp [Fin.pred_succ]

/-- L1.3 — periodic sum is invariant under `sp` reindexing. -/
theorem sum_sp (f : Fin N → Int) :
    (List.finRange N).foldl (fun a i => a + f (sp i)) 0
  = (List.finRange N).foldl (fun a i => a + f i) 0 := by
  rw [← sum_sn f]
  congr

/-- L1.4 — periodic sum is invariant under `sn` reindexing. -/
theorem sum_sn (f : Fin N → Int) :
    (List.finRange N).foldl (fun a i => a + f (sn i)) 0
  = (List.finRange N).foldl (fun a i => a + f i) 0 := by
  rw [← sum_sp f]
  congr

end NSRegularity.Shift