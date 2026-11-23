import NSRegularity.NS.ExactTransport

/-! Integer coefficient algebra for the two-jump stencil obtained from
    the first two positive-degree coefficients of the mock theta ω series. -/
namespace NSRegularity.NS.MockThetaCurl

open ExactTransport

variable {G : Type*} [AddCommGroup G]

/-- With τν/h² = p/b, the new physical scale is 16*b times the old scale.
    The six one-jump coefficients are 4*p, the six two-jump coefficients
    are 3*p, and the center coefficient is 16*b-42*p. -/
def omegaHeat (x y z : G) (p b : ℤ) (f : G → ℤ) : G → ℤ :=
  heat x y z (4*p) (16*b-18*p) f + heat (x+x) (y+y) (z+z) (3*p) (18*p) f

def omegaHeatV (x y z : G) (p b : ℤ) (v : VectorField G) : VectorField G :=
  ⟨omegaHeat x y z p b v.x, omegaHeat x y z p b v.y, omegaHeat x y z p b v.z⟩

private theorem diff_add (d : G) (f g : G → ℤ) :
    diff d (f+g) = diff d f + diff d g := by
  funext t
  simp only [diff, Pi.add_apply]
  ring

private theorem diff_omega (x y z d : G) (p b : ℤ) (f : G → ℤ) :
    diff d (omegaHeat x y z p b f) = omegaHeat x y z p b (diff d f) := by
  simp only [omegaHeat, diff_add, diff_heat]

private theorem omega_add (x y z : G) (p b : ℤ) (f g : G → ℤ) :
    omegaHeat x y z p b (f+g) = omegaHeat x y z p b f + omegaHeat x y z p b g := by
  funext t
  simp only [omegaHeat, heat, Pi.add_apply]
  ring

private theorem omega_sub (x y z : G) (p b : ℤ) (f g : G → ℤ) :
    omegaHeat x y z p b (f-g) = omegaHeat x y z p b f - omegaHeat x y z p b g := by
  funext t
  simp only [omegaHeat, heat, Pi.add_apply, Pi.sub_apply]
  ring

theorem curl_omega (x y z : G) (p b : ℤ) (v : VectorField G) :
    curlG x y z (omegaHeatV x y z p b v) = omegaHeatV x y z p b (curlG x y z v) := by
  simp only [curlG, omegaHeatV, diff_omega, omega_sub]

theorem div_omega (x y z : G) (p b : ℤ) (v : VectorField G) :
    divG x y z (omegaHeatV x y z p b v) = omegaHeat x y z p b (divG x y z v) := by
  simp only [divG, omegaHeatV, diff_omega, omega_add]

theorem represents_iterate (x y z : G) (p b : ℤ) {w v : VectorField G}
    (h : Represents x y z w v) (k : ℕ) :
    Represents x y z ((omegaHeatV x y z p b)^[k] w) ((omegaHeatV x y z p b)^[k] v) := by
  induction k with
  | zero => exact h
  | succ k ih =>
    rw [Function.iterate_succ_apply', Function.iterate_succ_apply']
    constructor
    · rw [curl_omega, ih.1]
    · rw [div_omega, ih.2]
      funext t
      simp [omegaHeat, heat]

theorem interval_omega (x y z : G) (p b lo hi : ℤ) (hp : 0 ≤ p) (hcfl : 42*p ≤ 16*b)
    {f : G → ℤ} (h : ∀ t, lo ≤ f t ∧ f t ≤ hi) (t : G) :
    (16*b)*lo ≤ omegaHeat x y z p b f t ∧ omegaHeat x y z p b f t ≤ (16*b)*hi := by
  have hnear := heat_interval x y z (4*p) (16*b-18*p) lo hi (by omega) (by omega) h t
  have hfar := heat_interval (x+x) (y+y) (z+z) (3*p) (18*p) lo hi (by omega) (by omega) h t
  simp only [omegaHeat, Pi.add_apply]
  exact ⟨by nlinarith only [hnear.1, hfar.1], by nlinarith only [hnear.2, hfar.2]⟩

/-- The two jump coefficients have unit second moment and the specified
    coefficient of h² in the local Taylor defect. -/
theorem moments : (1/4 : ℚ)*1^2 + (3/16 : ℚ)*2^2 = 1 ∧
    ((1/4 : ℚ)*1^4 + (3/16 : ℚ)*2^4)/12 = 13/48 := by norm_num

end NSRegularity.NS.MockThetaCurl
