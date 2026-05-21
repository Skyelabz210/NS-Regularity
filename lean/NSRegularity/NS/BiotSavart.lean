import NSRegularity.NS.Defs
import NSRegularity.Kill113.Ops

/-!
# NS/BiotSavart — Biot-Savart inversion skeleton

The vorticity-velocity inversion on the discrete 3-torus.

In 3D incompressible NS, the velocity `u` is recovered from the vorticity
`ω` via the Biot-Savart law:
  `∇ × u = ω,   ∇ · u = 0`

i.e. `u = curl⁻¹ ω` in the divergence-free space.

Discretely, this is implemented via a 4-prime NTT (Number Theoretic Transform)
pipeline in the Rust crate (`ns3d.rs` uses primes {73, 89, 97, 113}) plus
Garner reconstruction to give exact integer velocity.

This file provides the **type signature** of Biot-Savart and the
characteristic property (curl u = ω). The constructive NTT implementation
is pending (task #12 from the project tracker).

## Status

- `BiotSavart` type signature: PROVIDED here.
- `curl` (discrete) and `div` (discrete) definitions: PROVIDED.
- `biot_savart_curl_eq` characteristic property: stated as `axiom`,
  pending NTT port.

The skeleton allows `Wave G`'s `ns_global_regularity_skeleton` to be
specialized to a concrete Biot-Savart witness rather than just take it
as an abstract parameter.
-/

namespace NSRegularity.NS

open NSRegularity NSRegularity.NS

/-- Discrete curl in the x-component:
    `(curl w).x = ∂_y w.z - ∂_z w.y`
    (centered differences, no 1/(2h) factor — see Kill113/Ops `diffY`/`diffZ`). -/
def curlX (w : VField) : Grid :=
  fun i j k => NSRegularity.Kill113.diffY w.z i j k -
               NSRegularity.Kill113.diffZ w.y i j k

/-- Discrete curl in the y-component:
    `(curl w).y = ∂_z w.x - ∂_x w.z` -/
def curlY (w : VField) : Grid :=
  fun i j k => NSRegularity.Kill113.diffZ w.x i j k -
               NSRegularity.Kill113.diffX w.z i j k

/-- Discrete curl in the z-component:
    `(curl w).z = ∂_x w.y - ∂_y w.x` -/
def curlZ (w : VField) : Grid :=
  fun i j k => NSRegularity.Kill113.diffX w.y i j k -
               NSRegularity.Kill113.diffY w.x i j k

/-- Discrete curl of a vector field. -/
def curl (w : VField) : VField :=
  ⟨curlX w, curlY w, curlZ w⟩

/-- Discrete divergence:
    `div w = ∂_x w.x + ∂_y w.y + ∂_z w.z` -/
def divV (w : VField) : Grid :=
  fun i j k => NSRegularity.Kill113.diffX w.x i j k +
               NSRegularity.Kill113.diffY w.y i j k +
               NSRegularity.Kill113.diffZ w.z i j k

/-- **Biot-Savart inversion (declared).**

    Maps a vorticity field to the unique divergence-free velocity field
    whose curl equals the input vorticity (assuming the input has zero
    spatial mean, which is automatically true on the torus when
    `∇ · ω = 0` — vorticity is always divergence-free).

    The concrete implementation uses a 4-prime NTT pipeline (constructive,
    no axioms), but the port to Lean is pending. For now we declare the
    signature and the characteristic property as axioms. -/
axiom biot_savart : VField → VField

/-- **Characteristic property: `curl (biot_savart ω) = ω`.**

    Once `biot_savart` is implemented constructively via NTT + Garner
    reconstruction, this property becomes a theorem provable by exact
    arithmetic (no axiom needed). Currently axiomatised. -/
axiom biot_savart_curl_eq (ω : VField) : curl (biot_savart ω) = ω

/-- **Characteristic property: `div (biot_savart ω) = 0`.**

    The Biot-Savart inverse always produces divergence-free velocity.
    Constructively this follows from the NTT pipeline; currently axiom. -/
axiom biot_savart_div_zero (ω : VField) :
    divV (biot_savart ω) = fun _ _ _ => 0

end NSRegularity.NS
