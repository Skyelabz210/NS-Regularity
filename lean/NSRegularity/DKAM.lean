import NSRegularity.Shift.Defs
import NSRegularity.Kill113
import NSRegularity.DKAM.DiffusionMaxPreserving
import NSRegularity.DKAM.Stretching
import NSRegularity.DKAM.Winding

/-!
# Component maximum principle for the pure diffusion step

Apply the scalar rounded-stencil bound to each component and take the maximum.
-/

namespace NSRegularity.DKAM

open NSRegularity NSRegularity.NS

/-- The three-component diffusion update contracts the grid maximum under CFL. -/
theorem l_inf_non_increasing (w : VField) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) :
    lInf (⟨diffusion_step w.x p, diffusion_step w.y p, diffusion_step w.z p⟩ : VField)
    ≤ lInf w := by
  have hx : gridLInf (diffusion_step w.x p) ≤ gridLInf w.x := by
    unfold gridLInf
    apply Finset.sup'_le; intro i _
    apply Finset.sup'_le; intro j _
    apply Finset.sup'_le; intro k _
    exact Diffusion.diffusion_max_preserving w.x p h hν hdt i j k
  have hy : gridLInf (diffusion_step w.y p) ≤ gridLInf w.y := by
    unfold gridLInf
    apply Finset.sup'_le; intro i _
    apply Finset.sup'_le; intro j _
    apply Finset.sup'_le; intro k _
    exact Diffusion.diffusion_max_preserving w.y p h hν hdt i j k
  have hz : gridLInf (diffusion_step w.z p) ≤ gridLInf w.z := by
    unfold gridLInf
    apply Finset.sup'_le; intro i _
    apply Finset.sup'_le; intro j _
    apply Finset.sup'_le; intro k _
    exact Diffusion.diffusion_max_preserving w.z p h hν hdt i j k
  unfold lInf
  exact max_le_max hx (max_le_max hy hz)

end NSRegularity.DKAM
