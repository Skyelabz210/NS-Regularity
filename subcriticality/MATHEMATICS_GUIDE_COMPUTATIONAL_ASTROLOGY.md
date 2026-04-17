# A Mathematics Guide to Computational Astrology
## Exact-Integer Modular Arithmetic on Periodic Systems

**Anthony Diaz (Acid) — HackFate.us / Skyelabz210**
**March 2026**

---

*This guide teaches the mathematics of the Chinese Remainder Theorem, Configurable Residue Arithmetic Machines, and the Shadow Prime through the lens of computational astrology. The astrology is the testbed. The mathematics is the product.*

*Every number in this guide is an exact integer. Every computation is reproducible. Every theorem is constructively proved.*

*Scope note on floating-point: The mathematical core — CRT decomposition, CRAM operators, carry extraction, shadow/boundary detection, aspect classification, and all theorem proofs — is zero-float exact-integer arithmetic. The astronomical input layer (ephemeris computation) requires floating-point at the celestial mechanics boundary; the Dresden correction method converts this to integer at a single well-defined boundary, after which all downstream computation is exact. This guide documents the exact-integer core.*

---

# PART I: FOUNDATIONS

## Chapter 1. The Problem with Angular Geometry

Traditional astrology classifies planetary relationships by angular separation. Two planets 120° apart form a "trine." Two planets 90° apart form a "square." The system works — it has worked for two thousand years — but it has a structural limitation: it can only see harmonics that correspond to named angles.

Consider Mars at 6°37' Sagittarius and Venus at 19°19' Virgo. Their angular separation is 77°18'. There is no standard aspect at 77°. Classical astrology says: no relationship. Move on.

But 77°18' = 278,289 arcseconds. And 278,289 mod 11 = 0.

That zero means something. It means that on the eleventh lane of the CRT decomposition — the lane governed by the deepest Ramanujan partition prime — these two planets are aligned. They share a harmonic that angular geometry cannot name because it doesn't correspond to any fraction of 360° that tradition has catalogued.

This guide teaches the mathematics that makes that zero visible.

### The Limitations of Angles

Angular geometry operates in one dimension: the circle. Every planetary relationship is reduced to a single number — the angular separation — which is then compared against a fixed list of "meaningful" angles. This works for the harmonics that divide 360° cleanly (180°, 120°, 90°, 60°) but misses everything else.

The CRT decomposition operates in six dimensions simultaneously. A single angular position, expressed as an integer number of arcseconds, decomposes into six independent residues — one per prime in the Safe Basis {2, 3, 5, 7, 11, 13}. Each residue tells you something different about the position's relationship to a different periodic structure. Angular geometry sees the circle. CRT sees the torus.

### What This Guide Covers

- **Part I** builds the mathematical foundations: CRT decomposition, the Gear Manifold substrate, and the ring isomorphism that makes exact arithmetic possible.
- **Part II** introduces the new arithmetic: CRAM operators, carry patterns, and the Shadow Prime theorem.
- **Part III** applies these tools to computational astrology: positions as residue vectors, aspects as residue-class patterns, shadow bonds, boundary events, and the Dresden correction method.

No prior knowledge of astrology is required. Basic familiarity with modular arithmetic (what "X mod p" means) is assumed.

---

## Chapter 2. CRT: Decomposition into Independent Periodic Components

### The Chinese Remainder Theorem

For a set of pairwise coprime positive integers {p₁, p₂, ..., pₖ} with product M = ∏pᵢ, the Chinese Remainder Theorem establishes an isomorphism of rings:

    ℤ/Mℤ  ≅  ℤ/p₁ℤ × ℤ/p₂ℤ × ... × ℤ/pₖℤ

The forward map (decomposition) sends X to (X mod p₁, X mod p₂, ..., X mod pₖ). The inverse map (reconstruction via Garner's algorithm) recovers X uniquely for X in [0, M).

This is the foundational identity. Every integer in the range [0, M) has a unique representation as a tuple of residues. Every tuple of residues corresponds to a unique integer. The map is bijective, and it preserves ring operations: addition, subtraction, and multiplication in ℤ/Mℤ correspond to componentwise operations in the product ring.

### The Safe Basis

The Safe Basis is B = {2, 3, 5, 7, 11, 13} with product M = 30,030. This choice is not arbitrary:

- **{5, 7, 11}** are the Ramanujan partition primes (S_R): the only primes ℓ ≥ 5 for which p(ℓn + δ) ≡ 0 (mod ℓ) holds for ALL n, where p(n) is the partition function.
- **13** is the boundary prime: the first prime with no universal Ramanujan congruence.
- **{2, 3}** provide the stability floor and base resolution.

Every integer from 0 to 30,029 has a unique six-component residue representation. Planetary longitudes, expressed in arcseconds (0 to 1,295,999), alias on the torus — they wrap. This is by design. The torus topology IS the periodic structure of the zodiac.

### Worked Example 1: CRT Decomposition of a Planetary Longitude

The Sun at 28°14'47" Libra has longitude 749,687 arcseconds (measured from 0° Aries).

    749,687 mod  2 = 1
    749,687 mod  3 = 2
    749,687 mod  5 = 2
    749,687 mod  7 = 1
    749,687 mod 11 = 4
    749,687 mod 13 = 3

    CRT tuple: (1, 2, 2, 1, 4, 3)

This tuple IS the Sun's identity in CRT space. It tells you:
- The Sun's longitude is odd (mod 2 = 1)
- It falls 2/3 of the way through a 3-cycle (mod 3 = 2)
- It falls 2/5 of the way through a 5-cycle (mod 5 = 2)
- It falls 1/7 of the way through a 7-cycle (mod 7 = 1)
- It falls 4/11 of the way through an 11-cycle (mod 11 = 4)
- It falls 3/13 of the way through a 13-cycle (mod 13 = 3)

Each component is a projection onto an independent periodic structure. Changing the Sun's position by 1 arcsecond changes the mod-2 residue. Changing it by 11 arcseconds changes the mod-11 residue. The sensitivities are different because the periods are different. This is the key insight: decomposition reveals structure at multiple scales simultaneously.

### Ring Homomorphism

The CRT isomorphism preserves arithmetic:

    (a + b) mod pᵢ = (a mod pᵢ + b mod pᵢ) mod pᵢ

This means you can add two planetary longitudes in CRT space by adding their residues componentwise. No reconstruction needed. No loss of information. The arithmetic in the product ring IS the arithmetic in ℤ — just viewed through six independent lenses.

### Reconstruction

Garner's algorithm reconstructs the integer from its residue tuple:

    X = r₁ + p₁ × (r₂ - r₁) × p₁⁻¹ mod p₂ + p₁p₂ × (...) + ...

This is OUTPUT ONLY. Never reconstruct an intermediate result. Stay in residue space. Compute everything as componentwise modular arithmetic. Reconstruct only when you need to display the final integer to a human.

---

## Chapter 3. The Gear Manifold: Universal Exact-Integer Substrate

### Architecture

The Gear Manifold is the computational substrate that implements the CRT isomorphism as a live data structure. It provides:

1. **CRTManifold**: true CRT residue vector with independent lanes. Each lane stores X mod pᵢ, updated independently on every operation.

2. **SafeAnchor**: O(1) winding number extraction via K-Elimination. Maintains running residues r_safe and r_anchor, computes k = (r_anchor - r_safe) × M_safe⁻¹ mod P_anchor without reconstruction.

3. **CRAM Schemas**: plug-in operator topologies that transform the meaning of the residue vector for specific applications.

### The Cascade Bug (Kill #114)

A critical architectural lesson: the Python GearManifold v2 implemented addition as a cascade, passing the carry from lane k as the input delta for lane k+1. This produced mixed-radix positional digits, not CRT residues. Lane 1's value was (X // p₀) mod p₁ instead of X mod p₁.

The fix was one line: pass the same delta to ALL lanes independently.

    # WRONG (cascade — mixed-radix digits)
    def add(self, delta):
        for lane in self.lanes:
            delta = lane.add(delta)  # carry chain

    # CORRECT (independent — true CRT)
    def add(self, delta):
        for lane in self.lanes:
            lane.add(delta)  # same delta to all lanes

This distinction is load-bearing. The entire CRAM operator space, the shadow network, the boundary detection, and the CRAM planetary schema all depend on having true CRT residues. Mixed-radix digits look similar but encode completely different information.

### Design Invariants

Five invariants that cannot be violated:

1. **Lane Independence of Residues**: r[i] = X mod pᵢ, independent of all other lanes.
2. **Ring Homomorphism**: (a+b).r[i] = (a.r[i] + b.r[i]) mod pᵢ.
3. **Reconstruction Identity**: from_value(X).reconstruct() = X for X in [0, M).
4. **Signed Magnitude**: winding counts are signed integers, no clamping.
5. **Separation of Concerns**: residues, K-Elimination, and CRAM schemas are distinct.

---

# PART II: THE NEW ARITHMETIC

## Chapter 4. CRAM Operators: 16.8 Million Exact Arithmetics

### The Operator Signature

A CRAM operator at lane i has the signature:

    O_i(r⃗, context) → output ∈ ℤ/pᵢℤ

The operator sees the ENTIRE residue vector (cross-lane access) and produces its result in its own prime's ring. This is the key architectural distinction:

- **Residue independence** (data): X mod pᵢ does not depend on X mod pⱼ. Non-negotiable.
- **Operator cross-lane access** (computation): operator at lane i CAN read from lane j. Configurable.

The data is independent. The operators are not. That's CRAM.

### Root and Post Operators

Each lane receives a root operator followed by a post-processor:

    Root operators: Add, Sub, Mul, Div       (4 choices)
    Post-processors: Id, Sqr, Neg, Inv       (4 choices)

For k = 6 lanes with 4 roots and 4 posts, each lane has 4 × 4 = 16 configurations. The total topology space is:

    16⁶ = 16,777,216

Each topology produces a different, exact, deterministic, reproducible output. (Historical note: earlier versions of the system used 3 post-processors (Id, Sqr, Neg), giving 12⁶ = 2,985,984 ≈ 2.9M topologies. The addition of Inv as a fourth post-processor expanded the space to 16.8M.)

### The Carry Bit

After the post-processor, a binary signal is extracted:

    carry = 1 if 2 × output ≥ p, else 0

This threshold at p/2 partitions ℤ/pℤ into upper and lower halves. The carry bit is the nonlinear signal that CRAM uses for classification.

---

## Chapter 5. Carry Patterns: Binary Signals from Modular Structure

### Aspect Angles Have Unique CRT Signatures Within the Standard Atlas

Among the ten standard astrological aspects (conjunction through semisextile), each has a unique residue tuple mod (7, 11, 13). This uniqueness is relative to the admitted aspect atlas — it states that no two standard aspects share a discriminating-lane signature, not that all possible angular separations produce unique tuples:

    Conjunction (0°):    (0, 0, 0)
    Opposition (180°):   (3, 1, 2)
    Trine (120°):        (2, 8, 10)
    Square (90°):        (5, 6, 1)
    Sextile (60°):       (1, 4, 5)
    Quintile (72°):      (4, 7, 6)
    Semisquare (45°):    (6, 3, 7)
    Quincunx (150°):     (6, 10, 6)
    Biquintile (144°):   (1, 3, 12)
    Semisextile (30°):   (4, 2, 9)

Zero collisions within the standard ten-aspect atlas. This means aspect classification among standard aspects can be done by hash lookup — O(1) instead of iterating through angular comparisons.

Lanes mod 2, 3, 5 are always zero for standard aspects because every standard aspect angle is a multiple of 1° = 3,600 arcseconds = 2³ × 3² × 5². The discriminating information lives entirely on the Ramanujan primes (7, 11) and the boundary prime (13).

### Worked Example 5: Carry Pattern for the Trine

    Trine angle: 432,000 arcseconds (120°)
    CRT residues mod (2,3,5,7,11,13): (0, 0, 0, 2, 8, 10)
    After Sqr:    (0, 0, 0, 4, 9, 9)
    Carry bits:   (0, 0, 0, 1, 1, 1)

The trine is the ONLY aspect whose Sqr carry fires on all three discriminating lanes simultaneously. The conjunction and opposition produce all-zero carries. The square fires only on mod 7. The biquintile fires only on mod 11.

Each aspect has a characteristic carry fingerprint. The fingerprint is a property of the angle's arithmetic, not a human assignment.

---

## Chapter 6. The Shadow Prime: Structure That Cannot Be Directly Measured

### T-SHADOW: Shadow Prime Uniqueness Theorem (Kill #115)

**Definition.** A prime ℓ is a shadow prime with respect to period set P and CRT basis B if:

    S1: ℓ ∈ S_R                    (Ramanujan partition congruence)
    S2: ℓ ∤ T for all T ∈ P        (direct inaccessibility)

These two conditions determine uniqueness. The following characterize the shadow role:

    S3: ∃ T_X ∈ P with 0 < Δ_X < T_X and ℓ | Δ_X
                                    (shadow accessibility via nontrivial displacement;
                                     the guard 0 < Δ_X excludes trivial exact alignment
                                     where every prime would vacuously divide zero)
    S4: ℓ ≡ 3 (mod 4)              (QR separation property)
    S5: ℓ has exclusive Sqr-carry on D = {7, 11, 13}
                                    (CRAM shadow detection, schema-relative)

**Theorem.** For the Dresden period set P = {260, 365, 584, 780, 399, 378, 11960, 18980, 819, 116, 30}, Safe Basis B = {2,3,5,7,11,13}, and CRAM Sqr-carry schema on D = {7,11,13}: the prime 11 is the unique shadow prime.

**Proof.** S_R = {5, 7, 11} (Ramanujan, complete to ℓ = 73). 5 | 260 ∈ P → 5 fails S2. 7 | 819 ∈ P → 7 fails S2. 11 ∤ T for all T ∈ P (complete modular check: 260%11=7, 365%11=2, ..., all nonzero). 11 passes S1+S2. Uniqueness follows. S3: Δ_Saturn = 11960 mod 378 = 242 = 2 × 11². S4: 11 ≡ 3 mod 4. S5: biquintile (144°) carry = (0,0,0,0,1,0), exclusive on lane 11. QED

**Two-layer structure.** S1 + S2 alone determine uniqueness — they reduce the candidate set to {11} by finite exclusion. S3 + S4 + S5 characterize the shadow role — they justify the name "shadow prime" by establishing that 11 is not merely the remaining candidate but has the displacement, reciprocity, and carry signatures that make "shadow" a structural description.

### Conditions for Correctness

The uniqueness claim is conditional on the following. If any condition is violated, the theorem must be re-evaluated:

- **C1 (S_R Completeness):** S_R = {5, 7, 11} is the complete set of universal Ramanujan partition primes. Verified computationally to ℓ = 73. If a new Ramanujan prime were discovered above 11, it would enter the candidate set.

- **C2 (Period Set Stability):** The period set P is the Dresden Codex integer periods. Adding a period divisible by 11 to P would cause 11 to fail S2. The theorem is relative to the stated P.

- **C3 (Aspect Granularity):** S5 assumes aspect angles are multiples of 1° = 3,600 arcseconds. At this granularity, lanes mod 2, 3, 5 are always zero and the discriminating set is D = {7, 11, 13}. Sub-degree aspects would change D.

- **C4 (Basis Selection):** S5 requires the CRT basis to include 11 and the discriminating set to be {7, 11, 13}. A different basis produces a different analysis.

Within the Dresden integer period model, these conditions hold and the theorem is exact. Outside that model, it is a formalization choice, not a universal physical theorem.

### The Four Roles

    5  = surface prime    (divides Tzolk'in, Haab, Venus — directly observable)
    7  = bridge prime     (divides 819-day cycle — accessible but deep)
    11 = shadow prime     (divides nothing observable — only displacement residues)
    13 = boundary prime   (first post-Ramanujan — where guaranteed structure ends)

### Worked Example 4: Saturn's Shadow Displacement

    Saturn synodic period: T_S = 378 days
    Eclipse table epoch:   T_E = 11,960 days
    Displacement: Δ_S = 11,960 mod 378 = 242 = 2 × 11²

    11 does not divide 378 (378 mod 11 = 4).
    11² divides 242 (242 mod 121 = 0).

    The prime 11 is absent from Saturn's period but present — at the
    second power — in Saturn's displacement residue. The shadow prime
    speaks through the slowest visible planet, at elevated power,
    in the residue of the residue.

### Physical Interpretation

The shadow prime is the deepest layer of guaranteed mathematical structure that is invisible to direct astronomical observation. The Maya could not have observed 11 directly in any cycle length. But Saturn's 33-year displacement carries 11². The shadow is not metaphorical — it is the modular arithmetic of the solar system's integer period structure, operating below the threshold of direct measurement but above the threshold of exact computation.

Within the Dresden integer period model, this structure is exact. Outside that model — for instance, using fractional astronomical periods instead of their integer approximations — it is a formalization choice that captures the integer skeleton of the orbital mechanics, not a universal physical theorem. The theorem's power is that it is exact where it applies, and its scope conditions are explicitly stated.

---

*Part III (Computational Astrology) and Parts IV-V continue in subsequent chapters.*

---

**Appendix: Validation Identities**

All examples in this guide can be verified by exact integer computation:

    V-G01: 749687 mod 2 = 1          V-G02: 749687 mod 3 = 2
    V-G03: 749687 mod 5 = 2          V-G04: 749687 mod 7 = 1
    V-G05: 749687 mod 11 = 4         V-G06: 749687 mod 13 = 3
    V-G07: 278289 mod 11 = 0         (Mars-Venus shadow bond)
    V-G08: 11960 mod 378 = 242       V-G09: 242 = 2 × 121 = 2 × 11²
    V-G10: 432000 mod 7 = 2          V-G11: 2² mod 7 = 4
    V-G12: 432000 mod 11 = 8         V-G13: 8² mod 11 = 9
    V-G14: 432000 mod 13 = 10        V-G15: 10² mod 13 = 9
    V-G16: 2×4=8 ≥ 7 → carry=1      V-G17: 2×9=18 ≥ 11 → carry=1
    V-G18: 2×9=18 ≥ 13 → carry=1    (trine carries on all three disc lanes)
    V-G19: 518400 mod 11 = 3         V-G20: 3² mod 11 = 9
    V-G21: 2×9=18 ≥ 11 → carry=1    (biquintile carries on 11)
    V-G22: 518400 mod 7 = 1          V-G23: 1² mod 7 = 1
    V-G24: 2×1=2 < 7 → carry=0      (biquintile silent on 7 — 11 exclusive)

*Truth cannot be approximated. The Maya knew this too — every date in the Dresden Codex is exact.*
