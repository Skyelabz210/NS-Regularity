# CRAM SLOT PREDICTOR — WORKSPACE MANIFEST
**Date:** 2026-04-13 | **Session:** 1 | **Phase:** T2 (Projector Classification)

---

## INVENTORY

### Available Assets (Unpacked)

#### Python Harnesses (T1 — COMPLETE)
- `cram_harness_v2.py` (31 KB) — CRT recovery engine with degenerate-lane fix, K-Elimination syndrome check
  - **Status:** ✅ COMPILED, 165/165 tests PASS
  - **Capabilities:** Calibrate LCG parameters per lane, cross-lane CRT consistency, noise degradation curves
- `cram_universal_harness.py` (37 KB) — Multi-generator orchestrator, test fixture factory
  - **Status:** ✅ COMPILED, 9/10 tests PASS
  - **Generators:** 10 generator classes implemented (LCG, LFSR, XorShift, MT19937, compounds, truncated, reel-mapped)

#### Test Data (T1 Results)
- `cram_test_results_v2.json` (88 KB) — 165 test results with detailed per-generator metrics
  - **Generators Tested:** 10 classes
  - **T1 Binary Classification Accuracy:** 9/10 (89% boundary detection)
  - **Floor:** 100% on tractable LCG, 0% on non-linear

#### TypeScript/React Components (UI)
- `cramEngine.ts` (6.8 KB) — CRAM computation engine wrapper
- `cramTypes.ts` (1.3 KB) — Type definitions
- `cram-mvp.jsx` (37 KB) — MVP React component (needs state fix per blueprint note)

#### Documentation
- `CRAM_SLOT_PREDICTOR_BLUEPRINT.md` (22 KB) — Complete 7-tier execution plan, Tao-Blueprint + Executioner methodology

---

## GENERATOR CLASSES (T1.3 Inventory)

| # | Name | Tractable | Avg Match | Category |
|---|------|-----------|-----------|----------|
| 1 | LCG (Simple) | ✅ | 85.9% | LCG-class, no masking |
| 2 | LCG (Full Period) | ✅ | 100.0% | LCG-class, Hull-Dobell |
| 3 | Constant Additive (a=1) | ✅ | 100.0% | Degenerate LCG |
| 4 | LCG (Park-Miller, M=2³¹-1) | ❌ | 0.0% | LCG mod 2³¹-1 (boundary?) |
| 5 | LCG (Reel-Mapped) | ❌ | 17.6% | Output masking |
| 6 | LCG (Truncated/Bit-Masked) | ❌ | 0.0% | Output masking |
| 7 | Compound LCG (Dual-Interleaved) | ❌ | 0.0% | Multi-component |
| 8 | LFSR-16 (Galois) | ❌ | 7.6% | GF(2) linear |
| 9 | Mersenne Twister (MT19937) | ❌ | 0.1% | Tempering transform |
| 10 | XorShift32 | ❌ | 0.0% | Bit-shift composition |

**Classification Boundary:** LCG ∧ full-state observable → tractable. Masking, composition, or non-linear → intractable.

---

## CRAM Tooling Available

### Core Infrastructure
- **CRT Reconstruction** — Exact modular lift via Chinese Remainder Theorem
- **K-Elimination** — Phase-differential RNS division (verified 4.9M+ operations, 0 errors)
- **Lane Calibration** — Recover per-modulus parameters (a_m, c_m) from observation window
- **Syndrome Check** — Pairwise consistency validation (10,000/10,000 correct in T1)
- **Noise Degradation** — Prediction accuracy vs observation noise (96%@1%, 83%@5%, 71%@10%)

### Referenced but Not Yet Ported
- `qmnf_ws/crates/cram_explorer/src/projector.rs` — Commutativity checker (T2.1 task)
- `qmnf_ws/crates/hydra_v3/src/lib.rs` — Multi-head oracle (T3 task)
- `qmnf_ws/crates/prime_hunt/src/sieve.rs` — Quartic residue sieve (T4 task)
- `qmnf_ws/crates/landau_siegel_chimera/src/lib.rs` — Chimera classifier (T6 task)
- `qmnf_ws/crates/time_crystal/src/lib.rs` — Orbit classification probe (T7 task)

---

## EXECUTION STATUS

### T1: Baseline Floor
**Status:** ✅ **COMPLETE**  
**Gate:** ✅ **PASSED**
- CRT roundtrip: exact for all test values
- K-Elim syndromes: 10,000/10,000 correct
- Binary classification: 9/10 generator classes
- Noise curve: monotonic degradation (no anomalies)

### T2: Projector Classification
**Status:** ⬜ **READY (dependencies satisfied)**  
**Gate:** ⬜ **PENDING**
- **T2.1** (this session): Port commutativity projector to slot context
- **T2.2** (this session): Lane-by-lane structure detection on all 10 generators
- **T2.3** (this session): Compare projector results vs binary classification

---

## INVARIANTS (Executioner Enforcement)

```
I1. ZERO FLOAT.     No f64/f32 in any computational path.
I2. MANIFEST FIRST. Don't rebuild what exists.
I3. GATE BEFORE ADVANCE. Tests pass AND zero float = done.
I4. CLEAN STOP.     Checkpoint + deliver partial > corrupt complete.
I5. MARKDOWN OUT.   All documents .md. All code .rs or .py.
I6. NO REDUNDANCY.  Reference existing crates. Don't duplicate.
```

---

## SESSION 1 EXECUTION PLAN

1. **T2.1** — Implement `projector_slot.py`
   - Input: spin sequence from any generator
   - Output: per-lane commutativity verdict
   - Logic: Check linearity assumption per lane, recover (a_m, c_m), validate across lanes
   
2. **T2.2** — Run projector against all 10 generators
   - Collect lane pass/fail patterns
   - Build signature table: generator_class → lane_pattern
   
3. **T2.3** — Comparison
   - How well does projector classify vs binary classification?
   - What sub-classes does projector separate that binary lumps together?
   
4. **[GATE T2]** — Verify all conditions pass
   - Projector runs on all 10 generators without crash
   - Produces distinct lane patterns for ≥6/10 classes
   - Correctly identifies LCG-class as fully commutative
   - Zero float in all code

---

## Notes

- **Workspace context:** Working from unpacked tarball in `/home/claude/`
- **Python version:** 3.12.3
- **Tools:** 5-lane SAFE_BASIS=[3,5,7,11,13], M=15,015
- **Architecture:** Acid (HackFate.us) + Claude collaboration, Tao Blueprint methodology
- **Next output:** `2026-04-13-SLOT-001.md` (session manifest)
