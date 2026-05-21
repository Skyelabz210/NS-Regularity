import NSRegularity.NS.FullStep
import NSRegularity.GlobalRegularityGeneric

/-!
# NS/Trajectory — full NS trajectory generator and global regularity skeleton

Combines the full NS step from `NS/FullStep.lean` with the generic global
regularity closure from `GlobalRegularityGeneric.lean` to provide an
end-to-end Millennium-style claim **conditional on**:
  - the operator-supplied Biot-Savart inversion `biot_savart`
  - the operator-supplied stretching gradients
  - the L∞-preservation of the resulting `ns_step`

Once these three pieces are formally proved (task #12 for Biot-Savart;
task #18 for full-NS L∞), `ns_global_regularity_skeleton` becomes
unconditionally true modulo only the BKM axiom.
-/

namespace NSRegularity.NS

open NSRegularity NSRegularity.NS NSRegularity.GlobalRegularity

/-- Abstract NS trajectory: at each step, derive velocity `u` from vorticity
    via supplied `biot_savart`, derive stretching gradients, then apply
    `ns_step`. Encapsulates the operator inputs as opaque functions. -/
def ns_trajectory_generator
    (biot_savart : VField → VField)
    (stretching_x stretching_y stretching_z : VField → Grid)
    (w₀ : VField) (p : NsParams) : ℕ → VField
  | 0     => w₀
  | n + 1 =>
    let wn := ns_trajectory_generator biot_savart stretching_x stretching_y stretching_z w₀ p n
    ns_step (biot_savart wn) (stretching_x wn) (stretching_y wn) (stretching_z wn) wn p

/-- The `ns_trajectory_generator` is the `generic_trajectory` instance for
    the closed-form NS step. -/
theorem ns_trajectory_eq_generic
    (biot_savart : VField → VField)
    (stretching_x stretching_y stretching_z : VField → Grid)
    (w₀ : VField) (p : NsParams) :
    ns_trajectory_generator biot_savart stretching_x stretching_y stretching_z w₀ p =
    generic_trajectory
      (fun w => ns_step (biot_savart w) (stretching_x w)
                        (stretching_y w) (stretching_z w) w p)
      w₀ := by
  funext n
  induction n with
  | zero => rfl
  | succ n ih =>
      show ns_trajectory_generator _ _ _ _ _ _ (n + 1) =
            generic_trajectory _ _ (n + 1)
      simp only [ns_trajectory_generator, generic_trajectory, ih]

set_option maxHeartbeats 400000 in
/-- **NS global regularity skeleton.**

    Given:
    - an arbitrary Biot-Savart inversion `biot_savart : VField → VField`,
    - arbitrary stretching gradients `stretching_{x,y,z} : VField → Grid`,
    - the operator-supplied L∞-preservation hypothesis `h_step` for the
      composite NS step at every state,
    - non-negative initial L∞,
    - a time horizon `T`,

    the NS trajectory is smooth (modulo the BKM axiom).

    This is the architectural top of the Lean closure: once `biot_savart`,
    `stretching_*`, and `h_step` are filled in by operator-side proofs
    (task #12 for Biot-Savart NTT port; task #18 for full-NS L∞ via DKAM
    + Kill #113 composition), this theorem becomes unconditionally true
    modulo only the BKM 1984 axiom. -/
theorem ns_global_regularity_skeleton
    (biot_savart : VField → VField)
    (stretching_x stretching_y stretching_z : VField → Grid)
    (p : NsParams)
    (h_step : ∀ w : VField,
        lInf (ns_step (biot_savart w) (stretching_x w)
                      (stretching_y w) (stretching_z w) w p) ≤ lInf w)
    (w₀ : VField)
    (hLInf : 0 ≤ lInf w₀) (T : ℕ) :
    NSRegularity.BKM.IsSmooth
      (ns_trajectory_generator biot_savart stretching_x stretching_y stretching_z w₀ p) := by
  rw [ns_trajectory_eq_generic]
  exact generic_global_regularity
    (fun w => ns_step (biot_savart w) (stretching_x w)
                      (stretching_y w) (stretching_z w) w p)
    h_step w₀ hLInf T

end NSRegularity.NS
