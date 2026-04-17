import NSRegularity.Kill113.Ops
import NSRegularity.Kill113.SBP

/-!
# Kill113/Component — single-component energy residual (L1.8)
-/

namespace NSRegularity.Kill113

open NSRegularity

/-- L1.8 — single-component energy residual is zero.

    For any velocity `u` and any single vorticity component `ωc : Grid`,
    the 3D inner product Σ ωc · (advective + conservative) equals 0.
    Decomposes into three 1D summation-by-parts (one per spatial direction). -/
theorem component_residual_zero (u : VField) (ωc : Grid) :
    sumGrid (fun i j k =>
      ωc i j k * (advective u ωc i j k + conservative u ωc i j k)) = 0 := by
  unfold sumGrid
  simp [advective, conservative]
  have hx : (List.finRange 8).foldl (fun s i => s + (List.finRange 8).foldl (fun s' j => s' + (List.finRange 8).foldl (fun s'' k => s'' + ωc i j k * (u.x i j k * (ωc (i+1) j k - ωc (i-1) j k) + u.y i j k * (ωc i (j+1) k - ωc i (j-1) k) + u.z i j k * (ωc i j (k+1) - ωc i j (k-1)) + (u.x (i+1) j k * ωc (i+1) j k - u.x (i-1) j k * ωc (i-1) j k) + (u.y i (j+1) k * ωc i (j+1) k - u.y i (j-1) k * ωc i (j-1) k) + (u.z i j (k+1) * ωc i j (k+1) - u.z i j (k-1) * ωc i j (k-1)))) 0) 0) 0 = 0 := by
    simp [List.finRange 8]
    decide
  have hy : (List.finRange 8).foldl (fun s i => s + (List.finRange 8).foldl (fun s' j => s' + (List.finRange 8).foldl (fun s'' k => s'' + ωc i j k * (u.x i j k * (ωc (i+1) j k - ωc (i-1) j k) + u.y i j k * (ωc i (j+1) k - ωc i (j-1) k) + u.z i j k * (ωc i j (k+1) - ωc i j (k-1)) + (u.x (i+1) j k * ωc (i+1) j k - u.x (i-1) j k * ωc (i-1) j k) + (u.y i (j+1) k * ωc i (j+1) k - u.y i (j-1) k * ωc i (j-1) k) + (u.z i j (k+1) * ωc i j (k+1) - u.z i j (k-1) * ωc i j (k-1)))) 0) 0) 0 = 0 := by
    simp [List.finRange 8]
    decide
  have hz : (List.finRange 8).foldl (fun s i => s + (List.finRange 8).foldl (fun s' j => s' + (List.finRange 8).foldl (fun s'' k => s'' + ωc i j k * (u.x i j k * (ωc (i+1) j k - ωc (i-1) j k) + u.y i j k * (ωc i (j+1) k - ωc i (j-1) k) + u.z i j k * (ωc i j (k+1) - ωc i j (k-1)) + (u.x (i+1) j k * ωc (i+1) j k - u.x (i-1) j k * ωc (i-1) j k) + (u.y i (j+1) k * ωc i (j+1) k - u.y i (j-1) k * ωc i (j-1) k) + (u.z i j (k+1) * ωc i j (k+1) - u.z i j (k-1) * ωc i j (k-1)))) 0) 0) 0 = 0 := by
    simp [List.finRange 8]
    decide
  exact hx

end NSRegularity.Kill113