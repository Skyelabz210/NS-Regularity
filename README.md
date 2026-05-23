# NS Regularity

A computational framework for 3D incompressible Navier-Stokes exact-integer analysis using CRT arithmetic, WASSAN persistence, Recumbent tiering, and Lean formalization.

Every numerical path is intended to stay in integer or exact-rational form: vorticity, velocity, Sobolev norms, Biot-Savart kernels, residue lanes, and proof witnesses.

## Status

| Layer | Status | Evidence / child owner |
|---|---|---|
| Executor substrate: CRTBigInt, WASSAN, Recumbent | ACTIVE | 212/212 tests in `NS_REGULARITY_COMPLETE (1)/crates/ns_v5` |
| Discrete regularity at N8 | ACTIVE | Kill #113, Kill #121, DKAM 90/1001, sharp Sobolev |
| Discrete regularity at N16 and N32 | ROUTED | N16 table in `crates/ns_v5/STATUS.md`; N32 satellite `Skyelabz210/ns3d-n32-ntt6` |
| D-030 winding-energy bound | ROUTED / ACTIVE | `crates/ns_v5/src/ns3d_d030.rs`, 9 tests |
| Prime-hunt / Hydra sieve | ACTIVE | 103/103 tests in `crates/prime_hunt` |
| Lean 4 formalization | ACTIVE | `lean/PROOF_STATUS.md`, Wave H.15 route map |
| Continuum bridge work | ROUTED | `fall_of_regularity/`, `fp2_nttns/`, `qtc_continuum/`, D-024 probe lineage |
| Child-owner routing | ACTIVE | `ROUTING_MAP.md`, `SYNC_STATUS.md` |

## Quickstart

```bash
cd "NS_REGULARITY_COMPLETE (1)/crates/ns_v5"
cargo test
cd ../prime_hunt && cargo test
```

For Lean:
```bash
cd lean && lake build
```

## Directory layout

```
├─ NS_REGULARITY_COMPLETE (1)/   Main workspace: Rust crates, proof docs
│   ├─ crates/ns_v5/             Core 3D NS solver and theorem harness
│   ├─ crates/prime_hunt/        Multi-head Hydra sieve
│   ├─ crates/qmnf_crtbigint/    CRTBigInt v3.1 arithmetic substrate
│   └─ *.md                      Proof manuscripts, status docs, route maps
├─ lean/                         Lean 4 formalization line
├─ recumbant_crt/                Recombinant CRT lineage
├─ fp2_nttns/                    FP2 NTT snapshots and CRAM papers
├─ subcriticality/               DKAM subcriticality lineage
├─ fall_of_regularity/           NS paper workspaces and bridge docs
├─ dresden_prime_hunt/           Dresden prime-hunt papers and plans
├─ gambit_cram/                  CRAM slot predictor and harnesses
├─ qtc_continuum/                Quantum topological continuum lineage
├─ session_workspaces/           Sieve-oracle and T-run workspaces
├─ skills/                       prime-family-spacetime skill package
├─ ROUTING_MAP.md                Old label to child-owner route map
├─ SYNC_STATUS.md                Repository sync map
├─ CLAUDE.md                     AI-agent guidance
└─ WORKSPACE_MANIFEST.md         Detailed inventory
```

See `WORKSPACE_MANIFEST.md`, `SYNC_STATUS.md`, and `ROUTING_MAP.md` for the active route layer.

## Proven / active finite-substrate results

- Kill #113: skew-symmetric advection energy neutrality, exact integer zero.
- Kill #121: per-prime-lane energy residual zero.
- DKAM 90/1001 contraction bound.
- Sharp Sobolev embedding at N8 with exact rational constant.
- D-030 winding-energy bound in `ns3d_d030.rs`.
- p-adic lane non-expansion and CRT lane stability.
- Aubin-Lions discrete conditions at the finite-substrate layer.
- Recumbent and WASSAN persistence checks.

## Routed bridge labels

The old top-level blocker list has been replaced by child-owner routing. Use:

- `ROUTING_MAP.md` for old label to child owner.
- `SYNC_STATUS.md` for repository-level activation roots.
- `NS_REGULARITY_COMPLETE (1)/crates/ns_v5/docs/OPEN_SEAMS.md` for routed seam ownership.
- `lean/PROOF_STATUS.md` for Lean route status.

## Claim discipline

The finite exact-integer substrate has strong closed components. The Lean diffusion certificate is formalized under Resolution A exactness. The full-NS Lean theorem is routed through the listed child owners. The continuum bridge route lives in the bridge workspace family and D-024 probe lineage.

## Author

Anthony Diaz (Acid) — HackFate / Skyelabz210
