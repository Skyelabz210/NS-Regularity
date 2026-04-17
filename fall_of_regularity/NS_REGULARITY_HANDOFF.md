# NS Regularity Workspace — Agent Handoff Document

**Project:** QMNF Navier-Stokes Regularity via CRT Contraction
**Owner:** Anthony Diaz (Acid) — HackFate.us / Skyelabz210
**Workspace:** `ns_regularity_complete.tar.gz`
**Date:** April 4, 2026
**Status:** 49 tests passing, 0 failures, ZERO floating-point in production code

---

## 1. WHAT THIS PROJECT IS

This is a Rust workspace that implements a discrete 3D Navier-Stokes regularity argument using exact integer arithmetic on a CRT (Chinese Remainder Theorem) substrate. The goal is to prove that discrete NS solutions on the CRT torus contract uniformly, implying regularity of the continuum limit.

The workspace implements a three-layer architecture defined in a rigorous research framework (Theorems A through I — see `DEEP_RESEARCH_FRAMEWORK.md` in the tarball):

- **Layer 1 (CRT Executor):** `qmnf_crtbigint/` — 8×63-bit CRT lanes (~504-bit capacity), Montgomery form, K-Elimination for exact division. This is the arithmetic core.
- **Layer 2 (Overflow Geometry):** `src/recumbent.rs` — Tiered exact integer with primary corridor (where contraction bound B ≤ 90/1001 applies), overflow lane (tracks stretching transient), and hot lane (for very large values). Includes WASSAN holographic drive for persistence.
- **Layer 3 (NS Consumer):** `src/cram_ns.rs` — NS evolution on CRTBigInt lanes with K-Elimination division. Also `src/lib.rs` contains 23 theorems (T1-T23) with computational verification.

**Core result:** The self-coefficient of the 7-point NS stencil, reduced modulo discriminating primes {7, 11, 13}, contracts by factor B(N) ≤ 90/1001 < 1 for ALL grid sizes N. This is an algebraic identity proved in Theorem T1 (also formalized in Lean 4 in `theorem1.lean`, sorry-free).

**Key constraint:** ZERO floating-point arithmetic anywhere in the production code. All division is via K-Elimination (Garner reconstruction → integer divide → CRT re-encode). Initial conditions use `integer_ic()` (parabolic arch, all integer). Display ratios use integer permille (parts per thousand).

---

## 2. WHAT IS IN THE TARBALL

```
ns_regularity/
├── Cargo.toml                    # Rust workspace config, depends on qmnf_crtbigint
├── src/
│   ├── lib.rs                    # 1458 lines — 23 theorems (T1-T23), 31 tests
│   ├── recumbent.rs              # 519 lines  — Recumbent CRTBigInt, WASSAN, NS step, 13 tests
│   └── cram_ns.rs                # 175 lines  — CRTBigInt lane-level NS, 5 tests
├── qmnf_crtbigint/
│   └── src/lib.rs                # 1643 lines — CRTBigInt v3 (8×63-bit, Montgomery, K-Elim)
├── theorem1.lean                 # Lean 4 formalization of T1 (sorry-free)
├── DEEP_RESEARCH_FRAMEWORK.md    # Maps Theorems A-I to workspace
├── THEOREM_STACK.md              # Full theorem stack in crusher format
├── README.md                     # Honest assessment of what's proved/not proved
└── NS_REGULARITY_PAPER.docx      # Earlier paper draft (needs update)
```

Build: `cargo test` (requires Rust stable). Expected: **49 passed, 0 failed** in ~20 seconds.

---

## 3. WHAT NEEDS TO BE DONE — TASK LIST

### TASK 1: Wire CRTBigInt Lanes into Recumbent Primary Lane

**What:** The Recumbent primary lane currently stores a plain `i64`. It should store a `CRTBigInt` (or at minimum, residues modulo {7, 11, 13}) so the contraction bound B ≤ 90/1001 is computed on the actual CRT residues, not the reconstructed integer.

**Where:** `src/recumbent.rs`, struct `PrimaryLane`

**How:**
1. Add a `residues: [u64; 3]` field to `PrimaryLane` for residues mod {7, 11, 13}
2. On `assign()`, compute `value % 7`, `value % 11`, `value % 13` and store
3. Add a `contraction_rational()` method that returns the exact B(N) = Π|centered(r_p)|/p for this cell
4. The contraction check `B < 1` is then per-cell: `num < den` (integer comparison)
5. Test: verify that for every cell in corridor, `contraction_rational()` gives (num, den) with num < den

**Why:** This connects the algebraic bound (T1) directly to the Recumbent architecture. Currently the bound is proved for the stencil coefficients, but not wired to the actual cell values.

---

### TASK 2: Lane-Aware Arithmetic in Recumbent

**What:** Currently Recumbent arithmetic does `reconstruct() → compute → re-split`. This is correct but loses lane structure. Instead: do `primary + primary` with overflow carry — keep the CRT residues alive through operations.

**Where:** `src/recumbent.rs`, methods `add()`, `sub()`, `mul()`

**How:**
1. For `add(a, b)`: if both in corridor, add residues per-lane: `(a.res[i] + b.res[i]) % p_i`. If result still fits corridor, no reconstruction needed.
2. If result exceeds corridor: overflow = reconstruct(a) + reconstruct(b) - primary. Split as before.
3. This avoids the Garner reconstruction cost for operations that stay in corridor.

**Why:** Performance and architectural purity. The CRT residues should persist through the computation, not be discarded and rebuilt.

---

### TASK 3: Hot-Lane Phase-Lock via K-Elimination

**What:** When overflow is promoted to hot lane, the hot lane value should be connected to the primary lane via K-Elimination winding number. Currently the hot lane is just a plain `i64`.

**Where:** `src/recumbent.rs`, struct `HotLane`

**How:**
1. Add `winding_k: i64` field to `HotLane`
2. On promotion: `k = (overflow - primary) / M` where M = product of discriminating primes
3. This means: `total = primary + k × M + (overflow mod M)`, and the primary carries the CRT-significant residue
4. On demotion (hot → primary): verify `k × M + primary` reconstructs correctly

**Why:** The winding number k is the bridge between the modular world (where contraction holds) and the integer world (where the PDE lives). K-Elimination is the QMNF mechanism for this bridge.

---

### TASK 4: WASSAN Ingress for Recumbent States

**What:** The WASSAN drive currently stores plain `i64`. It should store the full Recumbent state (primary + overflow + hot) so that on read-back, the lane decomposition is preserved rather than rebuilt.

**Where:** `src/recumbent.rs`, struct `WassonDrive`

**How:**
1. Change `data: HashMap<String, i64>` to `data: HashMap<String, (i64, Option<i64>, Option<i64>)>` storing (primary, overflow, hot)
2. `write()`: store `(r.primary.value, r.overflow.map(|o| o.overflow), r.hot.map(|h| h.value))`
3. `read()`: reconstruct Recumbent with preserved lane values, not re-split from integer

**Why:** Persistence should preserve the computational structure, not flatten it. This matters for deep chained execution where intermediate states are checkpointed.

---

### TASK 5: Wire N=16 Exact Kernel into 3D Solver

**What:** The nsgrid3d 3D solver currently runs at N=8 with an exact integer kernel. The N=16 kernel (D=3.17 billion, 12,288 entries) exists and needs to be wired in. This gives Theorem E data at higher resolution.

**Where:** This is in the main `qmnf_unified` workspace (not in this tarball), specifically `crates/nsgrid3d/`.

**How:**
1. The N=16 Biot-Savart kernel was computed in a previous session. It's a 16×16×16 array of exact i128 values.
2. Load the kernel into nsgrid3d's `biot_savart_exact.rs`
3. Run ABC flow initial conditions for 50+ steps
4. Measure max|ω(t)| at each step — if bounded, Theorem E holds at N=16
5. Compare with N=8 data (max|ω| = 20000 → 18563, DECREASING)

**Why:** Theorem E (no-wrap corridor) is the central unproved step. Currently only verified at N=8. Getting N=16 and N=32 data would show whether the overshoot ratio is bounded across N, which would empirically close Theorem E.

---

### TASK 6: Formalize Discrete H^s Estimate

**What:** Prove that the discrete scheme preserves H^s norms (for s > 5/2) up to bounded growth. This is the continuum-side argument that makes Theorem E rigorous.

**Where:** New theorem in `src/lib.rs` or `THEOREM_STACK.md`

**How:**
1. The discrete H^s norm is Σ_k |k|^{2s} |ω̂(k)|². 
2. The update equation gives: d/dt ||ω||_{H^s}² ≤ C ||ω||_{H^s}³ (from the stretching term)
3. This ODE has solution ||ω||_{H^s}(t) ≤ ||ω₀||_{H^s} / (1 - C||ω₀||_{H^s} t) — finite for t < 1/(C||ω₀||_{H^s})
4. For the CRT scheme: C depends on the Biot-Savart operator norm, which is bounded
5. By Sobolev embedding (s > 3/2): ||ω||_∞ ≤ C_s ||ω||_{H^s} — giving the L^∞ bound
6. This gives Theorem E for small data or finite time T < 1/(C||ω₀||_{H^s})

**Why:** This is the standard Fujita-Kato local existence argument. Applied to the discrete scheme, it gives a rigorous version of Theorem E for finite time. The question is whether C is independent of N (it should be, by consistency of the discrete operators — Theorems B,C).

---

### TASK 7: Update the Paper

**What:** `NS_REGULARITY_PAPER.docx` is from before the Recumbent architecture and the honest audit. It needs to be rewritten to reflect:
1. The three-layer structure (CRT executor / Lifted scheme / Continuum NS)
2. The Recumbent architecture (corridor + overflow + demotion)
3. The honest assessment (what's proved vs what's measured)
4. The corridor dynamics data (step 47 return to corridor at N=64)
5. The zero-float constraint

**Where:** Create a new `.md` document (the `.docx` is stale).

---

### TASK 8: Fix CRTBigInt v3 Sub Bug

**What:** CRTBigInt v3's `Sub` implementation has `sign: self.sign` — it doesn't update the sign on subtraction. This was identified in the session and worked around by using lane-0 centering in `cram_ns.rs`.

**Where:** `qmnf_crtbigint/src/lib.rs`, line ~346, `impl Sub for CRTBigInt`

**How:**
1. After computing residues, reconstruct the result via Garner to determine the correct sign
2. Or: compare the unsigned Garner result against M/2 to determine sign (centered representation)
3. Update `sign` field accordingly
4. Add tests: `CRTBigInt::from_i64(3) - CRTBigInt::from_i64(5)` should give -2

**Why:** The current Sub is broken for cross-sign operations. The `cram_ns.rs` module works around this by using raw residue arrays instead of CRTBigInt. Fixing this would allow cleaner integration.

---

## 4. ARCHITECTURAL CONTEXT

### The QMNF Ecosystem
- QMNF = Quantum Modular Number Framework
- Core principle: "Truth cannot be approximated" — zero floating-point everywhere
- K-Elimination: exact division in CRT space via phase differential k ≡ (v_R − v_P)·C_P⁻¹ mod C_R
- Safe Basis: {2, 3, 5, 7, 11, 13} — six primes with proven phase-lock saturation
- Discriminating Primes: {7, 11, 13} — where the contraction bound operates (all ≥ 7, so ρ(B) ≥ 3 > deg(stretching) = 2)

### The Contraction Bound
B(N) = Π_{p ∈ {7,11,13}} |centered(self(N,p))| / p ≤ Π (p-1)/(2p) = (3/7)(5/11)(6/13) = 90/1001

This is an algebraic identity. `self(N,p) = (1 - 6·c_diff) mod p` where c_diff = Δt·ν·N²/SCALE². The centered value |centered(x)| ≤ (p-1)/2 for any x ∈ Z/pZ. The product 90/1001 < 1 because 90 < 1001. Period of B(N) divides lcm(7,11,13) = 1001. Exhaustively verified for N = 1..1001.

### The Recumbent Architecture
```
Value = 25000 (exceeds corridor R = 10000)

Primary:   9999   ← CRT contraction applies here
Overflow: 15001   ← exact integer, tracks stretching transient
Hot:       None   ← not promoted (|overflow| < threshold)

After diffusion brings value back to 8000:
Primary:   8000   ← DEMOTION fired, full contraction applies
Overflow:  None   ← cleared
Hot:       None   ← cleared
```

### Key Data Points
- Corridor dynamics (N=64, CFL stencil): leaves at step 1, peak 26213 at step 20, returns at step 47, decays to 1257 by step 200
- Contraction: B ≤ 90/1001 = 0.0899 (algebraic). Actual max B(N) = 60/1001 at N=19
- CRTBigInt capacity: M ≈ 2^504. Max measured vorticity: ~30000. Corridor margin: 10^148
- 3D solver (nsgrid3d, N=8): ABC flow stretching 1.5M, diffusion dominates 33,000×, solution DECAYS

---

## 5. HOW TO BUILD AND TEST

```bash
# Extract
tar xzf ns_regularity_complete.tar.gz
cd ns_regularity

# Build and test
cargo test -- --nocapture

# Expected output: 49 passed, 0 failed (~20 seconds)
```

Rust stable required. Zero external dependencies (qmnf_crtbigint is a path dependency included in the tarball).

---

## 6. CRITICAL INVARIANTS — DO NOT VIOLATE

1. **ZERO float in production code.** All `f64`/`f32` usage is banned. Division uses K-Elimination. Ratios use integer permille. Initial conditions use `integer_ic()`.
2. **Reconstruction invariant:** For any `Recumbent` value, `reconstruct() = primary.value + overflow.overflow + hot.value` must equal the exact mathematical integer. Test this exhaustively for any change to the split/demotion rules.
3. **Contraction bound is ALGEBRAIC:** B(N) ≤ 90/1001 is an identity of modular arithmetic, not a numerical approximation. Do not introduce approximations into this chain.
4. **Honest reporting:** If a test shows growth or divergence, report it as an honest negative. Do not suppress or reframe unfavorable data. The project's credibility depends on rigorous honesty.
5. **The 1D proxy is NOT a valid test for 3D NS.** The 1D model lacks divergence-free velocity, Biot-Savart cancellations, and 3D geometric structure. Only the nsgrid3d 3D solver gives valid Theorem E data.
