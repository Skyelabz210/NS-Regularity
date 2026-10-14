# Conflicting source paths (session archives)

Conflicts are content differences at the same normalized archive-relative path. They do not imply a failed build or a flawed implementation. Preserve variants, compare their diffs, and test against a common driver.

## Critical collisions

- `qcid_ns_bridge/src/h2_closure.rs`: 3 content variants across Beltrami/Atlas/Positive snapshots; further distinct versions under `NS_FINAL_PACKAGE`.
- `qcid_ns_bridge/src/lib.rs`: 3 variants. Export graph varies; selecting an H² implementation requires matching module exports.
- `ns_v5/src/lib.rs`, `qmnf_crtbigint/src/lib.rs`: differ between Positive Closes and v5 Final.
- `ns_v5/src/ns3d_n16.rs`, `ns_v5/src/ns3d_n32.rs`, `ns_v5/src/ns3d_wassan.rs`, `ns_v5/src/recumbent.rs`: differ between Positive Closes and v5 Final.
- `qmnf_unified/crates/nsgrid3d/src/biot_savart.rs`, `fused_solver.rs`, `regularity_tracker.rs`, `ns_grid_3d.rs`: differ between early workspace and Exact BS snapshots.
- `NS_FINAL_PACKAGE/README.md` and `qcid_ns_bridge/src/h2_closure.rs`: differ between Complete and Final v2 drops.
- `qcid_ns_bridge/Cargo.lock` and `qmnf_unified/Cargo.toml`: dependency/manifest variants.

**Disposition:** retain all versions (hashed local artifact); primary code remains in the original root. The independently copied satellite bridge source in `consolidated/bridges/qcid_satellite` is another separately identifiable line. No silent overwrite. Full `FILE_VARIANTS.json` supplied with session bundle.

## Promotion review for each colliding file

1. Determine generation and parent(s) from source path and manifest (not just timestamp or filename).
2. Compare public API, modular arithmetic overflow behavior, exact quotient contract, NTT prime count, normalization, module exports.
3. Run the same differential harness at N8/N16/N32 and at corridor thresholds, including the negative/expected-failure tests.
4. Resolve with an adapter or feature flag, record replacement parent SHA and theorem impact.
5. Promote into active crate only after CI and Lean proof dependencies are green.
