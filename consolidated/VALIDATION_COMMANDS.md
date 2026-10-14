# Reproducibility checklist and commands

## Primary Rust runner

```bash
cd 'NS_REGULARITY_COMPLETE (1)/crates/ns_v5'
cargo test --locked
cargo test --release --locked
cd ../qcid_ns_bridge
cargo test --locked
```

The default primary workspace may use nested relative paths and historical workspace manifests. Resolve cargo membership paths before reporting an individual crate's failure. Preserve original numerical and adversarial results.

## Imported QCID satellite

```bash
cd consolidated/bridges/qcid_satellite
cargo test
```

## Lean formal projects

```bash
cd lean
lake build
# evaluate theorem dependencies in the Lean REPL / temporary inspection module:
# #print axioms NSRegularity.NS.Capstone.ns_global_regularity_canonical
# #print axioms NSRegularity.GlobalRegularity.discrete_ns_regularity_certificate_exact

cd ../consolidated/formal/ns_lean
lake build
```

## Audit evidence required

- Record git SHA, rustc/cargo/Lean versions, exit code, failing test name, and complete stdout/stderr.
- Show independent baseline vs integrated result for N8, N16, N32 (Taylor-Green, ABC, arbitrary divergence-free random fields, adversarial fields).
- Verify BS kernel scaling and positive modes vs zero mode.
- Verify one-step centered reconstruction under arbitrary sign and amplitude, including boundary/corridor saturation.
- Enforce A1 on load-bearing computation paths (integer / exact rational; document symbolic analytical arguments separately).
- Measure physically normalized energies and compare N-dependent constants, not only residue and raw integer metrics.
- Assert explicit pass/fail for H² commutator and vortex stretching in genuinely 3D non-Beltrami data.
- Keep unconditional, conditional and counterexample tests in distinct reporting categories.

**Environment status (2026-10-14):** local session extraction and Python AST syntax validation completed, 23/23 pass. `cargo`, `rustc`, `lean`, and `lake` were unavailable on that sandbox. These commands are verification requirements, not asserted test runs.

## GitHub Actions test groups

Workflow `.github/workflows/ns-validation.yml` exercises provenance hashing, primary Rust, imported QCID Rust, active Lean, and independent Lean. Separate jobs preserve precise failure classification. The imported QCID manifest has an explicit canonical path adapter; its historical Cargo.lock is preserved, and the unlocked run resolves the workspace dependency graph.
