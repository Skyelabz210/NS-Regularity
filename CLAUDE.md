# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust-based computational attempt at proving global regularity of the 3D incompressible Navier-Stokes equations (Clay Millennium Prize Problem). The core innovation is exact-integer arithmetic via Chinese Remainder Theorem (CRT) to avoid floating-point errors and provide mathematically rigorous bounds.

## Commands

The main crate lives at `NS_REGULARITY_COMPLETE (1)/crates/ns_v5`.

```bash
cd "NS_REGULARITY_COMPLETE (1)/crates/ns_v5"

cargo test                        # Run all ~147 tests (expect 0 failures)
cargo test <test_name> -- --nocapture  # Run single test with output
cargo build                       # Debug build
cargo build --release             # Optimized build
cargo check                       # Syntax/type check only
cargo doc --open                  # Generate and view docs
```

## Architecture

### Three-Layer Stack

**Executor Layer** (exact arithmetic substrate):
- `qmnf_crtbigint/src/lib.rs` — CRTBigInt v3.1: eight 63-bit prime lanes (~504-bit capacity), Montgomery multiplication, K-Elimination exact division, Garner mixed-radix reconstruction, `integer_sine` (Bhaskara I, zero-float), `ExactRational`
- `src/recumbent.rs` — Tiered exact integer: primary corridor + overflow + hot lane; split/promotion/demotion; exact value preservation
- `src/wassan.rs` — WASSAN holographic key-value persistence for Recumbent state

**Discrete Scheme Layer** (3D N=8 periodic grid):
- `src/ns3d.rs` — Core 3D solver: NTT-based Biot-Savart (4 primes: 73, 89, 97, 113), centered finite-difference Laplacian, skew-symmetric advection, vortex stretching, time stepper, diagnostics
- `src/ns3d_energy.rs` — Kill #113 (exact zero advection inner product), Kill #121 (topological energy), Aubin-Lions conditions (H¹ bound, time-derivative bound in H⁻¹), BKM chain
- `src/ns3d_sobolev.rs` — Discrete H¹/H² seminorms (exact i128 via summation-by-parts), Sobolev embedding (sharp constant: `51979095679759/546264947916800`), stretching measurement
- `src/ns3d_wassan.rs` — WASSAN-backed persistent vorticity; checkpoint/restore; CRTBigInt-backed H² norm

**Continuum Layer**: Scaffolded but not fully proved — Aubin-Lions compactness, nonlinear passage, Leray-Hopf weak solution.

**Theorem Verification** (`src/lib.rs`): Theorems T1-T10 (contraction bounds B ≤ 90/1001), N-independence proof, discrete maximum principle, winding decomposition.

### Key Types & Constants

```rust
pub const N: usize = 8;           // Periodic lattice (ℤ/8ℤ)³
pub const NS_SCALE: i64 = 10_000; // Physical scale factor
pub const NTT_PRIMES: [u64; 4] = [73, 89, 97, 113];

pub struct Vort3D { pub data: [[[i64; N]; N]; N] }
pub struct Vel3D  { pub data: [[[i64; N]; N]; N] }
```

### Data Flow

1. Divergence-free initial condition on ℤ³_N (integer-valued or Bhaskara-sine-approximated)
2. Biot-Savart: vorticity → velocity via 4-prime NTT + Garner reconstruction (exact)
3. Time step: diffusion (Laplacian) → advection (skew-symmetric) → stretching (DKAM subcritical)
4. Norm tracking: H¹, H² (exact summation-by-parts), L∞, enstrophy
5. Overflow management: Recumbent tiers track exact values beyond primary corridor
6. WASSAN persistence: checkpoint full tiered state for long-time evolution

## Proof Status (Three Tiers)

**Tier 1 — Discrete Proved** (verified by `cargo test`): NTT exactness, Kill #113 energy identity, Kill #121 topological energy, L∞ non-increasing (DKAM), H² stability with sharp Sobolev constant, Aubin-Lions conditions, Recumbent invariance, WASSAN roundtrip.

**Tier 2 — Continuum Scaffolded** (N→∞ transfer pending): Aubin-Lions compactness, nonlinear term convergence, Leray-Hopf existence, BKM regularity corollary.

**Tier 3 — Open Gaps**: Super-threshold stretching bounds for arbitrary-amplitude 3D data; uniform-in-N transfer (N=16/32 solvers exist but NTT prime sets untested); T_FK(N) favorable scaling.

## Documentation Index

- `STATUS.md` — Canonical claim structure: which theorems are proved vs. scaffolded vs. open; test inventory; file manifest
- `NS_REGULARITY_PROOF_COMPLETE.md` — Compact 7-step proof outline
- `NS_REGULARITY_COMPLETE_PACKET.md` — Full formal manuscript (~100K words)
- `DEEP_RESEARCH_FRAMEWORK.md` — Three-layer architecture and Theorems A-I
- `BLUEPRINT.md` — Tao Blueprint DAG (PFR 2023 methodology)

## In-Code Annotation Convention

```
[CODE FACT]       — verified arithmetic, passes cargo test
[DISCRETE THEOREM] — proved for lifted N=8 scheme
[SCAFFOLDED]      — infrastructure ready, pending N→∞ transfer
[OPEN]            — honest gap (not yet proved)
```

## Dependencies

Only one external crate: `lazy_static` v1 (precomputed NTT constant tables). All arithmetic is handcrafted exact-integer — no floating-point anywhere in the core.
