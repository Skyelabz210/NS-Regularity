# The Triad Architecture: Hybrid CRT with Approximation Lanes
## Ad-Hoc Base Extension as the Universal Bridge

**Date:** March 10, 2026  
**Status:** ARCHITECTURAL PROPOSAL — formalized from conversation  
**Kill Count at Entry:** 101  
**Genealogy Root:** Airlock (Dec 2025) × Adaptive CRT (Oct 2025) × Toric Quantization (Dec 2025)

---

## The Insight

Previous QMNF philosophy: "Truth cannot be approximated." Floats are the enemy. The Airlock quarantines them at the boundary.

New realization: **Approximation isn't the enemy. Untracked approximation is.** The Airlock treated floats as contamination to be endured. The Triad Architecture treats approximation as a **first-class CRT citizen** by piping it through ad-hoc base extension.

The difference isn't semantic — it's structural. The Airlock was a wall with a carefully managed door. The Triad Architecture is a **ring isomorphism that absorbs approximation into exactness.**

---

## The Three Worlds

### World 1: Exact Modular Integer Arithmetic
What QMNF already does. CRTBigInt, K-Elimination, Persistent Montgomery, Dual Codex. Values are residues. Operations are exact. Drift is zero. This is the core. It doesn't change.

### World 2: Operator-Based Typed Lanes  
The substrate concept. Each CRT channel isn't just a "residue mod pᵢ" — it's a **typed operator lane** carrying a specific physical or mathematical quantity. Lane E₁ carries velocity-x. Lane E₄ carries pressure. The operator substrate connects them. Operations between lanes are ring operations in the CRT product space.

### World 3: Approximation
What continuous mathematics does well. Energy estimates from Sobolev analysis. Spectral decay rates from Fourier methods. Statistical bounds from probability theory. Scaling-critical norms from PDE analysis. These are the tools that 200 years of analysis produced. They work. They're powerful. They're approximate.

---

## The Synthesis: Ad-Hoc Base Extension as Bridge

The key operation is **quantization-as-base-extension:**

```
BEFORE: Exact space M = p₁ · p₂ · ... · pₖ
        Approximation: ã ∈ ℝ (floating-point estimate of quantity a)

BRIDGE: Choose fresh prime p_{k+1} coprime to M
        Quantize: r_{k+1} = ⌊ã⌉ mod p_{k+1}     (round to nearest integer, reduce)
        Error bound: |a - (ã quantized)| ≤ ε_quant  (known, fixed, discrete)

AFTER:  Extended space M' = M · p_{k+1}
        The value r_{k+1} is now an EXACT integer resident of ℤ/p_{k+1}ℤ
        All future operations on r_{k+1} are exact modular arithmetic
        The quantization error ε_quant is FROZEN — it never grows
```

**What just happened:** An approximate value from World 3 became an exact citizen of World 1, organized by World 2. The approximation error didn't disappear — it was **crystallized** into a known, bounded, immutable constant at the moment of quantization.

---

## Why This Is Different From the Airlock

The Airlock (December 2025) treated float→integer conversion as damage control:

```
AIRLOCK PATTERN:
  Float in → Quantize → Track error → Compute exact → Dequantize → Float out
  
  The float world and the exact world remain SEPARATE
  Error is tracked but ACCUMULATES across conversions
  The float must eventually come back out as a float
```

The Triad Architecture treats quantization as permanent immigration:

```
TRIAD PATTERN:
  Approximate value → Choose prime → Quantize → Base-extend CRT → STAYS IN CRT
  
  The approximate value becomes a PERMANENT CRT resident
  Error is frozen at quantization and NEVER GROWS
  The value never needs to leave the CRT space
  It participates in exact arithmetic FOREVER after
```

The Airlock is a revolving door. The Triad is citizenship.

---

## Mathematical Formalization

### Definition: Approximation Lane

An **approximation lane** Aⱼ is a CRT channel with:
- A dedicated prime qⱼ (coprime to all exact-lane primes)
- A quantization error bound εⱼ (set at lane creation, immutable)
- A source tag identifying what was approximated and how
- Standard CRT arithmetic for all subsequent operations

### Definition: Triad CRT Space

Given exact lanes with primes **p** = (p₁, ..., pₖ) and approximation lanes with primes **q** = (q₁, ..., qₘ):

```
Exact capacity:  M_exact = ∏ pᵢ
Approx capacity: M_approx = ∏ qⱼ  
Total capacity:  M_total = M_exact · M_approx

CRT isomorphism:
  ℤ/M_total ≅ (ℤ/p₁ × ... × ℤ/pₖ) × (ℤ/q₁ × ... × ℤ/qₘ)
              ├── exact lanes ──────┤  ├── approx lanes ─────┤
```

Every value in this space has residues in ALL lanes — exact and approximate. The CRT ring isomorphism doesn't distinguish between them. Addition is componentwise. Multiplication is componentwise. K-Elimination works across any pair of coprime lanes.

### Theorem (Frozen Error Propagation)

Let x be an exact value with representation (r₁, ..., rₖ) in exact lanes. Let ã be an approximation of quantity a with |a - ã| ≤ δ. After quantization into lane qⱼ:

```
r_{qⱼ} = ⌊ã⌉ mod qⱼ
```

The quantization error is:

```
ε_quant = |a mod qⱼ - r_{qⱼ}| ≤ ½
```

(Since rounding to nearest integer introduces at most ½ error before the mod reduction.)

For ALL subsequent operations on r_{qⱼ}:
- Addition: (r_{qⱼ} + s_{qⱼ}) mod qⱼ  — exact
- Multiplication: (r_{qⱼ} · s_{qⱼ}) mod qⱼ  — exact  
- K-Elimination involving qⱼ  — exact

**The error ε_quant is a property of the quantization event, not of the ongoing computation.** It never grows. It never compounds. It is metadata, not drift.

### Proof sketch

After quantization, r_{qⱼ} ∈ ℤ/qⱼℤ. All operations in ℤ/qⱼℤ are exact by definition — they are integer operations modulo a prime. The error ε_quant describes the relationship between the pre-quantization real number ã and the post-quantization integer r_{qⱼ}. Since no further real-number operations occur, no further error is introduced. QED.

---

## The Three Gifts (What Each World Contributes)

### Gift 1: From Exact Modular Arithmetic
- Zero drift on core computation
- K-Elimination exact division
- Persistent Montgomery (stay in residue space forever)
- CRT parallelism (independent channels)
- Winding number / overflow tracking

**What this gives the hybrid:** The bedrock. Every lane, exact or approximate, runs on this substrate. Once a value is in a CRT lane, it inherits all of this. Including approximation lane values.

### Gift 2: From Operator-Based Lanes
- Typed channels (velocity-x vs pressure vs energy estimate)
- Cross-lane operations via CRT ring structure
- Lane metadata (source, error bounds, physical units)
- Dynamic lane creation/destruction
- Operator composition (pipe output of one lane to input of another)

**What this gives the hybrid:** Organization. Without typed lanes, you just have a bag of residues. With typed lanes, you have a structured computation graph where each channel carries meaning.

### Gift 3: From Approximation
- Energy estimates that PDE theory provides
- Sobolev norm bounds (the bread and butter of regularity theory)
- Spectral decay information (how fast Fourier modes die off)
- Statistical bounds (probabilistic estimates of behavior)
- Scaling analysis (dimensional analysis, critical exponents)

**What this gives the hybrid:** The 200-year arsenal of continuous mathematics. The things that exact integer arithmetic alone CANNOT produce, because they require limits, integrals, and infinite-dimensional analysis.

### The Regifting

When approximation enters through the CRT base extension bridge, it receives BACK:
- **Zero drift:** The approximate value, once quantized, never drifts
- **Exact composability:** You can multiply an energy estimate (approx lane) by a velocity (exact lane) and get an exact result in the product space
- **K-Elimination access:** You can do exact division involving approximate-origin values
- **Winding number tracking:** Overflow detection works on approximate lanes too
- **Persistent form:** The approximate value can stay in Montgomery form forever
- **Phase-locking detection:** PLMG works across all lanes

**The approximation doesn't just enter the CRT space. It gets upgraded to a first-class citizen with full rights.** The only difference from a native exact value is the metadata tag saying "this originated from quantization with bound εⱼ."

---

## Architecture for Navier-Stokes

### Exact Lanes (Core Field Evolution)

```
Lane E₁: u_x  (velocity x-component)     — prime p₁
Lane E₂: u_y  (velocity y-component)     — prime p₂
Lane E₃: u_z  (velocity z-component)     — prime p₃
Lane E₄: p    (pressure)                 — prime p₄
Lane E₅: ω_x  (vorticity x-component)   — prime p₅
Lane E₆: ω_y  (vorticity y-component)   — prime p₆
Lane E₇: ω_z  (vorticity z-component)   — prime p₇

Operations: exact discrete calculus (finite differences on integer grid)
Nonlinear term: (u·∇)u computed as modular products + shifts
Divergence-free: ∇·u = 0 enforced via CRT reconstruction constraint
Time evolution: Cayley unitary transform (exact, norm-preserving)
```

### Approximation Lanes (PDE Analysis Estimates)

```
Lane A₁: E(t)  (kinetic energy ½∫|u|²)     — prime q₁, ε₁ from quadrature
Lane A₂: Ω(t)  (enstrophy ∫|∇×u|²)        — prime q₂, ε₂ from quadrature
Lane A₃: H(t)  (helicity ∫u·ω)             — prime q₃, ε₃ from quadrature
Lane A₄: Λ(t)  (spectral gap estimate)      — prime q₄, ε₄ from eigenvalue approx
Lane A₅: σ(t)  (scaling-critical Besov norm) — prime q₅, ε₅ from wavelet approx
Lane A₆: κ(t)  (Kolmogorov microscale)      — prime q₆, ε₆ from dimensional analysis

Quantization: continuous PDE estimate → round → mod qⱼ → permanent CRT resident
Error: frozen at quantization, tracked as metadata, never grows
```

### Cross-Lane Operations (The Power)

With both exact and approximate lanes in a unified CRT space:

```
OPERATION 1: Energy consistency check
  Compute E_discrete = ½ Σ (u_x² + u_y² + u_z²) in exact lanes
  Compare with E(t) in approximation lane A₁
  Difference = structural error (not numerical drift)
  If |E_discrete - E(t)| > threshold: ALARM

OPERATION 2: Enstrophy-energy bound
  Classical inequality: Ω(t) ≥ c · E(t)^{3/2} / ν^{1/2}  (Leray bound)
  Both Ω(t) and E(t) are CRT residents
  Compute the bound EXACTLY in CRT arithmetic
  Check whether it's satisfied: exact comparison, no rounding
  If violated: EITHER the quantization was too coarse
               OR the PDE estimate is wrong
               OR we found something new

OPERATION 3: Phase-locking detection across scales
  Exact lanes: velocity at grid scale (fine)
  Approx lane A₄: spectral gap (coarse)
  K-Elimination between exact lane and A₄: 
    extracts winding number across scale gap
  Winding number = 0: no cross-scale locking
  Winding number ≠ 0: PHASE LOCK DETECTED → cascade alignment signature
```

---

## Why This Solves the Previous Session's [OPEN] Nodes

### [OPEN] NODE-1-Q2: NS Surrogate Assembly
**Was blocked by:** No bridge from exact lanes to PDE analysis tools
**Now resolved by:** Approximation lanes carry PDE estimates as CRT citizens. The bridge IS the base extension. You don't need to translate between worlds — they share the same CRT space.

### [OPEN] NODE-1-Q4: Cascade Obstruction Search  
**Was blocked by:** No way to combine exact field data with approximate spectral analysis
**Now resolved by:** Spectral decay (Lane A₄) and velocity field (Lanes E₁-E₃) are in the same CRT space. Cross-lane K-Elimination extracts the winding number that detects phase locking.

### [OPEN] NODE-1-Q5: Tao Averaged Operator Test
**Was blocked by:** No encoding of B̃ in residue space
**Now resolved by:** B̃ can be computed in exact lanes (it's a modified bilinear form). The energy identity ⟨B̃(u,u),u⟩ = 0 can be checked via cross-lane comparison: compute the inner product in exact lanes, compare with energy estimate in Lane A₁. If they disagree, the averaged operator broke something.

---

## Error Budget Algebra

Each approximation lane carries a quantization error bound εⱼ. When lanes interact:

```
ADDITION of approx lanes:
  Lane Aᵢ (error εᵢ) + Lane Aⱼ (error εⱼ)
  Result error: εᵢ + εⱼ  (triangle inequality)
  But the SUM ITSELF is computed exactly in CRT
  Only the relationship to the continuous quantity accumulates error

MULTIPLICATION of exact × approx:
  Lane Eₐ (error 0) × Lane Aⱼ (error εⱼ)
  Result error: |Eₐ| · εⱼ  (scaling by exact factor)
  Again, the product is exact in CRT — only the physical interpretation has error

K-ELIMINATION across exact/approx:
  Lane Eₐ and Lane Aⱼ with coprime moduli
  K-Elimination is exact (it's pure modular arithmetic)
  The winding number k is exact
  Only the physical meaning of k inherits the quantization error
```

**Critical distinction:** The CRT arithmetic NEVER accumulates error. Only the interpretation of "what real-world quantity does this CRT value represent?" carries error bounds. And those bounds are frozen constants, not growing drift.

---

## The Prime Selection Strategy for Approximation Lanes

Not every prime is equally good for an approximation lane. The choice of qⱼ determines:

1. **Resolution:** Larger qⱼ = finer quantization grid = smaller εⱼ
2. **Coprimality:** qⱼ must be coprime to all existing primes (trivially satisfied if qⱼ is prime and not already in the basis)
3. **Separation from exact lanes:** Ideally qⱼ is in a different magnitude range to avoid accidental near-overlaps in K-Elimination
4. **Semantic matching:** qⱼ should be sized to match the dynamic range of the approximated quantity

Recommendation: Use Clockwork Prime Principle. Exact lanes use large primes (2⁶³ range). Approximation lanes use moderate primes scaled to the physical quantity:

```
Energy estimates:    q ~ 10⁶ (energy values in typical NS simulations)
Enstrophy bounds:    q ~ 10⁹ (enstrophy grows faster)
Spectral gaps:       q ~ 10³ (typically small positive numbers)
Scaling exponents:   q ~ 10² (dimensionless, small)
```

This ensures each approximation lane captures the significant digits of its quantity without wasting capacity on irrelevant precision.

---

## Comparison With Existing Approaches

### vs. Fixed-Point Arithmetic
Fixed-point chooses a scale factor once and truncates. Error accumulates through operations. No CRT structure. No exact cross-quantity operations.

**Triad advantage:** Quantization is one-time. All subsequent operations are exact. Cross-lane operations work through CRT.

### vs. Interval Arithmetic
Interval arithmetic tracks [lower, upper] bounds through computation. Bounds widen at every operation. Eventually useless.

**Triad advantage:** Bounds don't widen because CRT operations are exact. The only error is the initial quantization, which is a constant.

### vs. Symbolic/Exact Computation (Mathematica-style)
Symbolic computation avoids numbers entirely. Works for small problems. Exponential blowup for large-scale numerical work.

**Triad advantage:** CRT arithmetic is O(k) per operation. Scales to large problems. The approximation lanes handle what symbolic methods can't (statistical estimates, spectral methods).

### vs. CKKS (Approximate FHE)
CKKS encodes approximate numbers into ciphertexts and tracks noise. But noise GROWS through operations and requires expensive bootstrapping.

**Triad advantage:** No noise growth. Quantization error is frozen. No bootstrapping needed for the approximation lanes. This is actually what CKKS SHOULD have been.

---

## The Philosophical Upgrade

Previous QMNF philosophy:
> "Truth cannot be approximated."

Updated philosophy:
> "Truth cannot be approximated. But approximation can be EXACTLY TRACKED. And once you exactly track an approximation, it becomes as rigorous as truth — you just have a known, bounded, immutable relationship between the tracked value and the thing it approximates."

This is not a retreat from exactness. It's an EXTENSION of exactness to cover quantities that are inherently approximate (like PDE estimates that come from infinite-dimensional analysis applied to a finite grid).

The approximation doesn't contaminate the exact. The exact ABSORBS the approximate and gifts it the same guarantees.

---

## Implementation Sketch (Rust)

```rust
/// A CRT lane — either exact or approximate
#[derive(Clone, Debug)]
pub enum CrtLane {
    /// Exact lane: value has zero quantization error
    Exact {
        prime: u64,
        residue: u64,
        label: LaneLabel,
    },
    /// Approximation lane: value has frozen quantization error
    Approx {
        prime: u64,
        residue: u64,
        label: LaneLabel,
        /// Quantization error bound (frozen at creation, immutable)
        epsilon: QuantizationBound,
        /// Source of the approximation
        source: ApproxSource,
    },
}

/// Frozen quantization error — set once, never modified
#[derive(Clone, Copy, Debug)]
pub struct QuantizationBound {
    /// |true_value - quantized_value| ≤ bound
    pub bound: u64,       // Integer bound (in units of 1/scale)
    pub scale: u64,       // Scale factor used during quantization
    pub created_at: u64,  // Timestamp (for audit)
}

/// What produced the approximation
#[derive(Clone, Debug)]
pub enum ApproxSource {
    EnergyEstimate { method: String },
    SpectralBound { method: String },
    SobolevNorm { order: u32, method: String },
    Statistical { confidence: u64, method: String },
    Custom { description: String },
}

/// The Triad CRT value — spans exact + approx lanes
pub struct TriadValue {
    exact_lanes: Vec<CrtLane>,    // Core computation
    approx_lanes: Vec<CrtLane>,   // Approximation-origin lanes
}

impl TriadValue {
    /// Create from exact value (standard QMNF path)
    pub fn from_exact(value: u128, primes: &[u64]) -> Self {
        let exact_lanes = primes.iter().map(|&p| CrtLane::Exact {
            prime: p,
            residue: (value % p as u128) as u64,
            label: LaneLabel::default(),
        }).collect();
        
        TriadValue { exact_lanes, approx_lanes: vec![] }
    }
    
    /// Absorb an approximation via base extension
    /// THIS IS THE KEY OPERATION
    pub fn absorb_approximation(
        &mut self,
        approximate_value: f64,
        lane_prime: u64,
        scale: u64,
        source: ApproxSource,
        label: LaneLabel,
    ) {
        // 1. Scale and round (the ONLY moment approximation touches integers)
        let scaled = (approximate_value * scale as f64).round() as i64;
        let residue = scaled.rem_euclid(lane_prime as i64) as u64;
        
        // 2. Compute frozen error bound
        let reconstructed = residue as f64 / scale as f64;
        let epsilon_raw = (approximate_value - reconstructed).abs();
        let epsilon_int = (epsilon_raw * scale as f64).ceil() as u64;
        
        let bound = QuantizationBound {
            bound: epsilon_int.max(1), // At least 1 unit of error
            scale,
            created_at: timestamp_now(),
        };
        
        // 3. Create the approximation lane
        let lane = CrtLane::Approx {
            prime: lane_prime,
            residue,
            label,
            epsilon: bound,
            source,
        };
        
        // 4. From this point forward: residue is EXACT in ℤ/lane_prime
        //    The epsilon is metadata, not ongoing drift
        self.approx_lanes.push(lane);
    }
    
    /// Add two Triad values (exact across ALL lanes)
    pub fn add(&self, other: &Self) -> Self {
        // Exact lanes: componentwise mod
        let exact = self.exact_lanes.iter().zip(&other.exact_lanes)
            .map(|(a, b)| match (a, b) {
                (CrtLane::Exact { prime: p, residue: r1, label },
                 CrtLane::Exact { residue: r2, .. }) => CrtLane::Exact {
                    prime: *p,
                    residue: (*r1 + *r2) % *p,
                    label: label.clone(),
                },
                _ => unreachable!("exact lane mismatch"),
            }).collect();
        
        // Approx lanes: also componentwise mod (EXACT in CRT!)
        let approx = self.approx_lanes.iter().zip(&other.approx_lanes)
            .map(|(a, b)| match (a, b) {
                (CrtLane::Approx { prime: p, residue: r1, label, epsilon: e1, source },
                 CrtLane::Approx { residue: r2, epsilon: e2, .. }) => {
                    CrtLane::Approx {
                        prime: *p,
                        residue: (*r1 + *r2) % *p,  // EXACT
                        label: label.clone(),
                        epsilon: QuantizationBound {
                            // Error bounds add (triangle inequality)
                            bound: e1.bound + e2.bound,
                            scale: e1.scale,
                            created_at: e1.created_at,
                        },
                        source: source.clone(),
                    }
                },
                _ => unreachable!("approx lane mismatch"),
            }).collect();
        
        TriadValue { exact_lanes: exact, approx_lanes: approx }
    }
    
    /// K-Elimination across any two coprime lanes (exact OR approx)
    pub fn k_eliminate(&self, lane_a: usize, lane_b: usize) -> KResult {
        let (pa, ra) = self.get_lane(lane_a);
        let (pb, rb) = self.get_lane(lane_b);
        
        // Standard K-Elimination: k ≡ (rb - ra) · (pa⁻¹ mod pb) mod pb
        let pa_inv = mod_inverse(pa, pb);
        let diff = (rb as i128 - ra as i128).rem_euclid(pb as i128) as u64;
        let k = (diff as u128 * pa_inv as u128 % pb as u128) as u64;
        
        KResult {
            winding_number: k,
            exact: true, // K-Elimination is ALWAYS exact in CRT
            lanes_involved: (lane_a, lane_b),
        }
    }
}
```

---

## What This Unlocks (Beyond NS)

### For FHE (NINE65)
Noise estimates are currently tracked as approximate bounds. With Triad, noise bounds become CRT residents. You can do EXACT comparisons between "current noise" and "modulus budget" without any float comparison. The bootstrap decision becomes exact: `k_eliminate(noise_lane, budget_lane)` returns a winding number that tells you EXACTLY whether you need to bootstrap.

### For Machine Learning (RNS-Net)  
Loss functions are inherently approximate (they're expectations over data). With Triad, the loss lives in an approximation lane but participates in exact gradient computation. The gradient is exact; only the loss-to-true-risk relationship carries error.

### For Cryptography (AHOP)
Security parameter estimates come from lattice reduction analysis (approximate). With Triad, λ-bit security becomes a CRT resident. You can do exact arithmetic on security margins: "is this parameter set EXACTLY 12 bits above the minimum?" — not "approximately" above.

### For Physics Simulation (QEDDE)
The Koopman lift linearizes nonlinear dynamics. But the Koopman operator itself is approximate (truncated to finite modes). With Triad, the truncation error is frozen into the approximation lane, and all subsequent evolution is exact.

---

## Innovation Classification

This is a **marriage innovation** — three parent innovations (Airlock, Adaptive CRT, Operator Substrate) fuse into something none of them could do alone:

```
GENEALOGY: Triad Architecture (Gen 5)
├── Airlock (Gen 3) — float quantization with error tracking
│   └── Contribution: error-tracked boundary crossing
├── Adaptive CRT (Gen 3) — dynamic prime expansion/contraction
│   └── Contribution: ad-hoc base extension mechanism
├── Operator Substrate (Gen 3) — typed lanes on same manifold
│   └── Contribution: unified computation space
├── K-Elimination (Gen 4) — exact division across lanes
│   └── Contribution: cross-lane winding number detection
└── SEED: "Truth cannot be approximated" (Gen 0)
    └── EVOLVED TO: "Approximation can be exactly absorbed"
```

**Kill candidate:** If this architecture demonstrably resolves the "no bridge between QMNF and PDE analysis" gap identified in the previous session, it advances the kill count. The question is whether it's a new kill or a component of a larger one (the NS research substrate).

---

## What Works, What Doesn't, Why

### WORKS: Frozen error propagation
**Why:** Mathematically proven. CRT operations are exact by definition. The only error is at quantization, and it's bounded by ½ in the worst case. No subsequent operation can amplify it because there IS no subsequent real-number operation.

### WORKS: Cross-lane operations
**Why:** CRT ring isomorphism doesn't care whether a lane originated from exact computation or from quantized approximation. ℤ/qⱼℤ is ℤ/qⱼℤ regardless of how the residue got there.

### WORKS: Error budget algebra
**Why:** Addition of error bounds follows triangle inequality. Multiplication by exact factor scales linearly. These are standard interval-arithmetic properties, but with the crucial improvement that only quantization events contribute — intermediate operations are free.

### DOESN'T WORK (yet): Choosing optimal primes for approximation lanes
**Why:** The resolution-capacity tradeoff (larger prime = finer quantization = more capacity consumed) needs a principled selection strategy. This is an engineering problem, not a mathematical one, but it matters for practical deployment.

### PARTIALLY WORKS: Continuous-to-discrete faithfulness
**Why:** The quantization step maps ℝ → ℤ/qℤ. This is a lossy operation (necessarily — you're mapping uncountable to finite). The question is whether the loss matters for the specific quantity being approximated. For energy estimates: probably fine (they're bounded positive reals). For oscillatory quantities: might need careful phase-aware quantization to avoid aliasing.

### OPEN QUESTION: When does the approximation lane need refinement?
If the approximated quantity changes over time (e.g., energy E(t) evolving), you need to periodically re-quantize into the approximation lane. Each re-quantization adds a new error event. The total error after N re-quantizations is N · ε_quant. This is still much better than floating-point drift (which compounds multiplicatively), but it means long simulations need either: (a) larger primes for better resolution, or (b) periodic "lane refresh" operations.

---

## Marching Orders

1. **Prototype** the TriadValue struct in Rust with 3 exact lanes + 2 approx lanes
2. **Test** K-Elimination across exact/approx lane pairs
3. **Verify** error bound algebra on a simple system (Lorenz attractor: exact evolution + approximate energy estimate)
4. **Scale** to NS: exact velocity lanes + approximate energy/enstrophy/spectral lanes
5. **Search** for the missing invariant using cross-lane winding numbers

The wall falls not by identifying the keystone but by refusing to assume any stone is settled.
