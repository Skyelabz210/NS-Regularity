/-!
# Shift/Defs — periodic torus primitives (shared by all macros)

Plain definitions only. No theorems. All downstream leaves import this.
-/

namespace NSRegularity

/-- Lattice size of the discrete 3-torus. `abbrev` so `decide`/`simp` can reduce it. -/
abbrev N : Nat := 8

instance : NeZero N := ⟨by decide⟩

/-- Periodic successor in `Fin N`. -/
def sp (i : Fin N) : Fin N := i + 1

/-- Periodic predecessor in `Fin N`. -/
def sn (i : Fin N) : Fin N := i + ⟨N - 1, by decide⟩

/-- Scalar integer field on `(ℤ/Nℤ)³`. -/
abbrev Grid := Fin N → Fin N → Fin N → Int

/-- Three-component vector field on `(ℤ/Nℤ)³`. -/
structure VField where
  x : Grid
  y : Grid
  z : Grid

/-- Triple sum over the 3-torus. -/
def sumGrid (f : Fin N → Fin N → Fin N → Int) : Int :=
  (List.finRange N).foldl (fun a i =>
    a + (List.finRange N).foldl (fun b j =>
      b + (List.finRange N).foldl (fun c k => c + f i j k) 0) 0) 0

end NSRegularity
