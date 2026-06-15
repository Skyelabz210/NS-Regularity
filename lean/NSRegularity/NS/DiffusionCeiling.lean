import NSRegularity.NS.StretchingBound

/-!
# NS/DiffusionCeiling — Attack 3: quantitative diffusion ceiling (Piece B₁)

Resolution A gives the blanket bound `gridLInf (diffusion_step f p) ≤ gridLInf f`.
Piece B of the small-data closure needs more: a ceiling `D_max` with *spare
capacity* `gridLInf f − D_max > 0` to absorb the advection/stretching pressure.

This file proves the sharp local form. Under the Resolution-A exactness
hypothesis, one diffusion step satisfies the **exact stencil identity**

```
N²S² · D(x) = (N²S² − 6cv) · f(x) + cv · Σ_{6 neighbors} f
```

(`cv := ν·Δt`), from which the **local ceiling**

```
N²S² · |D(x)| ≤ (N²S² − 6cv) · |f(x)| + 6cv · G        (G := gridLInf f)
```

and the **spare-capacity gap formula**

```
N²S² · (G − |D(x)|) ≥ (N²S² − 6cv) · (G − |f(x)|)
```

follow. The gap is proportional to the distance of `f(x)` from the max.

## Structural finding (honest)

At points realising the max (`|f(x)| = G`) the pure-diffusion gap is **zero**
— constant fields show no strict uniform `D_max < G` is provable without
data hypotheses. Therefore the L∞ gate cannot be closed by blanket CFL
alone at max points; there the closure must come from the *sign structure*
of advection/stretching at extrema (discrete maximum principle route).
The two one-sided lemmas at the end are that route's first bricks:
diffusion strictly respects local maxima/minima (`Δf ≤ 0 → D ≤ f`,
`Δf ≥ 0 → D ≥ f`).
-/

namespace NSRegularity.NS.DiffusionCeiling

open NSRegularity NSRegularity.NS NSRegularity.NS.AdvectionBound

/-- Six-term triangle inequality. -/
theorem abs_add_six (a b c d e g : Int) :
    |a + b + c + d + e + g| ≤ |a| + |b| + |c| + |d| + |e| + |g| := by
  have h1 := abs_add_le (a + b + c + d + e) g
  have h2 := abs_add_le (a + b + c + d) e
  have h3 := abs_add_le (a + b + c) d
  have h4 := abs_add_le (a + b) c
  have h5 := abs_add_le a b
  linarith

/-- **Exact stencil identity.** Under Resolution-A exactness at the point,
    one diffusion step is the convex (integer) stencil combination

    `N²S² · D = (N²S² − 6cv) · f(x) + cv · Σ_{6 nbrs} f`. -/
theorem diffusion_step_stencil_exact (f : Grid) (p : NsParams)
    (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i' j' k' : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * laplacian f i' j' k'))
    (i j k : Fin N) :
    ((N : Int) ^ 2 * p.ns_scale ^ 2) * diffusion_step f p i j k
      = ((N : Int) ^ 2 * p.ns_scale ^ 2 - 6 * (p.nu_scaled * p.dt_latt)) * f i j k
        + (p.nu_scaled * p.dt_latt) *
            (f (sp i) j k + f (sn i) j k + f i (sp j) k + f i (sn j) k
             + f i j (sp k) + f i j (sn k)) := by
  set NS2 : Int := (N : Int) ^ 2 * p.ns_scale ^ 2 with hNS2_def
  set cv : Int := p.nu_scaled * p.dt_latt with hcv_def
  have hNS2_pos : 0 < NS2 := by positivity
  -- Exact division: NS2 · (cv·Δf / NS2) = cv·Δf
  have hq_exact : NS2 * (cv * laplacian f i j k / NS2) = cv * laplacian f i j k := by
    obtain ⟨n, hn⟩ := hExact i j k
    rw [hn, Int.mul_ediv_cancel_left _ (ne_of_gt hNS2_pos)]
  have hD : diffusion_step f p i j k = f i j k + cv * laplacian f i j k / NS2 := rfl
  have hkey : NS2 * diffusion_step f p i j k = NS2 * f i j k + cv * laplacian f i j k := by
    rw [hD, mul_add, hq_exact]
  rw [hkey]
  unfold laplacian
  ring

/-- **Local diffusion ceiling.** The post-diffusion value is bounded by the
    CFL-weighted mix of the local value and the global max:

    `N²S² · |D(x)| ≤ (N²S² − 6cv) · |f(x)| + 6cv · G`. -/
theorem diffusion_step_local_ceiling (f : Grid) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i' j' k' : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * laplacian f i' j' k'))
    (i j k : Fin N) :
    ((N : Int) ^ 2 * p.ns_scale ^ 2) * |diffusion_step f p i j k|
      ≤ ((N : Int) ^ 2 * p.ns_scale ^ 2 - 6 * (p.nu_scaled * p.dt_latt)) * |f i j k|
        + 6 * (p.nu_scaled * p.dt_latt) * gridLInf f := by
  set NS2 : Int := (N : Int) ^ 2 * p.ns_scale ^ 2 with hNS2_def
  set cv : Int := p.nu_scaled * p.dt_latt with hcv_def
  have hNS2_pos : 0 < NS2 := by positivity
  have hcv_nonneg : 0 ≤ cv := mul_nonneg hν hdt
  have hCFL : 6 * cv ≤ NS2 := by
    unfold cfl_ok at h
    have : 6 * cv = 6 * p.nu_scaled * p.dt_latt := by rw [hcv_def]; ring
    linarith
  have hstencil := diffusion_step_stencil_exact f p hns_pos hExact i j k
  -- |NS2 · D| = NS2 · |D|
  have hlhs : |NS2 * diffusion_step f p i j k| = NS2 * |diffusion_step f p i j k| := by
    rw [abs_mul, abs_of_pos hNS2_pos]
  -- Triangle inequality on the stencil form
  have htri : |NS2 * diffusion_step f p i j k|
      ≤ |(NS2 - 6 * cv) * f i j k|
        + |cv * (f (sp i) j k + f (sn i) j k + f i (sp j) k + f i (sn j) k
                 + f i j (sp k) + f i j (sn k))| := by
    rw [hstencil]
    exact abs_add_le _ _
  -- First term: coefficient is nonneg under CFL
  have hterm1 : |(NS2 - 6 * cv) * f i j k| = (NS2 - 6 * cv) * |f i j k| := by
    rw [abs_mul, abs_of_nonneg (by linarith : (0 : Int) ≤ NS2 - 6 * cv)]
  -- Second term: neighbor sum bounded by 6·G
  have hnb : |f (sp i) j k + f (sn i) j k + f i (sp j) k + f i (sn j) k
              + f i j (sp k) + f i j (sn k)| ≤ 6 * gridLInf f := by
    have h6 := abs_add_six (f (sp i) j k) (f (sn i) j k) (f i (sp j) k)
      (f i (sn j) k) (f i j (sp k)) (f i j (sn k))
    have b1 := abs_le_gridLInf f (sp i) j k
    have b2 := abs_le_gridLInf f (sn i) j k
    have b3 := abs_le_gridLInf f i (sp j) k
    have b4 := abs_le_gridLInf f i (sn j) k
    have b5 := abs_le_gridLInf f i j (sp k)
    have b6 := abs_le_gridLInf f i j (sn k)
    linarith
  have hterm2 : |cv * (f (sp i) j k + f (sn i) j k + f i (sp j) k + f i (sn j) k
                 + f i j (sp k) + f i j (sn k))| ≤ cv * (6 * gridLInf f) := by
    rw [abs_mul, abs_of_nonneg hcv_nonneg]
    exact mul_le_mul_of_nonneg_left hnb hcv_nonneg
  have hcombined : NS2 * |diffusion_step f p i j k|
      ≤ (NS2 - 6 * cv) * |f i j k| + cv * (6 * gridLInf f) := by
    rw [← hlhs]
    calc |NS2 * diffusion_step f p i j k|
        ≤ |(NS2 - 6 * cv) * f i j k|
          + |cv * (f (sp i) j k + f (sn i) j k + f i (sp j) k + f i (sn j) k
                   + f i j (sp k) + f i j (sn k))| := htri
      _ ≤ (NS2 - 6 * cv) * |f i j k| + cv * (6 * gridLInf f) := by
          rw [hterm1]; linarith
  linarith [hcombined]

/-- **Spare-capacity gap formula.** The slack the CFL condition can spend
    is proportional to the distance of `f(x)` from the global max:

    `N²S² · (G − |D(x)|) ≥ (N²S² − 6cv) · (G − |f(x)|)`.

    At interior (non-max) points the diffusion step opens strict room for
    the advection/stretching pressure; at max points (`|f(x)| = G`) the
    pure-diffusion gap is zero and the closure must come from the sign
    structure of the transport terms at extrema. -/
theorem diffusion_gap (f : Grid) (p : NsParams) (h : cfl_ok p)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i' j' k' : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * laplacian f i' j' k'))
    (i j k : Fin N) :
    ((N : Int) ^ 2 * p.ns_scale ^ 2 - 6 * (p.nu_scaled * p.dt_latt)) *
        (gridLInf f - |f i j k|)
      ≤ ((N : Int) ^ 2 * p.ns_scale ^ 2) *
        (gridLInf f - |diffusion_step f p i j k|) := by
  have hceil := diffusion_step_local_ceiling f p h hν hdt hns_pos hExact i j k
  have hfle := abs_le_gridLInf f i j k
  nlinarith [hceil, hfle]

/-! ## Discrete maximum principle, one-sided lemmas -/

/-- **Diffusion respects local maxima.** Where the discrete Laplacian is
    non-positive (e.g. at any local max), the diffusion step does not
    increase the value: `D(x) ≤ f(x)`. -/
theorem diffusion_step_le_of_laplacian_nonpos (f : Grid) (p : NsParams)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i' j' k' : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * laplacian f i' j' k'))
    (i j k : Fin N) (hlap : laplacian f i j k ≤ 0) :
    diffusion_step f p i j k ≤ f i j k := by
  set NS2 : Int := (N : Int) ^ 2 * p.ns_scale ^ 2 with hNS2_def
  set cv : Int := p.nu_scaled * p.dt_latt with hcv_def
  have hNS2_pos : 0 < NS2 := by positivity
  have hcv_nonneg : 0 ≤ cv := mul_nonneg hν hdt
  have hq_exact : NS2 * (cv * laplacian f i j k / NS2) = cv * laplacian f i j k := by
    obtain ⟨n, hn⟩ := hExact i j k
    rw [hn, Int.mul_ediv_cancel_left _ (ne_of_gt hNS2_pos)]
  have hkey : NS2 * diffusion_step f p i j k = NS2 * f i j k + cv * laplacian f i j k := by
    have hD : diffusion_step f p i j k = f i j k + cv * laplacian f i j k / NS2 := rfl
    rw [hD, mul_add, hq_exact]
  have hcvlap : cv * laplacian f i j k ≤ 0 :=
    mul_nonpos_of_nonneg_of_nonpos hcv_nonneg hlap
  have hmul : NS2 * diffusion_step f p i j k ≤ NS2 * f i j k := by linarith
  exact le_of_mul_le_mul_left hmul hNS2_pos

/-- **Diffusion respects local minima.** Where the discrete Laplacian is
    non-negative (e.g. at any local min), the diffusion step does not
    decrease the value: `f(x) ≤ D(x)`. -/
theorem diffusion_step_ge_of_laplacian_nonneg (f : Grid) (p : NsParams)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i' j' k' : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * laplacian f i' j' k'))
    (i j k : Fin N) (hlap : 0 ≤ laplacian f i j k) :
    f i j k ≤ diffusion_step f p i j k := by
  set NS2 : Int := (N : Int) ^ 2 * p.ns_scale ^ 2 with hNS2_def
  set cv : Int := p.nu_scaled * p.dt_latt with hcv_def
  have hNS2_pos : 0 < NS2 := by positivity
  have hcv_nonneg : 0 ≤ cv := mul_nonneg hν hdt
  have hq_exact : NS2 * (cv * laplacian f i j k / NS2) = cv * laplacian f i j k := by
    obtain ⟨n, hn⟩ := hExact i j k
    rw [hn, Int.mul_ediv_cancel_left _ (ne_of_gt hNS2_pos)]
  have hkey : NS2 * diffusion_step f p i j k = NS2 * f i j k + cv * laplacian f i j k := by
    have hD : diffusion_step f p i j k = f i j k + cv * laplacian f i j k / NS2 := rfl
    rw [hD, mul_add, hq_exact]
  have hcvlap : 0 ≤ cv * laplacian f i j k := mul_nonneg hcv_nonneg hlap
  have hmul : NS2 * f i j k ≤ NS2 * diffusion_step f p i j k := by linarith
  exact le_of_mul_le_mul_left hmul hNS2_pos

/-- **The Laplacian is non-positive at a global max of `f`.**
    If `f(x) = max` over the grid, every neighbor is ≤ `f(x)`, so the
    6-point Laplacian is ≤ 0. The hypothesis is stated pointwise
    (`∀ nbr, f nbr ≤ f x` for the 6 stencil neighbors). -/
theorem laplacian_nonpos_at_max (f : Grid) (i j k : Fin N)
    (hmax : ∀ i' j' k' : Fin N, f i' j' k' ≤ f i j k) :
    laplacian f i j k ≤ 0 := by
  unfold laplacian
  have h1 := hmax (sp i) j k
  have h2 := hmax (sn i) j k
  have h3 := hmax i (sp j) k
  have h4 := hmax i (sn j) k
  have h5 := hmax i j (sp k)
  have h6 := hmax i j (sn k)
  linarith

/-- **The Laplacian is non-negative at a global min of `f`.** -/
theorem laplacian_nonneg_at_min (f : Grid) (i j k : Fin N)
    (hmin : ∀ i' j' k' : Fin N, f i j k ≤ f i' j' k') :
    0 ≤ laplacian f i j k := by
  unfold laplacian
  have h1 := hmin (sp i) j k
  have h2 := hmin (sn i) j k
  have h3 := hmin i (sp j) k
  have h4 := hmin i (sn j) k
  have h5 := hmin i j (sp k)
  have h6 := hmin i j (sn k)
  linarith

/-- **Discrete maximum principle for exact diffusion (composed).**
    At a global max point the diffusion step does not increase the value;
    at a global min it does not decrease it. The first brick of the
    max-point closure route flagged in the header. -/
theorem diffusion_max_principle (f : Grid) (p : NsParams)
    (hν : 0 ≤ p.nu_scaled) (hdt : 0 ≤ p.dt_latt) (hns_pos : 0 < p.ns_scale)
    (hExact : ∀ i' j' k' : Fin N,
      ((N : Int) ^ 2 * p.ns_scale ^ 2) ∣
        (p.nu_scaled * p.dt_latt * laplacian f i' j' k'))
    (i j k : Fin N)
    (hmax : ∀ i' j' k' : Fin N, f i' j' k' ≤ f i j k) :
    diffusion_step f p i j k ≤ f i j k :=
  diffusion_step_le_of_laplacian_nonpos f p hν hdt hns_pos hExact i j k
    (laplacian_nonpos_at_max f i j k hmax)

end NSRegularity.NS.DiffusionCeiling
