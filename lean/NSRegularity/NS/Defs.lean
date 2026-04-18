import NSRegularity.Shift.Defs

/-!
# NS/Defs — Discrete Navier-Stokes operators on (ℤ/Nℤ)³

Definitions faithful to `ns3d.rs:570-760`. All integer arithmetic, no floats.
-/

namespace NSRegularity.NS

open NSRegularity

/-- 7-point discrete Laplacian on the periodic lattice. -/
def laplacian (f : Grid) (i j k : Fin N) : Int :=
  f (sp i) j k + f (sn i) j k
  + f i (sp j) k + f i (sn j) k
  + f i j (sp k) + f i j (sn k)
  - 6 * f i j k

/-- L∞ norm of a scalar grid — max |f(i,j,k)|. -/
def gridLInf (f : Grid) : Int :=
  Finset.sup' Finset.univ ⟨(0 : Fin N), Finset.mem_univ _⟩
    (fun i => Finset.sup' Finset.univ ⟨(0 : Fin N), Finset.mem_univ _⟩
      (fun j => Finset.sup' Finset.univ ⟨(0 : Fin N), Finset.mem_univ _⟩
        (fun k => (f i j k).natAbs)))

/-- L∞ norm of a vector field — max over 3 components. -/
def lInf (w : VField) : Int :=
  max (gridLInf w.x) (max (gridLInf w.y) (gridLInf w.z))

/-- Diffusion CFL number: c = 6 · ν_scaled · dt / (N² · S²).
    CFL is satisfied when c ≤ 1 (equivalently, each stencil weight ≥ 0). -/
structure NsParams where
  nu_scaled : Int
  dt_latt   : Int
  ns_scale  : Int := 10000

/-- CFL condition: the diffusion stencil coefficient `1 - 6c` is non-negative. -/
def cfl_ok (p : NsParams) : Prop :=
  6 * p.nu_scaled * p.dt_latt ≤ (N : Int) ^ 2 * p.ns_scale ^ 2

/-- Pure diffusion step on a single component. -/
def diffusion_step (f : Grid) (p : NsParams) : Grid :=
  fun i j k =>
    f i j k + p.nu_scaled * p.dt_latt * laplacian f i j k / ((N : Int) ^ 2 * p.ns_scale ^ 2)

end NSRegularity.NS
