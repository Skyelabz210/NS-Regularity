import NSRegularity.NS.Defs
import NSRegularity.Kill113.Ops
import NSRegularity.Shift.SpSn
import NSRegularity.Shift.SnSp
import NSRegularity.Shift.SumGrid
import NSRegularity.HSquared.SBP

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

/-- **Characteristic property: `curl (biot_savart ω) = ω` on divergence-free
    vorticities.**

    The precondition `divV ω = 0` is essential. Without it the axiom is
    unsound: composing with the proved theorem `curl_div_zero` would force
    `divV ω = 0` for *every* `ω`, which is false on the discrete torus
    (see `NSRegularity/SoundnessProbe.lean` for a concrete witness with
    `divV ω 0 0 0 = −6`). On the NS trajectory the precondition holds
    automatically: every vorticity field is itself a curl of a velocity,
    so `divV ω = 0` by `curl_div_zero`.

    Once `biot_savart` is implemented constructively via NTT + Garner
    reconstruction, this property becomes a theorem provable by exact
    arithmetic (no axiom needed). Currently axiomatised. -/
axiom biot_savart_curl_eq (ω : VField)
    (hdiv : divV ω = (fun (_ _ _ : Fin N) => (0 : Int))) :
    curl (biot_savart ω) = ω

/-- **Characteristic property: `div (biot_savart ω) = 0`.**

    The Biot-Savart inverse always produces divergence-free velocity.
    Constructively this follows from the NTT pipeline; currently axiom.
    No precondition is required: any candidate Biot-Savart inverse must
    map into the divergence-free subspace by construction. -/
axiom biot_savart_div_zero (ω : VField) :
    divV (biot_savart ω) = fun _ _ _ => 0

/-! ## Vector-calculus identities — provable from definitions (not axiomatic) -/

/-- Discrete gradient of a scalar field: `(grad f)_d = ∂_d f`. -/
def gradV (f : Grid) : VField :=
  ⟨fun i j k => NSRegularity.Kill113.diffX f i j k,
   fun i j k => NSRegularity.Kill113.diffY f i j k,
   fun i j k => NSRegularity.Kill113.diffZ f i j k⟩

/-- **`div ∘ curl = 0`** — the standard vector-calculus identity holds for
    the discrete operators via centered differences. Provable directly from
    the definitions: each second-difference term `∂_a ∂_b w_c` cancels with
    its commuted counterpart `∂_b ∂_a w_c` (centered diffs commute pointwise). -/
theorem curl_div_zero (w : VField) :
    divV (curl w) = fun _ _ _ => 0 := by
  funext i j k
  unfold divV curl curlX curlY curlZ
  unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
  ring

/-- **`curl ∘ grad = 0`** — second standard vector-calc identity.
    Each curl component `∂_a ∂_b f - ∂_b ∂_a f` vanishes by commutativity
    of centered differences. -/
theorem curl_grad_zero (f : Grid) :
    curl (gradV f) =
      ⟨fun _ _ _ => 0, fun _ _ _ => 0, fun _ _ _ => 0⟩ := by
  have hx : curlX (gradV f) = fun _ _ _ => 0 := by
    funext i j k
    unfold curlX gradV
    unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
    ring
  have hy : curlY (gradV f) = fun _ _ _ => 0 := by
    funext i j k
    unfold curlY gradV
    unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
    ring
  have hz : curlZ (gradV f) = fun _ _ _ => 0 := by
    funext i j k
    unfold curlZ gradV
    unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
    ring
  unfold curl
  rw [hx, hy, hz]

/-! ## Vector-calculus operator linearity -/

/-- **Discrete divergence is linear (additive).** -/
theorem divV_add (w v : VField) (i j k : Fin N) :
    divV (⟨fun a b c => w.x a b c + v.x a b c,
           fun a b c => w.y a b c + v.y a b c,
           fun a b c => w.z a b c + v.z a b c⟩) i j k =
    divV w i j k + divV v i j k := by
  unfold divV
  unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
  ring

/-- **Discrete gradient is linear (additive).** -/
theorem gradV_add (f g : Grid) :
    gradV (fun i j k => f i j k + g i j k) =
    ⟨fun i j k => (gradV f).x i j k + (gradV g).x i j k,
     fun i j k => (gradV f).y i j k + (gradV g).y i j k,
     fun i j k => (gradV f).z i j k + (gradV g).z i j k⟩ := by
  unfold gradV
  unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
  congr <;> (funext i j k; ring)

/-- **Discrete curl is linear (additive, component x).** -/
theorem curlX_add (w v : VField) (i j k : Fin N) :
    curlX (⟨fun a b c => w.x a b c + v.x a b c,
            fun a b c => w.y a b c + v.y a b c,
            fun a b c => w.z a b c + v.z a b c⟩) i j k =
    curlX w i j k + curlX v i j k := by
  unfold curlX
  unfold NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
  ring

/-- **Discrete curl is linear (additive, component y).** -/
theorem curlY_add (w v : VField) (i j k : Fin N) :
    curlY (⟨fun a b c => w.x a b c + v.x a b c,
            fun a b c => w.y a b c + v.y a b c,
            fun a b c => w.z a b c + v.z a b c⟩) i j k =
    curlY w i j k + curlY v i j k := by
  unfold curlY
  unfold NSRegularity.Kill113.diffZ NSRegularity.Kill113.diffX
  ring

/-- **Discrete curl is linear (additive, component z).** -/
theorem curlZ_add (w v : VField) (i j k : Fin N) :
    curlZ (⟨fun a b c => w.x a b c + v.x a b c,
            fun a b c => w.y a b c + v.y a b c,
            fun a b c => w.z a b c + v.z a b c⟩) i j k =
    curlZ w i j k + curlZ v i j k := by
  unfold curlZ
  unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY
  ring

/-! ## Composed Laplacian — `div ∘ grad` gives the second-neighbor stencil -/

/-- **Second-neighbor (composed) Laplacian.** This is the operator that
    naturally arises from composing centered-difference `div` and `grad`:
    `(laplacianH f)(i,j,k) = f(sp²i) + f(sn²i) + f(sp²j) + f(sn²j) + f(sp²k) + f(sn²k) − 6·f(i,j,k)`.

    Distinct from the standard 7-point `laplacian` in `NS/Defs.lean` (which
    uses first-neighbor differences `sp`/`sn`). The two laplacians coincide
    on the periodic torus modulo a rescaling of the index variable but are
    formally distinct operators. -/
def laplacianH (f : Grid) (i j k : Fin N) : Int :=
  f (sp (sp i)) j k + f (sn (sn i)) j k
  + f i (sp (sp j)) k + f i (sn (sn j)) k
  + f i j (sp (sp k)) + f i j (sn (sn k))
  - 6 * f i j k

/-- **Discrete Hodge bridge identity: `divV ∘ gradV = laplacianH`.**

    The composed centered-difference Laplacian equals the second-neighbor
    stencil. Proof: unfold definitions, collapse `sp ∘ sn = sn ∘ sp = id`,
    then ring. -/
theorem divV_gradV_eq_laplacianH (f : Grid) (i j k : Fin N) :
    divV (gradV f) i j k = laplacianH f i j k := by
  unfold divV gradV laplacianH
  unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
  simp only [NSRegularity.Shift.sn_sp, NSRegularity.Shift.sp_sn]
  ring

/-! ## Integration by parts (centered-difference form) -/

/-- **IBP for the x-component centered difference:** `Σ f · diffX g = -Σ diffX f · g`.

    Proved by composing the SBP cross-term reindex lemmas `cross_x_sp` and
    `cross_x_sn` (Wave F) with linearity of `sumGrid` and `diffX`. -/
theorem sumGrid_f_diffX_eq_neg_diffX_g (f g : Grid) :
    sumGrid (fun i j k => f i j k * NSRegularity.Kill113.diffX g i j k) =
    -sumGrid (fun i j k => NSRegularity.Kill113.diffX f i j k * g i j k) := by
  have hLHS_split : sumGrid (fun i j k => f i j k * NSRegularity.Kill113.diffX g i j k) =
                    sumGrid (fun i j k => f i j k * g (sp i) j k) -
                    sumGrid (fun i j k => f i j k * g (sn i) j k) := by
    unfold sumGrid NSRegularity.Kill113.diffX
    have hpt : ∀ i j k : Fin N,
        f i j k * (g (sp i) j k - g (sn i) j k) =
        f i j k * g (sp i) j k - f i j k * g (sn i) j k := by intros; ring
    simp only [hpt, Finset.sum_sub_distrib]
  have hRHS_split : sumGrid (fun i j k =>
                      NSRegularity.Kill113.diffX f i j k * g i j k) =
                    sumGrid (fun i j k => f (sp i) j k * g i j k) -
                    sumGrid (fun i j k => f (sn i) j k * g i j k) := by
    unfold sumGrid NSRegularity.Kill113.diffX
    have hpt : ∀ i j k : Fin N,
        (f (sp i) j k - f (sn i) j k) * g i j k =
        f (sp i) j k * g i j k - f (sn i) j k * g i j k := by intros; ring
    simp only [hpt, Finset.sum_sub_distrib]
  have hCx_sp := NSRegularity.HSquared.cross_x_sp f g
  have hCx_sn := NSRegularity.HSquared.cross_x_sn f g
  rw [hLHS_split, hCx_sp, hCx_sn, hRHS_split]
  ring

/-- **IBP for the y-component centered difference.** -/
theorem sumGrid_f_diffY_eq_neg_diffY_g (f g : Grid) :
    sumGrid (fun i j k => f i j k * NSRegularity.Kill113.diffY g i j k) =
    -sumGrid (fun i j k => NSRegularity.Kill113.diffY f i j k * g i j k) := by
  have hLHS_split : sumGrid (fun i j k => f i j k * NSRegularity.Kill113.diffY g i j k) =
                    sumGrid (fun i j k => f i j k * g i (sp j) k) -
                    sumGrid (fun i j k => f i j k * g i (sn j) k) := by
    unfold sumGrid NSRegularity.Kill113.diffY
    have hpt : ∀ i j k : Fin N,
        f i j k * (g i (sp j) k - g i (sn j) k) =
        f i j k * g i (sp j) k - f i j k * g i (sn j) k := by intros; ring
    simp only [hpt, Finset.sum_sub_distrib]
  have hRHS_split : sumGrid (fun i j k =>
                      NSRegularity.Kill113.diffY f i j k * g i j k) =
                    sumGrid (fun i j k => f i (sp j) k * g i j k) -
                    sumGrid (fun i j k => f i (sn j) k * g i j k) := by
    unfold sumGrid NSRegularity.Kill113.diffY
    have hpt : ∀ i j k : Fin N,
        (f i (sp j) k - f i (sn j) k) * g i j k =
        f i (sp j) k * g i j k - f i (sn j) k * g i j k := by intros; ring
    simp only [hpt, Finset.sum_sub_distrib]
  have hCy_sp := NSRegularity.HSquared.cross_y_sp f g
  have hCy_sn := NSRegularity.HSquared.cross_y_sn f g
  rw [hLHS_split, hCy_sp, hCy_sn, hRHS_split]
  ring

/-- **Pointwise vector-field dot product on grids.** -/
def vfDot (u v : VField) (i j k : Fin N) : Int :=
  u.x i j k * v.x i j k + u.y i j k * v.y i j k + u.z i j k * v.z i j k

/-- **IBP for the z-component centered difference.** -/
theorem sumGrid_f_diffZ_eq_neg_diffZ_g (f g : Grid) :
    sumGrid (fun i j k => f i j k * NSRegularity.Kill113.diffZ g i j k) =
    -sumGrid (fun i j k => NSRegularity.Kill113.diffZ f i j k * g i j k) := by
  have hLHS_split : sumGrid (fun i j k => f i j k * NSRegularity.Kill113.diffZ g i j k) =
                    sumGrid (fun i j k => f i j k * g i j (sp k)) -
                    sumGrid (fun i j k => f i j k * g i j (sn k)) := by
    unfold sumGrid NSRegularity.Kill113.diffZ
    have hpt : ∀ i j k : Fin N,
        f i j k * (g i j (sp k) - g i j (sn k)) =
        f i j k * g i j (sp k) - f i j k * g i j (sn k) := by intros; ring
    simp only [hpt, Finset.sum_sub_distrib]
  have hRHS_split : sumGrid (fun i j k =>
                      NSRegularity.Kill113.diffZ f i j k * g i j k) =
                    sumGrid (fun i j k => f i j (sp k) * g i j k) -
                    sumGrid (fun i j k => f i j (sn k) * g i j k) := by
    unfold sumGrid NSRegularity.Kill113.diffZ
    have hpt : ∀ i j k : Fin N,
        (f i j (sp k) - f i j (sn k)) * g i j k =
        f i j (sp k) * g i j k - f i j (sn k) * g i j k := by intros; ring
    simp only [hpt, Finset.sum_sub_distrib]
  have hCz_sp := NSRegularity.HSquared.cross_z_sp f g
  have hCz_sn := NSRegularity.HSquared.cross_z_sn f g
  rw [hLHS_split, hCz_sp, hCz_sn, hRHS_split]
  ring

/-- **Vector-field integration-by-parts: `Σ f · divV w = -Σ gradV f · w`.**

    The discrete Green's identity for the centered-difference div/grad pair.
    Direct composition of the three component-wise IBPs (Wave H.11-H.12). -/
theorem sumGrid_f_divV_eq_neg_gradV_dot (f : Grid) (w : VField) :
    sumGrid (fun i j k => f i j k * divV w i j k) =
    -sumGrid (fun i j k => vfDot (gradV f) w i j k) := by
  have hLHS_split : sumGrid (fun i j k => f i j k * divV w i j k) =
      sumGrid (fun i j k => f i j k * NSRegularity.Kill113.diffX w.x i j k) +
      sumGrid (fun i j k => f i j k * NSRegularity.Kill113.diffY w.y i j k) +
      sumGrid (fun i j k => f i j k * NSRegularity.Kill113.diffZ w.z i j k) := by
    unfold sumGrid divV
    have hpt : ∀ i j k : Fin N,
        f i j k * (NSRegularity.Kill113.diffX w.x i j k +
                   NSRegularity.Kill113.diffY w.y i j k +
                   NSRegularity.Kill113.diffZ w.z i j k) =
        f i j k * NSRegularity.Kill113.diffX w.x i j k +
        f i j k * NSRegularity.Kill113.diffY w.y i j k +
        f i j k * NSRegularity.Kill113.diffZ w.z i j k := by intros; ring
    simp only [hpt, Finset.sum_add_distrib]
  have hRHS_split : sumGrid (fun i j k => vfDot (gradV f) w i j k) =
      sumGrid (fun i j k => (gradV f).x i j k * w.x i j k) +
      sumGrid (fun i j k => (gradV f).y i j k * w.y i j k) +
      sumGrid (fun i j k => (gradV f).z i j k * w.z i j k) := by
    unfold sumGrid vfDot
    simp only [Finset.sum_add_distrib]
  have hX := sumGrid_f_diffX_eq_neg_diffX_g f w.x
  have hY := sumGrid_f_diffY_eq_neg_diffY_g f w.y
  have hZ := sumGrid_f_diffZ_eq_neg_diffZ_g f w.z
  rw [hLHS_split, hX, hY, hZ, hRHS_split]
  -- The three sumGrid (diffX f · w.x), etc. on the LHS equal the sumGrid (gradV f).c · w.c on RHS
  -- by definition of gradV: (gradV f).x = diffX f, etc.
  show -sumGrid (fun i j k => NSRegularity.Kill113.diffX f i j k * w.x i j k) +
       -sumGrid (fun i j k => NSRegularity.Kill113.diffY f i j k * w.y i j k) +
       -sumGrid (fun i j k => NSRegularity.Kill113.diffZ f i j k * w.z i j k)
     = -(sumGrid (fun i j k => (gradV f).x i j k * w.x i j k) +
         sumGrid (fun i j k => (gradV f).y i j k * w.y i j k) +
         sumGrid (fun i j k => (gradV f).z i j k * w.z i j k))
  unfold gradV
  ring

/-- **H¹ quadratic identity for `laplacianH`** (the second-neighbor Laplacian).
    `Σ f · laplacianH f = -Σ |gradV f|²`.

    Composes the vector-field IBP (Wave H.14) with the bridge identity
    `divV ∘ gradV = laplacianH` (Wave H.10). This is the discrete H¹
    energy identity for the centered-difference composed Laplacian. -/
theorem sbp_quadratic_H (f : Grid) :
    sumGrid (fun i j k => f i j k * laplacianH f i j k) =
    -sumGrid (fun i j k => vfDot (gradV f) (gradV f) i j k) := by
  -- Σ f · laplacianH f = Σ f · divV (gradV f) (by bridge identity)
  --                   = -Σ gradV f · gradV f (by vector-field IBP)
  have hbridge : ∀ i j k : Fin N,
      f i j k * laplacianH f i j k = f i j k * divV (gradV f) i j k := by
    intros i j k
    rw [divV_gradV_eq_laplacianH]
  have hstep1 : sumGrid (fun i j k => f i j k * laplacianH f i j k) =
                sumGrid (fun i j k => f i j k * divV (gradV f) i j k) := by
    unfold sumGrid
    apply Finset.sum_congr rfl; intros i _
    apply Finset.sum_congr rfl; intros j _
    apply Finset.sum_congr rfl; intros k _
    exact hbridge i j k
  rw [hstep1]
  exact sumGrid_f_divV_eq_neg_gradV_dot f (gradV f)

/-- **H² quadratic identity for `laplacianH`.**
    `Σ laplacianH f · laplacianH(laplacianH f) = -Σ |gradV(laplacianH f)|²`.

    Direct corollary of `sbp_quadratic_H` specialised to `f := laplacianH f`.
    The iterated second-neighbor Laplacian satisfies its own H² energy
    identity, analogous to `sbp_quadratic_h2` for the first-neighbor case. -/
theorem sbp_quadratic_H_h2 (f : Grid) :
    sumGrid (fun i j k => laplacianH f i j k * laplacianH (laplacianH f) i j k) =
    -sumGrid (fun i j k => vfDot (gradV (laplacianH f)) (gradV (laplacianH f)) i j k) :=
  sbp_quadratic_H (laplacianH f)

/-! ## Harmonic functions and the discrete Hodge harmonic space -/

/-- A scalar field is **harmonic** if its (second-neighbor) Laplacian
    vanishes everywhere. -/
def isHarmonicH (f : Grid) : Prop := ∀ i j k : Fin N, laplacianH f i j k = 0

/-- A scalar field is **harmonic (first-neighbor)** if its 7-point
    Laplacian vanishes everywhere. -/
def isHarmonic (f : Grid) : Prop := ∀ i j k : Fin N, laplacian f i j k = 0

/-- **Constants are harmonic (first-neighbor).** -/
theorem const_isHarmonic (c : Int) : isHarmonic (fun _ _ _ => c) := by
  intros i j k; unfold laplacian; ring

/-- **Constants are harmonic (second-neighbor).** -/
theorem const_isHarmonicH (c : Int) : isHarmonicH (fun _ _ _ => c) := by
  intros i j k; unfold laplacianH; ring

/-- **Sum of two harmonic functions is harmonic (first-neighbor).** -/
theorem isHarmonic_add (f g : Grid) (hf : isHarmonic f) (hg : isHarmonic g) :
    isHarmonic (fun i j k => f i j k + g i j k) := by
  intros i j k
  rw [laplacian_add]
  rw [hf, hg]
  ring

/-- **Scalar multiple of a harmonic function is harmonic (first-neighbor).** -/
theorem isHarmonic_smul (c : Int) (f : Grid) (hf : isHarmonic f) :
    isHarmonic (fun i j k => c * f i j k) := by
  intros i j k
  rw [laplacian_smul]
  rw [hf]
  ring

/-- **`gradV` of a constant is the zero vector field.** -/
theorem gradV_const_eq_zero (c : Int) :
    gradV (fun _ _ _ => c) =
      ⟨fun _ _ _ => 0, fun _ _ _ => 0, fun _ _ _ => 0⟩ := by
  unfold gradV
  unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
  congr <;> (funext i j k; ring)

/-- **`divV` of a constant vector field is zero.** -/
theorem divV_const_eq_zero (cx cy cz : Int) :
    divV (⟨fun _ _ _ => cx, fun _ _ _ => cy, fun _ _ _ => cz⟩) =
      fun _ _ _ => 0 := by
  funext i j k
  unfold divV
  unfold NSRegularity.Kill113.diffX NSRegularity.Kill113.diffY NSRegularity.Kill113.diffZ
  ring

/-! ## Discrete divergence theorem on the torus -/

open NSRegularity.Shift in
/-- **Σ diffX f = 0 over the torus** (zero-boundary). -/
theorem sumGrid_diffX_eq_zero (f : Grid) :
    sumGrid (fun i j k => NSRegularity.Kill113.diffX f i j k) = 0 := by
  unfold sumGrid NSRegularity.Kill113.diffX
  have hSplit : ∀ i j k : Fin N,
      f (sp i) j k - f (sn i) j k =
      f (sp i) j k - f (sn i) j k := fun _ _ _ => rfl
  simp only [Finset.sum_sub_distrib]
  have h_sp : sumGrid (fun i j k => f (sp i) j k) = sumGrid f := sumGrid_sp_x f
  have h_sn : sumGrid (fun i j k => f (sn i) j k) = sumGrid f := sumGrid_sn_x f
  unfold sumGrid at h_sp h_sn
  rw [h_sp, h_sn]
  ring

open NSRegularity.Shift in
/-- **Σ diffY f = 0 over the torus**. -/
theorem sumGrid_diffY_eq_zero (f : Grid) :
    sumGrid (fun i j k => NSRegularity.Kill113.diffY f i j k) = 0 := by
  unfold sumGrid NSRegularity.Kill113.diffY
  simp only [Finset.sum_sub_distrib]
  have h_sp : sumGrid (fun i j k => f i (sp j) k) = sumGrid f := sumGrid_sp_y f
  have h_sn : sumGrid (fun i j k => f i (sn j) k) = sumGrid f := sumGrid_sn_y f
  unfold sumGrid at h_sp h_sn
  rw [h_sp, h_sn]
  ring

open NSRegularity.Shift in
/-- **Σ diffZ f = 0 over the torus**. -/
theorem sumGrid_diffZ_eq_zero (f : Grid) :
    sumGrid (fun i j k => NSRegularity.Kill113.diffZ f i j k) = 0 := by
  unfold sumGrid NSRegularity.Kill113.diffZ
  simp only [Finset.sum_sub_distrib]
  have h_sp : sumGrid (fun i j k => f i j (sp k)) = sumGrid f := sumGrid_sp_z f
  have h_sn : sumGrid (fun i j k => f i j (sn k)) = sumGrid f := sumGrid_sn_z f
  unfold sumGrid at h_sp h_sn
  rw [h_sp, h_sn]
  ring

/-- **Discrete divergence theorem on the torus: `Σ divV w = 0`.**

    On the periodic 3-torus (no boundary), the integral of any divergence
    vanishes. Discrete version: sum over all grid points of `divV w` is
    identically zero, proved by decomposing into component sums each of
    which telescopes to zero by `sumGrid_{sp,sn}_{x,y,z}` invariance. -/
theorem sumGrid_divV_eq_zero (w : VField) :
    sumGrid (fun i j k => divV w i j k) = 0 := by
  unfold sumGrid divV
  simp only [Finset.sum_add_distrib]
  have hX := sumGrid_diffX_eq_zero w.x
  have hY := sumGrid_diffY_eq_zero w.y
  have hZ := sumGrid_diffZ_eq_zero w.z
  unfold sumGrid at hX hY hZ
  linarith

/-- **`Σ curlX w = 0`** — x-component of curl integrates to zero on torus. -/
theorem sumGrid_curlX_eq_zero (w : VField) :
    sumGrid (fun i j k => curlX w i j k) = 0 := by
  unfold sumGrid curlX
  simp only [Finset.sum_sub_distrib]
  have hY := sumGrid_diffY_eq_zero w.z
  have hZ := sumGrid_diffZ_eq_zero w.y
  unfold sumGrid at hY hZ
  linarith

/-- **`Σ curlY w = 0`**. -/
theorem sumGrid_curlY_eq_zero (w : VField) :
    sumGrid (fun i j k => curlY w i j k) = 0 := by
  unfold sumGrid curlY
  simp only [Finset.sum_sub_distrib]
  have hZ := sumGrid_diffZ_eq_zero w.x
  have hX := sumGrid_diffX_eq_zero w.z
  unfold sumGrid at hZ hX
  linarith

/-- **`Σ curlZ w = 0`**. -/
theorem sumGrid_curlZ_eq_zero (w : VField) :
    sumGrid (fun i j k => curlZ w i j k) = 0 := by
  unfold sumGrid curlZ
  simp only [Finset.sum_sub_distrib]
  have hX := sumGrid_diffX_eq_zero w.y
  have hY := sumGrid_diffY_eq_zero w.x
  unfold sumGrid at hX hY
  linarith

/-! ## Pointwise vector dot-product properties -/

/-- **`vfDot` is symmetric:** `vfDot u v = vfDot v u`. -/
theorem vfDot_comm (u v : VField) (i j k : Fin N) :
    vfDot u v i j k = vfDot v u i j k := by
  unfold vfDot; ring

/-- **`vfDot` is additive in the first argument.** -/
theorem vfDot_add_left (u₁ u₂ v : VField) (i j k : Fin N) :
    vfDot (⟨fun a b c => u₁.x a b c + u₂.x a b c,
            fun a b c => u₁.y a b c + u₂.y a b c,
            fun a b c => u₁.z a b c + u₂.z a b c⟩) v i j k =
    vfDot u₁ v i j k + vfDot u₂ v i j k := by
  unfold vfDot; ring

/-- **`vfDot` is additive in the second argument.** -/
theorem vfDot_add_right (u v₁ v₂ : VField) (i j k : Fin N) :
    vfDot u (⟨fun a b c => v₁.x a b c + v₂.x a b c,
              fun a b c => v₁.y a b c + v₂.y a b c,
              fun a b c => v₁.z a b c + v₂.z a b c⟩) i j k =
    vfDot u v₁ i j k + vfDot u v₂ i j k := by
  unfold vfDot; ring

/-- **`vfDot u u ≥ 0`** — non-negativity of the pointwise self-product. -/
theorem vfDot_self_nonneg (u : VField) (i j k : Fin N) :
    0 ≤ vfDot u u i j k := by
  unfold vfDot
  have h1 : 0 ≤ (u.x i j k) ^ 2 := sq_nonneg _
  have h2 : 0 ≤ (u.y i j k) ^ 2 := sq_nonneg _
  have h3 : 0 ≤ (u.z i j k) ^ 2 := sq_nonneg _
  have ex1 : u.x i j k * u.x i j k = (u.x i j k) ^ 2 := by ring
  have ex2 : u.y i j k * u.y i j k = (u.y i j k) ^ 2 := by ring
  have ex3 : u.z i j k * u.z i j k = (u.z i j k) ^ 2 := by ring
  rw [ex1, ex2, ex3]
  linarith

/-! ## Discrete L² inner product on vector fields -/

/-- **Discrete L² inner product** of two vector fields on the torus. -/
def innerL2 (u v : VField) : Int := sumGrid (fun i j k => vfDot u v i j k)

/-- **innerL2 is symmetric.** -/
theorem innerL2_comm (u v : VField) : innerL2 u v = innerL2 v u := by
  unfold innerL2 sumGrid
  apply Finset.sum_congr rfl; intros i _
  apply Finset.sum_congr rfl; intros j _
  apply Finset.sum_congr rfl; intros k _
  exact vfDot_comm u v i j k

/-- **innerL2 self-product is non-negative.** -/
theorem innerL2_self_nonneg (u : VField) : 0 ≤ innerL2 u u := by
  unfold innerL2 sumGrid
  apply Finset.sum_nonneg; intros i _
  apply Finset.sum_nonneg; intros j _
  apply Finset.sum_nonneg; intros k _
  exact vfDot_self_nonneg u i j k

/-- **innerL2 is additive in the first argument.** -/
theorem innerL2_add_left (u₁ u₂ v : VField) :
    innerL2 (⟨fun a b c => u₁.x a b c + u₂.x a b c,
              fun a b c => u₁.y a b c + u₂.y a b c,
              fun a b c => u₁.z a b c + u₂.z a b c⟩) v =
    innerL2 u₁ v + innerL2 u₂ v := by
  unfold innerL2 sumGrid
  have hpt := vfDot_add_left u₁ u₂ v
  simp only [hpt, Finset.sum_add_distrib]

/-- **Non-degeneracy at point: `vfDot u u = 0 ↔ u = 0` pointwise.** -/
theorem vfDot_self_eq_zero_iff (u : VField) (i j k : Fin N) :
    vfDot u u i j k = 0 ↔
      u.x i j k = 0 ∧ u.y i j k = 0 ∧ u.z i j k = 0 := by
  unfold vfDot
  constructor
  · intro h
    -- u_x² + u_y² + u_z² = 0 over Int, each squared term ≥ 0
    have h1 : 0 ≤ u.x i j k * u.x i j k := mul_self_nonneg _
    have h2 : 0 ≤ u.y i j k * u.y i j k := mul_self_nonneg _
    have h3 : 0 ≤ u.z i j k * u.z i j k := mul_self_nonneg _
    have hx : u.x i j k * u.x i j k = 0 := by linarith
    have hy : u.y i j k * u.y i j k = 0 := by linarith
    have hz : u.z i j k * u.z i j k = 0 := by linarith
    refine ⟨?_, ?_, ?_⟩
    · exact (mul_self_eq_zero.mp hx)
    · exact (mul_self_eq_zero.mp hy)
    · exact (mul_self_eq_zero.mp hz)
  · rintro ⟨hx, hy, hz⟩
    rw [hx, hy, hz]
    ring

/-- **Pointwise Cauchy-Schwarz for `vfDot`:** `(vfDot u v)² ≤ vfDot u u · vfDot v v`.

    The 3D analog of `(u_x v_x + u_y v_y + u_z v_z)² ≤ (u_x² + u_y² + u_z²)(v_x² + v_y² + v_z²)`.
    Proved via the standard SOS identity:
      LHS RHS - CS² = (u_x v_y − u_y v_x)² + (u_x v_z − u_z v_x)² + (u_y v_z − u_z v_y)² ≥ 0.
-/
theorem vfDot_cauchy_schwarz_pointwise (u v : VField) (i j k : Fin N) :
    (vfDot u v i j k) ^ 2 ≤ vfDot u u i j k * vfDot v v i j k := by
  unfold vfDot
  have h1 := sq_nonneg (u.x i j k * v.y i j k - u.y i j k * v.x i j k)
  have h2 := sq_nonneg (u.x i j k * v.z i j k - u.z i j k * v.x i j k)
  have h3 := sq_nonneg (u.y i j k * v.z i j k - u.z i j k * v.y i j k)
  nlinarith [h1, h2, h3, sq_nonneg (u.x i j k), sq_nonneg (u.y i j k),
             sq_nonneg (u.z i j k), sq_nonneg (v.x i j k), sq_nonneg (v.y i j k),
             sq_nonneg (v.z i j k)]

/-- **innerL2 is additive in the second argument.** -/
theorem innerL2_add_right (u v₁ v₂ : VField) :
    innerL2 u (⟨fun a b c => v₁.x a b c + v₂.x a b c,
                fun a b c => v₁.y a b c + v₂.y a b c,
                fun a b c => v₁.z a b c + v₂.z a b c⟩) =
    innerL2 u v₁ + innerL2 u v₂ := by
  unfold innerL2 sumGrid
  have hpt := vfDot_add_right u v₁ v₂
  simp only [hpt, Finset.sum_add_distrib]

open NSRegularity.Shift in
/-- **`Σ laplacian f = 0` on the torus.** -/
theorem sumGrid_laplacian_eq_zero (f : Grid) :
    sumGrid (fun i j k => laplacian f i j k) = 0 := by
  unfold sumGrid laplacian
  have h_xp : sumGrid (fun i j k => f (sp i) j k) = sumGrid f := sumGrid_sp_x f
  have h_xn : sumGrid (fun i j k => f (sn i) j k) = sumGrid f := sumGrid_sn_x f
  have h_yp : sumGrid (fun i j k => f i (sp j) k) = sumGrid f := sumGrid_sp_y f
  have h_yn : sumGrid (fun i j k => f i (sn j) k) = sumGrid f := sumGrid_sn_y f
  have h_zp : sumGrid (fun i j k => f i j (sp k)) = sumGrid f := sumGrid_sp_z f
  have h_zn : sumGrid (fun i j k => f i j (sn k)) = sumGrid f := sumGrid_sn_z f
  unfold sumGrid at h_xp h_xn h_yp h_yn h_zp h_zn
  simp only [Finset.sum_add_distrib, Finset.sum_sub_distrib, ← Finset.mul_sum]
  -- Goal reduces to: 6·Σf - 6·Σf = 0
  linarith

open NSRegularity.Shift in
/-- **`Σ laplacianH f = 0` on the torus.** -/
theorem sumGrid_laplacianH_eq_zero (f : Grid) :
    sumGrid (fun i j k => laplacianH f i j k) = 0 := by
  unfold sumGrid laplacianH
  -- Use second-neighbor shift invariance: Σ f(sp²) = Σ f, Σ f(sn²) = Σ f
  have h_xp : sumGrid (fun i j k => f (sp (sp i)) j k) = sumGrid f := by
    have h1 := sumGrid_sp_x (fun i j k => f (sp i) j k)
    have h2 := sumGrid_sp_x f
    rw [h1, h2]
  have h_xn : sumGrid (fun i j k => f (sn (sn i)) j k) = sumGrid f := by
    have h1 := sumGrid_sn_x (fun i j k => f (sn i) j k)
    have h2 := sumGrid_sn_x f
    rw [h1, h2]
  have h_yp : sumGrid (fun i j k => f i (sp (sp j)) k) = sumGrid f := by
    have h1 := sumGrid_sp_y (fun i j k => f i (sp j) k)
    have h2 := sumGrid_sp_y f
    rw [h1, h2]
  have h_yn : sumGrid (fun i j k => f i (sn (sn j)) k) = sumGrid f := by
    have h1 := sumGrid_sn_y (fun i j k => f i (sn j) k)
    have h2 := sumGrid_sn_y f
    rw [h1, h2]
  have h_zp : sumGrid (fun i j k => f i j (sp (sp k))) = sumGrid f := by
    have h1 := sumGrid_sp_z (fun i j k => f i j (sp k))
    have h2 := sumGrid_sp_z f
    rw [h1, h2]
  have h_zn : sumGrid (fun i j k => f i j (sn (sn k))) = sumGrid f := by
    have h1 := sumGrid_sn_z (fun i j k => f i j (sn k))
    have h2 := sumGrid_sn_z f
    rw [h1, h2]
  unfold sumGrid at h_xp h_xn h_yp h_yn h_zp h_zn
  simp only [Finset.sum_add_distrib, Finset.sum_sub_distrib, ← Finset.mul_sum]
  linarith

/-- **`Σ gradV f`** — gradient integrates to zero (componentwise) on torus. -/
theorem sumGrid_gradV_eq_zero (f : Grid) :
    sumGrid (fun i j k => (gradV f).x i j k) = 0 ∧
    sumGrid (fun i j k => (gradV f).y i j k) = 0 ∧
    sumGrid (fun i j k => (gradV f).z i j k) = 0 := by
  refine ⟨?_, ?_, ?_⟩
  · show sumGrid (fun i j k => NSRegularity.Kill113.diffX f i j k) = 0
    exact sumGrid_diffX_eq_zero f
  · show sumGrid (fun i j k => NSRegularity.Kill113.diffY f i j k) = 0
    exact sumGrid_diffY_eq_zero f
  · show sumGrid (fun i j k => NSRegularity.Kill113.diffZ f i j k) = 0
    exact sumGrid_diffZ_eq_zero f

end NSRegularity.NS
