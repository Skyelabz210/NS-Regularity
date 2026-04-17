# NSRegularity — Highly Decomposed Formalization DAG

Every node is a single lemma that closes with **one tactic block** (≤ 5 lines with Mathlib). Arrows point from dependency to dependent. Status: ✅ done · 🎯 next · ⬜ todo.

## Dependency graph (top-to-bottom)

```
                            ┌─────────────────────┐
                            │ L1.9  kill_113      │ ✅
                            │ residual u ω = 0    │
                            └──────────┬──────────┘
                                       │ 3× composition
                            ┌──────────▼──────────┐
                            │ L1.8  component_    │ ⬜
                            │  residual_zero      │
                            └──┬───────┬───────┬──┘
                               │       │       │
                  ┌────────────▼─┐ ┌───▼──┐ ┌──▼─────────┐
                  │ L1.8.axis_x  │ │axis_y│ │axis_z      │ ⬜ (helpers)
                  │ sumGrid→sum1 │ │      │ │            │
                  └──┬───────────┘ └──┬───┘ └──┬─────────┘
                     │                │        │
             ┌───────▼────┐   ┌───────▼──┐  ┌──▼────────┐
             │ L1.5 sbp_x │   │ L1.6 sbp_│  │ L1.7 sbp_z│ ⬜
             │            │   │      _y  │  │           │
             └──┬────┬────┘   └──┬───┬───┘  └──┬───┬────┘
                │    │           │   │         │   │
         ┌──────▼┐  ┌▼──────┐   (same structure × y × z)
         │L1.5.A │  │L1.5.B │
         │T1=T4  │  │T2=T3  │  ⬜ algebraic helpers
         └──┬────┘  └──┬────┘
            │          │
      ┌─────▼───┐  ┌───▼────┐
      │L1.3     │  │L1.4    │ ⬜
      │ sum_sp  │  │ sum_sn │
      └──┬──────┘  └──┬─────┘
         │            │
   ┌─────┼──────┐ ┌───┼──────┐
   │     │      │ │   │      │
   ▼     ▼      ▼ ▼   ▼      ▼
  L1.3.a L1.3.b..   L1.4.a L1.4.b..    (bijection → permutation)
   │     │            │     │
   └──┬──┘            └─┬───┘
      │                 │
   ┌──▼────────┐    ┌───▼──────┐
   │ L1.1      │    │ L1.2     │ ✅ (both)
   │ sp∘sn=id  │    │ sn∘sp=id │
   └───────────┘    └──────────┘
```

---

## D-nodes — Definitions (already in tree, no proof obligation)

| ID | File | Defines |
|---|---|---|
| D1 | `Shift/Defs.lean` | `abbrev N : Nat := 8` |
| D2 | `Shift/Defs.lean` | `sp`, `sn : Fin N → Fin N` |
| D3 | `Shift/Defs.lean` | `Grid`, `VField`, `sumGrid` |
| D4 | `Kill113/SBP/Defs.lean` | `sum1 : (Fin N → Int) → Int` |
| D5 | `Kill113/Ops.lean` | `diffX/Y/Z`, `advective`, `conservative`, `residual` |

---

## Tier 0 — Fin-8 decidables  *(both closed)*

| ID | Statement | File | Proof | Status |
|---|---|---|---|---|
| **L1.1** | `sp (sn i) = i` | `Shift/SpSn.lean` | `revert i; decide` | ✅ |
| **L1.2** | `sn (sp i) = i` | `Shift/SnSp.lean` | `revert i; decide` | ✅ |

---

## Tier 1 — Shift algebra (decomposed)  🎯 **next batch**

### `sum_sp` pipeline

| ID | Statement | Depends | Proof sketch | File |
|---|---|---|---|---|
| L1.3.a | `Function.Injective sp` | L1.2 | `intro a b h; have := congr_arg sn h; simp [sn_sp] at this; exact this` | *(new)* `Shift/SpInj.lean` |
| L1.3.b | `Function.Surjective sp` | L1.1 | `intro y; exact ⟨sn y, sp_sn y⟩` | *(new)* `Shift/SpSurj.lean` |
| L1.3.c | `Function.Bijective sp` | L1.3.a, L1.3.b | `⟨sp_injective, sp_surjective⟩` | *(new)* `Shift/SpBij.lean` |
| L1.3.d | `(List.finRange N).map sp ~ List.finRange N` | L1.3.c | `decide` *(N=8 concrete)* | *(new)* `Shift/SpPermFin.lean` |
| **L1.3** | `sum_sp` | L1.3.d | `simpa [List.foldl_map] using List.Perm.foldl_eq (·+·) h_perm` | `Shift/SumSp.lean` |

### `sum_sn` pipeline — same structure

| ID | Statement | Depends | File |
|---|---|---|---|
| L1.4.a | `Function.Injective sn` | L1.1 | `Shift/SnInj.lean` |
| L1.4.b | `Function.Surjective sn` | L1.2 | `Shift/SnSurj.lean` |
| L1.4.c | `Function.Bijective sn` | L1.4.a, L1.4.b | `Shift/SnBij.lean` |
| L1.4.d | `(List.finRange N).map sn ~ List.finRange N` | L1.4.c | `Shift/SnPermFin.lean` |
| **L1.4** | `sum_sn` | L1.4.d | `Shift/SumSn.lean` |

**Mathlib hooks:** `List.Perm`, `List.Perm.map_of_bijective` (or direct `decide` for L1.3.d since N=8), `List.foldl_map`, `List.Perm.foldl_eq`.

---

## Tier 2 — Reindex helpers (new, axis-agnostic)

| ID | Statement | Depends | Proof |
|---|---|---|---|
| L1.R.1 | `∑ᵢ f(sp i) = ∑ᵢ f i` *(sum1 form)* | L1.3 | `unfold sum1; exact sum_sp f` |
| L1.R.2 | `∑ᵢ f(sn i) = ∑ᵢ f i` | L1.4 | `unfold sum1; exact sum_sn f` |
| L1.R.3 | `∑ᵢ g(sp i)·h(sp i) = ∑ᵢ g(i)·h(i)` | L1.R.1 | `exact L1.R.1 (fun i => g i * h i)` |
| L1.R.4 | `∑ᵢ g(sn i)·h(sn i) = ∑ᵢ g(i)·h(i)` | L1.R.2 | same |

**File:** *(new)* `Kill113/SBP/Reindex.lean`.

---

## Tier 3 — 1D Summation-by-Parts (per axis)

Let the integrand be `P(i) = ω(i) · [u(i)·(ω(sp i) − ω(sn i)) + (u(sp i)·ω(sp i) − u(sn i)·ω(sn i))]`.

Expand `sum1 P` into four pieces:

| Piece | Definition |
|---|---|
| T1 | `∑ᵢ ω(i) · u(i) · ω(sp i)` |
| T2 | `∑ᵢ ω(i) · u(i) · ω(sn i)` |
| T3 | `∑ᵢ ω(i) · u(sp i) · ω(sp i)` |
| T4 | `∑ᵢ ω(i) · u(sn i) · ω(sn i)` |

Then `sum1 P = T1 − T2 + T3 − T4`. Reindex:

### Cancellation A: T1 = T4

| ID | Statement | Depends | Proof |
|---|---|---|---|
| L1.5.A.1 | `T4 = ∑ᵢ ω(sp i)·u(i)·ω(i)` | L1.R.4 | reindex `i ↦ sp i`, then `sn (sp i) = i` via L1.2 |
| L1.5.A.2 | `∑ᵢ ω(sp i)·u(i)·ω(i) = T1` | — | `simp_rw [mul_comm, mul_left_comm]; ring` |
| **L1.5.A** | `T4 = T1` | L1.5.A.1, L1.5.A.2 | `trans (∑ᵢ ω(sp i)·u(i)·ω(i)) <;> assumption` |

### Cancellation B: T2 = T3 *(symmetric)*

| ID | Statement | Depends | Proof |
|---|---|---|---|
| L1.5.B.1 | `T3 = ∑ᵢ ω(sn i)·u(i)·ω(i)` | L1.R.3 | reindex `i ↦ sn i`, `sp (sn i) = i` via L1.1 |
| L1.5.B.2 | `∑ᵢ ω(sn i)·u(i)·ω(i) = T2` | — | `ring` |
| **L1.5.B** | `T3 = T2` | L1.5.B.1, L1.5.B.2 | transitivity |

### Assembly

| ID | Statement | Depends | Proof |
|---|---|---|---|
| L1.5.C | `sum1 P = T1 − T2 + T3 − T4` | distributivity | `unfold sum1; simp_rw [mul_add, mul_sub]; ring` |
| **L1.5** | `sbp_x_1d = 0` | L1.5.A, L1.5.B, L1.5.C | `rw [L1.5.C, L1.5.A, L1.5.B]; ring` |

### L1.6, L1.7 — `sbp_y_1d`, `sbp_z_1d`

Identical structure with y- and z- axis variants. Each reuses L1.R.1–L1.R.4 and L1.5's scaffolding (generalize L1.5.A.1, L1.5.B.1 parametrically on the axis-axis pair).

**Optimization:** instead of triplicating the proof, write *one* generic `sbp_1d` lemma over arbitrary `Fin N → Int` and specialize it three times. Then L1.5, L1.6, L1.7 each become `exact sbp_1d _ _`.

---

## Tier 4 — 3D assembly  (`component_residual_zero`)

| ID | Statement | Depends | Proof |
|---|---|---|---|
| L1.8.a | `sumGrid (f + g) = sumGrid f + sumGrid g` | — | induction on nested folds |
| L1.8.b | `ω·(adv u ω) = x-piece + y-piece + z-piece` | defn | `unfold advective; ring` |
| L1.8.c | `ω·(cons u ω) = x-cpiece + y-cpiece + z-cpiece` | defn | `unfold conservative; ring` |
| L1.8.d | `sumGrid (fun i j k => P_x i j k) = ∑ⱼₖ sum1 (fun i => P_x i j k)` | defn | `unfold sumGrid sum1; rfl` |
| L1.8.e | `sum1 (fun i => P_x i j k) = 0` | L1.5 | `exact sbp_x_1d _ _` *(for each j, k)* |
| L1.8.f | `sumGrid (x-axis piece) = 0` | L1.8.d, L1.8.e | double nested rewrite |
| L1.8.g | `sumGrid (y-axis piece) = 0` | L1.6 (y-analogue) | same |
| L1.8.h | `sumGrid (z-axis piece) = 0` | L1.7 | same |
| **L1.8** | `component_residual_zero` | L1.8.a–h | linear combination of the three axis zeros |

---

## Tier 5 — Top  ✅

| ID | Statement | Depends | Proof |
|---|---|---|---|
| **L1.9** | `kill_113 u ω : residual u ω = 0` | L1.8 ×3 | `unfold residual; rw [component_residual_zero, component_residual_zero, component_residual_zero]; rfl` *(closed)* |

---

## Mathlib imports needed (per file)

| File | Import |
|---|---|
| `Shift/SpSurj.lean`, `SnSurj.lean`, `SpInj.lean`, `SnInj.lean` | `Mathlib.Logic.Function.Basic` |
| `Shift/SpBij.lean`, `SnBij.lean` | same |
| `Shift/SpPermFin.lean`, `SnPermFin.lean` | `Mathlib.Data.List.Perm` |
| `Shift/SumSp.lean`, `SumSn.lean` | `Mathlib.Data.List.Perm`, `Mathlib.Data.List.Basic` |
| `Kill113/SBP/Reindex.lean`, all `SBP/{X,Y,Z}.lean` | `Mathlib.Tactic.Ring`, `Mathlib.Algebra.Order.Ring.Int` |
| `Kill113/Component.lean` | same + `Mathlib.Data.Fin.Tuple.Basic` |

Top-level `import Mathlib` works but slows compilation; the fine-grained imports above keep each leaf's build time small.

---

## Proof plan: order of attack

1. ✅ **Tier 0** — L1.1, L1.2 (done)
2. 🎯 **Tier 1** — 10 atomic files (Spᴵ, Spˢ, Spᴮ, SpPermFin, SumSp + 5 for Sn). Each 1–2 lines of tactic.
3. **Tier 2** — L1.R.1–R.4 in `Reindex.lean`. All one-liners once Tier 1 is done.
4. **Tier 3** — Either write one `sbp_1d_generic` or triplicate. Favour the generic.
5. **Tier 4** — L1.8.a–h plus final assembly.
6. ✅ **Tier 5** — L1.9 already closed.

Critical-path length: **6 tiers × ~2 files/tier = ~12 Lean files** to touch, each closable in a single tactic block with Mathlib.

---

## Size audit

- Existing `.lean` files with sorry: **6**
- New `.lean` files this DAG calls for: **~10–12** (one per atomic lemma)
- Total leaf proofs to write: **~20** atomic tactic blocks
- Estimated tactic complexity: L1.1–L1.2 level for all of Tier 1; `ring` + `simp_rw` for Tier 3–4
- No new mathematical content — everything is mechanical discrete SBP
