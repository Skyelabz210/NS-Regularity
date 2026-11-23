import NSRegularity.DKAM.DiffusionMaxPreserving

/-! Periodic scalar diffusion on varying resolutions. The two coefficients
    are the integer numerator and denominator of the neighbour weight. -/
namespace NSRegularity.NS.RefinedDiffusion

open NSRegularity NSRegularity.NS

abbrev Field (n : ℕ) := Fin n → Fin n → Fin n → Int

def neighbours {n : ℕ} [NeZero n] (f : Field n) (i j k : Fin n) : Int :=
  let back : Fin n := Fin.ofNat n (n-1)
  f (i+1) j k + f (i+back) j k + f i (j+1) k + f i (j+back) k +
    f i j (k+1) + f i j (k+back)

def update {n : ℕ} [NeZero n] (f : Field n) (c d : Int) : Field n :=
  fun i j k => f i j k + c*(neighbours f i j k - 6*f i j k)/d

theorem agrees_at_eight (f : Grid) (p : NsParams) :
    update f (p.nu_scaled*p.dt_latt) ((N : Int)^2*p.ns_scale^2) = diffusion_step f p := rfl

/-- The same integer endpoints work at every resolution and for every step. -/
theorem preserves_interval {n : ℕ} [NeZero n] (f : Field n) (c d lo hi : Int)
    (hd : 0 < d) (hc : 0 ≤ c) (hcfl : 6*c ≤ d)
    (hb : ∀ i j k, lo ≤ f i j k ∧ f i j k ≤ hi) (i j k : Fin n) :
    lo ≤ update f c d i j k ∧ update f c d i j k ≤ hi := by
  apply DKAM.Diffusion.rounded_stencil_interval _ _ _ _ _ _ hd hc hcfl (hb i j k)
  unfold neighbours
  have := hb (i+1) j k
  have := hb (i+Fin.ofNat n (n-1)) j k
  have := hb i (j+1) k
  have := hb i (j+Fin.ofNat n (n-1)) k
  have := hb i j (k+1)
  have := hb i j (k+Fin.ofNat n (n-1))
  constructor <;> omega

/-- Comparison with a constant offset, uniform in the period. Offsets here
    are integer field units; adding an integer commutes with the floor. -/
theorem comparison {n : ℕ} [NeZero n] (f g : Field n) (c d e : Int)
    (hd : 0 < d) (hc : 0 ≤ c) (hcfl : 6*c ≤ d)
    (hb : ∀ i j k, f i j k ≤ g i j k + e) (i j k : Fin n) :
    update f c d i j k ≤ update g c d i j k + e := by
  let num := fun v : Field n => v i j k * d + c*(neighbours v i j k-6*v i j k)
  have hu (v : Field n) : update v c d i j k = num v / d :=
    (Int.mul_add_ediv_right _ _ (ne_of_gt hd)).symm
  have hn : neighbours f i j k ≤ neighbours g i j k + 6*e := by
    unfold neighbours
    have := hb (i+1) j k
    have := hb (i+Fin.ofNat n (n-1)) j k
    have := hb i (j+1) k
    have := hb i (j+Fin.ofNat n (n-1)) k
    have := hb i j (k+1)
    have := hb i j (k+Fin.ofNat n (n-1))
    omega
  have hnum : num f ≤ e*d + num g := by
    dsimp [num]
    nlinarith only [mul_nonneg (sub_nonneg.mpr hcfl) (sub_nonneg.mpr (hb i j k)),
      mul_nonneg hc (sub_nonneg.mpr hn)]
  rw [hu f, hu g]
  calc num f / d ≤ (e*d + num g) / d := Int.ediv_le_ediv hd hnum
       _ = num g / d + e := by rw [Int.mul_add_ediv_right _ _ (ne_of_gt hd), add_comm]

/-- Comparison against an arbitrary sampled reference. The local defect is
    evaluated on the reference, independently of the unknown solution error. -/
theorem error_from_local_residual {n : ℕ} [NeZero n] (f ref : ℕ → Field n)
    (c d initial residual : Int) (hd : 0 < d) (hc : 0 ≤ c) (hcfl : 6*c ≤ d)
    (hstep : ∀ t, f (t+1) = update (f t) c d)
    (hinit : ∀ i j k, |f 0 i j k - ref 0 i j k| ≤ initial)
    (hres : ∀ t i j k, |update (ref t) c d i j k - ref (t+1) i j k| ≤ residual)
    (t : ℕ) : ∀ i j k, |f t i j k - ref t i j k| ≤ initial + (t : Int)*residual := by
  induction t with
  | zero => simpa using hinit
  | succ t ih =>
    let e := initial + (t : Int)*residual
    have hfg (i j k : Fin n) : f t i j k ≤ ref t i j k + e := by
      have := abs_le.mp (ih i j k); dsimp [e]; omega
    have hgf (i j k : Fin n) : ref t i j k ≤ f t i j k + e := by
      have := abs_le.mp (ih i j k); dsimp [e]; omega
    intro i j k
    have h₁ := comparison (f t) (ref t) c d e hd hc hcfl hfg i j k
    have h₂ := comparison (ref t) (f t) c d e hd hc hcfl hgf i j k
    have hr := abs_le.mp (hres t i j k)
    rw [hstep t]
    simp only [Nat.cast_succ, add_mul, one_mul]
    dsimp [e] at h₁ h₂
    apply abs_le.mpr
    omega

/-- In units of one quantization step, two positive amplitudes below one
    have identical floor observations on every sample of a signed profile. -/
theorem subunit_amplitudes_same_reading (a b z : ℝ) (ha : 0 < a ∧ a < 1)
    (hb : 0 < b ∧ b < 1) (hz : -1 ≤ z ∧ z ≤ 1) :
    ⌊a*z⌋ = ⌊b*z⌋ := by
  have read (x : ℝ) (hx : 0 < x ∧ x < 1) :
      ⌊x*z⌋ = (if 0 ≤ z then 0 else -1 : Int) := by
    split
    next h =>
      apply Int.floor_eq_iff.mpr
      have h1 := mul_le_mul_of_nonneg_left hz.2 hx.1.le
      have h0 := mul_nonneg hx.1.le h
      norm_num at h1 ⊢
      exact ⟨h0, h1.trans_lt hx.2⟩
    next h =>
      apply Int.floor_eq_iff.mpr
      have h1 := mul_le_mul_of_nonneg_left hz.1 hx.1.le
      have h0 := mul_neg_of_pos_of_neg hx.1 (lt_of_not_ge h)
      norm_num at h1 ⊢
      exact ⟨by linarith, h0⟩
  rw [read a ha, read b hb]

/-- Same signed N32 shear, viscosity, time parameter and amplitude scale
    as the Rust regression. The positive unit peak falls to zero here. -/
theorem signed_shear_at_32 :
    let shear : Field 32 := fun _ j _ => if j.val = 31 then 1 else if j.val = 1 then -1 else 0
    update shear 1 ((32 : Int)^2 * 10000^2) 0 31 0 = 0 := by decide

end NSRegularity.NS.RefinedDiffusion
