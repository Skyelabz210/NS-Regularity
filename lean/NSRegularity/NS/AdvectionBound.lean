import NSRegularity.NS.FullStep
import NSRegularity.AubinLions

/-!
# NS/AdvectionBound — pointwise Lipschitz bound for discrete advection

This file proves a genuine partial result toward the full-step L∞ gate
(blocking item: "non-trivial full-step L∞ preservation"). It does **not**
close that gate — centered-difference advection lacks a discrete maximum
principle, so L∞ is not *preserved* by advection. What *is* provable, and
what this file establishes, is the sharp **Lipschitz growth bound**:

    |advective u f (i,j,k)| ≤ 2 · ‖f‖_∞ · (|u_x| + |u_y| + |u_z|)(i,j,k)

i.e. the advection operator's pointwise output is controlled by the field's
L∞ norm times the local velocity magnitude. This is the discrete analogue of
`‖u · ∇f‖_∞ ≤ ‖u‖_∞ · ‖∇f‖_∞` and is exactly the perturbation estimate that
the operator-splitting argument needs: it shows advection is *perturbatively
controlled* (the claim made in `NS/FullStep.lean`'s docstring), even though it
is not L∞-non-increasing on its own.

The full L∞ gate additionally needs: (a) the integer-division scaling by
`2·N·S`, and (b) the diffusion step's L∞ *contraction* dominating this growth
under CFL. Both remain open here; this file supplies the advection-side
estimate cleanly.
-/

namespace NSRegularity.NS.AdvectionBound

open NSRegularity NSRegularity.NS

/-- `|f i j k| ≤ gridLInf f` (Int absolute-value form of `natAbs_le_gridLInf`). -/
theorem abs_le_gridLInf (f : Grid) (i j k : Fin N) :
    |f i j k| ≤ gridLInf f := by
  rw [Int.abs_eq_natAbs]
  exact NSRegularity.AubinLions.natAbs_le_gridLInf f i j k

/-- `|diffX f| ≤ 2 · gridLInf f`. -/
theorem abs_diffX_le (f : Grid) (i j k : Fin N) :
    |NSRegularity.Kill113.diffX f i j k| ≤ 2 * gridLInf f := by
  unfold NSRegularity.Kill113.diffX
  have h1 := abs_le_gridLInf f (sp i) j k
  have h2 := abs_le_gridLInf f (sn i) j k
  have htri : |f (sp i) j k - f (sn i) j k| ≤ |f (sp i) j k| + |f (sn i) j k| :=
    abs_sub _ _
  linarith

/-- `|diffY f| ≤ 2 · gridLInf f`. -/
theorem abs_diffY_le (f : Grid) (i j k : Fin N) :
    |NSRegularity.Kill113.diffY f i j k| ≤ 2 * gridLInf f := by
  unfold NSRegularity.Kill113.diffY
  have h1 := abs_le_gridLInf f i (sp j) k
  have h2 := abs_le_gridLInf f i (sn j) k
  have htri : |f i (sp j) k - f i (sn j) k| ≤ |f i (sp j) k| + |f i (sn j) k| :=
    abs_sub _ _
  linarith

/-- `|diffZ f| ≤ 2 · gridLInf f`. -/
theorem abs_diffZ_le (f : Grid) (i j k : Fin N) :
    |NSRegularity.Kill113.diffZ f i j k| ≤ 2 * gridLInf f := by
  unfold NSRegularity.Kill113.diffZ
  have h1 := abs_le_gridLInf f i j (sp k)
  have h2 := abs_le_gridLInf f i j (sn k)
  have htri : |f i j (sp k) - f i j (sn k)| ≤ |f i j (sp k)| + |f i j (sn k)| :=
    abs_sub _ _
  linarith

/-- **Pure-arithmetic core** of the advection bound (opaque integers).

    Factored out so the heavy `gridLInf` Finset.sup' term never enters the
    arithmetic elaboration — here `L` is an opaque integer. -/
theorem advective_abs_le_aux (ux uy uz dX dY dZ L : Int)
    (hX : |dX| ≤ 2 * L) (hY : |dY| ≤ 2 * L) (hZ : |dZ| ≤ 2 * L) :
    |ux * dX + uy * dY + uz * dZ| ≤ 2 * L * (|ux| + |uy| + |uz|) := by
  have htri1 : |ux * dX + uy * dY + uz * dZ| ≤ |ux * dX + uy * dY| + |uz * dZ| :=
    abs_add_le (ux * dX + uy * dY) (uz * dZ)
  have htri2 : |ux * dX + uy * dY| ≤ |ux * dX| + |uy * dY| :=
    abs_add_le (ux * dX) (uy * dY)
  have hmX : |ux * dX| ≤ |ux| * (2 * L) := by
    rw [abs_mul]; exact mul_le_mul_of_nonneg_left hX (abs_nonneg ux)
  have hmY : |uy * dY| ≤ |uy| * (2 * L) := by
    rw [abs_mul]; exact mul_le_mul_of_nonneg_left hY (abs_nonneg uy)
  have hmZ : |uz * dZ| ≤ |uz| * (2 * L) := by
    rw [abs_mul]; exact mul_le_mul_of_nonneg_left hZ (abs_nonneg uz)
  nlinarith [htri1, htri2, hmX, hmY, hmZ]

/-- **Advection Lipschitz bound (pointwise).**

    `|advective u f (i,j,k)| ≤ 2 · ‖f‖_∞ · (|u_x| + |u_y| + |u_z|)(i,j,k)`.

    The discrete analogue of `‖u · ∇f‖ ≤ ‖u‖ · ‖∇f‖`, with `‖∇f‖` replaced by
    its sharp finite-difference bound `2·‖f‖_∞`. Reduces to
    `advective_abs_le_aux` with `L := gridLInf f` held opaque. -/
theorem advective_abs_le (u : VField) (f : Grid) (i j k : Fin N) :
    |NSRegularity.Kill113.advective u f i j k| ≤
      2 * gridLInf f * (|u.x i j k| + |u.y i j k| + |u.z i j k|) := by
  -- Hold `gridLInf f` opaque as `L` so the heavy Finset.sup' term never
  -- enters the arithmetic unification.
  obtain ⟨L, hL⟩ : ∃ L : Int, gridLInf f = L := ⟨gridLInf f, rfl⟩
  rw [hL]
  have hbX : |NSRegularity.Kill113.diffX f i j k| ≤ 2 * L := hL ▸ abs_diffX_le f i j k
  have hbY : |NSRegularity.Kill113.diffY f i j k| ≤ 2 * L := hL ▸ abs_diffY_le f i j k
  have hbZ : |NSRegularity.Kill113.diffZ f i j k| ≤ 2 * L := hL ▸ abs_diffZ_le f i j k
  exact advective_abs_le_aux (u.x i j k) (u.y i j k) (u.z i j k)
    (NSRegularity.Kill113.diffX f i j k)
    (NSRegularity.Kill113.diffY f i j k)
    (NSRegularity.Kill113.diffZ f i j k)
    L hbX hbY hbZ

/-- **Zero-velocity collapse.** When `u = 0`, the advection output is zero —
    consistent with the trivial-closure identity `advection_step 0 f = f`. -/
theorem advective_zero_velocity (f : Grid) (i j k : Fin N) :
    NSRegularity.Kill113.advective
      ⟨fun _ _ _ => 0, fun _ _ _ => 0, fun _ _ _ => 0⟩ f i j k = 0 := by
  unfold NSRegularity.Kill113.advective
  simp

/-- **Advection-step perturbation bound (pointwise).**

    The single advection sub-step moves each grid value by no more than

      `|Δt| · 2·‖f‖_∞ · (|u_x| + |u_y| + |u_z|)(i,j,k)`.

    Combines `advective_abs_le` with the integer-division contraction
    `Int.natAbs_ediv_le_natAbs` (dividing by `2·N·S` does not increase the
    absolute value). This is the discrete "how far can one advection step
    move the field" estimate — the operator-side input the full-step L∞
    argument needs to balance against the diffusion contraction. It does
    NOT by itself give L∞ preservation (the bound is a growth factor, not
    a contraction); that balance against diffusion under CFL stays open. -/
theorem advection_step_perturbation_le
    (u : VField) (f : Grid) (p : NsParams) (i j k : Fin N) :
    |advection_step u f p i j k - f i j k| ≤
      |p.dt_latt| * (2 * gridLInf f * (|u.x i j k| + |u.y i j k| + |u.z i j k|)) := by
  -- The step change is `-(Δt · advective / (2 N S))`.
  have key : advection_step u f p i j k - f i j k
      = -(p.dt_latt * NSRegularity.Kill113.advective u f i j k
            / (2 * (N : Int) * p.ns_scale)) := by
    simp only [advection_step]; ring
  rw [key, abs_neg]
  -- Dividing by `2 N S` does not increase the absolute value.
  have hdiv : |p.dt_latt * NSRegularity.Kill113.advective u f i j k
                / (2 * (N : Int) * p.ns_scale)|
      ≤ |p.dt_latt * NSRegularity.Kill113.advective u f i j k| := by
    rw [Int.abs_eq_natAbs, Int.abs_eq_natAbs]
    exact_mod_cast Int.natAbs_ediv_le_natAbs _ _
  refine le_trans hdiv ?_
  rw [abs_mul]
  exact mul_le_mul_of_nonneg_left (advective_abs_le u f i j k) (abs_nonneg _)

/-- **Stretching-step perturbation bound (pointwise).**

    The single stretching sub-step moves each grid value by no more than

      `|Δt| · |uc_grad (i,j,k)|`.

    The stretching step (`NS/FullStep.lean`) is

      `stretching_step uc_grad f p := f + Δt · uc_grad / (2·N·S)`,

    so the perturbation is `|Δt · uc_grad / (2·N·S)| ≤ |Δt · uc_grad|` by
    integer-division contraction. The third pointwise operator-side
    estimate (alongside diffusion contraction and the advection growth
    bound), completing the per-step perturbation chain that the full-step
    L∞ argument needs as input. The integer-division by `2·N·S` is the
    *only* contraction this sub-step provides — the raw `uc_grad` itself
    can be arbitrary, which is exactly why non-trivial stretching is the
    open hard part of the full-step L∞ gate. -/
theorem stretching_step_perturbation_le
    (uc_grad : Grid) (f : Grid) (p : NsParams) (i j k : Fin N) :
    |stretching_step uc_grad f p i j k - f i j k| ≤
      |p.dt_latt| * |uc_grad i j k| := by
  have key : stretching_step uc_grad f p i j k - f i j k
      = p.dt_latt * uc_grad i j k / (2 * (N : Int) * p.ns_scale) := by
    simp only [stretching_step]; ring
  rw [key]
  have hdiv : |p.dt_latt * uc_grad i j k / (2 * (N : Int) * p.ns_scale)|
      ≤ |p.dt_latt * uc_grad i j k| := by
    rw [Int.abs_eq_natAbs, Int.abs_eq_natAbs]
    exact_mod_cast Int.natAbs_ediv_le_natAbs _ _
  refine le_trans hdiv ?_
  rw [abs_mul]

/-- **Zero-stretching collapse.** Zero stretching gradient leaves the
    field unchanged - consistent with the trivial-closure identity
    `stretching_step (fun _ _ _ => 0) f p = f`. -/
theorem stretching_step_zero_perturbation
    (f : Grid) (p : NsParams) (i j k : Fin N) :
    stretching_step (fun _ _ _ => 0) f p i j k = f i j k := by
  simp [stretching_step]

end NSRegularity.NS.AdvectionBound
