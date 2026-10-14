# QCID H², FP² and SQR bridge

Origin: https://github.com/Skyelabz210/ns3d-n32-ntt6 under `arena/crates/qcid_ns_bridge/` (2026-10-14).

Complete source crate copied unchanged under this directory, including its Cargo metadata, full 14-module source tree, H² certificate variants and canonical sweep. Intended for independent `cargo test --locked` before comparing with the existing partial QCID bridge in the parent NS-Regularity workspace and the user-provided session archive variants.

Critical requirement: a passing certificate test for a finite/conditional case must not be labeled a proof for arbitrary smooth 3D Navier–Stokes continuum data.

## Build adapter (2026-10-14)

The imported upstream `Cargo.toml` assumed a sibling `../ns_v5` crate. To build inside this consolidated repository, `Cargo.toml` now points to the existing canonical solver at `../../../NS_REGULARITY_COMPLETE (1)/crates/ns_v5`. The unmodified source manifest is retained as `Cargo.upstream.toml`, retaining the source Git blob SHA. Rust code stays byte-for-byte identical.
