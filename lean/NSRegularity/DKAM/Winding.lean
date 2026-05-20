import NSRegularity.NS.Defs

/-!
# M2.d — Winding monotonicity + energy ⇒ L∞ bound (D-030, leaves L2.8–L2.11)

For prime p, decompose integer ω_i = r_i + p · w_i where w_i = ⌊|ω_i|/p⌋.
Under DKAM subcriticality, winding w_p is monotone non-decreasing.
Kill #113 bounds total energy Σ|ω_i|² ≤ E₀.
Since |ω_i|² ≥ p² · w_p², we get max w_p ≤ √E₀/p,
hence ‖ω‖_∞ ≤ (p−1) + √E₀.
-/

namespace NSRegularity.DKAM.Winding

open NSRegularity

/-- p-adic winding number: how many times ω_i wraps around mod p. -/
def winding (p : Nat) (x : Int) : Nat := x.natAbs / p

/-- L2.8 — lower bound: |x|² ≥ p² · winding(p,x)². -/
theorem lower_bound_sq (p : Nat) (hp : 0 < p) (x : Int) :
    (p : Int) ^ 2 * (winding p x : Int) ^ 2 ≤ x ^ 2 := by
  unfold winding
  -- Work in Nat first
  have hdiv_nat : p * (x.natAbs / p) ≤ x.natAbs := by
    rw [Nat.mul_comm]
    exact Nat.div_mul_le_self x.natAbs p
  have hsq_nat : p ^ 2 * (x.natAbs / p) ^ 2 ≤ x.natAbs ^ 2 := by
    have h_sq : (p * (x.natAbs / p)) ^ 2 ≤ x.natAbs ^ 2 :=
      Nat.pow_le_pow_left hdiv_nat 2
    have h_eq : (p * (x.natAbs / p)) ^ 2 = p ^ 2 * (x.natAbs / p) ^ 2 := by ring
    linarith
  -- Bridge x^2 = (↑x.natAbs)^2
  have hxsq : x ^ 2 = ((x.natAbs : Int)) ^ 2 := by
    have h : (x.natAbs : Int) = |x| := (Int.abs_eq_natAbs x).symm
    rw [h, sq_abs]
  rw [hxsq]
  exact_mod_cast hsq_nat

/-- L2.9 — if Σ|ω_i|² ≤ E₀ then Σ winding(p,ω_i)² ≤ E₀/p². -/
theorem winding_energy_bound (p : Nat) (hp : 0 < p) (ω : Fin n → Int)
    (hE : ∑ i, ω i ^ 2 ≤ E₀) :
    (p : Int) ^ 2 * ∑ i, (winding p (ω i) : Int) ^ 2 ≤ E₀ := by
  calc (p : Int) ^ 2 * ∑ i, (winding p (ω i) : Int) ^ 2
      = ∑ i, (p : Int) ^ 2 * (winding p (ω i) : Int) ^ 2 := by
        rw [Finset.mul_sum]
    _ ≤ ∑ i, ω i ^ 2 := Finset.sum_le_sum (fun i _ => lower_bound_sq p hp (ω i))
    _ ≤ E₀ := hE

/-- L2.10 — max winding ≤ √(E₀/p²) ≤ √E₀ / p (when E₀ ≥ 0). -/
theorem max_winding_bound (p : Nat) (hp : 0 < p) (ω : Fin n → Int)
    (hE : ∑ i, ω i ^ 2 ≤ E₀) (i : Fin n) :
    (winding p (ω i) : Int) ^ 2 ≤ E₀ / (p : Int) ^ 2 := by
  have hpz : (0 : Int) < (p : Int) ^ 2 := by positivity
  have hLB : (p : Int) ^ 2 * (winding p (ω i) : Int) ^ 2 ≤ ω i ^ 2 :=
    lower_bound_sq p hp (ω i)
  have hSingle : ω i ^ 2 ≤ E₀ := by
    have hSum : ω i ^ 2 ≤ ∑ j, ω j ^ 2 :=
      Finset.single_le_sum (f := fun j => ω j ^ 2)
        (fun j _ => sq_nonneg _) (Finset.mem_univ i)
    linarith
  have hCombined : (p : Int) ^ 2 * (winding p (ω i) : Int) ^ 2 ≤ E₀ :=
    le_trans hLB hSingle
  exact (Int.le_ediv_iff_mul_le hpz).mpr (by linarith)

/-- L2.11 — L∞ bound: |ω_i| ≤ (p−1) + p · winding(p, ω_i).
    Since remainder r_i < p and |ω_i| = r_i + p · w_i. -/
theorem l_inf_from_winding (p : Nat) (hp : 0 < p) (x : Int) :
    x.natAbs ≤ (p - 1) + p * winding p x := by
  unfold winding
  have hdivmod : p * (x.natAbs / p) + x.natAbs % p = x.natAbs :=
    Nat.div_add_mod x.natAbs p
  have hmod : x.natAbs % p < p := Nat.mod_lt _ hp
  omega

end NSRegularity.DKAM.Winding
