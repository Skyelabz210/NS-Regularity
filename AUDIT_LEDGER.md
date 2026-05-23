# Audit Ledger

Last corrected: 2026-05-23.

Routing identifies the owner of a work item. It does not mark the item finished.

| Item | Owner | State | Next discharge |
|---|---|---|---|
| D-030 | `NS_REGULARITY_COMPLETE (1)/crates/ns_v5/src/ns3d_d030.rs` | child evidence | verify stated scope against all-data and refinement requirements |
| Sobolev transfer | `docs/STATUS_MATRIX.md`, `src/ns3d_sobolev.rs` | child evidence | convert sampled scaling into a formal uniform theorem |
| Aubin-Lions checks | `src/ns3d_aubin_lions.rs`, `src/ns3d_energy.rs` | child evidence | connect finite checks to the final topology |
| N16 path | `STATUS.md` | child evidence | activate in build and verify normalization |
| N32 path | `Skyelabz210/ns3d-n32-ntt6` | child evidence | finish six-prime spectral path |
| Lean Biot-Savart | `lean/NSRegularity/NS/BiotSavart.lean` | port needed | replace axioms with construction |
| Full step control | `lean/NSRegularity/NS/Trajectory.lean`, `lean/NSRegularity/NS/FullStep.lean` | theorem needed | discharge step-control assumption |
| L2.3 | `lean/NSRegularity/DKAM/DiffusionMaxPreserving.lean` | conditional closure | discharge exactness side condition |
| L3.4 | `lean/NSRegularity/HSquared/ResolutionA.lean` | conditional closure | integrate exactness into full trajectory |
| L3.3 | `src/ns3d_sobolev.rs` | port needed | formalize exact rational argument |
| L9.2 | `lean/NSRegularity/ThreeStream.lean` | theorem needed | complete corrected statement |
| D-024 | `fall_of_regularity/`, `fp2_nttns/`, `qtc_continuum/` | bridge needed | prove the bridge statement |
| BKM item | `lean/NSRegularity/BKM.lean` | import or formalization needed | replace axiom path |

Audit rule: do not downgrade a hard item to finished merely because an owner folder exists.
