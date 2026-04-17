import NSRegularity.Shift.Defs

/-!
# Kill113/SBP/Defs — shared definitions for SBP leaves.
-/

namespace NSRegularity.Kill113.SBP

open NSRegularity

/-- 1D sum helper, via `Finset.sum`. -/
def sum1 (g : Fin N → Int) : Int :=
  ∑ i : Fin N, g i

end NSRegularity.Kill113.SBP
