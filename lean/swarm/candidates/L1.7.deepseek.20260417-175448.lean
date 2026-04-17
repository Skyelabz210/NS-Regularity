import NSRegularity.Shift
import NSRegularity.Kill113.SBP.Defs
import NSRegularity.Kill113.SBP.X

/-!
# Kill113/SBP/Z — L1.7. Symmetric to L1.5 in the z-direction.
-/

namespace NSRegularity.Kill113.SBP

open NSRegularity NSRegularity.Shift

/-- L1.7 — 1D summation-by-parts in z. -/
theorem sbp_z_1d (u ω : Fin N → Int) :
    sum1 (fun k =>
      ω k * (u k * (ω (sp k) - ω (sn k))
           + (u (sp k) * ω (sp k) - u (sn k) * ω (sn k)))) = 0 := by
  simp [sum1]
  have h : List.finRange N = List.finRange N := rfl
  simp [List.foldl, List.finRange]
  induction' N with n ih
  · rfl
  · simp [Fin.forall_fin_succ, Fin.sum_univ_succ]
    intro i
    fin_cases i <;> simp [sp, sn, Fin.add_def, Fin.ext_iff] <;> omega

end NSRegularity.Kill113.SBP