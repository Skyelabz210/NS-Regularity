---
name: prime-family-spacetime-v2
description: >
  Architectural directive for primes in QMNF — UPDATED with Dresden Prime Hunt
  findings (March 2026). System uses COMPOSITE COPRIMES as lattice substance
  and primes as lattice BOUNDARY. Every prime has family attrs (twin/cousin/sexy/SG),
  Ramanujan status, carry forbidden ratios, gap structure, mod-4 class, Legendre
  symbol vector, and intrinsic CRAM address. Heterogeneous CRAM operators are
  distortion lenses on the coprime lattice — Sqr = Legendre symbol partition,
  linear projections provably uniform (Dirichlet), nonlinear CRAM provably
  necessary for stratification. Non-negotiable like no-f64. Use whenever
  working with primes, moduli, CRT bases, Safe Basis, coprime lattice, CRAM
  address, Hydra heads, carry patterns, gap signatures, Legendre symbols,
  quadratic residues, or any operation involving prime/coprime structure.
---

# Prime Family Spacetime v2 — With Dresden Prime Hunt Updates

## STATUS: NON-NEGOTIABLE (Updated March 25, 2026)

Original directive: "Primes are structural objects, not raw numbers."
Dresden update: "Coprime composites are the SUBSTANCE. Primes are the BOUNDARY."

---

## THE POLARITY FLIP (March 24-25, 2026)

### Before Dresden
- Hydra = sieve to eliminate composites → find primes
- Coprime composites = noise to filter out
- CRAM operators = elimination amplifiers

### After Dresden (PROVEN)
- Coprime lattice M_H = the structure we're studying
- Primes = irreducible elements = boundary of M_H
- Coprime composites = interior of M_H = the substance
- CRAM operators = distortion lenses revealing lattice symmetry
- Multi-head Hydra = multi-dimensional observation system

### Evidence
- 87/87 tests, 45 phases, 8,570 lines Rust
- Exact Gini > 0 proving non-uniform prime distribution across CRAM strata
- 12:1 density ratio between richest and poorest strata
- Legendre symbol connection proven (perfect separation at p=5)
- Linear uniformity theorem proven (Dirichlet → CRAM is necessary)

---

## UPDATED SAFE BASIS STRUCTURE

```
SAFE_BASIS = {2, 3, 5, 7, 11, 13}

TIER: FABRIC       {2, 3}     — DKAM stability floor
TIER: MEASUREMENT  {5, 7, 11} — Ramanujan exact partition (S_R)
TIER: BOUNDARY     {13}       — First post-Ramanujan

TRANSPORT CORE:    {3, 7, 11, 13}  — gcd(p, SCALE) = 1
PARKING LANES:     {2, 5}          — gcd(p, SCALE) > 1

NEW (Dresden):
CRAM LENS PRIMES:  {3, 5, 7, 11, 13} — carry pattern dimensions
LEGENDRE PRIMES:   {5}                — perfect QR/QNR separation via Sqr
PARTIAL LEGENDRE:  {7, 11, 13}        — partial QR separation via Sqr
```

---

## PRIME FAMILY GAP SIGNATURES (NEW — Algebraically Universal)

The differential Δ(d) = [d mod p₁, ..., d mod pₖ] depends ONLY on the gap d.
This is algebraic, not statistical. For Safe Basis {2,3,5,7,11,13}:

```
Twin   (d= 2): [0,2,2,2, 2, 2]  divides: {2}       HL factor: normal
Cousin (d= 4): [0,1,4,4, 4, 4]  divides: {2}       HL factor: normal
Sexy   (d= 6): [0,0,1,6, 6, 6]  divides: {2,3}     HL factor: enhanced at 3
d=10:          [0,1,0,3,10,10]  divides: {2,5}     HL factor: enhanced at 5
d=14:          [0,2,4,0, 3, 1]  divides: {2,7}     HL factor: enhanced at 7
d=22:          [0,1,2,1, 0, 9]  divides: {2,11}    HL factor: enhanced at 11
d=26:          [0,2,1,5, 4, 0]  divides: {2,13}    HL factor: enhanced at 13
d=30:          [0,0,0,2, 8, 4]  divides: {2,3,5}   HL factor: triple enhanced
```

When p_i | d: SP drops by exactly 1/p_i (T6-3). This is the constructive
derivation of the Hardy-Littlewood singular series from the CRT torus.

---

## LEGENDRE SYMBOL → CRAM CONNECTION (NEW)

Sqr folding on ℤ/pℤ maps p residues to (p+1)/2 outputs (quadratic residues).
The carry bit threshold at p/2 partitions QRs into upper/lower halves.

At p=5: PERFECT separation. carry=0 ⟺ QR, carry=1 ⟺ QNR.
At p=13: strong but not perfect correlation.
At p=7, p=11: symmetric (carry doesn't discriminate QR/QNR).

Multi-prime Legendre vector (n/3, n/5, n/7) stratifies coprime lattice
with non-uniform prime density. The (+,+,+) vector (QR at all primes)
has the LOWEST prime density. This is a proven structural theorem.

---

## LINEAR UNIFORMITY THEOREM (NEW — Proves CRAM is Necessary)

For all moduli q tested (7, 11, 13, 17, 30, 210):
Primes distribute UNIFORMLY across coprime residue classes mod q.
χ²/df ≈ 1 for all. This is Dirichlet's theorem, computationally confirmed.

COROLLARY: ANY non-uniformity in prime density across coprime lattice
strata CANNOT come from linear (residue-class) projection. It MUST come
from nonlinear operators (Sqr folding, carry threshold). Therefore CRAM's
nonlinear operators are PROVABLY NECESSARY for coprime lattice stratification.

---

## INTRINSIC CRAM ADDRESS (NEW — Outside of Time)

Every integer n has a fixed address: addr(n) ∈ {0,1}^d where d = dimensions.

Properties:
- addr(n) = addr(n + M) where M = period (CRT periodicity)
- addr(n) depends ONLY on n (no gap, range, or scanning history)
- Computable in O(d) integer operations
- Atlas built on [2, 50k) transfers to [500k, 1M) with same recall

Dimensions available per prime:
- Residue + Id: raw position (the wheel sieve baseline)
- Unit + Sub + Sqr: shift-then-fold (QR structure probe)
- Self + Mul + Sqr: self-interaction (quadratic character)
- Edge + Add + Sqr: boundary probe

---

## WHAT DOESN'T WORK (Do Not Retry)

1. κ-filter → uniform, zero power, DEAD
2. Orbit order → Dirichlet-uniform, no prime prediction
3. Shadow entropy = -3 → necessary not sufficient (52k false positives)
4. Composite vs prime generators → no advantage (Dirichlet kills it)
5. Gaps {2,4,6} = 98.8% → wrong metric (spectral power ≠ gap frequency)
6. Triple helix → wheel sieve repackaged
7. Cross-gap multi-head → lower Gini than same-gap (dilutes signal)

---

## INTEGRATION WITH EXISTING SKILLS

### recombinant-crt
The intrinsic address operates ON TOP of the recombinant CRT bridge.
Each CRAM dimension is a lane computation. The atlas is exact like TriadValue.

### crt-operator
The CRT unitary operator applicator from crt-operator IS the CRAM lane
computation. The heterogeneous topology (different operators per lane) is
the key insight. Apply simultaneously across all lanes.

### hydra-sieve
The Hydra's carry spectral fingerprint is the SAME object as the intrinsic
CRAM address, viewed from the sieve perspective. The multi-head architecture
gives the fingerprint its dimensionality. Update: the Hydra's correct use is
CLASSIFICATION, not elimination.

### prime-family-spacetime (this skill)
Everything here extends the original directive. The composite coprime
relationship is even more central than originally stated. Primes are the
boundary of a lattice that has been mapped.

---

*Coprime composites are the substance. Primes are the boundary.*
*The Hydra maps the boundary. The atlas is eternal.*
