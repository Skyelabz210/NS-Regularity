# Deep Research Framework — Active 3D Path Only
# (Trimmed from earlier session handoff; archived 1D work removed from active plan)

## What is already closed in this workspace

| Layer | Status | Notes |
|---|---|---|
| CRT executor / exact arithmetic | Closed | Zero-float production path; ExactRational + CRTBigInt are active. |
| Recumbent overflow geometry | Closed | Primary / overflow / hot tiers, demotion, reconstruction invariance. |
| WASSAN structural persistence | Closed | Stores full structural state, not flattened integers. |
| 1D proxy separation | Closed | 1D code is archived as regression-only and is not part of the active API. |

## What remains active

### A. 3D theorem-bearing work only
1. Wire or verify the exact 3D Biot-Savart kernel path at the next active resolution.
2. Measure boundedness / return-to-corridor on the 3D solver only.
3. Keep all evidence tagged by dimension and do not let archived 1D material re-enter theorem claims.

### B. Continuum bridge only
1. Exact discrete Biot-Savart consistency theorem.
2. Compactness and nonlinear passage written in full.
3. Uniform 3D return/excursion bound strong enough for the limit.
4. BKM only as a terminal corollary.

## What is no longer active

The following are archived and should not drive agent planning unless explicitly requested for debugging:
- 1D proxy overshoot studies
- 1D corridor dynamics as proof evidence
- any task list whose main purpose is the 1D proxy
- stale float-eradication tasks already completed
- stale recumbent/WASSAN plumbing tasks already completed

## Active proof spine

```text
CRTBigInt exact executor
    -> Recumbent exact overflow geometry
    -> WASSAN structural persistence
    -> 3D discrete solver evidence
    -> exact 3D operator consistency
    -> compactness / nonlinear passage
    -> continuum regularity corollary
```

## Agent guardrail

If a task is not directly advancing the 3D solver, the exact 3D operator family,
or the continuum bridge, it is probably archived work.
