# Hydra Classification Atlas — T3.3 Output

**4-Head Oracle Fingerprinting of All 10 Generator Classes**

## Summary

- **Generators Tested:** 10
- **Unique Consensus Types:** 3
- **Timestamp:** 2026-04-13T14:33:52.415510

## Consensus Distribution

- **LCG-class:** 3 generators
  - LCG (Simple)
  - LCG (Full Period / Hull-Dobell)
  - Constant Additive (a=1)
- **LFSR-like:** 2 generators
  - LCG (Reel-Mapped Output)
  - LCG (Truncated / Bit-Masked)
- **Unknown:** 5 generators
  - LCG (Park-Miller, M=2^31-1)
  - Compound LCG (Dual-Interleaved)
  - LFSR-16 (Galois)
  - XorShift32
  - Mersenne Twister (MT19937)

## Head Confidence Matrix

Per-generator confidence scores for each head (0.0–1.0):

| Generator | Head A (LCG) | Head B (GF2) | Head C (QR) | Head D (Comp) | Consensus |
|-----------|---|---|---|---|---|
| LCG (Simple)                             |  1.00 |  0.44 |  0.57 |  0.63 | LCG-class            |
| LCG (Full Period / Hull-Dobell)          |  1.00 |  0.50 |  0.17 |  1.00 | LCG-class            |
| Constant Additive (a=1)                  |  1.00 |  0.12 |  0.30 |  0.89 | LCG-class            |
| LCG (Park-Miller, M=2^31-1)              |  0.00 |  0.12 |  0.24 |  0.54 | Unknown              |
| LCG (Reel-Mapped Output)                 |  0.00 |  0.88 |  0.33 |  0.03 | LFSR-like            |
| LCG (Truncated / Bit-Masked)             |  0.00 |  0.75 |  0.21 |  0.99 | LFSR-like            |
| Compound LCG (Dual-Interleaved)          |  0.20 |  0.12 |  0.49 |  0.58 | Unknown              |
| LFSR-16 (Galois)                         |  0.00 |  0.12 |  0.18 |  0.03 | Unknown              |
| XorShift32                               |  0.00 |  0.12 |  0.20 |  0.16 | Unknown              |
| Mersenne Twister (MT19937)               |  0.00 |  0.12 |  0.12 |  0.81 | Unknown              |

## Detailed Results

### Generator 1: LCG (Simple)

**Consensus Type:** LCG-class
**Confidences:** A=1.00, B=0.44, C=0.57, D=0.63
**Max Confidence Head:** A_LCG

**Head A (LCG Linearity):**
- Verdict: YES
- Fully Linear Lanes: 5/5
- Avg Consistency: 100.0%

**Head B (GF(2) Linearity):**
- Verdict: WEAK_GF2
- Linear Bits: 7/16

**Head C (QResidue Pattern):**
- Verdict: QR_NOISE
- Avg Deviation: 0.284

**Head D (Composition):**
- Verdict: MASKED
- Avg Correlation: -0.06
- Correlation Variance: 0.0635

### Generator 2: LCG (Full Period / Hull-Dobell)

**Consensus Type:** LCG-class
**Confidences:** A=1.00, B=0.50, C=0.17, D=1.00
**Max Confidence Head:** A_LCG

**Head A (LCG Linearity):**
- Verdict: YES
- Fully Linear Lanes: 5/5
- Avg Consistency: 100.0%

**Head B (GF(2) Linearity):**
- Verdict: WEAK_GF2
- Linear Bits: 8/16

**Head C (QResidue Pattern):**
- Verdict: QR_SPARSE
- Avg Deviation: 0.084

**Head D (Composition):**
- Verdict: COMPOUND
- Avg Correlation: 0.22
- Correlation Variance: 0.1372

### Generator 3: Constant Additive (a=1)

**Consensus Type:** LCG-class
**Confidences:** A=1.00, B=0.12, C=0.30, D=0.89
**Max Confidence Head:** A_LCG

**Head A (LCG Linearity):**
- Verdict: YES
- Fully Linear Lanes: 5/5
- Avg Consistency: 100.0%

**Head B (GF(2) Linearity):**
- Verdict: NON_LINEAR
- Linear Bits: 2/16

**Head C (QResidue Pattern):**
- Verdict: QR_NOISE
- Avg Deviation: 0.151

**Head D (Composition):**
- Verdict: MASKED
- Avg Correlation: -0.11
- Correlation Variance: 0.0889

### Generator 4: LCG (Park-Miller, M=2^31-1)

**Consensus Type:** Unknown
**Confidences:** A=0.00, B=0.12, C=0.24, D=0.54
**Max Confidence Head:** D_Comp

**Head A (LCG Linearity):**
- Verdict: NO
- Fully Linear Lanes: 0/5
- Avg Consistency: 18.4%

**Head B (GF(2) Linearity):**
- Verdict: NON_LINEAR
- Linear Bits: 2/16

**Head C (QResidue Pattern):**
- Verdict: QR_NOISE
- Avg Deviation: 0.120

**Head D (Composition):**
- Verdict: MASKED
- Avg Correlation: 0.20
- Correlation Variance: 0.0540

### Generator 5: LCG (Reel-Mapped Output)

**Consensus Type:** LFSR-like
**Confidences:** A=0.00, B=0.88, C=0.33, D=0.03
**Max Confidence Head:** B_GF2

**Head A (LCG Linearity):**
- Verdict: NO
- Fully Linear Lanes: 0/5
- Avg Consistency: 26.8%

**Head B (GF(2) Linearity):**
- Verdict: GF2_LINEAR
- Linear Bits: 14/16

**Head C (QResidue Pattern):**
- Verdict: QR_NOISE
- Avg Deviation: 0.164

**Head D (Composition):**
- Verdict: SINGLE
- Avg Correlation: -0.00
- Correlation Variance: 0.0028

### Generator 6: LCG (Truncated / Bit-Masked)

**Consensus Type:** LFSR-like
**Confidences:** A=0.00, B=0.75, C=0.21, D=0.99
**Max Confidence Head:** D_Comp

**Head A (LCG Linearity):**
- Verdict: NO
- Fully Linear Lanes: 0/5
- Avg Consistency: 32.0%

**Head B (GF(2) Linearity):**
- Verdict: GF2_LINEAR
- Linear Bits: 12/16

**Head C (QResidue Pattern):**
- Verdict: QR_NOISE
- Avg Deviation: 0.103

**Head D (Composition):**
- Verdict: MASKED
- Avg Correlation: -0.00
- Correlation Variance: 0.0986

### Generator 7: Compound LCG (Dual-Interleaved)

**Consensus Type:** Unknown
**Confidences:** A=0.20, B=0.12, C=0.49, D=0.58
**Max Confidence Head:** D_Comp

**Head A (LCG Linearity):**
- Verdict: NO
- Fully Linear Lanes: 1/5
- Avg Consistency: 42.0%

**Head B (GF(2) Linearity):**
- Verdict: NON_LINEAR
- Linear Bits: 2/16

**Head C (QResidue Pattern):**
- Verdict: QR_NOISE
- Avg Deviation: 0.246

**Head D (Composition):**
- Verdict: MASKED
- Avg Correlation: -0.05
- Correlation Variance: 0.0583

### Generator 8: LFSR-16 (Galois)

**Consensus Type:** Unknown
**Confidences:** A=0.00, B=0.12, C=0.18, D=0.03
**Max Confidence Head:** C_QR

**Head A (LCG Linearity):**
- Verdict: NO
- Fully Linear Lanes: 0/5
- Avg Consistency: 22.4%

**Head B (GF(2) Linearity):**
- Verdict: NON_LINEAR
- Linear Bits: 2/16

**Head C (QResidue Pattern):**
- Verdict: QR_SPARSE
- Avg Deviation: 0.090

**Head D (Composition):**
- Verdict: SINGLE
- Avg Correlation: 0.26
- Correlation Variance: 0.0034

### Generator 9: XorShift32

**Consensus Type:** Unknown
**Confidences:** A=0.00, B=0.12, C=0.20, D=0.16
**Max Confidence Head:** C_QR

**Head A (LCG Linearity):**
- Verdict: NO
- Fully Linear Lanes: 0/5
- Avg Consistency: 19.0%

**Head B (GF(2) Linearity):**
- Verdict: NON_LINEAR
- Linear Bits: 2/16

**Head C (QResidue Pattern):**
- Verdict: QR_SPARSE
- Avg Deviation: 0.098

**Head D (Composition):**
- Verdict: SINGLE
- Avg Correlation: 0.10
- Correlation Variance: 0.0164

### Generator 10: Mersenne Twister (MT19937)

**Consensus Type:** Unknown
**Confidences:** A=0.00, B=0.12, C=0.12, D=0.81
**Max Confidence Head:** D_Comp

**Head A (LCG Linearity):**
- Verdict: NO
- Fully Linear Lanes: 0/5
- Avg Consistency: 19.6%

**Head B (GF(2) Linearity):**
- Verdict: NON_LINEAR
- Linear Bits: 2/16

**Head C (QResidue Pattern):**
- Verdict: QR_SPARSE
- Avg Deviation: 0.059

**Head D (Composition):**
- Verdict: MASKED
- Avg Correlation: 0.10
- Correlation Variance: 0.0812
