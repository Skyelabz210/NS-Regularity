import NSRegularity.Shift
import NSRegularity.Kill113.SBP.Defs
import NSRegularity.Kill113.SBP.X

/-!
# Kill113/SBP/Y — L1.6. Same shape as L1.5 in the y-direction.
-/

namespace NSRegularity.Kill113.SBP

open NSRegularity NSRegularity.Shift

/-- L1.6 — 1D summation-by-parts in y.  Same statement as L1.5 up to
    α-renaming of the summation index. -/
theorem sbp_y_1d (u ω : Fin N → Int) :
    (∑ j : Fin N, ω j * (u j * (ω (sp j) - ω (sn j))
                       + (u (sp j) * ω (sp j) - u (sn j) * ω (sn j)))) = 0 :=
  sbp_x_1d u ω

end NSRegularity.Kill113.SBP
