import NSRegularity.HSquared.DiscreteH2

/-!
# HSquared/BasisCFL — centered-residue sign recoverability (Kill #130)

Port of `basis_cfl_sign_recoverable` from the workspace file
`QmnfNS_DiscreteH2.lean` (its 4th sorry) and Lemma L10 / Theorem T5 of
`BASIS_CFL_THEOREM_STACK.md` (April 28, 2026), onto repo definitions.
This completes the workspace file's port: all four of its sorries are now
either discharged (`kill_113` → `component_residual_zero`,
`discrete_h2_identity` → `discrete_h2_identity_exact`,
`kill_129` → `kill_129_beltrami_eigen`, `basis_cfl` → this file) or
precisely tracked as the open NS-13 inequality.

## Content

For an integer observable `X` and modulus `M` with `−M ≤ 2X < M`
(the **basis-CFL condition**: the basis range covers the dynamics),
the centered residue reads `X` back exactly:

```
centRes X M = X
```

In particular the *sign* of the H² commutator pairing `signedIp` is a
deterministic integer observable readable from the CRT bundle whenever
`M > 2·|signedIp|` — refuting the earlier "lattice fingerprint /
doppelgänger artifact" interpretation (error E6 in the theorem stack).
The empirical record: safe basis `M ≈ 9.7·10⁶` gave 48.3% sign accuracy
on TG flow (wrap-around), extended bases `M ≈ 7·10¹²` gave 100%.
-/

namespace NSRegularity.HSquared

open NSRegularity NSRegularity.NS

/-- Centered residue of `X` modulo `M`: the representative of `X mod M`
    in `[−M/2, M/2)`. Workspace `cent`; DEF D10 of the basis-CFL stack. -/
def centRes (X M : Int) : Int :=
  if 2 * (X % M) < M then X % M else X % M - M

/-- **Kill #130 — basis-CFL sign recoverability (L10/T5).**
    If the modulus covers the observable (`−M ≤ 2X < M`), the centered
    residue equals the observable exactly. Pure integer arithmetic;
    no NS structure needed. -/
theorem centRes_eq_self (X M : Int) (hM : 0 < M)
    (hlo : -M ≤ 2 * X) (hhi : 2 * X < M) :
    centRes X M = X := by
  unfold centRes
  rcases le_or_gt 0 X with hX0 | hX0
  · -- 0 ≤ X < M: the residue is X itself and the centering test passes
    have hXM : X < M := by linarith
    have hmod : X % M = X := Int.emod_eq_of_lt hX0 hXM
    rw [hmod]
    simp only [if_pos hhi]
  · -- −M/2 ≤ X < 0: the residue is X + M and the centering test fails
    have hmod : X % M = X + M := by
      have h2 : (0 : Int) ≤ X + M * 1 := by linarith
      have h3 : X + M * 1 < M := by linarith
      have h1 : (X + M * 1) % M = X % M := Int.add_mul_emod_self_left X M 1
      have h4 : (X + M * 1) % M = X + M * 1 := Int.emod_eq_of_lt h2 h3
      linarith
    rw [hmod]
    have hge : ¬ (2 * (X + M) < M) := by linarith
    simp only [if_neg hge]
    ring

/-- Absolute-value form of the basis-CFL condition: `2·|X| < M` suffices. -/
theorem centRes_eq_self_of_abs (X M : Int) (hM : 0 < M)
    (hcfl : 2 * |X| < M) :
    centRes X M = X := by
  have h1 : X ≤ |X| := le_abs_self X
  have h2 : -X ≤ |X| := neg_le_abs X
  exact centRes_eq_self X M hM (by linarith) (by linarith)

/-- **Kill #131 — sign predictability of the H² commutator (T6).**
    Under the basis-CFL condition `M > 2·|signedIp u ω|`, the centered
    residue reads the commutator pairing exactly — its sign is a
    deterministic integer observable of the CRT bundle, not a lattice
    artifact. -/
theorem signedIp_sign_recoverable (u : VField) (ω : Grid) (M : Int)
    (hM : 0 < M) (hcfl : 2 * |signedIp u ω| < M) :
    centRes (signedIp u ω) M = signedIp u ω :=
  centRes_eq_self_of_abs (signedIp u ω) M hM hcfl

/-- **T8 corollary — Beltrami eigenfields need no basis-CFL.**
    For the Beltrami eigen class `signedIp = 0` (Kill #129), so the
    centered residue is 0 for *every* positive modulus — matching the
    exact zeros observed for ABC flow at all amplitudes (V24). -/
theorem centRes_signedIp_beltrami_eigen (u : VField) (ωc : Grid)
    (lam μ : Int)
    (hbel : ∀ i j k : Fin N, ωc i j k = lam * u.x i j k)
    (heig : ∀ i j k : Fin N, laplacian u.x i j k = μ * u.x i j k)
    (M : Int) (hM : 0 < M) :
    centRes (signedIp u ωc) M = 0 := by
  have hzero : signedIp u ωc = 0 :=
    signedIp_smul_eigen_zero u u.x ωc lam μ hbel heig
  rw [hzero]
  exact centRes_eq_self 0 M hM (by linarith) (by linarith)

end NSRegularity.HSquared
