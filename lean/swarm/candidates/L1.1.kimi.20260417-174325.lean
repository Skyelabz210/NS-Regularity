import NSRegularity.Shift.Defs

/-! # Shift/SpSn — L1.1 (one leaf per file). Replace the `sorry` with a proof of `sp (sn i) = i` in `Fin 8`. -/

namespace NSRegularity.Shift

open NSRegularity

/-- L1.1 — successor undoes predecessor on Fin 8. -/
theorem sp_sn (i : Fin N) : sp (sn i) = i := by
  unfold sp sn
  simp [Fin.ext_iff]
  <;> omega

end NSRegularity.Shift