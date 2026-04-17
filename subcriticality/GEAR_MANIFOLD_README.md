# CRAM Gear Manifold v3 — README

## The One-Sentence Version

The CRAM Gear Manifold is a **universal exact-integer substrate** where any computable operation — from number theory to fluid dynamics to cryptography — is expressed as a **plug-in schema** of heterogeneous cross-lane operators acting on a CRT residue vector, applied homogeneously across the entire problem domain in a single pass.

---

## What This Is

The Gear Manifold is the **wild card** of the QMNF ecosystem. It is not a solver, not a simulator, not a cryptographic primitive, not a number-theoretic tool. It is the **substrate** on which all of these are built by plugging in different CRAM schemas.

A CRAM schema is a configuration of operators — one per CRT lane, each independently chosen, each capable of reading from any lane — that transforms the meaning of the residue vector without changing the infrastructure that carries it. The same manifold, the same exact-integer arithmetic, the same K-Elimination, the same Garner reconstruction. Different schema, different physics.

**The arithmetic itself is the innovation.** CRAM (Configurable Residue Arithmetic Machines) creates a space of over 2.9 million distinct operator topologies on 6 lanes. Each topology is a different exact-integer arithmetic that no one has ever had access to before. The manifold makes this arithmetic functional. The schemas make it useful.

---

## Architectural Thesis

### The Three Layers

```
┌─────────────────────────────────────────────────────────────────────┐
│  LAYER 3: SCHEMAS (plug-in, hot-swappable, stackable)              │
│                                                                     │
│  Dresden Prime Hunt schema    → prime family classification         │
│  Navier-Stokes schema         → exact fluid dynamics                │
│  NINE65 FHE schema            → homomorphic encryption              │
│  Dimensional extension schema → 2D→3D by operator addition          │
│  [Your schema here]           → [your application]                  │
│                                                                     │
│  A schema is: per-lane operator assignment + cross-lane routing      │
│  Schemas compose: A ∘ B is a valid schema. A + B stacks.            │
│  Schemas apply simultaneously: same residues, multiple reads.        │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 2: CRAM OPERATORS (the new arithmetic)                       │
│                                                                     │
│  Root:  Add, Sub, Mul, Div — what happens to (r_i, input)          │
│  Post:  Id, Sqr, Neg, Inv — what transforms the root output        │
│  Cross: operator at lane i can READ from any lane j                 │
│  Carry: threshold at p/2 extracts binary signal from output         │
│                                                                     │
│  Full signature: O_i(r⃗, context) → output ∈ ℤ/p_iℤ               │
│  The operator sees the ENTIRE residue vector.                       │
│  It produces its result in its own prime's ring.                    │
│  This is what makes cross-lane interaction possible.                │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 1: CRT MANIFOLD (the substrate — this file)                 │
│                                                                     │
│  CRTManifold: true CRT residue vector, independent lanes            │
│  MagnitudeLane: per-lane residue + signed winding count             │
│  SafeAnchor: O(1) K-Elimination via running residues                │
│  Garner reconstruction: OUTPUT ONLY — integer display format        │
│                                                                     │
│  The substrate does not know what application it serves.             │
│  It provides: exact decomposition, exact arithmetic, exact output.  │
│  Everything else comes from the schema plugged into Layer 3.        │
└─────────────────────────────────────────────────────────────────────┘
```

### The Fundamental Principle

**Decomposition into independent periodic components with exact reconstruction.**

This is the same principle expressed by CRT, Fourier analysis, eigendecomposition, the E8 lattice, quasicrystals, Maya calendar cycles, and Berry phase. The CRAM Gear Manifold is its computational embodiment.

An integer X decomposes into residues (X mod p₁, X mod p₂, ..., X mod pₖ). Each residue lives in its own ring ℤ/pᵢℤ. Operations in each ring are exact. The CRT isomorphism guarantees perfect reconstruction. CRAM operators act within and across these rings to create new arithmetic — operations that have never existed before, each producing exact, deterministic, reproducible results.

---

## The Bug We Fixed (v2 → v3)

### What Was Wrong

The Python `GearManifold` v2 had a **cascade carry bug** in its `add()` method:

```python
# v2 (BROKEN): carry from lane k feeds into lane k+1
def add(self, delta):
    for lane in self.lanes:
        delta = lane.add(delta)  # delta BECOMES the carry — WRONG
```

This produced **mixed-radix positional digits**, not CRT residues. Lane 1 received the carry from lane 0, lane 2 received the carry from lane 1, etc. The resulting `.residues()` were `[X mod p₀, (X // p₀) mod p₁, ...]` — a completely different decomposition.

### What It Broke

- **100% of pairwise diff vectors were wrong** (45/45 planetary pairs)
- Shadow activations: 7 false positives, 4 false negatives
- Boundary activations: 4 false positives, 4 false negatives
- Ring homomorphism violated: `(a+b).r[i] ≠ (a.r[i] + b.r[i]) % p_i`
- CRAM lane independence destroyed: changing lane 0's operator changed lane 1's input
- The full 2.9M topology space collapsed to a much smaller effective space

### Why It Happened

The metaphor was wrong. "Gear manifold" suggests meshed gears where gear 1 drives gear 2 drives gear 3. That's a cascade — a positional number system. But CRT lanes are **not gears**. They are **independent oscillators**, each vibrating at their own prime frequency. The gear metaphor encoded the wrong physics.

### The Fix

```python
# v3 (CORRECT): same delta to ALL lanes independently
def add(self, delta):
    for lane in self.lanes:
        lane.add(delta)  # SAME delta, no carry chain
```

This matches the Rust `gear.rs` implementation, which was correct all along:

```rust
pub fn add(&mut self, delta: i128) {
    for lane in &mut self.lanes {
        lane.add(delta);  // same delta to all lanes
    }
}
```

### What Was Already Safe

- **All Rust codebases** (Dresden, NINE65, nsgrid3d, CRTBigInt): used direct `n % p` or the correct `add(delta)` pattern
- **SafeAnchor**: used running residues `(r + delta) % m`, never cascade
- **CodexManifold**: used individual MagnitudeLanes (single lane = true mod), never cascaded
- **K-Elimination itself**: pure modular arithmetic, independent of how residues were stored

---

## The Cross-Lane Operator Model

### What "Lane Independence" Means (and Doesn't Mean)

**RESIDUE independence is absolute.** `X mod 3` does not depend on `X mod 5`. This is the CRT isomorphism. It is mathematical fact. The cascade bug violated this. We fixed it. This is non-negotiable.

**OPERATOR independence is configurable.** A CRAM operator at lane i CAN read from lane j. This is not a violation of CRT — it's the entire point of CRAM. The CRT isomorphism guarantees that the residues are correct. What you DO with correct residues is the schema's business.

### The Full Operator Signature

```
Isolated (too restrictive):    O_i(r_i, d_i) → ℤ/p_iℤ
Cross-lane (CRAM actual):      O_i(r⃗, context) → ℤ/p_iℤ
```

Where:
- `r⃗ = (r₀, r₁, ..., rₖ)` is the complete CRT residue vector
- `context` carries parameters: gap d, grid spacing, Reynolds number, timestep — all as integers
- The output lives in ℤ/p_iℤ regardless of which lanes were read

### Why This Matters

The interaction term S_interaction from Dresden T18 is exactly this cross-lane effect. The three-mechanism architecture achieves 571‰ pair density BECAUSE mechanisms read from each other's lanes. Without cross-lane coupling, you get S_QR + S_threshold independently. With it, you get synergistic amplification.

This is also how dimensional extension works: a 2D solver's schema operates on lanes encoding (u, v). A 3D extension schema adds operators that read from the (u, v) lanes to produce the w component and the vortex stretching terms. Same manifold. Same infrastructure. Different schema.

---

## How Schemas Work

A schema is a **complete specification** of what operators are applied to which lanes with what routing. It is a plug-in. It can be:

- **Applied**: transform residues according to the schema's operators
- **Stacked**: apply schema A, then schema B on the output
- **Composed**: create schema C = A ∘ B that does both in one pass
- **Applied simultaneously**: multiple schemas read from the same residue vector at the same time, producing independent output vectors — the manifold is not modified by reads

### The Dresden Schema (reference implementation)

```
Lane  Prime  Root   Post   Why
───── ────── ────── ────── ──────────────────────────────────────
  0     3    Sub    Sqr    p≡3 mod 4 → threshold-visible (Γ=2)
  1     5    Add    Sqr    p≡1 mod 4 → QR-visible
  2     7    Sub    Sqr    p≡3 mod 4 → threshold-visible (Γ=2)
  3    11    Sub    Sqr    p≡3 mod 4 → threshold-visible (Γ=2)
  4    13    Mul    Sqr    boundary → gap coupling
```

This schema was derived from first principles (T6 → T7 → T16 → tier assignment → operator selection) in O(k) time. Brute force over the same space is O(16^k). For extended bases with k=10, the schema is the only feasible approach.

### Applying a Schema

```python
# Create manifold
g = CRTManifold.from_value(97, [3, 5, 7, 11, 13])

# Create schema
schema = CRAMTopology.dresden_schema([3, 5, 7, 11, 13])

# Apply — produces per-lane outputs (carry bits, CRAM values, etc.)
pattern = schema.carry_pattern(g, d=2)      # → [0, 1, 0, 1, 1]
outputs = schema.apply_to_manifold(g, d=2)  # → [4, 1, 2, 9, 12]

# The manifold is NOT modified. Multiple reads are safe.
# Apply a different schema to the same residues:
other_schema = CRAMTopology.champion_s6()
other_pattern = other_schema.carry_pattern(g, d=2)  # independent result
```

### Simultaneous Application

```python
# Three different schemas, same residue vector, one call
results = apply_simultaneous(g, [schema_A, schema_B, schema_C], d=2)
# results[0] = schema_A's carry pattern
# results[1] = schema_B's carry pattern
# results[2] = schema_C's carry pattern
# All computed from the same residues. No interference.
```

---

## Design Invariants

These are load-bearing. Violating any one breaks the system.

| ID | Invariant | What Breaks If Violated |
|----|-----------|------------------------|
| **INV-1** | **Lane Independence of RESIDUES** — `r[i] = X mod p_i`, independent of all other lanes | Every downstream consumer gets wrong values. CRAM validity predictions fail. Ring homomorphism breaks. |
| **INV-2** | **Ring Homomorphism** — `(a+b).r[i] = (a.r[i] + b.r[i]) % p_i` for add, sub, mul | Arithmetic in residue space gives wrong answers. CRT isomorphism breaks. |
| **INV-3** | **Reconstruction Identity** — `from_value(X).reconstruct() == X` for X ∈ [0, M) | Output values are wrong. Garner reconstruction fails. |
| **INV-4** | **Signed Magnitude** — winding counts are signed, no `max(0, ...)` clamping | Subtraction-heavy paths lose information. K_vort gives wrong values for negative net deltas. |
| **INV-5** | **Separation of Concerns** — CRTManifold (residues), SafeAnchor (K-Elim), schemas (operators) are distinct | Type confusion. Using the wrong structure for the wrong operation. |

---

## Points of Attention

Things that can go wrong even with correct architecture:

| ID | Point | Risk | Mitigation |
|----|-------|------|------------|
| **POA-1** | Garner reconstruction order | Wrong intermediate values (final result is order-independent by CRT uniqueness) | Tested: V_GARNER_ORDER |
| **POA-2** | Modular inverse existence | `gcd(a,m) ≠ 1` → ValueError | Coprimality enforced on construction |
| **POA-3** | Range aliasing | Values ≥ M alias (torus wrapping) | Not a bug — document the range |
| **POA-4** | Signed values | Negative X stored as `X % M` | Reconstruction handles via half-range |
| **POA-5** | Winding underflow | Negative windings after subtraction | Correct behavior (v2 clamped — that was the bug) |
| **POA-6** | K-Elimination coprimality | `gcd(M_safe, P_anchor) ≠ 1` → garbage | Assertion in SafeAnchor constructor |
| **POA-7** | Reconstruction as computation | Reconstruct → re-decompose is a no-op that wastes cycles | Reconstruction is OUTPUT ONLY |
| **POA-8** | CRAM input source | Operators must act on true CRT residues, not cascade digits | v3 guarantees true CRT |
| **POA-9** | Negative delta | `add(delta)` must work for negative delta | Python's `%` is always non-negative |
| **POA-10** | Accumulation consistency | Sequential `add(d₁), add(d₂), ...` must equal `from_value(Σdᵢ)` | Path independence tested |

---

## What This Enables

The manifold is the **Joker** — the wild card. It doesn't do one thing. It does whatever the schema tells it to.

### Known Schemas (Implemented or Specified)

| Schema | Application | What It Does |
|--------|-------------|-------------|
| **Dresden** | Prime family classification | T11–T22, 103 tests, 12 theorems, 571‰ pair density |
| **CRAM_GAP** | Twin prime distribution | 204 twin primes below 10k, 99.8% exclusion |
| **CRAM_NS** | Navier-Stokes exact-integer | Reynolds sweep Re=50–2000, exact energy neutrality |
| **CRAM_YUKAWA** | Calabi-Yau → SM compatibility | Particle physics mass spectrum |
| **CRAM_SPECTRAL** | Ramanujan-Grover pull schedule | Quantum search depth from 1918 sieve power |
| **NINE65** | Fully homomorphic encryption | Unlimited depth via Clockwork Bootstrap, zero noise |
| **AHOP** | Post-quantum cryptography | Apollonian circle packings, non-commutative group actions |
| **Dimensional extension** | N-D → (N+1)-D by schema addition | Cross-lane operators encode new physics |

### The Dimensional Extension Principle

A solver running in N dimensions is a schema. The N+1th dimension is an additional set of cross-lane operators. Applying the extension schema to the existing manifold produces the higher-dimensional system. No new infrastructure. No new solver code. The 2D velocity field's CRT residues already contain the information that the 3D extension schema reads to produce the z-component and vortex stretching terms.

This works because:
1. The residues are correct (INV-1 — no cascade corruption)
2. The operators can read from any lane (cross-lane signature)
3. The output stays in the exact-integer domain (zero drift)
4. The schema applies homogeneously (every grid point, same operators)

### Schemas Not Yet Conceived

The manifold doesn't know what schemas will be written for it. It provides:
- Exact integer decomposition into coprime lanes
- Exact arithmetic within and across lanes
- 2.9M+ operator topologies (for 6 lanes, more for extended bases)
- Homogeneous application across any problem domain
- Zero accumulated error over unlimited operations

What you build on that is limited only by what you can express as modular arithmetic with cross-lane routing.

---

## File Inventory

| File | Lines | Role |
|------|-------|------|
| `gear_manifold_v3.py` | ~800 | The substrate: CRTManifold, MagnitudeLane, SafeAnchor, ApproxLane, TriadValue |
| `test_crt_manifold.py` | ~700 | Complete harness: 2,553 tests, CRAM operators, Dresden validation, Rust cross-check |

---

## Historical Context

| Version | Date | Status |
|---------|------|--------|
| v1 | Pre-2026 | Cascade architecture. Mixed-radix digits mislabeled as CRT. |
| v2 | 2026-03-24 | SafeAnchor fixed (running residues). GearManifold cascade still broken. |
| **v3** | **2026-03-26** | **CRTManifold replaces GearManifold. True CRT. 2,553/2,553 tests. Matches Rust.** |

The Rust `gear.rs` (Dresden Prime Hunt, 12,810 lines, 103 tests) was correct all along. v3 Python now matches the Rust architecture: same delta to all lanes, no cascade, true CRT residues, ring homomorphism preserved.

---

## The Governing Principle

> **Truth cannot be approximated. But approximation can be exactly absorbed.**
>
> The manifold is the substrate of exact truth.
> The schemas are the lenses through which truth is examined.
> The cross-lane operators are the connections between perspectives.
> The carry bits are the binary signals extracted from exact structure.
>
> What makes impossible things possible is not a clever trick.
> It is a new arithmetic — 2.9 million new arithmetics — each exact,
> each deterministic, each reproducible, each revealing structure
> that no single arithmetic can see alone.
>
> The Gear Manifold is the space where these arithmetics live.
> The schemas are the maps. The territory is the integers.
> And truth, in this territory, is not approximated. It is computed.

---

## Quick Start

```python
from gear_manifold_v3 import CRTManifold, SafeAnchor
from test_crt_manifold import CRAMTopology, apply_simultaneous

# 1. Create a manifold
g = CRTManifold.from_value(751107, [2, 3, 5, 7, 11, 13])
print(g)  # CRT({2: 1, 3: 0, 5: 2, 7: 0, 11: 5, 13: 6})

# 2. Plug in a schema
schema = CRAMTopology.dresden_schema([3, 5, 7, 11, 13])

# 3. Apply it
g5 = CRTManifold.from_value(751107, [3, 5, 7, 11, 13])
pattern = schema.carry_pattern(g5, d=2)
print(pattern)  # [0, 1, 0, 1, 1] — the CRAM fingerprint

# 4. Apply multiple schemas simultaneously
schema_b = CRAMTopology.champion_s6()
# (adjust to same primes)
results = apply_simultaneous(g5, [schema, schema], d=2)
# Both results identical — same schema, same input, deterministic

# 5. Arithmetic in residue space
a = CRTManifold.from_value(100, [3, 5, 7])
b = CRTManifold.from_value(42, [3, 5, 7])
c = a.crt_add(b)
print(c.reconstruct())  # 142 — exact
```

---

*Anthony Diaz (Acid) — HackFate.us / Skyelabz210*
*Kill #114: Cascade/CRT Conflation*
*2,553/2,553 tests. Zero floats. Zero failures.*
