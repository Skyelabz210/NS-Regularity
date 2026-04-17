import NSRegularity.Shift
import NSRegularity.Kill113.SBP.Defs

/-!
# Kill113/SBP/X — L1.5 (one leaf per file).

1D summation-by-parts in the x-direction.
Available lemmas: `Shift.sum_sp`, `Shift.sum_sn` (periodic reindex invariance).
-/

namespace NSRegularity.Kill113.SBP

open NSRegularity NSRegularity.Shift

/-- L1.5 — 1D summation-by-parts in x. -/
theorem sbp_x_1d (u ω : Fin N → Int) :
    (∑ i : Fin N, ω i * (u i * (ω (sp i) - ω (sn i))
                       + (u (sp i) * ω (sp i) - u (sn i) * ω (sn i)))) = 0 := by
  simp_rw [mul_add, mul_sub, mul_assoc]
  simp_rw [show ∀ (a b c d : Int), a * (b - c) + a * (d - e) = a * b - a * c + a * d - a * e from by
    intro a b c d e; ring]
  simp_rw [mul_comm (ω i) (u i)]
  simp_rw [← mul_assoc]
  have h1 : ∑ i : Fin N, ω i * u i * ω (sp i) = ∑ i : Fin N, u i * ω i * ω (sn i) := by
    rw [Shift.sum_sp]
  have h2 : ∑ i : Fin N, ω i * u (sp i) * ω (sp i) = ∑ i : Fin N, u i * ω i * ω (sn i) := by
    rw [Shift.sum_sp]
  have h3 : ∑ i : Fin N, ω i * u (sn i) * ω (sn i) = ∑ i : Fin N, u i * ω i * ω (sp i) := by
    rw [Shift.sum_sn]
  omega

end NSRegularity.Kill113.SBP