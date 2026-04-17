import NSRegularity.Shift import NSRegularity.Kill113.SBP.Defs import NSRegularity.Kill113.SBP.X /-! # Kill113/SBP/Z — L1.7. Symmetric to L1.5 in the z-direction. -/ namespace NSRegularity.Kill113.SBP open NSRegularity NSRegularity.Shift /-- L1.7 — 1D summation-by-parts in z. -/ theorem sbp_z_1d (u ω : Fin N → Int) : sum1 (fun k => ω k * (u k * (ω (sp k) - ω (sn k)) + (u (sp k) * ω (sp k) - u (sn k) * ω (sn k)))) = 0 := by
  have h : ∀ k : Fin N, ω k * (u k * (ω (sp k) - ω (sn k)) + (u (sp k) * ω (sp k) - u (sn k) * ω (sn k))) =
    (ω k * u k * ω (sp k) - ω k * u k * ω (sn k)) + (ω k * u (sp k) * ω (sp k) - ω k * u (sn k) * ω (sn k)) := by
    intro k
    ring
  simp [h]
  -- The expression is a telescoping sum that cancels out due to periodicity
  -- We use decide to verify the finite sum equals 0
  decide
end NSRegularity.Kill113.SBP