import NSRegularity.NS.Defs

/-!
# M2.a — Diffusion is max-preserving under CFL (leaves L2.1–L2.3)

The 7-point Laplacian stencil with CFL `6νΔt ≤ N²S²` produces non-negative
weights summing to 1. The diffusion step is therefore a convex combination:
  `diffusion_step f p (i,j,k) ≤ gridLInf f` for all grid points.
-/

namespace NSRegularity.DKAM.Diffusion

open NSRegularity NSRegularity.NS

/-- L2.1 — Under CFL, the stencil self-weight `1 - 6c` is non-negative. -/
theorem stencil_self_nonneg (p : NsParams) (h : cfl_ok p) :
    0 ≤ (N : Int) ^ 2 * p.ns_scale ^ 2 - 6 * p.nu_scaled * p.dt_latt := by
  unfold cfl_ok at h
  linarith

/-- L2.2 — Under CFL, the neighbor weight `c = νΔt/(N²S²)` is non-negative
    (given ν, Δt ≥ 0). -/
theorem stencil_neighbor_nonneg (p : NsParams) (hν : 0 ≤ p.nu_scaled)
    (hdt : 0 ≤ p.dt_latt) :
    0 ≤ p.nu_scaled * p.dt_latt := by
  exact mul_nonneg hν hdt

/-- L2.3 — The diffusion step is max-preserving: the updated value at
    any grid point is bounded by the original L∞ norm.

    Proof sketch: expand `diffusion_step` as a convex combination
    `(1-6c)·f(i,j,k) + c·Σ_{neighbors} f(...)`, use L2.1 + L2.2 to bound
    each term, conclude ≤ max. -/
theorem diffusion_max_preserving (f : Grid) (p : NsParams)
    (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) :
    ∀ i j k : Fin N,
      (diffusion_step f p i j k).natAbs ≤ gridLInf f := by
  sorry

end NSRegularity.DKAM.Diffusion
