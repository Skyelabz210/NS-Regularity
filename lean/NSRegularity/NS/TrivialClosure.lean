import NSRegularity.NS.Trajectory
import NSRegularity.GlobalRegularityGeneric

/-!
# NS/TrivialClosure — fully discharged NS skeleton at the trivial instantiation

`ns_global_regularity_skeleton` (in `NS/Trajectory.lean`) is *parametric* in
the Biot-Savart inversion, the three stretching gradients, and an L∞-step
hypothesis. With those four pieces supplied, it returns the discrete
`IsSmooth` witness for the resulting NS trajectory.

This file picks the **trivial choice**:

  bs   := fun _ => ⟨0, 0, 0⟩      -- velocity ≡ 0
  sx   := fun _ _ _ _ => 0        -- zero stretching gradient (x)
  sy   := fun _ _ _ _ => 0        -- zero stretching gradient (y)
  sz   := fun _ _ _ _ => 0        -- zero stretching gradient (z)

Under these choices the NS step reduces to **pure diffusion**, because:

* `advective 0 f` is identically zero (each velocity component multiplies a
  difference), so `advection_step 0 g p = g`.
* `stretching_step (fun _ _ _ => 0) g p = g`, since integer-quotient by
  `(2 N S)` sends `dt · 0` to zero.

The resulting trajectory is therefore `diffusion_trajectory w₀ p`, and the
fully proved Resolution-A closure `global_regularity_exact` discharges
`IsSmooth` — no axiom invocation required.

This is the first concrete *fully discharged* `IsSmooth (ns_trajectory_generator ...)`
instance: every other top-level NS-style theorem in the library either
takes the L∞-step hypothesis as a parameter or specialises to the axiomatic
Biot-Savart whose constructive port is still pending.
-/

namespace NSRegularity.NS.TrivialClosure

open NSRegularity NSRegularity.NS NSRegularity.GlobalRegularity

/-- Trivial Biot-Savart: every input maps to the zero velocity field. -/
def trivial_biot_savart : VField → VField :=
  fun _ => ⟨fun _ _ _ => 0, fun _ _ _ => 0, fun _ _ _ => 0⟩

/-- `divV (trivial_biot_savart ω) = 0` — the discharged form of the
    `biot_savart_div_zero` axiom for this constructive choice. -/
theorem trivial_biot_savart_div_zero (ω : VField) :
    divV (trivial_biot_savart ω) = fun (_ _ _ : Fin N) => (0 : Int) := by
  funext i j k
  unfold trivial_biot_savart divV
  unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
  ring

/-- For `ω = 0`, the trivial Biot-Savart satisfies the curl-inverse
    property `curl (trivial_biot_savart ω) = ω`. This is the discharged
    form of `biot_savart_curl_eq` on the zero subspace, with the
    `divV ω = 0` precondition trivially satisfied. -/
theorem trivial_biot_savart_curl_eq_on_zero :
    curl (trivial_biot_savart ⟨fun _ _ _ => 0, fun _ _ _ => 0, fun _ _ _ => 0⟩)
      = ⟨fun _ _ _ => 0, fun _ _ _ => 0, fun _ _ _ => 0⟩ := by
  unfold trivial_biot_savart curl curlX curlY curlZ
  unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
  refine VField.mk.injEq .. |>.mpr ⟨?_, ?_, ?_⟩ <;>
    (funext i j k; ring)

/-- Zero stretching gradient. Independent of `w`. -/
def zero_stretching : VField → Grid :=
  fun _ => fun _ _ _ => 0

/-- The advection step at zero velocity is the identity. -/
theorem advection_step_zero_eq (f : Grid) (p : NsParams) :
    advection_step ⟨fun _ _ _ => 0, fun _ _ _ => 0, fun _ _ _ => 0⟩ f p = f := by
  funext i j k
  unfold advection_step NSRegularity.Kill113.advective
  unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
  simp

/-- The stretching step with zero gradient is the identity. -/
theorem stretching_step_zero_eq (f : Grid) (p : NsParams) :
    stretching_step (fun _ _ _ => 0) f p = f := by
  funext i j k
  unfold stretching_step
  simp

/-- The NS component step under the trivial choices is pure diffusion. -/
theorem ns_step_component_trivial_eq (f : Grid) (p : NsParams) :
    ns_step_component ⟨fun _ _ _ => 0, fun _ _ _ => 0, fun _ _ _ => 0⟩
      (fun _ _ _ => 0) f p = diffusion_step f p := by
  unfold ns_step_component
  rw [advection_step_zero_eq, stretching_step_zero_eq]

/-- The NS full-field step under the trivial choices is the diffusion vstep. -/
theorem ns_step_trivial_eq (w : VField) (p : NsParams) :
    ns_step (trivial_biot_savart w) (zero_stretching w) (zero_stretching w)
      (zero_stretching w) w p = diffusion_vstep p w := by
  unfold ns_step trivial_biot_savart zero_stretching diffusion_vstep
  refine VField.mk.injEq .. |>.mpr ?_
  refine ⟨?_, ?_, ?_⟩ <;> exact ns_step_component_trivial_eq _ _

/-- The NS trajectory under the trivial choices equals the diffusion
    trajectory. -/
theorem ns_trajectory_trivial_eq (w₀ : VField) (p : NsParams) :
    ns_trajectory_generator trivial_biot_savart zero_stretching zero_stretching
      zero_stretching w₀ p = diffusion_trajectory w₀ p := by
  funext n
  induction n with
  | zero => rfl
  | succ n ih =>
      show ns_trajectory_generator _ _ _ _ _ _ (n + 1) =
            diffusion_trajectory w₀ p (n + 1)
      unfold ns_trajectory_generator diffusion_trajectory
      rw [ih]
      exact ns_step_trivial_eq _ _

/-- **Fully discharged NS skeleton, trivial instantiation, Resolution A.**

    The first concrete fully proved `IsSmooth (ns_trajectory_generator ...)`
    instance in the library. No axioms invoked (the three `biot_savart`
    axioms in `NS/BiotSavart.lean` are bypassed because we use
    `trivial_biot_savart` rather than the axiomatic one). -/
theorem ns_global_regularity_trivial_exact
    (w₀ : VField) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) (hns : 0 < p.ns_scale)
    (hLInf : 0 ≤ lInf w₀)
    (hExactAll : diffusion_exactness_hyp w₀ p) (T : ℕ) :
    NSRegularity.BKM.IsSmooth
      (ns_trajectory_generator trivial_biot_savart zero_stretching
        zero_stretching zero_stretching w₀ p) := by
  rw [ns_trajectory_trivial_eq]
  exact global_regularity_exact w₀ p h hν hdt hns hLInf hExactAll T

/-- The zero vector field. -/
def zero_vfield : VField :=
  ⟨fun _ _ _ => 0, fun _ _ _ => 0, fun _ _ _ => 0⟩

/-- The diffusion exactness hypothesis holds trivially for `w₀ = 0`. -/
theorem diffusion_exactness_hyp_zero (p : NsParams) :
    diffusion_exactness_hyp zero_vfield p := by
  intro n i j k
  -- The trajectory starting at zero is constant zero, so laplacian is zero,
  -- so the divisibility holds with quotient 0.
  -- Step 1: show `diffusion_trajectory zero_vfield p n = zero_vfield`.
  have htraj : ∀ n, diffusion_trajectory zero_vfield p n = zero_vfield := by
    intro n
    induction n with
    | zero => rfl
    | succ n ih =>
        unfold diffusion_trajectory
        rw [ih]
        unfold zero_vfield diffusion_step NSRegularity.NS.laplacian
        refine VField.mk.injEq .. |>.mpr ⟨?_, ?_, ?_⟩ <;>
          (funext i j k; simp)
  rw [htraj n]
  -- Each component's laplacian on the zero field is zero.
  refine ⟨?_, ?_, ?_⟩ <;> (unfold zero_vfield NSRegularity.NS.laplacian; simp)

/-- **Hypothesis-free closed NS trajectory: `w₀ = 0`.**

    The all-zero initial vorticity trajectory under the trivial
    Biot-Savart / zero-stretching choices is a closed instance of
    `IsSmooth (ns_trajectory_generator ...)` requiring only the
    standard CFL and non-negativity hypotheses on `NsParams`.

    Mathematically degenerate (the trajectory is constant zero), but
    the corollary completes the trivial-instantiation closure with no
    user-supplied initial-data hypothesis. -/
theorem ns_global_regularity_zero_initial
    (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) (hns : 0 < p.ns_scale)
    (T : ℕ) :
    NSRegularity.BKM.IsSmooth
      (ns_trajectory_generator trivial_biot_savart zero_stretching
        zero_stretching zero_stretching zero_vfield p) := by
  have hLInf : 0 ≤ lInf zero_vfield := by
    unfold lInf zero_vfield gridLInf
    -- The L∞ of the zero grid is `0`, since every entry has `.natAbs = 0`.
    simp
  exact ns_global_regularity_trivial_exact
    zero_vfield p h hν hdt hns hLInf (diffusion_exactness_hyp_zero p) T

end NSRegularity.NS.TrivialClosure
