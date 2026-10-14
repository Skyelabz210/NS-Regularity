# Source provenance and integration status

## GitHub sources (fetched 2026-10-14)

| Repository | Role | Integration | Primary source ref |
|---|---|---|---|
| [NS-Regularity](https://github.com/Skyelabz210/NS-Regularity) | canonical existing Rust/Lean/bridge workspace | preserved, active | main at `7043a7fcc9ccd728141db7d13b1df2953fc15d04` |
| [ns3d-n32-ntt6](https://github.com/Skyelabz210/ns3d-n32-ntt6) | wider-grid N32 CRT/NTT6 and QCID | exact source blobs copied into `consolidated/` | main at `83887a22e8c5aedbb2aad32172cf4bffdc3e62c2` |
| [ns-lean](https://github.com/Skyelabz210/ns-lean) | distinct Lean torus project | exact source blobs copied; separate Lake project | main at `22bfe0f90d939c101ac32dd3664ff7cb322a6204` |
| [NINE65_v7](https://github.com/Skyelabz210/NINE65_v7) | shared CRT/CRAM/K-Elim substrate | dependency/reference, intentionally not vendored wholesale | main tree audited |
| [k-elimination-lean4](https://github.com/Skyelabz210/k-elimination-lean4) | separate formal arithmetic proof | external formal dependency/reference | connector-visible |
| [QMNF_System](https://github.com/Skyelabz210/QMNF_System) | historical QMNF substrate | researched for shared components, not vendored wholesale | master tree audited |
| [Quantum_Modular_Numeric_Framework](https://github.com/Skyelabz210/Quantum_Modular_Numeric_Framework) | earlier arithmetic engine | dependency/reference | main tree audited |

## Content-addressed import rule

Each imported `.rs`, `.lean`, configuration or research document was fetched via the GitHub connector and stored in the destination as a blob. The destination Git blob SHA was required to equal the upstream source Git blob SHA. The archive snapshot SHA-256 hashes are documented separately.

To update an imported file, document the source commit, reconcile diffs with the canonical code, run crate/project tests, and record a promotion decision. Historical variants should remain addressable.

## Existing Lean PR history

PRs #2 through #11 are closed; #9, #10, and #11 were merged directly. Earlier #5, #7, #8 report no direct `merged_at` in the GitHub PR API but the current main tree contains their routed work. Do not assume 'closed' and 'merged' are interchangeable.

The active Lean proof status and axiom census are in `lean/PROOF_STATUS.md` and `lean/AXIOM_LEDGER.md`.

## Upload handling

Ten local `.tar.gz` session uploads and loose source/paper files were unpacked and SHA-256-indexed in the companion local consolidation bundle. A local upload is not claimed to be pushed to GitHub unless its content is actually present in this repository. The complete bundle is delivered separately in the conversation.

## Explicit build-adapter exception

The QCID `Cargo.toml` was deliberately updated after the initial SHA-exact import because its former `../ns_v5` path was invalid in the new directory structure. The upstream unchanged file is `consolidated/bridges/qcid_satellite/Cargo.upstream.toml`. This is a manifest adapter; all imported `.rs` and `.lean` sources remain source-SHA identical. The individual source fingerprints are enforced by `consolidated/PROVENANCE_BLOBS.tsv` and `.github/workflows/ns-validation.yml`.
