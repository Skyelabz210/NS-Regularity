# PRIME HUNT — Results Analysis v2
## QMNF Dresden Prime Manifold · 36/36 tests · 17 phases · 6 modules

---

## Architecture

6 Rust modules, ~1,800 lines, zero floats, zero external deps:

| Module | Lines | Tests | Purpose |
|--------|-------|-------|---------|
| crt.rs | ~180 | 7/7 | Extended GCD, K-Elimination, CRT, factorize, Miller-Rabin |
| gear.rs | ~175 | 4/4 | GearManifold, SafeAnchor (O(1) K-vort) |
| shadow.rs | ~225 | 5/5 | Three-tier classification, Shadow/Boundary layers, Ramanujan sums |
| sieve.rs | ~420 | 6/6 | Shadow gap sieve, cousin pairs, displacement analysis, K-vort scan |
| helical.rs | ~330 | 5/5 | CR=18980 cylinder, shadow minima test, gap concentration |
| monoid.rs | ~290 | 8/8 | M_H monoid, exponential reach, Algorithm 1 κ-filter |

---

## Phase 14-17: Monoid M_H Results

### What the document claims
The 52,749 "false positives" (composites with shadow_entropy=-3) are not failures — they are
**composite elements of a multiplicative monoid** M_H = {n : gcd(n, 730730) = 1}. The primes
are the irreducible (non-factorable) generators of this monoid. The monoid enables "exponential
reach" via modular exponentiation.

### What the data shows

**CONFIRMED ✓:**

1. **M = 730730 = 2×5×7×11×13×73** — correct factorization. 3 is deliberately excluded
   because gcd(3, CR) = 1 and 3 ∉ S_R, so 3 and its powers pass both tests. This is
   mathematically sound.

2. **Monoid closure** — 55/55 product pairs tested, all remain in M_H. The monoid is
   closed under multiplication by construction (if gcd(a,M)=1 and gcd(b,M)=1 then
   gcd(ab,M)=1). This is a theorem, not an empirical finding.

3. **Exponential reach** — Every power b^k with b ∈ M_H stays in M_H and has entropy=-3.
   Verified for bases 3, 17, 27, 51, 289 up to k=8. The θ = b^k mod 18980 fast path
   (O(log k) modular exponentiation) matches exact computation perfectly.

4. **27⁴ mod 18980 = 1** — The θ-orbit of 27 is periodic with period 4 on the cylinder.
   This is genuine structure: ord(27) mod 18980 = 4. Similarly 51⁸ mod 18980 = 1.

5. **Classification counts match (offset by 2)** — The document's "first 10 polluted"
   starting at 27 missed 9 (=3²) which also qualifies. Similarly, 3 is a "good prime"
   the document excluded by convention (restricting to p>13). These are boundary
   definitional choices, not errors.

**FALSIFIED ✗:**

1. **The κ-filter does not help.** Algorithm 1's κ = (θ - 27)·L⁻¹ mod 17 is
   **perfectly uniform** (χ² = 0.32, df=16, critical = 26.3). It provides:
   - Precision: 29.95% (WORSE than baseline 33.78%)
   - Recall: 15.66%
   - Lift: **0.89x** — the filter HURTS prediction vs random guessing within M_H
   
   κ carries zero primality information. It's a hash function, not a detector.

2. **"Irreducible in M_H" IS "prime."** The document claims that checking irreducibility
   (not expressible as a product inside M_H) identifies primes. This is tautologically
   true — but checking irreducibility in a multiplicative monoid IS the factoring problem.
   Algorithm 1 step 6 hides the entire difficulty in the phrase "N is not a proper
   power/product inside M_H."

3. **M_H is a standard wheel sieve.** The set {n : gcd(n, 730730) = 1} is exactly
   the candidates surviving after sieving by {2, 5, 7, 11, 13, 73}. This is a
   6-prime wheel sieve — a technique known since antiquity. The monoid language
   dresses it up but adds no computational power.

4. **Prime density in M_H still follows 1/ln(n).** The monoid eliminates ~72% of
   integers (those sharing a factor with M), but the remaining prime density is
   φ(M)/M · 1/ln(n), tracking the prime number theorem exactly. No anomalous
   concentration or crystallization detected.

### HONEST ASSESSMENT

The monoid reframe is **logically valid** — it correctly identifies the algebraic structure.
But it's **computationally empty**:

- "Primes are irreducible elements of M_H" = "primes are numbers that can't be factored"
  (restated in monoid language)
- "Exponential reach via b^k" = modular exponentiation (known since Euler)
- "M_H = {n : gcd(n,M)=1}" = wheel sieve (known since Eratosthenes)
- "κ-filter" = uniform hash with zero discriminatory power

The documents correctly identify that the shadow entropy condition is *necessary* but not
*sufficient*. The monoid reframe correctly absorbs the false positives into the structure.
But it doesn't close the gap between necessary and sufficient — it just renames it from
"false positive" to "composite element."

### WHERE THE REAL OPEN QUESTIONS ARE

1. **The 27⁴ ≡ 1 (mod CR) periodicity.** Different bases have different orbital periods
   on the cylinder. Can the distribution of these periods tell us something about the
   prime distribution in M_H? This is related to multiplicative order and primitive roots.

2. **The 3-exclusion.** 3 is the only prime ≤ 13 that passes both tests. This is because
   CR = 2²×5×13×73 excludes {2,5,13,73} and S_R = {5,7,11} excludes {5,7,11}. The
   intersection is {2,5,7,11,13,73}, leaving 3 as the sole survivor. Is there a deeper
   reason 3 threads the needle? (Answer: it's because the Maya chose base-20 and
   base-13 cycles, and 3 is coprime to both.)

3. **Composite anchor interaction modes.** The 98.8% figure from Theorem T3 is about
   *spectral power in interaction modes*, not about gap frequencies. Can the Ramanujan
   sum interaction structure on the CRT torus actually constrain prime gaps, or is the
   correlation superficial?
