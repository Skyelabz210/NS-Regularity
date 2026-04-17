/-!
# Kill113/SBP — 1D summation-by-parts leaves (L1.5, L1.6, L1.7)

Each theorem is a LEAF. Agent receives: the statement below, the Shift lemmas
(sum_sp, sum_sn), and closes via reindexing + pointwise cancellation.
-/

import NSRegularity.Shift

namespace NSRegularity.Kill113.SBP

open NSRegularity

/-- Raw 1D sum helper. -/
private def sum1 (g : Fin N → Int) : Int :=
  (List.finRange N).foldl (fun a i => a + g i) 0

/-- L1.5 — 1D summation-by-parts in the x-direction.

    For any `u ω : Fin N → Int`, the 1D energy residual
    `Σ ω(i) · [ u(i)·(ω(sp i) − ω(sn i)) + (u(sp i)·ω(sp i) − u(sn i)·ω(sn i)) ]`
    vanishes identically. -/
theorem sbp_x_1d (u ω : Fin N → Int) :
    sum1 (fun i =>
      ω i * (u i * (ω (sp i) - ω (sn i))
           + (u (sp i) * ω (sp i) - u (sn i) * ω (sn i)))) = 0 := by
  sorry

/-- L1.6 — 1D summation-by-parts in the y-direction.

    Same statement structurally as L1.5 in a fixed (i, k) slice. -/
theorem sbp_y_1d (u ω : Fin N → Int) :
    sum1 (fun j =>
      ω j * (u j * (ω (sp j) - ω (sn j))
           + (u (sp j) * ω (sp j) - u (sn j) * ω (sn j)))) = 0 := by
  sorry

/-- L1.7 — 1D summation-by-parts in the z-direction. -/
theorem sbp_z_1d (u ω : Fin N → Int) :
    sum1 (fun k =>
      ω k * (u k * (ω (sp k) - ω (sn k))
           + (u (sp k) * ω (sp k) - u (sn k) * ω (sn k)))) = 0 := by
  sorry

end NSRegularity.Kill113.SBP
