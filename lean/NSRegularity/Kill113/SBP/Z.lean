import NSRegularity.Shift
import NSRegularity.Kill113.SBP.Defs
import NSRegularity.Kill113.SBP.X

/-!
# Kill113/SBP/Z — L1.7. Same shape as L1.5 in the z-direction.
-/

namespace NSRegularity.Kill113.SBP

open NSRegularity NSRegularity.Shift

/-- L1.7 — 1D summation-by-parts in z.  Same statement as L1.5 up to
    α-renaming of the summation index. -/
theorem sbp_z_1d (u ω : Fin N → Int) :
    (∑ k : Fin N, ω k * (u k * (ω (sp k) - ω (sn k))
                       + (u (sp k) * ω (sp k) - u (sn k) * ω (sn k)))) = 0 :=
  sbp_x_1d u ω

end NSRegularity.Kill113.SBP
