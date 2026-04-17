import NSRegularity.Shift.Defs /-! # Shift — periodic reindex lemmas (leaves L1.1–L1.4) Each theorem is a LEAF: agents close each `sorry` independently. No cross-leaf reasoning. Each takes only the `Shift.Defs` imports and closes via arithmetic on `Fin N`. -/ namespace NSRegularity.Shift open NSRegularity /-- L1.1 — successor undoes predecessor on Fin 8. -/ theorem sp_sn (i : Fin N) : sp (sn i) = i := by
  simp [sp, sn, Fin.succAbove, Fin.predAbove, Fin.succ, Fin.castPred, Fin.castLE, Fin.castLT, Fin.cast, Fin.addNat, Fin.natAdd]
  <;> try { omega }
  <;> try { tauto }
  <;> try { native_decide } /-- L1.2 — predecessor undoes successor on Fin 8. -/ theorem sn_sp (i : Fin N) : sn (sp i) = i := by
  simp [sp, sn, Fin.succAbove, Fin.predAbove, Fin.succ, Fin.castPred, Fin.castLE, Fin.castLT, Fin.cast, Fin.addNat, Fin.natAdd]
  <;> try { omega }
  <;> try { tauto }
  <;> try { native_decide } /-- L1.3 — periodic sum is invariant under `sp` reindexing. -/ theorem sum_sp (f : Fin N → Int) : (List.finRange N).foldl (fun a i => a + f (sp i)) 0 = (List.finRange N).foldl (fun a i => a + f i) 0 := by
  have h1 : ∀ (f : Fin N → Int), (List.finRange N).foldl (fun a i => a + f (sp i)) 0 = (List.finRange N).foldl (fun a i => a + f i) 0 := by
    intro f
    have h2 : (List.finRange N).foldl (fun a i => a + f (sp i)) 0 = (List.finRange N).foldl (fun a i => a + f i) 0 := by
      have h3 : (List.finRange N).map (fun i => f (sp i)) = (List.finRange N).map (fun i => f i) := by
        apply List.map_congr_left
        intro i hi
        have h4 : sp i = i := by
          simp [sp, Fin.succAbove, Fin.predAbove, Fin.succ, Fin.castPred, Fin.castLE, Fin.castLT, Fin.cast, Fin.addNat, Fin.natAdd]
          <;> try { omega }
          <;> try { tauto }
          <;> try { native_decide }
        rw [h4]
      have h4 : (List.finRange N).foldl (fun a i => a + f (sp i)) 0 = ((List.finRange N).map (fun i => f (sp i))).sum := by
        rw [List.foldl_map]
        simp
      have h5 : (List.finRange N).foldl (fun a i => a + f i) 0 = ((List.finRange N).map (fun i => f i)).sum := by
        rw [List.foldl_map]
        simp
      rw [h4, h5, h3]
    exact h2
  exact h1 f /-- L1.4 — periodic sum is invariant under `sn` reindexing. -/ theorem sum_sn (f : Fin N → Int) : (List.finRange N).foldl (fun a i => a + f (sn i)) 0 = (List.finRange N).foldl (fun a i => a + f i) 0 := by
  have h1 : ∀ (f : Fin N → Int), (List.finRange N).foldl (fun a i => a + f (sn i)) 0 = (List.finRange N).foldl (fun a i => a + f i) 0 := by
    intro f
    have h2 : (List.finRange N).foldl (fun a i => a + f (sn i)) 0 = (List.finRange N).foldl (fun a i => a + f i) 0 := by
      have h3 : (List.finRange N).map (fun i => f (sn i)) = (List.finRange N).map (fun i => f i) := by
        apply List.map_congr_left
        intro i hi
        have h4 : sn i = i := by
          simp [sn, Fin.succAbove, Fin.predAbove, Fin.succ, Fin.castPred, Fin.castLE, Fin.castLT, Fin.cast, Fin.addNat, Fin.natAdd]
          <;> try { omega }
          <;> try { tauto }
          <;> try { native_decide }
        rw [h4]
      have h4 : (List.finRange N).foldl (fun a i => a + f (sn i)) 0 = ((List.finRange N).map (fun i => f (sn i))).sum := by
        rw [List.foldl_map]
        simp
      have h5 : (List.finRange N).foldl (fun a i => a + f i) 0 = ((List.finRange N).map (fun i => f i)).sum := by
        rw [List.foldl_map]
        simp
      rw [h4, h5, h3]
    exact h2
  exact h1 f end NSRegularity.Shift