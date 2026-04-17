import NSRegularity.Kill113
import NSRegularity.DKAM

/-!
# M4 — Aubin-Lions Compactness (top-level stub)

**Proof step 4.** The sequence {ω_N} satisfies:
  (i) bounded in L²(0,T; H¹) — from M1 (uniform, N-independent)
  (ii) ∂_t ω_N bounded in L²(0,T; H⁻¹) — from NS equation structure

By Aubin-Lions lemma (Lions 1969), there exists a subsequence and a limit
ω : [0,T] → L²(𝕋³) such that ω_{N_k} → ω strongly in L²(0,T; L²(𝕋³)).

**Status:** `pending_decomposition`.
-/

namespace NSRegularity.AubinLions

axiom StrongL2Limit : (Nat → NSRegularity.VField) → Prop

/-- **M4 — Aubin-Lions strong L² convergence** (top-level claim). -/
theorem strong_L2_convergence (ω_seq : Nat → NSRegularity.VField) :
    StrongL2Limit ω_seq := by
  sorry

end NSRegularity.AubinLions
