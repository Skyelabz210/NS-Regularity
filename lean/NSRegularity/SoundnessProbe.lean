import NSRegularity.NS.BiotSavart

/-!
# Soundness probe: BiotSavart axiom set is consistent under `hdiv` precondition

Before the fix, `biot_savart_curl_eq` stated `∀ ω, curl (biot_savart ω) = ω`
without a divergence hypothesis. Composing with the proved theorem
`curl_div_zero` then gave `∀ ω, divV ω = 0`, which is false on the discrete
torus.

After the fix, the axiom requires `divV ω = 0` as a precondition. The
contradiction no longer goes through: `all_fields_are_div_free` becomes a
trivial restatement (`divV ω = 0 → divV ω = 0`), not a universal claim
about arbitrary `ω`.

This file *checks* the fix: it derives the post-fix theorem and exhibits
the concrete counter-example that broke the pre-fix axiom set.
-/

namespace NSRegularity.NS.SoundnessProbe

open NSRegularity NSRegularity.NS

/-- Post-fix consequence of the axioms. Now correctly requires `hdiv`
    rather than asserting it for all `ω`. -/
theorem div_free_under_biot_savart (ω : VField)
    (hdiv : divV ω = (fun (_ _ _ : Fin N) => (0 : Int))) :
    divV ω = (fun (_ _ _ : Fin N) => (0 : Int)) := by
  have h1 : divV (curl (biot_savart ω)) = (fun _ _ _ => (0 : Int)) :=
    curl_div_zero (biot_savart ω)
  have h2 : curl (biot_savart ω) = ω := biot_savart_curl_eq ω hdiv
  rw [h2] at h1
  exact h1

/-- A concrete vorticity with nonzero divergence at the origin.
    `ω.x i j k = i.val`, the rest zero. Then
    `divV ω 0 0 0 = diffX ω.x 0 0 0 = ω.x 1 0 0 - ω.x 7 0 0 = 1 − 7 = −6`.

    Pre-fix, applying the unconditional axiom to this `ω` produced False.
    Post-fix, the axiom is unavailable here because `hdiv` cannot be
    discharged for this `ω`. -/
def omega_witness : VField :=
  ⟨fun i _ _ => (i.val : Int), fun _ _ _ => 0, fun _ _ _ => 0⟩

theorem omega_witness_divV_not_zero :
    divV omega_witness 0 0 0 = -6 := by
  unfold divV omega_witness
  unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
  decide

end NSRegularity.NS.SoundnessProbe
