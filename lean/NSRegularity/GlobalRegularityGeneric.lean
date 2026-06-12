import NSRegularity.GlobalRegularity

/-!
# Global Regularity — generic L∞-preserving step closure

The trajectory-level closure in `GlobalRegularity.lean` is specialised to
`diffusion_step`. This file lifts the closure to **any** step function
`step : VField → VField` that preserves L∞.

This is the natural form to plug full NS dynamics into — once
advection_step, stretching_step, and their L∞-preservation properties
are available, the global_regularity claim follows mechanically.

## Architecture

```
Hypothesis: step preserves lInf (∀ w, lInf (step w) ≤ lInf w)
   ↓
generic_trajectory step w₀ : ℕ → VField  (iterated step)
   ↓
generic_ns_lInf_uniform : ∀ n, lInf (traj n) ≤ lInf w₀
   ↓
generic_ns_lInfGrid_uniform : ∀ n, lInfGrid (traj n) ≤ 3·N³ · lInf w₀
   ↓
generic_global_regularity : IsSmooth (traj)                  [mod BKM axiom]
```
-/

namespace NSRegularity.GlobalRegularity

open NSRegularity NSRegularity.NS NSRegularity.Continuum

/-- Generic iterated trajectory from any step function. -/
def generic_trajectory (step : VField → VField) (w₀ : VField) : ℕ → VField
  | 0     => w₀
  | n + 1 => step (generic_trajectory step w₀ n)

/-- **Generic L∞ uniform bound.** Any L∞-preserving step gives a
    uniformly-L∞-bounded trajectory. -/
theorem generic_ns_lInf_uniform
    (step : VField → VField)
    (hStep : ∀ w, lInf (step w) ≤ lInf w)
    (w₀ : VField) :
    ∀ n, lInf (generic_trajectory step w₀ n) ≤ lInf w₀ := by
  intro n
  induction n with
  | zero => exact le_refl _
  | succ n ih =>
      unfold generic_trajectory
      exact le_trans (hStep _) ih

/-- **Generic uniform L¹-proxy bound.** -/
theorem generic_ns_lInfGrid_uniform
    (step : VField → VField)
    (hStep : ∀ w, lInf (step w) ≤ lInf w)
    (w₀ : VField) :
    ∀ n, lInfGrid (generic_trajectory step w₀ n) ≤ 3 * (N : Int) ^ 3 * lInf w₀ := by
  intro n
  exact le_trans (lInfGrid_le_proxy _)
    (by
      have := generic_ns_lInf_uniform step hStep w₀ n
      have hN3 : 0 ≤ 3 * (N : Int) ^ 3 := by positivity
      exact mul_le_mul_of_nonneg_left this hN3)

/-- **Generic global regularity.** Any L∞-preserving step generates a
    trajectory whose continuum limit is smooth (mod BKM axiom).

    Pure composition — no sorries. Plug in your step function and L∞-
    preservation proof to get global regularity. -/
theorem generic_global_regularity
    (step : VField → VField)
    (hStep : ∀ w, lInf (step w) ≤ lInf w)
    (w₀ : VField)
    (hLInf : 0 ≤ lInf w₀)
    (_T : ℕ) :
    NSRegularity.BKM.IsSmooth (generic_trajectory step w₀) :=
  NSRegularity.BKM.bkm_regularity_criterion
    (generic_trajectory step w₀)
    (3 * (N : Int) ^ 3 * lInf w₀)
    (by positivity)
    (generic_ns_lInfGrid_uniform step hStep w₀)

/-- **Generic global regularity with uniform bound (most general form).**

    The maximally general closure: given ONLY a uniform L∞ bound by some
    constant `M ≥ 0` along the trajectory, BKM gives smoothness. The
    bound `M` does not need to come from L∞ non-increasing — any source
    (DKAM winding analysis, Kill #113 energy, a-priori estimates) works.

    This is the right form for the full NS step, where L∞ is uniformly
    BOUNDED (not non-increasing) due to advection + stretching being only
    perturbatively controlled by diffusion. -/
theorem generic_global_regularity_uniform
    (step : VField → VField)
    (w₀ : VField)
    (M : Int) (hM : 0 ≤ M)
    (h_uniform : ∀ n, lInf (generic_trajectory step w₀ n) ≤ M)
    (_T : ℕ) :
    NSRegularity.BKM.IsSmooth (generic_trajectory step w₀) := by
  have hLInfGrid_bound : ∀ n, lInfGrid (generic_trajectory step w₀ n) ≤
                              3 * (N : Int) ^ 3 * M := by
    intro n
    exact le_trans (lInfGrid_le_proxy _)
      (by
        have hN3 : 0 ≤ 3 * (N : Int) ^ 3 := by positivity
        exact mul_le_mul_of_nonneg_left (h_uniform n) hN3)
  exact NSRegularity.BKM.bkm_regularity_criterion
    (generic_trajectory step w₀)
    (3 * (N : Int) ^ 3 * M)
    (by positivity)
    hLInfGrid_bound

-- ═══════════════════════════════════════════════════════════════════════════
-- Specialisation: pure diffusion (recovers the original global_regularity)
-- ═══════════════════════════════════════════════════════════════════════════

/-- The pure-diffusion vector-step. -/
def diffusion_vstep (p : NsParams) (w : VField) : VField :=
  ⟨diffusion_step w.x p, diffusion_step w.y p, diffusion_step w.z p⟩

/-- Show diffusion_trajectory ≡ generic_trajectory diffusion_vstep. -/
theorem diffusion_trajectory_eq_generic (w₀ : VField) (p : NsParams) :
    diffusion_trajectory w₀ p = generic_trajectory (diffusion_vstep p) w₀ := by
  funext n
  induction n with
  | zero => rfl
  | succ n ih =>
      show diffusion_trajectory w₀ p (n + 1) =
            generic_trajectory (diffusion_vstep p) w₀ (n + 1)
      simp only [diffusion_trajectory, generic_trajectory, diffusion_vstep, ih]

/-- **Per-component scalar `gridLInf` non-increase under one diffusion step
    (Resolution A).**

    Lift of `DKAM.Diffusion.diffusion_max_preserving_exact_natAbs` (pointwise)
    to the grid-level `gridLInf` via the three nested `Finset.sup'_le`. The
    inline copies of this argument inside `diffusion_vstep_preserves_lInf_exact`
    and `diffusion_max_preserving_exact_natAbs` (in `DKAM.lean`) are replaced
    by a call to this named lemma. Useful as an externally-reusable building
    block for any operator-splitting argument that needs to bound the
    `gridLInf` of `diffusion_step f p` (e.g. the full-step L∞ gate). -/
theorem gridLInf_diffusion_step_le_exact
    (f : Grid) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i j k : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * NSRegularity.NS.laplacian f i j k)) :
    gridLInf (NSRegularity.NS.diffusion_step f p) ≤ gridLInf f := by
  unfold gridLInf
  apply Finset.sup'_le; intro i _
  apply Finset.sup'_le; intro j _
  apply Finset.sup'_le; intro k _
  exact NSRegularity.DKAM.Diffusion.diffusion_max_preserving_exact_natAbs
    f p h hν hdt hns_pos hExact i j k

/-- **Diffusion-step L∞ preservation** (Resolution A formulation).
    Needs the exactness hypothesis on the input field. Returns the
    L∞-preservation property in form usable by `generic_global_regularity`. -/
theorem diffusion_vstep_preserves_lInf_exact
    (p : NsParams) (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hns_pos : 0 < p.ns_scale)
    (w : VField)
    (hExact :
        (∀ i j k : Fin N,
          ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
            (p.nu_scaled * p.dt_latt * NSRegularity.NS.laplacian w.x i j k)) ∧
        (∀ i j k : Fin N,
          ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
            (p.nu_scaled * p.dt_latt * NSRegularity.NS.laplacian w.y i j k)) ∧
        (∀ i j k : Fin N,
          ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
            (p.nu_scaled * p.dt_latt * NSRegularity.NS.laplacian w.z i j k))) :
    lInf (diffusion_vstep p w) ≤ lInf w := by
  unfold diffusion_vstep
  -- per-component bounds via the extracted named helper.
  have hx := gridLInf_diffusion_step_le_exact w.x p h hν hdt hns_pos hExact.1
  have hy := gridLInf_diffusion_step_le_exact w.y p h hν hdt hns_pos hExact.2.1
  have hz := gridLInf_diffusion_step_le_exact w.z p h hν hdt hns_pos hExact.2.2
  unfold lInf
  exact max_le_max hx (max_le_max hy hz)

end NSRegularity.GlobalRegularity
