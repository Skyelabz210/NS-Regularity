# WORKSPACE REVIEW: Three Codebases, One Target
## March 29, 2026

---

## I. WHAT EXISTS

### Codebase A: Our Workspace (`qmnf_unified`)
- **16 crates**, 398 tests, 0 failures
- GearManifold (E8 240/240 on composite coprimes)
- CRAM Ops with D-002/006/007 error handling
- CRAM Safety with D-017/018/019 guards
- Lean 4 (18 remediated theorems)
- nsgrid3d (65 tests), ns_winding_v2 (50 tests)
- All the original hackfate_alpha crates

### Codebase B: QEDDE v3 (`qedde_unified_substrate_v3`)
- **1 crate**, 20 tests, 0 failures
- 1,060 lines — the unified substrate
- WassanDrive with DriveSectors (8-lane CRT values)
- DoubleHelixVerifier (dual-strand checksums)
- AttractorCell (self-correcting memory)
- ShadowEntropyPool (Montgomery quotient harvest)
- OracleRegistry: Grover, Ramanujan, Shor as resident services
- CRTBeacon: shared K-Elimination + reconstruction accelerator
- CellState architecture (FRST learning)
- **Clean, compilable, tested.** This is the UNIFIED substrate design.

### Codebase C: Wave3 Package (`qmnf_execution_wave3`)
- **4-crate workspace + 2 standalone binaries**, 14 tests, 0 failures
- **5 NEW NS modules** (not in our workspace):
  - `biot_savart_exact.rs` (1,250 lines) — OP-3 integer kernel, Q(√2), D=7680
  - `biot_savart_ntt.rs` (538 lines) — two-prime NTT convolution
  - `dkam_monitor.rs` (327 lines) — per-cell DKAM diagnostic with ternary zones
  - `sobolev_winding.rs` (172 lines) — K_E and K_H via K-Elimination
  - `cram_amplified.rs` (488 lines) — CRAM-enhanced NS
- **prime_hunt** (12,810 lines!) — 16 modules:
  - `coprime_families.rs` — the Separation Principle in action
  - `intrinsic.rs` — CRAM addresses (outside-of-time atlas)
  - `prove.rs` — algebraic proof infrastructure (1,963 lines)
  - `multi_hydra.rs` — multi-head Hydra sieve
  - `orbits.rs` — orbital/period structure
  - `gear.rs` — gear manifold predecessor
- **cram_explorer** (3,477 lines) — apollonian, holodrive, shadow ladder
- **hydra_v3** (1,237 lines) — full Hydra sieve
- **k_elim_period** (371 lines) — K-Elimination period analysis
- **17 Python scripts** (6,058 lines) — test batteries, analyses
- **15 docs** (7,242 lines) — theorem stacks, compendium, blueprint
- **2 skill definitions** — cram-operator, prime-family-spacetime-v2

---

## II. GAP ANALYSIS

### What Our Workspace Is Missing

| Module | Source | Lines | Impact | Priority |
|--------|--------|-------|--------|----------|
| `biot_savart_exact.rs` | Wave3 | 1,250 | OP-3 COMPLETE — integer BS kernel | CRITICAL |
| `biot_savart_ntt.rs` | Wave3 | 538 | Two-prime NTT for O(N³ log N) | CRITICAL |
| `dkam_monitor.rs` | Wave3 | 327 | Per-cell DKAM with ternary zones | HIGH |
| `sobolev_winding.rs` | Wave3 | 172 | K_E, K_H exact integer | HIGH |
| `cram_amplified.rs` | Wave3 | 488 | CRAM-enhanced NS solver | HIGH |
| `prime_hunt/` | Wave3 | 12,810 | Coprime families, intrinsic addresses, Hydra | HIGH |
| `cram_explorer/` | Wave3 | 3,477 | Apollonian, holodrive, projector | MEDIUM |
| QEDDE v3 substrate | QEDDE | 1,060 | Unified drive architecture | HIGH |
| `hydra_v3` | Wave3 | 1,237 | Full Hydra sieve | MEDIUM |
| `k_elim_period` | Wave3 | 371 | K-Elimination period structure | MEDIUM |
| 17 Python scripts | Wave3 | 6,058 | Test batteries, analyses | MEDIUM |
| 15 docs | Wave3 | 7,242 | Theorem stacks, compendium | LOW (reference) |
| 2 skills | Wave3 | — | cram-operator, prime-family-v2 | HIGH (install) |

### What Wave3/QEDDE Are Missing

| Module | Lines | What It Provides |
|--------|-------|-----------------|
| `gear_manifold.rs` | 420 | GearManifold + E8 240/240 composite coprimes |
| `cram_ops.rs` | 280 | D-002/006/007 error handling (no silent pass-through) |
| `cram_safety.rs` | 310 | D-017/018/019 (error budget, versioning, fold cap) |
| QMNF_Remediated.lean | 250 | 18 theorems, 7 maintenance fixes |
| QmnfNS_Extended.lean | 465 | T-INFINITE-DKAM, Φ(2.2) morphism |

---

## III. CRITICAL FINDING

**OP-3 (integer Biot-Savart) is ALREADY BUILT in wave3.** The `biot_savart_exact.rs` 
uses Q(√2) rational arithmetic to compute an exact kernel with D=7680. Zero f64 
anywhere. This is the module we've been listing as "3 months of work" — and it exists 
in 1,250 lines of Rust with 4 tests.

This changes the NS roadmap fundamentally. With OP-3 available:
- Exact energy neutrality extends to ALL timesteps (not just t=0)
- The f64 Biot-Savart defect (HN-2) is eliminated
- The attack path to OP-2 (continuum convergence) gets its first prerequisite

---

## IV. MERGE PLAN

### Phase 1: Critical NS Modules (Merge from Wave3 → Workspace)

Copy the 5 new NS modules into our nsgrid3d crate:
```
biot_savart_exact.rs   → crates/nsgrid3d/src/
biot_savart_ntt.rs     → crates/nsgrid3d/src/
dkam_monitor.rs        → crates/nsgrid3d/src/
sobolev_winding.rs     → crates/nsgrid3d/src/
cram_amplified.rs      → crates/nsgrid3d/src/
```
Wire into lib.rs. Fix any compilation issues against our existing nsgrid3d.
Run tests. This gives us OP-3.

### Phase 2: Prime Hunt + CRAM Explorer (Add as New Crates)

```
prime_hunt/            → crates/prime_hunt/ (new workspace member)
cram_explorer/         → crates/cram_explorer/ (new workspace member)
hydra_v3/              → crates/hydra_v3/ (new workspace member)
k_elim_period/         → crates/k_elim_period/ (new workspace member)
```
These are standalone binaries — add to workspace, fix deps, run tests.

### Phase 3: QEDDE v3 Substrate (Merge or Replace)

Our workspace has `qedde_v2` (22 tests). QEDDE v3 (20 tests) is a redesign 
with the unified WassanDrive architecture. Options:
- **Replace:** Drop qedde_v2, install v3 as qedde_v3 crate
- **Bridge:** Keep v2 for backward compat, add v3 as new crate

Recommendation: Add as `qedde_v3` crate alongside v2. Migrate incrementally.

### Phase 4: Python + Docs + Skills

```
python/                → tools/python/ (reference scripts)
docs/                  → docs/wave3/ (reference docs)
skills/                → skills/ (INSTALL as active skills)
```

### Phase 5: Cross-Integration

Wire GearManifold → QEDDE v3 DriveSector (the GearManifold IS the DriveSector)
Wire CRAM Ops → prime_hunt schemas (same 8 operators, now with error handling)
Wire E8 composite-coprime map → cram_explorer apollonian geometry

---

## V. REVISED TEST TARGET

After full merge:

| Source | Tests |
|--------|-------|
| Current workspace | 398 |
| QEDDE v3 | 20 |
| Wave3 nsgrid3d new modules | ~18 |
| prime_hunt | ~50 (estimate from 16 modules) |
| cram_explorer | ~20 (estimate from 10 modules) |
| hydra_v3 | ~5 |
| k_elim_period | ~5 |
| **Target** | **~516** |

Combined with the 1,625 Python/integration tests from the WASSAN×Recombinant×CRAM 
suite: **~2,141 total tests across the ecosystem.**

---

## VI. UPDATED ACTION PLAN

### Immediate (This Session If Time Permits)
1. Copy the 5 new NS modules from wave3 into our nsgrid3d
2. Wire into lib.rs, fix compilation
3. Run tests — if biot_savart_exact passes, OP-3 is merged

### Next Session
4. Add prime_hunt, cram_explorer, hydra_v3, k_elim_period as crates
5. Add QEDDE v3 as qedde_v3 crate
6. Install skills

### Ongoing
7. Cross-integrate GearManifold ↔ DriveSector ↔ prime_hunt schemas
8. Wire E8 into NS diagnostic chain
9. Phase 3 security (D-009/010/011/023 — still OPEN)
