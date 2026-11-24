import Std

/-! Exact finite certificate for the raw centered inverse at N=8.
    Each parity class is a 4³ torus. The Green table has denominator 1920.
    The connection to the centered curl and integer centering is derived in
    consolidated/CENTERED_BIOT_SAVART_BOUND.md. -/

namespace NSRegularity.NS.CenteredKernel8

abbrev Site := Fin 4 × Fin 4 × Fin 4

def coords : List (Fin 4) := [0, 1, 2, 3]
def sites : List Site :=
  coords.flatMap fun i => coords.flatMap fun j => coords.map fun k => (i,j,k)

private def dist (i : Fin 4) := min i.val (4-i.val)

/-- Numerator of the parity-grid Green function in the gauge G(0)=0. -/
def green (x : Site) : Int :=
  match dist x.1 + dist x.2.1 + dist x.2.2 with
  | 0 => 0
  | 1 => -315
  | 2 => -372
  | 3 => -393
  | 4 => -404
  | 5 => -411
  | _ => -416

def forward (axis : Fin 3) (x : Site) : Site :=
  if axis = 0 then (x.1+1, x.2.1, x.2.2)
  else if axis = 1 then (x.1, x.2.1+1, x.2.2)
  else (x.1, x.2.1, x.2.2+1)

def lap (x : Site) : Int :=
  let (i,j,k) := x
  6*green x - (green (i+1,j,k) + green (i-1,j,k) +
    green (i,j+1,k) + green (i,j-1,k) + green (i,j,k+1) + green (i,j,k-1))

theorem green_equation : ∀ i j k : Fin 4,
    lap (i,j,k) = (if i=0 ∧ j=0 ∧ k=0 then 1920 else 0) - 30 := by decide

def derivative (axis : Fin 3) (x : Site) := green (forward axis x) - green x

def budget {α : Type} (w : α → Int) : List α → Nat
  | [] => 0
  | x::xs => (w x).natAbs + budget w xs

def weighted {α : Type} (w f : α → Int) : List α → Int
  | [] => 0
  | x::xs => w x * f x + weighted w f xs

theorem derivative_budget : ∀ axis : Fin 3, budget (derivative axis) sites = 1920 := by decide

theorem weighted_bound {α : Type} (w f : α → Int) (m : Nat) (h : ∀ x, (f x).natAbs ≤ m)
    (xs : List α) : (weighted w f xs).natAbs ≤ budget w xs * m := by
  induction xs with
  | nil => simp [weighted, budget]
  | cons x xs ih =>
    simp only [weighted, budget, Nat.add_mul]
    exact Nat.le_trans (Int.natAbs_add_le _ _) (Nat.add_le_add
      (by rw [Int.natAbs_mul]; exact Nat.mul_le_mul_left _ (h x)) ih)

/-- A raw curl output is the difference of two derivative rows. The
    sampled inputs may be translates on their respective parity classes. -/
theorem curl_row_bound (a b : Fin 3) (f g : Site → Int) (m : Nat)
    (hf : ∀ x, (f x).natAbs ≤ m) (hg : ∀ x, (g x).natAbs ≤ m) :
    (weighted (derivative a) f sites - weighted (derivative b) g sites).natAbs ≤ 3840 * m := by
  have ha := weighted_bound (derivative a) f m hf sites
  have hb := weighted_bound (derivative b) g m hg sites
  rw [derivative_budget] at ha hb
  rw [Int.sub_eq_add_neg]
  have h := Int.natAbs_add_le (weighted (derivative a) f sites) (-weighted (derivative b) g sites)
  rw [Int.natAbs_neg] at h
  omega

/-- Translate an integer parity class into the signed window of radius m
    whenever its measured diameter is at most 2m. -/
theorem center_integer_class (lo hi z m : Int) (hl : lo ≤ z) (hh : z ≤ hi)
    (hd : hi-lo ≤ 2*m) :
    -m ≤ z-(lo+hi)/2 ∧ z-(lo+hi)/2 ≤ m := by omega

end NSRegularity.NS.CenteredKernel8
