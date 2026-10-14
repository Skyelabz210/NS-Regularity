# Session archive and loose-file intake (2026-10-14)

Ten user-uploaded compressed archives were read directly and safely extracted in an independent local working package. **318 archive members**, **20 loose artifacts**, **213 distinct archive-relative paths**. Exactly **18 path names have multiple distinct contents** across the snapshots. All 23 Python files pass AST parsing. SHA-256 and Git blob SHA-1 for every extracted file are recorded in the companion `audit/SOURCE_MANIFEST.csv` delivered in the session package.

| Archive | Files | SHA-256 |
|---|---:|---|
| NS_BELTRAMI_CLOSURE 2.tar.gz | 14 | 958a81a33e07a6dedb020681b5c6f6bab2f2e03d57bd7744f7c105885fa6dbfa |
| NS_CRAM_ATLAS_PROBE.tar.gz | 14 | 2d267b81de6369450463e98a39931650c6caf31aa3e1c97f0340abb97a33b009 |
| NS_POSITIVE_CLOSES_v1.tar.gz | 110 | 6007cfba9a2a6c473d059db5c0d0f8066a7c985e783ba7401759b69d76ae6d37 |
| NS_REGULARITY_BOOTSTRAP.tar.gz | 13 | a7d1cd99a2c941a1bfb8afdcbbbb953e7c436aa09c31210fbae421d4d1602c8f |
| NS_REGULARITY_COMPLETE.tar.gz | 23 | b4abb0357f88a0c3b7cac88b434d9adcb0dae5030c43151df78811b0035cb734 |
| NS_REGULARITY_FINAL_v2.tar.gz | 19 | ad1336cc08cca548137001754f75aba7bc8b7332f4e9005fc421206d0d26de57 |
| ns_transfinite_cram_complete.tar.gz | 23 | ae1798a14be193d5eaa7d730313dd8732f72af438afafd07402c3d102d63b36c |
| ns_v5_final.tar.gz | 32 | e8c08fabff6babbdad8ad03d5a4666b3bfbaf62db0c867a723ea01858909140c |
| qmnf_ns_exact_bs.tar.gz | 50 | e563818a9ff6ee312268dbbb9e37b575418e623e58d597e596b7170cfefe474c |
| qmnf_ns_workspace.tar.gz | 20 | f391538135a82024b017be0b6790a91c9b3e27e762e4242d7ead4661a030ae35 |

The companion session tarball retains each archive under a distinct `lineage/archives/<source-name>/` directory, plus paper/test files and version manifest. Source files in that companion tarball **have not all been uploaded to GitHub**; GitHub imports in this change are fetched separately and are enumerated under `consolidated/SOURCE_PROVENANCE.md`.

## Paper/ledger inputs

- `Final NS Regularity Closure Compendium — Transfinite Edition.md`, and duplicate canonical title.
- `NS_CLOSURES_GENEALOGY.md`, `NS_CLOSURE_LEDGER_COMPLETE.md`.
- `PRIME_PAIR_NS_UNIFORMITY_CLOSURE.md` and duplicate title.
- `ROUND2.pdf`, `PRIME PAIR FOUR PROOFS.pdf`.
- `NS_FULL_TEST_OUTPUT.txt`; CRT/BS/bisect/corridor Rust probes.
- `Comprehensive Report on CRAM Frameworks and Prime Families Ontology.pdf`.

See the proof-claim audit for detailed scope reconciliation. All historical material is retained verbatim in the local bundle.
