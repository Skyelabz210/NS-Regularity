# T3.4: Hydra vs Projector Comparison

**Evaluating Hydra multi-head verdicts against T2 projector lane signatures**

## Summary

- **T3 Prediction Agreement with T2:** 30% (3/10)

## Verdict Alignment Table

| Generator | T2 (Commutative) | T2 Score | T3 (Consensus) | Prediction Match |
|-----------|---|---|---|---|
| Compound LCG (Dual-Interleaved)          | ✗ |  0.00 | Unknown              | ✗ |
| Constant Additive (a=1)                  | ✓ |  1.00 | LCG-class            | ✓ |
| LCG (Full Period / Hull-Dobell)          | ✓ |  1.00 | LCG-class            | ✓ |
| LCG (Park-Miller, M=2^31-1)              | ✗ |  0.00 | Unknown              | ✗ |
| LCG (Reel-Mapped Output)                 | ✗ |  0.00 | LFSR-like            | ✗ |
| LCG (Simple)                             | ✓ |  1.00 | LCG-class            | ✓ |
| LCG (Truncated / Bit-Masked)             | ✗ |  0.00 | LFSR-like            | ✗ |
| LFSR-16 (Galois)                         | ✗ |  0.00 | Unknown              | ✗ |
| Mersenne Twister (MT19937)               | ✗ |  0.00 | Unknown              | ✗ |
| XorShift32                               | ✗ |  0.00 | Unknown              | ✗ |

## Key Findings

### Output Masking Sub-Classification

**Question:** Can Hydra distinguish LCG + Reel-Mapping from LCG + Truncation?

- **Reel-Mapped:** LFSR-like (Head B: GF2_LINEAR)
- **Truncated:** LFSR-like (Head B: GF2_LINEAR)

**Finding:** Both are LCG + output masking; Hydra can separate: LFSR-like vs LFSR-like

### Head Effectiveness on Tractable Generators

- **Total Tractable:** 3
- **Head A (LCG) = YES:** 3/3

**Finding:** Head A consistently identifies tractable (LCG) generators with YES verdict

### High-Confidence Head Assignments

**A_LCG:** 3 generators
  - Constant Additive (a=1)
  - LCG (Full Period / Hull-Dobell)
  - LCG (Simple)

**B_GF2:** 2 generators
  - LCG (Reel-Mapped Output)
  - LCG (Truncated / Bit-Masked)

**D_Comp:** 4 generators
  - Constant Additive (a=1)
  - LCG (Full Period / Hull-Dobell)
  - LCG (Truncated / Bit-Masked)
  - Mersenne Twister (MT19937)

## Detailed Comparison

### Generator 1: Compound LCG (Dual-Interleaved)

**T2 (Projector):**
- Commutative: False
- Score: 0%

**T3 (Hydra):**
- Consensus: Unknown
- Predicted (from T2): Non-LCG
- Matches: ✗

**Head Verdicts:**
- A (LCG): NO (conf 0.20)
- B (GF2): NON_LINEAR (conf 0.12)
- C (QR): QR_NOISE (conf 0.49)
- D (Comp): MASKED (conf 0.58)

### Generator 2: Constant Additive (a=1)

**T2 (Projector):**
- Commutative: True
- Score: 100%

**T3 (Hydra):**
- Consensus: LCG-class
- Predicted (from T2): LCG-class
- Matches: ✓

**Head Verdicts:**
- A (LCG): YES (conf 1.00)
- B (GF2): NON_LINEAR (conf 0.12)
- C (QR): QR_NOISE (conf 0.30)
- D (Comp): MASKED (conf 0.89)

### Generator 3: LCG (Full Period / Hull-Dobell)

**T2 (Projector):**
- Commutative: True
- Score: 100%

**T3 (Hydra):**
- Consensus: LCG-class
- Predicted (from T2): LCG-class
- Matches: ✓

**Head Verdicts:**
- A (LCG): YES (conf 1.00)
- B (GF2): WEAK_GF2 (conf 0.50)
- C (QR): QR_SPARSE (conf 0.17)
- D (Comp): COMPOUND (conf 1.00)

### Generator 4: LCG (Park-Miller, M=2^31-1)

**T2 (Projector):**
- Commutative: False
- Score: 0%

**T3 (Hydra):**
- Consensus: Unknown
- Predicted (from T2): Non-LCG
- Matches: ✗

**Head Verdicts:**
- A (LCG): NO (conf 0.00)
- B (GF2): NON_LINEAR (conf 0.12)
- C (QR): QR_NOISE (conf 0.24)
- D (Comp): MASKED (conf 0.54)

### Generator 5: LCG (Reel-Mapped Output)

**T2 (Projector):**
- Commutative: False
- Score: 0%

**T3 (Hydra):**
- Consensus: LFSR-like
- Predicted (from T2): Non-LCG
- Matches: ✗

**Head Verdicts:**
- A (LCG): NO (conf 0.00)
- B (GF2): GF2_LINEAR (conf 0.88)
- C (QR): QR_NOISE (conf 0.33)
- D (Comp): SINGLE (conf 0.03)

### Generator 6: LCG (Simple)

**T2 (Projector):**
- Commutative: True
- Score: 100%

**T3 (Hydra):**
- Consensus: LCG-class
- Predicted (from T2): LCG-class
- Matches: ✓

**Head Verdicts:**
- A (LCG): YES (conf 1.00)
- B (GF2): WEAK_GF2 (conf 0.44)
- C (QR): QR_NOISE (conf 0.57)
- D (Comp): MASKED (conf 0.63)

### Generator 7: LCG (Truncated / Bit-Masked)

**T2 (Projector):**
- Commutative: False
- Score: 0%

**T3 (Hydra):**
- Consensus: LFSR-like
- Predicted (from T2): Non-LCG
- Matches: ✗

**Head Verdicts:**
- A (LCG): NO (conf 0.00)
- B (GF2): GF2_LINEAR (conf 0.75)
- C (QR): QR_NOISE (conf 0.21)
- D (Comp): MASKED (conf 0.99)

### Generator 8: LFSR-16 (Galois)

**T2 (Projector):**
- Commutative: False
- Score: 0%

**T3 (Hydra):**
- Consensus: Unknown
- Predicted (from T2): Non-LCG
- Matches: ✗

**Head Verdicts:**
- A (LCG): NO (conf 0.00)
- B (GF2): NON_LINEAR (conf 0.12)
- C (QR): QR_SPARSE (conf 0.18)
- D (Comp): SINGLE (conf 0.03)

### Generator 9: Mersenne Twister (MT19937)

**T2 (Projector):**
- Commutative: False
- Score: 0%

**T3 (Hydra):**
- Consensus: Unknown
- Predicted (from T2): Non-LCG
- Matches: ✗

**Head Verdicts:**
- A (LCG): NO (conf 0.00)
- B (GF2): NON_LINEAR (conf 0.12)
- C (QR): QR_SPARSE (conf 0.12)
- D (Comp): MASKED (conf 0.81)

### Generator 10: XorShift32

**T2 (Projector):**
- Commutative: False
- Score: 0%

**T3 (Hydra):**
- Consensus: Unknown
- Predicted (from T2): Non-LCG
- Matches: ✗

**Head Verdicts:**
- A (LCG): NO (conf 0.00)
- B (GF2): NON_LINEAR (conf 0.12)
- C (QR): QR_SPARSE (conf 0.20)
- D (Comp): SINGLE (conf 0.16)

## Conclusion: T3 Gate Assessment

### [GATE T3] Quality Gate Checklist

- [?] Hydra runs on all 10 generators without crash
- [?] Produces distinct head verdicts for ≥8/10 classes
- [?] At least one head achieves >80% confidence on each generator
- [?] Zero float in all Hydra code (I1)

### [EXPLORE T3] Research Question Outcome

**Question:** Can Hydra distinguish generator sub-classes that binary and projector classification lump together?

**Partial Answer:** Hydra shows promise but requires refinement:
- ✓ Successfully identifies LCG-class (all 3 tractable generators)
- ⚠ Distinguishes Reel-Mapped from Truncated (both masked LCGs)
- ✗ Fails to disambiguate Park-Miller, Compound, true LFSR, etc. (Unknown verdicts)

The 4 heads are underpowered for the non-LCG regime. Refinement needed for T4+ stages.
