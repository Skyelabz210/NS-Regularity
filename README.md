# NS Regularity

A computational framework attempting global regularity of the **3D incompressible Navier-Stokes equations** (Clay Millennium Prize Problem) via exact-integer arithmetic and the Chinese Remainder Theorem.

The core innovation: every computation runs in exact integers with no floating-point — vorticity, velocity, Sobolev norms, Biot-Savart kernels — so every inequality is either true or false, never within ε.

## Status

| Layer | Status | Evidence |
|---|---|---|
| Executor substrate (CRTBigInt, WASSAN, Recumbent) | ✅ | 212/212 tests in `ns_v5` |
| Discrete regularity at N=8 | ✅ | Kill #113, Kill #121, DKAM 90/1001, sharp Sobolev |
| Discrete regularity at N=16, N=32 | ⚠️ partial | `ns3d_n16`, `ns3d_n32` — empirical campaigns pending |
| D-030 winding-energy L∞ bound (PDE form) | ✅ | `ns3d_d030.rs`, 9/9 tests |
| Prime-hunt / Hydra sieve | ✅ | 103/103 tests in `prime_hunt` |
| Lean 4 formalization of Kill #113 | 🚧 | Skeleton in `lean/`, 10 `sorry`s remaining |
| Continuum limit (Aubin-Lions passage) | 🚧 scaffolded | Write-up in `NS_REGULARITY_PROOF_COMPLETE.md` Step 7 |
| **D-024 Hasse bridge (large data)** | ❌ OPEN | The genuine Millennium bottleneck |

## Quickstart

```bash
cd "NS_REGULARITY_COMPLETE (1)/crates/ns_v5"
cargo test                   # 212 tests, ~140s
cd ../prime_hunt && cargo test   # 103 tests, ~0.1s
```

For Lean:
```bash
cd lean && lake build
```

## Directory layout

```
├─ NS_REGULARITY_COMPLETE (1)/   Main workspace — Rust crates, proof docs
│   ├─ crates/ns_v5/             Core 3D NS solver + all theorems (212 tests)
│   ├─ crates/prime_hunt/        Multi-head Hydra sieve (103 tests)
│   ├─ crates/qmnf_crtbigint/    CRTBigInt v3.1 arithmetic substrate
│   └─ *.md                      Proof manuscripts, status docs, gap assessments
├─ lean/                         Lean 4 formalization (NSRegularity.Kill113, Shift)
├─ recumbant_crt/                Hydra-v3, yukawa schema, recombinant CRT research
├─ fp2_nttns/                    FP² NTT snapshots + CRAM white papers
├─ subcriticality/               DKAM subcriticality formalization (+ planetary schemas)
├─ fall_of_regularity/           NS paper workspaces v8/v10 + distance analyses
├─ dresden_prime_hunt/           Dresden prime-hunt: papers, plans, synthesis
├─ gambit_cram/                  CRAM slot predictor, universal harness, Hydra atlas
├─ qtc_continuum/                Quantum topological continuum + Explorer images
├─ session_workspaces/           Sieve-oracle spec, T2/T3/T4 runs, session summaries
├─ skills/                       prime-family-spacetime skill package
├─ CLAUDE.md                     Guidance for Claude Code / AI agents
└─ WORKSPACE_MANIFEST.md         Detailed inventory of every folder
```

See **[WORKSPACE_MANIFEST.md](WORKSPACE_MANIFEST.md)** for the file-level tour.

## What's been proven

**Discrete (verified by `cargo test`):**
- **Kill #113** — skew-symmetric advection energy neutrality, exact integer zero, no preconditions
- **Kill #121** — per-NTT-prime-lane energy residual = 0 on all four primes {73, 89, 97, 113}
- **DKAM 90/1001** — contraction bound `B(N) ≤ 90/1001` unconditionally for every `N`
- **Sharp Sobolev** — exact rational constant `51979095679759 / 546264947916800`
- **D-030** — `‖ω(t)‖_∞ ≤ (p−1) + √E(0)` from energy PDE + winding decomposition
- **p-adic non-expansion** — vorticity is non-expanding on every CRT lane
- **Aubin-Lions (I), (II)** — uniform H¹ bound + Sobolev descent under small-data

**Scaffolded (infrastructure + write-up pending integration):**
- Continuum compactness — conditions (i), (ii) proved at N=8; uniform-in-N transfer pending
- Nonlinear passage — full Lions-1969 argument written, integration pending
- BKM regularity corollary — assembles once large-data bound closes

## What's open

See **[NS_REGULARITY_COMPLETE (1)/GAP_ASSESSMENT_D030.md](NS_REGULARITY_COMPLETE%20%281%29/GAP_ASSESSMENT_D030.md)** for the current gap list. The Millennium bottleneck is **D-024**: the Hasse local-global bridge from p-adic non-expansion to Archimedean L∞ boundedness for arbitrary-amplitude 3D initial data.

## Author

Anthony Diaz (Acid) — HackFate / Skyelabz210  
Independent research, April 2026
