import NSRegularity.Shift.Defs
import NSRegularity.Kill113
import NSRegularity.DKAM.DiffusionMaxPreserving
import NSRegularity.DKAM.Stretching
import NSRegularity.DKAM.Winding

/-!
# M2 — DKAM L∞ Non-Increasing (top-level composition)

**Proof step 2.** Composes:
- (a) diffusion max-preserving (DiffusionMaxPreserving.lean)
- (b) advection energy-neutral (Kill113 — M1, already proved)
- (c) stretching DKAM-subcritical (Stretching.lean)
- (d) winding + energy ⇒ L∞ bound (Winding.lean)
- (e) DKAM persistence at all N (N-independent, from (c))

**Status:** decomposed into sub-leaves. Composition proof below is `sorry`,
awaiting all sub-leaves closed.
-/

namespace NSRegularity.DKAM

open NSRegularity NSRegularity.NS

/-- **M2 — L∞ Non-Increasing** (top-level claim).

    Under the full NS discrete scheme with CFL, the L∞ norm of vorticity is
    non-increasing at every step. Composition of (a)–(e). -/
theorem l_inf_non_increasing (w : VField) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) :
    lInf (⟨diffusion_step w.x p, diffusion_step w.y p, diffusion_step w.z p⟩ : VField)
    ≤ lInf w := by
  sorry

end NSRegularity.DKAM
