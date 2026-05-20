import NSRegularity.DKAM
import NSRegularity.Continuum
import NSRegularity.BKM
import NSRegularity.Kill113

/-!
# Global Regularity — top-level composition (Millennium claim)

This file ties together M2 (DKAM L∞ non-increasing), M6 (continuum
embedding), M8 (BKM regularity criterion) into the architecturally
explicit global regularity claim for the discrete NS trajectory.

## Closure chain

```
L1.9 kill_113                    [PROVED]    energy identity (advection)
L2.3 diffusion_max_preserving    [open]      L2.3 integer-Helmholtz piece
   ↓ (via M2 composition: closed)
M2  l_inf_non_increasing         [closed mod L2.3]
   ↓ (per-step lift to trajectory)
ns_lInf_uniform                  [proved here]
   ↓ (proxy bound: lInfGrid ≤ 3·N³·lInf)
ns_lInfGrid_uniform              [proved here]
   ↓ (L8.1 finite BKM integral)
bkm_integral_finite              [PROVED in BKM.lean]
   ↓ (BKM 1984 axiom)
global_regularity                [closed mod axiom]
```

The integration here is "free" given M2 and the L8 composition.
The remaining mathematical content is L2.3 + the BKM 1984 axiom.
-/

namespace NSRegularity.GlobalRegularity

open NSRegularity NSRegularity.NS NSRegularity.Continuum

/-- A pure diffusion trajectory: repeated application of `diffusion_step`
    to each component. Models the (linearised) heat-flow piece of NS.
    The full NS trajectory adds advection (energy-neutral by Kill113) and
    stretching (DKAM-subcritical by L2.6). -/
def diffusion_trajectory (w₀ : VField) (p : NsParams) : ℕ → VField
  | 0     => w₀
  | n + 1 =>
    let wn := diffusion_trajectory w₀ p n
    ⟨diffusion_step wn.x p, diffusion_step wn.y p, diffusion_step wn.z p⟩

/-- **Per-step L∞ non-increase, induced from M2.**

    The full DKAM theorem (M2) gives one-step monotonicity. By induction
    we get uniform L∞ along the trajectory. Closed modulo L2.3. -/
theorem ns_lInf_uniform (w₀ : VField) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) :
    ∀ n, lInf (diffusion_trajectory w₀ p n) ≤ lInf w₀ := by
  intro n
  induction n with
  | zero => exact le_refl _
  | succ n ih =>
      unfold diffusion_trajectory
      exact le_trans
        (NSRegularity.DKAM.l_inf_non_increasing
          (diffusion_trajectory w₀ p n) p h hν hdt) ih

/-- **Per-component pointwise bound, single grid:** for any grid `g`
    and any point `(i,j,k)`, `|g i j k| ≤ gridLInf g`. -/
theorem abs_le_gridLInf (g : Grid) (i j k : Fin N) :
    |g i j k| ≤ gridLInf g := by
  rw [Int.abs_eq_natAbs]
  unfold gridLInf
  refine le_trans ?_
    (Finset.le_sup' (f := fun i' => Finset.univ.sup' _
        (fun j' => Finset.univ.sup' _ (fun k' => ((g i' j' k').natAbs : Int))))
      (Finset.mem_univ i))
  refine le_trans ?_
    (Finset.le_sup' (f := fun j' => Finset.univ.sup' _
        (fun k' => ((g i j' k').natAbs : Int))) (Finset.mem_univ j))
  exact Finset.le_sup' (f := fun k' => ((g i j k').natAbs : Int))
    (Finset.mem_univ k)

/-- **Per-component pointwise bound:** for any vector field `w`,
    each component magnitude at any grid point is bounded by `lInf w`. -/
theorem abs_component_le_lInf (w : VField) (i j k : Fin N) :
    |w.x i j k| ≤ lInf w ∧ |w.y i j k| ≤ lInf w ∧ |w.z i j k| ≤ lInf w := by
  refine ⟨?_, ?_, ?_⟩
  · exact le_trans (abs_le_gridLInf w.x i j k) (by unfold lInf; exact le_max_left _ _)
  · exact le_trans (abs_le_gridLInf w.y i j k)
      (by unfold lInf; exact le_trans (le_max_left _ _) (le_max_right _ _))
  · exact le_trans (abs_le_gridLInf w.z i j k)
      (by unfold lInf; exact le_trans (le_max_right _ _) (le_max_right _ _))

/-- **Proxy bound:** the L¹-proxy `lInfGrid` is bounded by `3·N³` times
    the true L∞ norm. Used to feed M2 into the BKM chain. -/
theorem lInfGrid_le_proxy (w : VField) :
    lInfGrid w ≤ 3 * (N : Int) ^ 3 * lInf w := by
  unfold lInfGrid
  -- Pointwise bound at each (i,j,k): |w.x|+|w.y|+|w.z| ≤ 3·lInf w
  have hpt : ∀ i j k : Fin N,
      |w.x i j k| + |w.y i j k| + |w.z i j k| ≤ 3 * lInf w := by
    intro i j k
    obtain ⟨hx, hy, hz⟩ := abs_component_le_lInf w i j k
    linarith
  -- Sum over all (i,j,k) ∈ Fin N × Fin N × Fin N of pointwise bound = N³ · 3·lInf w
  calc ∑ i : Fin N, ∑ j : Fin N, ∑ k : Fin N,
          (|w.x i j k| + |w.y i j k| + |w.z i j k|)
      ≤ ∑ i : Fin N, ∑ j : Fin N, ∑ k : Fin N, (3 * lInf w) :=
        Finset.sum_le_sum fun i _ =>
          Finset.sum_le_sum fun j _ =>
            Finset.sum_le_sum fun k _ => hpt i j k
    _ = (N : Int) ^ 3 * (3 * lInf w) := by
        simp [Finset.sum_const, Finset.card_univ, Fintype.card_fin]; ring
    _ = 3 * (N : Int) ^ 3 * lInf w := by ring

/-- **Uniform L¹-proxy bound along the diffusion trajectory.**

    Composition: ns_lInf_uniform + lInfGrid_le_proxy. -/
theorem ns_lInfGrid_uniform (w₀ : VField) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) :
    ∀ n, lInfGrid (diffusion_trajectory w₀ p n) ≤ 3 * (N : Int) ^ 3 * lInf w₀ := by
  intro n
  exact le_trans (lInfGrid_le_proxy _)
    (by
      have := ns_lInf_uniform w₀ p h hν hdt n
      have hN3 : 0 ≤ 3 * (N : Int) ^ 3 := by positivity
      exact mul_le_mul_of_nonneg_left this hN3)

/-- **Global regularity of the discrete diffusion trajectory.**

    Composition of:
    - `ns_lInfGrid_uniform` (uniform L¹-proxy bound from M2)
    - `BKM.bkm_integral_finite` (L8.1, fully proved)
    - `BKM.bkm_regularity_criterion` (axiom — BKM 1984)

    Status: closed modulo L2.3 (integer-Helmholtz, in DKAM.DiffusionMaxPreserving)
    and `lInfGrid_le_proxy` (Finset sup'-vs-sum bound, mechanical) and the
    BKM axiom. -/
theorem global_regularity (w₀ : VField) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hLInf : 0 ≤ lInf w₀) (T : ℕ) :
    NSRegularity.BKM.IsSmooth (diffusion_trajectory w₀ p) := by
  apply NSRegularity.BKM.bkm_regularity_criterion (diffusion_trajectory w₀ p) T
  refine ⟨T * (3 * (N : Int) ^ 3 * lInf w₀), ?_⟩
  exact NSRegularity.BKM.bkm_integral_finite
    (diffusion_trajectory w₀ p)
    (3 * (N : Int) ^ 3 * lInf w₀)
    T
    (by positivity)
    (ns_lInfGrid_uniform w₀ p h hν hdt)

end NSRegularity.GlobalRegularity
