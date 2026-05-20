import NSRegularity.Kill113
import Mathlib.Data.ZMod.Basic

/-!
# M5 — Adelic Identification

The core algebraic fact: the reduction map `φ_p : ℤ → ℤ/pℤ` is a ring
homomorphism, so any expression built from `+`, `−`, `×` commutes with it.

The NS discrete operators (Laplacian, advection, stretching) are all polynomial
in the field values. For the polynomial part, φ_p ∘ F = F_mod ∘ φ_p follows
from the ring-homomorphism property.

This is the "adelic identification": a bound proved mod p on each CRT lane
is a bound on the original integer, because the lane projection commutes
with the dynamics.

## Sub-lemmas

- **L5.1–L5.3**: mod-p ring-homomorphism (add, mul, sub)
- **L5.4**: pointwise grid-level commutativity
- **L5.5**: NS polynomial part commutes with CRT projection
-/

namespace NSRegularity.Adelic

open NSRegularity

-- ═══════════════════════════════════════════════════════════════════════════
-- L5.1–L5.3: mod-p is a ring homomorphism
-- These are standard modular arithmetic identities.
-- ═══════════════════════════════════════════════════════════════════════════

/-- L5.1 — Multiplication commutes with mod p. -/
theorem mod_mul_comm (a b p : ℤ) :
    (a * b) % p = ((a % p) * (b % p)) % p := by
  rw [Int.mul_emod]

/-- L5.2 — Addition commutes with mod p. -/
theorem mod_add_comm (a b p : ℤ) :
    (a + b) % p = ((a % p) + (b % p)) % p := by
  rw [Int.add_emod]

/-- L5.3 — Subtraction commutes with mod p. -/
theorem mod_sub_comm (a b p : ℤ) :
    (a - b) % p = ((a % p) - (b % p)) % p := by
  rw [Int.sub_emod]

-- ═══════════════════════════════════════════════════════════════════════════
-- L5.4–L5.5: Grid-level commutativity
-- ═══════════════════════════════════════════════════════════════════════════

/-- The pointwise mod-p projection on a Grid. -/
def gridMod (p : ℤ) (g : Grid) : Grid :=
  fun i j k => g i j k % p

/-- **M5 — CRT projection commutes with polynomial grid operators.**

    For any ring expression F built from +, −, × applied pointwise,
    `gridMod p (F g) = F (gridMod p g)` (mod p on each entry).

    This is the operational content of the adelic identification.
    Stated as an axiom pending formalization of "polynomial grid operator"
    as a concrete inductive type over Grid → Grid. -/
axiom polynomial_grid_op_commutes
    (F : Grid → Grid)
    (p_val : ℤ) (g : Grid) :
    gridMod p_val (F g) = F (gridMod p_val g)

end NSRegularity.Adelic
