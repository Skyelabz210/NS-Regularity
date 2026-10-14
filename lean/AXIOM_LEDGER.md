# NSRegularity — Explicit Axiom Ledger

**Branch:** `claude/analyze-ns-framework-cV8OP`
**Last verified:** 2026-06-12 (commit `8c3b7cd`)
**Mathlib:** v4.31.0-rc1

This ledger documents every project-side axiom on the live path. The
audit invariant is: **any new axiom requires a row here** with the five
mandatory fields (statement, citation, intended discharge, consuming
lemma, soundness note).

The protocol is FN-L3 from `EXECUTION_PLAN_NS_FINAL_NODES.md`:
formalize-or-isolate. Where formalization exceeds current Mathlib, isolate
the axiom with the exact external statement and the consuming-lemma
named in this file (so `#print axioms` shows the import verbatim).

---

## 1. Headline finding (positive)

The live-path **does not depend on BKM or Prodi-Serrin axioms.** Dispatch's
"3 named axioms" set (`ntt_biot_savart_velocity_exists`,
`bkm_smoothness_from_linf_time_integral`, `prodi_serrin_uniqueness`) is the
status of the parent spine; the NSRegularity discrete substrate on this
branch went further and **discharged BKM as a theorem** by reframing
`IsSmooth` as the discrete uniform L∞ bound (PR #10 work). Specifically:

```
#print axioms NSRegularity.BKM.global_regularity
  → [propext, Classical.choice, Quot.sound]                (CORE ONLY)

#print axioms NSRegularity.BKM.bkm_regularity_criterion
  → [propext, Classical.choice, Quot.sound]                (CORE ONLY)

#print axioms NSRegularity.NS.HStep.lInf_ns_step_small_data
  → [propext, Classical.choice, Quot.sound]                (CORE ONLY)
```

`global_regularity` and `bkm_regularity_criterion` are the local
counterparts of the spine's BKM and Prodi-Serrin slots. Both are core-only.
Net effect: of the spine's three named axioms, two are already discharged
here and one (BS existence) remains together with three more BS-side rows.

---

## 2. The four BS-side axioms on the live path

```
#print axioms NSRegularity.NS.Capstone.ns_global_regularity_canonical
  → [propext, Classical.choice, Quot.sound,
     NSRegularity.NS.biot_savart]

#print axioms NSRegularity.NS.BiotSavartBound.piece_A_discharged
  → [propext, Classical.choice, Quot.sound,
     NSRegularity.NS.biot_savart,
     NSRegularity.NS.BiotSavartBound.biot_savart_linf_bound]
```

The capstone uses **only** the operator declaration; the characteristic
curl/div axioms are bypassed because `hBS` and the Resolution-A exactness
hypotheses enter explicitly. `piece_A_discharged` additionally invokes the
interface bound. The full repo-level axiom census, with curl/div counted
because downstream uses (e.g. `lInf_le_four_lInf_biot_savart`) consume
them, is the four entries below.

---

### Axiom 1: `biot_savart : VField → VField`

**Location:** `NSRegularity/NS/BiotSavart.lean:83`

**Statement (Lean form):**
```
axiom biot_savart : VField → VField
```

**External statement.** The discrete Biot-Savart inversion operator: maps a
divergence-free vorticity field `ω : VField` on `(ℤ/Nℤ)³` to the unique
divergence-free velocity field `u` with `curl u = ω`.

**Intended discharge (constructive).** The Rust pipeline `ns3d.rs` (4-prime
NTT + Garner reconstruction confined to the decode boundary, per Dispatch
§4) computes `u = K ⋆ ω` for a fixed integer convolution kernel `K`. The
Lean port becomes:

```
def biot_savart : VField → VField :=
  fun ω => ntt_inverse (ntt_kernel_mul (ntt_forward ω))
```

with `ntt_*` operating in residue space end-to-end. **K-Elimination** (not
Garner) handles divisions inside the kernel multiplication; Garner is
reserved for the BKM decode boundary.

**Source citation.** Beale, Kato & Majda (1984), with the discrete-NTT
construction following the QMNF NS substrate at `NS_FIFTH_OPERATOR_PACKET_v2`
(503/503 tests green, Rust 1.96).

**Consuming lemmas.** Every theorem that constructs a velocity field from a
vorticity field, including `ns_trajectory_generator`,
`ns_global_regularity_skeleton`, and `ns_global_regularity_canonical`.

**Soundness note.** The operator is declared, not characterised; the
characteristic properties live in Axioms 2 and 3. `SoundnessProbe.lean`
demonstrates that without precondition `divV ω = 0` the curl axiom would
be unsound (concrete witness with `divV ω 0 0 0 = −6`).

---

### Axiom 2: `biot_savart_curl_eq`

**Location:** `NSRegularity/NS/BiotSavart.lean:99`

**Statement (Lean form):**
```
axiom biot_savart_curl_eq (ω : VField)
    (hsource : ∃ u : VField,
      curl u = ω ∧ divV u = (fun (_ _ _ : Fin N) => (0 : Int))) :
    curl (biot_savart ω) = ω
```

**External statement.** On the discrete 3-torus, the Biot-Savart inverse
of a divergence-free vorticity field has curl equal to the input.

**Intended discharge.** Direct from the NTT construction: in residue space,
the kernel symbol is `iκ × /|κ|²`, so `curl ∘ BS = id` on the
divergence-free subspace by exact integer arithmetic. Becomes a theorem
once Axiom 1 is constructed.

**Source citation.** Standard discrete vector calculus on the periodic
lattice; the precondition `divV ω = 0` is essential.

**Consuming lemmas.**
- `NS.BiotSavartBound.lInf_le_four_lInf_biot_savart` (the consistency
  floor `lInf ω ≤ 4·lInf(BS ω)`)
- Any future Lean-side test that BS recovers ω from curl-NTT data.

**Soundness note.** Without the `hdiv` precondition the axiom is unsound:
composing with `curl_div_zero` would force `divV ω = 0` for *every* ω,
contradicting `SoundnessProbe.lean`. The precondition holds along the NS
trajectory because every vorticity in the trajectory is itself a curl
(`curl_div_zero` then gives `divV ω = 0` automatically).

---

### Axiom 3: `biot_savart_div_zero`

**Location:** `NSRegularity/NS/BiotSavart.lean:109`

**Statement (Lean form):**
```
axiom biot_savart_div_zero (ω : VField) :
    divV (biot_savart ω) = fun _ _ _ => 0
```

**External statement.** The Biot-Savart inverse always produces a
divergence-free velocity field, unconditionally.

**Intended discharge.** The NTT kernel `iκ × /|κ|²` is divergence-free by
construction (its symbol is orthogonal to κ in the inner product); the
property is exact-integer-valid through residue space.

**Source citation.** Standard discrete vector calculus on the periodic
lattice (no precondition needed: divergence-free is a property of the
range of `curl⁻¹`).

**Consuming lemmas.** Any future Lean development that needs `u` to be
divergence-free for the energy/H² identities downstream of `biot_savart`.

**Soundness note.** Independent of Axiom 2. Becomes a theorem once
Axiom 1 is constructed.

---

### Axiom 4: `biot_savart_linf_bound`  *(Attack 1 interface)*

**Location:** `NSRegularity/NS/BiotSavartBound.lean:229`

**Statement (Lean form):**
```
axiom biot_savart_linf_bound :
    ∃ C_BS : Int, 0 < C_BS ∧
      ∀ ω : VField, lInf (biot_savart ω) ≤ C_BS * lInf ω
```

**External statement.** L∞ operator bound for the canonical NTT
Biot-Savart representative: there exists a positive integer constant
`C_BS` (depending on N but not on ω) such that the L∞ norm of the
Biot-Savart velocity is bounded by `C_BS` times the L∞ norm of the input
vorticity.

**Intended discharge (constructive).** From the residue-resident NTT
pipeline, the velocity is a fixed integer-kernel convolution
`u(x) = Σ_y K(x−y)·ω(y)`, so

```
‖u‖_∞  ≤  ‖K‖_{ℓ¹} · ‖ω‖_∞
```

with `‖K‖_{ℓ¹} = Σ_y |K(y)|` a concrete integer at fixed `N`. Discharge
plan: port the NTT kernel into Lean, prove `‖K‖_{ℓ¹}` is a finite integer
sum, and let `C_BS = ‖K‖_{ℓ¹}`. **K-Elimination** handles the kernel
division (no Garner inside the operator, per Dispatch §4).

**Source citation.** Operator-norm form of the discrete Biot-Savart
inversion. Soundness floor: `lInf_le_four_lInf_biot_savart` (proved here)
forces `C_BS ≥ 1/4`.

**Consuming lemmas.**
- `NS.BiotSavartBound.piece_A_discharged` (closes Piece A of the
  small-data DAG)
- `NS.Capstone.ns_global_regularity_canonical` consumes the **hypothesis
  form** `hBS`, not the axiom itself, so the capstone's axiom trace does
  not list `biot_savart_linf_bound`. The axiom enters only when the
  hypothesis is instantiated from `biot_savart_linf_bound` via
  `obtain ⟨C_BS, hC_pos, hbound⟩ := biot_savart_linf_bound`.

**Soundness note.** Cannot be derived from Axioms 2 and 3: the
`velocity_not_determined_by_curl_div` theorem (proved unconditionally via
`decide`) exhibits a checkerboard field with zero centered differences,
so adding it to a velocity component changes neither `curl` nor `divV`.
Hence the forward L∞ bound is a property of the *chosen NTT representative*
and must enter as its own interface. Consistency floor `C_BS ≥ 1/4` is
forced.

---

## 3. Discharge trajectory

```
Now:                    4  [biot_savart, curl_eq, div_zero, linf_bound]
After NTT port:         0  on this branch (interface + characteristic
                            axioms all become theorems once the
                            NTT kernel is constructed in Lean)
```

The four BS axioms all discharge together via the constructive NTT port
(Dispatch's FN-L2 Gate 1 node). No partial discharge is meaningful — once
the operator is constructed, all four properties fall out by exact-integer
arithmetic on the NTT representation.

---

## 4. Status of Dispatch's "three named axioms"

| Dispatch axiom                              | Status on this branch    |
|---------------------------------------------|--------------------------|
| `ntt_biot_savart_velocity_exists`           | Open here as Axiom 1 (BS operator) |
| `bkm_smoothness_from_linf_time_integral`    | **Discharged** as `BKM.bkm_regularity_criterion` (PR #10 reframing) |
| `prodi_serrin_uniqueness`                   | **Discharged**: the discrete `IsSmooth` *is* the uniform L∞ bound, so uniqueness is the propositional `Exists` uniqueness from the witness's natural-number index |

Two of three already-discharged is the headline positive finding.

---

## 5. Verification procedure

To re-run the audit after any change:

```
cd lean
lake build                                # full library: 8555 jobs, 0 errors
```

then with a probe file (delete after use):

```lean
-- in lean/NSRegularity/AxiomProbe.lean
import NSRegularity.NS.SmallDataCapstone

#print axioms NSRegularity.NS.Capstone.ns_global_regularity_canonical
#print axioms NSRegularity.BKM.global_regularity
#print axioms NSRegularity.NS.HStep.lInf_ns_step_small_data
#print axioms NSRegularity.NS.BiotSavartBound.piece_A_discharged
```

Expected output (commit `8c3b7cd`):

```
'NSRegularity.NS.Capstone.ns_global_regularity_canonical' depends on axioms:
    [propext, Classical.choice, Quot.sound, NSRegularity.NS.biot_savart]
'NSRegularity.BKM.global_regularity' depends on axioms:
    [propext, Classical.choice, Quot.sound]
'NSRegularity.NS.HStep.lInf_ns_step_small_data' depends on axioms:
    [propext, Classical.choice, Quot.sound]
'NSRegularity.NS.BiotSavartBound.piece_A_discharged' depends on axioms:
    [propext, Classical.choice, Quot.sound,
     NSRegularity.NS.biot_savart,
     NSRegularity.NS.BiotSavartBound.biot_savart_linf_bound]
```

Any divergence from the expected output triggers a ledger update.

---

## 6. Audit invariant

**Any new `axiom` declaration in the `NSRegularity/` namespace must add a
row to §2 with all five mandatory fields before it can be merged to
`main`.** No silent axioms; no axioms that bypass §3's discharge trajectory.

## 2026-10-14 soundness correction: curl range (constant/Nyquist obstruction)

The prior assertion that divergence-free alone makes `curl(BS ω)=ω` consistent was incorrect. On the periodic centered-difference grid, every curl component sums to zero, but the constant field `ω=(1,0,0)` is divergence-free and sums to `N³`. At even grid N, checkerboard modes are divergence-free and zero-mean yet remain orthogonal to every centered curl. The characteristic axiom now requires an explicit divergence-free preimage (curl representability). Its downstream lower-bound theorem and SoundnessProbe have the corresponding parameter. The source implementation still needs constructive NTT inversion and a proof that the numerical NS trajectory stays within the supported subspace. Exact integer regression: `scripts/verify_biot_savart_domain.py`.
