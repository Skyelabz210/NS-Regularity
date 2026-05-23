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
- `lean/` — Lean formalization line.
- `Skyelabz210/ns3d-n32-ntt6` — N32 six-prime spectral satellite.
- `fall_of_regularity/`, `fp2_nttns/`, `qtc_continuum/`, `recumbant_crt/` — bridge and CRT formations.

## Main Rust lanes

- `src/ns3d.rs` — core 3D solver and NTT Biot-Savart lineage.
- `src/ns3d_energy.rs` — Kill #113, Kill #121, energy identities.
- `src/ns3d_sobolev.rs` — H1/H2 seminorms and Sobolev route.
- `src/ns3d_d030.rs` — D-030 winding-energy route.
- `src/ns3d_aubin_lions.rs` — compactness route.
- `src/recumbent.rs`, `src/wassan.rs`, `src/ns3d_wassan.rs` — Recumbent/WASSAN route.

## Lean lanes

- `lean/NSRegularity/Kill113*`
- `lean/NSRegularity/DKAM/`
- `lean/NSRegularity/HSquared/ResolutionA.lean`
- `lean/NSRegularity/NS/BiotSavart.lean`
- `lean/NSRegularity/NS/FullStep.lean`
- `lean/NSRegularity/NS/Trajectory.lean`
- `lean/NSRegularity/BKM.lean`

## Execution posture

Use the route files to locate the child owner, then wire forward through the correct workspace, feature path, or adapter. Top-level docs should prefer `ROUTED`, `ACTIVE`, or `CHILD OWNER` language when a child formation exists.
