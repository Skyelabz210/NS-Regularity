import NSRegularity.NS.HStepSmallData
import NSRegularity.NS.BiotSavartBound

/-!
# NS/SmallDataCapstone — the assembled small-data global regularity theorem

Composes the whole small-data arc into one theorem with *explicit*
hypotheses and no buried existentials:

```
ns_global_regularity_canonical :
    {Biot-Savart L∞ constant C_BS}            (interface; NTT port discharges)
  + {Resolution-A exactness at every state}    (CRAM arithmetic provides)
  + {Piece B: diffusion ceiling + CFL at every state}   (small-data condition)
  → IsSmooth (NS trajectory with canonical stretching)
```

The stretching functions are here *fixed to the canonical NS form*
`(w·∇)(biot_savart w)_c` — not abstract parameters. Piece A is fully
derived inside the proof (Attack 2 + the interface bound); the data
constant is the explicit

```
K_c(w) = 6·gridLInf w.c · M_u(w) + 6·M_u(w)·lInf w,   M_u(w) = C_BS·lInf w
```

The theorem is **axiom-honest**: it takes the Biot-Savart bound as an
explicit hypothesis `hBS` rather than invoking `biot_savart_linf_bound`,
so its `#print axioms` footprint is exactly that of the trajectory
skeleton (the three BS characteristic axioms + BKM).
-/

namespace NSRegularity.NS.Capstone

open NSRegularity NSRegularity.NS
open NSRegularity.NS.AdvectionBound NSRegularity.NS.StretchingBound
open NSRegularity.NS.HStep

/-- Canonical NS stretching, x-component: `(w·∇)(biot_savart w).x`. -/
noncomputable def ns_stretching_x : VField → Grid := fun w => fun i j k =>
  NSRegularity.Kill113.advective w ((biot_savart w).x) i j k

/-- Canonical NS stretching, y-component. -/
noncomputable def ns_stretching_y : VField → Grid := fun w => fun i j k =>
  NSRegularity.Kill113.advective w ((biot_savart w).y) i j k

/-- Canonical NS stretching, z-component. -/
noncomputable def ns_stretching_z : VField → Grid := fun w => fun i j k =>
  NSRegularity.Kill113.advective w ((biot_savart w).z) i j k

/-- **Small-data global regularity, canonical assembly.**

    Under
    - `hBS`: the Biot-Savart L∞ interface bound with constant `C_BS ≥ 0`,
    - `hExact_*`: Resolution-A exactness at every state (CRAM-exact
      diffusion division),
    - `hPieceB`: at every state, per-component diffusion ceilings `D_c`
      and the CFL margin
      `|Δt| · (6·gridLInf w.c·M_u + 6·M_u·lInf w) ≤ gridLInf w.c − D_c`
      with `M_u = C_BS·lInf w`,

    the NS trajectory with canonical stretching is smooth. Piece A is
    derived, not assumed. -/
theorem ns_global_regularity_canonical
    (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) (hns_pos : 0 < p.ns_scale)
    (C_BS : Int) (hC : 0 ≤ C_BS)
    (hBS : ∀ ω : VField, lInf (biot_savart ω) ≤ C_BS * lInf ω)
    (hExact_x : ∀ w : VField, ∀ i j k : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * laplacian w.x i j k))
    (hExact_y : ∀ w : VField, ∀ i j k : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * laplacian w.y i j k))
    (hExact_z : ∀ w : VField, ∀ i j k : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * laplacian w.z i j k))
    (hPieceB : ∀ w : VField, ∃ D_x D_y D_z : Int,
      (∀ i j k : Fin N, |diffusion_step w.x p i j k| ≤ D_x) ∧
      (∀ i j k : Fin N, |diffusion_step w.y p i j k| ≤ D_y) ∧
      (∀ i j k : Fin N, |diffusion_step w.z p i j k| ≤ D_z) ∧
      |p.dt_latt| * (6 * gridLInf w.x * (C_BS * lInf w)
        + 6 * (C_BS * lInf w) * lInf w) ≤ gridLInf w.x - D_x ∧
      |p.dt_latt| * (6 * gridLInf w.y * (C_BS * lInf w)
        + 6 * (C_BS * lInf w) * lInf w) ≤ gridLInf w.y - D_y ∧
      |p.dt_latt| * (6 * gridLInf w.z * (C_BS * lInf w)
        + 6 * (C_BS * lInf w) * lInf w) ≤ gridLInf w.z - D_z)
    (w₀ : VField) (hLInf : 0 ≤ lInf w₀) (T : ℕ) :
    NSRegularity.BKM.IsSmooth
      (ns_trajectory_generator biot_savart
        ns_stretching_x ns_stretching_y ns_stretching_z w₀ p) := by
  apply ns_global_regularity_skeleton biot_savart
    ns_stretching_x ns_stretching_y ns_stretching_z p
    ?h_step w₀ hLInf T
  intro w
  -- Per-state data: M_u and the three K_c, all explicit
  obtain ⟨D_x, D_y, D_z, hDx, hDy, hDz, hcfl_x, hcfl_y, hcfl_z⟩ := hPieceB w
  have hw_nn : 0 ≤ lInf w := lInf_nonneg_int w
  have hM_nn : 0 ≤ C_BS * lInf w := mul_nonneg hC hw_nn
  have hu_le : lInf (biot_savart w) ≤ C_BS * lInf w := hBS w
  -- velocity component bounds
  have hux : ∀ i j k : Fin N, |(biot_savart w).x i j k| ≤ C_BS * lInf w :=
    fun i j k => le_trans (abs_x_le_lInf (biot_savart w) i j k) hu_le
  have huy : ∀ i j k : Fin N, |(biot_savart w).y i j k| ≤ C_BS * lInf w :=
    fun i j k => le_trans (abs_y_le_lInf (biot_savart w) i j k) hu_le
  have huz : ∀ i j k : Fin N, |(biot_savart w).z i j k| ≤ C_BS * lInf w :=
    fun i j k => le_trans (abs_z_le_lInf (biot_savart w) i j k) hu_le
  -- Piece A data bounds (Attack 2), per component
  have hdata_x : 6 * gridLInf w.x * (C_BS * lInf w) + gridLInf (ns_stretching_x w)
      ≤ 6 * gridLInf w.x * (C_BS * lInf w) + 6 * (C_BS * lInf w) * lInf w :=
    piece_A_data_bound w ((biot_savart w).x) w.x (C_BS * lInf w)
      (le_trans (gridLInf_x_le_lInf (biot_savart w)) hu_le)
  have hdata_y : 6 * gridLInf w.y * (C_BS * lInf w) + gridLInf (ns_stretching_y w)
      ≤ 6 * gridLInf w.y * (C_BS * lInf w) + 6 * (C_BS * lInf w) * lInf w :=
    piece_A_data_bound w ((biot_savart w).y) w.y (C_BS * lInf w)
      (le_trans (gridLInf_y_le_lInf (biot_savart w)) hu_le)
  have hdata_z : 6 * gridLInf w.z * (C_BS * lInf w) + gridLInf (ns_stretching_z w)
      ≤ 6 * gridLInf w.z * (C_BS * lInf w) + 6 * (C_BS * lInf w) * lInf w :=
    piece_A_data_bound w ((biot_savart w).z) w.z (C_BS * lInf w)
      (le_trans (gridLInf_z_le_lInf (biot_savart w)) hu_le)
  -- K_c non-negativity
  have hKx_nn : 0 ≤ 6 * gridLInf w.x * (C_BS * lInf w) + 6 * (C_BS * lInf w) * lInf w := by
    have h1 : 0 ≤ 6 * gridLInf w.x * (C_BS * lInf w) :=
      mul_nonneg (mul_nonneg (by norm_num) (gridLInf_nonneg_int w.x)) hM_nn
    have h2 : 0 ≤ 6 * (C_BS * lInf w) * lInf w :=
      mul_nonneg (mul_nonneg (by norm_num) hM_nn) hw_nn
    linarith
  have hKy_nn : 0 ≤ 6 * gridLInf w.y * (C_BS * lInf w) + 6 * (C_BS * lInf w) * lInf w := by
    have h1 : 0 ≤ 6 * gridLInf w.y * (C_BS * lInf w) :=
      mul_nonneg (mul_nonneg (by norm_num) (gridLInf_nonneg_int w.y)) hM_nn
    have h2 : 0 ≤ 6 * (C_BS * lInf w) * lInf w :=
      mul_nonneg (mul_nonneg (by norm_num) hM_nn) hw_nn
    linarith
  have hKz_nn : 0 ≤ 6 * gridLInf w.z * (C_BS * lInf w) + 6 * (C_BS * lInf w) * lInf w := by
    have h1 : 0 ≤ 6 * gridLInf w.z * (C_BS * lInf w) :=
      mul_nonneg (mul_nonneg (by norm_num) (gridLInf_nonneg_int w.z)) hM_nn
    have h2 : 0 ≤ 6 * (C_BS * lInf w) * lInf w :=
      mul_nonneg (mul_nonneg (by norm_num) hM_nn) hw_nn
    linarith
  -- assemble via the small-data lInf preservation
  exact lInf_ns_step_small_data (biot_savart w)
    (ns_stretching_x w) (ns_stretching_y w) (ns_stretching_z w) w p
    h hν hdt hns_pos (hExact_x w) (hExact_y w) (hExact_z w)
    (C_BS * lInf w) hM_nn hux huy huz
    (6 * gridLInf w.x * (C_BS * lInf w) + 6 * (C_BS * lInf w) * lInf w)
    (6 * gridLInf w.y * (C_BS * lInf w) + 6 * (C_BS * lInf w) * lInf w)
    (6 * gridLInf w.z * (C_BS * lInf w) + 6 * (C_BS * lInf w) * lInf w)
    hKx_nn hKy_nn hKz_nn
    hdata_x hdata_y hdata_z
    D_x D_y D_z hDx hDy hDz
    hcfl_x hcfl_y hcfl_z

end NSRegularity.NS.Capstone
