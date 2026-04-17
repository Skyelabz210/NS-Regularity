import NSRegularity.Kill113.Ops
import NSRegularity.Kill113.SBP

/-!
# Kill113/Component — single-component energy residual (L1.8)

Strategy: split the summand into three axis-aligned pieces; for each, swap
the triple sum so the direction-axis is innermost; the inner sum is then
exactly the 1D SBP identity.  Sum of three zeros is zero.
-/

namespace NSRegularity.Kill113

open NSRegularity NSRegularity.Kill113.SBP

/-- L1.8 — single-component energy residual is zero. -/
theorem component_residual_zero (u : VField) (ωc : Grid) :
    sumGrid (fun i j k =>
      ωc i j k * (advective u ωc i j k + conservative u ωc i j k)) = 0 := by
  -- Reshape the summand into X + Y + Z, each a `sbp_x_1d`-shaped factor
  -- in exactly one of the three axes.
  have reshape : ∀ i j k : Fin N,
      ωc i j k * (advective u ωc i j k + conservative u ωc i j k)
    = (ωc i j k * (u.x i j k * (ωc (sp i) j k - ωc (sn i) j k)
                  + (u.x (sp i) j k * ωc (sp i) j k - u.x (sn i) j k * ωc (sn i) j k)))
    + (ωc i j k * (u.y i j k * (ωc i (sp j) k - ωc i (sn j) k)
                  + (u.y i (sp j) k * ωc i (sp j) k - u.y i (sn j) k * ωc i (sn j) k)))
    + (ωc i j k * (u.z i j k * (ωc i j (sp k) - ωc i j (sn k))
                  + (u.z i j (sp k) * ωc i j (sp k) - u.z i j (sn k) * ωc i j (sn k))))
    := by
      intros i j k
      unfold advective conservative diffX diffY diffZ
      ring
  -- Expand sumGrid and reshape.
  unfold sumGrid
  simp_rw [reshape, Finset.sum_add_distrib]
  -- Three triple sums; show each is 0.
  -- X axis: inner loop is over i with j,k fixed — exactly sbp_x_1d.
  have zeroX : (∑ i : Fin N, ∑ j : Fin N, ∑ k : Fin N,
      ωc i j k * (u.x i j k * (ωc (sp i) j k - ωc (sn i) j k)
                + (u.x (sp i) j k * ωc (sp i) j k - u.x (sn i) j k * ωc (sn i) j k))) = 0 := by
    rw [Finset.sum_comm (s := Finset.univ) (t := Finset.univ)]
    refine Finset.sum_eq_zero ?_
    intros j _
    rw [Finset.sum_comm (s := Finset.univ) (t := Finset.univ)]
    refine Finset.sum_eq_zero ?_
    intros k _
    exact sbp_x_1d (fun i => u.x i j k) (fun i => ωc i j k)
  -- Y axis: swap so j is innermost, then apply sbp.
  have zeroY : (∑ i : Fin N, ∑ j : Fin N, ∑ k : Fin N,
      ωc i j k * (u.y i j k * (ωc i (sp j) k - ωc i (sn j) k)
                + (u.y i (sp j) k * ωc i (sp j) k - u.y i (sn j) k * ωc i (sn j) k))) = 0 := by
    refine Finset.sum_eq_zero ?_
    intros i _
    rw [Finset.sum_comm (s := Finset.univ) (t := Finset.univ)]
    refine Finset.sum_eq_zero ?_
    intros k _
    exact sbp_x_1d (fun j => u.y i j k) (fun j => ωc i j k)
  -- Z axis: j is the middle so already innermost for k; apply sbp directly on innermost.
  have zeroZ : (∑ i : Fin N, ∑ j : Fin N, ∑ k : Fin N,
      ωc i j k * (u.z i j k * (ωc i j (sp k) - ωc i j (sn k))
                + (u.z i j (sp k) * ωc i j (sp k) - u.z i j (sn k) * ωc i j (sn k)))) = 0 := by
    refine Finset.sum_eq_zero ?_
    intros i _
    refine Finset.sum_eq_zero ?_
    intros j _
    exact sbp_x_1d (fun k => u.z i j k) (fun k => ωc i j k)
  rw [zeroX, zeroY, zeroZ]
  ring

end NSRegularity.Kill113
