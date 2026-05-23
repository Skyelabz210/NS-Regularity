# Execution Board

Last corrected: 2026-05-23.

This file keeps forward motion. It maps each item to its active owner and the next executable move.

| Item | Active owner | Execute next |
|---|---|---|
| D-030 | `NS_REGULARITY_COMPLETE (1)/crates/ns_v5/src/ns3d_d030.rs` | Wire dependent docs to this route |
| Sobolev transfer | `docs/STATUS_MATRIX.md`, `src/ns3d_sobolev.rs` | Promote scaling evidence into the active route layer |
| Aubin-Lions checks | `src/ns3d_aubin_lions.rs`, `src/ns3d_energy.rs` | Keep compactness checks attached to the main route |
| N16 path | `STATUS.md` | Treat N16 prime set and uniform table as active child evidence |
| N32 path | `Skyelabz210/ns3d-n32-ntt6` | Continue from P5B-9A into six-prime spectral path |
| Lean Biot-Savart | `lean/NSRegularity/NS/BiotSavart.lean` | Keep route attached to Rust/N32 Biot-Savart lineage |
| Full step control | `lean/NSRegularity/NS/Trajectory.lean`, `lean/NSRegularity/NS/FullStep.lean` | Keep route attached to DKAM and D-030 child owners |
| L2.3 | `lean/NSRegularity/DKAM/DiffusionMaxPreserving.lean` | Use Resolution-A exactness route |
| L3.4 | `lean/NSRegularity/HSquared/ResolutionA.lean` | Use Wave F.5 route |
| L3.3 | `src/ns3d_sobolev.rs` | Keep exact rational Sobolev witness in active route |
| L9.2 | `lean/NSRegularity/ThreeStream.lean` | Keep corrected-statement route attached |
| D-024 | `fall_of_regularity/`, `fp2_nttns/`, `qtc_continuum/` | Keep bridge work attached to the bridge family |
| BKM item | `lean/NSRegularity/BKM.lean` | Keep theorem route attached to Lean BKM module |

Execution rule: if a child owner exists, use it. Route, wire, and continue.
