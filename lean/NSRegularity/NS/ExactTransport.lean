import NSRegularity.NS.CurlPreimage

/-! Integer numerators for periodic diffusion at a common positive scale, and
    finite energy algebra for a backward Euler velocity step. -/
namespace NSRegularity.NS.ExactTransport

variable {G : Type*} [AddCommGroup G]

structure VectorField (G : Type*) where
  x : G → ℤ
  y : G → ℤ
  z : G → ℤ

/-- The six neighbours have weight c. Divide the physical amplitude by
    the new scale d*A after this numerator update. -/
def heat (x y z : G) (c d : ℤ) (f : G → ℤ) (p : G) : ℤ :=
  (d-6*c)*f p + c*(f (p+x) + f (p+ -x) + f (p+y) + f (p+ -y) + f (p+z) + f (p+ -z))

def diff (e : G) (f : G → ℤ) (p : G) := f (p+e) - f (p+ -e)

def curlG (x y z : G) (u : VectorField G) : VectorField G :=
  ⟨diff y u.z - diff z u.y, diff z u.x - diff x u.z, diff x u.y - diff y u.x⟩

def divG (x y z : G) (u : VectorField G) : G → ℤ := diff x u.x + diff y u.y + diff z u.z

def heatV (x y z : G) (c d : ℤ) (u : VectorField G) : VectorField G :=
  ⟨heat x y z c d u.x, heat x y z c d u.y, heat x y z c d u.z⟩

theorem diff_heat (x y z e : G) (c d : ℤ) (f : G → ℤ) :
    diff e (heat x y z c d f) = heat x y z c d (diff e f) := by
  funext p
  simp only [diff, heat, add_right_comm p e, add_right_comm p (-e)]
  ring

private theorem heat_sub (x y z : G) (c d : ℤ) (f g : G → ℤ) :
    heat x y z c d (f-g) = heat x y z c d f - heat x y z c d g := by
  funext p
  simp only [heat, Pi.sub_apply]
  ring

private theorem heat_add (x y z : G) (c d : ℤ) (f g : G → ℤ) :
    heat x y z c d (f+g) = heat x y z c d f + heat x y z c d g := by
  funext p
  simp only [heat, Pi.add_apply]
  ring

theorem curl_heat (x y z : G) (c d : ℤ) (u : VectorField G) :
    curlG x y z (heatV x y z c d u) = heatV x y z c d (curlG x y z u) := by
  simp only [curlG, heatV, diff_heat, heat_sub]

theorem div_heat (x y z : G) (c d : ℤ) (u : VectorField G) :
    divG x y z (heatV x y z c d u) = heat x y z c d (divG x y z u) := by
  simp only [divG, heatV, diff_heat, heat_add]

def Represents (x y z : G) (w u : VectorField G) := curlG x y z u = w ∧ divG x y z u = 0

theorem represents_step (x y z : G) (c d : ℤ) {w u : VectorField G} (h : Represents x y z w u) :
    Represents x y z (heatV x y z c d w) (heatV x y z c d u) := by
  obtain ⟨hw, hu⟩ := h
  constructor
  · rw [curl_heat, hw]
  · rw [div_heat, hu]
    funext p
    simp [heat]

theorem represents_iterate (x y z : G) (c d : ℤ) {w u : VectorField G}
    (h : Represents x y z w u) (k : ℕ) :
    Represents x y z ((heatV x y z c d)^[k] w) ((heatV x y z c d)^[k] u) := by
  induction k with
  | zero => exact h
  | succ k ih =>
    simpa only [Function.iterate_succ_apply'] using represents_step x y z c d ih

theorem heat_interval (x y z : G) (c d lo hi : ℤ) (hc : 0 ≤ c) (hcfl : 6*c ≤ d)
    {f : G → ℤ} (h : ∀ p, lo ≤ f p ∧ f p ≤ hi) (p : G) :
    d*lo ≤ heat x y z c d f p ∧ heat x y z c d f p ≤ d*hi := by
  have hn : 6*lo ≤ f (p+x)+f (p+ -x)+f (p+y)+f (p+ -y)+f (p+z)+f (p+ -z) ∧
      f (p+x)+f (p+ -x)+f (p+y)+f (p+ -y)+f (p+z)+f (p+ -z) ≤ 6*hi := by
    have := h (p+x); have := h (p+ -x); have := h (p+y)
    have := h (p+ -y); have := h (p+z); have := h (p+ -z); omega
  unfold heat
  constructor
  · nlinarith only [mul_nonneg (sub_nonneg.mpr hcfl) (sub_nonneg.mpr (h p).1),
      mul_le_mul_of_nonneg_left hn.1 hc]
  · nlinarith only [mul_nonneg (sub_nonneg.mpr hcfl) (sub_nonneg.mpr (h p).2),
      mul_le_mul_of_nonneg_left hn.2 hc]

/-- Finite rational backward Euler energy identity once the step has been
    tested against its new velocity. The PDE estimates theorems are separate. -/
theorem backward_energy {ι : Type*} [Fintype ι] (a b : ι → ℚ) (tau nu diss : ℚ)
    (balance : (∑ i, (b i-a i)*b i) + tau*nu*diss = 0) :
    (∑ i, (b i)^2) + (∑ i, (b i-a i)^2) + 2*tau*nu*diss = ∑ i, (a i)^2 := by
  have ids : (∑ i, (b i)^2) + (∑ i, (b i-a i)^2) - (∑ i, (a i)^2) =
      2 * (∑ i, (b i-a i)*b i) := by
    rw [← Finset.sum_add_distrib, ← Finset.sum_sub_distrib, Finset.mul_sum]
    apply Finset.sum_congr rfl
    intro i _
    ring
  linarith


/-- Returning to the old integer scale recovers the original floor operation. -/
theorem floor_observation (x y z : G) (c d : ℤ) (hd : 0 < d) (f : G → ℤ) (p : G) :
    heat x y z c d f p / d = f p + c*(f (p+x)+f (p+ -x)+f (p+y)+f (p+ -y)+
      f (p+z)+f (p+ -z)-6*f p)/d := by
  unfold heat
  rw [← Int.mul_add_ediv_right _ _ (ne_of_gt hd)]
  congr 1
  ring

abbrev Site8 := Fin 8 × Fin 8 × Fin 8
private def ex : Site8 := (1,0,0)
private def ey : Site8 := (0,1,0)
private def ez : Site8 := (0,0,1)
private def at8 (f : NSRegularity.Grid) (p : Site8) := f p.1 p.2.1 p.2.2
private def vec8 (u : NSRegularity.VField) : VectorField Site8 := ⟨at8 u.x, at8 u.y, at8 u.z⟩

private theorem prev8 (i : Fin 8) : i + -(1 : Fin 8) = NSRegularity.sn i := by
  fin_cases i <;> decide

private theorem dx8 (f : NSRegularity.Grid) : diff ex (at8 f) = at8 (NSRegularity.Kill113.diffX f) := by
  funext ⟨i,j,k⟩
  simp [ex, at8, diff, NSRegularity.Kill113.diffX, NSRegularity.sp, prev8]

private theorem dy8 (f : NSRegularity.Grid) : diff ey (at8 f) = at8 (NSRegularity.Kill113.diffY f) := by
  funext ⟨i,j,k⟩
  simp [ey, at8, diff, NSRegularity.Kill113.diffY, NSRegularity.sp, prev8]

private theorem dz8 (f : NSRegularity.Grid) : diff ez (at8 f) = at8 (NSRegularity.Kill113.diffZ f) := by
  funext ⟨i,j,k⟩
  simp [ez, at8, diff, NSRegularity.Kill113.diffZ, NSRegularity.sp, prev8]

theorem agrees_curl_eight (u : NSRegularity.VField) : curlG ex ey ez (vec8 u) = vec8 (curl u) := by
  unfold curlG vec8
  rw [dy8, dz8, dz8, dx8, dx8, dy8]
  rfl

theorem agrees_div_eight (u : NSRegularity.VField) : divG ex ey ez (vec8 u) = at8 (divV u) := by
  simp only [divG, vec8, dx8, dy8, dz8]
  rfl

/-- The weak transport form on an oriented graph, with arithmetic face averages. -/
def faceForm {E V : Type*} [Fintype E] (src dst : E → V) (flux : E → ℚ)
    (b v : V → ℚ) : ℚ :=
  ∑ e, flux e * ((b (src e)+b (dst e))/2) * (v (src e)-v (dst e))

/-- Equal outgoing and incoming face flux at every vertex cancels transport work. -/
theorem face_energy_zero {E V : Type*} [Fintype E] [Fintype V] [DecidableEq V]
    (src dst : E → V) (flux : E → ℚ) (b : V → ℚ)
    (hdiv : ∀ i, (∑ e, if src e = i then flux e else 0) = ∑ e, if dst e = i then flux e else 0) :
    faceForm src dst flux b b = 0 := by
  classical
  have push (ends : E → V) : (∑ e, flux e * (b (ends e))^2) =
      ∑ i, (∑ e, if ends e = i then flux e else 0) * (b i)^2 := by
    simp_rw [Finset.sum_mul]
    rw [Finset.sum_comm]
    apply Finset.sum_congr rfl
    intro e _
    simp [ite_mul]
  have eq : (∑ e, flux e * (b (src e))^2) = ∑ e, flux e * (b (dst e))^2 := by
    rw [push src, push dst]
    simp_rw [hdiv]
  have twice : 2 * faceForm src dst flux b b =
      (∑ e, flux e*(b (src e))^2) - ∑ e, flux e*(b (dst e))^2 := by
    rw [faceForm, Finset.mul_sum, ← Finset.sum_sub_distrib]
    apply Finset.sum_congr rfl
    intro e _
    ring
  linarith


/-- A single projection of the shared-scale heat step is the production N8 floor stencil. -/
theorem agrees_diffusion_eight (f : NSRegularity.Grid) (q : NsParams)
    (hd : 0 < (NSRegularity.N : ℤ)^2*q.ns_scale^2) (i j k : Fin 8) :
    heat ex ey ez (q.nu_scaled*q.dt_latt) ((NSRegularity.N : ℤ)^2*q.ns_scale^2)
      (at8 f) (i,j,k) / ((NSRegularity.N : ℤ)^2*q.ns_scale^2) = diffusion_step f q i j k := by
  rw [floor_observation _ _ _ _ _ hd]
  simp [ex, ey, ez, at8, diffusion_step, laplacian, NSRegularity.sp, prev8]

/-- Arithmetic face averages have exactly the centered cell divergence. -/
theorem face_divergence {J : Type*} [Fintype J] (left center right : J → ℚ) :
    (∑ j, ((center j+right j)/2 - (left j+center j)/2)) = (∑ j, (right j-left j))/2 := by
  rw [Finset.sum_div]
  apply Finset.sum_congr rfl
  intro j _
  ring

theorem conservative_step_energy {E V : Type*} [Fintype E] [Fintype V] [DecidableEq V]
    (src dst : E → V) (flux : E → ℚ) (a b : V → ℚ) (tau nu diss : ℚ)
    (hdiv : ∀ i, (∑ e, if src e = i then flux e else 0) = ∑ e, if dst e = i then flux e else 0)
    (tested : (∑ i, (b i-a i)*b i) + tau*faceForm src dst flux b b + tau*nu*diss = 0) :
    (∑ i, (b i)^2) + (∑ i, (b i-a i)^2) + 2*tau*nu*diss = ∑ i, (a i)^2 := by
  apply backward_energy
  simpa only [face_energy_zero src dst flux b hdiv, mul_zero, add_zero] using tested

/-- Strict positivity of the diagonal of the projected rational step implies
    unique solvability on the finite-dimensional solenoidal space. -/
theorem unique_rational_step {Z : Type*} [AddCommGroup Z] [Module ℚ Z] [FiniteDimensional ℚ Z]
    (L : Z →ₗ[ℚ] Z) (pair : Z → Z → ℚ) (hz : ∀ z, pair 0 z = 0)
    (hpos : ∀ z, z ≠ 0 → 0 < pair (L z) z) (a : Z) : ∃! b, L b = a := by
  have inj : Function.Injective L := by
    intro x y h
    have kernel : L (x-y) = 0 := by rw [map_sub, h, sub_self]
    by_contra hne
    have hpositive := hpos (x-y) (sub_ne_zero.mpr hne)
    rw [kernel, hz] at hpositive
    exact (lt_irrefl _ hpositive)
  obtain ⟨b,hb⟩ := (LinearMap.injective_iff_surjective.mp inj) a
  exact ⟨b,hb,fun z h => inj (h.trans hb.symm)⟩

/-- The squared finite Loomis–Whitney sum used in the uniform grid Sobolev bound. -/
theorem three_plane_sum {I J K : Type*} [Fintype I] [Fintype J] [Fintype K]
    (F : J → K → ℝ) (G₁ : I → K → ℝ) (H : I × J → ℝ) :
    (∑ p : I × J, H p * (∑ k, F p.2 k * G₁ p.1 k))^2 ≤
      (∑ p, (H p)^2) * ((∑ j, ∑ k, (F j k)^2) * (∑ i, ∑ k, (G₁ i k)^2)) := by
  have inner (p : I × J) := Finset.sum_mul_sq_le_sq_mul_sq Finset.univ (F p.2) (G₁ p.1)
  have outer := Finset.sum_mul_sq_le_sq_mul_sq Finset.univ H
    (fun p : I × J => ∑ k, F p.2 k * G₁ p.1 k)
  refine outer.trans (mul_le_mul_of_nonneg_left ?_ (Finset.sum_nonneg (fun p _ => sq_nonneg (H p))))
  calc (∑ p : I × J, (∑ k, F p.2 k * G₁ p.1 k)^2) ≤
        ∑ p : I × J, (∑ k, (F p.2 k)^2) * (∑ k, (G₁ p.1 k)^2) :=
          Finset.sum_le_sum (fun p _ => inner p)
       _ = (∑ j, ∑ k, (F j k)^2) * (∑ i, ∑ k, (G₁ i k)^2) := by
         rw [Fintype.sum_prod_type, Finset.mul_sum]
         apply Finset.sum_congr rfl
         intro i _
         rw [Finset.sum_mul]

/-- Pointwise chain estimate, applied to the absolute values of neighbouring samples. -/
theorem fourth_power_edge (a b : ℝ) (ha : 0 ≤ a) (hb : 0 ≤ b) :
    |a^4-b^4| ≤ 2*(a^3+b^3)*|a-b| := by
  have fac : a^4-b^4 = (a-b)*(a^3+a^2*b+a*b^2+b^3) := by ring
  have pos : 0 ≤ a^3+a^2*b+a*b^2+b^3 := by positivity
  rw [fac, abs_mul, abs_of_nonneg pos, mul_comm (2*(a^3+b^3))]
  apply mul_le_mul_of_nonneg_left _ (abs_nonneg _)
  nlinarith only [mul_nonneg (sq_nonneg (a-b)) (add_nonneg ha hb)]

/-- Absorb nonlinear enstrophy work into positive viscosity. The written
    grid estimate supplies the constant 54 uniformly in the period. -/
theorem absorb_enstrophy (oldE newE jump Y work X nu tau : ℝ)
    (ht : 0 ≤ tau) (hY : 0 ≤ Y) (hsmall : 108*X ≤ nu)
    (hw : -work ≤ 54*X*Y)
    (balance : newE - oldE + jump + 2*tau*nu*Y = -2*tau*work) :
    newE + jump + tau*nu*Y ≤ oldE := by
  have absorbed : -2*work ≤ nu*Y := by
    nlinarith only [hw, mul_le_mul_of_nonneg_right hsmall hY]
  nlinarith only [balance, mul_le_mul_of_nonneg_left absorbed ht]

/-- Testing a perturbed implicit step against its error gives a one-step
    stability estimate. All norms in this lemma are nonnegative scalars. -/
theorem error_from_tested_balance (previous current tau forcing : ℝ)
    (hprev : 0 ≤ previous) (hcur : 0 ≤ current) (ht : 0 ≤ tau) (hf : 0 ≤ forcing)
    (tested : current^2 - previous*current ≤ tau*forcing*current) :
    current ≤ previous + tau*forcing := by
  rcases eq_or_lt_of_le hcur with hzero | hpos
  · rw [← hzero]
    positivity
  · have h : (current-(previous+tau*forcing))*current ≤ 0 := by nlinarith only [tested]
    linarith only [(nonpos_of_mul_nonpos_left h hpos)]

/-- Accumulating forcing or residuals over a finite time interval. -/
theorem residual_growth (err residual : ℕ → ℝ) (factor tau : ℝ) (hf : 0 ≤ factor)
    (step : ∀ n, err (n+1) ≤ factor*err n + tau*residual n) (n : ℕ) :
    err n ≤ factor^n*err 0 + ∑ j ∈ Finset.range n, factor^(n-1-j)*(tau*residual j) := by
  induction n with
  | zero => simp
  | succ n ih =>
    have hs : factor * (∑ j ∈ Finset.range n, factor^(n-1-j)*(tau*residual j)) =
        ∑ j ∈ Finset.range n, factor^(n-j)*(tau*residual j) := by
      rw [Finset.mul_sum]
      apply Finset.sum_congr rfl
      intro j hj
      have hex : n-j = n-1-j+1 := by have := Finset.mem_range.mp hj; omega
      rw [hex, pow_succ']
      ring
    refine (step n).trans ((add_le_add (mul_le_mul_of_nonneg_left ih hf) le_rfl).trans_eq ?_)
    simp only [Finset.sum_range_succ, Nat.add_sub_cancel, Nat.sub_self,
      pow_zero, one_mul]
    rw [mul_add, hs, pow_succ']
    ring

/-- Improved small-data absorption after estimating the work by 9/4 of
    the product of the advector gradient and the squared Laplacian norm. -/
theorem absorb_enstrophy_refined (oldE newE jump Y work X nu tau : ℝ)
    (ht : 0 ≤ tau) (hY : 0 ≤ Y) (hsmall : 9*X ≤ 2*nu)
    (hw : -work ≤ (9/4)*X*Y)
    (balance : newE - oldE + jump + 2*tau*nu*Y = -2*tau*work) :
    newE + jump + tau*nu*Y ≤ oldE := by
  have absorbed : -2*work ≤ nu*Y := by
    nlinarith only [hw, mul_le_mul_of_nonneg_right hsmall hY]
  nlinarith only [balance, mul_le_mul_of_nonneg_left absorbed ht]

end NSRegularity.NS.ExactTransport
