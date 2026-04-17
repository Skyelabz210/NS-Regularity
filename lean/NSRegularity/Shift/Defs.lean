import Mathlib

/-!
# Shift/Defs — periodic torus primitives (shared by all macros)

Plain definitions only. No theorems. All downstream leaves import this.
Mathlib is imported here so all downstream leaves have access to its tactics
and lemmas (Finset.sum, List.Perm, etc.).
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

/-- Triple sum over the 3-torus, using mathlib's `Finset.sum`. This form
    makes reindexing by bijections trivial via `Finset.sum_equiv`. -/
def sumGrid (f : Fin N → Fin N → Fin N → Int) : Int :=
  ∑ i : Fin N, ∑ j : Fin N, ∑ k : Fin N, f i j k

end NSRegularity
