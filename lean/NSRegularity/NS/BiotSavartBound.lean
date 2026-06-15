import NSRegularity.NS.BiotSavart
import NSRegularity.NS.StretchingBound

/-!
# NS/BiotSavartBound — Attack 1: the Biot-Savart L∞ interface

Three results, in decreasing order of unconditionality:

1. **Reverse (consistency) bound, proved:** `lInf ω ≤ 4 · lInf (biot_savart ω)`
   for divergence-free ω. The curl of a bounded field is bounded, so the
   characteristic axiom `curl (biot_savart ω) = ω` forces the velocity to
   be at least a quarter of the vorticity in L∞. Any future constant
   `C_BS` in the forward bound must satisfy `C_BS ≥ 1/4`.

2. **Underdetermination witness, proved:** the checkerboard field
   `cb(i,j,k) = (−1)^(i+j+k)` has identically zero *centered* differences
   (the centered stencil couples only same-parity points), so adding any
   multiple of it to a velocity component changes neither `curl` nor
   `divV`. Hence the two Biot-Savart axioms (`curl_eq`, `div_zero`)
   provably **cannot** determine `biot_savart` up to L∞ — a forward bound
   is a property of the *chosen representative* (the NTT-canonical one in
   `ns3d.rs`), not of the axioms. This settles, with a Lean witness, why
   Attack 1 must enter as its own interface.

3. **The interface axiom + Piece A discharge:** `biot_savart_linf_bound`
   asserts the L∞ operator bound for the canonical NTT representative
   (constructively true for the Rust pipeline: the velocity is a fixed
   integer-kernel convolution of the vorticity, so
   `lInf u ≤ ‖K‖₁ · lInf ω` with `‖K‖₁` a concrete integer at fixed N).
   `piece_A_discharged` then produces *all* Piece A inputs of
   `h_step_component_pointwise_small_data` — velocity bounds and data
   bound with explicit `K = 6·G·M_u + 6·M_u·lInf ω`, `M_u = C_BS·lInf ω`.

After this file the small-data h_step gate depends only on:
- 🔵 `biot_savart_linf_bound` (this interface; discharged by the NTT port)
- 🟠 Piece B (diffusion ceiling D_max + CFL) — quantified in
  `NS/DiffusionCeiling.lean`, open at max points.
-/

namespace NSRegularity.NS.BiotSavartBound

open NSRegularity NSRegularity.NS
open NSRegularity.NS.AdvectionBound NSRegularity.NS.StretchingBound

/-! ## 1. Reverse bound: curl of bounded is bounded -/

/-- Triangle inequality for differences: `|a − b| ≤ |a| + |b|`. -/
theorem abs_sub_le_abs_add_abs (a b : Int) : |a - b| ≤ |a| + |b| := by
  calc |a - b| = |a + -b| := by rw [sub_eq_add_neg]
    _ ≤ |a| + |-b| := abs_add_le a (-b)
    _ = |a| + |b| := by rw [abs_neg]

/-- Pointwise curl bound, x-component: `|curlX w| ≤ 4·lInf w`. -/
theorem abs_curlX_le (w : VField) (i j k : Fin N) :
    |curlX w i j k| ≤ 4 * lInf w := by
  unfold curlX
  have htri := abs_sub_le_abs_add_abs
    (NSRegularity.Kill113.diffY w.z i j k) (NSRegularity.Kill113.diffZ w.y i j k)
  have hY := abs_diffY_le w.z i j k
  have hZ := abs_diffZ_le w.y i j k
  have hz_le := gridLInf_z_le_lInf w
  have hy_le := gridLInf_y_le_lInf w
  linarith

/-- Pointwise curl bound, y-component. -/
theorem abs_curlY_le (w : VField) (i j k : Fin N) :
    |curlY w i j k| ≤ 4 * lInf w := by
  unfold curlY
  have htri := abs_sub_le_abs_add_abs
    (NSRegularity.Kill113.diffZ w.x i j k) (NSRegularity.Kill113.diffX w.z i j k)
  have hX := abs_diffZ_le w.x i j k
  have hZ := abs_diffX_le w.z i j k
  have hx_le := gridLInf_x_le_lInf w
  have hz_le := gridLInf_z_le_lInf w
  linarith

/-- Pointwise curl bound, z-component. -/
theorem abs_curlZ_le (w : VField) (i j k : Fin N) :
    |curlZ w i j k| ≤ 4 * lInf w := by
  unfold curlZ
  have htri := abs_sub_le_abs_add_abs
    (NSRegularity.Kill113.diffX w.y i j k) (NSRegularity.Kill113.diffY w.x i j k)
  have hY := abs_diffX_le w.y i j k
  have hX := abs_diffY_le w.x i j k
  have hy_le := gridLInf_y_le_lInf w
  have hx_le := gridLInf_x_le_lInf w
  linarith

/-- **Curl operator L∞ bound (proved):** `lInf (curl w) ≤ 4 · lInf w`. -/
theorem lInf_curl_le (w : VField) : lInf (curl w) ≤ 4 * lInf w := by
  have hx : gridLInf (curl w).x ≤ 4 * lInf w :=
    gridLInf_le_of_pointwise_abs _ _ (fun i j k => abs_curlX_le w i j k)
  have hy : gridLInf (curl w).y ≤ 4 * lInf w :=
    gridLInf_le_of_pointwise_abs _ _ (fun i j k => abs_curlY_le w i j k)
  have hz : gridLInf (curl w).z ≤ 4 * lInf w :=
    gridLInf_le_of_pointwise_abs _ _ (fun i j k => abs_curlZ_le w i j k)
  unfold lInf
  exact max_le hx (max_le hy hz)

/-- **Consistency lower bound on Biot-Savart (proved from the axiom):**
    for divergence-free vorticity, `lInf ω ≤ 4 · lInf (biot_savart ω)`.
    Any forward L∞ constant must satisfy `C_BS ≥ 1/4`. -/
theorem lInf_le_four_lInf_biot_savart (ω : VField)
    (hdiv : divV ω = (fun (_ _ _ : Fin N) => (0 : Int))) :
    lInf ω ≤ 4 * lInf (biot_savart ω) := by
  have hcurl := biot_savart_curl_eq ω hdiv
  calc lInf ω = lInf (curl (biot_savart ω)) := by rw [hcurl]
    _ ≤ 4 * lInf (biot_savart ω) := lInf_curl_le (biot_savart ω)

/-! ## 2. Underdetermination: curl + div do not pin the velocity -/

/-- The checkerboard field: `+1` on even-parity points, `−1` on odd. -/
def checkerboard : Grid :=
  fun i j k => if (i.val + j.val + k.val) % 2 = 0 then (1 : Int) else -1

/-- All centered differences of the checkerboard vanish: the centered
    stencil only couples points of equal parity (N = 8 is even). -/
theorem diffX_checkerboard_eq_zero :
    ∀ i j k : Fin N, NSRegularity.Kill113.diffX checkerboard i j k = 0 := by
  decide

theorem diffY_checkerboard_eq_zero :
    ∀ i j k : Fin N, NSRegularity.Kill113.diffY checkerboard i j k = 0 := by
  decide

theorem diffZ_checkerboard_eq_zero :
    ∀ i j k : Fin N, NSRegularity.Kill113.diffZ checkerboard i j k = 0 := by
  decide

/-- Same-parity agreement under the x-shift pair. -/
theorem checkerboard_sp_sn_x (i j k : Fin N) :
    checkerboard (sp i) j k = checkerboard (sn i) j k := by
  have h0 := diffX_checkerboard_eq_zero i j k
  unfold NSRegularity.Kill113.diffX at h0
  linarith

/-- Same-parity agreement under the y-shift pair. -/
theorem checkerboard_sp_sn_y (i j k : Fin N) :
    checkerboard i (sp j) k = checkerboard i (sn j) k := by
  have h0 := diffY_checkerboard_eq_zero i j k
  unfold NSRegularity.Kill113.diffY at h0
  linarith

/-- Same-parity agreement under the z-shift pair. -/
theorem checkerboard_sp_sn_z (i j k : Fin N) :
    checkerboard i j (sp k) = checkerboard i j (sn k) := by
  have h0 := diffZ_checkerboard_eq_zero i j k
  unfold NSRegularity.Kill113.diffZ at h0
  linarith

/-- Adding any multiple of the checkerboard to the x-component changes
    neither the curl nor the divergence of a velocity field. -/
theorem curl_div_blind_to_checkerboard (w : VField) (c : Int) :
    curl ⟨fun i j k => w.x i j k + c * checkerboard i j k, w.y, w.z⟩ = curl w ∧
    divV ⟨fun i j k => w.x i j k + c * checkerboard i j k, w.y, w.z⟩ = divV w := by
  -- component equalities of the curl
  have hX : curlX ⟨fun i j k => w.x i j k + c * checkerboard i j k, w.y, w.z⟩
      = curlX w := rfl
  have hY : curlY ⟨fun i j k => w.x i j k + c * checkerboard i j k, w.y, w.z⟩
      = curlY w := by
    funext i j k
    show (w.x i j (sp k) + c * checkerboard i j (sp k))
        - (w.x i j (sn k) + c * checkerboard i j (sn k))
        - NSRegularity.Kill113.diffX w.z i j k
      = NSRegularity.Kill113.diffZ w.x i j k - NSRegularity.Kill113.diffX w.z i j k
    rw [checkerboard_sp_sn_z i j k]
    unfold NSRegularity.Kill113.diffZ
    ring
  have hZ : curlZ ⟨fun i j k => w.x i j k + c * checkerboard i j k, w.y, w.z⟩
      = curlZ w := by
    funext i j k
    show NSRegularity.Kill113.diffX w.y i j k
        - ((w.x i (sp j) k + c * checkerboard i (sp j) k)
           - (w.x i (sn j) k + c * checkerboard i (sn j) k))
      = NSRegularity.Kill113.diffX w.y i j k - NSRegularity.Kill113.diffY w.x i j k
    rw [checkerboard_sp_sn_y i j k]
    unfold NSRegularity.Kill113.diffY
    ring
  constructor
  · show (⟨curlX ⟨fun i j k => w.x i j k + c * checkerboard i j k, w.y, w.z⟩,
          curlY ⟨fun i j k => w.x i j k + c * checkerboard i j k, w.y, w.z⟩,
          curlZ ⟨fun i j k => w.x i j k + c * checkerboard i j k, w.y, w.z⟩⟩ : VField)
        = ⟨curlX w, curlY w, curlZ w⟩
    rw [hX, hY, hZ]
  · funext i j k
    show (w.x (sp i) j k + c * checkerboard (sp i) j k)
        - (w.x (sn i) j k + c * checkerboard (sn i) j k)
        + NSRegularity.Kill113.diffY w.y i j k
        + NSRegularity.Kill113.diffZ w.z i j k
      = NSRegularity.Kill113.diffX w.x i j k
        + NSRegularity.Kill113.diffY w.y i j k
        + NSRegularity.Kill113.diffZ w.z i j k
    rw [checkerboard_sp_sn_x i j k]
    unfold NSRegularity.Kill113.diffX
    ring

/-- **Underdetermination of Biot-Savart by its axioms (proved).**
    There exist *distinct* velocity fields with identical curl and
    divergence. Hence no L∞ bound on `biot_savart` is derivable from
    `biot_savart_curl_eq` + `biot_savart_div_zero` alone: the forward
    bound is a property of the canonical (NTT) representative and must
    enter as its own interface. -/
theorem velocity_not_determined_by_curl_div :
    ∃ w w' : VField, w ≠ w' ∧ curl w = curl w' ∧ divV w = divV w' := by
  obtain ⟨hcurl, hdiv⟩ := curl_div_blind_to_checkerboard
    ⟨fun _ _ _ => 0, fun _ _ _ => 0, fun _ _ _ => 0⟩ 1
  refine ⟨_, _, ?_, hcurl.symm, hdiv.symm⟩
  intro h
  have h000 : (0 : Int) = 0 + 1 * checkerboard 0 0 0 := by
    have hx := congrArg VField.x h
    exact congrFun (congrFun (congrFun hx (0 : Fin N)) (0 : Fin N)) (0 : Fin N)
  have hcb : checkerboard (0 : Fin N) (0 : Fin N) (0 : Fin N) = 1 := rfl
  rw [hcb] at h000
  norm_num at h000

/-! ## 3. The interface axiom and the Piece A discharge -/

/-- **Attack 1 interface (axiom).** L∞ operator bound for the canonical
    NTT Biot-Savart representative.

    Soundness: the Rust pipeline (`ns3d.rs`, 4-prime NTT + Garner) computes
    `u = K ∗ ω` with a *fixed* integer kernel at resolution N, so
    `lInf u ≤ ‖K‖₁ · lInf ω` holds constructively with `‖K‖₁` a concrete
    integer. The underdetermination witness above shows this bound cannot
    be derived from the two characteristic axioms; it is a property of the
    canonical representative. Consistency floor: `C_BS ≥ 1/4` is forced by
    `lInf_le_four_lInf_biot_savart`. Replacing this axiom by the
    constructive NTT port is the remaining content of Attack 1. -/
axiom biot_savart_linf_bound :
    ∃ C_BS : Int, 0 < C_BS ∧
      ∀ ω : VField, lInf (biot_savart ω) ≤ C_BS * lInf ω

/-- **Piece A, fully discharged modulo the interface axiom.**
    For every vorticity `ω` and transported component `f`, there are
    `M_u, K ≥ 0` satisfying *all* Piece A inputs of
    `h_step_component_pointwise_small_data`:
    the three velocity component bounds and the data bound, with the
    stretching grid `uc_grad := advective ω (biot_savart ω).x`. -/
theorem piece_A_discharged (ω : VField) (f : Grid) :
    ∃ M_u K : Int, 0 ≤ M_u ∧ 0 ≤ K ∧
      (∀ i j k : Fin N, |(biot_savart ω).x i j k| ≤ M_u) ∧
      (∀ i j k : Fin N, |(biot_savart ω).y i j k| ≤ M_u) ∧
      (∀ i j k : Fin N, |(biot_savart ω).z i j k| ≤ M_u) ∧
      6 * gridLInf f * M_u
        + gridLInf (fun i j k =>
            NSRegularity.Kill113.advective ω ((biot_savart ω).x) i j k) ≤ K := by
  obtain ⟨C_BS, hC_pos, hbound⟩ := biot_savart_linf_bound
  set u : VField := biot_savart ω
  set M_u : Int := C_BS * lInf ω with hM_def
  have hω_nn : 0 ≤ lInf ω := lInf_nonneg_int ω
  have hM_nn : 0 ≤ M_u := mul_nonneg (le_of_lt hC_pos) hω_nn
  have hu_le : lInf u ≤ M_u := hbound ω
  -- component bounds from the vector-field bound
  have hux : ∀ i j k : Fin N, |u.x i j k| ≤ M_u := fun i j k =>
    le_trans (abs_x_le_lInf u i j k) hu_le
  have huy : ∀ i j k : Fin N, |u.y i j k| ≤ M_u := fun i j k =>
    le_trans (abs_y_le_lInf u i j k) hu_le
  have huz : ∀ i j k : Fin N, |u.z i j k| ≤ M_u := fun i j k =>
    le_trans (abs_z_le_lInf u i j k) hu_le
  -- data bound with K := 6·G·M_u + 6·M_u·lInf ω (Attack 2)
  have h_ucx : gridLInf u.x ≤ M_u :=
    le_trans (gridLInf_x_le_lInf u) hu_le
  have hdata := piece_A_data_bound ω u.x f M_u h_ucx
  have hG_nn : 0 ≤ gridLInf f := gridLInf_nonneg_int f
  refine ⟨M_u, 6 * gridLInf f * M_u + 6 * M_u * lInf ω, hM_nn, ?_, hux, huy, huz, hdata⟩
  have h1 : 0 ≤ 6 * gridLInf f * M_u :=
    mul_nonneg (mul_nonneg (by norm_num) hG_nn) hM_nn
  have h2 : 0 ≤ 6 * M_u * lInf ω :=
    mul_nonneg (mul_nonneg (by norm_num) hM_nn) hω_nn
  linarith

end NSRegularity.NS.BiotSavartBound
