import NSRegularity.NS.Defs
import NSRegularity.Kill113.Ops
import NSRegularity.Kill113

/-!
# NS/FullStep — full Navier-Stokes time step (diffusion + advection + stretching)

The full discrete NS step composes three sub-operators:
  1. **Diffusion**:   `f → f + νΔt·Δf / N²S²`              (heat-equation contribution)
  2. **Advection**:   `f → f − Δt·(u·∇)f / (2·N·S)`         (skew-symmetric transport)
  3. **Stretching**:  `f → f + Δt·(ω·∇)u / (2·N·S)`         (vortex-line stretching)

The velocity field `u` is obtained from the vorticity `ω` via Biot-Savart
(NTT-based). This module abstracts `u` as a parameter — Biot-Savart is
ported separately (task #12).

## Status

This file gives the **definitions** of the full step. The L∞-preservation
proof for the full step requires:
  - diffusion: M2 mod L2.3 (PROVED)
  - advection: Kill #113 energy-neutral (PROVED), L∞ behavior bounded by Δf
  - stretching: DKAM-subcritical L2.6 (PROVED), bounded by L2.7

Plugging into `GlobalRegularityGeneric.generic_global_regularity` gives the
end-to-end NS closure once the L∞-preservation is established.
-/

namespace NSRegularity.NS

open NSRegularity NSRegularity.Kill113

/-- **Advection step (centered, integer-rounded).**
    `f - Δt · (u · ∇) f / (2 N S)`. The factor `2NS` accounts for the
    `1/(2h)` in centered differences (with grid spacing `h = 1/(NS)`). -/
def advection_step (u : VField) (f : Grid) (p : NsParams) : Grid :=
  fun i j k =>
    f i j k - p.dt_latt * advective u f i j k / (2 * (N : Int) * p.ns_scale)

/-- **Stretching step for vorticity component c.**
    Adds `Δt · (ω · ∇) u_c / (2 N S)` to the c-th vorticity component.
    Here `uc_grad : Grid` represents `(ω · ∇) u_c` as a precomputed grid
    (typically computed using `advective ω uc`). -/
def stretching_step (uc_grad : Grid) (f : Grid) (p : NsParams) : Grid :=
  fun i j k =>
    f i j k + p.dt_latt * uc_grad i j k / (2 * (N : Int) * p.ns_scale)

/-- **Full NS step for a single vorticity component.**
    Composes diffusion → advection → stretching in standard operator-splitting
    order. The velocity `u` and stretching term `uc_grad` are inputs (computed
    from `ω` via Biot-Savart and `advective ω uc` respectively). -/
def ns_step_component (u : VField) (uc_grad : Grid)
    (f : Grid) (p : NsParams) : Grid :=
  stretching_step uc_grad (advection_step u (diffusion_step f p) p) p

/-- **Full NS vector-field step.** Takes a velocity field `u` (Biot-Savart
    of input ω) and three stretching grids `uc_grad_{x,y,z}` (one per
    vorticity component). -/
def ns_step (u : VField) (uc_grad_x uc_grad_y uc_grad_z : Grid)
    (w : VField) (p : NsParams) : VField :=
  ⟨ns_step_component u uc_grad_x w.x p,
   ns_step_component u uc_grad_y w.y p,
   ns_step_component u uc_grad_z w.z p⟩

end NSRegularity.NS
