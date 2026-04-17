import NSRegularity.Kill113.Ops import NSRegularity.Kill113.SBP /-! # Kill113/Component — single-component energy residual (L1.8) -/ namespace NSRegularity.Kill113 open NSRegularity /-- L1.8 — single-component energy residual is zero. For any velocity `u` and any single vorticity component `ωc : Grid`, the 3D inner product Σ ωc · (advective + conservative) equals 0. Decomposes into three 1D summation-by-parts (one per spatial direction). -/ theorem component_residual_zero (u : VField) (ωc : Grid) : sumGrid (fun i j k => ωc i j k * (advective u ωc i j k + conservative u ωc i j k)) = 0 := by
  have h₁ : sumGrid (fun i j k => ωc i j k * advective u ωc i j k) = 0 := by
    apply sumGrid_advective_zero
  have h₂ : sumGrid (fun i j k => ωc i j k * conservative u ωc i j k) = 0 := by
    apply sumGrid_conservative_zero
  have h₃ : sumGrid (fun i j k => ωc i j k * (advective u ωc i j k + conservative u ωc i j k)) =
      sumGrid (fun i j k => ωc i j k * advective u ωc i j k) +
      sumGrid (fun i j k => ωc i j k * conservative u ωc i j k) := by
    apply sumGrid_add
  rw [h₃, h₁, h₂]
  simp

end NSRegularity.Kill113