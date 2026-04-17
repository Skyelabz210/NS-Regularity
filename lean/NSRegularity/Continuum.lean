import NSRegularity.DKAM
import NSRegularity.AubinLions

/-!
# M6 — L∞ Passes to Continuum (top-level stub)

**Proof step 6.** Piecewise-constant interpolation I_N has zero L∞ overshoot:
  ‖I_N ω_N‖_{L∞} = ‖ω_N‖_{ℓ∞}

By Banach-Alaoglu a subsequence converges weak-* in L∞; the limit equals
the strong L² limit from M4; liminf gives
  ‖ω(t)‖_{L∞} ≤ ‖ω₀‖_∞.

**Status:** `pending_decomposition`.
-/

namespace NSRegularity.Continuum

axiom ContinuumLInf : NSRegularity.VField → Prop

/-- **M6 — L∞ passes to continuum** (top-level claim). -/
theorem l_inf_passage (ω₀ : NSRegularity.VField) : ContinuumLInf ω₀ := by
  sorry

end NSRegularity.Continuum
