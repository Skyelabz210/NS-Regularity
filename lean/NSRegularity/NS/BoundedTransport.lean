import NSRegularity.NS.ExactTransport

/-! A bounded-scale solenoidal encoding and its residue window. -/
namespace NSRegularity.NS.BoundedTransport

open ExactTransport

variable {G : Type*} [AddCommGroup G]

/-- An integer potential plus a field constant on each two-step orbit has
    exactly zero centered divergence. -/
def encode (x y z : G) (potential harmonic : VectorField G) : VectorField G :=
  let c := curlG x y z potential
  ⟨c.x + harmonic.x, c.y + harmonic.y, c.z + harmonic.z⟩

theorem encode_divergence (x y z : G) (potential harmonic : VectorField G)
    (hx : ∀ p, harmonic.x (p+x) = harmonic.x (p+ -x))
    (hy : ∀ p, harmonic.y (p+y) = harmonic.y (p+ -y))
    (hz : ∀ p, harmonic.z (p+z) = harmonic.z (p+ -z)) :
    divG x y z (encode x y z potential harmonic) = 0 := by
  funext p
  simp only [divG, encode, curlG, diff, Pi.add_apply, Pi.sub_apply, Pi.zero_apply,
    hx, hy, hz, add_right_comm p y x, add_right_comm p z x,
    add_right_comm p y (-x), add_right_comm p z (-x), add_right_comm p (-y) x,
    add_right_comm p (-z) x, add_right_comm p (-y) (-x), add_right_comm p (-z) (-x),
    add_right_comm p z y, add_right_comm p z (-y), add_right_comm p (-z) y,
    add_right_comm p (-z) (-y)]
  ring

/-- A curl component uses four signed samples of the rounded potential. -/
theorem potential_error_component (a b c d step : ℚ)
    (ha : |a| ≤ step/2) (hb : |b| ≤ step/2) (hc : |c| ≤ step/2) (hd : |d| ≤ step/2) :
    |a-b-c+d| ≤ 2*step := by
  obtain ⟨ha₁,ha₂⟩ := abs_le.mp ha
  obtain ⟨hb₁,hb₂⟩ := abs_le.mp hb
  obtain ⟨hc₁,hc₂⟩ := abs_le.mp hc
  obtain ⟨hd₁,hd₂⟩ := abs_le.mp hd
  exact abs_le.mpr ⟨by linarith,by linarith⟩

/-- Once modular evidence combines into a sufficiently wide modulus, the
    signed velocity numerator is uniquely determined. -/
theorem signed_residue_unique (a b lo hi modulus : ℤ)
    (ha : lo ≤ a ∧ a ≤ hi) (hb : lo ≤ b ∧ b ≤ hi)
    (hwide : hi-lo < modulus) (hres : a ≡ b [ZMOD modulus]) : a = b := by
  have hpos : 0 < modulus := by omega
  rcases le_total a b with hab | hba
  · have hz := Int.eq_zero_of_dvd_of_nonneg_of_lt (sub_nonneg.mpr hab)
      (show b-a < modulus by omega) (Int.modEq_iff_dvd.mp hres)
    omega
  · have hz := Int.eq_zero_of_dvd_of_nonneg_of_lt (sub_nonneg.mpr hba)
      (show a-b < modulus by omega) (Int.modEq_iff_dvd.mp hres.symm)
    omega

/-- The finite energy budget includes the cost of a bounded per-step state
    compression, expressed using a common upper bound on both norms. -/
theorem compression_energy (old intermediate stored diss loss upper : ℝ)
    (hmid : 0 ≤ intermediate) (hstore : 0 ≤ stored) (hmidmax : intermediate ≤ upper)
    (hstoremax : stored ≤ upper) (hloss : 0 ≤ loss) (hclose : stored - intermediate ≤ loss)
    (hstep : intermediate^2 + diss ≤ old^2) : stored^2 + diss ≤ old^2 + 2*upper*loss := by
  have h := mul_le_mul hclose (show stored + intermediate ≤ 2*upper by linarith)
    (add_nonneg hstore hmid) hloss
  nlinarith only [h,hstep]

end NSRegularity.NS.BoundedTransport
