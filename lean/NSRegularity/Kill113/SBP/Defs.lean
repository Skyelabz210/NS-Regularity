import NSRegularity.Shift.Defs

/-!
# Kill113/SBP/Defs — shared definitions for SBP leaves.
-/

namespace NSRegularity.Kill113.SBP

open NSRegularity

/-- 1D sum helper. -/
def sum1 (g : Fin N → Int) : Int :=
  (List.finRange N).foldl (fun a i => a + g i) 0

end NSRegularity.Kill113.SBP
