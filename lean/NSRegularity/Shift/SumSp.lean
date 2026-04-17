import NSRegularity.Shift.Defs
import NSRegularity.Shift.SpSn
import NSRegularity.Shift.SnSp
import Mathlib.Algebra.BigOperators.Group.Finset.Basic

/-!
# Shift/SumSp — L1.3 (one leaf per file).

Periodic sum is invariant under the `sp` reindexing. Since `sp` is a
bijection (inverse `sn`, see `sp_sn`/`sn_sp`), this is a direct application
of `Equiv.sum_comp` with the equivalence `⟨sp, sn, sn_sp, sp_sn⟩`.
-/

namespace NSRegularity.Shift

open NSRegularity

/-- The shift `sp` packaged as an equivalence on `Fin N`. -/
def spEquiv : Fin N ≃ Fin N :=
  ⟨sp, sn, sn_sp, sp_sn⟩

/-- L1.3 — periodic sum is invariant under `sp`. -/
theorem sum_sp (f : Fin N → Int) :
    (∑ i : Fin N, f (sp i)) = ∑ i : Fin N, f i :=
  Equiv.sum_comp spEquiv f

end NSRegularity.Shift
