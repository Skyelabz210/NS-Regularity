# Workspace Manifest

Directory-by-directory inventory. Each entry lists provenance, key files, and current status.

---

## `NS_REGULARITY_COMPLETE (1)/` — main workspace

**Provenance:** original canonical project drop (`NS Regularity.zip`)  
**Size:** 18 MB (excluding `target/`)  
**Status:** active development

### Rust crates

| Crate | Role | Tests |
|---|---|---|
| `crates/ns_v5` | Core 3D NS solver — all theorems, 27 modules, ~15K LOC | 212 pass |
| `crates/prime_hunt` | Multi-head Hydra prime sieve | 103 pass |
| `crates/qmnf_crtbigint` | CRTBigInt v3.1: 8×63-bit lanes, Montgomery mult, Garner reconstruction | embedded |
| `crates/qedde_v2` | QEDDE-D2 schema cell architecture | placeholder |
| `crates/mmbf_grover_bridge` | Ramanujan-Grover spectral search on CRT torus | placeholder |
| 22 others | Various helper/bridge modules | mostly placeholder |

### Key proof docs

| File | Role |
|---|---|
| `NS_REGULARITY_COMPLETE_PACKET.md` (1.9 MB) | Full formal manuscript, all 5 proof steps + corollaries |
| `NS_REGULARITY_PROOF_COMPLETE.md` | 7-step proof outline |
| `BRIDGE_ENGINE_DEEP_ANALYSIS.md` | Adelic identification + bridge analysis |
| `GAP_ASSESSMENT_D030.md` | Current open-gap punch list |
| `SMALL_DATA_REGULARITY_THEOREM.md` | Small-amplitude regularity result |
| `SOLENOID_NONEXPANSION_THEOREM.md` | p-adic non-expansion (proved) |
| `EXECUTION_PLAN_CONTINUUM_CLOSURE.md` | Write-up plan for Step 7 |
| `EXECUTIONER_DAG_*.md` | DAG checkpoints and final execution plan |
| `NS_REGULARITY_COMPLETE.tar`, `QMNF_WORKSPACE.tar` | Timestamped code snapshots |

### Key `ns_v5` modules (in-tree)

| Module | Lines | Role |
|---|---:|---|
| `src/lib.rs` | 681 | Theorems T1-T10, contraction bounds, `decompose`, `DISC_PRIMES` |
| `src/ns3d.rs` | 1326 | NTT Biot-Savart, Laplacian, advection, stretching, time stepper |
| `src/ns3d_energy.rs` | 814 | Kill #113, Kill #121, skew stepper, Aubin-Lions conditions |
| `src/ns3d_sobolev.rs` | 1694 | H¹/H² seminorms, sharp embedding, stretching measurement |
| `src/ns3d_wassan.rs` | 664 | WASSAN-backed persistent vorticity; CRT-BigInt H² norm |
| `src/ns3d_d030.rs` | 317 | **D-030 PDE bound** `‖ω‖_∞ ≤ (p−1) + √E(0)` — 9 tests |
| `src/ns3d_aubin_lions.rs` | — | Conditions (i)-(iii) verification |
| `src/ns3d_n16.rs`, `src/ns3d_n32.rs` | — | Extended-grid solvers (partial) |
| `src/recumbent.rs` | 751 | Tiered overflow — primary + overflow + hot lanes |
| `src/wassan.rs` | 326 | Holographic key-value persistence |

---

## `lean/` — Lean 4 formalization

**Provenance:** pre-existing Lean project  
**Size:** 316 KB  
**Status:** 🚧 in progress — 10 `sorry`s remaining

| File | Role | `sorry`s |
|---|---|---:|
| `NSRegularity/Basic.lean` | Base definitions (VField, ops) | 0 |
| `NSRegularity/Kill113.lean` | **Top-level Kill #113 theorem** (composition of component zeros) | 1 |
| `NSRegularity/Kill113/Component.lean` | Per-component zero lemma (L1.8) | 1 |
| `NSRegularity/Shift.lean` | Shift lemmas index | 0 |
| `NSRegularity/Shift/Defs.lean` | Shift definitions | 0 |
| `NSRegularity/Shift/SnSp.lean`, `SpSn.lean`, `SumSn.lean`, `SumSp.lean` | Shift algebra lemmas | 2 each |
| `NSRegularity.lean` | Top import | 0 |

Build: `cd lean && lake build`

---

## `recumbant_crt/` — Recombinant CRT research

**Provenance:** `recumbantcrt.zip`  
**Size:** 1.6 MB, 48 files  
**Status:** research artifacts (self-contained sessions)

Key files:
- `HACKFATE_BLUEPRINT_v1.md` — blueprint overview (21 KB)
- `GAP_SCOUT_NS_SESSION_2026-03-10.md` — gap scout report (31 KB)
- `hydra_v3_stress.py`, `hydra_optimal_ops.py`, `summit_scaffold.py` — Hydra benchmarks
- `cram_yukawa_schema.py` — Yukawa CRAM schema
- `recombinant_crt_session_2026-03-11.tar.gz` — complete session snapshot (148 KB)
- Sub-tarballs: `01_safe_basis_extension.tar.gz` through `05_summit_computation.tar.gz`
- `SUPPLEMENTAL_TECHNICAL_PACKET.docx`

---

## `fp2_nttns/` — FP² NTT NS workspace

**Provenance:** `FP2nttns.zip`  
**Size:** 11 MB, 59 files  
**Status:** historical snapshots

Key files:
- `CRAM_CHIMERA_WHITE_PAPER.docx`, `CRAM_DEFINITIVE_WHITE_PAPER.docx`, `CRAM_ERRATA_AND_MHD_EXTENSION.docx`
- `CORPUS_BACKTRACE.md`, `EXECUTION_PLAN.md`
- `fp2_ns_prototype.py` — early FP² prototype
- `setup_env.sh` — environment bootstrap
- 7 nested tarballs: `qmnf_complete_2026-03-28.tar.gz`, `qmnf_ns_workspace.tar.gz`, `qmnf_ns_exact_bs.tar.gz`, `qmnf_ns_ntt_complete.tar.gz`, `qmnf_unified_v2.6.tar.gz` (747 KB), `qmnf_execution_wave2.tar.gz` (534 KB), `NS_REGULARITY_COMPLETE.tar.gz`

---

## `subcriticality/` — DKAM subcriticality + mixed material

**Provenance:** `subcriticalility.zip` (sic)  
**Size:** 4.4 MB, 46 files  
**Status:** mixed — math content alongside astrology engines

Math files:
- `DKAM_SUBCRITICALITY_FORMALIZATION.py` (20 KB) — DKAM degree-vs-resonance formalization
- `NS_CRT_FORMALIZATION.py` — NS CRT formalization
- `NS_EXECUTION_DAG.py` — execution DAG
- `T_SHADOW_v2_FORMALIZATION.docx` — T-shadow formalization
- `gear_manifold_v3.py`, `cram_planetary_schema.py`

Astrology files (bundled in the same drop; keep or relocate):
- `astrology_engine.py`, `astrology_support.py`, `astrology_system.py`, `astrology_wrapper.py`
- `natal_chart.py`, `transit_engine.py`, `house_data.py`, `vedic_layer.py`
- `ephemeris_offline.py`, `ephemeris_corrections.py`

---

## `fall_of_regularity/` — NS paper workspaces

**Provenance:** `fallofregularity.zip`  
**Size:** 2.1 MB, 28 files  
**Status:** paper drafts + active-path snapshots

Key files:
- `NS_REGULARITY_PAPER.docx` — paper draft
- `EXACT_INTEGER_NAVIER_STOKES.docx` — technical writeup
- `QMNF_RIGOROUS_RESTATEMENT.md`, `NS_PREP.md`, `NS_REGULARITY_HANDOFF.md`
- `PARTITION_FUNCTION_CONNECTION.md`
- `COMPLETE_DISTANCE_ANALYSIS.md`, `CORRECTED_DISTANCE_ANALYSIS.md`, `FAIL_POINT_GENEALOGY.md`
- `energy_certificate.rs` — Rust energy certificate
- Nested tarballs: `ns_active_path_v8_final.tar.gz`, `ns_active_path_v10.tar.gz`, `ns_regularity_complete.tar.gz`, `qmnf_unified_complete.tar.gz` (842 KB), `qmnf_unified_workspace.tar.gz`

---

## `dresden_prime_hunt/` — Dresden prime-hunt package

**Provenance:** `dresden.zip` (= `dres.zip`, duplicate skipped)  
**Size:** 1008 KB, 19 files  
**Status:** papers + research plan

Key files:
- `PAPER_1_COPRIME_LATTICE_STRATIFICATION.docx` — paper 1
- `PAPER_3_CRT_NS_SUBCRITICALITY.docx` — paper 3
- `DRESDEN_PRIME_HUNT_REPORT.docx`, `DRESDEN_PRIME_HUNT_SYNTHESIS.md`
- `COMPREHENSIVE_RESEARCH_PLAN.md`
- `FORMALIZATION_T11_T15_INTRINSIC_CRAM_ADDRESS.md`
- `EXECUTIONER_TRUTH_PERTURBER_REPORT.md`, `INSIGHT_MINING_REPORT.md`
- `PRIME_HUNT_RESULTS.md` — hunt results summary
- `dresden_codex_bridge.rs` — Rust bridge
- Nested: `dresden-prime-hunt.tar.gz`, `prime_hunt.tar.gz`

---

## `gambit_cram/` — CRAM slot predictor & harness

**Provenance:** `gambit.zip`  
**Size:** 608 KB, 15 files  
**Status:** harness + reports

Key files:
- `CRAM_SLOT_PREDICTOR_BLUEPRINT.md`, `CRAM_Slot_Predictor_Workspace.tar.gz` (41 KB)
- `CRAM_CAPABILITY_REPORT.md`
- `CRT_SLINGSHOT_THEOREM_STACK.md`
- `cram_universal_harness.py`, `cram_harness_v2.py`, `edge_probe.py`
- `gambit_main.rs`, `gambit_Cargo.toml` — Rust gambit
- `hydra_atlas_v2.json` — Hydra atlas data
- `cram-mvp.jsx` — web MVP (JSX)
- `gap_scout_session.md`

---

## `qtc_continuum/` — Quantum Topological Continuum

**Provenance:** `qtc continuum.zip`  
**Size:** 6.1 MB, 11 files  
**Status:** blueprints + visualizations

Key files:
- `CRAM_Explorer_Blueprint_Apr5_2026.md` (36 KB) — blueprint
- `CRAM_Explorer_Manifest_Apr5_2026.md`, `.docx`
- `QMNF_Session_Compendium_Apr5_2026.docx`
- `CRAM_Explorer_Apr5_2026.tar.gz` (3.2 MB) — full Explorer snapshot
- `bridge-engine.zip` — bridge engine
- Scanner images (PNG): `qmnf_spectral_scanner.png`, `qmnf_scanner_vortex.png`, `qmnf_lane_audit.png`, `qmnf_precise_audit.png`, `qmnf_shadow_disambiguator.png`

---

## `session_workspaces/` — Sieve oracle + session runs

**Provenance:** `workspaces.zip`  
**Size:** 376 KB, 31 files  
**Status:** working materials

Key files:
- `SESSION_1_SUMMARY.md`, `SESSIONS_1-3_COMPREHENSIVE.md`
- `WORKSPACE_MANIFEST.md` — inline manifest
- `sieve_oracle_spec.md`, `sieve_oracle_results.md`
- `t2_3_run.py`, `t3_4_run.py` — tier-2/3/4 run scripts
- `head_b_fixed.py`, `head_d_fixed.py`, `consensus_fixed.py` — Hydra head fixes
- `projector_signatures.json`, `hydra_projector_comparison.json`
- Per-slot markdowns `2026-04-13-SLOT-00{1,2,3}.md`

---

## `skills/` — prime-family-spacetime

**Provenance:** `prime-family-spacetime-skill.tar.gz`  
**Size:** 20 KB  
**Status:** single skill package

Contains `prime-family-spacetime/` — a Claude Code skill package.

---

## `_archives/` — raw original drops (local only)

**Gitignored.** The original `.zip` / `.tar` files kept locally for reference:

```
_archives/
├─ recumbantcrt.zip           (1.2 MB)
├─ FP2nttns.zip               (11 MB)
├─ subcriticalility.zip       (3.6 MB)
├─ fallofregularity.zip       (1.9 MB)
├─ dres.zip, dresden.zip      (829 KB each — duplicates)
├─ gambit.zip                 (241 KB)
├─ qtc continuum.zip          (5.8 MB)
├─ workspaces.zip             (94 KB)
├─ NS Regularity.zip          (3.5 MB — duplicates main workspace)
├─ NS_REGULARITY_COMPLETE.tar, NS_REGULARITY_COMPLETE (1).tar  (6.4 MB each)
└─ prime-family-spacetime-skill.tar.gz  (7.1 KB)
```

---

## Cross-references

### Theorem / claim → supporting artifacts

| Claim | Rust | Lean | Paper/doc |
|---|---|---|---|
| Kill #113 (advection energy ≡ 0) | `ns3d_energy.rs` | `lean/NSRegularity/Kill113.lean` (1 `sorry`) | `NS_REGULARITY_PROOF_COMPLETE.md` |
| Kill #121 (per-lane energy) | `ns3d_energy.rs` | — | `NS_REGULARITY_PROOF_COMPLETE.md` |
| DKAM 90/1001 | `lib.rs` (T1–T10) | — | `NS_REGULARITY_COMPLETE_PACKET.md` |
| Sharp Sobolev | `ns3d_sobolev.rs` | — | `NS_REGULARITY_COMPLETE_PACKET.md` |
| D-030 L∞ bound | `ns3d_d030.rs` | — | `GAP_ASSESSMENT_D030.md` |
| p-adic non-expansion | `ns3d_n32.rs::padic_non_expansion_d024` | — | `SOLENOID_NONEXPANSION_THEOREM.md` |
| Aubin-Lions (I)(II) | `ns3d_aubin_lions.rs`, `ns3d_energy.rs` | — | Step 5 of proof |
| Aubin-Lions (III) | **argued** — circularity check pending | — | — |
| D-024 Hasse bridge | — | — | **OPEN** |

### Folder → research session timeline

| Folder | Dates |
|---|---|
| `recumbant_crt/` | 2026-03-10 / 2026-03-11 sessions |
| `fp2_nttns/` | 2026-03-28 complete + wave-2 execution |
| `qtc_continuum/` | 2026-04-05 Explorer blueprint |
| `session_workspaces/` | 2026-04-13 slots 1-3 |
| `NS_REGULARITY_COMPLETE (1)/` | 2026-04-17 canonical |
