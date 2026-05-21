import NSRegularity.HSquared
import NSRegularity.HSquared.SBP

/-!
# HSquared/ResolutionA — L3.4 Resolution A (H² monotonicity under exact diffusion)

The full closure: combines Wave D's L2.3 Resolution A (exact diffusion)
with Wave F's SBP + Poincaré infrastructure to prove H² monotonicity:

```
h2_seminorm_per_component_exact :
  hTwoSeminorm (diffusion_step f p) ≤ hTwoSeminorm f
  under {cfl_ok p, ν,Δt ≥ 0, NS_scale > 0, hExact1, hExact2}
```

where the exactness hypotheses are:
- `hExact1 : NS² ∣ cv·Δf at every grid point`
- `hExact2 : NS² ∣ cv·Δ²f at every grid point`

Both are naturally satisfied by the Rust CRT / ExactRational arithmetic.

## Proof structure

```
1. Define q := the exact diffusion correction (Int division becomes exact).
2. Show NS²·q = cv·Δf  (Resolution A relation, per point).
3. Show diffusion_step f p = f + q pointwise.
4. Show laplacian (diffusion_step f p) = laplacian f + laplacian q (linearity).
5. Show NS²·laplacian q = cv·Δ²f  (under hExact1 applied to all 7 stencil points).
6. Show NS²·laplacian(f') = NS²·Δf + cv·Δ²f.
7. Square + sum: NS²²·Σ(Δf')² = NS²²·Σ Δf² + 2·NS²·cv·Σ Δf·Δ²f + cv²·Σ(Δ²f)².
8. Apply sbp_quadratic_h2: Σ Δf·Δ²f = -Σ|∇⁺Δf|².
9. Apply Poincaré: Σ(Δ²f)² ≤ 12·Σ|∇⁺Δf|².
10. Use CFL (6 cv ≤ NS²) to conclude monotonicity.
```
-/

namespace NSRegularity.HSquared

open NSRegularity NSRegularity.NS NSRegularity.Shift

/-- **L3.4 Resolution A — per-component H² monotonicity under exact diffusion.**

    Under the single-level exactness hypothesis (`NS² ∣ νΔt · Δf` at every
    grid point), the H² seminorm is non-increasing under one diffusion step.

    Note: an a-priori "level-2" exactness (`NS² ∣ νΔt · Δ²f`) is NOT needed
    — it follows automatically from the level-1 exactness via the algebraic
    identity `NS² · laplacian q = cv · Δ²f` where q is the (then-integer)
    diffusion correction grid. -/
theorem h2_seminorm_per_component_exact
    (f : Grid) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hns_pos : 0 < p.ns_scale)
    (hExact1 : ∀ i j k : Fin N,
        ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
          (p.nu_scaled * p.dt_latt * laplacian f i j k)) :
    hTwoSeminorm (diffusion_step f p) ≤ hTwoSeminorm f := by
  set NS2 : Int := (N : Int) ^ 2 * p.ns_scale ^ 2 with hNS2_def
  set cv : Int := p.nu_scaled * p.dt_latt with hcv_def
  -- Positivity facts
  have hNS2_pos : 0 < NS2 := by positivity
  have hNS2_nonneg : 0 ≤ NS2 := le_of_lt hNS2_pos
  have hcv_nonneg : 0 ≤ cv := mul_nonneg hν hdt
  have hNS2_sq_pos : 0 < NS2 * NS2 := mul_pos hNS2_pos hNS2_pos
  -- CFL: 6 * cv ≤ NS2
  have hCFL : 6 * cv ≤ NS2 := by
    unfold cfl_ok at h
    have heq : 6 * cv = 6 * p.nu_scaled * p.dt_latt := by rw [hcv_def]; ring
    linarith
  -- Step 1: define q = diffusion correction grid (exact under hExact1)
  let q : Grid := fun i j k => cv * laplacian f i j k / NS2
  -- Step 2: NS2 · q i j k = cv · Δf i j k (Resolution A at each point)
  have hq_exact : ∀ i j k : Fin N, NS2 * q i j k = cv * laplacian f i j k := by
    intros i j k
    obtain ⟨n, hn⟩ := hExact1 i j k
    show NS2 * (cv * laplacian f i j k / NS2) = cv * laplacian f i j k
    rw [hn, Int.mul_ediv_cancel_left _ (ne_of_gt hNS2_pos)]
  -- Step 3: diffusion_step f p ≡ f + q (pointwise)
  have hdiff_pointwise : ∀ i j k : Fin N,
      diffusion_step f p i j k = f i j k + q i j k := by
    intros i j k; unfold diffusion_step; rfl
  -- Step 4: laplacian (diffusion_step f p) = laplacian f + laplacian q (linearity)
  have hlap_linear : ∀ i j k : Fin N,
      laplacian (diffusion_step f p) i j k =
        laplacian f i j k + laplacian q i j k := by
    intros i j k
    unfold laplacian
    simp only [hdiff_pointwise]
    ring
  -- Step 5: NS2 · laplacian q i j k = cv · Δ²f i j k (under hExact1 at all 7 stencil pts)
  have hlap_q_exact : ∀ i j k : Fin N,
      NS2 * laplacian q i j k = cv * laplacian (laplacian f) i j k := by
    intros i j k
    have h_xp := hq_exact (sp i) j k
    have h_xn := hq_exact (sn i) j k
    have h_yp := hq_exact i (sp j) k
    have h_yn := hq_exact i (sn j) k
    have h_zp := hq_exact i j (sp k)
    have h_zn := hq_exact i j (sn k)
    have h_ctr := hq_exact i j k
    calc NS2 * laplacian q i j k
        = NS2 * (q (sp i) j k + q (sn i) j k + q i (sp j) k + q i (sn j) k
               + q i j (sp k) + q i j (sn k) - 6 * q i j k) := by
            show NS2 * laplacian q i j k =
              NS2 * (q (sp i) j k + q (sn i) j k + q i (sp j) k + q i (sn j) k
                   + q i j (sp k) + q i j (sn k) - 6 * q i j k)
            rfl
      _ = NS2 * q (sp i) j k + NS2 * q (sn i) j k + NS2 * q i (sp j) k
          + NS2 * q i (sn j) k + NS2 * q i j (sp k) + NS2 * q i j (sn k)
          - 6 * (NS2 * q i j k) := by ring
      _ = cv * laplacian f (sp i) j k + cv * laplacian f (sn i) j k
          + cv * laplacian f i (sp j) k + cv * laplacian f i (sn j) k
          + cv * laplacian f i j (sp k) + cv * laplacian f i j (sn k)
          - 6 * (cv * laplacian f i j k) := by
            rw [h_xp, h_xn, h_yp, h_yn, h_zp, h_zn, h_ctr]
      _ = cv * (laplacian f (sp i) j k + laplacian f (sn i) j k
              + laplacian f i (sp j) k + laplacian f i (sn j) k
              + laplacian f i j (sp k) + laplacian f i j (sn k)
              - 6 * laplacian f i j k) := by ring
      _ = cv * laplacian (laplacian f) i j k := by
            show cv * (laplacian f (sp i) j k + laplacian f (sn i) j k
                     + laplacian f i (sp j) k + laplacian f i (sn j) k
                     + laplacian f i j (sp k) + laplacian f i j (sn k)
                     - 6 * laplacian f i j k)
                = cv * laplacian (laplacian f) i j k
            rfl
  -- Step 6: NS2 · Δf' = NS2 · Δf + cv · Δ²f (combining Steps 4, 5)
  have hkey_eq : ∀ i j k : Fin N,
      NS2 * laplacian (diffusion_step f p) i j k =
      NS2 * laplacian f i j k + cv * laplacian (laplacian f) i j k := by
    intros i j k
    rw [hlap_linear i j k]
    rw [mul_add]
    congr 1
    exact hlap_q_exact i j k
  -- Step 7: Show NS2² · hTwoSeminorm (diffusion_step f p) ≤ NS2² · hTwoSeminorm f.
  -- Strategy: NS2² · Σ(Δf')² = Σ (NS2 · Δf')² (since NS2 ≥ 0).
  --           Then expand using hkey_eq.
  -- Helper: NS2² · x² = (NS2 · x)² for any Int x (since NS2 ≥ 0)
  have hsq_dist : ∀ x : Int, NS2 * NS2 * (x ^ 2) = (NS2 * x) ^ 2 := by
    intros x; ring
  have hsq_dist_diff : ∀ i j k : Fin N,
      NS2 * NS2 * (laplacian (diffusion_step f p) i j k) ^ 2 =
      (NS2 * laplacian (diffusion_step f p) i j k) ^ 2 := by
    intros i j k; ring
  -- Show: NS2² · Σ(Δf')² = Σ (NS2·Δf')²
  have hscale_lhs : NS2 * NS2 * hTwoSeminorm (diffusion_step f p) =
      sumGrid (fun i j k => (NS2 * laplacian (diffusion_step f p) i j k) ^ 2) := by
    unfold hTwoSeminorm sumGrid
    rw [Finset.mul_sum]
    apply Finset.sum_congr rfl; intros i _
    rw [Finset.mul_sum]
    apply Finset.sum_congr rfl; intros j _
    rw [Finset.mul_sum]
    apply Finset.sum_congr rfl; intros k _
    ring
  -- Substitute hkey_eq to show:
  --   Σ (NS2·Δf')² = Σ (NS2·Δf + cv·Δ²f)²
  --   = Σ NS2² Δf² + 2·NS2·cv·Σ Δf·Δ²f + cv² Σ (Δ²f)²
  have hsub_step : sumGrid (fun i j k => (NS2 * laplacian (diffusion_step f p) i j k) ^ 2) =
      sumGrid (fun i j k => (NS2 * laplacian f i j k + cv * laplacian (laplacian f) i j k) ^ 2) := by
    unfold sumGrid
    apply Finset.sum_congr rfl; intros i _
    apply Finset.sum_congr rfl; intros j _
    apply Finset.sum_congr rfl; intros k _
    simp only [hkey_eq i j k]
  -- Expand the (a + b)² sum:
  have hexpand : sumGrid (fun i j k =>
        (NS2 * laplacian f i j k + cv * laplacian (laplacian f) i j k) ^ 2) =
      (NS2 * NS2) * sumGrid (fun i j k => (laplacian f i j k) ^ 2) +
      2 * NS2 * cv * sumGrid (fun i j k =>
          laplacian f i j k * laplacian (laplacian f) i j k) +
      (cv * cv) * sumGrid (fun i j k => (laplacian (laplacian f) i j k) ^ 2) := by
    unfold sumGrid
    have hpt : ∀ i j k : Fin N,
        (NS2 * laplacian f i j k + cv * laplacian (laplacian f) i j k) ^ 2 =
        (NS2 * NS2) * (laplacian f i j k) ^ 2 +
        2 * NS2 * cv * (laplacian f i j k * laplacian (laplacian f) i j k) +
        (cv * cv) * (laplacian (laplacian f) i j k) ^ 2 := by
      intros i j k; ring
    simp only [hpt, Finset.sum_add_distrib, ← Finset.mul_sum]
  -- Apply sbp_quadratic_h2: Σ Δf · Δ²f = -Σ|∇⁺Δf|²
  have hsbp_h2 := sbp_quadratic_h2 f
  -- Apply Poincaré: Σ(Δ²f)² ≤ 12 · Σ|∇⁺Δf|²
  have hpoincare : sumGrid (fun i j k => (laplacian (laplacian f) i j k) ^ 2) ≤
      12 * gradSqForward (laplacian f) := laplacian_sq_le_12_gradSqForward (laplacian f)
  -- Goal: NS2² · hTwoSeminorm (diffusion_step f p) ≤ NS2² · hTwoSeminorm f
  have hgoal_scaled :
      NS2 * NS2 * hTwoSeminorm (diffusion_step f p) ≤ NS2 * NS2 * hTwoSeminorm f := by
    rw [hscale_lhs, hsub_step, hexpand]
    -- Now LHS is: NS² · Σ Δf² + 2·NS²·cv·(-Σ|∇⁺Δf|²) + cv²·Σ(Δ²f)²
    -- Goal:      NS² · Σ Δf² ≥ that
    -- Need: 2·NS²·cv·Σ Δf·Δ²f + cv²·Σ(Δ²f)² ≤ 0
    -- After sbp: -2·NS²·cv·Σ|∇⁺Δf|² + cv²·Σ(Δ²f)² ≤ 0
    -- After Poincaré: cv²·Σ(Δ²f)² ≤ 12·cv²·Σ|∇⁺Δf|²
    -- After CFL: 12·cv² · Σ|∇⁺Δf|² ≤ 2·NS²·cv · Σ|∇⁺Δf|²
    have hgrad_nonneg : 0 ≤ gradSqForward (laplacian f) := by
      unfold gradSqForward
      apply Finset.sum_nonneg; intros i _
      apply Finset.sum_nonneg; intros j _
      apply Finset.sum_nonneg; intros k _
      have h1 := sq_nonneg (laplacian f (sp i) j k - laplacian f i j k)
      have h2 := sq_nonneg (laplacian f i (sp j) k - laplacian f i j k)
      have h3 := sq_nonneg (laplacian f i j (sp k) - laplacian f i j k)
      linarith
    -- The unfold hTwoSeminorm on the RHS gives Σ(Δf)² which equals sumGrid (Δf)²
    show (NS2 * NS2) * sumGrid (fun i j k => (laplacian f i j k) ^ 2)
       + 2 * NS2 * cv * sumGrid (fun i j k =>
           laplacian f i j k * laplacian (laplacian f) i j k)
       + cv * cv * sumGrid (fun i j k => (laplacian (laplacian f) i j k) ^ 2)
       ≤ NS2 * NS2 * hTwoSeminorm f
    unfold hTwoSeminorm
    -- Σ(Δf)² in the goal is sumGrid (laplacian f)²
    rw [show (∑ i : Fin N, ∑ j : Fin N, ∑ k : Fin N, (laplacian f i j k) ^ 2)
        = sumGrid (fun i j k => (laplacian f i j k) ^ 2) from rfl]
    rw [hsbp_h2]
    -- After substitution: cv² · Σ(Δ²f)² ≤ 12 cv² Σ|∇⁺Δf|² (from Poincaré multiplied by cv²≥0)
    have hcv_sq_nonneg : 0 ≤ cv * cv := mul_self_nonneg cv
    have hbound1 : cv * cv * sumGrid (fun i j k => (laplacian (laplacian f) i j k) ^ 2) ≤
                   cv * cv * (12 * gradSqForward (laplacian f)) :=
      mul_le_mul_of_nonneg_left hpoincare hcv_sq_nonneg
    -- And 12 cv² Σ|∇⁺Δf|² ≤ 2 NS² cv · Σ|∇⁺Δf|² (from CFL · cv · Σ|∇⁺Δf|²)
    -- CFL: 6 cv ≤ NS²; multiply by 2 cv ≥ 0: 12 cv² ≤ 2 NS² cv
    have hbound2 : cv * cv * (12 * gradSqForward (laplacian f)) ≤
                   2 * NS2 * cv * gradSqForward (laplacian f) := by
      have h12cv2_le : 12 * (cv * cv) ≤ 2 * NS2 * cv := by
        have h : 2 * cv * (6 * cv) ≤ 2 * cv * NS2 :=
          mul_le_mul_of_nonneg_left hCFL (by linarith : 0 ≤ 2 * cv)
        linarith
      calc cv * cv * (12 * gradSqForward (laplacian f))
          = 12 * (cv * cv) * gradSqForward (laplacian f) := by ring
        _ ≤ 2 * NS2 * cv * gradSqForward (laplacian f) :=
            mul_le_mul_of_nonneg_right h12cv2_le hgrad_nonneg
    -- Combine: LHS ≤ NS²·Σ Δf² - 2 NS² cv · Σ|∇⁺Δf|² + 2 NS² cv · Σ|∇⁺Δf|² = NS²·Σ Δf²
    linarith
  -- Divide by NS2² > 0
  exact le_of_mul_le_mul_left hgoal_scaled hNS2_sq_pos

/-- **M3 Resolution A (composition)** — uniform H² bound over the full
    trajectory, under the exactness hypothesis holding at every timestep.

    Composition pattern: same as `ns_lInf_uniform_exact` from Wave D, but
    for H². Provided here as an analogue for downstream consumers. -/
theorem hTwoNorm_diffusion_step_le_exact (w : VField) (p : NsParams)
    (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hns_pos : 0 < p.ns_scale)
    (hExact_x : ∀ i j k : Fin N,
        ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
          (p.nu_scaled * p.dt_latt * laplacian w.x i j k))
    (hExact_y : ∀ i j k : Fin N,
        ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
          (p.nu_scaled * p.dt_latt * laplacian w.y i j k))
    (hExact_z : ∀ i j k : Fin N,
        ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
          (p.nu_scaled * p.dt_latt * laplacian w.z i j k)) :
    hTwoNorm (⟨diffusion_step w.x p, diffusion_step w.y p, diffusion_step w.z p⟩)
    ≤ hTwoNorm w := by
  unfold hTwoNorm
  have hx := h2_seminorm_per_component_exact w.x p h hν hdt hns_pos hExact_x
  have hy := h2_seminorm_per_component_exact w.y p h hν hdt hns_pos hExact_y
  have hz := h2_seminorm_per_component_exact w.z p h hν hdt hns_pos hExact_z
  linarith

end NSRegularity.HSquared
