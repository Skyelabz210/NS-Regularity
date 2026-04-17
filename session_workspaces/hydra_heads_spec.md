# T3.1: Hydra Oracle Heads Specification
## Multi-Head Fingerprinting for Generator Classification

**Date:** April 13, 2026 | **Author:** Acid + Claude  
**Objective:** Design 4 independent oracle heads to detect algebraic properties of observed generators.

---

## OVERVIEW

The Hydra system uses 4 parallel oracle heads, each testing a distinct algebraic property:

```
Observed Spin Stream
       ↓
   ┌───┴───┬────────┬────────┬────────┐
   ↓       ↓        ↓        ↓        ↓
Head A   Head B   Head C   Head D
(LCG)   (GF(2)) (QResidue)(Xcorr)
   ↓       ↓        ↓        ↓        ↓
   └───┬───┴────────┴────────┴────────┘
       ↓
   Consensus Verdict
       ↓
Generator Class
```

Each head outputs:
- **Verdict:** Boolean or categorical (yes/no, type A/B/C, etc.)
- **Confidence:** 0.0–1.0 (strength of signal)
- **Details:** Per-lane or per-modulus breakdown

---

## HEAD A: LCG LINEARITY DETECTOR

### Purpose
Detect whether generator is LCG-class (single linear recurrence modulo M).

### Specification

**Input:**
- Spin sequence (≥100 values)
- Lane moduli (default: [3, 5, 7, 11, 13])
- Calibration window (default: 20)

**Algorithm:**
1. For each modulus m in basis:
   - Extract lane: spins[i] % m
   - Fit linear recurrence: x_{n+1} = a·x_n + c (mod m)
   - Via diff-pair solve (existing from T1.1)
2. Aggregate lane results:
   - How many lanes are fully linear (consistency = 100%)?
   - Consistency = (correct_predictions / total_predictions)
3. Compute verdict:
   - Fully linear (all 5 lanes at 100%) → **LCG: YES**
   - Any lane < 100% → **LCG: NO**

**Output:**
```json
{
  "head": "A_LCG_Linearity",
  "verdict": "YES" | "NO",
  "confidence": 0.0–1.0,
  "fully_commutative_lanes": int,
  "total_lanes": int,
  "avg_consistency": float,
  "details": {
    "lane_0": {"modulus": 3, "is_linear": bool, "consistency": float},
    "lane_1": {"modulus": 5, "is_linear": bool, "consistency": float},
    ...
  }
}
```

**Confidence Formula:**
```
confidence = (fully_linear_lanes / total_lanes)
```

**Generator Matches:**
- ✓ LCG (Simple), LCG (Full Period), Constant Additive → verdict = YES, confidence = 1.0
- ✗ All others → verdict = NO, confidence = 0.0

**Implementation:** Reuse `calibrate_lane()` and consistency check from T1.1 + T2.1.

---

## HEAD B: GF(2)-LINEAR (LFSR) DETECTOR

### Purpose
Detect whether generator exhibits linear feedback shift register (GF(2)-linear) behavior.

### Specification

**Input:**
- Spin sequence (≥100 values)
- Bit width (default: 16 for typical LFSRs; may vary)

**Algorithm:**
1. **Hypothesis:** Output follows x_{n+1} = (x_n >> 1) | ((bit_0(x_n) ⊕ bit_k(x_n)) << (width-1))
   - This is the Galois LFSR structure
2. **Test for linear recurrence in GF(2):**
   - Represent each spin as bit vector b[0..15]
   - For each position i in [0..15], test: bit_i(x_{n+1}) = f_i(x_n) where f_i is XOR of some bits of x_n
   - Scan all 2^16 possible feedback vectors for those matching > threshold
3. **Per-lane GF(2) linearity (bonus):**
   - For each modulus m, extract lane values [s % m for s in spins]
   - Treat lane as GF(2) vector if m is power of 2 or small prime
   - Measure linear structure in GF(2) subspace
4. **Confidence metric:**
   - Measure fraction of bit positions where linear recurrence holds
   - Count matching feedback polynomials

**Output:**
```json
{
  "head": "B_GF2_Linear",
  "verdict": "GF2_LINEAR" | "WEAK_GF2" | "NON_LINEAR",
  "confidence": 0.0–1.0,
  "linear_bits": int,
  "total_bits": int,
  "feedback_matches": int,
  "best_feedback_polynomial": "hex string" | null,
  "details": {
    "bit_linearity": [0.0–1.0 per bit],
    "max_feedback_score": float
  }
}
```

**Confidence Formula:**
```
confidence = (linear_bits / total_bits) if verdict == GF2_LINEAR
           = 0.0 otherwise
```

**Generator Matches:**
- ✓ LFSR-16 (Galois) → verdict = GF2_LINEAR, confidence ≥ 0.8 (high)
- ⚠ Compound LCG → verdict = WEAK_GF2, confidence ≈ 0.3–0.5 (some bits linear, some not)
- ✗ All others → verdict = NON_LINEAR, confidence < 0.3

**Implementation:**
- Bit extraction: `(x >> i) & 1`
- XOR operations: `^` operator
- Feedback polynomial search: test all 2^16 or use heuristic (scan for common LFSR polys)

---

## HEAD C: QUADRATIC RESIDUE PATTERN DETECTOR

### Purpose
Detect whether output distribution follows quadratic residue (QR) structure, indicating non-linear algebraic composition.

### Specification

**Input:**
- Spin sequence (≥100 values)
- Lane moduli (default: [3, 5, 7, 11, 13])

**Algorithm:**
1. **Per-modulus QR analysis:**
   - For each modulus m:
     - Compute QR set: {x² mod m : x ∈ [0, m-1]}
     - Count how many observed lane values (s % m) fall in QR set
     - QR_ratio = (qr_count / len(spins))
   - Compare against uniform random expectation: ~0.5 for odd primes
2. **Aggregate QR signal:**
   - Measure deviation from expected QR density
   - High deviation → suggests deliberate QR structure (rare in simple generators)
   - Moderate deviation → possibly incidental
3. **Confidence metric:**
   - Measure chi-square or deviation of observed QR ratio from 0.5
   - Bound to [0.0, 1.0]

**Output:**
```json
{
  "head": "C_QResidue_Pattern",
  "verdict": "QR_STRUCTURED" | "QR_NOISE" | "QR_SPARSE",
  "confidence": 0.0–1.0,
  "details": {
    "modulus_3": {"qr_ratio": float, "deviation": float},
    "modulus_5": {"qr_ratio": float, "deviation": float},
    "modulus_7": {"qr_ratio": float, "deviation": float},
    "modulus_11": {"qr_ratio": float, "deviation": float},
    "modulus_13": {"qr_ratio": float, "deviation": float}
  },
  "avg_deviation": float
}
```

**Confidence Formula:**
```
deviation_per_modulus = |qr_ratio - 0.5|
avg_deviation = mean(deviations)
confidence = min(avg_deviation * 2, 1.0)  # 0.5 max deviation → 1.0 confidence
```

**Generator Matches:**
- ✓ Mersenne Twister (tempered, designed to spread bits) → confidence ≈ 0.4–0.6 (moderate QR noise)
- ⚠ LCG (Reel-Mapped) → confidence ≈ 0.2–0.3 (weak QR structure from masking)
- ✗ Others → confidence < 0.2 (no deliberate QR design)

**Implementation:**
- QR computation: `qr_set = {(x*x) % m for x in range(m)}`
- Count: `qr_count = sum(1 for s in spins if (s % m) in qr_set)`

**Note:** This head is exploratory; not all generators deliberately use QR structure. Used as tie-breaker.

---

## HEAD D: CROSS-CORRELATION / COMPOSITION DETECTOR

### Purpose
Detect compound generators (multiple LCGs or mixed sources) via cross-correlation of successive outputs.

### Specification

**Input:**
- Spin sequence (≥100 values)
- Lane moduli (default: [3, 5, 7, 11, 13])
- Lag (default: 1, testing x_n vs x_{n+1})

**Algorithm:**
1. **Cross-correlation per lane:**
   - For each modulus m:
     - Extract lane: lane = [s % m for s in spins]
     - Compute Pearson correlation between lane[:-1] and lane[1:]
     - For LCG, expect high correlation (linear dependency)
     - For compound (two interleaved LCGs), expect lower or anticorrelation
2. **Variance analysis:**
   - If lanes show high variance in per-lane correlation → indicates different component behaviors
   - Measure: max_corr - min_corr (higher = more composition)
3. **Multi-lag detection:**
   - Test lags 1, 2, 3 to detect shift patterns
   - Compound generators often show repeating lag patterns
4. **Verdict logic:**
   - All lanes highly correlated → single component (likely simple LCG)
   - Mixed correlations (some high, some low) → likely compound
   - Consistent low/negative correlation → possible output masking or XOR composition

**Output:**
```json
{
  "head": "D_Composition",
  "verdict": "COMPOUND" | "SINGLE" | "MASKED",
  "confidence": 0.0–1.0,
  "details": {
    "modulus_3": {"correlation": float, "lag_1": float},
    "modulus_5": {"correlation": float, "lag_1": float},
    "modulus_7": {"correlation": float, "lag_1": float},
    "modulus_11": {"correlation": float, "lag_1": float},
    "modulus_13": {"correlation": float, "lag_1": float}
  },
  "correlation_variance": float,
  "avg_correlation": float
}
```

**Confidence Formula:**
```
correlation_variance = var(per_lane_correlations)
confidence = min(correlation_variance * 10, 1.0)  # Tuned to [0, 0.1] typical range
```

**Generator Matches:**
- ✓ Compound LCG (Dual-Interleaved) → high correlation variance, verdict = COMPOUND, confidence ≥ 0.7
- ✓ Single LCG → low variance, verdict = SINGLE, confidence ≥ 0.6
- ⚠ Truncated/Reel-Mapped → verdict = MASKED, confidence ≈ 0.3–0.5

**Implementation:**
```python
from statistics import mean, variance
def pearson_correlation(x, y):
    n = len(x)
    mean_x = sum(x) / n
    mean_y = sum(y) / n
    numerator = sum((x[i] - mean_x) * (y[i] - mean_y) for i in range(n))
    denom_x = sum((x[i] - mean_x)**2 for i in range(n))
    denom_y = sum((y[i] - mean_y)**2 for i in range(n))
    if denom_x == 0 or denom_y == 0:
        return 0.0
    return numerator / (denom_x * denom_y)**0.5
```

---

## CONSENSUS & AGGREGATION

### Multi-Head Voting

After all 4 heads run, aggregate verdicts:

```python
consensus = {
    "all_verdicts": [head_a_verdict, head_b_verdict, head_c_verdict, head_d_verdict],
    "confidence_scores": [head_a_confidence, ..., head_d_confidence],
    "max_confidence_head": head_name_with_highest_confidence,
    "consensus_type": infer_from_verdicts(),
    "tie_breaking": use_highest_confidence_head_verdict
}
```

**Inference rules:**
- If Head A = YES → class is LCG (end)
- Else if Head B = GF2_LINEAR → class is LFSR-like
- Else if Head D = COMPOUND → class is Compound or XOR-composed
- Else if Head C shows QR_STRUCTURED → class is Masking variant
- Else → Unknown or novel class

**Tie-breaking:** When multiple heads have similar confidence, use highest confidence head's verdict.

---

## IMPLEMENTATION NOTES

### Zero Float (I1 Compliance)
All 4 heads use only integer arithmetic:
- **Head A:** Modular arithmetic only
- **Head B:** Bitwise operations
- **Head C:** Integer counting
- **Head D:** Integer correlation (scaled integers, no floating-point division until final confidence)

For Head D correlation, compute using scaled integers:
```python
# Instead of: correlation = numerator / (denom_x * denom_y)**0.5
# Use: correlation_score = (numerator * 1000) // (denom_x * denom_y / 1000)
# Then normalize to [0, 1]
```

### Modularization
- Each head as independent function
- Shared utilities: `decompose()`, `k_elim()`, QR computation
- Single harness orchestrates all 4 in parallel

### Deliverable Files
- `hydra_heads_spec.md` (this document)
- `hydra_slot.py` (implementation, to follow in T3.2)

---

## VALIDATION TARGETS (for T3.2/T3.3)

### Expected Head Signals

| Generator | Head A (LCG) | Head B (GF2) | Head C (QR) | Head D (Comp) |
|-----------|---|---|---|---|
| LCG (Simple) | ✓ YES, 1.0 | NO, ~0.2 | ~0.1 | SINGLE, 0.6 |
| LCG (Full Period) | ✓ YES, 1.0 | NO, ~0.2 | ~0.1 | SINGLE, 0.6 |
| Constant Add | ✓ YES, 1.0 | NO, ~0.2 | ~0.1 | SINGLE, 0.6 |
| Park-Miller | NO, 0.0 | NO, ~0.2 | ~0.2 | MASKED, 0.4 |
| Reel-Mapped | NO, 0.0 | NO, ~0.2 | ⚠ 0.3 | MASKED, 0.5 |
| Truncated | NO, 0.0 | NO, ~0.2 | ~0.1 | MASKED, 0.4 |
| Compound | NO, 0.0 | WEAK, ~0.4 | ~0.1 | ✓ COMPOUND, 0.7 |
| LFSR-16 | NO, 0.0 | ✓ GF2, 0.8+ | ~0.15 | ~0.3 |
| XorShift | NO, 0.0 | WEAK, ~0.3 | ~0.1 | ~0.2 |
| MT19937 | NO, 0.0 | NO, ~0.2 | ⚠ 0.4 | ~0.3 |

**Note:** These are predictions; actual values will be measured in T3.3.

---

## SUMMARY TABLE

| Head | Property | Input | Output | Resolution | Float-Free |
|------|----------|-------|--------|-----------|-----------|
| **A** | LCG linearity | Spin sequence | YES/NO + confidence | Tractable boundary | ✓ |
| **B** | GF(2)-linearity | Spins + bits | GF2_LINEAR/WEAK/NON + conf | LFSR vs others | ✓ |
| **C** | QR pattern | Lane values | QR_STRUCT/NOISE/SPARSE + conf | Masking signature (weak) | ✓ |
| **D** | Composition | Lane correlations | COMPOUND/SINGLE/MASKED + conf | Multi-component detection | ✓ |

---

**This specification is ready for T3.2 implementation.**

Next: Code the harness and run all 10 generators through the Hydra oracle bank.
