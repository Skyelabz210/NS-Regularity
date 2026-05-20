import NSRegularity.Kill113
import NSRegularity.DKAM.Winding

/-!
# M9 — Three-Stream Structural Reinforcement (my macro)

**Proof step 9.** p-adic non-expansion (from ultrametric inequality):
  v_p(ω_i^{n+1}) ≥ v_p(ω_i^n)

Combined with M1 (energy bounded) and the integer product formula:
  |ω_i| = smooth_i × coprime_i

gives: coprime part bounded in L² unconditionally.

**Status:** `pending_decomposition`. Sub-leaves:
- L9.1: Define p-adic valuation v_p
- L9.2: Ultrametric non-expansion: v_p(F(x)) ≥ v_p(x) for poly F with F(0)=0
- L9.3: Product formula decomposition |ω| = smooth × coprime
- L9.4: Coprime bounded in L²: Σ coprime² ≤ E(0)/min(smooth)²
-/

namespace NSRegularity.ThreeStream

open NSRegularity

/-- p-adic valuation: largest power of p dividing x (0 for x=0). -/
def pValuation (p : Nat) (x : Int) : Nat :=
  if x = 0 then 0
  else (List.range (x.natAbs + 1)).countP (fun k => (p ^ k : Int) ∣ x)

/-- Smooth part: ∏ p^{v_p(x)} for a list of primes. -/
def smoothPart (primes : List Nat) (x : Int) : Int :=
  primes.foldl (fun (acc : Int) (p : Nat) =>
    acc * (p : Int) ^ pValuation p x) 1

/-- Coprime part: x / smoothPart. -/
def coprimePart (primes : List Nat) (x : Int) : Int :=
  if smoothPart primes x = 0 then x else x / smoothPart primes x

/-- **L9.2 — Ultrametric non-expansion.** For polynomial F with F(0)=0 and
    integer coefficients, |F(x)|_p ≤ |x|_p, equivalently v_p(F(x)) ≥ v_p(x).

    **Statement flagged for operator review:** the hypothesis as written —
    `F(x) = 0 → x = 0` (a quasi-injectivity statement) — is the wrong shape
    for the ultrametric non-expansion. The correct hypothesis is `F(0) = 0`
    (constant term vanishes), so that `F(x) = x · G(x)` for some integer-
    coefficient polynomial G. With that hypothesis, v_p(F(x)) ≥ v_p(x)
    follows from p-adic multiplicativity. See `ultrametric_nonexpansion_intended`
    below for the corrected statement form. -/
theorem ultrametric_nonexpansion (p : Nat) (hp : Nat.Prime p) (x : Int)
    (c : List Int) (hF0 : (c.foldl (fun acc ci => acc * x + ci) 0) = 0 → x = 0) :
    pValuation p (c.foldl (fun acc ci => acc * x + ci) 0) ≥ pValuation p x := by
  sorry

/-- L9.2' (corrected intent) — Ultrametric non-expansion with the proper
    hypothesis `F(0) = 0`. Under the Horner-form foldl encoding,
    `F(0) = c.getLast? .getD 0` (the last list entry, defaulting to 0 for
    an empty list).

    Proof outline (deferred):
    1. F(0) = 0 ↔ `c = []` or `c.getLast? = some 0`.
    2. From step 1, F(x) = x · G(x) where G is the polynomial whose
       coefficient list is `c.dropLast`.
    3. p-adic multiplicativity: v_p(x · G(x)) = v_p(x) + v_p(G(x)) ≥ v_p(x). -/
theorem ultrametric_nonexpansion_intended (p : Nat) (hp : Nat.Prime p) (x : Int)
    (c : List Int) (hF0 : c.foldl (fun acc ci => acc * 0 + ci) 0 = 0) :
    pValuation p (c.foldl (fun acc ci => acc * x + ci) 0) ≥ pValuation p x := by
  sorry

/-- **L9.4 — Coprime bounded in L².**
    The coprime part `ω / s` has |ω/s| ≤ |ω| when s ≥ 1 (integer division),
    so its L² norm is bounded by ‖ω‖_{L²} ≤ E₀. -/
theorem coprime_bounded_l2 (primes : List Nat) (ω : Fin n → Int)
    (hE : ∑ i, ω i ^ 2 ≤ E₀)
    (hSmooth : ∀ i, 0 < smoothPart primes (ω i)) :
    ∑ i, coprimePart primes (ω i) ^ 2 ≤ E₀ := by
  have hPer : ∀ i, coprimePart primes (ω i) ^ 2 ≤ ω i ^ 2 := by
    intro i
    unfold coprimePart
    have hs : smoothPart primes (ω i) ≠ 0 := ne_of_gt (hSmooth i)
    rw [if_neg hs]
    have habs : |ω i / smoothPart primes (ω i)| ≤ |ω i| :=
      Int.abs_ediv_le_abs _ _
    have hsq : |ω i / smoothPart primes (ω i)| ^ 2 ≤ |ω i| ^ 2 :=
      pow_le_pow_left₀ (abs_nonneg _) habs 2
    rwa [sq_abs, sq_abs] at hsq
  calc ∑ i, coprimePart primes (ω i) ^ 2
      ≤ ∑ i, ω i ^ 2 := Finset.sum_le_sum (fun i _ => hPer i)
    _ ≤ E₀ := hE

end NSRegularity.ThreeStream
