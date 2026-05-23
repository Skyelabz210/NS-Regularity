# NS-Regularity Sync Status

_Last corrected: 2026-05-23._

This file is the repository-level sync map for nested workspaces, archived drops, active crates, satellite repositories, and Lean formalization waves.

## Canonical rule

Nested material is build-lineage material. A nested crate, archive, probe, branch, or workspace is presumed to fit the top-level architecture because it belongs to the sequence that produced the proceeding version. The audit question is therefore which root, adapter, feature gate, module export, or version bridge activates the artifact.

Do not classify nested work as orphan scratch merely because it does not compile from a reduced historical root.

## Active roots

| Root | Role | Status |
|---|---|---|
| `NS_REGULARITY_COMPLETE (1)/` | Main Rust proof/executor workspace | ACTIVE |
| `NS_REGULARITY_COMPLETE (1)/crates/ns_v5` | Core exact-integer NS crate | ACTIVE |
| `lean/` | Lean 4 formalization line | ACTIVE through Wave H.15 |
| `Skyelabz210/ns3d-n32-ntt6` | N=32 / six-prime NTT satellite | ACTIVE satellite, P5B-9A gate-pass |
| Uploaded `ns_active_path_v7.tar.gz` / old `STATUS.md` line | Historical N=8 substrate snapshot | ARCHIVED lineage stratum |

## Lineage strata

| Stratum | Activation root | Meaning |
|---|---|---|
| v3 roundtrip probes | matching v3/v4/v5 probe harness root | Historical equivalence and regression probes |
| v4 corridor / edge probes | v4/v5 Recumbent transition root | Overflow/corridor transition evidence |
| v5 BS pipeline / bisect / stress probes | active `ns_v5` lineage root or N32 satellite root | Biot-Savart and CRT stress evidence |
| WASSAN / Recumbent probes | `crates/ns_v5` with WASSAN modules | Persistent exact-vorticity storage evidence |
| D-024 probes | D-024 / continuum bridge root | Bridge classification evidence |
| N32 NTT6 probes | `Skyelabz210/ns3d-n32-ntt6` | N=32 six-prime spectral lineage |
| Lean Wave A-H files | `lean/` | Formal proof stack lineage |
| nested tarballs / zips | their manifest-declared root | Frozen generational snapshots |

## Current status documents

| File | Status |
|---|---|
| `SYNC_STATUS.md` | Repository-level sync map |
| `WORKSPACE_MANIFEST.md` | Directory inventory and provenance map |
| `lean/PROOF_STATUS.md` | Active Lean proof-status map; corrected after Wave H merge |
| `NS_REGULARITY_COMPLETE (1)/crates/ns_v5/STATUS.md` | Historical + crate-local status; contains older N=8 baseline plus later appended April 4 campaign data |

## Activation discipline

1. Preserve all nested lineage artifacts.
2. Activate artifacts through the correct root rather than flattening them into the wrong generation.
3. When a file references modules absent from a reduced root, locate the proceeding root that contains those modules before declaring mismatch.
4. Document missing activation bridges as `adapter needed`, not `orphan`.
5. Keep proof claims separated into: finite substrate, formal Lean skeleton, empirical campaign, and continuum bridge.

## Current live gates

| Gate | Root | Status |
|---|---|---|
| Constructive Lean Biot-Savart port | `lean/` + Rust NTT source | OPEN |
| Full-NS `h_step` / L-infinity preservation | `lean/` + `ns_v5` operator facts | OPEN |
| BKM formal axiom replacement | `lean/` | OPEN |
| N32 true six-prime spectral Biot-Savart | `ns3d-n32-ntt6` | OPEN after P5B-9A gate-pass |
| N16/N32 uniform bridge evidence | `ns_v5` + N32 satellite | ACTIVE |
| D-024 / continuum bridge | bridge lineage | OPEN |

## Claim discipline

The finite exact-integer substrate has strong closed components. The Lean diffusion certificate is formalized under Resolution A exactness. The full-NS Lean theorem is currently a conditional skeleton. The continuum global theorem remains a bridge target until the active gates close.
