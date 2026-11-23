import NSRegularity.NS.Trajectory
import NSRegularity.NS.StretchingBound

/-! Bounds and observables for the actual integer split step. -/

namespace NSRegularity.NS.RoundedStep

open NSRegularity NSRegularity.NS NSRegularity.NS.StretchingBound

/-- The signed contribution left after diffusion. The advection numerator
    uses the diffused scalar field, as in the definition of the split step. -/
def increment (u : VField) (stretch f : Grid) (p : NsParams) : Grid :=
  fun i j k =>
    p.dt_latt * stretch i j k / (2 * (N : Int) * p.ns_scale) -
    p.dt_latt * Kill113.advective u (diffusion_step f p) i j k /
      (2 * (N : Int) * p.ns_scale)

def incrementField (u : VField) (sx sy sz : Grid) (w : VField) (p : NsParams) : VField :=
  ⟨increment u sx w.x p, increment u sy w.y p, increment u sz w.z p⟩

private theorem component_bound (u : VField) (stretch f : Grid) (p : NsParams)
    (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (ht : 0 ≤ p.dt_latt) :
    gridLInf (ns_step_component u stretch f p) ≤ gridLInf f + gridLInf (increment u stretch f p) := by
  conv_lhs => unfold gridLInf
  apply Finset.sup'_le; intro i _
  apply Finset.sup'_le; intro j _
  apply Finset.sup'_le; intro k _
  have he : ns_step_component u stretch f p i j k =
      diffusion_step f p i j k + increment u stretch f p i j k := by
    unfold ns_step_component stretching_step advection_step increment
    ring
  have hdiff := DKAM.Diffusion.diffusion_max_preserving f p h hν ht i j k
  rw [← Int.abs_eq_natAbs] at hdiff ⊢
  rw [he]
  exact (abs_add_le _ _).trans
    (add_le_add hdiff (DKAM.Diffusion.abs_le_gridLInf _ i j k))

/-- Full split step, including both nonlinear terms. A growing maximum is
    allowed and its increase is charged to a signed, computable field. -/
theorem step_bound (u : VField) (sx sy sz : Grid) (w : VField) (p : NsParams)
    (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (ht : 0 ≤ p.dt_latt) :
    lInf (ns_step u sx sy sz w p) ≤ lInf w + lInf (incrementField u sx sy sz w p) := by
  let e := incrementField u sx sy sz w p
  apply max_le
  · exact (component_bound u sx w.x p h hν ht).trans
      (add_le_add (gridLInf_x_le_lInf w) (gridLInf_x_le_lInf e))
  · apply max_le
    · exact (component_bound u sy w.y p h hν ht).trans
        (add_le_add (gridLInf_y_le_lInf w) (gridLInf_y_le_lInf e))
    · exact (component_bound u sz w.z p h hν ht).trans
        (add_le_add (gridLInf_z_le_lInf w) (gridLInf_z_le_lInf e))

/-- Each term is evaluated on the reached state. No divisibility or
    full-step maximum principle is assumed. -/
theorem trajectory_bound (velocity : VField → VField) (sx sy sz : VField → Grid)
    (w₀ : VField) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (ht : 0 ≤ p.dt_latt) (n : ℕ) :
    let w := ns_trajectory_generator velocity sx sy sz w₀ p
    lInf (w n) ≤ lInf w₀ + ∑ t ∈ Finset.range n,
      lInf (incrementField (velocity (w t)) (sx (w t)) (sy (w t)) (sz (w t)) (w t) p) := by
  let w := ns_trajectory_generator velocity sx sy sz w₀ p
  let b := fun t => lInf (incrementField (velocity (w t)) (sx (w t)) (sy (w t))
    (sz (w t)) (w t) p)
  change lInf (w n) ≤ lInf w₀ + ∑ t ∈ Finset.range n, b t
  induction n with
  | zero => simp [w, ns_trajectory_generator]
  | succ n ih =>
    rw [Finset.sum_range_succ, ← add_assoc]
    exact (step_bound _ _ _ _ (w n) p h hν ht).trans (add_le_add_right ih _)

/-- Q times the change in total scalar mass is exactly minus the sum
    of the Euclidean remainders. Each summand is observable at its own site. -/
theorem diffusion_mass_defect (f : Grid) (p : NsParams) :
    let d := (N : Int)^2 * p.ns_scale^2
    let c := p.nu_scaled * p.dt_latt
    d * (sumGrid (diffusion_step f p) - sumGrid f) =
      -sumGrid (fun i j k => (c * laplacian f i j k) % d) := by
  let d := (N : Int)^2 * p.ns_scale^2
  let c := p.nu_scaled * p.dt_latt
  have local_id : (fun i j k => d * diffusion_step f p i j k - d * f i j k +
      (c * laplacian f i j k) % d) = (fun i j k => c * laplacian f i j k) := by
    funext i j k
    change d * (f i j k + (c * laplacian f i j k) / d) - d * f i j k + _ = _
    rw [mul_add]
    linarith only [Int.emod_add_ediv (c * laplacian f i j k) d]
  have hs := congrArg sumGrid local_id
  have hz := sumGrid_laplacian_eq_zero f
  simp only [sumGrid, Finset.sum_add_distrib, Finset.sum_sub_distrib, ← Finset.mul_sum] at hs
  unfold sumGrid at hz
  rw [hz, mul_zero] at hs
  change d * (sumGrid (diffusion_step f p) - sumGrid f) = _
  unfold sumGrid
  linarith only [hs]

/-- The universal x-component exactness in the historical capstone forces
    zero diffusion under its CFL and positivity assumptions. -/
theorem universal_exactness_forces_zero (p : NsParams) (hcfl : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (ht : 0 ≤ p.dt_latt) (hscale : 0 < p.ns_scale)
    (hx : ∀ w : VField, ∀ i j k : Fin N, ((N : Int)^2 * p.ns_scale^2) ∣
      (p.nu_scaled * p.dt_latt * laplacian w.x i j k)) :
    p.nu_scaled * p.dt_latt = 0 := by
  let w : VField := ⟨fun i j k => if i = 1 ∧ j = 0 ∧ k = 0 then 1 else 0,
    fun _ _ _ => 0, fun _ _ _ => 0⟩
  have hone : laplacian w.x 0 0 0 = 1 := by decide
  have hdvd := hx w 0 0 0
  rw [hone, mul_one] at hdvd
  have hnn := mul_nonneg hν ht
  by_contra hn
  have hp : 0 < p.nu_scaled * p.dt_latt := by omega
  have hle := Int.le_of_dvd hp hdvd
  unfold cfl_ok at hcfl
  nlinarith only [hp, hle, hcfl]

/-- One representable input with genuine stretching. Its centered curl
    has component maximum 64, and the y component at the origin reaches
    -256. A particular right-inverse velocity is supplied explicitly. -/
private def wave (j : Fin N) : Int :=
  if j.val % 4 = 1 then 32 else if j.val % 4 = 3 then -32 else 0

def layeredVelocity : VField := ⟨fun _ j _ => wave j, fun _ _ k => wave k, fun _ _ _ => 0⟩

def layeredParams : NsParams := ⟨1, 1, 1⟩

theorem layered_solenoidal : divV layeredVelocity = fun _ _ _ => 0 := by
  funext i j k
  simp [divV, layeredVelocity, Kill113.diffX, Kill113.diffY, Kill113.diffZ]

private theorem wave_difference_bound (j : Fin N) : |wave (sp j) - wave (sn j)| ≤ 64 := by
  fin_cases j <;> decide

theorem layered_initial_bound : lInf (curl layeredVelocity) ≤ 64 := by
  have grid_bound (f : Grid) (hb : ∀ i j k, |f i j k| ≤ 64) : gridLInf f ≤ 64 := by
    unfold gridLInf
    apply Finset.sup'_le; intro i _
    apply Finset.sup'_le; intro j _
    apply Finset.sup'_le; intro k _
    simpa only [Int.abs_eq_natAbs] using hb i j k
  apply max_le
  · apply grid_bound; intro i j k
    simpa [curl, curlX, layeredVelocity, Kill113.diffY, Kill113.diffZ] using wave_difference_bound k
  · apply max_le
    · apply grid_bound; intro i j k
      simp [curl, curlY, layeredVelocity, Kill113.diffX, Kill113.diffZ]
    · apply grid_bound; intro i j k
      simpa [curl, curlZ, layeredVelocity, Kill113.diffX, Kill113.diffY] using wave_difference_bound j

theorem layered_step_grows :
    let u := layeredVelocity
    let w := curl u
    let w' := ns_step u (Kill113.advective w u.x) (Kill113.advective w u.y)
      (Kill113.advective w u.z) w layeredParams
    cfl_ok layeredParams ∧ lInf w < lInf w' := by
  let u := layeredVelocity
  let w := curl u
  let w' := ns_step u (Kill113.advective w u.x) (Kill113.advective w u.y)
    (Kill113.advective w u.z) w layeredParams
  have sample : w'.y 0 0 0 = -256 := by decide
  have h := abs_y_le_lInf w' 0 0 0
  rw [sample] at h
  exact ⟨by decide, lt_of_le_of_lt layered_initial_bound (by change (64 : Int) < lInf w'; omega)⟩

end NSRegularity.NS.RoundedStep
