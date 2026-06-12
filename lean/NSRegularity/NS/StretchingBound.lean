import NSRegularity.NS.AdvectionBound

/-!
# NS/StretchingBound — Attack 2: the stretching gradient bound (Piece A, half)

The stretching input `uc_grad` of `ns_step_component` is the vorticity
transporting a velocity component: `uc_grad = advective ω u_c` (the discrete
`(ω·∇)u_c`). This file proves the L∞ bound

```
gridLInf (advective ω u_c) ≤ 6 · gridLInf u_c · lInf ω
```

by instantiating the proved advection Lipschitz bound `advective_abs_le`
with the roles swapped (transporter := ω, transported := u_c) and lifting
pointwise → `gridLInf` via the `Finset.sup'_le` pattern.

## DAG payoff

Piece A of the small-data closure (`h_data_bound` in `HStepSmallData.lean`)
had two open inputs: the Biot-Savart L∞ bound and the stretching gradient
bound. This file closes the second. What remains of Piece A is the single
statement

```
∀ c ∈ {x,y,z}, gridLInf ((biot_savart ω).c) ≤ M_u      (open, NTT port)
```

With that bound, `piece_A_K_twelve` below gives the explicit closed-form
data constant `K = 12 · M_u · lInf w`, so the CFL condition (Piece B₂)
takes the quadratic small-data shape

```
|Δt| · 12 · C_BS · (lInf w)² ≤ gridLInf w.c − D_max
```

— exactly the `ε₀(ν, N) > ‖ω₀‖²` threshold structure of the small-data
regularity theorem.
-/

namespace NSRegularity.NS.StretchingBound

open NSRegularity NSRegularity.NS NSRegularity.NS.AdvectionBound

/-! ## lInf basics -/

/-- Component bound: `gridLInf w.x ≤ lInf w`. -/
theorem gridLInf_x_le_lInf (w : VField) : gridLInf w.x ≤ lInf w :=
  le_max_left _ _

/-- Component bound: `gridLInf w.y ≤ lInf w`. -/
theorem gridLInf_y_le_lInf (w : VField) : gridLInf w.y ≤ lInf w :=
  le_trans (le_max_left _ _) (le_max_right _ _)

/-- Component bound: `gridLInf w.z ≤ lInf w`. -/
theorem gridLInf_z_le_lInf (w : VField) : gridLInf w.z ≤ lInf w :=
  le_trans (le_max_right _ _) (le_max_right _ _)

/-- `lInf` is non-negative (each `gridLInf` is a sup of `natAbs` values). -/
theorem lInf_nonneg_int (w : VField) : 0 ≤ lInf w :=
  le_trans (gridLInf_nonneg_int w.x) (gridLInf_x_le_lInf w)

/-- Pointwise component value bounded by the vector-field L∞. -/
theorem abs_x_le_lInf (w : VField) (i j k : Fin N) : |w.x i j k| ≤ lInf w :=
  le_trans (abs_le_gridLInf w.x i j k) (gridLInf_x_le_lInf w)

theorem abs_y_le_lInf (w : VField) (i j k : Fin N) : |w.y i j k| ≤ lInf w :=
  le_trans (abs_le_gridLInf w.y i j k) (gridLInf_y_le_lInf w)

theorem abs_z_le_lInf (w : VField) (i j k : Fin N) : |w.z i j k| ≤ lInf w :=
  le_trans (abs_le_gridLInf w.z i j k) (gridLInf_z_le_lInf w)

/-! ## Pointwise → gridLInf lift (generic) -/

/-- Generic lift: a uniform pointwise absolute bound gives a `gridLInf`
    bound. The `Finset.sup'_le` pattern from `HStepSmallData`, factored
    out for reuse. -/
theorem gridLInf_le_of_pointwise_abs (g : Grid) (B : Int)
    (h : ∀ i j k : Fin N, |g i j k| ≤ B) : gridLInf g ≤ B := by
  unfold gridLInf
  apply Finset.sup'_le; intro i _
  apply Finset.sup'_le; intro j _
  apply Finset.sup'_le; intro k _
  have hp := h i j k
  rwa [Int.abs_eq_natAbs] at hp

/-! ## Attack 2 — the stretching gradient bound -/

/-- **Stretching Lipschitz bound (pointwise).**

    `|advective ω u_c (i,j,k)| ≤ 6 · gridLInf u_c · lInf ω`

    The discrete `‖(ω·∇)u_c‖_∞ ≤ ‖∇u_c‖_∞ · 3‖ω‖_∞` with the sharp
    centered-difference bound `‖∇u_c‖_∞ ≤ 2·‖u_c‖_∞`. Instantiates
    `advective_abs_le` with the transporter and transported swapped
    relative to the advection use. -/
theorem stretching_abs_le (ω : VField) (uc : Grid) (i j k : Fin N) :
    |NSRegularity.Kill113.advective ω uc i j k| ≤
      6 * gridLInf uc * lInf ω := by
  have h1 := advective_abs_le ω uc i j k
  have hsum : |ω.x i j k| + |ω.y i j k| + |ω.z i j k| ≤ 3 * lInf ω := by
    linarith [abs_x_le_lInf ω i j k, abs_y_le_lInf ω i j k, abs_z_le_lInf ω i j k]
  have hG : (0 : Int) ≤ gridLInf uc := gridLInf_nonneg_int uc
  have h2 : 2 * gridLInf uc * (|ω.x i j k| + |ω.y i j k| + |ω.z i j k|)
      ≤ 2 * gridLInf uc * (3 * lInf ω) :=
    mul_le_mul_of_nonneg_left hsum (by linarith)
  have h3 : 2 * gridLInf uc * (3 * lInf ω) = 6 * gridLInf uc * lInf ω := by ring
  linarith

/-- **Stretching gradient bound (gridLInf).**

    `gridLInf (advective ω u_c) ≤ 6 · gridLInf u_c · lInf ω`

    Attack 2 of the DAG, closed. -/
theorem gridLInf_stretching_le (ω : VField) (uc : Grid) :
    gridLInf (fun i j k => NSRegularity.Kill113.advective ω uc i j k)
      ≤ 6 * gridLInf uc * lInf ω :=
  gridLInf_le_of_pointwise_abs _ _ (fun i j k => stretching_abs_le ω uc i j k)

/-! ## Piece A reduction — only the Biot-Savart bound remains open -/

/-- **Piece A, explicit form.** If the velocity component is bounded
    (`gridLInf u_c ≤ M_u`, the Biot-Savart L∞ bound — the single remaining
    open input), then the data bound of
    `h_step_component_pointwise_small_data` holds with the explicit
    constant `K := 6·gridLInf f·M_u + 6·M_u·lInf ω`. -/
theorem piece_A_data_bound (ω : VField) (uc f : Grid) (M_u : Int)
    (h_uc : gridLInf uc ≤ M_u) :
    6 * gridLInf f * M_u
      + gridLInf (fun i j k => NSRegularity.Kill113.advective ω uc i j k)
    ≤ 6 * gridLInf f * M_u + 6 * M_u * lInf ω := by
  have h1 := gridLInf_stretching_le ω uc
  have hω : (0 : Int) ≤ lInf ω := lInf_nonneg_int ω
  have h2 : 6 * gridLInf uc * lInf ω ≤ 6 * M_u * lInf ω := by
    have := mul_le_mul_of_nonneg_right h_uc hω
    linarith
  linarith

/-- **Piece A, closed form for the NS step.** When the transported field is
    a component of the vorticity `w` itself (as in the NS step, where the
    stretching is `(w·∇)u_c` and the diffused field is `w.c`), the data
    bound closes with the single constant

    `K = 12 · M_u · lInf w`.

    The CFL condition of Piece B₂ then reads
    `|Δt| · 12 · M_u · lInf w ≤ gridLInf w.c − D_max`, and with the
    Biot-Savart bound `M_u = C_BS · lInf w` this is the quadratic
    small-data threshold `|Δt| · 12 · C_BS · (lInf w)² ≤ gridLInf w.c − D_max`. -/
theorem piece_A_K_twelve (w : VField) (uc : Grid) (M_u : Int)
    (hM_u : 0 ≤ M_u) (h_uc : gridLInf uc ≤ M_u) :
    6 * gridLInf w.x * M_u
      + gridLInf (fun i j k => NSRegularity.Kill113.advective w uc i j k)
    ≤ 12 * M_u * lInf w := by
  have h1 := gridLInf_stretching_le w uc
  have hw : (0 : Int) ≤ lInf w := lInf_nonneg_int w
  have h2 : 6 * gridLInf uc * lInf w ≤ 6 * M_u * lInf w := by
    have := mul_le_mul_of_nonneg_right h_uc hw
    linarith
  have h3 : 6 * gridLInf w.x * M_u ≤ 6 * lInf w * M_u := by
    have := mul_le_mul_of_nonneg_right (gridLInf_x_le_lInf w) hM_u
    linarith
  linarith

end NSRegularity.NS.StretchingBound
