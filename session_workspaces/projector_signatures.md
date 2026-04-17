# Projector Signatures — Lane-by-Lane Structure Detection

**T2.2 Output: Generator Class → Lane Commutativity Pattern**

## Summary

- **Fully Commutative:** 3/10 generators
- **Avg Commutativity Score:** 30.0%
- **Range:** 0.0% – 100.0%

## Signature Table

| # | Generator | Commutative | Score | Lane Pattern (Mod 3,5,7,11,13) |
|---|-----------|-------------|-------|--------------------------------|
|  1 | LCG (Simple)                             | ✅ YES      | 100%   | ✓3, ✓5, ✓7, ✓11, ✓13 |
|  2 | LCG (Full Period / Hull-Dobell)          | ✅ YES      | 100%   | ✓3, ✓5, ✓7, ✓11, ✓13 |
|  3 | Constant Additive (a=1)                  | ✅ YES      | 100%   | ✓3, ✓5, ✓7, ✓11, ✓13 |
|  4 | LCG (Park-Miller, M=2^31-1)              | ❌ NO       | 0%     | ✗3(34%), ✗5(20%), ✗7(18%), ✗11(10%), ✗13(11%) |
|  5 | LCG (Reel-Mapped Output)                 | ❌ NO       | 0%     | ✗3(33%), ✗5(14%), ✗7(14%), ✗11(14%), ✗13(14%) |
|  6 | LCG (Truncated / Bit-Masked)             | ❌ NO       | 0%     | ✗3(21%), ✗5(17%), ✗7(17%), ✗11(17%), ✗13(9%) |
|  7 | Compound LCG (Dual-Interleaved)          | ❌ NO       | 0%     | ✗3(33%), ✗5(50%), ✗7(66%), ✗11(9%), ✗13(17%) |
|  8 | LFSR-16 (Galois)                         | ❌ NO       | 0%     | ✗3(62%), ✗5(65%), ✗7(57%), ✗11(51%), ✗13(51%) |
|  9 | XorShift32                               | ❌ NO       | 0%     | ✗3(40%), ✗5(21%), ✗7(13%), ✗11(6%), ✗13(10%) |
| 10 | Mersenne Twister (MT19937)               | ❌ NO       | 0%     | ✗3(39%), ✗5(15%), ✗7(14%), ✗11(8%), ✗13(11%) |

## Detailed Lane Breakdown

### Generator 1: LCG (Simple)

**Commutativity: 100.0% (5/5 lanes)**

- **Lane 0 (mod 3):** ✓ LINEAR
  - Recovered: a=1, c=2
  - Consistency: 100.0%
- **Lane 1 (mod 5):** ✓ LINEAR
  - Recovered: a=2, c=1
  - Consistency: 100.0%
- **Lane 2 (mod 7):** ✓ LINEAR
  - Recovered: a=6, c=1
  - Consistency: 100.0%
- **Lane 3 (mod 11):** ✓ LINEAR
  - Recovered: a=8, c=6
  - Consistency: 100.0%
- **Lane 4 (mod 13):** ✓ LINEAR
  - Recovered: a=12, c=10
  - Consistency: 100.0%

### Generator 2: LCG (Full Period / Hull-Dobell)

**Commutativity: 100.0% (5/5 lanes)**

- **Lane 0 (mod 3):** ✓ LINEAR
  - Recovered: a=1, c=1
  - Consistency: 100.0%
- **Lane 1 (mod 5):** ✓ LINEAR
  - Recovered: a=1, c=1
  - Consistency: 100.0%
- **Lane 2 (mod 7):** ✓ LINEAR
  - Recovered: a=1, c=1
  - Consistency: 100.0%
- **Lane 3 (mod 11):** ✓ LINEAR
  - Recovered: a=1, c=1
  - Consistency: 100.0%
- **Lane 4 (mod 13):** ✓ LINEAR
  - Recovered: a=1, c=1
  - Consistency: 100.0%

### Generator 3: Constant Additive (a=1)

**Commutativity: 100.0% (5/5 lanes)**

- **Lane 0 (mod 3):** ✓ LINEAR
  - Recovered: a=1, c=0
  - Consistency: 100.0%
- **Lane 1 (mod 5):** ✓ LINEAR
  - Recovered: a=1, c=3
  - Consistency: 100.0%
- **Lane 2 (mod 7):** ✓ LINEAR
  - Recovered: a=1, c=2
  - Consistency: 100.0%
- **Lane 3 (mod 11):** ✓ LINEAR
  - Recovered: a=1, c=8
  - Consistency: 100.0%
- **Lane 4 (mod 13):** ✓ LINEAR
  - Recovered: a=1, c=0
  - Consistency: 100.0%

### Generator 4: LCG (Park-Miller, M=2^31-1)

**Commutativity: 0.0% (0/5 lanes)**

- **Lane 0 (mod 3):** ✗ NON-LINEAR
  - Recovered: a=0, c=0
  - Consistency: 34.2%
  - First failure at transition 3→4
- **Lane 1 (mod 5):** ✗ NON-LINEAR
  - Recovered: a=1, c=4
  - Consistency: 20.1%
  - First failure at transition 2→3
- **Lane 2 (mod 7):** ✗ NON-LINEAR
  - Recovered: a=5, c=1
  - Consistency: 18.6%
  - First failure at transition 2→3
- **Lane 3 (mod 11):** ✗ NON-LINEAR
  - Recovered: a=7, c=3
  - Consistency: 10.1%
  - First failure at transition 2→3
- **Lane 4 (mod 13):** ✗ NON-LINEAR
  - Recovered: a=10, c=6
  - Consistency: 11.1%
  - First failure at transition 4→5

### Generator 5: LCG (Reel-Mapped Output)

**Commutativity: 0.0% (0/5 lanes)**

- **Lane 0 (mod 3):** ✗ NON-LINEAR
  - Recovered: a=2, c=0
  - Consistency: 33.7%
  - First failure at transition 0→1
- **Lane 1 (mod 5):** ✗ NON-LINEAR
  - Recovered: a=4, c=3
  - Consistency: 14.1%
  - First failure at transition 0→1
- **Lane 2 (mod 7):** ✗ NON-LINEAR
  - Recovered: a=6, c=3
  - Consistency: 14.1%
  - First failure at transition 0→1
- **Lane 3 (mod 11):** ✗ NON-LINEAR
  - Recovered: a=10, c=3
  - Consistency: 14.1%
  - First failure at transition 0→1
- **Lane 4 (mod 13):** ✗ NON-LINEAR
  - Recovered: a=12, c=3
  - Consistency: 14.1%
  - First failure at transition 0→1

### Generator 6: LCG (Truncated / Bit-Masked)

**Commutativity: 0.0% (0/5 lanes)**

- **Lane 0 (mod 3):** ✗ NON-LINEAR
  - Recovered: a=0, c=2
  - Consistency: 21.6%
  - First failure at transition 2→3
- **Lane 1 (mod 5):** ✗ NON-LINEAR
  - Recovered: a=1, c=2
  - Consistency: 17.1%
  - First failure at transition 0→1
- **Lane 2 (mod 7):** ✗ NON-LINEAR
  - Recovered: a=0, c=4
  - Consistency: 17.6%
  - First failure at transition 2→3
- **Lane 3 (mod 11):** ✗ NON-LINEAR
  - Recovered: a=1, c=6
  - Consistency: 17.1%
  - First failure at transition 2→3
- **Lane 4 (mod 13):** ✗ NON-LINEAR
  - Recovered: a=3, c=9
  - Consistency: 9.0%
  - First failure at transition 2→3

### Generator 7: Compound LCG (Dual-Interleaved)

**Commutativity: 0.0% (0/5 lanes)**

- **Lane 0 (mod 3):** ✗ NON-LINEAR
  - Recovered: a=0, c=1
  - Consistency: 33.2%
  - First failure at transition 0→1
- **Lane 1 (mod 5):** ✗ NON-LINEAR
  - Recovered: a=1, c=1
  - Consistency: 50.3%
  - First failure at transition 2→3
- **Lane 2 (mod 7):** ✗ NON-LINEAR
  - Recovered: a=6, c=4
  - Consistency: 66.3%
  - First failure at transition 0→1
- **Lane 3 (mod 11):** ✗ NON-LINEAR
  - Recovered: a=0, c=5
  - Consistency: 9.0%
  - First failure at transition 0→1
- **Lane 4 (mod 13):** ✗ NON-LINEAR
  - Recovered: a=9, c=3
  - Consistency: 17.1%
  - First failure at transition 2→3

### Generator 8: LFSR-16 (Galois)

**Commutativity: 0.0% (0/5 lanes)**

- **Lane 0 (mod 3):** ✗ NON-LINEAR
  - Recovered: a=2, c=0
  - Consistency: 62.3%
  - First failure at transition 3→4
- **Lane 1 (mod 5):** ✗ NON-LINEAR
  - Recovered: a=3, c=0
  - Consistency: 65.3%
  - First failure at transition 2→3
- **Lane 2 (mod 7):** ✗ NON-LINEAR
  - Recovered: a=4, c=0
  - Consistency: 57.3%
  - First failure at transition 2→3
- **Lane 3 (mod 11):** ✗ NON-LINEAR
  - Recovered: a=6, c=0
  - Consistency: 51.8%
  - First failure at transition 2→3
- **Lane 4 (mod 13):** ✗ NON-LINEAR
  - Recovered: a=7, c=0
  - Consistency: 51.8%
  - First failure at transition 2→3

### Generator 9: XorShift32

**Commutativity: 0.0% (0/5 lanes)**

- **Lane 0 (mod 3):** ✗ NON-LINEAR
  - Recovered: a=0, c=1
  - Consistency: 40.7%
  - First failure at transition 2→3
- **Lane 1 (mod 5):** ✗ NON-LINEAR
  - Recovered: a=1, c=3
  - Consistency: 21.1%
  - First failure at transition 0→1
- **Lane 2 (mod 7):** ✗ NON-LINEAR
  - Recovered: a=2, c=2
  - Consistency: 13.6%
  - First failure at transition 2→3
- **Lane 3 (mod 11):** ✗ NON-LINEAR
  - Recovered: a=6, c=7
  - Consistency: 6.5%
  - First failure at transition 2→3
- **Lane 4 (mod 13):** ✗ NON-LINEAR
  - Recovered: a=4, c=4
  - Consistency: 10.6%
  - First failure at transition 2→3

### Generator 10: Mersenne Twister (MT19937)

**Commutativity: 0.0% (0/5 lanes)**

- **Lane 0 (mod 3):** ✗ NON-LINEAR
  - Recovered: a=0, c=0
  - Consistency: 39.7%
  - First failure at transition 0→1
- **Lane 1 (mod 5):** ✗ NON-LINEAR
  - Recovered: a=3, c=0
  - Consistency: 15.6%
  - First failure at transition 2→3
- **Lane 2 (mod 7):** ✗ NON-LINEAR
  - Recovered: a=2, c=0
  - Consistency: 14.6%
  - First failure at transition 2→3
- **Lane 3 (mod 11):** ✗ NON-LINEAR
  - Recovered: a=5, c=4
  - Consistency: 8.0%
  - First failure at transition 2→3
- **Lane 4 (mod 13):** ✗ NON-LINEAR
  - Recovered: a=10, c=4
  - Consistency: 11.1%
  - First failure at transition 2→3
