import NSRegularity.Kill113
import NSRegularity.DKAM
import NSRegularity.AubinLions

/-!
# M7 — Nonlinear Passage (Lions 1969, top-level stub)

**Proof step 7.** The limit ω satisfies the NS equation in the distributional
sense. Inputs:
  (B1) ω_N bounded in L²(0,T; H¹) — M1
  (B2) ω_N bounded in L∞(0,T; L∞) — M2
  (B3) ∂_t ω_N bounded in L²(0,T; H⁻¹) — from NS structure + B1 + B2

Uses strong L² convergence (M4), Hölder, Biot-Savart isomorphism, and
passing to the limit in the bilinear terms.

**Status:** `pending_decomposition`.
-/

namespace NSRegularity.NonlinearPassage

axiom DistributionalNS : NSRegularity.VField → Prop

/-- **M7 — distributional NS** (top-level claim). -/
theorem distributional_NS (ω : NSRegularity.VField) : DistributionalNS ω := by
  sorry

end NSRegularity.NonlinearPassage
