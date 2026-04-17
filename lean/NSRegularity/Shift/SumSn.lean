import NSRegularity.Shift.Defs
import NSRegularity.Shift.SpSn
import NSRegularity.Shift.SnSp
import Mathlib.Algebra.BigOperators.Group.Finset.Basic

/-!
# Shift/SumSn — L1.4 (one leaf per file).

Periodic sum is invariant under the `sn` reindexing. Same pattern as L1.3.
-/

namespace NSRegularity.Shift

open NSRegularity

/-- The shift `sn` packaged as an equivalence on `Fin N`. -/
def snEquiv : Fin N ≃ Fin N :=
  ⟨sn, sp, sp_sn, sn_sp⟩

/-- L1.4 — periodic sum is invariant under `sn`. -/
theorem sum_sn (f : Fin N → Int) :
    (∑ i : Fin N, f (sn i)) = ∑ i : Fin N, f i :=
  Equiv.sum_comp snEquiv f

end NSRegularity.Shift
