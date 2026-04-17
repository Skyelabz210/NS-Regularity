import NSRegularity.Shift
import NSRegularity.Kill113.SBP.Defs

/-!
# Kill113/SBP/X — L1.5 (one leaf per file).

1D summation-by-parts in the x-direction: Σ ω(i)·[advective + conservative] = 0.
Available lemmas: `sum_sp`, `sum_sn` (both witness periodic shift invariance).
-/

namespace NSRegularity.Kill113.SBP

open NSRegularity NSRegularity.Shift

/-- L1.5 — 1D summation-by-parts in x. -/
theorem sbp_x_1d (u ω : Fin N → Int) :
    sum1 (fun i =>
      ω i * (u i * (ω (sp i) - ω (sn i))
           + (u (sp i) * ω (sp i) - u (sn i) * ω (sn i)))) = 0 := by
  sorry

end NSRegularity.Kill113.SBP
