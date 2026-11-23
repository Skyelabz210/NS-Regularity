# Which Navier–Stokes estimates survive refinement?

The periodic, unforced problem on the unit three-torus asks for a smooth velocity for every smooth divergence-free initial velocity, every positive viscosity and every finite time interval. The results below repair the finite arithmetic, identify an obstruction to iterating the centered-curl inverse, and give a restricted continuum comparison. General three-dimensional stretching still requires an a priori estimate.

## 1. A mesh-independent maximum principle for the actual rounded stencil

Write $D=N^2 S^2>0$, $c=\nu_{\rm scaled}\,dt_{\rm latt}\ge0$, and let $L f=\sum_{y\sim x} f_y-6f_x$. The scalar Lean update is $G_x=f_x+\lfloor c Lf_x/D\rfloor$. Since $f_x$ is integral, this is the floor of
$$
(D-6c) f_x/D + (c/D)\sum_{y\sim x} f_y .
$$
Under $6c\le D$ the averaging weights are nonnegative and sum to one. Integer endpoints of an enclosing interval survive flooring. The argument covers signed data, every resolution and every number of steps by induction. Exact-division hypotheses still matter for other identities, including the published joint energy certificate. The energy proof uses a separate quadratic identity.

The Lean declarations `rounded_stencil_interval`, `diffusion_max_preserving` and `RefinedDiffusion.preserves_interval` implement this argument. `agrees_at_eight` identifies the new family with the original scalar update by definitional equality. `comparison` proves that an integer offset between two inputs bounds their one-step separation. `error_from_local_residual` iterates this comparison against an independently sampled reference. An initial error $E$ and a one-step reference defect bounded by $R$ imply a time-$m$ error at most $E+mR$, at every site and mesh size.

## 2. Full-step growth and the precise meaning of the grid bound

For each component let $g$ be the rounded diffusion output, $a$ the signed integer advection quotient evaluated on $g$, and $s$ the signed stretching quotient evaluated on the input vorticity and velocity. Lean evolves $g-a+s$. Write $e=s-a$ for the three-component increment field. The maximum principle and triangle inequality give, on the actual reached states,
$$
 \|w_n\|_{\ell^\infty}\le \|w_0\|_{\ell^\infty}+
      \sum_{m<n}\|e_m\|_{\ell^\infty}.
$$
`RoundedStep.trajectory_bound` has no per-step contraction or exact-division premise. The sum is a measurable quantity in grid units. A bound in physical units uniform in resolution and horizon up to each fixed $T$ remains an analytical obligation. The triangle inequality can lose important cancellations over time.

Here is a counterexample to contraction based only on a divergence-free inverse witness, canonical-form stretching and diffusion CFL. At $N=8$ let $b=(0,32,0,-32)$ repeat with period four. Take $u=(b_j,b_k,0)$, $w=\mathrm{curl}_h u$ with the repository's unscaled centered differences, and $\nu_{\rm scaled}=dt_{\rm latt}=S=1$. All diffusion quotients are exact. Initially the component maximum is 64. The y-component of $w$ vanishes identically, so its diffusion and advection contributions vanish. Stretching gives $w'_y(0)=-64\cdot64/16=-256$. The velocity has zero mean and no blind Fourier modes. Lean checks the step with this explicit velocity. The same y-component also grows for **every** solenoidal right inverse of the raw centered curl. Indeed, for any other admissible velocity $v$, $v-u$ has zero curl and divergence. The [parity-kernel calculation](CENTERED_BIOT_SAVART_BOUND.md) shows that every centered derivative of $v-u$ vanishes. Thus the stretching contribution is identical. Advection in the y-component acts on an identically zero diffused field for either velocity. In particular the written argument applies to the original Lean inverse under its stated curl and divergence contracts, independent of its gauge. The Rust spectral operator has different symbols. One-step growth is compatible with global smoothness.

`NS.lInf` is the component maximum. `Continuum.lInfGrid` is the **sum** of the absolute values of all three components over all cells. For the fixed grid,
$$
 \|w\|_{\ell^\infty}\le \mathrm{lInfGrid}(w)\le 3N^3\|w\|_{\ell^\infty}.
$$
`UniformlyBoundedGridTrajectory` now names the latter bound over time. `IsSmooth` is its compatibility alias. With values stored in units $1/A_N$, the physical cellwise supremum is the component maximum divided by $A_N$, and the integral of the component sum is $h^3\mathrm{lInfGrid}/A_N$. These normalizations have different uses at refinement.

## 3. Divisibility, curl inversion and Theory of the One

**Degenerate quantifier.** At the origin take a field with value one at the positive x-neighbour and zero elsewhere. Its Laplacian at the origin is one. The historical capstone's x-component premise therefore implies $D\mid c$. With $D>0$ and $0\le6c\le D$, this forces $c=0$. The new maximum principle and reached-trajectory estimate admit positive $c$ without this premise. As a positive-control orbit, take $N=8,S=3,c=96,dt=1$, and $u=(0,0,b_j(-1)^k)$ with $b=(0,1,0,-1)$ periodic. Its centered curl has maximum two, the scalar Laplacian on that mode is $-6w$, and both nonlinear terms vanish. The supplied inverse pair evolves to zero in one step and stays there with zero velocity.

There is a second obstruction even if universal exactness is removed. On any nonzero field with identically zero y-component, its y diffusion is zero, so the old ceiling requires $D_y\ge0$. Yet its y-budget requires $6|dt|C_{BS}\|w\|_\infty^2\le-D_y$. This excludes positive time steps when $C_{BS}>0$, including the representable shear just exhibited. The signed-increment estimate uses the actual y-increment, which vanishes on that shear.

**Failure of curl-range invariance.** Put $r_x=(c Lf_x)\bmod D$. Summing the local division identity and using periodicity gives the exact equation
$$
 D\left(\sum_x G_x-\sum_x f_x\right)=-\sum_x r_x .
$$
Consequently mean preservation for this scalar sweep is equivalent to exact divisibility at every site on this state when $D>0$. For $N=8,c=S=dt=1$, start with $u=(0,0,\mathbf1_{j=0})$, $w=\mathrm{curl}_h u$. Advection and stretching vanish. Its initial x-component has Laplacian one at row $j=6$. Thus universal divisibility restricted even to all representable inputs still forces $c=0$ under CFL. Total x-vorticity changes from zero to $-128$, with $\sum r_x=8192$ and $D=64$. Every periodic centered curl has zero component sums. Even a correct inverse on its representable domain therefore needs a repair to propagate this orbit. One scalar repair assigns a single oriented flux $F_{xy}=\operatorname{trunc}_0(c(f_y-f_x)/D)$ to each face, adding it at $x$ and subtracting it at $y$. The total change is identically zero by construction. For unequal endpoint values, the weight $F_{xy}/(f_y-f_x)$ lies in $[0,c/D]$; take weight zero when they coincide. The new center value is thus a convex combination with total neighbour weight at most $6c/D$. This proves both mass and interval preservation at every resolution. The Python fixture exercises this proposed stencil at four resolutions. It changes the rounding location. Preservation of the whole integer curl image, including the centered-derivative blind modes, still needs proof.

**Algebraic readout.** Use the authentic numerator $X=cLf_x$, its primary residue $X\bmod D$ and an adjacent reading $X\bmod(D+1)$, all from the same site, operation and time. Their modular difference reads $\lfloor X/D\rfloor\bmod(D+1)$. A signed public window of width $D+1$ identifies the integer quotient. Without a window, $X$ and $X+D(D+1)$ have identical readings and different quotients. The mass defect needs only the primary remainders and their exact sum. No winding is stored. For spatial observations the same fiber criterion gives a negative result. Zero and a unit spike at fine index one agree on all eight even samples of a 16-point periodic slice. Their fine Laplacians at zero are zero and one. Thus a decoder for that stencil target from those coarse samples alone cannot exist on unrestricted fields. The adapter imports the original signed-window and fiber theorems. [Theory of the One source](https://github.com/Skyelabz210/Theory-of-the-One/tree/6a04e4bcd9adae8284a0f9a433ac866858578ea0/formal/universal)

**An integer right inverse on its certified domain.** `CurlPreimage.minimumVelocity_spec` selects a divergence-free integer preimage of minimum component maximum on the actual centered-curl image. Its norm is bounded by that of every supplied admissible preimage. The construction uses classical choice, and ties can occur. `CurlPreimage.velocity` extends the selector by zero to a total velocity map of the type accepted by the existing trajectory generator. `velocity_admissible` proves curl recovery and zero divergence on the representable domain, and `velocity_div_zero` proves zero divergence on every input. The sharp norm bound follows from the written construction below. Identification with the Rust NTT remains open.

**A specified Biot–Savart operator in real arithmetic.** For even $N\ge4$, normalize the centered derivative on the unit torus and put $s_j(k)=N\sin(2\pi k_j/N)$. Set all modes with $s=0$ to zero. For $s\ne0$ and $s\cdot\hat w=0$, set $\hat u=i\,s\times\hat w/|s|^2$. Require $\hat w=0$ on the blind modes. Then $i s\times\hat u=\hat w$, $s\cdot\hat u=0$, and $|s|\,|\hat u|=|\hat w|$. Every nonzero symbol has length at least $N\sin(2\pi/N)\ge4$. With normalized discrete Parseval this proves
$$
 \|\nabla_h^0 u\|_2=\|w\|_2,\qquad \|u\|_2\le\tfrac14\|w\|_2.
$$
The [uniform Biot–Savart proof](CENTERED_BIOT_SAVART_BOUND.md) further derives a componentwise velocity infinity bound with constant $1/2$ on every normalized even grid and on the continuum unit torus. An exact gauge adjustment supplies an integer right inverse with raw constant two at $N=8$.

This written construction fixes the gauge and gives resolution-independent **centered-derivative** bounds on its real-valued range. For $k=(N/2-1,0,0)$, the forward-difference multiplier is of order $N$ while the centered one stays bounded. A coercive physical interpolation estimate needs control of these modes. Identifying either gauged inverse with the production NTT and bounding the nonlinear stretching term each require a separate proof. The Lean declaration restricts curl inversion to fields with a divergence-free preimage. [Integer operator contract](../lean/NSRegularity/NS/BiotSavart.lean)

## 4. A continuum comparison that can be proved for a shear

Fix $\nu>0$, $T<\infty$, and a smooth periodic scalar initial profile. The continuum velocity $u=(v(y,t),0,0)$ with constant pressure solves the full unforced equations when $v_t=\nu v_{yy}$. Its nonlinear velocity term vanishes identically. The following is a written convergence proof for the scalar family above, interpreted as evolution of that **velocity component**.

Let $N$ run through positive multiples of eight, $h=1/N$, $\tau=h^2/(8\nu)$, and quantize amplitudes in units $1/A_N$ with $A_N=N^4$. Choose $S=A_N$, $D=N^2S^2$, $c=D/8$ and $dt_{\rm latt}=1$. Start from $\lfloor A_N v(jh,0)\rfloor$. On this shear the exact real stencil $P$ has weights $1/8,3/4,1/8$. For normalized integer iterates $v_h$, each step satisfies $v_h^{m+1}=P v_h^m+\eta^m$ with $\|\eta^m\|_\infty<1/A_N$. The integral coefficient $c$ here calibrates this scalar scheme to the chosen physical viscosity and time step.

Taylor's theorem on the fixed time slab gives $\|v(t+\tau)-P v(t)\|_{\infty,\rm samples}\le C_T\tau(\tau+h^2)$, using derivatives through $v_{tt}$ and $v_{yyyy}$. The real stencil is sup-norm nonexpansive. Iterating the error inequality at the sampled nodes gives
$$
 \max_{m\tau\le T}\|v_h^m-v(m\tau)\|_{\infty,\rm samples}
 \le A_N^{-1}+T/(A_N\tau)+C_T T(\tau+h^2).
$$
Piecewise-constant spatial and temporal interpolation adds at most $h\|v_y\|_\infty+\tau\|v_t\|_\infty$. All terms tend to zero. This comparison actually passes through finer grids and yields a smooth continuum shear. Evolving a rounded vorticity and reconstructing velocity is a different algorithm, as the mass defect demonstrates. The proof does not identify that algorithm with the velocity shear or with the Rust N32 evolution. The Taylor estimate and the real Fourier inverse above are written derivations. The machine-checked parts and numerical fixtures are listed below.

**Why amplitude resolution matters even on every mesh.** Fix an amplitude scale $A$ and choose distinct $0<a<b<1/A$. The exact smooth, mean-zero periodic NS shears
$$
u^a(y,t)=(a e^{-4\pi^2\nu t}\sin(2\pi y),0,0),\qquad
u^b(y,t)=(b e^{-4\pi^2\nu t}\sin(2\pi y),0,0)
$$
have identical componentwise floor observations in units $1/A$ at every point and finite time. Both observed x-components are zero where the sine is nonnegative and $-1$ otherwise. The underlying velocities differ. `subunit_amplitudes_same_reading` proves the scalar observation identity for every signed profile sample in $[-1,1]$. This applies the Theory-of-the-One fiber test even to the collection of all refining grids. It explains the separate amplitude refinement in the convergence proof.

## 5. General nonlinear limit and the two solver paths

The main Lean `Continuum.l_inf_passage` concludes `∃ C', C' ≤ C`. `NonlinearPassage.SatisfiesDistribNS` is `True`. The independent `ns-lean` project has a varying-size field definition, but its compactness declarations keep one fixed size in the family type, include admitted proofs and return placeholder propositions. [Independent compactness source](https://github.com/Skyelabz210/ns-lean/blob/22bfe0f90d939c101ac32dd3664ff7cb322a6204/NSTorus/Compactness.lean)

| Operation | Lean split path | Rust N32 v4 path |
|---|---|---|
| Signed division | Euclidean floor for positive denominator | Extract sign, divide magnitude, restore sign (toward zero) |
| Advection | Centered advective term on the diffused field | Skew average of advective and conservative terms on the input |
| Time increment | Each split term carries its quotient | Separately scaled right-hand-side terms, then a final signed time quotient |
| Inversion | Unscaled centered curl and an opaque inverse on the representable domain | Four NTT primes with signed integer wave numbers, extended integer lanes and an explicit normalization |

A matched N32 shear test uses $S=10000$, $c=dt=1$ and a centered curl with x-component +1 at row 31 and −1 at row 1. Lean's generalized stencil gives zero at the positive unit peak. Across the full mesh it gives total x-vorticity −2048. The canonical Rust and CRT v4 production steps preserve the input at every site. Both nonlinear terms vanish on this shear. This supplies an explicit same-resolution discrepancy with positive viscosity. [Rust shear fixture](https://github.com/Skyelabz210/NS-3D-solver_n32-ntt6/blob/d9792c74dd1c40586303a1bdfbf17493ad7ee8c5/arena/crates/ns_v5/tests/rounded_shear.rs)

The separately implemented six-prime transform in `ns3d_n32_ntt6.rs` extends the available spectral representation. The active v4 step loops over four transform primes and uses two extra arithmetic lanes.

The existing N32 within-Rust comparisons exercise canonical integer and CRT implementations of the same Rust step. A cross-resolution consistency proof must identify both paths with a common physical evolution, including the velocity gauge, the normalization of time and fields, and the error from each division. Literal same-step equivalence would already fail on negative nonmultiples under the two division conventions. [Lean update](../lean/NSRegularity/NS/FullStep.lean) · [Rust update](https://github.com/Skyelabz210/NS-3D-solver_n32-ntt6/blob/83887a22e8c5aedbb2aad32172cf4bffdc3e62c2/arena/crates/ns_v5/src/ns3d_n32_crt_v4.rs)

For general smooth divergence-free data, a specific acceptance test for the next proof is the weak velocity equation against each smooth periodic divergence-free test field, with the correct initial trace and positive viscosity. On a refining family, establish a physically normalized energy inequality, a spatial translation or coercive gradient bound, time compactness in a negative Sobolev norm, and consistency of the discrete weak equation including its rounding, splitting and projection errors. Strong $L^2_{t,x}$ convergence of velocity then identifies the quadratic term, since
$$
 \|u_h\otimes u_h-u\otimes u\|_{L^1_{t,x}}\le
 (\|u_h\|_{L^2_{t,x}}+\|u\|_{L^2_{t,x}})\|u_h-u\|_{L^2_{t,x}}.
$$
Weak lower semicontinuity and an appropriate discrete energy inequality are also needed for a Leray–Hopf solution. A separate sufficient regularity target is a uniform $L^4(0,T;L^6(\mathbb T^3))$ **velocity** bound for each $T$. It obeys Serrin's relation $2/4+3/6=1$, whereas the ordinary energy-class bounds alone do not yield it. The unresolved point is deriving an estimate of this strength from arbitrary smooth data for the actual evolution. [Periodic problem statement](https://www.claymath.org/wp-content/uploads/2022/06/navierstokes.pdf) · [Serrin criterion and weak solutions](https://doi.org/10.1007/s00028-020-00645-3)

## Reproduction

From the NS-Regularity checkout run `python3 scripts/verify_rounded_ns_step.py`, `python3 scripts/verify_centered_inverse.py` and `cd lean && lake build`. In the pinned Theory-of-the-One checkout, from `formal/universal`, run `lake build +StarLift +UniversalLift` followed by `lake env lean /absolute/path/to/NSObservations.lean` from this repository's `consolidated/bridges/theory_observations` directory (pass the file's full path while keeping the Theory package as working directory).

The November 23 checks used NS-Regularity base `bcc3587`, the Theory revision linked above, and Rust base `83887a2`. The Python fixtures exhaust 13,375 bounded stencil cases. The Rust checks passed both existing v4 comparisons, the new signed-shear integration test and all ten tests in the separate six-prime NTT module.

At the pinned Rust revision, from `arena`, the targeted check is `RUST_MIN_STACK=67108864 cargo test --release -p ns_regularity --lib ns3d_n32_crt_v4`. Cargo 1.100.0 required local lockfile resolution for that checkout.
