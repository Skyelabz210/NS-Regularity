/-!
# Kill113/Ops — operator definitions for Kill #113

Definitions only. No theorems. Downstream files import this.
-/

import NSRegularity.Shift.Defs

namespace NSRegularity.Kill113

open NSRegularity

/-- Centered difference in x. -/
def diffX (f : Grid) (i j k : Fin N) : Int := f (sp i) j k - f (sn i) j k
def diffY (f : Grid) (i j k : Fin N) : Int := f i (sp j) k - f i (sn j) k
def diffZ (f : Grid) (i j k : Fin N) : Int := f i j (sp k) - f i j (sn k)

/-- Advective form `u · ∇ωc` (centered, no 1/(2h) factor). -/
def advective (u : VField) (ωc : Grid) (i j k : Fin N) : Int :=
  u.x i j k * diffX ωc i j k
  + u.y i j k * diffY ωc i j k
  + u.z i j k * diffZ ωc i j k

/-- Conservative form `∇ · (u ωc)` (centered, no 1/(2h) factor). -/
def conservative (u : VField) (ωc : Grid) (i j k : Fin N) : Int :=
  (u.x (sp i) j k * ωc (sp i) j k - u.x (sn i) j k * ωc (sn i) j k)
  + (u.y i (sp j) k * ωc i (sp j) k - u.y i (sn j) k * ωc i (sn j) k)
  + (u.z i j (sp k) * ωc i j (sp k) - u.z i j (sn k) * ωc i j (sn k))

/-- Energy residual: Σ ω · (adv + cons), summed over all grid points and the
    three vorticity components. Kill #113 claims this is identically 0. -/
def residual (u ω : VField) : Int :=
    sumGrid (fun i j k => ω.x i j k * (advective u ω.x i j k + conservative u ω.x i j k))
  + sumGrid (fun i j k => ω.y i j k * (advective u ω.y i j k + conservative u ω.y i j k))
  + sumGrid (fun i j k => ω.z i j k * (advective u ω.z i j k + conservative u ω.z i j k))

end NSRegularity.Kill113
