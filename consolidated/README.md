# Navier–Stokes consolidation hub

Consolidation date: **2026-10-14**. Primary owner: Anthony Diaz / Skyelabz210.

This directory is an indexed, non-destructive import of NS research from connected GitHub repositories, with a separate intake audit of the session uploads. The pre-existing main development roots continue to own runnable code:

- Rust 3D primary: [NS_REGULARITY_COMPLETE (1)/crates/ns_v5](../NS_REGULARITY_COMPLETE%20%281%29/crates/ns_v5).
- Existing exact NS grid: [crates/nsgrid3d](../NS_REGULARITY_COMPLETE%20%281%29/crates/nsgrid3d).
- Existing Lean active proof project: [lean/](../lean/).
- Historical provenance: [WORKSPACE_MANIFEST.md](../WORKSPACE_MANIFEST.md), [ROUTING_MAP.md](../ROUTING_MAP.md), [SYNC_STATUS.md](../SYNC_STATUS.md).

## Newly imported, SHA-verified GitHub source

1. `solvers/n32_six_prime/`: `ns3d-n32-ntt6` N32 CRT v1..v4, 6-prime NTT, three exact-division paths, P5B gate documents.
2. `bridges/qcid_satellite/`: QCID bridge crate as its complete satellite variant, including H², calibration, FP², step chimera, SQR, and winding components.
3. `formal/ns_lean/`: the separate `ns-lean` Lean project, with its own Lake metadata.
4. `bridges/n32_satellite/`: D-024 April 27 notebook.

The copies preserve exact upstream Git blob SHA values and existing module content. Imports are intentionally isolated rather than replacing newer main-root modules with older satellite revisions. See `SOURCE_PROVENANCE.md` for original repositories and `INTEGRATION_DAG.md` for promotion gates.

## Documentation

- [CLAIM_AUDIT.md](CLAIM_AUDIT.md): results, assumptions and mathematical counterchecks.
- [SOURCE_PROVENANCE.md](SOURCE_PROVENANCE.md): GitHub sources, branch/commit evidence and archive intake.
- [INTEGRATION_DAG.md](INTEGRATION_DAG.md): executable dependency ordering.
- [VALIDATION_COMMANDS.md](VALIDATION_COMMANDS.md): build/test and gate checklist.
- [session-2026-10-14/ARCHIVE_INVENTORY.md](session-2026-10-14/ARCHIVE_INVENTORY.md): uploaded archive audit.
- [session-2026-10-14/CONFLICT_REGISTER.md](session-2026-10-14/CONFLICT_REGISTER.md): version collisions requiring comparison.

**Publication standard:** a finite-grid identity, a passing program test, a Lean theorem with remaining axioms, and an analytic global-regularity theorem are different kinds of evidence. Theorem status must be linked to its actual hypotheses and verified command.

The classical three-dimensional incompressible Navier–Stokes global smooth regularity problem is not yet established by the source material audited here. The dependency register identifies the remaining mathematical bridge explicitly.
