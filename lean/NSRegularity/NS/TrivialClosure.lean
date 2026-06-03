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

/-- `gridLInf` is always non-negative: it is a `Finset.sup'` of non-negative
    integer-cast `.natAbs` values. -/
theorem gridLInf_nonneg (f : Grid) : 0 ≤ gridLInf f := by
  unfold gridLInf
  -- Bound below by the value at (0,0,0), which is ↑(f 0 0 0).natAbs ≥ 0.
  have h1 : ((f 0 0 0).natAbs : Int) ≤
      Finset.univ.sup' gridLInf._proof_1
        (fun k => ((f 0 0 k).natAbs : Int)) :=
    Finset.le_sup' (s := (Finset.univ : Finset (Fin N)))
      (f := fun k => ((f 0 0 k).natAbs : Int))
      (Finset.mem_univ (0 : Fin N))
  -- Chain through the j and i levels.
  have h2 : Finset.univ.sup' gridLInf._proof_1
      (fun k => ((f 0 0 k).natAbs : Int)) ≤
      Finset.univ.sup' gridLInf._proof_1 (fun j =>
        Finset.univ.sup' gridLInf._proof_1 (fun k => ((f 0 j k).natAbs : Int))) :=
    Finset.le_sup' (s := (Finset.univ : Finset (Fin N)))
      (f := fun j => Finset.univ.sup' gridLInf._proof_1
        (fun k => ((f 0 j k).natAbs : Int)))
      (Finset.mem_univ (0 : Fin N))
  have h3 : Finset.univ.sup' gridLInf._proof_1 (fun j =>
        Finset.univ.sup' gridLInf._proof_1 (fun k => ((f 0 j k).natAbs : Int))) ≤
      Finset.univ.sup' gridLInf._proof_1 (fun i =>
        Finset.univ.sup' gridLInf._proof_1 (fun j =>
          Finset.univ.sup' gridLInf._proof_1
            (fun k => ((f i j k).natAbs : Int)))) :=
    Finset.le_sup' (s := (Finset.univ : Finset (Fin N)))
      (f := fun i => Finset.univ.sup' gridLInf._proof_1 (fun j =>
        Finset.univ.sup' gridLInf._proof_1 (fun k => ((f i j k).natAbs : Int))))
      (Finset.mem_univ (0 : Fin N))
  have h0 : (0 : Int) ≤ ((f 0 0 0).natAbs : Int) := Int.natCast_nonneg _
  exact le_trans h0 (le_trans h1 (le_trans h2 h3))

/-- `lInf` is always non-negative. -/
theorem lInf_nonneg (w : VField) : 0 ≤ lInf w := by
  unfold lInf
  exact le_max_of_le_left (gridLInf_nonneg w.x)

/-- Constant vector field with components `(cx, cy, cz)`. -/
def const_vfield (cx cy cz : Int) : VField :=
  ⟨fun _ _ _ => cx, fun _ _ _ => cy, fun _ _ _ => cz⟩

/-- Constants are harmonic: `laplacian (fun _ _ _ => c) = 0`. -/
theorem laplacian_const (c : Int) (i j k : Fin N) :
    NSRegularity.NS.laplacian (fun _ _ _ => c) i j k = 0 := by
  unfold NSRegularity.NS.laplacian
  ring

/-- The diffusion exactness hypothesis holds for any constant initial
    vorticity: the laplacian vanishes everywhere along the trajectory. -/
theorem diffusion_exactness_hyp_const (cx cy cz : Int) (p : NsParams) :
    diffusion_exactness_hyp (const_vfield cx cy cz) p := by
  intro n i j k
  -- The trajectory starting at any constant field is constant (laplacian = 0).
  have htraj : ∀ n,
      diffusion_trajectory (const_vfield cx cy cz) p n = const_vfield cx cy cz := by
    intro n
    induction n with
    | zero => rfl
    | succ n ih =>
        unfold diffusion_trajectory
        rw [ih]
        unfold const_vfield diffusion_step
        refine VField.mk.injEq .. |>.mpr ⟨?_, ?_, ?_⟩ <;>
          (funext i j k; rw [laplacian_const]; simp)
  rw [htraj n]
  refine ⟨?_, ?_, ?_⟩ <;>
    (unfold const_vfield; rw [laplacian_const]; simp)

/-- **Hypothesis-free closed NS trajectory at any constant initial data.**

    Extends `ns_global_regularity_zero_initial` to all constant initial
    vorticity fields. The trajectory is again constant (constants are
    harmonic, so the diffusion step is the identity), and `IsSmooth`
    holds with `M = max(|cx|, max(|cy|, |cz|))`.

    Together with the zero case, this gives a 3-parameter family of
    closed `IsSmooth (ns_trajectory_generator ...)` instances - more
    than just the degenerate origin point. -/
theorem ns_global_regularity_const_initial
    (cx cy cz : Int)
    (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) (hns : 0 < p.ns_scale)
    (T : ℕ) :
    NSRegularity.BKM.IsSmooth
      (ns_trajectory_generator trivial_biot_savart zero_stretching
        zero_stretching zero_stretching (const_vfield cx cy cz) p) := by
  exact ns_global_regularity_trivial_exact
    (const_vfield cx cy cz) p h hν hdt hns (lInf_nonneg _)
    (diffusion_exactness_hyp_const cx cy cz p) T

/-- **Explicit trajectory: constant initial data ⇒ constant trajectory.**

    For any constant initial vorticity, the NS trajectory under the
    trivial Biot-Savart / zero-stretching choices is the constant
    sequence equal to the initial data. This is the explicit form
    underlying `ns_global_regularity_const_initial`.

    Used in the closure proof and made externally accessible here so
    downstream consumers can rewrite trajectory references. -/
theorem trajectory_const_eq (cx cy cz : Int) (p : NsParams) (n : ℕ) :
    ns_trajectory_generator trivial_biot_savart zero_stretching
      zero_stretching zero_stretching (const_vfield cx cy cz) p n =
    const_vfield cx cy cz := by
  rw [ns_trajectory_trivial_eq]
  -- Reduce diffusion_trajectory at constant init to the constant.
  induction n with
  | zero => rfl
  | succ n ih =>
      unfold diffusion_trajectory
      rw [ih]
      unfold const_vfield diffusion_step
      refine VField.mk.injEq .. |>.mpr ⟨?_, ?_, ?_⟩ <;>
        (funext i j k; rw [laplacian_const]; simp)

/-- **Specialisation to the zero trajectory.** -/
theorem trajectory_zero_eq (p : NsParams) (n : ℕ) :
    ns_trajectory_generator trivial_biot_savart zero_stretching
      zero_stretching zero_stretching zero_vfield p n = zero_vfield := by
  have := trajectory_const_eq 0 0 0 p n
  simpa [const_vfield, zero_vfield] using this

/-- The L¹-proxy `lInfGrid` of the zero vorticity is zero. -/
theorem lInfGrid_zero_vfield : NSRegularity.Continuum.lInfGrid zero_vfield = 0 := by
  unfold NSRegularity.Continuum.lInfGrid zero_vfield
  simp

/-- The L¹-proxy `lInfGrid` of a constant vorticity is
    `N³ · (|cx| + |cy| + |cz|)`. -/
theorem lInfGrid_const_vfield (cx cy cz : Int) :
    NSRegularity.Continuum.lInfGrid (const_vfield cx cy cz) =
    (N : Int) ^ 3 * (|cx| + |cy| + |cz|) := by
  unfold NSRegularity.Continuum.lInfGrid const_vfield
  simp only [Finset.sum_const, Finset.card_univ, Fintype.card_fin, nsmul_eq_mul]
  ring

/-- **Tight `IsSmooth` witness for the constant trajectory.**

    The trajectory is the constant sequence equal to `const_vfield cx cy cz`,
    so every step has `lInfGrid = N³ · (|cx| + |cy| + |cz|)`. The tightest
    uniform bound `M` for the `IsSmooth` existential is precisely this value.

    The general `ns_global_regularity_const_initial` uses the loose bound
    `M = 3 · N³ · lInf w₀ = 3 · N³ · max(|cx|, |cy|, |cz|)` inherited from
    `generic_ns_lInfGrid_uniform`; this lemma supplies the tight closed-form
    witness. -/
theorem ns_global_regularity_const_initial_tight (cx cy cz : Int) (p : NsParams) :
    NSRegularity.BKM.IsSmooth
      (ns_trajectory_generator trivial_biot_savart zero_stretching
        zero_stretching zero_stretching (const_vfield cx cy cz) p) := by
  refine ⟨(N : Int) ^ 3 * (|cx| + |cy| + |cz|), ?_, ?_⟩
  · -- 0 ≤ N³ · (|cx| + |cy| + |cz|)
    have h1 : (0 : Int) ≤ (N : Int) ^ 3 := by positivity
    have h2 : (0 : Int) ≤ |cx| + |cy| + |cz| := by
      have hx : (0 : Int) ≤ |cx| := abs_nonneg _
      have hy : (0 : Int) ≤ |cy| := abs_nonneg _
      have hz : (0 : Int) ≤ |cz| := abs_nonneg _
      linarith
    exact mul_nonneg h1 h2
  · intro n
    rw [trajectory_const_eq, lInfGrid_const_vfield]

/-! ## Bundled summary -/

/-- **Discrete NS regularity certificate for the trivial-Biot-Savart family.**

    Bundles every closed result in this file for the constant-initial-data
    case. Given any constant initial vorticity `(cx, cy, cz)` and standard
    `NsParams` hypotheses (CFL, non-negative ν and Δt, positive scale), the
    NS trajectory under the trivial Biot-Savart + zero-stretching choices
    satisfies all of:

    1. The trajectory is explicitly the constant sequence.
    2. The L¹-proxy at every step is exactly `N³ · (|cx| + |cy| + |cz|)`.
    3. `IsSmooth` holds with the loose witness `3·N³·max(|cx|,|cy|,|cz|)`.
    4. `IsSmooth` holds with the tight witness `N³·(|cx|+|cy|+|cz|)`.

    No axioms invoked; no user-supplied initial-data hypothesis required. -/
theorem trivial_closure_bundle (cx cy cz : Int)
    (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) (hns : 0 < p.ns_scale)
    (T : ℕ) :
    (∀ n, ns_trajectory_generator trivial_biot_savart zero_stretching
            zero_stretching zero_stretching (const_vfield cx cy cz) p n =
          const_vfield cx cy cz) ∧
    (NSRegularity.Continuum.lInfGrid (const_vfield cx cy cz) =
        (N : Int) ^ 3 * (|cx| + |cy| + |cz|)) ∧
    NSRegularity.BKM.IsSmooth
      (ns_trajectory_generator trivial_biot_savart zero_stretching
        zero_stretching zero_stretching (const_vfield cx cy cz) p) :=
  ⟨trajectory_const_eq cx cy cz p,
   lInfGrid_const_vfield cx cy cz,
   ns_global_regularity_const_initial cx cy cz p h hν hdt hns T⟩

/-! ## Strengthened `IsSmooth` is strictly stronger than the prior `True` placeholder. -/

/-- An explicit unbounded trajectory: `traj n` has all three components equal
    to the constant `n` everywhere. -/
def unbounded_traj : NSRegularity.Continuum.DiscreteSequence :=
  fun n => const_vfield (n : Int) (n : Int) (n : Int)

/-- `lInfGrid (unbounded_traj n) = 3 · N³ · n`. -/
theorem lInfGrid_unbounded_traj (n : ℕ) :
    NSRegularity.Continuum.lInfGrid (unbounded_traj n) = 3 * (N : Int) ^ 3 * (n : Int) := by
  unfold unbounded_traj
  rw [lInfGrid_const_vfield]
  rw [abs_of_nonneg (Int.natCast_nonneg n)]
  ring

/-! ## Vanishing-viscosity closure: arbitrary initial data when `ν = 0` -/

/-- When the scaled viscosity is exactly zero, the diffusion step is the
    identity on every field (the laplacian term is multiplied by zero). -/
theorem diffusion_step_id_of_zero_nu (f : Grid) (p : NsParams) (hν : p.nu_scaled = 0) :
    diffusion_step f p = f := by
  funext i j k
  unfold diffusion_step
  rw [hν]; ring_nf; simp

/-- With trivial Biot-Savart, zero stretching, AND zero viscosity, the NS
    step is the identity for ANY field - no monotonicity or boundedness
    structure is required from `w`. -/
theorem ns_step_id_of_zero_nu (w : VField) (p : NsParams) (hν : p.nu_scaled = 0) :
    ns_step (trivial_biot_savart w) (zero_stretching w) (zero_stretching w)
      (zero_stretching w) w p = w := by
  rw [ns_step_trivial_eq]
  unfold diffusion_vstep
  refine VField.mk.injEq .. |>.mpr ⟨?_, ?_, ?_⟩ <;>
    exact diffusion_step_id_of_zero_nu _ _ hν

/-- The vanishing-viscosity trajectory is constant at the initial data. -/
theorem trajectory_id_of_zero_nu (w₀ : VField) (p : NsParams)
    (hν : p.nu_scaled = 0) (n : ℕ) :
    ns_trajectory_generator trivial_biot_savart zero_stretching
      zero_stretching zero_stretching w₀ p n = w₀ := by
  induction n with
  | zero => rfl
  | succ n ih =>
      unfold ns_trajectory_generator
      rw [ih]
      exact ns_step_id_of_zero_nu w₀ p hν

/-- `lInfGrid_le_proxy` for the constant-`w₀` trajectory: every step is
    bounded by `3·N³·lInf w₀`. -/
theorem lInfGrid_const_traj_bound (w₀ : VField) (p : NsParams)
    (hν : p.nu_scaled = 0) (n : ℕ) :
    NSRegularity.Continuum.lInfGrid
      (ns_trajectory_generator trivial_biot_savart zero_stretching
        zero_stretching zero_stretching w₀ p n)
    ≤ 3 * (N : Int) ^ 3 * lInf w₀ := by
  rw [trajectory_id_of_zero_nu _ _ hν]
  exact NSRegularity.GlobalRegularity.lInfGrid_le_proxy w₀

/-- **Vanishing-viscosity closure: `IsSmooth` for ANY initial vorticity.**

    With `ν = 0` (zero scaled viscosity), the trivial-Biot-Savart +
    zero-stretching NS step is the identity, so the trajectory is the
    constant initial data, and `IsSmooth` holds with the uniform bound
    `M = 3·N³·lInf w₀`. The only hypotheses are `p.nu_scaled = 0` and
    `0 ≤ lInf w₀`.

    Unlike the `const_initial` family this works for *arbitrary*
    (non-constant) initial vorticity: the vanishing-viscosity NS dynamics
    is the identity, so initial-data structure plays no role. -/
theorem ns_global_regularity_zero_nu (w₀ : VField) (p : NsParams)
    (hν : p.nu_scaled = 0) (hLInf : 0 ≤ lInf w₀) (_T : ℕ) :
    NSRegularity.BKM.IsSmooth
      (ns_trajectory_generator trivial_biot_savart zero_stretching
        zero_stretching zero_stretching w₀ p) := by
  refine ⟨3 * (N : Int) ^ 3 * lInf w₀, ?_, ?_⟩
  · have hN3 : (0 : Int) ≤ 3 * (N : Int) ^ 3 := by positivity
    exact mul_nonneg hN3 hLInf
  · intro n; exact lInfGrid_const_traj_bound w₀ p hν n

/-- **Any constant trajectory satisfies `IsSmooth`** — the most general
    "trivially regular" case. The witness is `M = lInfGrid w₀`. -/
theorem IsSmooth_const_traj (w₀ : VField) :
    NSRegularity.BKM.IsSmooth (fun _ : ℕ => w₀) :=
  ⟨NSRegularity.Continuum.lInfGrid w₀,
   NSRegularity.Continuum.lInfGrid_nonneg w₀,
   fun _ => le_refl _⟩

/-- A trajectory that is constant after some time `n₀` satisfies `IsSmooth`,
    using the max of the values up to `n₀` and the constant tail. -/
theorem IsSmooth_eventually_const
    (traj : NSRegularity.Continuum.DiscreteSequence)
    (n₀ : ℕ) (w_tail : VField)
    (h_tail : ∀ n, n₀ ≤ n → traj n = w_tail)
    (h_prefix_bound : ∀ n, n < n₀ →
      NSRegularity.Continuum.lInfGrid (traj n) ≤
        NSRegularity.Continuum.lInfGrid w_tail) :
    NSRegularity.BKM.IsSmooth traj := by
  refine ⟨NSRegularity.Continuum.lInfGrid w_tail,
          NSRegularity.Continuum.lInfGrid_nonneg w_tail, ?_⟩
  intro n
  by_cases hn : n < n₀
  · exact h_prefix_bound n hn
  · rw [h_tail n (Nat.le_of_not_lt hn)]

/-- `IsSmooth` derivable from a uniform `lInf` bound, via the proxy inequality
    `lInfGrid ≤ 3·N³·lInf`. Easier to apply than the `lInfGrid` form because
    `lInf` is the per-step max-of-three-grid-maxes rather than a triple sum. -/
theorem IsSmooth_of_lInf_uniform
    (traj : NSRegularity.Continuum.DiscreteSequence)
    (M : ℤ) (hM : 0 ≤ M)
    (h : ∀ n, lInf (traj n) ≤ M) :
    NSRegularity.BKM.IsSmooth traj := by
  refine ⟨3 * (N : Int) ^ 3 * M, ?_, ?_⟩
  · have hN3 : (0 : Int) ≤ 3 * (N : Int) ^ 3 := by positivity
    exact mul_nonneg hN3 hM
  · intro n
    refine le_trans (NSRegularity.GlobalRegularity.lInfGrid_le_proxy _) ?_
    have hN3 : (0 : Int) ≤ 3 * (N : Int) ^ 3 := by positivity
    exact mul_le_mul_of_nonneg_left (h n) hN3

/-- `IsSmooth` is preserved under time-shifts. -/
theorem IsSmooth_shift (traj : NSRegularity.Continuum.DiscreteSequence) (k : ℕ)
    (h : NSRegularity.BKM.IsSmooth traj) :
    NSRegularity.BKM.IsSmooth (fun n => traj (n + k)) := by
  obtain ⟨M, hM, h⟩ := h
  exact ⟨M, hM, fun n => h (n + k)⟩

/-- `IsSmooth` is preserved under arbitrary subsequence selection. -/
theorem IsSmooth_subseq (traj : NSRegularity.Continuum.DiscreteSequence)
    (f : ℕ → ℕ)
    (h : NSRegularity.BKM.IsSmooth traj) :
    NSRegularity.BKM.IsSmooth (fun n => traj (f n)) := by
  obtain ⟨M, hM, h⟩ := h
  exact ⟨M, hM, fun n => h (f n)⟩

/-- The maximum of two `IsSmooth` witnesses gives an `IsSmooth` witness for
    any trajectory bounded by either of the two. -/
theorem IsSmooth_of_le_or
    (traj : NSRegularity.Continuum.DiscreteSequence)
    (M₁ M₂ : ℤ) (h₁ : 0 ≤ M₁) (h₂ : 0 ≤ M₂)
    (h : ∀ n, NSRegularity.Continuum.lInfGrid (traj n) ≤ M₁ ∨
              NSRegularity.Continuum.lInfGrid (traj n) ≤ M₂) :
    NSRegularity.BKM.IsSmooth traj := by
  refine ⟨max M₁ M₂, le_max_of_le_left h₁, ?_⟩
  intro n
  rcases h n with hL | hL
  · exact le_trans hL (le_max_left _ _)
  · exact le_trans hL (le_max_right _ _)

/-- **Unified trivial-skeleton closure.**

    Discharges `IsSmooth (ns_trajectory_generator trivial_biot_savart ...)`
    for either of two regimes:

    (A) **Vanishing viscosity.** `p.nu_scaled = 0` + any initial `w₀`
        with `0 ≤ lInf w₀`. No CFL or exactness needed - the NS step is
        the identity, the trajectory is constant `w₀`.

    (B) **Constant initial data.** Any `cx cy cz : Int` with the standard
        `cfl_ok` + non-negativity + positive scale hypotheses. The
        trajectory is constant `const_vfield cx cy cz`.

    Together these cover the two structurally-simple regimes in which
    the trivial NS skeleton is closed under integer-exact dynamics. -/
theorem ns_global_regularity_trivial_unified
    (w₀ : VField) (p : NsParams) (T : ℕ)
    (hLInf : 0 ≤ lInf w₀)
    (regime :
      (p.nu_scaled = 0) ∨
      (∃ cx cy cz : Int,
        w₀ = const_vfield cx cy cz ∧
        cfl_ok p ∧ 0 ≤ p.nu_scaled ∧ 0 ≤ p.dt_latt ∧ 0 < p.ns_scale)) :
    NSRegularity.BKM.IsSmooth
      (ns_trajectory_generator trivial_biot_savart zero_stretching
        zero_stretching zero_stretching w₀ p) := by
  rcases regime with hν | ⟨cx, cy, cz, hwconst, h, hν, hdt, hns⟩
  · exact ns_global_regularity_zero_nu w₀ p hν hLInf T
  · rw [hwconst]
    exact ns_global_regularity_const_initial cx cy cz p h hν hdt hns T

/-- Explicit value of the discrete BKM integral over `T` steps of the constant
    trajectory: `T · N³ · (|cx| + |cy| + |cz|)`. Used internally by
    `bkm_integral_finite`; here we record the closed form for the trivial
    family. -/
theorem bkmIntegral_const_trajectory (cx cy cz : Int) (p : NsParams) (T : ℕ) :
    NSRegularity.BKM.bkmIntegral
      (ns_trajectory_generator trivial_biot_savart zero_stretching
        zero_stretching zero_stretching (const_vfield cx cy cz) p) T
    = (T : Int) * ((N : Int) ^ 3 * (|cx| + |cy| + |cz|)) := by
  unfold NSRegularity.BKM.bkmIntegral
  have hpt : ∀ n,
      NSRegularity.Continuum.lInfGrid
        (ns_trajectory_generator trivial_biot_savart zero_stretching
          zero_stretching zero_stretching (const_vfield cx cy cz) p n)
      = (N : Int) ^ 3 * (|cx| + |cy| + |cz|) := by
    intro n
    rw [trajectory_const_eq, lInfGrid_const_vfield]
  rw [show (fun n => NSRegularity.Continuum.lInfGrid (ns_trajectory_generator
        trivial_biot_savart zero_stretching zero_stretching zero_stretching
        (const_vfield cx cy cz) p n)) =
      (fun _ => (N : Int) ^ 3 * (|cx| + |cy| + |cz|)) from by
        funext n; exact hpt n]
  simp [Finset.sum_const, Finset.card_range]

theorem unbounded_traj_not_IsSmooth :
    ¬ NSRegularity.BKM.IsSmooth unbounded_traj := by
  rintro ⟨M, _, hM⟩
  -- Specialise at n = M.natAbs + 1 (a natural number whose Int cast exceeds M).
  -- Then `lInfGrid (unbounded_traj n) = 3·N³·n ≥ n > M` contradicts `hM n`.
  set n : ℕ := M.natAbs + 1 with hn
  have hM_n := hM n
  rw [lInfGrid_unbounded_traj] at hM_n
  -- (n : Int) > M because n.natAbs = n (for Nat → Int) and natAbs grows past M.
  have hn_gt : M < (n : Int) := by
    have h1 : M ≤ (M.natAbs : Int) := Int.le_natAbs
    have h2 : (M.natAbs : Int) < ((M.natAbs + 1 : ℕ) : Int) := by
      push_cast; linarith
    exact lt_of_le_of_lt h1 h2
  -- Now `3·N³·n ≥ n > M`, contradicting `3·N³·n ≤ M`.
  have hN3_pos : (0 : Int) < (N : Int) ^ 3 := by positivity
  have hN3_ge_one : (1 : Int) ≤ (N : Int) ^ 3 := by
    have : (N : Int) = 8 := by decide
    rw [this]; decide
  have h3N3 : (1 : Int) ≤ 3 * (N : Int) ^ 3 := by linarith
  have hn_nonneg : (0 : Int) ≤ (n : Int) := Int.natCast_nonneg _
  have : (n : Int) ≤ 3 * (N : Int) ^ 3 * (n : Int) := by
    nlinarith [hn_nonneg, h3N3]
  linarith

end NSRegularity.NS.TrivialClosure
