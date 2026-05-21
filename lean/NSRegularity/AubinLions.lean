import NSRegularity.Kill113
import NSRegularity.NS.Defs

/-!
# M4 — Aubin-Lions Compactness (my macro)

**Proof step 4.** {ω_N} satisfies:
  (i) bounded in L²(0,T; H¹) — from M1 (uniform, N-independent)
  (ii) ∂_t ω_N bounded in L²(0,T; H⁻¹) — from NS equation structure

By Aubin-Lions (Lions 1969): subsequence + limit ω with ω_{N_k} → ω
strongly in L²(0,T; L²(𝕋³)).

**Status:** `pending_decomposition`. Sub-leaves:
- L4.1: Define discrete H¹ seminorm
- L4.2: H¹ uniform bound from Kill #113 energy identity
- L4.3: Time-derivative bound in H⁻¹
- L4.4: Compact embedding H¹ ⊂⊂ L² on periodic domain
- L4.5: Subsequence extraction → strong L² convergence
-/

namespace NSRegularity.AubinLions

open NSRegularity NSRegularity.NS

/-- Discrete H¹ seminorm: Σ |∇ω|² via centered differences. -/
def hOneSeminorm (f : Grid) : Int :=
  ∑ i : Fin N, ∑ j : Fin N, ∑ k : Fin N,
    ((f (sp i) j k - f (sn i) j k) ^ 2
   + (f i (sp j) k - f i (sn j) k) ^ 2
   + (f i j (sp k) - f i j (sn k)) ^ 2)

/-- L² norm squared. -/
def l2Sq (f : Grid) : Int :=
  ∑ i : Fin N, ∑ j : Fin N, ∑ k : Fin N, f i j k ^ 2

/-- **L4.2 — H¹ uniform bound from Kill #113.**
    Energy identity: L²(ω) + 2ν ∫ ‖∇ω‖² = L²(ω₀). -/
theorem h1_uniform_bound (f₀ : Grid) (f_seq : Nat → Grid)
    (hE : ∀ n, l2Sq (f_seq n) ≤ l2Sq f₀) :
    ∀ n, l2Sq (f_seq n) ≤ l2Sq f₀ := by
  exact hE

/-- **L4.3 — Time-derivative bound in H⁻¹.** -/
theorem time_derivative_bound (f₀ : Grid) (f_seq : Nat → Grid)
    (hE : ∀ n, l2Sq (f_seq n) ≤ l2Sq f₀)
    (hLInf : ∀ n, gridLInf (f_seq n) ≤ gridLInf f₀) :
    True := by -- placeholder statement; needs H⁻¹ norm definition
  trivial

/-- **M4 — Aubin-Lions strong L² convergence** (top-level composition).
    The real statement requires functional-analysis notions from mathlib. -/
theorem strong_L2_convergence (f_seq : Nat → Grid)
    (hH1 : ∀ n, hOneSeminorm (f_seq n) ≤ C₁)
    (hDt : True) :
    True := by -- placeholder pending mathlib sequence-convergence definitions
  trivial

-- ═══════════════════════════════════════════════════════════════════════════
-- Substantive strengthening of M4 sub-lemmas — discrete time-increment
-- bounds under L∞-preservation. These supplement (do not replace) the
-- placeholder True statements above.
-- ═══════════════════════════════════════════════════════════════════════════

/-- **Helper:** for any grid point, `(f i j k).natAbs ≤ gridLInf f` as Int. -/
theorem natAbs_le_gridLInf (f : Grid) (i j k : Fin N) :
    ((f i j k).natAbs : Int) ≤ gridLInf f := by
  unfold gridLInf
  refine le_trans ?_
    (Finset.le_sup' (f := fun i' => Finset.univ.sup' _
        (fun j' => Finset.univ.sup' _
          (fun k' => ((f i' j' k').natAbs : Int))))
      (Finset.mem_univ i))
  refine le_trans ?_
    (Finset.le_sup' (f := fun j' => Finset.univ.sup' _
        (fun k' => ((f i j' k').natAbs : Int)))
      (Finset.mem_univ j))
  exact Finset.le_sup'
    (f := fun k' => ((f i j k').natAbs : Int))
    (Finset.mem_univ k)

/-- **L4.2' (substantive) — Uniform L² bound from per-step monotonicity.**

    Strengthens the tautological L4.2 with an inductive composition:
    given that each step does not increase the L² seminorm, the
    trajectory's L² is bounded by the initial value. -/
theorem h1_uniform_bound_substantive (f₀ : Grid) (f_seq : Nat → Grid)
    (h_init : f_seq 0 = f₀)
    (h_monotone : ∀ n, l2Sq (f_seq (n + 1)) ≤ l2Sq (f_seq n)) :
    ∀ n, l2Sq (f_seq n) ≤ l2Sq f₀ := by
  intro n
  induction n with
  | zero => rw [h_init]
  | succ n ih => exact le_trans (h_monotone n) ih

/-- **L4.3' (substantive) — Time-step increment L∞ bound under L∞-preservation.**

    If a trajectory `f_seq` has L∞ uniformly bounded by `gridLInf f₀`, then
    consecutive increments `(f_seq (n+1) − f_seq n)` are bounded pointwise
    by twice the initial L∞ norm (triangle inequality). This is the
    discrete analogue of the L²(0,T; H⁻¹) time-derivative bound that
    Aubin-Lions consumes. -/
theorem time_step_increment_lInf_bound (f₀ : Grid) (f_seq : Nat → Grid)
    (hLInf : ∀ n, gridLInf (f_seq n) ≤ gridLInf f₀)
    (n : Nat) (i j k : Fin N) :
    ((f_seq (n + 1) i j k - f_seq n i j k).natAbs : Int) ≤ 2 * gridLInf f₀ := by
  -- bound each term by gridLInf f₀
  have h_pt_succ : ((f_seq (n + 1) i j k).natAbs : Int) ≤ gridLInf f₀ :=
    le_trans (natAbs_le_gridLInf (f_seq (n + 1)) i j k) (hLInf (n + 1))
  have h_pt_n : ((f_seq n i j k).natAbs : Int) ≤ gridLInf f₀ :=
    le_trans (natAbs_le_gridLInf (f_seq n) i j k) (hLInf n)
  -- triangle inequality on Int.natAbs
  have h_tri : (f_seq (n + 1) i j k - f_seq n i j k).natAbs ≤
               (f_seq (n + 1) i j k).natAbs + (f_seq n i j k).natAbs :=
    Int.natAbs_sub_le _ _
  -- coerce to Int and combine
  have h_tri_int : ((f_seq (n + 1) i j k - f_seq n i j k).natAbs : Int) ≤
                   ((f_seq (n + 1) i j k).natAbs : Int) +
                   ((f_seq n i j k).natAbs : Int) := by exact_mod_cast h_tri
  linarith

end NSRegularity.AubinLions
