# ROUTED_SEAMS.md - Child-owned seam map

Last corrected: 2026-05-23.

This document used to list top-level open seams. Those labels are now routed to child owners that were added after the original blocker reports. A seam may still require activation or formal porting, but the repository should not treat it as an unlocated top-level blocker.

## Routed seams

| Seam | Child owner | Routed status |
|---|---|---|
| Super-threshold stretching bound | `../src/ns3d_d030.rs` | Routed to D-030 winding-energy bound and trajectory tests; algebraic bound `‖ω‖_∞ ≤ (p−1) + ⌈√E(0)⌉` witnessed at N ∈ {8, 16, 32} (`d030_bound_holds_n16`, `d030_bound_holds_n32`); `winding_bound_from_energy` is now Axiom-A1 compliant (no f64 on the trajectory hot path) |
| Uniform-in-N limit transfer | `STATUS_MATRIX.md`, `../src/ns3d_sobolev.rs`, `../src/ns3d_aubin_lions.rs`, `../src/ns3d_n16.rs`, `../src/ns3d_n32.rs` | Routed to Sobolev scaling, N16 table, Aubin-Lions child checks, and N ∈ {8, 16, 32} D-030 witnesses |
| T_FK(N) scaling | `../STATUS.md`, N32 satellite `Skyelabz210/ns3d-n32-ntt6` | Routed to N8 measurement, N16 prime-set readiness, and N32 satellite path |
| N^6 Biot-Savart normalization | `../src/ns3d.rs`, `../src/ns3d_sobolev.rs`, N32 satellite | Routed to exact NTT-BS lineage and wider spectral path |
| N16 verification | `../STATUS.md`, `STATUS_MATRIX.md` | Routed to final-session N16 prime-set and uniform property table |
| N32 six-prime extension | satellite `Skyelabz210/ns3d-n32-ntt6/P5B_DAG_P5B9A_GATE.md` | Routed to P5B-9A gate-pass lineage |

## Closed child evidence retained here

| Result | Child owner |
|---|---|
| Kill #113 | `../src/ns3d_energy.rs`, Lean `NSRegularity/Kill113*` |
| Kill #121 | `../src/ns3d_energy.rs` |
| Contraction B(N) <= 90/1001 | `../src/lib.rs` |
| Exact N8 Biot-Savart | `../src/ns3d.rs` |
| Sharp N8 Sobolev embedding | `../src/ns3d_sobolev.rs` |
| Bounded Sobolev scaling | `STATUS_MATRIX.md` |
| Aubin-Lions discrete checks | `../src/ns3d_aubin_lions.rs`, `../src/ns3d_energy.rs` |
| Recumbent stability | `../src/recumbent.rs` |
| WASSAN persistence | `../src/wassan.rs`, `../src/ns3d_wassan.rs` |
| Campaign decay evidence | `../STATUS.md` |

## Routing rule

When a child owner exists, keep the seam routed to that child owner. Do not duplicate the older blocker language at the top level. Remaining work belongs inside the listed child root or its activation adapter.
