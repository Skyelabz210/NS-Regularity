import NSRegularity.Adelic
import NSRegularity.NS.Defs

/-!
# Adelic / Laplacian as PolyGridOp

Demonstrates that the 7-point discrete Laplacian is expressible as a
concrete `PolyGridOp`, and that the constructive ring-hom descent
(`polyGridOp_mod_descent`) therefore applies to it as a theorem
(not an axiom).

This is the worked example demonstrating that the M5 axiom
`polynomial_grid_op_commutes` is dischargeable for actual NS operators
via the constructive `PolyGridOp` class.
-/

namespace NSRegularity.Adelic

open NSRegularity NSRegularity.NS NSRegularity.Adelic

/-- The 7-point discrete Laplacian, expressed as a `PolyGridOp`.

    `Δf = f(x+1) + f(x-1) + f(y+1) + f(y-1) + f(z+1) + f(z-1) − 6 f`

    The `6 · f` term is written as six copies of `id` summed (since the
    constructive `PolyGridOp` class intentionally omits `constG`). -/
def laplacianOp : PolyGridOp :=
  -- 6 · f = id + id + id + id + id + id  (six copies)
  let sixId : PolyGridOp :=
    .add (.add (.add .id .id) .id) (.add (.add .id .id) .id)
  -- sum of 6 neighbours
  let neighbours : PolyGridOp :=
    .add (.add (.shiftXp .id) (.shiftXn .id))
         (.add (.add (.shiftYp .id) (.shiftYn .id))
               (.add (.shiftZp .id) (.shiftZn .id)))
  .sub neighbours sixId

/-- The 7-point Laplacian agrees with `laplacianOp.eval`. -/
theorem laplacianOp_eval (f : Grid) (i j k : Fin N) :
    laplacianOp.eval f i j k = laplacian f i j k := by
  simp only [laplacianOp, laplacian, PolyGridOp.eval]
  ring

/-- **Adelic identification for the discrete Laplacian (pointwise).**

    For every grid point `(i,j,k)`,
      `laplacian g i j k ≡ laplacian (gridMod p g) i j k (mod p)`.

    Direct corollary of `polyGridOp_mod_descent_pointwise` for `laplacianOp`.
    This is the operational content of M5 (the adelic identification) for
    the Laplacian — discharged via constructive induction, no axiom appeal. -/
theorem laplacian_mod_descent_pointwise (p_val : ℤ) (g : Grid)
    (i j k : Fin N) :
    laplacian g i j k % p_val = laplacian (gridMod p_val g) i j k % p_val := by
  have h := polyGridOp_mod_descent_pointwise laplacianOp p_val g i j k
  rw [laplacianOp_eval, laplacianOp_eval] at h
  exact h

/-- **Adelic identification for the discrete Laplacian (grid).** -/
theorem laplacian_mod_descent (p_val : ℤ) (g : Grid) :
    gridMod p_val (fun i j k => laplacian g i j k) =
    gridMod p_val (fun i j k => laplacian (gridMod p_val g) i j k) := by
  funext i j k
  exact laplacian_mod_descent_pointwise p_val g i j k

end NSRegularity.Adelic
