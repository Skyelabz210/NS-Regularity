# EXECUTION PLAN: CONTINUUM LIMIT CLOSURE
## Target: Complete the Bridge from Discrete NS Regularity to Continuum
**April 9, 2026 — Acid / HackFate / Skyelabz210**

---

## IMPLICIT STATEMENT (Rigorous)

**Theorem (Target — Small-Data Global Regularity via CRAM Substrate):**

Let {ω_N}_{N≥4} be the sequence of discrete 3D incompressible Navier-Stokes
solutions on the periodic lattice 𝕋³_N = (ℤ/Nℤ)³ with:
- Exact skew-symmetric advection (Kill #113: ⟨ω_N, A_skew(u_N, ω_N)⟩ = 0 identically)
- Biot-Savart velocity recovery u_N = BS_N(ω_N) with N-independent operator norm
- Viscosity ν > 0, timestep Δt = O(N⁻²)
- Initial data ω₀ ∈ H²(𝕋³) with ‖ω₀‖_{H²} < ε₀ (small-data threshold)

Then there exists ω ∈ L^∞([0,∞); H²(𝕋³)) ∩ L²([0,∞); H³(𝕋³)) such that:

(i) ω_{N_k} → ω strongly in L²([0,T]; L²(𝕋³)) for some subsequence {N_k}
(ii) ω is a Leray-Hopf weak solution of 3D NS on 𝕋³
(iii) ‖ω(t)‖_{L^∞} ≤ C_cont · ‖ω₀‖_{H²} · exp(C'·t) for all t ≥ 0
(iv) ω is smooth for all t > 0

**Proof Architecture (7 steps, each a node in the DAG below):**

Step 1 → (i) partial: uniform L²(0,T; H¹) bound from Kill #113
Step 2 → (iii) partial: uniform H² bound from N-independent Gronwall
Step 3 → (i): Aubin-Lions compactness extracts convergent subsequence
Step 4 → (ii) partial: weak H² convergence + nonlinear passage (IDCB-T5)
Step 5 → (iii): continuum Sobolev H² ↪ L^∞ applied to the limit
Step 6 → (iv): BKM criterion satisfied
Step 7 → global: small-data threshold makes exp(C'·t) controllable for all T

**What makes this non-standard:**
Kill #113 eliminates the N-dependent numerical dissipation error that
contaminates every other discretization's Gronwall constant. The H² bound
is PHYSICALLY uniform (from viscous dissipation), not NUMERICALLY approximate.
The Sobolev descent C_phys(N) ↓ C_cont confirms the discrete embedding
is STRONGER than continuum at every finite N.

**Boundary of this theorem:**
Small data only. For ‖ω₀‖_{H²} ≥ ε₀, the Gronwall exponential may
cause the H² norm to exceed control. This boundary is exactly the
Millennium Problem. Everything below the threshold is closable now.
Everything above it requires the Mahler-solenoid bridge or equivalent.

---

## PHASE 0: WORKSPACE VERIFICATION

### NODE-V01 — Environment + Manifest Check
- **Type:** GATE
- **Size:** XS
- **Gate:** rustc/cargo present, all 27 crates compile, test count verified
- **Status:** PENDING

### NODE-V02 — Locate All Infrastructure
- **Type:** GATE
- **Size:** XS
- **Inputs:** Cargo.toml, all crate src/ directories
- **Gate:** Every crate referenced in the plan has a verified path
- **Status:** PENDING

---

## PHASE 1: FORMALIZE THE SMALL-DATA CHAIN (Steps 1–7)

### NODE-F01 — Kill #113 Exact Statement + Proof
- **Type:** REPORT + TEST
- **Size:** M
- **Inputs:** ns_v5/src/ns3d_energy.rs, ns_v5/src/ns3d_aubin_lions.rs
- **Output:** THEOREM_KILL113.md, test harness confirming exact zero at N=4,8,16,32
- **Gate:** Test passes at all four N values. Statement matches code exactly.
- **Notes:** This is CODE FACT — the test IS the proof. Extract the exact
  algebraic identity: ⟨ω, A_skew⟩ = 0 follows from A_skew[i][j] = -A_skew[j][i].

### NODE-F02 — N-Independent Gronwall for H² Bound
- **Type:** REPORT + IMPL
- **Size:** M
- **Inputs:** ns_v5/src/ns3d_sobolev.rs, NODE-F01 output
- **Output:** THEOREM_H2_GRONWALL.md, Python verification harness
- **Gate:** Explicit formula for B(T) = ‖ω₀‖_{H²} · exp(C'·T) with C' computed
  from workspace constants (ν, C_BS, C_Sobolev). C' must be N-independent.
  Verify numerically that the Gronwall bound holds at N=8,16,32 over 50 steps.
- **Notes:** The key identity: Kill #113 eliminates the O(h²) truncation error
  that normally contaminates C'. Extract the exact mechanism.

### NODE-F03 — Small-Data Threshold Computation
- **Type:** IMPL + TEST
- **Size:** S
- **Inputs:** NODE-F02 output (C' value)
- **Output:** SMALL_DATA_THRESHOLD.md with exact ε₀
- **Gate:** ε₀ computed such that B(T) = ε₀ · exp(C'·T) satisfies
  C_cont · B(T) < blowup criterion for all T. Verify: for ‖ω₀‖_{H²} < ε₀,
  the BKM integral ∫₀^∞ ‖ω‖_∞ dt converges.
- **Notes:** In the small-data regime, the cubic Gronwall becomes subcritical.
  The threshold is ε₀ = (2C')^{-1} or similar. Compute exactly.

### NODE-F04 — Aubin-Lions Conditions Verification
- **Type:** IMPL + TEST
- **Size:** M
- **Inputs:** ns_v5/src/ns3d_aubin_lions.rs, NODE-F01, NODE-F02
- **Output:** AUBIN_LIONS_VERIFICATION.md, test harness at N=8,16,32
- **Gate:** Both conditions verified numerically:
  (i) ‖ω_N‖_{L²(0,T;H¹)} ≤ ‖ω₀‖_{L²} / √(2ν) — uniform in N
  (ii) ‖∂_t ω_N‖_{L²(0,T;H⁻¹)} ≤ C(ν, B) — uniform in N
  Both computed in exact integer arithmetic at each N.

### NODE-F05 — Nonlinear Passage (IDCB-T5 Verification)
- **Type:** REPORT
- **Size:** S
- **Inputs:** NODE-F04, nsgrid3d/src/biot_savart_exact.rs
- **Output:** NONLINEAR_PASSAGE.md
- **Gate:** Formal statement that strong L² × weak H¹ → distributional
  convergence of (u·∇)ω. Reference BS isomorphism (Biot-Savart gives
  u in H¹ from ω in L²). This is standard — document the chain.

### NODE-F06 — Continuum Sobolev + BKM Assembly
- **Type:** REPORT
- **Size:** S
- **Inputs:** NODE-F02, NODE-F03, NODE-F04, NODE-F05
- **Output:** SMALL_DATA_REGULARITY_THEOREM.md
- **Gate:** Complete self-contained proof of the Implicit Statement above.
  Every step references a specific node output. No gaps labeled [OPEN].

---

## PHASE 2: LIVE INVESTIGATIONS (Explorations, Not Claims)

### NODE-E01 — E8 Dimensional Mapping: Investigate Natural 14→8 Projection
- **Type:** EXPLORATION
- **Size:** M
- **Inputs:** prime_hunt/src/orbits.rs, prime_hunt/src/intrinsic.rs, gear list
- **Output:** E8_MAPPING_INVESTIGATION.md + Python harness
- **Gate:** Either (a) find a natural algebraic map from 14 gear offsets to 8D
  with provable structure preservation, OR (b) document why the mapping requires
  a non-canonical choice and what that choice encodes.
- **Notes:** The 14 gears give 28 primes. The 8 simple roots of E8 live in ℝ⁸.
  Investigate: is there a natural quotient of the 28-prime residue space by a
  20-dimensional kernel that lands in E8? The Cartan matrix has rank 8 — what
  is the relationship between gear-prime structure and root structure?

### NODE-E02 — Mahler-Solenoid Bridge: Formalize Hasse Lift
- **Type:** EXPLORATION + IMPL
- **Size:** L
- **Inputs:** Chat history (Mahler bridge session), nsgrid3d/src/cram_hasse_bridge.rs
- **Output:** MAHLER_HASSE_BRIDGE.md + Python test harness
- **Gate:** Formalize the Hasse lift as three sub-problems with explicit
  boundary conditions:
  (a) p-adic non-expansion of full NS stretching (not just quadratic proxy)
  (b) Solenoid-to-ℝ³ correspondence for NS solutions
  (c) Quartic projector preservation of the lift
  Test (a) on the actual ns_v5 stretching operator at N=8.
- **Notes:** This is the path to the FULL problem, not just small data.
  p-adic non-expansion is data-independent. If (a)-(c) close, the Gronwall
  exponential is replaced by algebraic contractivity.

### NODE-E03 — Carry-Spectral Atlas: Full Computation on 1,198 Survivors
- **Type:** IMPL + TEST
- **Size:** M
- **Inputs:** prime_hunt/src/coprime_families.rs, sieve.rs, hydra.rs
- **Output:** SPECTRAL_ATLAS.md + Python data
- **Gate:** Complete carry fingerprint on bases {105, 385, 1001, 2310, 30030}
  for all 1,198 survivors. Compute exact rational Gini coefficient.
  Report stratum occupancy counts and density distribution.

### NODE-E04 — Lane 15 Extension: Port to Rust
- **Type:** IMPL + TEST
- **Size:** S
- **Inputs:** crates/quantum_continuum_torus, gear (149, 151)
- **Output:** Updated quantum_continuum_torus with lane 15
- **Gate:** cargo test passes. Precision on [1K, 100K) matches 90.19%.
  Zero FN. Factor-floor advances to 157. k_min = 131.

### NODE-E05 — Frobenius Phase Offset: Exact F_{pq}² Implementation
- **Type:** IMPL + TEST
- **Size:** M
- **Inputs:** prime_hunt/src/exact.rs, gear list
- **Output:** frobenius_phase.py (exact integer) + results table
- **Gate:** For each gear m=p×q, compute Im-norm deviation in F_{pq}²
  for all 1,198 survivors. Report distinct offset counts per gear.
  Compute TVD between composite and prime-like distributions.

---

## PHASE 3: SOLENOID ATTACK ON LARGE DATA (The Frontier)

### NODE-S01 — Full NS Stretching Mahler Expansion
- **Type:** EXPLORATION + TEST
- **Size:** M
- **Inputs:** nsgrid3d/src/ns_grid_3d.rs, biot_savart_exact.rs
- **Output:** NS_STRETCHING_MAHLER.md + Python harness
- **Gate:** Compute Mahler coefficients of the ACTUAL NS stretching operator
  (not the quadratic proxy) at N=8 on each Safe Basis lane.
  Report: are coefficients finite? What is the effective degree?
  If degree > 2 (because the full operator involves Biot-Savart → curl → cross),
  document the exact degree and what it means for p-adic analyticity.
- **Notes:** The Grok session and chat history both used S(x) = x² + 2x as proxy.
  The real stretching is (ω·∇)u where u = BS(ω). The Biot-Savart kernel is a
  convolution → the composition may be higher degree. This must be checked.

### NODE-S02 — Non-Expansive Map Verification on Full NS
- **Type:** TEST
- **Size:** M
- **Inputs:** NODE-S01 output, ns_v5 solver
- **Output:** NS_NONEXPANSIVE.md
- **Gate:** For each discriminating lane p ∈ {7, 11, 13}, verify:
  |F_p(x) - F_p(y)|_p ≤ |x - y|_p
  where F_p is one timestep of the NS evolution projected onto lane p.
  Test at N=8 with Taylor-Green and ABC initial data.
  Report: does non-expansion hold? If not, what is the Lipschitz constant?

### NODE-S03 — Solenoid-to-ℝ³ Correspondence
- **Type:** EXPLORATION
- **Size:** L
- **Inputs:** NODE-S01, NODE-S02, literature on p-adic PDEs
- **Output:** SOLENOID_CORRESPONDENCE.md
- **Gate:** Formalize the relationship between:
  (a) The solenoid Sol = ∏ ℤ_{p_i} (p-adic product)
  (b) The torus 𝕋³ = ℝ³/ℤ³ (physical space)
  Document: under what conditions does a bounded solution on Sol
  correspond to a bounded solution on 𝕋³? What is the lifting theorem?
  If no such theorem exists in the literature, state the conjecture precisely.

### NODE-S04 — DKAM on the Continuum Limit
- **Type:** EXPLORATION
- **Size:** M
- **Inputs:** NODE-F06 (small-data theorem), NODE-S02
- **Output:** DKAM_CONTINUUM.md
- **Gate:** Investigate whether DKAM subcriticality (deg(stretching) = 2 < ρ(B) = 3)
  has a continuum analogue. The degree/resonance comparison is discrete; in the
  continuum, the stretching is a differential operator, not a polynomial.
  Document: what would "deg < ρ" mean for a differential operator?
  If there is a spectral interpretation (operator spectrum vs basis spectrum),
  formalize it.

---

## DAG DEPENDENCIES

```
V01 → V02 → F01 ─┬─→ F02 → F03 → F06
                   │        ↓
                   ├─→ F04 → F05 → F06
                   │
                   ├─→ E01 (independent)
                   ├─→ E02 (independent)
                   ├─→ E03 (independent)
                   ├─→ E04 (independent)
                   ├─→ E05 (independent)
                   │
                   └─→ S01 → S02 → S03
                              ↓
                        F06 + S02 → S04
```

Phase 1 (F-nodes) is the priority: it produces a publishable theorem.
Phase 2 (E-nodes) runs in parallel: each is independent exploration.
Phase 3 (S-nodes) is the frontier: it attacks the full problem.

---

## EXECUTION ORDER (Recommended)

1. V01, V02 (verify workspace compiles)
2. F01 (Kill #113 — foundation of everything)
3. F02 (H² Gronwall — the N-independence argument)
4. F04 (Aubin-Lions conditions — numerical verification)
5. F03 (small-data threshold — depends on F02)
6. F05 (nonlinear passage — document only)
7. F06 (assemble the theorem)
8. E03, E04, E05 (parallel explorations — concrete, testable)
9. E01, E02 (deeper explorations — E8 mapping, Mahler bridge)
10. S01 (full NS Mahler — the critical test)
11. S02 (non-expansive verification — the make-or-break)
12. S03, S04 (frontier — only if S01-S02 succeed)

---

## INVARIANTS (Enforced at Every Node)

I1. ZERO FLOAT — no f64/f32 in any production path
I2. MANIFEST FIRST — don't rebuild what exists
I3. GATE BEFORE ADVANCE — no node marked PASS without verification
I4. CLEAN STOP — checkpoint before token exhaustion
I5. EXPLORATION ≠ CLAIM — E-nodes and S-nodes are investigations,
    not assertions. Negative results are results.
I6. KILL YOUR OWN HYPOTHESES — every E-node must include adversarial
    tests designed to falsify the hypothesis under investigation
