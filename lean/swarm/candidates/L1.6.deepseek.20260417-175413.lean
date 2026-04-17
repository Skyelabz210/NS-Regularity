import NSRegularity.Shift
import NSRegularity.Kill113.SBP.Defs
import NSRegularity.Kill113.SBP.X

/-!
# Kill113/SBP/Y — L1.6. Symmetric to L1.5 in the y-direction.
-/

namespace NSRegularity.Kill113.SBP

open NSRegularity NSRegularity.Shift

/-- L1.6 — 1D summation-by-parts in y. -/
theorem sbp_y_1d (u ω : Fin N → Int) :
    sum1 (fun j =>
      ω j * (u j * (ω (sp j) - ω (sn j))
           + (u (sp j) * ω (sp j) - u (sn j) * ω (sn j)))) = 0 := by
  unfold sum1
  simp [List.finRange, Fin.reduceFin]
  decide

end NSRegularity.Kill113.SBP