import NSRegularity.NS.FullStep
import NSRegularity.AubinLions
import NSRegularity.GlobalRegularityGeneric
import NSRegularity.DKAM.DiffusionMaxPreserving

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

/-! ## Unified full-step perturbation bound -/

/-- **Full-step perturbation bound (pointwise, per component).**

    Wraps the three sub-step estimates as a single statement: the
    composed `ns_step_component` moves any single grid value `f i j k`
    by no more than the sum of the advection and stretching pointwise
    perturbations, *measured from the post-diffusion field*. Specifically

      `|ns_step_component u uc_grad f p (i,j,k) − (diffusion_step f p) (i,j,k)|
          ≤ |Δt| · 2·gridLInf (diffusion_step f p)
                 · (|u_x| + |u_y| + |u_z|)(i,j,k)
            + |Δt| · |uc_grad (i,j,k)|`

    Triangle-inequality composition of `advection_step_perturbation_le`
    (applied to the post-diffusion field) and `stretching_step_perturbation_le`
    (applied to the post-advection field). The base point is the
    post-diffusion field rather than the original `f` because the
    operator-splitting order is `stretching ∘ advection ∘ diffusion`
    (see `NS/FullStep.lean`); composing further with the diffusion L∞-
    non-increase bound (Resolution A) requires the diffusion contraction
    argument to be folded in, which stays open here.

    What this is: the *clean per-step perturbation budget* for the full
    NS sub-step composition — exactly the form a contraction-vs-growth
    argument needs to compare against the diffusion contraction. -/
theorem ns_step_component_perturbation_le
    (u : VField) (uc_grad : Grid) (f : Grid) (p : NsParams)
    (i j k : Fin N) :
    |ns_step_component u uc_grad f p i j k - diffusion_step f p i j k| ≤
      |p.dt_latt| * (2 * gridLInf (diffusion_step f p)
                      * (|u.x i j k| + |u.y i j k| + |u.z i j k|))
      + |p.dt_latt| * |uc_grad i j k| := by
  -- Let g := diffusion_step f p; a := advection_step u g p
  set g := diffusion_step f p with hg
  set a := advection_step u g p with ha
  -- ns_step_component = stretching_step uc_grad a p
  have hns : ns_step_component u uc_grad f p i j k =
             stretching_step uc_grad a p i j k := by
    simp [ns_step_component, ha, hg]
  rw [hns]
  -- |stretching - g| ≤ |stretching - a| + |a - g|
  have htri : |stretching_step uc_grad a p i j k - g i j k|
      ≤ |stretching_step uc_grad a p i j k - a i j k|
        + |a i j k - g i j k| := by
    have := abs_add_le
      (stretching_step uc_grad a p i j k - a i j k)
      (a i j k - g i j k)
    have heq : (stretching_step uc_grad a p i j k - a i j k)
                + (a i j k - g i j k)
              = stretching_step uc_grad a p i j k - g i j k := by ring
    rw [heq] at this
    exact this
  refine le_trans htri ?_
  -- bound each term
  have hadv : |a i j k - g i j k|
      ≤ |p.dt_latt| * (2 * gridLInf g
            * (|u.x i j k| + |u.y i j k| + |u.z i j k|)) := by
    have := advection_step_perturbation_le u g p i j k
    exact this
  have hstr : |stretching_step uc_grad a p i j k - a i j k|
      ≤ |p.dt_latt| * |uc_grad i j k| :=
    stretching_step_perturbation_le uc_grad a p i j k
  linarith

/-- **Refined full-step component perturbation (in terms of the *original*
    `gridLInf f`, under Resolution A).**

    Strengthens `ns_step_component_perturbation_le` by folding in the
    Resolution-A diffusion `gridLInf` non-increase from
    `gridLInf_diffusion_step_le_exact`. The bound's right-hand side is
    now expressed in `gridLInf f` rather than `gridLInf (diffusion_step f p)`,
    which is exactly the form the contraction-vs-growth argument needs
    to compare against `gridLInf f` directly.

    The bound is

      `|ns_step_component u uc_grad f p (i,j,k) − diffusion_step f p (i,j,k)|
          ≤ |Δt| · 2·gridLInf f · (|u_x| + |u_y| + |u_z|)(i,j,k)
            + |Δt| · |uc_grad (i,j,k)|`.

    Closure of the full-step L∞ gate still requires the contraction
    argument under CFL — showing the diffusion contraction on the base
    term dominates the advection + stretching growth. This commit
    eliminates one degree of freedom (`gridLInf (diffusion_step f p)`
    vs `gridLInf f`) by absorbing the diffusion contraction directly. -/
theorem ns_step_component_perturbation_le_exact
    (u : VField) (uc_grad : Grid) (f : Grid) (p : NsParams)
    (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i' j' k' : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * NSRegularity.NS.laplacian f i' j' k'))
    (i j k : Fin N) :
    |ns_step_component u uc_grad f p i j k - diffusion_step f p i j k| ≤
      |p.dt_latt| * (2 * gridLInf f
                      * (|u.x i j k| + |u.y i j k| + |u.z i j k|))
      + |p.dt_latt| * |uc_grad i j k| := by
  refine le_trans (ns_step_component_perturbation_le u uc_grad f p i j k) ?_
  -- Replace `gridLInf (diffusion_step f p)` by `gridLInf f` using Resolution A.
  have hgrid : gridLInf (diffusion_step f p) ≤ gridLInf f :=
    NSRegularity.GlobalRegularity.gridLInf_diffusion_step_le_exact
      f p h hν hdt hns_pos hExact
  -- Σ |u_c| ≥ 0; |Δt| ≥ 0; 2 ≥ 0; gridLInf_nonneg already proved.
  have hu_nn : (0 : Int) ≤ |u.x i j k| + |u.y i j k| + |u.z i j k| := by
    have hx : (0 : Int) ≤ |u.x i j k| := abs_nonneg _
    have hy : (0 : Int) ≤ |u.y i j k| := abs_nonneg _
    have hz : (0 : Int) ≤ |u.z i j k| := abs_nonneg _
    linarith
  have hdt_nn : (0 : Int) ≤ |p.dt_latt| := abs_nonneg _
  have htwo_nn : (0 : Int) ≤ (2 : Int) := by decide
  have hfactor_nn :
      (0 : Int) ≤ |p.dt_latt| * (2 * (|u.x i j k| + |u.y i j k| + |u.z i j k|)) := by
    have h2u : (0 : Int) ≤ 2 * (|u.x i j k| + |u.y i j k| + |u.z i j k|) :=
      mul_nonneg htwo_nn hu_nn
    exact mul_nonneg hdt_nn h2u
  -- Monotone in the grid factor on the right.
  have key :
      |p.dt_latt| * (2 * gridLInf (diffusion_step f p)
                      * (|u.x i j k| + |u.y i j k| + |u.z i j k|))
      ≤ |p.dt_latt| * (2 * gridLInf f
                       * (|u.x i j k| + |u.y i j k| + |u.z i j k|)) := by
    -- factor: |Δt| · 2 · (|u_x|+|u_y|+|u_z|) ≥ 0, and grid factor on the right.
    have expand_le :
        2 * gridLInf (diffusion_step f p) * (|u.x i j k| + |u.y i j k| + |u.z i j k|)
        ≤ 2 * gridLInf f * (|u.x i j k| + |u.y i j k| + |u.z i j k|) := by
      have h2g : 2 * gridLInf (diffusion_step f p) ≤ 2 * gridLInf f :=
        mul_le_mul_of_nonneg_left hgrid htwo_nn
      exact mul_le_mul_of_nonneg_right h2g hu_nn
    exact mul_le_mul_of_nonneg_left expand_le hdt_nn
  linarith

/-! ## From difference bounds to direct upper bounds -/

/-- **Direct upper bound on `|ns_step_component|`.**

    Triangle inequality applied to the perturbation bound: the absolute
    value of the composed step output is bounded by the absolute value of
    the post-diffusion base plus the perturbation budget:

      `|ns_step_component u uc_grad f p (i,j,k)|
          ≤ |diffusion_step f p (i,j,k)|
            + |Δt| · 2·gridLInf (diffusion_step f p)·(|u_x|+|u_y|+|u_z|)
            + |Δt| · |uc_grad (i,j,k)|`.

    This is the cleanest form a contraction-vs-growth argument can
    consume: the right-hand side has the post-diffusion *contraction*
    (which Resolution A bounds by `gridLInf f`) plus an explicit
    *growth* term. For L∞ preservation we need the entire RHS ≤
    `gridLInf f` pointwise — which reduces to the (open) inequality
    `Π_pert ≤ gridLInf f − |diffusion_step f p (i,j,k)|`. -/
theorem ns_step_component_abs_le
    (u : VField) (uc_grad : Grid) (f : Grid) (p : NsParams)
    (i j k : Fin N) :
    |ns_step_component u uc_grad f p i j k| ≤
      |diffusion_step f p i j k|
      + |p.dt_latt| * (2 * gridLInf (diffusion_step f p)
                       * (|u.x i j k| + |u.y i j k| + |u.z i j k|))
      + |p.dt_latt| * |uc_grad i j k| := by
  -- |x| = |(x − y) + y| ≤ |x − y| + |y|
  have htri : |ns_step_component u uc_grad f p i j k|
      ≤ |ns_step_component u uc_grad f p i j k - diffusion_step f p i j k|
        + |diffusion_step f p i j k| := by
    have habs := abs_add_le
      (ns_step_component u uc_grad f p i j k - diffusion_step f p i j k)
      (diffusion_step f p i j k)
    have heq : (ns_step_component u uc_grad f p i j k - diffusion_step f p i j k)
                + diffusion_step f p i j k
              = ns_step_component u uc_grad f p i j k := by ring
    rw [heq] at habs
    exact habs
  refine le_trans htri ?_
  have hpert := ns_step_component_perturbation_le u uc_grad f p i j k
  linarith

/-- **Full operator-side `|ns_step_component|` upper bound under Resolution A.**

    Chains `ns_step_component_abs_le` with the Resolution-A diffusion
    pointwise contraction `diffusion_max_preserving_exact` to express the
    upper bound purely in terms of `gridLInf f`:

      `|ns_step_component u uc_grad f p (i,j,k)|
          ≤ gridLInf f
            + |Δt| · 2·gridLInf f·(|u_x|+|u_y|+|u_z|)(i,j,k)
            + |Δt| · |uc_grad (i,j,k)|`.

    First two terms `gridLInf f + |Δt|·2·gridLInf f·(|u_x|+|u_y|+|u_z|)` arise
    from the diffusion contraction's pointwise output and the advection
    growth. The third is the raw stretching budget. For L∞ preservation we
    need the entire RHS ≤ `gridLInf f` pointwise, which reduces to

      `|Δt| · 2·gridLInf f·(|u_x|+|u_y|+|u_z|) + |Δt|·|uc_grad (i,j,k)| ≤ 0`.

    Since each summand is non-negative, this collapses to the trivial
    case (`u = 0` AND `uc_grad = 0`). That is exactly the trivial-closure
    family `NS/TrivialClosure.lean` discharges. Non-trivial L∞ preservation
    requires a tighter analysis that does NOT bound the diffusion term by
    its sup but instead uses its *spare* contraction capacity at each grid
    point relative to the local advection + stretching pressure. This
    capacity-vs-pressure inequality is the genuinely-open hard piece. -/
theorem ns_step_component_abs_le_exact
    (u : VField) (uc_grad : Grid) (f : Grid) (p : NsParams)
    (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i' j' k' : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * NSRegularity.NS.laplacian f i' j' k'))
    (i j k : Fin N) :
    |ns_step_component u uc_grad f p i j k| ≤
      gridLInf f
      + |p.dt_latt| * (2 * gridLInf f
                       * (|u.x i j k| + |u.y i j k| + |u.z i j k|))
      + |p.dt_latt| * |uc_grad i j k| := by
  -- Chain the refined perturbation bound (under Res-A) with the pointwise
  -- |diffusion_step ...| ≤ gridLInf f from diffusion_max_preserving_exact.
  have hpert := ns_step_component_perturbation_le_exact
    u uc_grad f p h hν hdt hns_pos hExact i j k
  have hdiff : |diffusion_step f p i j k| ≤ gridLInf f :=
    NSRegularity.DKAM.Diffusion.diffusion_max_preserving_exact
      f p h hν hdt hns_pos hExact i j k
  -- |ns_step_component| ≤ |diffusion_step| + Π (refined) ≤ gridLInf f + Π.
  have habs : |ns_step_component u uc_grad f p i j k|
      ≤ |ns_step_component u uc_grad f p i j k - diffusion_step f p i j k|
        + |diffusion_step f p i j k| := by
    have h := abs_add_le
      (ns_step_component u uc_grad f p i j k - diffusion_step f p i j k)
      (diffusion_step f p i j k)
    have heq : (ns_step_component u uc_grad f p i j k - diffusion_step f p i j k)
                + diffusion_step f p i j k
              = ns_step_component u uc_grad f p i j k := by ring
    rw [heq] at h
    exact h
  linarith

/-! ## Capacity-vs-pressure conditional closure -/

/-- **Local diffusion contraction capacity** at a grid point:
    the spare contraction room the diffusion step leaves at `(i,j,k)`. -/
def localCapacity (f : Grid) (p : NsParams) (i j k : Fin N) : Int :=
  gridLInf f - |diffusion_step f p i j k|

/-- **Local perturbation pressure** at a grid point: the per-step growth
    budget the advection + stretching operators exert. -/
def localPressure (u : VField) (uc_grad : Grid) (f : Grid) (p : NsParams)
    (i j k : Fin N) : Int :=
  |p.dt_latt| * (2 * gridLInf f * (|u.x i j k| + |u.y i j k| + |u.z i j k|))
  + |p.dt_latt| * |uc_grad i j k|

/-- **Local capacity is non-negative under Resolution A.** -/
theorem localCapacity_nonneg
    (f : Grid) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i' j' k' : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * NSRegularity.NS.laplacian f i' j' k'))
    (i j k : Fin N) :
    0 ≤ localCapacity f p i j k := by
  unfold localCapacity
  have h_diff : |diffusion_step f p i j k| ≤ gridLInf f :=
    NSRegularity.DKAM.Diffusion.diffusion_max_preserving_exact
      f p h hν hdt hns_pos hExact i j k
  linarith

set_option maxHeartbeats 800000 in
/-- `gridLInf f ≥ 0` (Int form). Direct via `le_trans`. -/
theorem gridLInf_nonneg_int (f : Grid) : (0 : Int) ≤ gridLInf f := by
  have h1 : (0 : Int) ≤ ((f 0 0 0).natAbs : Int) := Int.natCast_nonneg _
  exact le_trans h1 (NSRegularity.AubinLions.natAbs_le_gridLInf f 0 0 0)

/-- Pure-arithmetic core for the conditional-capacity L∞ preservation. -/
theorem lInf_preserved_of_capacity_aux
    (A B D P G : Int)
    (habs : A ≤ B + D) (hperturb : B ≤ P) (hcap : P ≤ G - D) :
    A ≤ G := by
  linarith

/-- Pure-arithmetic core: the sum of two non-negative product terms is
    non-negative. Opaque integers to avoid whnf blowup on heavy terms. -/
theorem localPressure_nonneg_aux
    (D G UX UY UZ UC : Int)
    (hD : 0 ≤ D) (hG : 0 ≤ G)
    (hUX : 0 ≤ UX) (hUY : 0 ≤ UY) (hUZ : 0 ≤ UZ) (hUC : 0 ≤ UC) :
    0 ≤ D * (2 * G * (UX + UY + UZ)) + D * UC := by
  have h_uxyz : 0 ≤ UX + UY + UZ := by linarith
  have h_2G : 0 ≤ 2 * G := by linarith
  have h_2G_u : 0 ≤ 2 * G * (UX + UY + UZ) := mul_nonneg h_2G h_uxyz
  have hA : 0 ≤ D * (2 * G * (UX + UY + UZ)) := mul_nonneg hD h_2G_u
  have hB : 0 ≤ D * UC := mul_nonneg hD hUC
  linarith

/-- **Local pressure is non-negative.** -/
theorem localPressure_nonneg
    (u : VField) (uc_grad : Grid) (f : Grid) (p : NsParams)
    (i j k : Fin N) :
    0 ≤ localPressure u uc_grad f p i j k := by
  unfold localPressure
  exact localPressure_nonneg_aux
    |p.dt_latt| (gridLInf f)
    |u.x i j k| |u.y i j k| |u.z i j k| |uc_grad i j k|
    (abs_nonneg _) (gridLInf_nonneg_int f)
    (abs_nonneg _) (abs_nonneg _) (abs_nonneg _) (abs_nonneg _)

/-- **Uniform upper bound on local pressure.**

    The local pressure at any grid point is bounded by

      `|Δt| · (6·gridLInf f·M_u + M_uc)`

    where `M_u = max(gridLInf u.x, gridLInf u.y, gridLInf u.z)` and
    `M_uc = gridLInf uc_grad`. The factor 6 = 2 × 3 comes from the
    advection stencil constant 2 times the three components of u. -/
theorem localPressure_uniform_le_aux
    (Δt G M_u M_uc UX UY UZ UC : Int)
    (hΔt : 0 ≤ Δt) (hG : 0 ≤ G) (hM_u : 0 ≤ M_u)
    (hUX_le : UX ≤ M_u) (hUY_le : UY ≤ M_u) (hUZ_le : UZ ≤ M_u)
    (hUC_le : UC ≤ M_uc)
    (hUX_nn : 0 ≤ UX) (hUY_nn : 0 ≤ UY) (hUZ_nn : 0 ≤ UZ) (hUC_nn : 0 ≤ UC) :
    Δt * (2 * G * (UX + UY + UZ)) + Δt * UC ≤ Δt * (6 * G * M_u + M_uc) := by
  -- UX + UY + UZ ≤ 3 · M_u
  have h_uxyz_le : UX + UY + UZ ≤ 3 * M_u := by linarith
  have h_uxyz_nn : 0 ≤ UX + UY + UZ := by linarith
  -- 2 · G · (UX + UY + UZ) ≤ 2 · G · (3 · M_u) = 6 · G · M_u
  have h2G_nn : 0 ≤ 2 * G := by linarith
  have hadv : 2 * G * (UX + UY + UZ) ≤ 2 * G * (3 * M_u) :=
    mul_le_mul_of_nonneg_left h_uxyz_le h2G_nn
  have hadv' : 2 * G * (3 * M_u) = 6 * G * M_u := by ring
  -- Δt · (2 G ·(UX+UY+UZ)) + Δt · UC ≤ Δt · (6·G·M_u) + Δt · M_uc
  have hA : Δt * (2 * G * (UX + UY + UZ)) ≤ Δt * (6 * G * M_u) := by
    have : Δt * (2 * G * (UX + UY + UZ)) ≤ Δt * (2 * G * (3 * M_u)) :=
      mul_le_mul_of_nonneg_left hadv hΔt
    rw [hadv'] at this; exact this
  have hB : Δt * UC ≤ Δt * M_uc := mul_le_mul_of_nonneg_left hUC_le hΔt
  have hsum : Δt * (6 * G * M_u + M_uc) = Δt * (6 * G * M_u) + Δt * M_uc := by ring
  linarith

/-- **Local-capacity uniform lower bound — pure-integer arithmetic core.**

    If the diffusion-step output's absolute value at every grid point is
    uniformly bounded above by some `D_max` (the "post-diffusion ceiling"),
    then the local capacity is uniformly bounded below by
    `gridLInf f − D_max`. In particular if `D_max ≤ gridLInf f` (which
    Resolution A guarantees with `D_max = gridLInf f` as the trivial
    upper bound), the lower bound is non-negative. -/
theorem localCapacity_uniform_ge_aux
    (G D D_max : Int) (hD_le : D ≤ D_max) :
    G - D_max ≤ G - D := by linarith

/-- **CFL-style sufficient condition — pure-integer arithmetic core.**

    If the uniform pressure upper bound is dominated by a uniform lower
    bound on the capacity, then the per-grid-point capacity-vs-pressure
    inequality holds at every grid point:

      pressure_uniform ≤ capacity_uniform
      ∧ pressure_local ≤ pressure_uniform
      ∧ capacity_uniform ≤ capacity_local
      ⟹ pressure_local ≤ capacity_local.

    This is the reduction the next attack on the full-step L∞ gate must
    target: a single scalar inequality on uniform L∞-norms of (u, uc_grad,
    f, p) rather than a per-grid-point pointwise hypothesis. -/
theorem capacity_vs_pressure_sufficient_aux
    (pressure_local capacity_local pressure_uniform capacity_uniform : Int)
    (h_p_le : pressure_local ≤ pressure_uniform)
    (h_c_ge : capacity_uniform ≤ capacity_local)
    (h_ineq : pressure_uniform ≤ capacity_uniform) :
    pressure_local ≤ capacity_local := by linarith

/-- **Explicit small-data sufficient condition — pure-integer arithmetic core.**

    The CFL-style inequality `pressure_uniform ≤ capacity_uniform` factors
    through an explicit small-data threshold: if there exists a scalar `K`
    bounding the velocity / stretching contribution

      `6·G·M_u + M_uc ≤ K`

    and the time-step is small enough relative to the spare capacity

      `|Δt|·K ≤ G − D_max`

    then the full uniform inequality

      `|Δt|·(6·G·M_u + M_uc) ≤ G − D_max`

    holds. This is the explicit form a small-data closure of the L∞ gate
    takes: bound the velocity and stretching budget by a scalar `K`, then
    pick the time-step small enough that `|Δt|·K` fits inside the
    diffusion's spare contraction capacity. The two pieces decouple
    cleanly: one is a bound on the operator data, the other a CFL-style
    refinement on `|Δt|`. -/
theorem small_data_sufficient_aux
    (Δt G M_u M_uc D_max K : Int)
    (hΔt : 0 ≤ Δt) (h_K_bound : 6 * G * M_u + M_uc ≤ K)
    (h_data_pos : 0 ≤ 6 * G * M_u + M_uc)
    (h_K_nn : 0 ≤ K)
    (h_cfl : Δt * K ≤ G - D_max) :
    Δt * (6 * G * M_u + M_uc) ≤ G - D_max := by
  have h_data_le_K : Δt * (6 * G * M_u + M_uc) ≤ Δt * K :=
    mul_le_mul_of_nonneg_left h_K_bound hΔt
  linarith

/-! ### Capacity-vs-pressure conditional L∞ closure — composition recipe

    The conditional closure of the per-component L∞ gate is a one-step
    composition the caller can do inline using the building blocks above:

      ```
      example (u uc_grad f p)
          (h_cap : ∀ i j k, localPressure u uc_grad f p i j k
                            ≤ localCapacity f p i j k)
          (h hν hdt hns_pos hExact)
          (i j k : Fin N) :
          |ns_step_component u uc_grad f p i j k| ≤ gridLInf f := by
        have hperturb := ns_step_component_perturbation_le u uc_grad f p i j k
        have habs : |ns_step_component u uc_grad f p i j k|
            ≤ |ns_step_component u uc_grad f p i j k - diffusion_step f p i j k|
              + |diffusion_step f p i j k| := abs_add_le ...
        have hcap_ijk := h_cap i j k
        exact lInf_preserved_of_capacity_aux
          _ _ _ _ _ habs hperturb hcap_ijk
      ```

    The packaged form is not provided as a top-level theorem here because
    the wrapper definitions `localPressure` / `localCapacity` unfold to
    `gridLInf f`-bearing expressions that trigger an elaboration-time
    whnf blowup when stated together in a single signature, even at
    raised heartbeat budgets. The arithmetic-core helper
    `lInf_preserved_of_capacity_aux` exists precisely so the composition
    is one `exact` line at the use site without re-triggering the
    elaboration overhead. The remaining open piece is `h_cap` itself:
    the capacity-vs-pressure inequality at every grid point under
    physically reasonable hypotheses. -/

/-! ## VField-level lift -/

/-- **VField-level full-step perturbation (per-component decomposition).**

    The composed `ns_step` differs from the (vector) `diffusion_vstep p w`
    by at most the sum of three per-component perturbations, each one a
    `ns_step_component_perturbation_le` instance.

    For each component `c ∈ {x, y, z}` and grid point `(i,j,k)`:

      `|(ns_step u sx sy sz w p).c (i,j,k) − (diffusion_vstep p w).c (i,j,k)|
          ≤ |Δt| · 2 · gridLInf ((diffusion_vstep p w).c)
                  · (|u_x| + |u_y| + |u_z|)(i,j,k)
            + |Δt| · |sc (i,j,k)|`

    where `sc ∈ {sx, sy, sz}` is the matching stretching gradient. This
    lifts `ns_step_component_perturbation_le` to the VField level cleanly,
    so any future contraction argument can address the three components
    independently. -/
theorem ns_step_x_perturbation_le
    (u : VField) (sx sy sz : Grid) (w : VField) (p : NsParams)
    (i j k : Fin N) :
    |(ns_step u sx sy sz w p).x i j k - diffusion_step w.x p i j k| ≤
      |p.dt_latt| * (2 * gridLInf (diffusion_step w.x p)
                      * (|u.x i j k| + |u.y i j k| + |u.z i j k|))
      + |p.dt_latt| * |sx i j k| := by
  show |ns_step_component u sx w.x p i j k - diffusion_step w.x p i j k| ≤ _
  exact ns_step_component_perturbation_le u sx w.x p i j k

theorem ns_step_y_perturbation_le
    (u : VField) (sx sy sz : Grid) (w : VField) (p : NsParams)
    (i j k : Fin N) :
    |(ns_step u sx sy sz w p).y i j k - diffusion_step w.y p i j k| ≤
      |p.dt_latt| * (2 * gridLInf (diffusion_step w.y p)
                      * (|u.x i j k| + |u.y i j k| + |u.z i j k|))
      + |p.dt_latt| * |sy i j k| := by
  show |ns_step_component u sy w.y p i j k - diffusion_step w.y p i j k| ≤ _
  exact ns_step_component_perturbation_le u sy w.y p i j k

theorem ns_step_z_perturbation_le
    (u : VField) (sx sy sz : Grid) (w : VField) (p : NsParams)
    (i j k : Fin N) :
    |(ns_step u sx sy sz w p).z i j k - diffusion_step w.z p i j k| ≤
      |p.dt_latt| * (2 * gridLInf (diffusion_step w.z p)
                      * (|u.x i j k| + |u.y i j k| + |u.z i j k|))
      + |p.dt_latt| * |sz i j k| := by
  show |ns_step_component u sz w.z p i j k - diffusion_step w.z p i j k| ≤ _
  exact ns_step_component_perturbation_le u sz w.z p i j k

/-! ## Open content as machine-readable hypothesis signatures

    These are the EXACT remaining open theorems whose proofs would close
    the full-step L∞ gate. They are stated as comments rather than axioms
    to avoid adding to the trust base; a future closure attempt would
    convert them to `theorem`/`lemma` declarations.

    ```
    -- Piece A: uniform data bound (open)
    theorem Piece_A_data_bound
        (u : VField) (uc_grad : Grid) (f : Grid)
        : ∃ K : Int, 0 ≤ K ∧
          6 * gridLInf f *
            (max (gridLInf u.x) (max (gridLInf u.y) (gridLInf u.z)))
          + gridLInf uc_grad ≤ K

    -- Piece B: CFL refinement (open)
    theorem Piece_B_cfl_refinement
        (f : Grid) (p : NsParams) (K : Int)
        (hK : 0 ≤ K)
        : ∃ D_max : Int, (∀ i j k, |diffusion_step f p i j k| ≤ D_max)
                       ∧ |p.dt_latt| * K ≤ gridLInf f - D_max
    ```

    Composition recipe (given both Pieces):

    ```
    theorem h_step_closure
        (u : VField) (uc_grad : Grid) (f : Grid) (p : NsParams)
        (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
        (hns_pos : 0 < p.ns_scale)
        (hExact : ∀ i j k, … Res-A exactness …)
        (hA : Piece_A_data_bound u uc_grad f)
        (hB : ∀ K, 0 ≤ K → Piece_B_cfl_refinement f p K)
        : gridLInf (ns_step_component u uc_grad f p) ≤ gridLInf f := by
      obtain ⟨K, hK_nn, hK_bound⟩ := hA
      obtain ⟨D_max, hD_bound, hCFL⟩ := hB K hK_nn
      intro i j k
      -- compose via small_data_sufficient_aux + capacity_vs_pressure_sufficient_aux
      --       + lInf_preserved_of_capacity_aux + ns_step_component_perturbation_le
      sorry  -- this composition step is provable from the aux helpers;
             -- left as `sorry` to keep the file axiom-free.
    ```

    The `sorry` above is intentional and isolated: it would unfold by
    direct invocation of the named aux helpers proved earlier in this
    file. Future PRs replacing it with the explicit composition is a
    bookkeeping exercise. The substantive open content is Piece_A and
    Piece_B above. -/

/-! ## Concrete sanity checks

    Decidable-arithmetic instances of the aux helpers on concrete integer
    inputs, evaluated at elaboration time via `#guard`. These confirm the
    arithmetic cores produce the right numeric outputs - they are
    type-checking smoke tests, not theorems. -/

example :
    -- localPressure_nonneg_aux: 0 ≤ Δt·(2·G·(UX+UY+UZ)) + Δt·UC
    -- Δt=2, G=10, UX=1, UY=1, UZ=1, UC=5
    (0 : Int) ≤ 2 * (2 * 10 * (1 + 1 + 1)) + 2 * 5 := by decide

example :
    -- lInf_preserved_of_capacity_aux: A ≤ G when A ≤ B + D, B ≤ P, P ≤ G − D
    -- With A=7, B=3, D=4, P=3, G=7: 7 ≤ 3+4 ✓, 3 ≤ 3 ✓, 3 ≤ 7−4 ✓, conclude 7 ≤ 7 ✓
    let A : Int := 7; let B : Int := 3; let D : Int := 4
    let P : Int := 3; let G : Int := 7
    A ≤ B + D ∧ B ≤ P ∧ P ≤ G - D ∧ A ≤ G := by decide

example :
    -- localPressure_uniform_le_aux concrete: with the same data
    -- bound by 2·(6·10·1 + 5) = 130
    -- pressure = 2·(2·10·3) + 2·5 = 120 + 10 = 130; 130 ≤ 130 ✓
    (2 * (2 * 10 * (1 + 1 + 1)) + 2 * 5 : Int) ≤ 2 * (6 * 10 * 1 + 5) := by decide

example :
    -- capacity_vs_pressure_sufficient_aux: with the same data
    -- p_uniform = 130, c_uniform = 130, p_uniform ≤ c_uniform ✓ ⟹ p_local ≤ c_local
    let p_local : Int := 100; let c_local : Int := 150
    let p_uniform : Int := 130; let c_uniform : Int := 130
    (p_local ≤ p_uniform ∧ c_uniform ≤ c_local ∧ p_uniform ≤ c_uniform)
      → p_local ≤ c_local := by decide

example :
    -- small_data_sufficient_aux: K=20, Δt=2, G=10, M_u=1, M_uc=5, D_max=−30
    -- Want: Δt·(6·G·M_u + M_uc) ≤ G − D_max
    -- LHS = 2·(60 + 5) = 130; G − D_max = 10 − (−30) = 40
    -- With negative D_max representing room for growth, both bounds hold:
    --   data bound: 6·G·M_u + M_uc = 65 ≤ K = 70 (extra margin)
    --   CFL:        Δt·K = 140 ≤ G − D_max = 40? FALSE — must pick smaller Δt.
    -- Adjusted: Δt=1, K=10 ⟹ 6·G·M_u + M_uc = 65 ≤ K? FALSE; K=70 ⟹ Δt·K=70 > 40. FALSE.
    -- Correct concrete witness: D_max = −60, G − D_max = 70
    -- Δt=1, M_u=1, M_uc=5 ⟹ 6·10·1 + 5 = 65 ≤ K=65, Δt·K=65 ≤ 70 ✓
    (1 * (6 * 10 * 1 + 5) : Int) ≤ 10 - (-60) := by decide

end NSRegularity.NS.AdvectionBound
