import NSRegularity.NS.RoundedStep

/-! The split operator can exit the integer curl image even when every
    quotient is exact. This fixture uses the layered velocity in RoundedStep. -/

namespace NSRegularity.NS.LegacySplit

open NSRegularity NSRegularity.NS NSRegularity.NS.RoundedStep NSRegularity.Kill113

private def w : VField := curl layeredVelocity

private def advance : VField :=
  ns_step layeredVelocity (advective w layeredVelocity.x) (advective w layeredVelocity.y)
    (advective w layeredVelocity.z) w layeredParams

private theorem ux_multiple (i j k : Fin N) : (16 : ℤ) ∣ layeredVelocity.x i j k := by
  change (16 : ℤ) ∣ layeredVelocity.x 0 j 0
  fin_cases j <;> decide

private theorem uy_multiple (i j k : Fin N) : (16 : ℤ) ∣ layeredVelocity.y i j k := by
  change (16 : ℤ) ∣ layeredVelocity.y 0 0 k
  fin_cases k <;> decide

private theorem wx_multiple (i j k : Fin N) : (64 : ℤ) ∣ w.x i j k := by
  change (64 : ℤ) ∣ w.x 0 0 k
  fin_cases k <;> decide

private theorem wz_multiple (i j k : Fin N) : (64 : ℤ) ∣ w.z i j k := by
  have h : (64 : ℤ) ∣ w.z 0 j 0 := by fin_cases j <;> decide
  simpa only [w, curl, curlZ, diffX, diffY, layeredVelocity, sub_self] using h

private theorem wy_zero (i j k : Fin N) : w.y i j k = 0 := by
  simp [w, curl, curlY, layeredVelocity, diffX, diffZ]

private theorem lap_multiple (f : Grid) (h : ∀ i j k, (64 : ℤ) ∣ f i j k) (i j k : Fin N) :
    (64 : ℤ) ∣ laplacian f i j k := by
  unfold laplacian
  exact dvd_sub (dvd_add (dvd_add (dvd_add (dvd_add (dvd_add (h _ _ _) (h _ _ _))
    (h _ _ _)) (h _ _ _)) (h _ _ _)) (h _ _ _)) (dvd_mul_of_dvd_right (h _ _ _) _)

private theorem adv_multiple (u : VField) (f : Grid) (i j k : Fin N)
    (hx : (16 : ℤ) ∣ u.x i j k) (hy : (16 : ℤ) ∣ u.y i j k) (hz : (16 : ℤ) ∣ u.z i j k) :
    (16 : ℤ) ∣ advective u f i j k := by
  unfold advective
  exact dvd_add (dvd_add (dvd_mul_of_dvd_left hx _) (dvd_mul_of_dvd_left hy _))
    (dvd_mul_of_dvd_left hz _)

/-- All three layers of division are exact on the fixture, at every cell. -/
theorem layered_all_divisions_exact (i j k : Fin N) :
    (∀ f ∈ [w.x, w.y, w.z], (64 : ℤ) ∣ laplacian f i j k ∧
      (16 : ℤ) ∣ advective layeredVelocity (diffusion_step f layeredParams) i j k) ∧
    (∀ f ∈ [layeredVelocity.x, layeredVelocity.y, layeredVelocity.z],
      (16 : ℤ) ∣ advective w f i j k) := by
  constructor
  · intro f hf
    refine ⟨?_, adv_multiple _ _ _ _ _ (ux_multiple _ _ _) (uy_multiple _ _ _)
      (by simp [layeredVelocity])⟩
    simp only [List.mem_cons, List.not_mem_nil, or_false] at hf
    rcases hf with hf | hf | hf
    · rw [hf]; exact lap_multiple _ wx_multiple _ _ _
    · rw [hf]; exact lap_multiple _ (by intro a b c; rw [wy_zero]; exact dvd_zero _) _ _ _
    · rw [hf]; exact lap_multiple _ wz_multiple _ _ _
  · intro f _
    have h : (16 : ℤ) ∣ 64 := by decide
    exact adv_multiple _ _ _ _ _ (h.trans (wx_multiple _ _ _))
      (by rw [wy_zero]; exact dvd_zero _) (h.trans (wz_multiple _ _ _))

/-- Exact division still leaves a nonzero raw divergence at one cell. -/
theorem layered_split_divergence : divV advance 0 1 0 = 16 := by decide

/-- No centered-curl velocity can reproduce this one-step output. -/
theorem layered_split_no_preimage : ¬ ∃ v : VField, curl v = advance := by
  rintro ⟨v, hv⟩
  have hc := congrFun (congrFun (congrFun (curl_div_zero v) 0) 1) 0
  change divV (curl v) 0 1 0 = 0 at hc
  rw [hv, layered_split_divergence] at hc
  omega

end NSRegularity.NS.LegacySplit
