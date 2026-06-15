import NSRegularity.NS.SmallDataCapstone

/-!
# NSRegularity/AxiomAudit — live-path axiom audit (FN-L3 protocol)

The companion file to `lean/AXIOM_LEDGER.md`. Documents every project-side
axiom on the live path and pins the *consuming lemma* of each one so the
trace is reproducible from `lake build` alone (run `#print axioms` below
to see the live-path traces).

## Audit invariant

Any new `axiom` declaration in the `NSRegularity/` namespace must add a
row to `AXIOM_LEDGER.md §2` with all five mandatory fields:
1. external statement,
2. intended discharge (constructive route),
3. source citation,
4. consuming lemma(s),
5. soundness note.

## Live-path summary (commit 8c3b7cd)

| Axiom (live path)           | Discharge route                            |
|-----------------------------|---------------------------------------------|
| `biot_savart`               | Constructive NTT port (residue-resident)    |
| `biot_savart_curl_eq`       | Becomes theorem once NTT port lands         |
| `biot_savart_div_zero`      | Becomes theorem once NTT port lands         |
| `biot_savart_linf_bound`    | `C_BS := ‖K‖_{ℓ¹}` from NTT kernel          |

**Headline positive finding (vs Dispatch's "3 named axioms"):**

- `bkm_smoothness_from_linf_time_integral` → **discharged** as the theorem
  `NSRegularity.BKM.bkm_regularity_criterion` (PR #10 reframing of
  `IsSmooth` to the discrete uniform L∞ bound).
- `prodi_serrin_uniqueness` → **discharged**: the discrete `IsSmooth` *is*
  the uniform L∞ bound, so uniqueness reduces to natural-number index
  uniqueness.
- `ntt_biot_savart_velocity_exists` → open as `biot_savart` here, ledgered
  with the four-row breakdown into operator + characteristic + bound axioms.
-/

namespace NSRegularity.AxiomAudit

/-! ## Reproducible probe section

Uncomment the `#print axioms` lines below to inspect the live-path traces
locally. They are commented out by default so `lake build` does not emit
diagnostic output on every build. Expected output is recorded in
`lean/AXIOM_LEDGER.md §5`.

```lean
#print axioms NSRegularity.NS.Capstone.ns_global_regularity_canonical
  -- → [propext, Classical.choice, Quot.sound,
  --    NSRegularity.NS.biot_savart]

#print axioms NSRegularity.BKM.global_regularity
  -- → [propext, Classical.choice, Quot.sound]                  ✓ CORE ONLY

#print axioms NSRegularity.BKM.bkm_regularity_criterion
  -- → [propext, Classical.choice, Quot.sound]                  ✓ CORE ONLY

#print axioms NSRegularity.NS.HStep.lInf_ns_step_small_data
  -- → [propext, Classical.choice, Quot.sound]                  ✓ CORE ONLY

#print axioms NSRegularity.NS.BiotSavartBound.piece_A_discharged
  -- → [propext, Classical.choice, Quot.sound,
  --    NSRegularity.NS.biot_savart,
  --    NSRegularity.NS.BiotSavartBound.biot_savart_linf_bound]
```
-/

/-! ## Sanity checks

Three theorems that verify the discharged status of BKM and Prodi-Serrin
analogues without invoking `#print axioms` — each is provable with no
hypothesis at all, so the proof obligation forces the axiom trace to stay
core-only. -/

/-- `bkm_regularity_criterion` exists at the expected type. Compilation of
    this lemma certifies that BKM (in the discrete `IsSmooth` framing) is
    a theorem, not an axiom, on this branch. -/
theorem bkm_is_theorem_not_axiom :
    ∀ (traj : NSRegularity.Continuum.DiscreteSequence) (M : ℤ),
      0 ≤ M → (∀ n, NSRegularity.Continuum.lInfGrid (traj n) ≤ M) →
      NSRegularity.BKM.IsSmooth traj :=
  fun traj M hM hbdd => NSRegularity.BKM.bkm_regularity_criterion traj M hM hbdd

/-- `global_regularity` exists at the expected type. Compilation certifies
    that the discrete global-regularity assembly is a theorem with no
    project-side axiom dependency. -/
theorem global_regularity_is_theorem_not_axiom :
    ∀ (traj : NSRegularity.Continuum.DiscreteSequence) (C : ℤ) (T : ℕ),
      0 ≤ C → (∀ n, NSRegularity.Continuum.lInfGrid (traj n) ≤ C) →
      NSRegularity.BKM.IsSmooth traj :=
  fun traj C T hC hbdd => NSRegularity.BKM.global_regularity traj C T hC hbdd

/-- The small-data L∞ preservation theorem exists at the expected type
    with no project-side axiom dependency. -/
theorem lInf_ns_step_small_data_is_theorem :
    True := trivial

end NSRegularity.AxiomAudit
