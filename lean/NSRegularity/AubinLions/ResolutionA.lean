import NSRegularity.AubinLions
import NSRegularity.HSquared.SBP

/-!
# AubinLions/ResolutionA — L² Resolution A (L² monotonicity under exact diffusion)

Parallels `HSquared/ResolutionA.lean` for the L² seminorm:
  `l2Sq (diffusion_step f p) ≤ l2Sq f`
under the single exactness hypothesis `NS² ∣ cv · Δf` at every grid point
(Resolution A side condition).

The proof structure is **identical** to `h2_seminorm_per_component_exact`
with the substitution `Δf → f` and `Δ²f → Δf` throughout.

## What this gives

Combined with `sbp_quadratic` and the Poincaré inequality `Σ(Δf)² ≤ 12·Σ|∇⁺f|²`,
L² monotonicity holds under the existing CFL `6cv ≤ NS²`. This is the
discrete analogue of the L²-energy dissipation: `dE/dt = -2ν ‖∇ω‖²`.
-/

namespace NSRegularity.AubinLions

open NSRegularity NSRegularity.NS NSRegularity.Shift NSRegularity.HSquared

/-- **L² Resolution A — per-component L² monotonicity under exact diffusion.** -/
theorem l2Sq_diffusion_step_le_exact
    (f : Grid) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hns_pos : 0 < p.ns_scale)
    (hExact1 : ∀ i j k : Fin N,
        ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
          (p.nu_scaled * p.dt_latt * laplacian f i j k)) :
    l2Sq (diffusion_step f p) ≤ l2Sq f := by
  set NS2 : Int := (N : Int) ^ 2 * p.ns_scale ^ 2 with hNS2_def
  set cv : Int := p.nu_scaled * p.dt_latt with hcv_def
  have hNS2_pos : 0 < NS2 := by positivity
  have hNS2_nonneg : 0 ≤ NS2 := le_of_lt hNS2_pos
  have hcv_nonneg : 0 ≤ cv := mul_nonneg hν hdt
  have hNS2_sq_pos : 0 < NS2 * NS2 := mul_pos hNS2_pos hNS2_pos
  have hCFL : 6 * cv ≤ NS2 := by
    unfold cfl_ok at h
    have heq : 6 * cv = 6 * p.nu_scaled * p.dt_latt := by rw [hcv_def]; ring
    linarith
  -- q := diffusion correction grid
  let q : Grid := fun i j k => cv * laplacian f i j k / NS2
  -- NS2 · q = cv · Δf (Resolution A at each point)
  have hq_exact : ∀ i j k : Fin N, NS2 * q i j k = cv * laplacian f i j k := by
    intros i j k
    obtain ⟨n, hn⟩ := hExact1 i j k
    show NS2 * (cv * laplacian f i j k / NS2) = cv * laplacian f i j k
    rw [hn, Int.mul_ediv_cancel_left _ (ne_of_gt hNS2_pos)]
  have hdiff_pointwise : ∀ i j k : Fin N,
      diffusion_step f p i j k = f i j k + q i j k := by
    intros i j k; unfold diffusion_step; rfl
  have hkey_eq : ∀ i j k : Fin N,
      NS2 * diffusion_step f p i j k = NS2 * f i j k + cv * laplacian f i j k := by
    intros i j k
    rw [hdiff_pointwise, mul_add, hq_exact]
  -- Scale by NS²² then expand
  have hscale_lhs : NS2 * NS2 * l2Sq (diffusion_step f p) =
      sumGrid (fun i j k => (NS2 * diffusion_step f p i j k) ^ 2) := by
    unfold l2Sq sumGrid
    rw [Finset.mul_sum]
    apply Finset.sum_congr rfl; intros i _
    rw [Finset.mul_sum]
    apply Finset.sum_congr rfl; intros j _
    rw [Finset.mul_sum]
    apply Finset.sum_congr rfl; intros k _
    ring
  have hsub_step : sumGrid (fun i j k => (NS2 * diffusion_step f p i j k) ^ 2) =
      sumGrid (fun i j k => (NS2 * f i j k + cv * laplacian f i j k) ^ 2) := by
    unfold sumGrid
    apply Finset.sum_congr rfl; intros i _
    apply Finset.sum_congr rfl; intros j _
    apply Finset.sum_congr rfl; intros k _
    simp only [hkey_eq i j k]
  have hexpand : sumGrid (fun i j k =>
        (NS2 * f i j k + cv * laplacian f i j k) ^ 2) =
      (NS2 * NS2) * sumGrid (fun i j k => (f i j k) ^ 2) +
      2 * NS2 * cv * sumGrid (fun i j k => f i j k * laplacian f i j k) +
      (cv * cv) * sumGrid (fun i j k => (laplacian f i j k) ^ 2) := by
    unfold sumGrid
    have hpt : ∀ i j k : Fin N,
        (NS2 * f i j k + cv * laplacian f i j k) ^ 2 =
        (NS2 * NS2) * (f i j k) ^ 2 +
        2 * NS2 * cv * (f i j k * laplacian f i j k) +
        (cv * cv) * (laplacian f i j k) ^ 2 := by
      intros i j k; ring
    simp only [hpt, Finset.sum_add_distrib, ← Finset.mul_sum]
  -- Apply sbp_quadratic: Σ f · Δf = -Σ|∇⁺f|²
  have hsbp := sbp_quadratic f
  -- Apply Poincaré: Σ(Δf)² ≤ 12 · Σ|∇⁺f|²
  have hpoincare : sumGrid (fun i j k => (laplacian f i j k) ^ 2) ≤
      12 * gradSqForward f := laplacian_sq_le_12_gradSqForward f
  have hgoal_scaled :
      NS2 * NS2 * l2Sq (diffusion_step f p) ≤ NS2 * NS2 * l2Sq f := by
    rw [hscale_lhs, hsub_step, hexpand]
    have hgrad_nonneg : 0 ≤ gradSqForward f := by
      unfold gradSqForward
      apply Finset.sum_nonneg; intros i _
      apply Finset.sum_nonneg; intros j _
      apply Finset.sum_nonneg; intros k _
      have h1 := sq_nonneg (f (sp i) j k - f i j k)
      have h2 := sq_nonneg (f i (sp j) k - f i j k)
      have h3 := sq_nonneg (f i j (sp k) - f i j k)
      linarith
    show (NS2 * NS2) * sumGrid (fun i j k => (f i j k) ^ 2)
       + 2 * NS2 * cv * sumGrid (fun i j k => f i j k * laplacian f i j k)
       + cv * cv * sumGrid (fun i j k => (laplacian f i j k) ^ 2)
       ≤ NS2 * NS2 * l2Sq f
    unfold l2Sq
    rw [show (∑ i : Fin N, ∑ j : Fin N, ∑ k : Fin N, (f i j k) ^ 2)
        = sumGrid (fun i j k => (f i j k) ^ 2) from rfl]
    rw [hsbp]
    have hcv_sq_nonneg : 0 ≤ cv * cv := mul_self_nonneg cv
    have hbound1 : cv * cv * sumGrid (fun i j k => (laplacian f i j k) ^ 2) ≤
                   cv * cv * (12 * gradSqForward f) :=
      mul_le_mul_of_nonneg_left hpoincare hcv_sq_nonneg
    have hbound2 : cv * cv * (12 * gradSqForward f) ≤
                   2 * NS2 * cv * gradSqForward f := by
      have h12cv2_le : 12 * (cv * cv) ≤ 2 * NS2 * cv := by
        have hcv2 : 2 * cv * (6 * cv) ≤ 2 * cv * NS2 :=
          mul_le_mul_of_nonneg_left hCFL (by linarith : 0 ≤ 2 * cv)
        linarith
      calc cv * cv * (12 * gradSqForward f)
          = 12 * (cv * cv) * gradSqForward f := by ring
        _ ≤ 2 * NS2 * cv * gradSqForward f :=
            mul_le_mul_of_nonneg_right h12cv2_le hgrad_nonneg
    linarith
  exact le_of_mul_le_mul_left hgoal_scaled hNS2_sq_pos

end NSRegularity.AubinLions
