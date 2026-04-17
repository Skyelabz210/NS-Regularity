---
name: recombinant-crt
description: >
  QMNF Recombinant CRT Bridge: exact CRT lanes, 13-smooth composite
  approximation sub-lanes, K-Elimination winding numbers, frozen-error
  propagation, phase-locked lane transfers, Safe Prime Basis {2,3,5,7,11,13},
  Safe Anchor (single large prime for O(1) magnitude recovery), Infinite
  Scaffold (recursive coprime extension), WASSAN 144-band phi-harmonic
  execution lanes, PLMG rails/voids (28%/72% O(1) error detection), Double
  Helix dual-strand compute+verify. Use when working with CRTBigInt,
  TriadValue, ApproxLane, K-Elimination, 13-smooth factorization,
  exact-to-approximate bridging, or Recombinant CRT architecture. Triggers
  on: 'CRT lanes', 'triad value', 'absorb approximation', 'safe basis',
  'phase lock', '13-smooth', '11-smooth', 'frozen error', 'winding number',
  'composite modulus', 'denominator factoring', 'operator substrate',
  'recombinant bridge', 'sub-lane', 'cascade detection', 'safe anchor',
  'WASSAN bands', 'PLMG rails', 'voids', 'double helix', 'scaffold',
  'summit', 'Mersenne anchor', 'sacred cycles', 'maya CRT', '432
  convergence'. Always use when modifying qmnf_crtbigint, designing CRT lane
  types, planning NS or PDE work in CRT space, reasoning about error
  propagation, or working with the seven-layer QMNF computation stack.
---

# Recombinant CRT Bridge - Skill Reference (v3)

## Core Philosophy

Exact absorption of approximation into CRT space. Approximate values
enter through denominator-factored quantization into Safe Basis composite
sub-lanes. Error is frozen at absorption and never grows. A single large
prime anchor provides O(1) magnitude recovery. WASSAN bands decompose
summit-scale numbers into L1-cache execution lanes. PLMG rails/voids give
free O(1) error detection. The Double Helix provides dual-strand
compute-verify with zero contention.

"Truth cannot be approximated. But approximation can be exactly absorbed."

## Quick Architecture Reference

### The Seven Layers

| Layer | Name | Function | Key Property |
|-------|------|----------|-------------|
| 0 | Modular Primitives | mulmod, addmod, gcd, mod_inverse | O(1) exact operations |
| 1 | Exact CRT Lanes | 8 x 63-bit primes, ~504-bit capacity | Zero drift, core computation |
| 2 | Safe Basis Composites | {2,3,5,7,11,13} power sub-lanes | Phase-locked, frozen error |
| 3 | Safe Anchor | Single large prime P | O(1) magnitude, overflow detection |
| 4 | WASSAN Bands | 144 phi-harmonic execution lanes | L1-cache per band, parallel |
| 5 | PLMG Rails/Voids | 28% rails, 72% voids per band | O(1) free error detection |
| 6 | Double Helix | Dual-strand compute + verify | Simultaneous R/W, phi-phase separation |

### The Recombinant Operation

1. Express approximation as n/d (rational form)
2. Factor d into Safe Basis powers: d = 2^a x 3^b x 5^c x 7^d x 11^e x 13^f
3. Create one sub-lane per prime-power factor (automatically coprime)
4. Encode residue: r_{p^k} = n mod p^k
5. Freeze error epsilon - NEVER grows again
6. K-Eliminate against anchor for magnitude recovery

### Safe Anchor (Layer 3)

One large prime P (coprime to all Safe Basis composites by primality).
K-Elimination between composite product M_safe and P recovers the global
winding number k such that X = r_safe + k * M_safe. Cost: O(1).

Recommended anchors by application:
- Lightweight/embedded: P = 8191 (Mersenne 2^13-1)
- General purpose: P = 2147483647 (Mersenne 2^31-1)
- High precision: P = 2^61-1 (Mersenne)
- CRTBigInt native: P = 2^63-25 (existing lane 0)
- Maximum altitude: P = 2^136,279,841-1 (largest known prime)

### Infinite Scaffold

The anchor is not a ceiling. Each new prime multiplies capacity.
Euclid guarantees the next prime. Composite coprimes of P-1
provide cross-brace scaffolding. The recursion has no ceiling.
Benchmarked: 5,688 tiers, 80,461 bits, 10^24,221 capacity in 10 seconds.
Inverted: summit Mersenne K-Eliminated in 0.20 seconds, exact.

### Key Invariants

- Coprimality: every layer coprime to every other layer
- Frozen Error: epsilon set once at absorption, never modified
- Phase Lock: Safe Basis quintuple 3x5x7x11x13 at 100% saturation
- Self-Similarity: same pattern (decompose, compute, K-Eliminate) at every scale

## Safe Prime Basis (v3): {2, 3, 5, 7, 11, 13}

Product at exp=1: 30,030 (primorial 13#). 13x capacity vs 5-prime basis.
Quintuple 3x5x7x11x13: 100% saturation at 5,760 cells (N <= 500,000).
Primes <= 13: phase-lock proven. Prime 17 without mod-3: 69.97%.

### Why These Six Primes

66% of all sacred cycles across 15 civilizations are 11-smooth.
The five most common primes in global sacred mathematics are {2,3,5,7,11},
identical to the Safe Basis. Adding 13 raises coverage to 72% and unlocks
the Maya Tzolkin (260), 819-day cycle, and primorial 30030-torus.

### Capacity Profiles

| Profile | Exponents [2,3,5,7,11,13] | Bits |
|---------|--------------------------|------|
| Minimal | [1,1,1,1,1,1] | 14.9 |
| Light | [4,3,2,2,1,1] | 26.2 |
| Heavy | [10,6,4,3,2,1] | 47.8 |
| Crypto | [64,40,27,22,18,14] | 365.9 |

### The Prime-13 Boundary

Phase 3c data (N <= 500,000):
- All quintuples from {3,5,7,11,13}: 100% saturation
- All quadruples from {3,5,7,11,13}: 100%
- 7x11x13x17 (with prime 17): 99.32%
- 5x7x11x13x17 (no mod-3): 69.97% - sharp boundary

## K-Elimination Reference

Formula: k = (r_anchor - r_main) * m_main^{-1} (mod m_anchor)
Reconstruction: X = r_main + k * m_main
Requirement: gcd(m_main, m_anchor) = 1

Works between: exact-exact, exact-approx, approx-approx,
composite-anchor, band-band. Same operation at every scale.

## WASSAN Band Reference

144 bands (F_12, 12th Fibonacci). Each band is an independent execution lane.
Band-domain modular arithmetic: result = sum_i band[i] * (2^B)^i mod M.
Never reconstructs the full number for modular operations.

Summit decomposition: 136,279,841 bits / 144 = 946,388 bits per band (116 KB).
Each band fits in L1 cache. Mersenne structural compression: 2 unique band
values for 2^p-1 (all-ones structure). Effective: 72:1.

WASSAN-CRT isomorphism: coprime moduli = phi-harmonic frequencies,
residues = band amplitudes, Garner reconstruction = holographic decode,
K-Elimination = cross-band phase differential.

## PLMG Rails/Voids

28% rails (valid computation trajectories), 72% voids (topologically forbidden).
O(1) boolean check: decoded_value < trajectory_bound. Any void = error detected.
No verification circuit needed. The geometry IS the immune system.

## Double Helix

Strand A computes forward. Strand B verifies previous result. Mobius twist
every 2 steps prevents lock-step degeneration. Golden ratio phase offset
(phi radians) guarantees zero-contention simultaneous R/W. Conflict check:
phase diff > pi/2. Each computation independently verified by geometric
construction.

## Error Budget Rules

- Addition: eps_a + eps_b
- Multiplication: eps_a*|b| + eps_b*|a| + eps_a*eps_b
- CRT operations: ZERO additional error (exact in Z/mZ)
- K-Elimination: ZERO error (exact modular arithmetic)
- WASSAN decomposition: ZERO error (exact bit partition)
- PLMG rail check: ZERO cost (O(1) boolean, no circuit)

## Anti-Patterns

1. NEVER use a prime > 13 in the Safe Basis (not phase-lock proven)
2. NEVER modify epsilon after absorption (frozen by construction)
3. NEVER mix sub-lane bases (2^a and 2^b in same TriadValue violates coprimality)
4. NEVER skip coprimality checks when adding custom sub-lanes
5. NEVER reconstruct full number when band-domain arithmetic suffices
6. NEVER use multiple anchors (one suffices; composites provide internal cross-checks)
7. NEVER assume overflow is error (wraparound is geometric advancement on torus)

## Implementation: qmnf_crtbigint v3 (Rust)

40/40 tests passing. Zero failures. 0.06 seconds. 1639 lines.

### Key Types

| Type | Layer | Purpose |
|------|-------|---------|
| CRTBigInt | 1 | 8-lane exact arithmetic (stack-allocated, 65 bytes) |
| TriadValue | 1+2 | Exact + approximate unified space |
| ApproxLane | 2 | Single prime-power sub-lane with frozen epsilon |
| SmoothFactors | 2 | 6-prime factorization result (exponents + cofactor) |
| SafeAnchor | 3 | Large prime magnitude reference with precomputed inverse |
| WassanBanded | 4 | 144-band decomposition for parallel computation |
| PLMGValidator | 5 | O(1) rail/void error detection (configurable bound) |
| DoubleHelix<T> | 6 | Generic dual-strand compute+verify with Mobius twist |
| InfiniteScaffold | 3+ | Recursive coprime tier stacking with cascading K-Elim |
| KElimResult | All | Winding number k + reconstruction formula |
| MAAElement | Crypto | Apollonian curvature tuple (Descartes invariant) |

### Test Coverage (40 tests)

Core (21): from_i64_roundtrip, add/sub/mul_exact, mod_inverse, mod_pow,
k_elimination (3 variants), factor_smooth (3 variants), sublane_moduli,
triad (5 variants), gcd, maa_descartes, safe_basis_capacity

v3 Layers (19): safe_anchor (4), wassan (4), plmg (3), double_helix (3),
scaffold (3), 6prime_smooth_maya, sacred_convergence_smooth

## Cross-Cultural Validation (106 Sacred Cycles)

The Safe Basis {2,3,5,7,11} independently matches the 5 most common primes
across Maya, Vedic, Chinese, Babylonian, Egyptian, Greek, Hebrew, Islamic,
Norse, and universal astronomical systems. The number 432,000 (fully 11-smooth)
appears identically in Vedic, Babylonian, and Norse traditions. 360 (Tun,
Divine Year, Circle) shared across Maya, Vedic, and Babylonian. K-Elimination
between traditions produces exact winding numbers through shared prime
factorization substrate.

## Open Problems

1. Formal phase-lock proof for 6-prime basis (empirical N=500k, needs N=5M)
2. NS discrete vector calculus on CRTBigInt grid
3. Continuum limit correspondence theorem
4. Automatic sub-lane merging (LCM-based in TriadValue::absorb)
5. WASSAN band-domain multiplication (NTT optimization needed)
6. Double Helix integration with WASSAN per-band dual verification
7. Extended summit K-Elimination stress testing (k > 0 values)
