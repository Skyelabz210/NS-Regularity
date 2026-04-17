# SESSION MANIFEST — April 2, 2026
## QMNF Unified v2.5

### Kill Count: 127 → 127 (no new kills this session — structural discovery, not innovation)

### Tests: 641 passed, 0 failed, 23 crates

### Key Discovery: The Second-Order Arrow of Time

The K-Elim winding asymmetry k(7→11) ≠ k(11→7) has period 15 = lcm(3,5) on the backbone torus T₇ × T₁₁. The CHANGE in this arrow between consecutive primes (delta_arrow) has only 8 distinct values out of 77 possible and distinguishes gap families:

| delta_arrow | Cousin frac | Twin frac | Sexy frac | Role |
|------------|-------------|-----------|-----------|------|
| 76 (=-1) | **54.7%** | 39.8% | **0.0%** | Correction (reset) |
| 0 | 13.1% | 23.5% | 24.0% | Signal (unchanged) |
| 1 | 13.4% | 5.6% | 23.3% | Drift (smooth) |
| 4 | **never** | never | never | Invisible (backbone) |

**TVD(twin_delta, cousin_delta) = 0.3123** — distinguishable.

### Oracle Performance (validated on [50K, 200K))
- **Cousin detection** at delta=76: Precision 52.3%, Recall 13.9%, F1 0.220
- **Sexy exclusion** at delta=76: 100.0% accuracy (zero sexys ever appear)
- **Multi-delta classifier**: 60.3% overall (vs 25% random baseline)

### Structural Findings

1. **b-injection = [2, 1, 4]** on lane 7: twin, unit, cousin. Conservative (sums to 7). The backbone heartbeat encodes the three gap families.

2. **Arrow period = 15**, full chimera period = 30. The arrow lives on T₇ × T₁₁ jointly.

3. **delta=4 (cousin gap) NEVER occurs** as a second-order arrow change. The backbone can't produce its own spacing.

4. **Cousins are at the antipodal position** (76 = -1 mod 77): they're the "last step before reset." 76 + 4 = 80 ≡ 3 mod 77 (ground prime).

5. **Face of zero (143-torus)** has 0 primes at position 0 (structurally empty — 143 = 11×13 is composite). 120/143 states occupied. Cousin-enriched states identified.

6. **Maya calendar correspondence** computationally verified: 11 absent from all 8 cycle factorizations, present as 11² in eclipse displacement 242.

### Honest Negatives This Session
- Simple unwinding (subtract arrow from backbone position): 0.0% variance reduction. The rotation doesn't change the distribution shape.
- Multi-delta classifier misclassifies twins and sexys entirely (0% per-class accuracy). Only classifies "other" and sometimes "cousin."

### White Paper
- **CRAM_DEFINITIVE_WHITE_PAPER.docx** — 198 paragraphs, 11 sections
  - §6 NEW: Spectral Fingerprint Bridge + Permanent Non-Resonance Theorem
  - §7 NEW: MHD Extension
  - All errata corrected (SCALE not SCALE², fit a=0.901)

### Crate Changes
- **ramanujan_memory_lane**: 569 lines, 10 tests. Reviewer bug #2 FIXED (resonance_at instead of delta check). PNR theorem tested. Spectral fingerprint bridge tested. Sub-operator shadow test added.
- **quantum_continuum_torus**: 1,397 lines, 26 tests. Clean rebuild from user's 506-line core. Added: CramOp (8 ops), Schema (heterogeneous 5-lane), SchemaOrbit (trajectory + period), ResolutionLadder (N-refinement with b-injection), FaceOfZero (143-torus tracker), ArrowOfTime (K-Elim asymmetry), WindingCorrector (prime detection oracle).

### Files
- qmnf_unified_v2.5.tar.gz — 23 crates, 641 tests
- CRAM_DEFINITIVE_WHITE_PAPER.docx — definitive edition with spectral bridge
- CRAM_ERRATA_AND_MHD_EXTENSION.docx — standalone corrections + MHD
- SESSION_MANIFEST_APR2.md — this file

### Open Problems Updated
- OP-COUSIN: Can the second-order arrow oracle be improved beyond 52% precision? The signal is real but the recall is low. Multi-feature classifier (delta_arrow + face position + excess) might reach higher F1.
- OP-WINDING-EXCESS: The face-of-zero excess grows quadratically. Does its mod-143 residue cycle predict when gap distributions shift?
- OP-660-STATE: The full chimera orbit (lcm(30 multiplicative, 11 additive) = 330 or 660) hasn't been computed. Does the shadow prime's additive period multiply the orbit?
- OP-MAYA-BRIDGE: The 819-day correction cycle (3²×7×13) is a PARTIAL correction (bridge × boundary). What would the FULL correction cycle (including shadow 11) look like? 819 × 11 = 9009 = 3² × 7 × 11 × 13. Is 9009 days astronomically significant?
