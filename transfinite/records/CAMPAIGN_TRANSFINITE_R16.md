# Campaign TRANSFINITE — Round 1 Record (2026-06-10)

## Design
Exact-rational physical-track trajectories mirroring the ns_step_c update law
(w' = w + dt·(ν·lap/S² + (w·∇)curl w)/n2), denominators composed of 2 and 5 only — coprime to
every carrier prime by construction. Observable: activation level ℓ(t) of the 18-level
winding tower (M₁₈ ≈ 5.6×10²⁴) on x_int = ⌊cell⌋, with floor γ(t). Discriminant: the exact
per-step growth ratio — a flat ratio is the explicit scheme's linear mode; an accelerating
ratio is the nonlinear signature. Control: Taylor-Green (stretch-null seed). Stress:
xorshift adversarial seeds, amplitudes 50 / 500 / 5000, per-amplitude exact dt = 2^−k
normalized to first-increment ≤ X₀/8.

## Instrument findings
- **F1.** Adaptive dt from a stretch-null seed selects dt=1 and detonates the explicit scheme
  once stretch activates — controls take the conservative fixed dt.
- **F2.** During that detonation the tower read the explosion exactly: ℓ marched 7→10→16→18
  with γ one above, through 31 orders of magnitude, zero sandwich violations. The observable
  works precisely under blowup conditions — when it must.
- **F3.** D-026 live: the residue-track step divides via mod_inverse; activation reads belong
  to the decoded physical track (dual-track separation load-bearing).

## Round 1 data
| run | dt | steps | ℓ trajectory | Δ = ℓ* − (γ₀−1) | ratio behavior |
|---|---|---|---|---|---|
| TG-5000 control | 2⁻¹⁹ | 12 | pinned 7, decay onset t=11 | **0** | 1.0000 → 0.9999 (decay) |
| ASYM-50 | 2⁻¹⁹ | 9 | pinned 6 | **0** | flat ≈ 1.053 |
| ASYM-500 | 2⁻²³ | 6 | pinned 6 | **0** | decreasing 1.066 → 1.054 |
| ASYM-5000 | 2⁻²⁶ | 6 | pinned 7 | **0** | ≈ 1.043–1.053, no acceleration |

## Verdict
**Δ = 0 uniformly across the sweep** — the activation level never rose above the seed's
energy floor at any amplitude, and the acceleration discriminant fired nowhere (one stress
ratio is decreasing). The control decayed as physics requires. Scope held exactly: horizons
6–12 steps, N = 8, per-amplitude dt; CSTRETCH_UNIF (= D-024) and D-026 remain **OPEN**. Round
1 establishes the instrument end-to-end and the first uniform-in-amplitude bounded window.

## Round 2 specification
Production port into ns_v5 (CRTBigInt + DynCRT growable carrier; the Python twin was the
prototype): asym seeds, horizons 10³–10⁴ steps, N = 8 → 16 → 32 (the uniformity-in-N axis is
the node's own axis), exact dt ladder, ratio-acceleration detector as the standing blowup
discriminant, ledger append per run.
