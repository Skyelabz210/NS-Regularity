# Route Map

_Last corrected: 2026-05-23._

This file maps old top-level labels to the child workspace or module that owns the follow-up work.

| Label | Child owner | Handling |
|---|---|---|
| Super-threshold stretching / D-030 | `NS_REGULARITY_COMPLETE (1)/crates/ns_v5/src/ns3d_d030.rs` | Routed to D-030 winding-energy bound and tests |
| Uniform Sobolev transfer | `NS_REGULARITY_COMPLETE (1)/crates/ns_v5/docs/STATUS_MATRIX.md`; `src/ns3d_sobolev.rs` | Routed to C_phys(N) descent evidence and exact N=8 embedding |
| Aubin-Lions discrete checks | `src/ns3d_aubin_lions.rs`; `src/ns3d_energy.rs` | Routed to child compactness checks |
| N16 extension | `NS_REGULARITY_COMPLETE (1)/crates/ns_v5/STATUS.md` | Routed to N16 prime-set section and uniform property table |
| N32 wider spectral wall | satellite `Skyelabz210/ns3d-n32-ntt6`; `P5B_DAG_P5B9A_GATE.md` | Routed to P5B-9A gate-pass and six-prime BS path |
| Biot-Savart Lean port | `lean/NSRegularity/NS/BiotSavart.lean`; Rust `src/ns3d.rs`; N32 satellite | Routed to constructive port owner |
| Full NS step condition | `lean/NSRegularity/NS/Trajectory.lean`; `lean/NSRegularity/NS/FullStep.lean`; `lean/NSRegularity/DKAM/` | Routed to operator-side composition owner |
| L2.3 exact diffusion | `lean/NSRegularity/DKAM/DiffusionMaxPreserving.lean` | Routed to Resolution-A exact-division proof |
| L3.4 H2 monotonicity | `lean/NSRegularity/HSquared/ResolutionA.lean`; `lean/PROOF_STATUS.md` | Routed to Wave F.5 Resolution-A proof |
| L3.3 sharp Sobolev | `src/ns3d_sobolev.rs` | Routed to Rust exact rational witness; Lean port remains owned by Lean child |
| L9.2 corrected statement | `lean/NSRegularity/ThreeStream.lean` | Routed to corrected sibling path |
| Winding transport | `lean/`; `recumbant_crt/`; `src/ns3d_wassan.rs`; `src/recumbent.rs` | Routed to lineage strata and activation root |
| D-024 bridge | `fall_of_regularity/`; `fp2_nttns/`; `qtc_continuum/`; D-024 probe lineage | Routed to bridge workspace family |

Rule: when a child owner exists, the top level records the route. Remaining work is activation inside the listed child roots.
