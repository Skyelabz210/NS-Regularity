import NSRegularity.NS.ExactTransport

/-! Exact scalar normalization and Fourier-symbol bounds for the two-jump
    diffusion stencil from the third-order omega coefficients. -/
namespace NSRegularity.NS.MockThetaDiffusion

/-- First two nonconstant coefficients of omega at q=1/2, normalized by the second moment. -/
theorem omega_weights :
    let w₁ : ℚ := 2 * (1/2)
    let w₂ : ℚ := 3 * (1/2)^2
    let moment := w₁ + 4*w₂
    w₁/moment = 1/4 ∧ w₂/moment = 3/16 ∧ (w₁/moment + 4*(w₂/moment)) = 1 := by
  norm_num

/-- Per-axis multipliers. Summing with the same positive mesh factor gives
    the corresponding three-dimensional bound. -/
theorem omega_symbol_bounds (x : ℝ) :
    (1/4)*(Real.sin x)^2 ≤ (1/4)*(Real.sin x)^2 + (3/16)*(Real.sin (2*x))^2 ∧
    (1/4)*(Real.sin x)^2 + (3/16)*(Real.sin (2*x))^2 ≤ (Real.sin x)^2 := by
  constructor
  · nlinarith only [sq_nonneg (Real.sin (2*x))]
  · have hc : (Real.cos x)^2 ≤ 1 := by nlinarith only [Real.sin_sq_add_cos_sq x, sq_nonneg (Real.sin x)]
    have h := mul_le_mul_of_nonneg_left hc (sq_nonneg (Real.sin x))
    rw [Real.sin_two_mul]
    nlinarith only [h]

/-- The coefficient at a cell center is nonnegative under the three-axis
    explicit CFL bound. The six near and six far weights sum with it to one. -/
theorem explicit_weights (r : ℚ) (hr : 0 ≤ r) (hcfl : r ≤ 8/21) :
    0 ≤ 1-6*r*(1/4+3/16) ∧ 0 ≤ r/4 ∧ 0 ≤ 3*r/16 ∧
      1-6*r*(1/4+3/16) + 6*(r/4) + 6*(3*r/16) = 1 := by
  constructor
  · linarith
  · refine ⟨by positivity,by positivity,by ring⟩

end NSRegularity.NS.MockThetaDiffusion
