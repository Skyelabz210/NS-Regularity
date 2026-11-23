import NSRegularity.NS.BiotSavart
import NSRegularity.NS.Defs

/-! A finite-grid right inverse on the actual integer image. Its gauge
    minimizes the component maximum. Realization by the NTT is separate. -/
namespace NSRegularity.NS.CurlPreimage

open NSRegularity NSRegularity.NS

def Admissible (w u : VField) : Prop :=
  curl u = w ∧ divV u = fun _ _ _ => 0

private def HasCost (w : VField) (k : ℕ) : Prop :=
  ∃ u, Admissible w u ∧ (lInf u).toNat = k

private theorem costs_exist {w : VField} (h : ∃ u, Admissible w u) :
    ∃ k, HasCost w k := by
  obtain ⟨u, hu⟩ := h
  exact ⟨_, u, hu, rfl⟩

/-- Choose a minimum-norm divergence-free integer preimage on the image.
    The minimizer can have ties. -/
noncomputable def minimumVelocity (w : VField) (h : ∃ u, Admissible w u) : VField := by
  classical
  exact Classical.choose (Nat.find_spec (costs_exist h))

theorem minimumVelocity_spec (w : VField) (h : ∃ u, Admissible w u) :
    Admissible w (minimumVelocity w h) ∧
      ∀ u, Admissible w u → (lInf (minimumVelocity w h)).toNat ≤ (lInf u).toNat := by
  classical
  let e := costs_exist h
  have spec := Classical.choose_spec (Nat.find_spec e)
  refine ⟨spec.1, ?_⟩
  intro u hu
  change (lInf (Classical.choose (Nat.find_spec e))).toNat ≤ _
  rw [spec.2]
  exact Nat.find_min' e ⟨u, hu, rfl⟩

/-- A total gauge choice for the existing trajectory generator. Outside the
    integer range its value is zero, so its divergence vanishes everywhere. -/
noncomputable def velocity (w : VField) : VField := by
  classical
  exact if h : ∃ u, Admissible w u then minimumVelocity w h
    else ⟨fun _ _ _ => 0, fun _ _ _ => 0, fun _ _ _ => 0⟩

theorem velocity_admissible (w : VField) (h : ∃ u, Admissible w u) :
    Admissible w (velocity w) := by
  classical
  simpa only [velocity, dif_pos h] using (minimumVelocity_spec w h).1

theorem velocity_div_zero (w : VField) : divV (velocity w) = fun _ _ _ => 0 := by
  classical
  by_cases h : ∃ u, Admissible w u
  · exact (velocity_admissible w h).2
  · rw [velocity, dif_neg h]
    rfl

end NSRegularity.NS.CurlPreimage
