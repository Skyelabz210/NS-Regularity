# Campaign TRANSFINITE R2 — Wave 1 Deliverable: N3-STRUCTB + N2-SEEDMAX

**Anthony Diaz (Acid) — HackFate.us / Skyelabz210**
**2026-06-12 · Executor: this instance · A1 zero-float on every path (integers + exact rationals + ℚ(√2))**
**Anchors cited per §A of the course: SAEI, DKAM, GR05, κ-contraction, GR06–09, transfinite ledger 109–122. ID 01 (April 1) cited for the impossibility map and the Beltrami reduction.**

---

## N3-STRUCTB — The exact rearrangement of the stretching pairing

The course asked for the exact rearrangement of ⟨S(ω,u),ω⟩ into flux/boundary terms on tower coordinates — identity + proof, or the impossibility map. ID 01 already delivered the impossibility side of the *zero-design* question (test field ≠ operator field ⟹ the skew trick that closes SAEI leaves stretching production nonzero; Beltrami data reduces stretching to SAEI and gives exact zero). What follows is the identity side: two exact rearrangements, both verified to integer zero across all seeds at N = 4, 8, 16.

### Identity 1 — Strain form (pure algebra, holds in ℤ)

For any fields ω, u on (ℤ/Nℤ)³ with central differences d_d f[x] = f[x+e_d] − f[x−e_d] and G_cd = d_d u_c:

**⟨S(ω,u), ω⟩ = Σ_x ω(x)ᵀ E_h(x) ω(x),  with 2·⟨S,ω⟩ = Σ_x ωᵀ(G + Gᵀ)ω exactly,**

because the antisymmetric part of G annihilates the quadratic form identically — ωᵀAω = 0 for any antisymmetric A is integer algebra, requiring no analysis, no limits, no smallness. Verified: `anti = 0` and `2·⟨S,ω⟩ = sym` exact across TG, checkerboard, delta, random (amplitudes to 999), at N = 4, 8, 16.

Two structural consequences, both exact:

- **trace(E_h) = div_h u**, so for solenoidal u the strain is traceless and its eigenvalues sum to zero — amplification in one principal direction is paid for by contraction in another, at the integer level.
- The stretching obstacle is now a *quadratic form in ω against a traceless symmetric matrix field*, which is the object the tower can read pointwise.

### Identity 2 — Flux/cocycle form (the carry term made explicit)

With forward differences D⁺_d f[x] = f[x+e_d] − f[x] and the discrete Leibniz defect

L_d(f,g)[x] = D⁺f[x]·D⁺g[x] − D⁺f[x−e_d]·D⁺g[x−e_d]

(a backward difference of the forward-difference product — it telescopes to zero on the torus), the following holds **exactly in ℤ** for solenoidal ω:

**⟨S(ω,u), ω⟩ + ⟨(ω·∇_h)ω, u⟩ + Σ_{x,c,d} u_c·L_d(ω_c, ω_d) + Σ_x (u·ω)(div_h ω) = 0**

Derivation chain (each step an exact identity): summation by parts Σ f·d_d g = −Σ g·d_d f on the torus; discrete Leibniz d_d(fg) = f·d_d g + g·d_d f + L_d; div_h curl_h = 0. Verified to integer zero at N = 4, 8, 16 across all seed families, with the solenoidality term independently confirmed zero.

The third term is the **cubic difference flux** — the discrete-only remainder, a trilinear form in first differences. It is supported on high-frequency content and scales as O(h²) relative to the H¹ norms: this is the carry term the course's perturbation table predicted (SAEI × winding-as-information → "stretching winding-flux cocycle", class D-6). The D-6 target is delivered as an identity, with ID 01's D-7 map standing for the complementary zero-design question.

### The Frobenius certificate (exact, pointwise, integer-comparable)

At every cell, with q = ωᵀ(G+Gᵀ)ω, F² = Σ_cd (G_cd+G_dc)², W = |ω|²:

**q² ≤ F²·W²  (Cauchy–Schwarz for the quadratic form), all integer quantities.**

Verified to hold at every cell, every seed, N = 4, 8, 16. Tightest observed ratio ≈ 0.66 of the bound (exact fractions recorded in the run log). This converts the stretching pairing into a per-cell integer-certifiable bound:

dZ/dt ≤ 2·max_x ‖E_h(x)‖_F · Z − 2ν‖∇_h ω‖²

with max_x ‖E_h‖²_F an exact integer telemetry channel per step. This is the quantity N1-PORT should carry alongside {ℓ(t), γ(t), ratio, enstrophy} — it is the exact per-step Lyapunov-rate observable for the physical track, and it is the natural candidate for the conditional step bound in hypothesis H (ℓ' ≤ max(ℓ, γ(E))): the activation level can only climb as fast as the strain field pays for it, and the strain field is now an exact integer read.

### Discharge path toward CSTRETCH-PHYS (stated, not claimed)

CSTRETCH-PHYS stays [OPEN] per the course. What N3 adds to the route: the physical-track production is now (a) a traceless quadratic form bounded cell-by-cell by an exact integer certificate, and (b) decomposed into transport + cubic flux where the flux carries the h² factor. The N5 bridge (decode-validity: numerator activation < lane capacity) applied to the strain telemetry is the remaining composition. N7b inherits a concrete formalization target: the two identities are `rfl`-grade lattice algebra in the GR05 no-axioms style.

---

## N2-SEEDMAX — Exact adversarial catalog (N = 8, flat BS, ℚ(√2) carrier)

Biot–Savart computed exactly: forward/inverse DFT in ℚ(ζ₈) (4-tuples of rationals mod ζ⁴ = −1, roundtrip and conjugate symmetry verified exact), symbol iκ×/|κ|², k = 0 and Nyquist planes zeroed. The output lands in the real subfield: every cell of u has the form (a, b, 0, −b) = a + b√2.

**Finding 0 (convergent verification of #128).** A fresh, independent implementation reproduces the ℚ(√2) content of the N=8 BS kernel — the value field is real and exact in ℚ(√2), with genuinely nonzero √2 parts. Ordering and sign of all catalog quantities computed by the exact ℚ(√2) comparator (sign of a + b√2 via integer comparison of a² vs 2b²).

**Production quotient catalog** R² = ⟨ω, S(ω,BS(ω))⟩² / ‖ω‖₂⁶, exact in ℚ(√2), sign of P recorded:

| rank | seed | sign(P) | magnitude |
|---|---|---|---|
| 1 | delta (3-point, broadband) | + | 1.35×10⁻⁷ |
| 2 | random #2 (amp 99) | + | 8.1×10⁻⁸ |
| 3 | random #1 (amp 9) | − | 3.2×10⁻⁸ |
| 4 | random #3 (amp 999) | − | 1.6×10⁻⁸ |
| 5–14 | TG, ABC, stripes mod 3/5/7, single modes k=(1,0,0)…(3,1,1) | **0 exact** | 0 |

**Finding 1 (exact zero on sparse spectra).** Every single-mode seed and every separable structured seed (TG, ABC-like, residue-class stripes) produces **exact integer-zero** production. The discrete operator inherits the triad-interaction requirement at the integer level: one mode cannot stretch itself. The Hydra stripe families, being low-mode in disguise, are non-amplifying — the carry-discriminative role they play in the sieve does not transfer to stretching amplification by themselves.

**Finding 2 (broadband is the adversary).** Amplifying seeds are exactly the broadband ones — the 3-point delta (full spectral support) leads the catalog. This is consistent with the cubic-flux term of Identity 2 living on first differences: production requires spectral *spread*, and the worst case concentrates differences, not amplitudes. Amplitude alone is neutral (random #3 at amp 999 ranks below random #2 at amp 99 and decays).

**Worst-case catalog for N4-NSWEEP:** {delta-broadband, random-positive seeds}, with the next refinement being deliberate triad pairs (two-mode seeds with resonant k₁+k₂ interactions) — single modes are exact zeros, so the minimal amplifying structure is the pair, and the pair catalog is the right N4 stress input.

---

## Proposed ledger entries (append-only, operator's canonical file)

| # | property tags | stressed with | source | status |
|---|---|---|---|---|
| 123 | `OPERATOR-CONSISTENCY` `EXACT-ARITH` | mixed | N3-STRUCTB Identity 1: ⟨S,ω⟩ = Σ ωᵀE_hω, antisym annihilation exact, N=4/8/16 | ✓ survived |
| 124 | `OPERATOR-CONSISTENCY` `EXACT-ARITH` | mixed | N3-STRUCTB Identity 2: stretching cocycle with explicit cubic Leibniz flux, integer zero residual, N=4/8/16 | ✓ survived |
| 125 | `EXACT-ARITH` | mixed | Frobenius pointwise certificate q² ≤ F²W², every cell every seed, exact integers | ✓ survived |
| 126 | `EXACT-ARITH` `OPERATOR-CONSISTENCY` | none(identity) | Independent ℚ(ζ₈) BS implementation reconfirms #128 ℚ(√2) kernel content | ✓ survived |
| 127 | `SURVIVES-ADVERSARIAL` `EXACT-ARITH` | adversarial | N2-SEEDMAX exact catalog: sparse-spectrum seeds produce integer-zero production; broadband delta is worst case | ✓ survived |

## Run artifacts

- `structb.py` — Identity 1 + 2 verification, N=8, six seed families
- `structb_rigor.py` — N=4/8/16 sweep + Frobenius certificate with tightest-ratio fractions
- `n2_seedmax2.py` — ℚ(ζ₈) DFT, ℚ(√2) carrier + comparator, exact catalog

CSTRETCH-PHYS, D-024, D-026 remain [OPEN] per course invariants. The contraction-track closures are cited as exactly that.
