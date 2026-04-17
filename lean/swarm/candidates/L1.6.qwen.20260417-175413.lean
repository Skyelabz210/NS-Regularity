theorem sbp_y_1d (u ω : Fin N → Int) : sum1 (fun j => ω j * (u j * (ω (sp j) - ω (sn j)) + (u (sp j) * ω (sp j) - u (sn j) * ω (sn j)))) = 0 := by
  have h : ∀ j : Fin N, ω j * (u j * (ω (sp j) - ω (sn j)) + (u (sp j) * ω (sp j) - u (sn j) * ω (sn j))) = 0 := by
    intro j
    simp [sp, sn, Fin.add_def, Fin.ext_iff]
    <;>
    (try omega) <;>
    (try decide) <;>
    (try
      {
        cases j with
        | mk val hval =>
          simp_all [Fin.val_mk]
          <;> omega
      })
  simp [h]
  <;> decide