import ExactObservation
import StarLift

/-! NS observables checked against the pinned Theory-of-the-One package.
    See the claim audit for source, toolchain and check command. -/
namespace NSObservations
open KElimination

/-- Read the signed diffusion quotient from two observations of the same
    numerator. The public integer window singles out its representative. -/
theorem diffusion_quotient (c lap d primary anchor lo : Int)
    (hp : primary = (c*lap) % d) (ha : anchor = (c*lap) % (d+1))
    (hl : lo ≤ (c*lap)/d) (hu : (c*lap)/d < lo+(d+1)) :
    StarLift.decodeNegative (StarLift.negativeAdjacent primary anchor d) (d+1) lo =
      (c*lap)/d :=
  StarLift.negative_adjacent_exact_window (c*lap) primary anchor d lo hp ha hl hu

/-- A 1D slice of the 16³ torus sampled on the eight even nodes. -/
def sampleEven (f : Fin 16 → Int) (j : Fin 8) : Int :=
  f ⟨2*j.val, by omega⟩

def fineLapAtOrigin (f : Fin 16 → Int) : Int := f 1 + f 15 - 2*f 0

/-- The fine Laplacian cannot be recovered on unrestricted fields from the
    coarse nodal observation, even with exact arithmetic. -/
theorem fine_laplacian_not_observable :
    ¬ ∃ decode : ExactObservation.Observed sampleEven → Int,
        ∀ f, decode (ExactObservation.reading sampleEven f) = fineLapAtOrigin f := by
  intro h
  let base : Fin 16 → Int := fun _ => 0
  let spike : Fin 16 → Int := fun i => if i.val = 1 then 1 else 0
  have same : sampleEven base = sampleEven spike := by
    funext j
    have hn : 2*j.val ≠ 1 := by omega
    simp [sampleEven, base, spike, hn]
  have eq := (ExactObservation.recoverable_iff sampleEven fineLapAtOrigin).mp h base spike same
  have a : fineLapAtOrigin base = 0 := by decide
  have b : fineLapAtOrigin spike = 1 := by decide
  omega

#print axioms diffusion_quotient
#print axioms fine_laplacian_not_observable
end NSObservations
