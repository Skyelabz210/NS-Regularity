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

/-- **Auxiliary L2.3* — Diffusion step under EXACT division is max-preserving.**

    Multiplying through by `N²S²`, the discrete update at each grid point
    satisfies
      `N²S² · (f + Δf-correction) = (N²S² − 6νΔt) · f + νΔt · Σ neighbors`.
    The RHS is a convex combination with nonneg integer weights summing to
    `N²S²` of seven values each bounded by `M := gridLInf f`. Hence
    `|N²S² · (...)| ≤ N²S² · M`, so `|...| ≤ M`.

    This is the algebraic content of L2.3. The unconditional statement
    requires handling `Int.ediv` rounding (the integer-Helmholtz obstruction),
    which is the open piece below. -/
theorem diffusion_max_preserving_algebraic
    (f : Grid) (p : NsParams) (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled)
    (hdt : 0 ≤ p.dt_latt) (i j k : Fin N) :
    -- Multiply through: N²S² · update = (N²S² − 6νΔt) · f + νΔt · Σ neighbors
    let M := (N : Int) ^ 2 * p.ns_scale ^ 2
    let c := p.nu_scaled * p.dt_latt
    (M * (f i j k) + c * laplacian f i j k)
    = (M - 6 * c) * f i j k
      + c * (f (sp i) j k + f (sn i) j k
           + f i (sp j) k + f i (sn j) k
           + f i j (sp k) + f i j (sn k)) := by
  simp only [laplacian]
  ring

/-- L2.3 — The diffusion step is max-preserving: the updated value at
    any grid point is bounded by the original L∞ norm.

    **Status (D-030 / integer-Helmholtz obstruction).** The algebraic
    convex-combination argument is captured in `diffusion_max_preserving_algebraic`
    above. To lift it to L2.3 as stated requires controlling the rounding
    of `Int.ediv` in `diffusion_step`. There are three resolutions tracked
    in the parent project (M3-B-2.6 decision point):
      A. exact-division hypothesis `(N²S²) ∣ νΔt · Δf`
      B. relax the bound to `≤ gridLInf f + 1` (off-by-one)
      C. redefine `diffusion_step` via `ExactRational`/CRT (matches Rust)

    Pending operator selection from {A,B,C}, L2.3 remains a documented
    open piece. Composition `M2 = l_inf_non_increasing` reduces cleanly to L2.3
    (see DKAM.lean). -/
theorem diffusion_max_preserving (f : Grid) (p : NsParams)
    (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) :
    ∀ i j k : Fin N,
      (diffusion_step f p i j k).natAbs ≤ gridLInf f := by
  sorry

end NSRegularity.DKAM.Diffusion
