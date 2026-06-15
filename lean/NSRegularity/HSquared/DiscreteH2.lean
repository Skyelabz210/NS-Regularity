import NSRegularity.Kill113.Component
import NSRegularity.HSquared
import NSRegularity.HSquared.SBP

/-!
# HSquared/DiscreteH2 — Discrete H² energy identity + Kill #129 (eigen-Beltrami class)

Port of the workspace file `QmnfNS_DiscreteH2.lean` (session 2026-04-28,
NS_LEAN_FORMALIZED) onto the repository's proven operator stack. The
workspace file carried 4 sorries; this port discharges three of them by
wiring into existing repo theorems:

| Workspace item                     | Repo discharge                                  |
|------------------------------------|-------------------------------------------------|
| `kill_113` (sorry: SBP)            | `Kill113.component_residual_zero` (proved, and  |
|                                    | *unconditional* — no div-free hypothesis needed) |
| `discrete_h2_identity` (sorry)     | `h2_update_identity` + `h2_cross_decomposition` |
| `kill_129_beltrami_signed_ip`      | `signedIp_eigen_zero` — proved for discrete     |
| (sorry)                            | Laplacian eigenfields (the ABC/Beltrami class    |
|                                    | where the exact integer zeros were observed)     |
| `h2_monotonicity_inequality` (open)| `h2_step_monotone_iff` makes the equivalence a  |
|                                    | theorem; the inequality itself remains open      |

## The commutator collapse

The H² commutator pairing (`signed_ip` in the Rust/workspace lineage) is

```
signedIp u ω = ⟨Δω, Δ(A_skew u ω) − A_skew u (Δω)⟩
```

The diagonal term `⟨Δω, A_skew u (Δω)⟩` vanishes **unconditionally** by
Kill #113 applied at the field `Δω` (`component_residual_zero` holds for
*any* grid field, with no divergence-free hypothesis — the skew form
adv+cons cancels by pure 1D summation-by-parts). Hence

```
signedIp u ω = ⟨Δω, Δ(A_skew u ω)⟩        (signedIp_eq_cross)
```

## Kill #129, eigen-Beltrami class

If `Δ_h f = μ·f` (discrete Laplacian eigenfield), then by self-adjointness
(`sbp_self_adjoint_bilinear`):

```
⟨Δf, Δ(A f)⟩ = ⟨A f, Δ²f⟩ = μ²·⟨f, A f⟩ = μ²·0 = 0     (Kill #113)
```

so `signedIp u f = 0` — an exact integer zero, matching the empirical
`signed_ip = 0` observed for ABC flows at amplitudes 10–10000 in
`qcid_ns_bridge`. Beltrami fields `ω = λ·u` over an eigenfield velocity
inherit the eigenfield property, giving Kill #129 for that class with
**no sorry**. General Beltrami data (alignment only, no eigen structure)
reduces to `signedIp u u_c = 0`, which remains open here.

## Honest scope

- All identities in this file are exact integer algebra over the
  *division-free* update increment `δ = ca·Δω − cb·A_skew ω + cs·S`
  (matching the Resolution-A exactness posture; the `/den` of the raw
  step is handled by the divisibility hypotheses of `ResolutionA.lean`).
- The H² monotonicity inequality (NS-13) is stated as an equivalence
  theorem, not resolved. It is the open diffusion-vs-stretching balance.
-/

namespace NSRegularity.HSquared

open NSRegularity NSRegularity.NS NSRegularity.Kill113

/-! ## sumGrid linearity helpers -/

theorem sumGrid_add (f g : Grid) :
    sumGrid (fun i j k => f i j k + g i j k) = sumGrid f + sumGrid g := by
  unfold sumGrid
  simp only [Finset.sum_add_distrib]

theorem sumGrid_sub (f g : Grid) :
    sumGrid (fun i j k => f i j k - g i j k) = sumGrid f - sumGrid g := by
  unfold sumGrid
  simp only [Finset.sum_sub_distrib]

theorem sumGrid_smul (c : Int) (f : Grid) :
    sumGrid (fun i j k => c * f i j k) = c * sumGrid f := by
  unfold sumGrid
  simp only [← Finset.mul_sum]

/-! ## Discrete L² pairing and the skew advection operator -/

/-- Discrete L² inner product on the grid (workspace `inner_product`). -/
def innerProduct (f g : Grid) : Int :=
  sumGrid (fun i j k => f i j k * g i j k)

/-- Skew-symmetric advection (Kill #113 form): advective + conservative.
    Workspace `advection_skew`, on the repo's `Kill113.Ops` operators. -/
def askew (u : VField) (f : Grid) : Grid :=
  fun i j k => advective u f i j k + conservative u f i j k

/-- **Kill #113, Sobolev form.** `⟨g, A_skew u g⟩ = 0` for *any* grid field
    `g` — in particular `g := Δω`, which is the H²-level vanishing.
    Direct restatement of `component_residual_zero`; note the repo theorem
    needs no divergence-free hypothesis (the skew form cancels by pure
    summation-by-parts). -/
theorem kill_113_sobolev (u : VField) (g : Grid) :
    innerProduct g (askew u g) = 0 :=
  component_residual_zero u g

/-! ## The H² commutator pairing (signed_ip) -/

/-- H² commutator pairing: `⟨Δω, [Δ, A_skew(u,·)] ω⟩`.
    Workspace/Rust `signed_ip`. For Beltrami eigenfields this is an exact
    integer zero (Kill #129, below); for general flows it oscillates. -/
def signedIp (u : VField) (ω : Grid) : Int :=
  innerProduct (laplacian ω)
    (fun i j k => laplacian (askew u ω) i j k - askew u (laplacian ω) i j k)

/-- **Commutator collapse.** The diagonal term `⟨Δω, A_skew u (Δω)⟩`
    vanishes by Kill #113 at the field `Δω`, so the commutator pairing
    reduces to the single cross term `⟨Δω, Δ(A_skew u ω)⟩`. -/
theorem signedIp_eq_cross (u : VField) (ω : Grid) :
    signedIp u ω = innerProduct (laplacian ω) (laplacian (askew u ω)) := by
  unfold signedIp
  show sumGrid (fun i j k => laplacian ω i j k *
        (laplacian (askew u ω) i j k - askew u (laplacian ω) i j k))
      = innerProduct (laplacian ω) (laplacian (askew u ω))
  have hpt : (fun i j k : Fin N => laplacian ω i j k *
        (laplacian (askew u ω) i j k - askew u (laplacian ω) i j k))
      = (fun i j k : Fin N =>
          laplacian ω i j k * laplacian (askew u ω) i j k
          - laplacian ω i j k * askew u (laplacian ω) i j k) := by
    funext i j k; ring
  rw [hpt, sumGrid_sub]
  have h0 : sumGrid (fun i j k => laplacian ω i j k * askew u (laplacian ω) i j k) = 0 :=
    kill_113_sobolev u (laplacian ω)
  rw [h0, sub_zero]
  rfl

/-! ## Kill #129 — eigen-Beltrami commutator vanishing -/

/-- **Kill #129 (eigenfield form).** If `Δ_h f = μ·f` at every grid point,
    the H² commutator pairing vanishes exactly:

    `⟨Δf, Δ(A f)⟩ = ⟨A f, Δ²f⟩ = μ²·⟨f, A f⟩ = 0` by self-adjointness then
    Kill #113. This is the structural reason for the exact integer zeros
    observed for ABC flows in `qcid_ns_bridge` (amplitudes 10–10000). -/
theorem signedIp_eigen_zero (u : VField) (f : Grid) (μ : Int)
    (heig : ∀ i j k : Fin N, laplacian f i j k = μ * f i j k) :
    signedIp u f = 0 := by
  rw [signedIp_eq_cross]
  show sumGrid (fun i j k => laplacian f i j k * laplacian (askew u f) i j k) = 0
  rw [sbp_self_adjoint_bilinear (laplacian f) (askew u f)]
  have hfun : laplacian f = fun i' j' k' => μ * f i' j' k' := by
    funext i' j' k'; exact heig i' j' k'
  have hlap2 : ∀ i j k : Fin N,
      laplacian (laplacian f) i j k = μ * (μ * f i j k) := by
    intro i j k
    calc laplacian (laplacian f) i j k
        = laplacian (fun i' j' k' => μ * f i' j' k') i j k := by rw [hfun]
      _ = μ * laplacian f i j k := by simp only [laplacian]; ring
      _ = μ * (μ * f i j k) := by rw [heig i j k]
  have hpt : (fun i j k : Fin N => askew u f i j k * laplacian (laplacian f) i j k)
      = (fun i j k : Fin N => μ * μ * (f i j k * askew u f i j k)) := by
    funext i j k; rw [hlap2 i j k]; ring
  rw [hpt, sumGrid_smul]
  have h0 : sumGrid (fun i j k => f i j k * askew u f i j k) = 0 :=
    kill_113_sobolev u f
  rw [h0, mul_zero]

/-- Beltrami alignment: `ω = λ·u` componentwise (workspace `IsBeltrami`). -/
def IsBeltrami (ω u : VField) (lam : Int) : Prop :=
  (∀ i j k : Fin N, ω.x i j k = lam * u.x i j k) ∧
  (∀ i j k : Fin N, ω.y i j k = lam * u.y i j k) ∧
  (∀ i j k : Fin N, ω.z i j k = lam * u.z i j k)

/-- A scalar multiple of a discrete Laplacian eigenfield is again an
    eigenfield with the same eigenvalue, hence its commutator pairing
    vanishes. The Beltrami workhorse lemma. -/
theorem signedIp_smul_eigen_zero (u : VField) (g f : Grid) (lam μ : Int)
    (hf : ∀ i j k : Fin N, f i j k = lam * g i j k)
    (heig : ∀ i j k : Fin N, laplacian g i j k = μ * g i j k) :
    signedIp u f = 0 := by
  apply signedIp_eigen_zero u f μ
  intro i j k
  have hfun : f = fun i' j' k' => lam * g i' j' k' := by
    funext i' j' k'; exact hf i' j' k'
  calc laplacian f i j k
      = laplacian (fun i' j' k' => lam * g i' j' k') i j k := by rw [hfun]
    _ = lam * laplacian g i j k := by simp only [laplacian]; ring
    _ = lam * (μ * g i j k) := by rw [heig i j k]
    _ = μ * (lam * g i j k) := by ring
    _ = μ * f i j k := by rw [hf i j k]

/-- **Kill #129 (Beltrami eigen class), all three components.**
    For `ω = λ·u` with each velocity component a discrete Laplacian
    eigenfield, the H² commutator pairing is an exact integer zero on
    every vorticity component. No sorry; no divergence-free hypothesis. -/
theorem kill_129_beltrami_eigen (ω u : VField) (lam μ : Int)
    (hbel : IsBeltrami ω u lam)
    (heig_x : ∀ i j k : Fin N, laplacian u.x i j k = μ * u.x i j k)
    (heig_y : ∀ i j k : Fin N, laplacian u.y i j k = μ * u.y i j k)
    (heig_z : ∀ i j k : Fin N, laplacian u.z i j k = μ * u.z i j k) :
    signedIp u ω.x = 0 ∧ signedIp u ω.y = 0 ∧ signedIp u ω.z = 0 :=
  ⟨signedIp_smul_eigen_zero u u.x ω.x lam μ hbel.1 heig_x,
   signedIp_smul_eigen_zero u u.y ω.y lam μ hbel.2.1 heig_y,
   signedIp_smul_eigen_zero u u.z ω.z lam μ hbel.2.2 heig_z⟩

/-! ## The discrete H² energy identity -/

/-- `hTwoSeminorm` in `sumGrid` form (definitional). -/
theorem hTwoSeminorm_eq_sumGrid (f : Grid) :
    hTwoSeminorm f = sumGrid (fun i j k => (laplacian f i j k) ^ 2) := rfl

/-- **Discrete H² expansion** — exact, no continuous-calculus approximation.
    For any update `ω' = ω + δ`:

    `‖Δω'‖² = ‖Δω‖² + 2⟨Δω, Δδ⟩ + ‖Δδ‖²`

    The `‖Δδ‖²` term is the discrete-time correction the reviewer flagged
    as missing from continuous-calculus arguments; it is a sum of squares,
    hence always ≥ 0. -/
theorem h2_update_identity (ω δ : Grid) :
    hTwoSeminorm (fun i j k => ω i j k + δ i j k)
    = hTwoSeminorm ω + 2 * innerProduct (laplacian ω) (laplacian δ)
      + hTwoSeminorm δ := by
  rw [hTwoSeminorm_eq_sumGrid, hTwoSeminorm_eq_sumGrid ω, hTwoSeminorm_eq_sumGrid δ]
  show sumGrid (fun i j k =>
        (laplacian (fun i' j' k' => ω i' j' k' + δ i' j' k') i j k) ^ 2)
      = sumGrid (fun i j k => (laplacian ω i j k) ^ 2)
        + 2 * sumGrid (fun i j k => laplacian ω i j k * laplacian δ i j k)
        + sumGrid (fun i j k => (laplacian δ i j k) ^ 2)
  have hpt : (fun i j k : Fin N =>
        (laplacian (fun i' j' k' => ω i' j' k' + δ i' j' k') i j k) ^ 2)
      = (fun i j k : Fin N =>
          (laplacian ω i j k) ^ 2
          + 2 * (laplacian ω i j k * laplacian δ i j k)
          + (laplacian δ i j k) ^ 2) := by
    funext i j k
    simp only [laplacian]
    ring
  rw [hpt, sumGrid_add, sumGrid_add, sumGrid_smul]

/-- **NS-13 as an equivalence.** Per-step H² monotonicity holds iff the
    single integer inequality `2⟨Δω, Δδ⟩ + ‖Δδ‖² ≤ 0` holds. The repo form
    of the workspace's `h2_monotonicity_inequality` — here the equivalence
    is a theorem; the inequality at reachable states is the open problem. -/
theorem h2_step_monotone_iff (ω δ : Grid) :
    hTwoSeminorm (fun i j k => ω i j k + δ i j k) ≤ hTwoSeminorm ω ↔
    2 * innerProduct (laplacian ω) (laplacian δ) + hTwoSeminorm δ ≤ 0 := by
  rw [h2_update_identity]
  constructor <;> intro h <;> linarith

/-- **H² cross-term decomposition.** For the division-free NS increment
    `δ = ca·Δω − cb·A_skew ω + cs·S` (diffusion / advection / stretching
    coefficients kept separate):

    `⟨Δω, Δδ⟩ = ca·(−‖∇⁺Δω‖²) − cb·signedIp(u, ω) + cs·⟨Δω, ΔS⟩`

    The diffusion term is dissipative by `sbp_quadratic_h2`; the advection
    term is exactly the commutator pairing (the diagonal already vanished
    via Kill #113); the stretching cross term is the open balance. -/
theorem h2_cross_decomposition (u : VField) (ω S : Grid) (ca cb cs : Int) :
    innerProduct (laplacian ω)
      (laplacian (fun i j k =>
        ca * laplacian ω i j k - cb * askew u ω i j k + cs * S i j k))
    = ca * (-gradSqForward (laplacian ω)) - cb * signedIp u ω
      + cs * innerProduct (laplacian ω) (laplacian S) := by
  rw [signedIp_eq_cross, ← sbp_quadratic_h2 ω]
  show sumGrid (fun i j k => laplacian ω i j k *
        laplacian (fun i' j' k' =>
          ca * laplacian ω i' j' k' - cb * askew u ω i' j' k' + cs * S i' j' k') i j k)
      = ca * sumGrid (fun i j k =>
            laplacian ω i j k * laplacian (laplacian ω) i j k)
        - cb * sumGrid (fun i j k =>
            laplacian ω i j k * laplacian (askew u ω) i j k)
        + cs * sumGrid (fun i j k =>
            laplacian ω i j k * laplacian S i j k)
  have hpt : (fun i j k : Fin N => laplacian ω i j k *
        laplacian (fun i' j' k' =>
          ca * laplacian ω i' j' k' - cb * askew u ω i' j' k' + cs * S i' j' k') i j k)
      = (fun i j k : Fin N =>
          ca * (laplacian ω i j k * laplacian (laplacian ω) i j k)
          - cb * (laplacian ω i j k * laplacian (askew u ω) i j k)
          + cs * (laplacian ω i j k * laplacian S i j k)) := by
    funext i j k
    simp only [laplacian]
    ring
  rw [hpt, sumGrid_add, sumGrid_sub, sumGrid_smul, sumGrid_smul, sumGrid_smul]

/-- **The discrete H² energy identity** (workspace `discrete_h2_identity`,
    now sorry-free). For the division-free increment
    `δ = ca·Δω − cb·A_skew ω + cs·S`:

    `‖Δ(ω+δ)‖² − ‖Δω‖² = 2(ca·(−‖∇⁺Δω‖²) − cb·signedIp + cs·⟨Δω,ΔS⟩) + ‖Δδ‖²`

    With `ca = ν·Δt`, `cb = cs = Δt` this is the exact integer H² budget of
    one NS step (pre-division; the `/den` rounding is covered by the
    Resolution-A divisibility hypotheses elsewhere). -/
theorem discrete_h2_identity_exact (u : VField) (ω S : Grid) (ca cb cs : Int) :
    hTwoSeminorm (fun i j k => ω i j k +
        (ca * laplacian ω i j k - cb * askew u ω i j k + cs * S i j k))
      - hTwoSeminorm ω
    = 2 * (ca * (-gradSqForward (laplacian ω)) - cb * signedIp u ω
            + cs * innerProduct (laplacian ω) (laplacian S))
      + hTwoSeminorm (fun i j k =>
          ca * laplacian ω i j k - cb * askew u ω i j k + cs * S i j k) := by
  have hupd : hTwoSeminorm (fun i j k => ω i j k +
        (ca * laplacian ω i j k - cb * askew u ω i j k + cs * S i j k))
      = hTwoSeminorm ω
        + 2 * innerProduct (laplacian ω)
            (laplacian (fun i j k =>
              ca * laplacian ω i j k - cb * askew u ω i j k + cs * S i j k))
        + hTwoSeminorm (fun i j k =>
            ca * laplacian ω i j k - cb * askew u ω i j k + cs * S i j k) :=
    h2_update_identity ω
      (fun i j k => ca * laplacian ω i j k - cb * askew u ω i j k + cs * S i j k)
  have hcross := h2_cross_decomposition u ω S ca cb cs
  rw [hcross] at hupd
  linarith

/-- **Beltrami-eigen H² budget — no advection leakage.** For Beltrami
    eigenfields the commutator term drops from the H² identity entirely:

    `‖Δ(ω+δ)‖² − ‖Δω‖² = 2(ca·(−‖∇⁺Δω‖²) + cs·⟨Δω,ΔS⟩) + ‖Δδ‖²`

    leaving only the dissipative diffusion term against the stretching
    cross term — theorem T2 of the Beltrami theorem stack. -/
theorem discrete_h2_identity_beltrami_eigen (u : VField) (ωc S : Grid)
    (lam μ : Int) (ca cb cs : Int)
    (hbel : ∀ i j k : Fin N, ωc i j k = lam * u.x i j k)
    (heig : ∀ i j k : Fin N, laplacian u.x i j k = μ * u.x i j k) :
    hTwoSeminorm (fun i j k => ωc i j k +
        (ca * laplacian ωc i j k - cb * askew u ωc i j k + cs * S i j k))
      - hTwoSeminorm ωc
    = 2 * (ca * (-gradSqForward (laplacian ωc))
            + cs * innerProduct (laplacian ωc) (laplacian S))
      + hTwoSeminorm (fun i j k =>
          ca * laplacian ωc i j k - cb * askew u ωc i j k + cs * S i j k) := by
  have hzero : signedIp u ωc = 0 :=
    signedIp_smul_eigen_zero u u.x ωc lam μ hbel heig
  have hid := discrete_h2_identity_exact u ωc S ca cb cs
  rw [hzero] at hid
  linarith

/-!
## Summary of formal status

PROVED (no sorry, this file):
- `kill_113_sobolev` — ⟨g, A_skew g⟩ = 0 for any field (unconditional).
- `signedIp_eq_cross` — commutator collapse to single cross term.
- `signedIp_eigen_zero` — Kill #129 for discrete Laplacian eigenfields.
- `kill_129_beltrami_eigen` — Kill #129, Beltrami eigen class, 3 components.
- `h2_update_identity` — exact discrete H² expansion (with the |Δδ|² term).
- `h2_step_monotone_iff` — NS-13 stated as an equivalence theorem.
- `h2_cross_decomposition` — diffusion/commutator/stretching split.
- `discrete_h2_identity_exact` — the full discrete H² energy identity.
- `discrete_h2_identity_beltrami_eigen` — T2: no advection leakage.

REMAINING OPEN (tracked in PROOF_DAG.md):
- NS-13 inequality at reachable states:
  `2(ca·(−‖∇⁺Δω‖²) − cb·signedIp + cs·⟨Δω,ΔS⟩) + ‖Δδ‖² ≤ 0`.
  For the Beltrami eigen class the signedIp term is gone, so the balance
  is `2·ca·‖∇⁺Δω‖² ≥ 2·cs·⟨Δω,ΔS⟩ + ‖Δδ‖²` — diffusion vs stretching
  with the discrete correction explicit.
- General-Beltrami (non-eigen) Kill #129: reduces to `signedIp u u_c = 0`.
-/

end NSRegularity.HSquared
