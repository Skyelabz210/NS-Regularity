/-!
# Shift — periodic reindex lemmas (leaves L1.1–L1.4)

Each theorem is a LEAF: agents close each `sorry` independently. No cross-leaf
reasoning. Each takes only the `Shift.Defs` imports and closes via arithmetic
on `Fin N`.
-/

import NSRegularity.Shift.Defs

namespace NSRegularity.Shift

open NSRegularity

/-- L1.1 — successor undoes predecessor on Fin 8. -/
theorem sp_sn (i : Fin N) : sp (sn i) = i := by
  sorry

/-- L1.2 — predecessor undoes successor on Fin 8. -/
theorem sn_sp (i : Fin N) : sn (sp i) = i := by
  sorry

/-- L1.3 — periodic sum is invariant under `sp` reindexing. -/
theorem sum_sp (f : Fin N → Int) :
    (List.finRange N).foldl (fun a i => a + f (sp i)) 0
  = (List.finRange N).foldl (fun a i => a + f i) 0 := by
  sorry

/-- L1.4 — periodic sum is invariant under `sn` reindexing. -/
theorem sum_sn (f : Fin N → Int) :
    (List.finRange N).foldl (fun a i => a + f (sn i)) 0
  = (List.finRange N).foldl (fun a i => a + f i) 0 := by
  sorry

end NSRegularity.Shift
