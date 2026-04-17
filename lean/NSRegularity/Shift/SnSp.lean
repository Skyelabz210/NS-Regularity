import NSRegularity.Shift.Defs

/-!
# Shift/SnSp — L1.2 (one leaf per file).

Replace the `sorry` with a proof of `sn (sp i) = i` in `Fin 8`.
-/

namespace NSRegularity.Shift

open NSRegularity

/-- L1.2 — predecessor undoes successor on Fin 8. -/
theorem sn_sp (i : Fin N) : sn (sp i) = i := by
  revert i; decide

end NSRegularity.Shift
