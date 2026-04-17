import NSRegularity.Shift
import NSRegularity.Kill113.SBP.Defs
import Mathlib.Tactic.Ring
import Mathlib.Algebra.BigOperators.Group.Finset.Basic

/-!
# Kill113/SBP/X — L1.5 (one leaf per file).

1D summation-by-parts in the x-direction.

Strategy: expand the summand into four components T1 − T2 + T3 − T4, where
  T1 = ∑ ω·u·(ω∘sp)         T2 = ∑ ω·u·(ω∘sn)
  T3 = ∑ ω·(u∘sp)·(ω∘sp)    T4 = ∑ ω·(u∘sn)·(ω∘sn)
Reindex T4 via `sum_sn`  ⇒  T4 = ∑ (ω∘sp)·u·ω ;  then by mul-commutativity T4 = T1.
Reindex T3 via `sum_sp`  ⇒  T3 = T2.
Hence T1 − T2 + T3 − T4 = 0.
-/

namespace NSRegularity.Kill113.SBP

open NSRegularity NSRegularity.Shift

/-- L1.5 — 1D summation-by-parts in x. -/
theorem sbp_x_1d (u ω : Fin N → Int) :
    (∑ i : Fin N, ω i * (u i * (ω (sp i) - ω (sn i))
                       + (u (sp i) * ω (sp i) - u (sn i) * ω (sn i)))) = 0 := by
  -- Rewrite LHS as T1 − T2 + T3 − T4 via pointwise expansion.
  have split : (∑ i : Fin N, ω i * (u i * (ω (sp i) - ω (sn i))
                       + (u (sp i) * ω (sp i) - u (sn i) * ω (sn i))))
             = (∑ i : Fin N, ω i * u i * ω (sp i))
             - (∑ i : Fin N, ω i * u i * ω (sn i))
             + (∑ i : Fin N, ω i * u (sp i) * ω (sp i))
             - (∑ i : Fin N, ω i * u (sn i) * ω (sn i)) := by
    rw [← Finset.sum_sub_distrib, ← Finset.sum_add_distrib, ← Finset.sum_sub_distrib]
    apply Finset.sum_congr rfl
    intros i _; ring
  rw [split]
  -- T4 = T1 via sum_sn reindex + pointwise commutativity
  have hT4_T1 : (∑ i : Fin N, ω i * u (sn i) * ω (sn i))
              = (∑ i : Fin N, ω i * u i * ω (sp i)) := by
    calc (∑ i : Fin N, ω i * u (sn i) * ω (sn i))
        = ∑ i : Fin N, ω (sp i) * u i * ω i := by
          have h := Shift.sum_sn (fun j => ω (sp j) * u j * ω j)
          simpa [sp_sn] using h
      _ = ∑ i : Fin N, ω i * u i * ω (sp i) :=
          Finset.sum_congr rfl (fun i _ => by ring)
  -- T3 = T2 via sum_sp reindex + pointwise commutativity
  have hT3_T2 : (∑ i : Fin N, ω i * u (sp i) * ω (sp i))
              = (∑ i : Fin N, ω i * u i * ω (sn i)) := by
    calc (∑ i : Fin N, ω i * u (sp i) * ω (sp i))
        = ∑ i : Fin N, ω (sn i) * u i * ω i := by
          have h := Shift.sum_sp (fun j => ω (sn j) * u j * ω j)
          simpa [sn_sp] using h
      _ = ∑ i : Fin N, ω i * u i * ω (sn i) :=
          Finset.sum_congr rfl (fun i _ => by ring)
  rw [hT3_T2, hT4_T1]
  ring

end NSRegularity.Kill113.SBP
