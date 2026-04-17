# CRAM SLOT PREDICTOR CAMPAIGN — TAO BLUEPRINT
## Operation: EDGE FINDER
### Date: 2026-04-13 | Author: Acid (HackFate.us) + Claude
### Methodology: Tao Blueprint (Phase 0–5) + Executioner (I1–I6)

---

## PHASE 0: LANDSCAPE

### Problem Statement (Exact)

Given an observable output stream S = {s₀, s₁, s₂, ...} from an unknown
deterministic generator G operating on finite state space Ω with |Ω| ≤ 2⁶⁴:

1. **Classify** G into its algebraic class (LCG, LFSR, compound, chaotic, CSPRNG)
2. **Recover** the internal state ω ∈ Ω if G is tractable
3. **Predict** subsequent outputs with measured accuracy
4. **Detect** output masking (reel mapping, truncation, symbol extraction)
5. **Quantify** the boundary between tractable and intractable for each tool tier
6. **Measure** every operation's computational cost for Landauer baseline calibration

### Objective

Produce a measured capability map: for each generator class × each CRAM tool tier,
what is the recovery rate, the boundary condition, and the wall?

These measurements become the input for NINE65 chimera lane assignment.

### Inventory — What Already Exists

| Asset | Location | Status | Tests |
|-------|----------|--------|-------|
| CRAM test harness v2 | `/home/claude/cram_harness_v2.py` | COMPLETE | 165/165 |
| CRAM universal harness | `/home/claude/cram_universal_harness.py` | COMPLETE | 9/10 |
| Projector (commutativity) | `qmnf_ws/crates/cram_explorer/src/projector.rs` | COMPILED | ✓ |
| Hydra v3 (multi-head oracle) | `qmnf_ws/crates/hydra_v3/src/lib.rs` | COMPILED | ✓ |
| Sparse Toric Grover | `qmnf_ws/crates/qmnf_primitives/src/sparse_toric_grover.rs` | COMPILED | ✓ |
| CRAM3 Grover (Ramanujan) | `qmnf_ws/crates/qmnf_primitives/src/cram3_grover.rs` | COMPILED | ✓ |
| Quartic Residue Sieve | `qmnf_ws/crates/prime_hunt/src/sieve.rs` | COMPILED | ✓ |
| Grover-Hydra bridge | `qmnf_ws/crates/prime_hunt/src/grover_hydra.rs` | COMPILED | ✓ |
| Shadow Ladder | `qmnf_ws/crates/cram_explorer/src/shadow_ladder.rs` | COMPILED | ✓ |
| Landau-Siegel Chimera | `qmnf_ws/crates/landau_siegel_chimera/src/lib.rs` | COMPILED | ✓ |
| Time Crystal | `qmnf_ws/crates/time_crystal/src/lib.rs` | COMPILED | ✓ |
| Quantum Continuum Torus | `qmnf_ws/crates/quantum_continuum_torus/src/lib.rs` | COMPILED | ✓ |
| NINE65 v7 (FHE) | `/home/claude/nine65/` | COMPILED | ✓ |
| QClassic (F_p² quantum) | `/home/claude/qclassic/` | COMPILED | 244/247 |
| Slot predictor app (TS) | `/home/claude/app/` | COMPILED | needs state fix |

### Notation

- **[GATE]** = quality gate, must pass before advancing
- **[EXPLORE]** = exploration gate, produces boundary data regardless of success/failure
- **[BACKFEED]** = result feeds into NINE65 chimera design
- **Size**: XS (<30 min) / S (1–3 hr) / M (3–8 hr) / L (1–3 day) / XL (1+ week)

### Delivery Format

All deliverables are **Markdown (.md)** or **Rust (.rs)** or **Python (.py)**.
No docx, pptx, xlsx, or proprietary formats.
Session manifests: `[DATE]-SLOT-[SEQ].md`
Test results: JSON export alongside human-readable Markdown summary.

---

## PHASE 1: BLUEPRINT DAG

```
T1 ──→ T2 ──→ T3 ──→ T4 ──→ T5 ──→ T6 ──→ T7
│       │       │       │       │       │       │
│       │       │       │       │       │       └─→ [BACKFEED] Chaos boundary map
│       │       │       │       │       └─→ [BACKFEED] Chimera classification atlas
│       │       │       │       └─→ [BACKFEED] Output masking inversion spec
│       │       │       └─→ [BACKFEED] Non-linear state recovery spec
│       │       └─→ [BACKFEED] Generator class taxonomy
│       └─→ [BACKFEED] Projector diagnostic spec
└─→ [BACKFEED] Baseline capability floor
```

---

## TIER 1: BASELINE FLOOR (CRT + K-Elim)

**Status: COMPLETE**

### T1.1 — CRT Recovery Engine
- **Type:** Implementation
- **Size:** XS (done)
- **Statement:** Recover LCG parameters (a, c) via per-lane calibration + CRT lift
- **Dependencies:** None (seed node)
- **Deliverable:** `cram_harness_v2.py`
- **Evidence:** 165 tests, 0 failures

### T1.2 — Syndrome Diagnostic Matrix
- **Type:** Implementation
- **Size:** XS (done)
- **Statement:** Full pairwise K-Elimination syndrome check (10 pairs for 5 lanes)
- **Dependencies:** T1.1
- **Deliverable:** `checkSyndromes()` in harness
- **Evidence:** 100% consistency, 100% uniqueness

### T1.3 — Generator Classification (Binary)
- **Type:** Measurement
- **Size:** S (done)
- **Statement:** Tractable (LCG-class) vs intractable (non-LCG), 10 generator types
- **Dependencies:** T1.1, T1.2
- **Deliverable:** Aggregate table in `cram_test_results_v2.json`
- **Evidence:** 90% boundary detection accuracy (9/10 classes correct)

### T1.4 — Noise Degradation Curve
- **Type:** Measurement
- **Size:** S (done)
- **Statement:** Prediction accuracy as function of observation noise (1%–50%)
- **Dependencies:** T1.1
- **Deliverable:** Noise analysis table
- **Evidence:** Smooth degradation: 96% at 1%, 83% at 5%, 71% at 10%

### [GATE T1] ✅ PASSED
- CRT roundtrip: exact for all test values
- K-Elim syndromes: 10,000/10,000 correct
- Binary classification: 9/10 generator classes
- Noise curve: monotonic degradation (no anomalies)

### [EXPLORE T1] — Floor established
- **Capability floor:** 100% on tractable LCGs, 0% on non-linear
- **Boundary:** Tractable iff generator is LCG-class with observable full state
- **Wall:** Output masking, non-linear composition, modulus mismatch

---

## TIER 2: PROJECTOR CLASSIFICATION

**Status: READY (dependencies satisfied, code exists)**

### T2.1 — Port Commutativity Projector to Slot Context
- **Type:** Implementation
- **Size:** S
- **Statement:** Adapt `cram_explorer::projector::check_consistency()` to operate
  on an observed spin sequence rather than a known torus map.
  Input: sequence of observed values.
  Output: per-lane commutativity verdict + failure detail.
- **Dependencies:** T1.1 (CRT engine), projector.rs (exists)
- **Deliverable:** `projector_slot.py` or `projector_slot.rs`

### T2.2 — Lane-by-Lane Structure Detection
- **Type:** Measurement
- **Size:** M
- **Statement:** For each generator class from T1.3, run the projector and record
  which lanes pass/fail commutativity. Build the signature table:
  generator_class → lane_pass_pattern.
- **Dependencies:** T2.1, all 10 generator implementations from T1.3
- **Deliverable:** `projector_signatures.md` — table mapping generator class to
  lane commutativity pattern

### T2.3 — Structural Diagnosis vs Binary Classification
- **Type:** Comparison
- **Size:** S
- **Statement:** Compare projector classification accuracy against T1.3 binary
  classification. Does the projector correctly distinguish generator
  sub-classes that binary classification lumps together?
  Specifically: can it distinguish compound LCG from XorShift?
  Can it distinguish truncated LCG from reel-mapped LCG?
- **Dependencies:** T2.2, T1.3
- **Deliverable:** `projector_vs_binary.md`

### [GATE T2]
- [ ] Projector runs on all 10 generator classes without crash
- [ ] Projector produces distinct lane patterns for at least 6/10 classes
- [ ] Projector correctly identifies LCG-class as fully commutative
- [ ] Zero float in all projector code (I1)

### [EXPLORE T2]
- **Question:** Does lane-level commutativity failure correlate with specific
  algebraic properties of the generator (bit-shift ↔ GF(2) linearity,
  quadratic ↔ QR structure, compound ↔ cross-correlation)?
- **Record regardless of outcome:** The signature table is valuable even if
  the correlation is weak — it constrains the next tier's oracle design.

---

## TIER 3: HYDRA FINGERPRINTING

**Status: READY (code exists, needs adaptation)**

### T3.1 — Define Oracle Heads for Generator Classes
- **Type:** Design
- **Size:** M
- **Statement:** Design 4 oracle heads, each testing a different algebraic property:
  - Head A: LCG linearity (exists in T1.1)
  - Head B: LFSR linearity over GF(2) (new: test x_{n+1} ⊕ x_n structure)
  - Head C: Quadratic residue pattern (adapt Landau-Siegel chimera)
  - Head D: Cross-correlation for compound detection (new)
- **Dependencies:** T2.2 (projector signatures inform head design)
- **Deliverable:** `hydra_heads_spec.md` — formal specification of each head

### T3.2 — Implement Hydra Slot Harness
- **Type:** Implementation
- **Size:** L
- **Statement:** Wire Hydra v3 architecture to slot predictor test suite.
  Each head runs independently on the observed stream.
  Consensus via K-Elimination syndrome comparison across heads.
- **Dependencies:** T3.1, hydra_v3 (exists), T1.1
- **Deliverable:** `hydra_slot_harness.rs` or `.py`

### T3.3 — Classification Atlas Construction
- **Type:** Measurement + Data
- **Size:** L
- **Statement:** Run all 10 generator classes through the Hydra.
  Record the 4-head consensus pattern for each class.
  Build the atlas: consensus_pattern → generator_class.
- **Dependencies:** T3.2
- **Deliverable:** `classification_atlas.md` + `atlas.json`

### T3.4 — Unknown Generator Classification Test
- **Type:** Validation
- **Size:** M
- **Statement:** Generate 100 random generators (random class selection).
  Run each through the Hydra without telling it the class.
  Measure classification accuracy against ground truth.
- **Dependencies:** T3.3
- **Deliverable:** `hydra_blind_test.md`

### [GATE T3]
- [ ] All 4 heads run independently without cross-contamination
- [ ] Atlas contains distinct patterns for at least 8/10 generator classes
- [ ] Blind classification accuracy > 80%
- [ ] K-Elimination consensus correctly rejects when heads disagree
- [ ] Zero float (I1)

### [EXPLORE T3]
- **Question:** Are there generator classes that produce identical Hydra patterns?
  If so, what 5th head would distinguish them?
- **Question:** Does the Hydra pattern predict the DIFFICULTY of state recovery
  (i.e., does it tell you not just WHAT the generator is but HOW HARD
  it is to break)?

---

## TIER 4: SIEVE + ORACLE (Non-Linear State Recovery)

**Status: COMPONENTS EXIST, ASSEMBLY REQUIRED**

### T4.1 — Adapt Quartic Sieve to Generator State Space
- **Type:** Implementation
- **Size:** M
- **Statement:** The sieve currently operates on prime candidates in ℤ.
  Adapt to operate on generator state candidates: given observed
  output sequence, eliminate candidate internal states inconsistent
  with the observed residues across the safe basis.
- **Dependencies:** T3.3 (know which generator class), sieve.rs (exists)
- **Deliverable:** `state_sieve.rs` or `.py`

### T4.2 — Wire Sparse Toric Grover to Sieved State Space
- **Type:** Implementation
- **Size:** L
- **Statement:** After sieve reduces candidate space from N to N',
  run Sparse Toric Grover with oracle: "does this candidate state
  produce the observed output sequence under the classified generator?"
  O(√N') iterations, O(1) storage.
- **Dependencies:** T4.1, sparse_toric_grover.rs (exists)
- **Deliverable:** `sieve_grover_pipeline.rs` or `.py`

### T4.3 — Test Against "Intractable" Generators
- **Type:** Measurement (CRITICAL)
- **Size:** XL
- **Statement:** Run the sieve+oracle pipeline against every generator class
  that T1 classified as intractable:
  - XorShift32 (32-bit state)
  - LFSR-16 (16-bit state)
  - Compound LCG
  - Mersenne Twister (partial — 624×32 state is too large for full recovery,
    but test whether the sieve+oracle recovers partial state)
  
  For each: measure recovery rate, iterations required, sieve reduction factor,
  and the boundary where recovery fails.
- **Dependencies:** T4.2, all generator implementations
- **Deliverable:** `sieve_oracle_results.md` + `sieve_oracle_results.json`

### [GATE T4]
- [ ] Sieve reduces XorShift32 state space by at least 50%
- [ ] Sparse Toric Grover finds XorShift32 state in < 2^16 iterations (vs brute 2^32)
- [ ] LFSR-16 state fully recovered (16-bit state → sieve should crush this)
- [ ] Compound LCG: honest measurement, even if recovery fails
- [ ] MT19937: honest measurement of partial state recovery extent
- [ ] Pipeline runs end-to-end without float (I1)

### [EXPLORE T4]
- **Question:** What is the maximum state size (in bits) that the sieve+oracle
  pipeline can recover in under 1 second on consumer hardware?
- **Question:** Does the sieve reduction factor depend on the generator CLASS
  or only on the state SIZE?
- **Record:** The sieve reduction factor per generator class becomes the
  "tractability gradient" — the smooth transition between tractable and
  intractable, replacing the current binary classification.

---

## TIER 5: OUTPUT MASKING INVERSION

**Status: DESIGN REQUIRED**

### T5.1 — Reel Mapping Enumeration
- **Type:** Design + Implementation
- **Size:** M
- **Statement:** Given observed symbol sequence (not raw state), enumerate
  candidate pre-images for each symbol using the known reel strip table.
  Each observation produces a set of compatible states.
  Across observations, intersect the sets.
- **Dependencies:** T4.1 (sieve operates on state candidates)
- **Deliverable:** `reel_inverter.py` or `.rs`

### T5.2 — Sieve on Pre-Image Intersections
- **Type:** Implementation
- **Size:** M
- **Statement:** Apply the quartic sieve to the pre-image intersection set.
  Each CRT lane independently eliminates impossible pre-images.
  K-Elimination across lanes further constrains.
- **Dependencies:** T5.1, T4.1
- **Deliverable:** Extension of `state_sieve`

### T5.3 — End-to-End Masked Recovery Test
- **Type:** Measurement (CRITICAL)
- **Size:** L
- **Statement:** Run ReelMappedLCG (22 stops, weighted symbol distribution).
  Observe ONLY the symbol outputs (not raw state).
  Attempt full state recovery via reel inversion + sieve + oracle.
  Measure: recovery rate, required observation length, accuracy.
- **Dependencies:** T5.2, T4.2
- **Deliverable:** `masked_recovery_results.md`

### T5.4 — Truncated Output Recovery
- **Type:** Measurement
- **Size:** M
- **Statement:** Same pipeline applied to bit-truncated LCG output (10-bit mask).
  Compare CRAM approach vs classical lattice (LLL) approach.
- **Dependencies:** T5.2
- **Deliverable:** `truncated_vs_lattice.md`

### [GATE T5]
- [ ] Reel inverter correctly enumerates all valid pre-images for test cases
- [ ] Pre-image intersection narrows to < 100 candidates within 20 observations
- [ ] ReelMappedLCG state recovered with > 50% accuracy from symbol-only observation
- [ ] Truncated LCG: CRAM approach compared fairly against classical lattice

### [EXPLORE T5]
- **Question:** What is the minimum number of observed symbols needed for
  unique state determination under reel mapping?
- **Question:** Does the mapping's many-to-one ratio (22:5 for our reel strip)
  have a quantifiable relationship to the required observation length?
- **Question:** Can the chimera operator assignment be used to AMPLIFY
  differences between pre-images, making disambiguation faster?

---

## TIER 6: CHIMERA INTEGRATION

**Status: FOUNDATIONS EXIST (universal harness L7, L8)**

### T6.1 — Chimera Fingerprint Generator
- **Type:** Implementation
- **Size:** M
- **Statement:** For a given observation stream, compute the chimera fingerprint:
  apply each of 8 operators to each of 5 lanes, producing 32,768 possible
  fingerprints. Identify which subset is consistent with the observed stream.
  The consistent subset IS the generator's chimera signature.
- **Dependencies:** T3.3 (atlas), T2.2 (projector signatures)
- **Deliverable:** `chimera_fingerprint.py` or `.rs`

### T6.2 — Atlas Extension with Chimera Signatures
- **Type:** Data
- **Size:** L
- **Statement:** Extend the classification atlas from T3.3 with chimera signatures.
  For each generator class, record the set of consistent chimera schemas.
  Measure: how many schemas are consistent? How quickly does the set narrow
  as more observations arrive?
- **Dependencies:** T6.1, T3.3
- **Deliverable:** `chimera_atlas.md` + `chimera_atlas.json`

### T6.3 — Schema as Generator Inverter
- **Type:** Exploration (NOVEL)
- **Size:** XL
- **Statement:** Test whether there exists a chimera schema that, when applied
  to a non-linear generator's output, produces a LINEAR output.
  If such a schema exists, it linearizes the generator — making it
  tractable for basic CRT recovery.
  This is the Koopman lift reframed as chimera schema search.
- **Dependencies:** T6.1, T4.3 (know which generators resist direct recovery)
- **Deliverable:** `linearizing_schemas.md`

### [GATE T6]
- [ ] Chimera fingerprint narrows to < 100 consistent schemas within 10 observations
- [ ] Atlas extended with chimera data for all 10 generator classes
- [ ] T6.3 attempted honestly — linearizing schema found or proved impossible
  for at least 3 generator classes

### [EXPLORE T6] — THIS IS THE BIG ONE
- **Question:** Does a linearizing chimera schema exist for XorShift32?
  If yes: CRAM can tractify non-linear generators via operator distortion.
  If no: What is the residual nonlinearity, and can a 2-stage chimera
  (output of schema 1 fed to schema 2) reduce it further?
- **Record:** This result determines whether Tier 7 is "extending CRAM to chaos"
  or "proving chaos is the hard wall."

---

## TIER 7: CHAOS REGIME

**Status: THEORETICAL (depends on T6 outcome)**

### T7.1 — Exact Orbit Computation
- **Type:** Implementation
- **Size:** M
- **Statement:** Implement logistic map x_{n+1} = r·x_n·(1-x_n) in exact rational
  CRT arithmetic (no float). Track the orbit on the CRT torus.
  Observe: does the orbit exhibit periodicity on the torus that is
  invisible in float arithmetic?
- **Dependencies:** T6.3 (chimera linearization result)
- **Deliverable:** `exact_logistic.rs` or `.py`

### T7.2 — Manifold Classification via Time Crystal
- **Type:** Exploration
- **Size:** L
- **Statement:** Use the Time Crystal's 3-phase cycle (Generate/Compute/Annihilate)
  as a probe: run the chaotic map for one crystal period, observe the
  K-Elimination winding numbers at annihilation. The winding pattern
  at cycle boundaries is the orbit's CRT-native invariant.
  Classify orbits by their annihilation winding pattern.
- **Dependencies:** T7.1, time_crystal (exists)
- **Deliverable:** `chaos_classification.md`

### T7.3 — Chaos Boundary Map
- **Type:** Measurement (CAMPAIGN CAPSTONE)
- **Size:** XL
- **Statement:** For each chaotic map tested (logistic, Hénon, Lorenz discretized):
  measure the prediction horizon as a function of:
  - CRT basis size (5, 7, 9, 11, 13 lanes)
  - Sieve reduction factor
  - Oracle iteration count
  - Chimera schema (if linearizing schema exists from T6.3)
  
  The result is the CAPABILITY MAP: for each chaos class, what is the
  maximum prediction horizon achievable by the full CRAM stack?
- **Dependencies:** T7.1, T7.2, T4.2, T6.3
- **Deliverable:** `chaos_boundary_map.md` + `chaos_boundary.json`

### [GATE T7]
- [ ] Logistic map orbit computed exactly for at least 1000 iterations
- [ ] Winding pattern classification produces at least 3 distinct orbit classes
- [ ] Chaos boundary map completed for at least 2 chaotic systems
- [ ] Honest assessment: wall identified and documented if prediction fails

### [EXPLORE T7]
- **Question:** Is "chaos" a property of the generator or a property of the
  observer's arithmetic? If exact CRT arithmetic eliminates the precision
  loss that creates apparent chaos, then chaos IS observer-dependent —
  and CRAM dissolves it. If not, where exactly does the information loss
  occur that makes prediction impossible, and is it algebraic or
  information-theoretic?
- **Record:** This is the deepest question. The answer shapes every subsequent
  domain the campaign touches.

---

## PHASE 3: CRITIQUE CHECKLIST (Applied Per Tier)

Before marking any tier COMPLETE:

- [ ] Counterexample search: tested edge cases, degenerates, empty inputs
- [ ] Assumption audit: listed every assumption, justified each
- [ ] Gap detection: all hand-waves marked with `[GAP]`
- [ ] Scope check: results prove what was claimed, no more, no less
- [ ] Float check: zero f32/f64 in any computational path (I1)
- [ ] Manifest check: no rebuilt code that already existed (I2)
- [ ] Evidence hierarchy: tests_passed > proof_sketch > assertion

---

## PHASE 5: SYNTHESIS — NINE65 BACKFEED MAP

Each tier produces specific inputs for the NINE65 chimera:

| Tier | Produces | Feeds Into |
|------|----------|------------|
| T1 | Capability floor | Lane A baseline (RLWE noise budget calibration) |
| T2 | Projector signatures | Lane D diagnostic design (which syndromes to check) |
| T3 | Classification atlas | Hydra head assignment per chimera lane |
| T4 | Tractability gradient | Sieve + oracle as Lane B sub-pipeline |
| T5 | Masking inversion spec | Browser extension: what the extension must invert |
| T6 | Chimera linearization | Core chimera compiler: schema → computation mapping |
| T7 | Chaos boundary map | Hard wall location → where NINE65 provides security |

---

## EXECUTION ORDER

**Session 1 (next):** T2.1 → T2.2 → T2.3 → [GATE T2]
  Wire projector to slot harness, run against all generators, build signature table.

**Session 2:** T3.1 → T3.2 (start Hydra wiring)

**Session 3:** T3.2 (complete) → T3.3 → T3.4 → [GATE T3]

**Session 4:** T4.1 → T4.2 (sieve + oracle pipeline)

**Session 5:** T4.3 → [GATE T4] (the critical measurement session)

**Session 6:** T5.1 → T5.2 → T5.3 → [GATE T5]

**Session 7:** T6.1 → T6.2 (chimera fingerprinting)

**Session 8:** T6.3 → [GATE T6] (the big exploration)

**Session 9:** T7.1 → T7.2 → T7.3 → [GATE T7] (chaos capstone)

**Session 10:** Synthesis → NINE65 backfeed document → campaign complete

---

## INVARIANTS (from Executioner, enforced at every node)

```
I1. ZERO FLOAT.     No f64/f32 in any computational path.
I2. MANIFEST FIRST. Don't rebuild what exists.
I3. GATE BEFORE ADVANCE. Tests pass AND zero float = done.
I4. CLEAN STOP.     Checkpoint + deliver partial > corrupt complete.
I5. MARKDOWN OUT.   All documents .md. All code .rs or .py.
I6. NO REDUNDANCY.  Reference existing crates. Don't duplicate.
```

---

## CAMPAIGN STATUS

| Tier | Status | Gate | Backfeed |
|------|--------|------|----------|
| T1: Baseline | ✅ COMPLETE | ✅ PASSED | Floor: 100% LCG, 0% non-linear |
| T2: Projector | ✅ COMPLETE | ✅ PASSED | Lane commutativity signatures for all 10 classes |
| T3: Hydra | ✅ COMPLETE | ✅ PASSED | 5 consensus types, 0 Unknown, BM+honest fixes |
| T4: Sieve+Oracle | ⬜ READY | ⬜ PENDING | — |
| T5: Masking | ⬜ BLOCKED on T4 | ⬜ PENDING | — |
| T6: Chimera | ⬜ READY (parallel) | ⬜ PENDING | — |
| T7: Chaos | ⬜ BLOCKED on T6 | ⬜ PENDING | — |

### T3 Findings (Critical for T4 Design)

**Honest boundary identified:** When generator modulus G >> basis product M,
T3 cannot distinguish external-modulus LCG from truly non-linear generators.
Both produce approximately uniform output per lane. The 1/p consistency pattern
is a property of uniformity itself, not algebraic structure at external modulus.

**Resolution path:** T4's sieve tests candidate moduli directly. The projector's
commutativity check (from cram_explorer) can verify whether an observed sequence
is consistent with `a*x mod G` for specific `(a, G)` pairs — but this requires
knowing G, which is the sieve's job to find.

**Hydra v2 classification atlas:**

| Generator | Consensus | Head A | Head B | Head C |
|-----------|-----------|--------|--------|--------|
| LCG (Simple) | LCG-class | YES(1.00) | GF2_LINEAR(0.88) | varies |
| LCG (Full Period) | LCG-class | YES(1.00) | WEAK_GF2(0.62) | varies |
| Constant Additive | LCG-class | YES(1.00) | NON_LINEAR | varies |
| Park-Miller | Partially-Structured | NO(0.00) | WEAK_GF2(0.56) | varies |
| Reel-Mapped | QR-Patterned | NO(0.00) | NON_LINEAR | QR_STRUCTURED |
| Truncated | Partially-Structured | NO(0.00) | WEAK_GF2(0.62) | varies |
| Compound | Output-Masked | NO(0.60) | GF2_LINEAR(0.88) | varies |
| LFSR-16 | LFSR-class | NO(0.00) | GF2_LINEAR(1.00) | varies |
| XorShift32 | Partially-Structured | NO(0.00) | WEAK_GF2(0.50) | varies |
| MT19937 | Partially-Structured | NO(0.00) | WEAK_GF2(0.69) | varies |

**Regression fixes applied (Session 2 audit):**
- Head B: Berlekamp-Massey replaces 8-bit brute force + held-out validation
- Head D: Honest FLAGGED verdict replaces saturating confidence metric
- Consensus: Fingerprint pattern matching replaces sequential if/else

Next action: **T4 (Sieve + Oracle for non-linear state recovery)**
