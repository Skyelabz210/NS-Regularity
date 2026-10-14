# N32 six-prime solver lineage

Origin: https://github.com/Skyelabz210/ns3d-n32-ntt6, `main` (2026-10-14 inventory).

Copied unchanged: `ns3d_n32_ntt6.rs`, CRT solver revisions v1-v4, `cram_div.rs`, `cram_div_ext.rs`, `cram_div_u128.rs`, P5B DAG and execution proposal.

This directory is a **source-preserving integration lane**, not an automatically exported module of the primary `ns_v5` crate. Follow `INTEGRATION_DAG.md` and compare the v4 residual/overflow fixes and 6-prime NTT path against the active primary `ns3d.rs` operator and `ns3d_n32.rs` before enabling. The original satellite repository is the runnable parent workspace.
