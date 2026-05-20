import NSRegularity.NS.Defs

/-!
# M2.a — Diffusion is max-preserving under CFL (leaves L2.1–L2.3)

The 7-point Laplacian stencil with CFL `6νΔt ≤ N²S²` produces non-negative
weights summing to 1. The diffusion step is therefore a convex combination:
  `diffusion_step f p (i,j,k) ≤ gridLInf f` for all grid points.
-/

namespace NSRegularity.DKAM.Diffusion

open NSRegularity NSRegularity.NS

/-- L2.1 — Under CFL, the stencil self-weight `1 - 6c` is non-negative. -/
theorem stencil_self_nonneg (p : NsParams) (h : cfl_ok p) :
    0 ≤ (N : Int) ^ 2 * p.ns_scale ^ 2 - 6 * p.nu_scaled * p.dt_latt := by
  unfold cfl_ok at h
  linarith

/-- L2.2 — Under CFL, the neighbor weight `c = νΔt/(N²S²)` is non-negative
    (given ν, Δt ≥ 0). -/
theorem stencil_neighbor_nonneg (p : NsParams) (hν : 0 ≤ p.nu_scaled)
    (hdt : 0 ≤ p.dt_latt) :
    0 ≤ p.nu_scaled * p.dt_latt := by
  exact mul_nonneg hν hdt

/-- **Auxiliary L2.3* — Diffusion step under EXACT division is max-preserving.**

    Multiplying through by `N²S²`, the discrete update at each grid point
    satisfies
      `N²S² · (f + Δf-correction) = (N²S² − 6νΔt) · f + νΔt · Σ neighbors`.
    The RHS is a convex combination with nonneg integer weights summing to
    `N²S²` of seven values each bounded by `M := gridLInf f`. Hence
    `|N²S² · (...)| ≤ N²S² · M`, so `|...| ≤ M`.

    This is the algebraic content of L2.3. The unconditional statement
    requires handling `Int.ediv` rounding (the integer-Helmholtz obstruction),
    which is the open piece below. -/
theorem diffusion_max_preserving_algebraic
    (f : Grid) (p : NsParams) (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled)
    (hdt : 0 ≤ p.dt_latt) (i j k : Fin N) :
    -- Multiply through: N²S² · update = (N²S² − 6νΔt) · f + νΔt · Σ neighbors
    let M := (N : Int) ^ 2 * p.ns_scale ^ 2
    let c := p.nu_scaled * p.dt_latt
    (M * (f i j k) + c * laplacian f i j k)
    = (M - 6 * c) * f i j k
      + c * (f (sp i) j k + f (sn i) j k
           + f i (sp j) k + f i (sn j) k
           + f i j (sp k) + f i j (sn k)) := by
  simp only [laplacian]
  ring

/-- **Helper:** for any grid point, `|f i j k| ≤ gridLInf f` (as Int).
    Lifts the per-point natAbs bound to the Int-valued comparison. -/
theorem abs_le_gridLInf (f : Grid) (i j k : Fin N) :
    |f i j k| ≤ gridLInf f := by
  rw [Int.abs_eq_natAbs]
  unfold gridLInf
  refine le_trans ?_
    (Finset.le_sup' (f := fun i' => Finset.univ.sup' _
        (fun j' => Finset.univ.sup' _ (fun k' => ((f i' j' k').natAbs : Int))))
      (Finset.mem_univ i))
  refine le_trans ?_
    (Finset.le_sup' (f := fun j' => Finset.univ.sup' _
        (fun k' => ((f i j' k').natAbs : Int))) (Finset.mem_univ j))
  exact Finset.le_sup' (f := fun k' => ((f i j k').natAbs : Int))
    (Finset.mem_univ k)

/-- **Resolution A — L2.3 under exact division.**

    Assuming the diffusion correction divides cleanly (no `Int.ediv` rounding
    at the grid point in question), the convex-combination argument yields
    the strict max-preserving bound `|diffusion_step f p (i,j,k)| ≤ M`.

    Proof: under exactness, `diffusion_step f p i j k = f i j k + q` for an
    integer `q` with `q · N²S² = νΔt · Δf`. Multiply through:
       `N²S² · (f + q) = (N²S² − 6νΔt) · f + νΔt · Σ neighbors`.
    The RHS is a nonneg-integer linear combination (weights `N²S² − 6νΔt ≥ 0`
    by CFL, and `νΔt ≥ 0`) summing to `N²S²`, of seven values each `≤ M`.
    Hence `N²S² · |f + q| ≤ N²S² · M`, so `|f + q| ≤ M`. -/
theorem diffusion_max_preserving_exact
    (f : Grid) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i j k : Fin N,
        ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
          (p.nu_scaled * p.dt_latt * laplacian f i j k))
    (i j k : Fin N) :
    |diffusion_step f p i j k| ≤ gridLInf f := by
  set M := gridLInf f with hM
  set NS2 : Int := (N : Int) ^ 2 * p.ns_scale ^ 2 with hNS2_def
  set cv : Int := p.nu_scaled * p.dt_latt with hcv_def
  -- positivity
  have hNS2_pos : 0 < NS2 := by
    have : (0 : Int) < (N : Int) := by decide
    positivity
  have hNS2_nonneg : 0 ≤ NS2 := le_of_lt hNS2_pos
  have hcv_nonneg : 0 ≤ cv := mul_nonneg hν hdt
  -- L2.1: NS2 - 6 cv ≥ 0
  have hSelf_nonneg : 0 ≤ NS2 - 6 * cv := by
    have := stencil_self_nonneg p h
    -- this is `0 ≤ NS2 - 6 * p.nu_scaled * p.dt_latt`
    -- but 6 * cv = 6 * p.nu_scaled * p.dt_latt by ring
    have heq : (6 : Int) * cv = 6 * p.nu_scaled * p.dt_latt := by
      rw [hcv_def]; ring
    linarith
  -- exact quotient at (i,j,k)
  obtain ⟨q, hq⟩ := hExact i j k
  -- hq : cv * laplacian f i j k = NS2 * q
  -- Show diffusion_step = f i j k + q under exactness
  have hdiff_eq : diffusion_step f p i j k = f i j k + q := by
    unfold diffusion_step
    -- f + cv * Δf / NS2 = f + q (since cv * Δf = NS2 * q)
    have hrw : p.nu_scaled * p.dt_latt * laplacian f i j k = NS2 * q := hq
    rw [hrw]
    rw [Int.mul_ediv_cancel_left _ (ne_of_gt hNS2_pos)]
  rw [hdiff_eq]
  -- Key algebraic identity: NS2 * (f + q) = (NS2 - 6 cv) * f + cv * Σ neighbors
  have hKey : NS2 * (f i j k + q)
      = (NS2 - 6 * cv) * f i j k
        + cv * (f (sp i) j k + f (sn i) j k
              + f i (sp j) k + f i (sn j) k
              + f i j (sp k) + f i j (sn k)) := by
    have hExpand : NS2 * (f i j k + q) = NS2 * f i j k + NS2 * q := by ring
    rw [hExpand, ← hq]
    simp only [laplacian]
    ring
  -- Bound: |NS2 * (f + q)| ≤ NS2 * M via convex combination
  have hBound : |NS2 * (f i j k + q)| ≤ NS2 * M := by
    rw [hKey]
    -- |(NS2 - 6cv) * f + cv * (n1+...+n6)|
    -- ≤ (NS2 - 6cv) * |f| + cv * (|n1|+...+|n6|)
    -- ≤ (NS2 - 6cv) * M + 6 cv * M  = NS2 * M
    have hSum_bound :
        |(NS2 - 6 * cv) * f i j k + cv * (f (sp i) j k + f (sn i) j k
              + f i (sp j) k + f i (sn j) k
              + f i j (sp k) + f i j (sn k))|
        ≤ (NS2 - 6 * cv) * M + 6 * cv * M := by
      have h1 : |f i j k| ≤ M := abs_le_gridLInf f i j k
      have h2 : |f (sp i) j k| ≤ M := abs_le_gridLInf f _ _ _
      have h3 : |f (sn i) j k| ≤ M := abs_le_gridLInf f _ _ _
      have h4 : |f i (sp j) k| ≤ M := abs_le_gridLInf f _ _ _
      have h5 : |f i (sn j) k| ≤ M := abs_le_gridLInf f _ _ _
      have h6 : |f i j (sp k)| ≤ M := abs_le_gridLInf f _ _ _
      have h7 : |f i j (sn k)| ≤ M := abs_le_gridLInf f _ _ _
      -- Triangle inequality on the sum
      have hT1 : |(NS2 - 6 * cv) * f i j k| ≤ (NS2 - 6 * cv) * M := by
        rw [abs_mul, abs_of_nonneg hSelf_nonneg]
        exact mul_le_mul_of_nonneg_left h1 hSelf_nonneg
      have hT2 : |cv * (f (sp i) j k + f (sn i) j k
                       + f i (sp j) k + f i (sn j) k
                       + f i j (sp k) + f i j (sn k))| ≤ 6 * cv * M := by
        rw [abs_mul, abs_of_nonneg hcv_nonneg]
        have hnbrs : |f (sp i) j k + f (sn i) j k
                       + f i (sp j) k + f i (sn j) k
                       + f i j (sp k) + f i j (sn k)| ≤ 6 * M :=
          calc |f (sp i) j k + f (sn i) j k
                       + f i (sp j) k + f i (sn j) k
                       + f i j (sp k) + f i j (sn k)|
              ≤ |f (sp i) j k| + |f (sn i) j k|
                + |f i (sp j) k| + |f i (sn j) k|
                + |f i j (sp k)| + |f i j (sn k)| := by
                  refine le_trans (abs_add_le _ _) ?_
                  refine add_le_add ?_ (le_refl _)
                  refine le_trans (abs_add_le _ _) ?_
                  refine add_le_add ?_ (le_refl _)
                  refine le_trans (abs_add_le _ _) ?_
                  refine add_le_add ?_ (le_refl _)
                  refine le_trans (abs_add_le _ _) ?_
                  refine add_le_add ?_ (le_refl _)
                  exact abs_add_le _ _
            _ ≤ M + M + M + M + M + M := by linarith
            _ = 6 * M := by ring
        calc cv * |f (sp i) j k + f (sn i) j k
                       + f i (sp j) k + f i (sn j) k
                       + f i j (sp k) + f i j (sn k)|
            ≤ cv * (6 * M) := mul_le_mul_of_nonneg_left hnbrs hcv_nonneg
          _ = 6 * cv * M := by ring
      calc |(NS2 - 6 * cv) * f i j k
              + cv * (f (sp i) j k + f (sn i) j k
                    + f i (sp j) k + f i (sn j) k
                    + f i j (sp k) + f i j (sn k))|
          ≤ |(NS2 - 6 * cv) * f i j k|
            + |cv * (f (sp i) j k + f (sn i) j k
                   + f i (sp j) k + f i (sn j) k
                   + f i j (sp k) + f i j (sn k))| := abs_add_le _ _
        _ ≤ (NS2 - 6 * cv) * M + 6 * cv * M := add_le_add hT1 hT2
    -- Final: (NS2 - 6 cv) M + 6 cv M = NS2 M
    have hCombine : (NS2 - 6 * cv) * M + 6 * cv * M = NS2 * M := by ring
    linarith
  -- Divide by NS2 to get |f + q| ≤ M
  have hM_nonneg : 0 ≤ M := by
    rw [hM]
    unfold gridLInf
    exact_mod_cast Nat.zero_le _
  have hAbs : NS2 * |f i j k + q| ≤ NS2 * M := by
    have := hBound
    rwa [abs_mul, abs_of_nonneg hNS2_nonneg] at this
  exact le_of_mul_le_mul_left hAbs hNS2_pos

/-- **Resolution A (natAbs form) — L2.3 under exact division.**

    Natural-number version of `diffusion_max_preserving_exact`. -/
theorem diffusion_max_preserving_exact_natAbs
    (f : Grid) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt)
    (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i j k : Fin N,
        ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
          (p.nu_scaled * p.dt_latt * laplacian f i j k))
    (i j k : Fin N) :
    (diffusion_step f p i j k).natAbs ≤ gridLInf f := by
  have habs := diffusion_max_preserving_exact f p h hν hdt hns_pos hExact i j k
  rw [Int.abs_eq_natAbs] at habs
  exact_mod_cast habs

/-- L2.3 — The diffusion step is max-preserving: the updated value at
    any grid point is bounded by the original L∞ norm.

    **Status (D-030 / integer-Helmholtz obstruction).** The algebraic
    convex-combination argument is captured in `diffusion_max_preserving_algebraic`
    above. Under the exact-division hypothesis (`Resolution A`), the bound
    holds — see `diffusion_max_preserving_exact{_natAbs}`. The unconditional
    statement below remains open; the three resolutions tracked are:
      A. **exact-division hypothesis** `(N²S²) ∣ νΔt · Δf`  — proved (above)
      B. relax the bound to `≤ gridLInf f + 1` (off-by-one rounding)
      C. redefine `diffusion_step` via `ExactRational`/CRT (matches Rust)

    Composition `M2 = l_inf_non_increasing` reduces cleanly to L2.3
    (see DKAM.lean). -/
theorem diffusion_max_preserving (f : Grid) (p : NsParams)
    (h : cfl_ok p) (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) :
    ∀ i j k : Fin N,
      (diffusion_step f p i j k).natAbs ≤ gridLInf f := by
  sorry

end NSRegularity.DKAM.Diffusion
