# NS Regularity

Exact-integer and exact-rational research on incompressible Navier–Stokes flow on the periodic three-torus. The project connects finite-grid computation, CRT arithmetic, and Lean 4 verification. The fixed-grid Rust solvers, the original Lean model, and the newer rational velocity scheme have separate update rules.

## Research status

| Component | What it establishes |
|---|---|
| Fixed-grid Lean model | The original development proves finite periodic identities and conditional diffusion bounds. The proof ledger records assumptions and admitted declarations. [Lean proof status](lean/PROOF_STATUS.md) |
| Rounded diffusion and centered curl | The merged development proves a maximum principle for the rounded diffusion step at every grid size under CFL, and an exact shared-denominator heat update that keeps vorticity in the integer centered-curl image. [Transfer analysis](consolidated/NS_GAP_ANALYSIS.md) |
| Nonlinear refinement | The development specifies a separate rational velocity scheme with exact kinetic-energy dissipation. A written proof gives a subsequential limit to a periodic Leray–Hopf weak solution and global smoothness under an explicit small-data bound. [Transfer analysis](consolidated/NS_GAP_ANALYSIS.md) |
| Constructed continuum certificates | An exact Fourier predictor generates interval chains and a separate replay recomputes every residual and source match. Reproduced global certificates cover cyclic amplitudes through $1/4$ at viscosity one, unequal component amplitudes and a declared initial neighborhood. [Certificate search](consolidated/NS_CONTINUATION_CERTIFICATE.md#automated-search-and-independent-replay) |
| General three-dimensional regularity | Smoothness for arbitrary smooth initial data remains open. The legacy split Lean and Rust solvers each need a consistency and stability argument for their own updates. [Claim audit](consolidated/CLAIM_AUDIT.md) |

## Verify the current checkout

Use Python 3, Rust with Cargo, and the Lean toolchain pinned in [lean/lean-toolchain](lean/lean-toolchain). With `elan` installed, run these commands from the repository root.

```sh
(cd lean && lake exe cache get && lake build)
python3 scripts/verify_ns_source_blobs.py
python3 scripts/verify_biot_savart_domain.py
RUST_MIN_STACK=67108864 cargo test --locked --manifest-path 'NS_REGULARITY_COMPLETE (1)/crates/ns_v5/Cargo.toml'
```

The exact Python checks exercise the rounded stencil, centered inverse, rational nonlinear velocity step and continuum residual certificates. To generate and independently replay a continuum certificate for cyclic Fourier data, run the commands below.

```sh
python3 scripts/certify_fourier_flow.py --amplitude 1/12 --step 1/1024 --max-steps 48 --output /tmp/ns-certificate.json
python3 scripts/certify_fourier_flow.py --amplitude 1/12 --verify /tmp/ns-certificate.json
python3 -O scripts/verify_certificate_search.py
```

[Certificate inputs and mathematical scope](consolidated/NS_CONTINUATION_CERTIFICATE.md#automated-search-and-independent-replay)

## Repository map

- [lean/](lean/) contains the primary Lean formalization and axiom ledger.
- [consolidated/](consolidated/) indexes the claim audit, imported source provenance, independent Lean project, and bridge experiments.
- [Primary Rust workspace](NS_REGULARITY_COMPLETE%20%281%29/) contains the integer solver, CRT arithmetic, and finite-grid experiments.
- [Routing map](ROUTING_MAP.md) and [workspace inventory](WORKSPACE_MANIFEST.md) locate earlier research variants.
- [N32 solver repository](https://github.com/Skyelabz210/NS-3D-solver_n32-ntt6) contains the companion CRT v4 and six-prime NTT implementations.

## License

Anthony Diaz’s original project code and documentation are available under the [MIT License](LICENSE). [Scope and upstream notices](THIRD_PARTY.md)
