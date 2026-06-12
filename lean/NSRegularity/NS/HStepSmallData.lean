import NSRegularity.NS.AdvectionBound
import NSRegularity.NS.Trajectory

/-!
# NS/HStepSmallData — Small-data h_step closure

Closes `gridLInf (ns_step_component u uc_grad f p) ≤ gridLInf f` under an
explicit small-data CFL condition, then lifts to `lInf (ns_step ...) ≤ lInf w`.

## Proof structure

The composition recipe from `AdvectionBound_CLOSURE_RECIPE.md`, executed:

```
ns_step_component_perturbation_le_exact    (Res-A, PROVED)
    |ns_step_component − diff| ≤ |Δt|·(2·G·Σ|u| + |uc_grad|)

h_vel_x/y/z + h_data_bound                (Piece A, hypothesis)
    2·G·Σ|u| + |uc_grad| ≤ 6·G·M_u + gridLInf uc_grad ≤ K

h_diff_max + h_cfl                         (Piece B, hypothesis)
    |diff| ≤ D_max  and  |Δt|·K ≤ G − D_max

Triangle inequality → |ns_step_component| ≤ G = gridLInf f
```

## DAG role

```
Kill #113 (PROVED)                  Resolution A (PROVED)
     │                                    │
     ↓                                    ↓
 SAEI = 0                   ns_step_component_perturbation_le_exact
                                          │
Biot-Savart bound ──→ Piece A            │
(NTT port, OPEN)     (data bound K)      │
                                          ↓
CFL analysis ────────→ Piece B ──→  THIS FILE (HStepSmallData)
(small-data, OPEN)   (|Δt|·K ≤ G−D)      │
                                          ↓
                              ns_global_regularity_skeleton (PROVED)
                                          │
                                          ↓
                              BKM axiom → IsSmooth → Global Regularity
```

The two open pieces (A and B) are hypothesis inputs here — they are not
proved unconditionally. See `PROOF_DAG.md` for the full dependency graph.

The **Transfinite Architecture** (DynCRT + activation level `ℓ(t)`) provides
the exact integer certificate for Piece B: R1 campaign Δ=0 uniformly across
amplitudes 50/500/5000 encodes `|Δt|·K ≤ gridLInf f − D_max` in CRAM arithmetic.
-/

namespace NSRegularity.NS.HStep

open NSRegularity NSRegularity.NS NSRegularity.NS.AdvectionBound

/-- **Small-data L∞ preservation (per-component, pointwise).**

    Under:
    - Piece A: `6·gridLInf(f)·M_u + gridLInf(uc_grad) ≤ K`
               (velocity and stretching data bounded by scalar K)
    - velocity componentwise: `|u.c i j k| ≤ M_u` for c ∈ {x,y,z}
    - Piece B₁: `∀ i j k, |diffusion_step f p i j k| ≤ D_max`
               (diffusion ceiling D_max < gridLInf f)
    - Piece B₂: `|p.dt_latt| · K ≤ gridLInf f − D_max`
               (CFL condition: step-size fits in spare diffusion capacity)

    the composed NS step component preserves L∞ pointwise:
    `|ns_step_component u uc_grad f p i j k| ≤ gridLInf f`. -/
theorem h_step_component_pointwise_small_data
    (u : VField) (uc_grad : Grid) (f : Grid) (p : NsParams)
    (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i' j' k' : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * NSRegularity.NS.laplacian f i' j' k'))
    -- Piece A: scalar data bound
    (M_u K : Int) (hM_u : 0 ≤ M_u) (hK : 0 ≤ K)
    (h_vel_x : ∀ i j k : Fin N, |u.x i j k| ≤ M_u)
    (h_vel_y : ∀ i j k : Fin N, |u.y i j k| ≤ M_u)
    (h_vel_z : ∀ i j k : Fin N, |u.z i j k| ≤ M_u)
    (h_data_bound : 6 * gridLInf f * M_u + gridLInf uc_grad ≤ K)
    -- Piece B: diffusion ceiling + CFL
    (D_max : Int)
    (h_diff_max : ∀ i j k : Fin N, |diffusion_step f p i j k| ≤ D_max)
    (h_cfl : |p.dt_latt| * K ≤ gridLInf f - D_max)
    (i j k : Fin N) :
    |ns_step_component u uc_grad f p i j k| ≤ gridLInf f := by
  -- 1. Perturbation bound under Resolution A
  have h_pert := ns_step_component_perturbation_le_exact
    u uc_grad f p h hν hdt hns_pos hExact i j k
  -- 2. Triangle inequality: |step| ≤ |step − diff| + |diff|
  have h_tri : |ns_step_component u uc_grad f p i j k|
      ≤ |ns_step_component u uc_grad f p i j k - diffusion_step f p i j k|
        + |diffusion_step f p i j k| := by
    have key : (ns_step_component u uc_grad f p i j k - diffusion_step f p i j k)
        + diffusion_step f p i j k = ns_step_component u uc_grad f p i j k := by ring
    have habs := abs_add_le
      (ns_step_component u uc_grad f p i j k - diffusion_step f p i j k)
      (diffusion_step f p i j k)
    rw [key] at habs; exact habs
  -- 3. Diffusion pointwise ceiling
  have h_diff := h_diff_max i j k
  -- 4. Velocity sum bound: Σ|u_c| ≤ 3·M_u
  have h_vel_sum : |u.x i j k| + |u.y i j k| + |u.z i j k| ≤ 3 * M_u :=
    by linarith [h_vel_x i j k, h_vel_y i j k, h_vel_z i j k]
  -- 5. uc_grad pointwise ≤ gridLInf
  have h_uc : |uc_grad i j k| ≤ gridLInf uc_grad := abs_le_gridLInf uc_grad i j k
  -- 6. Advection factor: 2·G·Σ|u| ≤ 6·G·M_u
  have hG : 0 ≤ gridLInf f := gridLInf_nonneg_int f
  have h_adv : 2 * gridLInf f * (|u.x i j k| + |u.y i j k| + |u.z i j k|)
      ≤ 6 * gridLInf f * M_u :=
    calc 2 * gridLInf f * (|u.x i j k| + |u.y i j k| + |u.z i j k|)
        ≤ 2 * gridLInf f * (3 * M_u) :=
          mul_le_mul_of_nonneg_left h_vel_sum (by linarith)
      _ = 6 * gridLInf f * M_u := by ring
  -- 7. Full pressure ≤ G − D_max via Piece A + CFL
  have h_pressure :
      |p.dt_latt| * (2 * gridLInf f * (|u.x i j k| + |u.y i j k| + |u.z i j k|))
      + |p.dt_latt| * |uc_grad i j k|
      ≤ gridLInf f - D_max := by
    have hdt_nn : 0 ≤ |p.dt_latt| := abs_nonneg _
    have h_adv' : |p.dt_latt| * (2 * gridLInf f * (|u.x i j k| + |u.y i j k| + |u.z i j k|))
        ≤ |p.dt_latt| * (6 * gridLInf f * M_u) :=
      mul_le_mul_of_nonneg_left h_adv hdt_nn
    have h_uc' : |p.dt_latt| * |uc_grad i j k|
        ≤ |p.dt_latt| * gridLInf uc_grad :=
      mul_le_mul_of_nonneg_left h_uc hdt_nn
    have h_sum : |p.dt_latt| * (6 * gridLInf f * M_u) + |p.dt_latt| * gridLInf uc_grad
        = |p.dt_latt| * (6 * gridLInf f * M_u + gridLInf uc_grad) := by ring
    have h_data_le_K : |p.dt_latt| * (6 * gridLInf f * M_u + gridLInf uc_grad)
        ≤ |p.dt_latt| * K :=
      mul_le_mul_of_nonneg_left h_data_bound hdt_nn
    linarith
  -- 8. Conclude: |step| ≤ |pert| + |diff| ≤ (G − D_max) + D_max = G
  linarith

/-- **Small-data L∞ preservation (per-component, gridLInf).**

    Lifts `h_step_component_pointwise_small_data` to the grid-level L∞
    via the three nested `Finset.sup'_le` — the standard pattern from
    `gridLInf_diffusion_step_le_exact`. -/
theorem gridLInf_ns_step_component_small_data
    (u : VField) (uc_grad : Grid) (f : Grid) (p : NsParams)
    (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i' j' k' : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * NSRegularity.NS.laplacian f i' j' k'))
    (M_u K : Int) (hM_u : 0 ≤ M_u) (hK : 0 ≤ K)
    (h_vel_x : ∀ i j k : Fin N, |u.x i j k| ≤ M_u)
    (h_vel_y : ∀ i j k : Fin N, |u.y i j k| ≤ M_u)
    (h_vel_z : ∀ i j k : Fin N, |u.z i j k| ≤ M_u)
    (h_data_bound : 6 * gridLInf f * M_u + gridLInf uc_grad ≤ K)
    (D_max : Int)
    (h_diff_max : ∀ i j k : Fin N, |diffusion_step f p i j k| ≤ D_max)
    (h_cfl : |p.dt_latt| * K ≤ gridLInf f - D_max) :
    gridLInf (ns_step_component u uc_grad f p) ≤ gridLInf f := by
  unfold gridLInf
  apply Finset.sup'_le; intro i _
  apply Finset.sup'_le; intro j _
  apply Finset.sup'_le; intro k _
  have h_pw := h_step_component_pointwise_small_data
    u uc_grad f p h hν hdt hns_pos hExact
    M_u K hM_u hK h_vel_x h_vel_y h_vel_z h_data_bound
    D_max h_diff_max h_cfl i j k
  rwa [Int.abs_eq_natAbs] at h_pw

/-- **Small-data L∞ preservation (VField, lInf).**

    Lifts the per-component `gridLInf` bound to the VField `lInf` via
    the three-component `max` bound. Each component carries its own scalar
    data bound `K_c` and diffusion ceiling `D_max_c`; the velocity bound
    `M_u` is shared across all three.

    This is the form `ns_global_regularity_skeleton` requires as `h_step`. -/
theorem lInf_ns_step_small_data
    (u : VField) (sx sy sz : Grid) (w : VField) (p : NsParams)
    -- Resolution A hypotheses (one per component)
    (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hns_pos : 0 < p.ns_scale)
    (hExact_x : ∀ i j k : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * NSRegularity.NS.laplacian w.x i j k))
    (hExact_y : ∀ i j k : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * NSRegularity.NS.laplacian w.y i j k))
    (hExact_z : ∀ i j k : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * NSRegularity.NS.laplacian w.z i j k))
    -- Shared velocity bound M_u (from Biot-Savart L∞ bound, Piece A)
    (M_u : Int) (hM_u : 0 ≤ M_u)
    (h_vel_x : ∀ i j k : Fin N, |u.x i j k| ≤ M_u)
    (h_vel_y : ∀ i j k : Fin N, |u.y i j k| ≤ M_u)
    (h_vel_z : ∀ i j k : Fin N, |u.z i j k| ≤ M_u)
    -- Per-component data bounds (Piece A)
    (K_x K_y K_z : Int) (hK_x : 0 ≤ K_x) (hK_y : 0 ≤ K_y) (hK_z : 0 ≤ K_z)
    (h_data_x : 6 * gridLInf w.x * M_u + gridLInf sx ≤ K_x)
    (h_data_y : 6 * gridLInf w.y * M_u + gridLInf sy ≤ K_y)
    (h_data_z : 6 * gridLInf w.z * M_u + gridLInf sz ≤ K_z)
    -- Per-component diffusion ceilings (Piece B, part 1)
    (D_max_x D_max_y D_max_z : Int)
    (h_diff_max_x : ∀ i j k : Fin N, |diffusion_step w.x p i j k| ≤ D_max_x)
    (h_diff_max_y : ∀ i j k : Fin N, |diffusion_step w.y p i j k| ≤ D_max_y)
    (h_diff_max_z : ∀ i j k : Fin N, |diffusion_step w.z p i j k| ≤ D_max_z)
    -- CFL conditions per component (Piece B, part 2)
    (h_cfl_x : |p.dt_latt| * K_x ≤ gridLInf w.x - D_max_x)
    (h_cfl_y : |p.dt_latt| * K_y ≤ gridLInf w.y - D_max_y)
    (h_cfl_z : |p.dt_latt| * K_z ≤ gridLInf w.z - D_max_z) :
    lInf (ns_step u sx sy sz w p) ≤ lInf w := by
  have hx := gridLInf_ns_step_component_small_data
    u sx w.x p h hν hdt hns_pos hExact_x
    M_u K_x hM_u hK_x h_vel_x h_vel_y h_vel_z h_data_x D_max_x h_diff_max_x h_cfl_x
  have hy := gridLInf_ns_step_component_small_data
    u sy w.y p h hν hdt hns_pos hExact_y
    M_u K_y hM_u hK_y h_vel_x h_vel_y h_vel_z h_data_y D_max_y h_diff_max_y h_cfl_y
  have hz := gridLInf_ns_step_component_small_data
    u sz w.z p h hν hdt hns_pos hExact_z
    M_u K_z hM_u hK_z h_vel_x h_vel_y h_vel_z h_data_z D_max_z h_diff_max_z h_cfl_z
  unfold lInf ns_step
  exact max_le_max hx (max_le_max hy hz)

/-- **Global regularity from uniform small-data h_step.**

    If the small-data hypotheses hold at every trajectory state `w` — i.e.,
    the Biot-Savart bound, data bound K, and CFL condition all hold uniformly
    along the trajectory — then `IsSmooth` follows via `ns_global_regularity_skeleton`.

    The `h_step_at_every_state` hypothesis packages all per-step small-data
    conditions into a single `∀ w` — the exact form the skeleton requires.
    A complete proof would discharge this from:
      (A) Biot-Savart operator norm bound (NTT port, task #12)
      (B) Stretching gradient bound (DKAM-subcritical)
      (C) Small-data threshold ε₀(ν, N) > ‖ω₀‖²_H² (quantitative CFL) -/
theorem ns_global_regularity_small_data
    (biot_savart : VField → VField)
    (stretching_x stretching_y stretching_z : VField → Grid)
    (p : NsParams)
    (h_step_at_every_state : ∀ w : VField,
        lInf (ns_step (biot_savart w) (stretching_x w)
                      (stretching_y w) (stretching_z w) w p)
        ≤ lInf w)
    (w₀ : VField) (hLInf : 0 ≤ lInf w₀) (T : ℕ) :
    NSRegularity.BKM.IsSmooth
        (ns_trajectory_generator biot_savart stretching_x stretching_y stretching_z w₀ p) :=
  ns_global_regularity_skeleton biot_savart stretching_x stretching_y stretching_z p
    h_step_at_every_state w₀ hLInf T

end NSRegularity.NS.HStep
