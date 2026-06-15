# CLAUDE.md

Guidance for AI agents working in this repository.

## Route-first workflow

Historical status labels are route hints. This repository contains nested child formations with follow-up work for earlier labels. Start with:

1. `SYNC_STATUS.md`
2. `ROUTING_MAP.md`
3. `AUDIT_LEDGER.md`
4. `NS_REGULARITY_COMPLETE (1)/crates/ns_v5/docs/OPEN_SEAMS.md`
5. `lean/PROOF_STATUS.md`

Then continue from the child owner. Nested work is build-lineage material.

## Main commands

```bash
cd "NS_REGULARITY_COMPLETE (1)/crates/ns_v5"
cargo test
cargo build
cargo check
```

```bash
cd lean
lake build
```

## Active roots

- `NS_REGULARITY_COMPLETE (1)/crates/ns_v5` — main Rust executor and theorem harness.
- `NS_REGULARITY_COMPLETE (1)/crates/ns_v5/qmnf_crtbigint` — DynCRT growable carrier (Transfinite Architecture, Wave I). Substrate underneath the Rust executor; ≈2^504 native lanes via `DynBasis::{canonical, extend, capacity_bits}`.
- `lean/` — Lean formalization line.
- `Skyelabz210/ns3d-n32-ntt6` — N32 six-prime spectral satellite.
- `fall_of_regularity/`, `fp2_nttns/`, `qtc_continuum/`, `recumbant_crt/` — bridge and CRT formations.
- `qcid_ns_bridge` — Beltrami H² closure / bootstrap probe; **adapter needed** (latest snapshot in upload `NS_REGULARITY_FINAL_v2.tar.gz`; historical snapshots in `fp2_nttns/qcid_ns_bridge*.tar.gz`).

## Main Rust lanes

- `src/ns3d.rs` — core 3D solver and NTT Biot-Savart lineage (RNS+CRT spectral lane).
- `src/ns3d_energy.rs` — Kill #113, Kill #121, energy identities.
- `src/ns3d_sobolev.rs` — H1/H2 seminorms and Sobolev route.
- `src/ns3d_d030.rs` — D-030 winding-energy route.
- `src/ns3d_aubin_lions.rs` — compactness route.
- `src/recumbent.rs`, `src/wassan.rs`, `src/ns3d_wassan.rs` — Recumbent/WASSAN route.
- `src/cram_ns.rs` — CRAM lane (distinct from the RNS+CRT spectral lane).
- `src/ns3d_adversarial.rs` — adversarial seed harness.
- `src/ns3d_n16.rs`, `src/ns3d_n32.rs` — N16 / N32 prime-set lanes (safe-basis primes for DKAM contraction).
- `src/ns3d_stencil_bs.rs` — stencil Biot-Savart.
- `qmnf_crtbigint/src/dyncrt.rs` — DynCRT carrier (`DynBasis::{canonical, extend, capacity_bits}`).

## Lean lanes

- `lean/NSRegularity/Kill113*`
- `lean/NSRegularity/DKAM/`
- `lean/NSRegularity/HSquared/ResolutionA.lean`
- `lean/NSRegularity/HSquared/DiscreteH2.lean` — Kill #129 eigen-Beltrami commutator (sorry-free).
- `lean/NSRegularity/HSquared/BasisCFL.lean` — Kill #130/#131 centered residue + sign recoverability (sorry-free).
- `lean/NSRegularity/NS/BiotSavart.lean`
- `lean/NSRegularity/NS/FullStep.lean`
- `lean/NSRegularity/NS/Trajectory.lean`
- `lean/NSRegularity/NS/AdvectionBound.lean` + companion files (`HStepSmallData.lean`, `SmallDataCapstone.lean`, `BiotSavartBound.lean`, `DiffusionCeiling.lean`, `StretchingBound.lean`, `TrivialClosure.lean`, `Defs.lean`) — operator-side h_step factoring; closure recipe in `NSRegularity/NS/AdvectionBound_CLOSURE_RECIPE.md`.
- `lean/NSRegularity/AxiomAudit.lean` + `lean/AXIOM_LEDGER.md` — FN-L3 live-path axiom audit discipline.
- `lean/NSRegularity/BKM.lean`

## Campaign register

| Campaign | Wave | Status | Owner |
|---|---|---|---|
| Transfinite Architecture | Wave I | R1 closed (Δ=0 across asym sweep, `CAMPAIGN_TRANSFINITE_R16.md`); R2 production port into `ns_v5` pending | `qmnf_crtbigint/src/dyncrt.rs` |
| Beltrami / Kill #129 | H² closure | sorry-free | `HSquared/DiscreteH2.lean` |
| Basis-CFL / Kill #130/#131 | basis recovery | sorry-free | `HSquared/BasisCFL.lean` |
| Small-data h_step | NS skeleton | capstone landed (parallel L∞/h_step track); Pieces A & B open | `NS/SmallDataCapstone.lean`, `NS/AdvectionBound*` |
| FN-L3 axiom isolation | audit | live-path traces to 4 `biot_savart*` axioms | `NSRegularity/AxiomAudit.lean`, `lean/AXIOM_LEDGER.md` |
| NTT Biot-Savart port | open | discharges all 4 axioms when landed | `lean/NSRegularity/NS/BiotSavart.lean` ← `src/ns3d.rs` |

## Scope rules

1. **Do not modify `lean/` unless the task explicitly names a Lean lane.** The Lean tree is the formalization line; drift here breaks the axiom audit and the `lake build` clean state. If a task touches Rust only, `lean/` is off-limits.
2. **CRAM ≠ RNS+CRT.** The CRAM lane is `src/cram_ns.rs`. The RNS+CRT spectral lane is `src/ns3d.rs` + `qmnf_crtbigint`. They share a substrate but compose differently; conflating them silently breaks the safe-basis primes invariant.
3. **Safe-basis primes are {7, 11, 13}** for the DKAM contraction B ≈ 0.032 < 1 (Beltrami theorem stack §L5, V04). Any lane that skips this set is not part of the certified route.
4. **Activation, not flattening.** Nested artifacts (crates, tarballs, probes) belong to specific roots. If a workspace reference appears broken, locate the activation root rather than declaring orphan (see `SYNC_STATUS.md` § Activation discipline).

## Execution posture

Use the route files to locate the child owner, then wire forward through the correct workspace, feature path, or adapter. Top-level docs should prefer `ROUTED`, `ACTIVE`, or `CHILD OWNER` language when a child formation exists.
