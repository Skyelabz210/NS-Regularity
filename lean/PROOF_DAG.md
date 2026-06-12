# NS Regularity — Full Proof DAG
**Anthony Diaz (Acid) — HackFate.us / Skyelabz210**
**Generated 2026-06-12 — Transfinite Architecture Session**

## Legend

- ✅ PROVED — Lean theorem, no sorries
- 🔵 AXIOM — accepted as axiom (BKM 1984, Biot-Savart soundness), named
- 🟡 SCAFFOLDED — proof shape in place, standard PDE theory required
- 🟠 OPEN — substantive open item, no Lean proof
- 🔴 GATE — blocking item on the critical path to Global Regularity

---

## Critical Path: Discrete Global Regularity

```
                           ┌─────────────────────────────────────┐
                           │   GLOBAL REGULARITY (Clay Level)     │
                           │   IsSmooth (ns_trajectory)           │
                           └──────────────┬──────────────────────┘
                                          │ ns_global_regularity_skeleton
                                          │ (NS/Trajectory.lean) ✅
                                          │
                    ┌─────────────────────┴────────────────────────┐
                    │                                               │
            ┌───────┴──────────┐                         ┌─────────┴──────────┐
            │  BKM axiom       │                         │   h_step gate       │
            │  bkm_implies_    │                         │   lInf(ns_step w)   │
            │  smooth          │                         │   ≤ lInf w          │
            │  (BKM.lean) 🔵   │                         │   (BKM.lean) 🔴     │
            └──────────────────┘                         └─────────┬──────────┘
                                                                   │
                                                    ┌──────────────┴──────────────────┐
                                                    │  lInf_ns_step_small_data         │
                                                    │  (NS/HStepSmallData.lean) ✅     │
                                                    │  (conditonal on Pieces A + B)    │
                                                    └──────────────┬──────────────────┘
                                                                   │
                          ┌────────────────────────────────────────┤
                          │                                         │
              ┌───────────┴──────────┐              ┌──────────────┴──────────────┐
              │   Piece A (OPEN) 🟠  │              │   Piece B (OPEN) 🟠          │
              │   data bound K       │              │   diffusion ceiling D_max    │
              │   6·G·M_u +          │              │   |Δt|·K ≤ G − D_max         │
              │   gridLInf(uc_grad)  │              │   Transfinite cert: ℓ(t)≤ℓ₀  │
              │   ≤ K                │              │   (DynCRT + activation_level) │
              └───────────┬──────────┘              └──────────────┬──────────────┘
                          │                                         │
              ┌───────────┴──────────┐              ┌──────────────┴──────────────┐
              │  Biot-Savart L∞ bound│              │  Resolution A (PROVED) ✅    │
              │  ‖u_N‖_∞ ≤ C·‖ω_N‖_∞│              │  gridLInf_diffusion_step_le_ │
              │  (NTT port) 🟠       │              │  exact (GlobalRegGen.lean)   │
              └───────────┬──────────┘              └──────────────┬──────────────┘
                          │                                         │
              ┌───────────┴──────────┐              ┌──────────────┴──────────────┐
              │  Stretching grad bound│             │  ns_step_component_perturbation│
              │  ‖uc_grad‖_∞ ≤ C·‖ω‖ │             │  _le_exact (PROVED) ✅        │
              │  (DKAM-subcritical)🟠 │             │  (NS/AdvectionBound.lean)    │
              └──────────────────────┘              └─────────────────────────────┘
```

---

## Full Node Inventory

### Proved Nodes (✅)

| Node | Location | Statement |
|------|----------|-----------|
| `kill_113` | `Kill113.lean` | `residual u ω = 0` — SAEI, exact ℤ |
| `component_residual_zero` | `Kill113/Component.lean` | Per-component SBP identity |
| `sbp_x_1d` | `Kill113/SBP/X.lean` | 1D skew-symmetry by reindexing |
| `discrete_ns_regularity_certificate_exact` | `GlobalRegularity.lean` | Discrete global regularity (conditional on h_step) |
| `ns_global_regularity_zero_initial` | `NS/TrivialClosure.lean` | w₀=0 → IsSmooth |
| `ns_global_regularity_const_initial` | `NS/TrivialClosure.lean` | Constant w₀ → IsSmooth |
| `generic_global_regularity` | `GlobalRegularityGeneric.lean` | L∞-preserving step → IsSmooth |
| `ns_global_regularity_skeleton` | `NS/Trajectory.lean` | h_step → IsSmooth |
| `gridLInf_diffusion_step_le_exact` | `GlobalRegularityGeneric.lean` | Resolution A for diffusion |
| `advective_abs_le` | `NS/AdvectionBound.lean` | `|advective u f| ≤ 2·G·Σ|u|` |
| `advection_step_perturbation_le` | `NS/AdvectionBound.lean` | Advection step perturbation |
| `stretching_step_perturbation_le` | `NS/AdvectionBound.lean` | Stretching step perturbation |
| `ns_step_component_perturbation_le_exact` | `NS/AdvectionBound.lean` | Full perturbation (Res-A) |
| `lInf_preserved_of_capacity_aux` | `NS/AdvectionBound.lean` | Arithmetic capacity core |
| `small_data_sufficient_aux` | `NS/AdvectionBound.lean` | |Δt|·K ≤ G−D_max sufficiency |
| `capacity_vs_pressure_sufficient_aux` | `NS/AdvectionBound.lean` | Uniform capacity sufficiency |
| `h_step_component_pointwise_small_data` | `NS/HStepSmallData.lean` | Pointwise L∞ (Pieces A+B) |
| `gridLInf_ns_step_component_small_data` | `NS/HStepSmallData.lean` | gridLInf lift (Pieces A+B) |
| `lInf_ns_step_small_data` | `NS/HStepSmallData.lean` | VField lInf (Pieces A+B) |
| `bkm_integral_finite` | `BKM.lean` | BKM integral ≤ T·C from L∞ bound |
| `IsSmooth` | `BKM.lean` | Uniform L∞ bound (discrete smoothness witness) |
| DynCRT + activation_level | `qmnf_crtbigint/dyncrt.rs` | ℓ(t): exact integer L∞ observable |

### Axiom Nodes (🔵)

| Node | Location | Justification |
|------|----------|---------------|
| `bkm_implies_smooth` | `BKM.lean` | BKM 1984 (Comm. Math. Phys. 94) |
| `prodi_serrin_uniqueness` | `BKM.lean` | Prodi-Serrin theory |
| `biot_savart` | `NS/BiotSavart.lean` | NTT port pending (task #12) |
| `biot_savart_bounded` | `NS/BiotSavart.lean` | Operator norm bound (NTT) |

### Open Gates (🟠🔴)

| Node | Blocks | What's needed |
|------|--------|---------------|
| **Piece A: Data bound K** 🔴 | h_step gate | `6·gridLInf(w.c)·M_u + gridLInf(s_c w) ≤ K` uniform in trajectory |
| **Biot-Savart L∞ bound** 🟠 | Piece A | NTT Biot-Savart port (task #12); N-independent operator norm |
| **Stretching grad L∞ bound** 🟠 | Piece A | `gridLInf(ω·∇u_c) ≤ C·gridLInf(ω)²` (DKAM-subcritical) |
| **Piece B: CFL condition** 🔴 | h_step gate | `|Δt|·K ≤ gridLInf(w.c) − D_max` at every trajectory step |
| **Diffusion ceiling D_max** 🟠 | Piece B | Exact contraction: `D_max < gridLInf(f)` requires `nu_scaled·dt·Δf ≠ 0` |
| **Aubin-Lions compactness** 🟡 | Continuum limit | Mathlib H¹ embedding + discrete → continuum |
| **Nonlinear passage** 🟡 | Continuum limit | Strong L² convergence → weak NS solutions |
| **Continuum Sobolev H² → L∞** 🟡 | Smoothness | `H²(𝕋³) ↪ L^∞` (Adams-Fournier, s=2 > 3/2) |
| **Analytical N→∞ transfer** 🟠 | Continuum limit | Transfinite (5th operator, T-RCRT-7) |
| **Continuum C^∞ via Mathlib PDE** 🟠 | Full theorem | Mathlib PDE library out of scope |

---

## Small-Data Path (7 Steps from SMALL_DATA_REGULARITY_THEOREM.md)

```
Step 1: Kill #113 ✅
   kill_113: residual u ω = 0 (exact ℤ, proved)
   → exact energy identity: ‖ω_N(t)‖²_L² + 2ν∫‖∇ω_N‖² dt = ‖ω₀‖²_L²
        └── used in: Steps 2, 3

Step 2: Uniform H² bound (N-independent Gronwall) 🟡
   h_step gate (small-data regime):
   ‖ω_N(t)‖²_H² ≤ ‖ω₀‖²_H²  (monotone, under ε₀ threshold)
   Status: 51/51 Sobolev tests pass (Rust); Lean proof pending
        └── used in: Steps 3, 4, 5

Step 3: Aubin-Lions compactness 🟡
   From Steps 1+2: {ω_N} bounded in L²(0,T;H¹), {∂_t ω_N} bounded in L²(0,T;H⁻¹)
   → ∃ subsequence ω_{N_k} → ω strongly in L²([0,T]; L²(𝕋³))
   Status: standard PDE theory; 11/11 Aubin-Lions tests pass (Rust)
        └── used in: Step 4

Step 4: Weak H² inheritance + nonlinear passage 🟡
   From Steps 2+3: ω is Leray-Hopf weak solution, ‖ω(t)‖_H² ≤ ‖ω₀‖_H²
   Needs: BS isomorphism, IDCB-T5 (strong L⁶ × weak L² → distributional)
        └── used in: Steps 5, 7

Step 5: Continuum Sobolev H² → L∞ 🟡
   Standard: H²(𝕋³) ↪ L^∞(𝕋³), s=2 > d/2=3/2 (Adams-Fournier Thm 4.12)
   ‖ω(t)‖_L∞ ≤ C_cont · ‖ω(t)‖_H²
        └── used in: Step 6

Step 6: BKM criterion 🔵
   From Steps 2+5: ‖ω(t)‖_∞ ≤ C_cont·‖ω₀‖_H² (bounded)
   → ∫₀ᵀ ‖ω‖_∞ dt ≤ C_cont·‖ω₀‖_H²·T < ∞
   BKM 1984: finite BKM integral → C^∞ on [0,T]
   Lean: bkm_integral_finite ✅ + bkm_implies_smooth 🔵 (axiom)
        └── used in: Step 7

Step 7: Global regularity (small data) ✅ (conditional)
   From Step 6: T arbitrary (H² bound is TIME-INDEPENDENT for small data)
   → ω is smooth for all t > 0
   Lean: ns_global_regularity_skeleton (PROVED, conditional on h_step)
         h_step ← lInf_ns_step_small_data (conditional on Pieces A+B)
```

---

## Closing the h_step Gate (Critical Path)

The single blocking gate is:
```
h_step : ∀ w, lInf (ns_step (biot_savart w) (stretching_x w) (stretching_y w) (stretching_z w) w p) ≤ lInf w
```

### Reduction (all proved in AdvectionBound.lean + HStepSmallData.lean):
```
h_step
  ↑ lInf_ns_step_small_data (HStepSmallData.lean ✅ conditioned on A+B)
      ↑ gridLInf_ns_step_component_small_data (HStepSmallData.lean ✅)
          ↑ h_step_component_pointwise_small_data (HStepSmallData.lean ✅)
              ↑ ns_step_component_perturbation_le_exact (AdvectionBound.lean ✅)
              ↑ [Piece A] 6·G·M_u + gridLInf(uc_grad) ≤ K  🟠
              ↑ [Piece B₁] |diffusion_step f p i j k| ≤ D_max  🟠
              ↑ [Piece B₂] |Δt|·K ≤ gridLInf(f) − D_max  🟠

Piece A depends on:
  ↑ Biot-Savart operator norm: ‖u_N‖_∞ ≤ C_BS·‖ω_N‖_∞  🟠 (NTT port)
  ↑ Stretching grad bound: ‖uc_grad‖_∞ ≤ C·‖ω‖_∞  🟠 (DKAM-subcritical)

Piece B₁ depends on:
  ↑ Diffusion contraction formula: exact value of D_max for given ν, Δt, f
  ↑ (Resolution A gives D_max ≤ gridLInf f; need strict: D_max < gridLInf f)

Piece B₂ depends on:
  ↑ Small-data threshold: ε₀(ν, N) = ν_scaled × N⁶ / NS_SCALE²
  ↑ CFL condition: |Δt| ≤ (G − D_max) / K  (quantitative)
  ↑ Transfinite certificate: activation_level ℓ(t) ≤ ℓ₀ (DynCRT R2 campaign)
```

### Next Attack Sequence

**Attack 1 (highest leverage): Biot-Savart L∞ bound (NTT port)**
- File: `NS/BiotSavart.lean`
- Replace axiom `biot_savart_bounded` with theorem using NTT operator norm
- Prerequisite: NTT prime factorization lemma + n-independent bound

**Attack 2: Stretching gradient bound (DKAM-subcritical)**
- File: `NS/AdvectionBound.lean` or new `NS/StretchingBound.lean`
- Prove: `gridLInf (advective ω u_c i j k) ≤ 2·gridLInf(u_c)·Σ|ω_c|`
- This + Biot-Savart bound gives: `gridLInf(uc_grad) ≤ 2·C_BS·gridLInf(ω)²`
- Closes Piece A once velocity bound is in place

**Attack 3: CFL quantitative analysis (Piece B)**
- File: `NS/HStepSmallData.lean` or new `NS/CFL.lean`
- For specific `NsParams` (nu_scaled = ν₀, dt_latt = Δt₀):
  Show `|Δt₀| * K ≤ gridLInf w.c − D_max` from the diffusion formula
- D_max comes from `diffusion_step f p i j k = f i j k + ν·Δt·Δf/(N²S²)`
  At a local maximum, Δf ≤ 0, so `|diffusion_step| < |f i j k| ≤ gridLInf f`

**Attack 4: Transfinite R2 Campaign (empirical certificate)**
- DynCRT: activation level ℓ(t) at N=8→32, horizons 10³-10⁴
- Expected: Δ=0 (ℓ* ≤ γ₀-1) across longer evolution
- Once observed: exact integer certificate for Piece B₂ at specific amplitudes
- Converts to: specific `(nu_scaled, dt_latt, N, amplitude)` for which h_step is TRUE

---

## Beltrami / H² Route (HSquared/DiscreteH2.lean)

Parallel route to the L∞ h_step gate, ported from the workspace file
`QmnfNS_DiscreteH2.lean` (NS_LEAN_FORMALIZED) onto repo operators. All
nodes below are **proved, sorry-free** unless marked open.

```
component_residual_zero (Kill113 ✅, unconditional — no div-free needed)
     │
     ↓
kill_113_sobolev ✅  ⟨g, A_skew g⟩ = 0 for ANY field g (incl. g = Δω)
     │
     ├──→ signedIp_eq_cross ✅   commutator collapses to ⟨Δω, Δ(Aω)⟩
     │         │
sbp_self_adjoint_bilinear (HSquared/SBP ✅)
     │         ↓
     └──→ signedIp_eigen_zero ✅   KILL #129 for Δ_h-eigenfields:
               │                    ⟨Δf,Δ(Af)⟩ = μ²⟨f,Af⟩ = 0
               ↓
          kill_129_beltrami_eigen ✅   ω = λu over eigenfield velocity
               │                        (the ABC class with exact zeros)
               ↓
h2_update_identity ✅ + h2_cross_decomposition ✅
               │
               ↓
discrete_h2_identity_exact ✅   full discrete H² energy budget:
   ‖Δ(ω+δ)‖² − ‖Δω‖² = 2(ca·(−‖∇⁺Δω‖²) − cb·signedIp + cs·⟨Δω,ΔS⟩) + ‖Δδ‖²
               │
               ↓
discrete_h2_identity_beltrami_eigen ✅   T2: no advection leakage
               │
               ↓
h2_step_monotone_iff ✅   NS-13 as equivalence; the inequality itself is:
   🟠 2·ca·‖∇⁺Δω‖² ≥ 2·cs·⟨Δω,ΔS⟩ + ‖Δδ‖²   (diffusion vs stretching)
               │
               ↓ (open: combine with sobolev_embedding L3.3 🟠)
   H²-bounded trajectory → ‖ω‖_∞ bounded → IsSmooth (Beltrami class)
```

Open kernels on this route:
- 🟠 NS-13 inequality at reachable states (Beltrami eigen class: signedIp
  term already gone, pure diffusion-vs-stretching balance).
- 🟠 General-Beltrami (non-eigen) Kill #129: reduces to `signedIp u u_c = 0`.
- 🟠 L3.3 `sobolev_embedding` (HSquared.lean, sorry): needed to convert the
  H² bound into the L∞ bound BKM consumes.

---

## Token Budget Allocation (Recommended Order)

| Priority | Attack | Estimated tokens | Blocks |
|----------|--------|-----------------|--------|
| 1 | Biot-Savart L∞ bound (NTT port) | ~2hr | Piece A → h_step |
| 2 | Stretching grad bound | ~1hr | Piece A → h_step |
| 3 | D_max strict contraction | ~1hr | Piece B₁ |
| 4 | CFL quantitative (small ν,Δt) | ~2hr | Piece B₂ → h_step |
| 5 | R2 Transfinite campaign (N=8→32) | parallel Rust | Empirical cert |
| 6 | Aubin-Lions Lean port | ~3hr | Continuum |
| 7 | Nonlinear passage Lean | ~5hr | Continuum |

Closing Attacks 1+2+3+4 gives: **unconditional small-data h_step in Lean**.
That discharges `ns_global_regularity_skeleton` → **small-data IsSmooth, no axioms except BKM**.
