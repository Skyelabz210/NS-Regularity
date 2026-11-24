# Navier–Stokes proof gaps and refinement

The legacy Lean split step, the N32 production solvers, and the rational velocity scheme have different update rules. This audit follows each claim back to its own scheme. The nonlinear limit and estimates below concern unforced incompressible flow on the unit periodic three-torus.


## Latest results

| Work | Verified result and scope |
|---|---|
| CRAM and mock theta transfer | The positive coefficients of third-order $\omega(q)$ define an exact two-jump viscosity stencil. Its symbol bounds, consistency, and energy inequality support a written nonlinear continuum limit with CRAM-style signed numerator certificates. [Mock theta transfer](CRAM_MOCK_THETA_TRANSFER.md) |
| Bounded-scale arithmetic | Rounding an integer vector potential and compatible orbit means retains exact centered divergence and momentum. A run on one mesh holds the stored scale fixed, and accumulated encoding error vanishes when $1/(A\tau)\to0$. [Bounded transport](BOUNDED_TRANSPORT.md) |
| Continuation with no assumed smooth reference | A rational Fourier residual test certifies existence on each accepted continuum slab. The exact cyclic three-dimensional fixture starts above the direct small-data threshold. A mock theta rational grid step with fixed-scale CRAM residue evidence supplies the candidate, and four continuum residual slabs certify the terminal threshold. [Continuation certificate](NS_CONTINUATION_CERTIFICATE.md) |
| Automated certificate construction | A Taylor predictor, bounded anchor rounding and exact radius search generate interval chains. Independent replay binds initial data, viscosity and error budget and recomputes the full residual. [Search and replay](NS_CONTINUATION_CERTIFICATE.md#automated-search-and-independent-replay) |
| Legacy Lean refinement | An exact-division example exits the curl image with divergence 16. A separate curl-range-projected split converges at order $h^2$ on known smooth intervals under its strict diffusive CFL. [Legacy split analysis](LEGACY_SPLIT_REFINEMENT.md) |
| N32 production arithmetic | A checked corridor certifies step-body roundoff on admissible data. A period-four shear exposes the production Biot–Savart normalization gap. An opt-in six-lane centered inverse returns a checked integer witness on its certified domain. [N32 draft](https://github.com/Skyelabz210/NS-3D-solver_n32-ntt6/pull/1) |

The automated search extends the reproduced global certificates to cyclic amplitudes $1/12$ and $1/8$ at viscosity one, and to a specified $A_3$ neighborhood of the $1/12$ center. An unequal-amplitude cyclic field also passes. These are data-specific certificates with independently recomputed residuals. The remaining universal obligation is to guarantee a finite accepted chain, or an equivalent a priori continuation bound, for every admitted smooth datum.

### Reused dependencies for the search

| Consumer obligation | Existing source | Use in this change | Evidence |
|---|---|---|---|
| Correct finite Fourier residual | `scripts/verify_continuation.py`, `nonlinear`, `residual_majorant` | Full time and spatial product support for every replayed interval. A degree limit is used only during Taylor prediction. | Reproduced both existing continuation fixtures and a coefficient regression. |
| Interval existence and error enclosure | [Slab theorem](NS_CONTINUATION_CERTIFICATE.md#slab-test), `check_slab` | Reuse the written contraction argument and the exact scalar check. | Source inspected and new saved chains replayed. |
| Initial source and endpoint matching | [Chaining rule](NS_CONTINUATION_CERTIFICATE.md#slab-chaining-and-the-terminal-test) | New implementation computes every jump and binds initial data, viscosity and the initial error budget during replay. | Positive and rejected mutation fixtures. |
| Global continuation from a terminal gradient | [Refined small-data condition](#refined-small-data-condition), `check_terminal` | Reused after the last accepted interval. | Exact terminal checks in each reported chain. |
| Earlier finite Lean operators | `lean/NSRegularity/NS/` | Their source and dependency boundary are retained. The search reuses the written continuum certificate. | The previous pinned build is recorded separately from the new Python executions. |

The inventory for this change covers the merged NS-Regularity and N32 repositories and the named Theory of the One adapter routes. It supplies no assertion about proof artifacts outside that inspected scope.

## Remaining criteria

| Gap | Evidence and consequence | Criterion that closes the gap |
|---|---|---|
| Fixed-grid results alone cannot establish a continuum PDE | The rational velocity scheme has a written Leray–Hopf subsequence limit for general divergence-free finite-energy data. Below, the same scheme gains a quantitative full-sequence comparison on every interval with a smooth reference solution. [Weak limit](EXACT_VELOCITY_LIMIT.md) | For either legacy solver, specify a refining family with physical scale, mesh, viscosity and time step. Prove estimates and identify the weak nonlinear equation for that family. |
| Stretching and general smoothness | On the legacy N8 split step, the component maximum can grow from 64 to 256 in one step under the diffusion CFL. In the rational velocity scheme the refined small-data threshold is $9\|\nabla u_0\|_2<2\nu$. [N8 counterexample](NS_REFINEMENT.md) | For arbitrary smooth data, derive uniform control through each finite interval in a continuation class, such as a bound on $\|u_h\|_{L^4_tL^6_x}$, or certify a sufficiently strong solution enclosure on arbitrary slabs. |
| Finite boundedness is different from smoothness | `UniformlyBoundedGridTrajectory` bounds a sum of absolute components on one mesh. The rational-velocity energy identity uses physical cell weights and the forward gradient. [Predicate and scaling](NS_REFINEMENT.md) | State transfer results in physical norms and audit that the bound is uniform in $h$ and $\tau$. |
| Integer curl, inversion, division | The shared-denominator heat family preserves the integer curl image. The nonlinear rational scheme evolves divergence-free velocity and computes its curl. The old capstone's universal divisibility under CFL forces zero scaled diffusion. [Discrete repairs](EXACT_VELOCITY_LIMIT.md) | Demonstrate a conservative, divergence-preserving production update with quantified arithmetic error and a certified residue range. Identify the production inverse's gauge and symbol on its declared domain. |
| Production Rust/Lean equivalence | Rust truncates signed division toward zero. Lean's legacy split uses floor. The signed shear fixture measures unequal results against the exact shared-denominator stencil. [N32 regression](https://github.com/Skyelabz210/NS-3D-solver_n32-ntt6/pull/1) | Bound the aggregate error from each actual division site, normalization and update ordering. Include near-blind centered modes and overflow on the selected refinement path. |
| Formalization and scale | Lean checks finite identities, comparison algebra and absorption. The present spatial estimates and nonlinear continuum arguments are written. Dense exact rational reference solves already use denominators hundreds of bits wide at tiny resolutions. [Reproduction](EXACT_VELOCITY_LIMIT.md) | Formalize the grid-to-continuum statements independently. For a scalable implementation, carry a verifiable residual and certified range and exercise non-shear tests through increasing grid sizes. |

## Refined small-data condition

Use the notation of the [velocity scheme](EXACT_VELOCITY_LIMIT.md#2-a-fully-specified-nonlinear-velocity-update). Set $X(a)=\|D^+a\|_{2,h}$ and $Q(b)=\|\Delta_h b\|_{2,h}$. The scalar Sobolev argument in that proof gives $\|f\|_{6,h}\le S\|D^+f\|_{2,h}$ for mean-zero $f$, where $S=1/4+4/\sqrt3<8/3$. Interpolation with $\|f\|_{2,h}\le\|D^+ f\|_{2,h}/4$ yields

$$\|f\|_{3,h}\le\sqrt{S/4}\,\|D^+f\|_{2,h}.$$

For each direction $j$, the outgoing and incoming arithmetic face averages of $a_j-\overline a_j$ have $L^6_h$ norm at most $S\|D^+a_j\|_{2,h}$. For $g=D_j^+b$ the vector $L^3_h$ norm obeys $\|g\|_{3,h}^2\le\sum_\ell\|g_\ell\|_{3,h}^2$ by the triangle inequality in $L^{3/2}_h$. Every component has zero mean, so $\|D_j^+b\|_{3,h}\le\sqrt{S/4}\,\|D^+D_j^+b\|_{2,h}$. Backward differences are translates. Apply Hölder to each outgoing and incoming product in the strong form of transport. Sum over $j$ using Cauchy–Schwarz, then use $\sum_j\|D^+D_j^+b\|^2_{2,h}=Q(b)^2$ by discrete Parseval. This gives

$$\|C_h(a-\bar a,b)\|_{2,h}\le S\sqrt{S/4}\,X(a)Q(b)<\tfrac94 X(a)Q(b)$$

when the norm product is positive, with the non-strict inequality also valid when it vanishes. The constant part of transport is skew-adjoint and commutes with $-\Delta_h$, so its work against $-\Delta_h b$ is zero. Therefore $|c_h(a;b,-\Delta_h b)|\le(9/4)X(a)Q(b)^2$. Testing the step with $-\Delta_h b$ and using `absorb_enstrophy_refined` gives

$$X(b)^2+X(b-a)^2+\tau\nu_h Q(b)^2\le X(a)^2\qquad\text{when } 9X(a)\le 2\nu_h.$$

The same initialization and Serrin argument as in the velocity-limit proof apply with this threshold. It permits 24 times the initial gradient of the earlier sufficient condition $108X\le\nu_h$. The rational fixture exercises genuine nonlinear transport above the earlier threshold and below the refined one. For general finite-energy data the argument for eventual regularity uses the revised sufficient starting time $81\|u_0\|_2^2/(8\nu^3)$.

## Quantitative comparison until a smooth reference loses regularity

Let $P_h$ be the orthogonal projection onto the centered-divergence-free space. For any sequence $U^n$ in that space, define its residual in physical velocity per unit time by

$$R^{n+1}=\frac{U^{n+1}-U^n}{\tau}+P_h C_h(U^n,U^{n+1})-\nu_h\Delta_h U^{n+1}.$$

Write $e^n=u_h^n-U^n$, $E_n=\|e^n\|_{2,h}$, and let $L_n=(\sum_j\|D^+_j U^{n+1}\|_{\infty,h}^2)^{1/2}$, using the Euclidean magnitude on vector values. Subtract the two steps and use bilinearity to write the transport difference as $C_h(u_h^n,e^{n+1})+C_h(e^n,U^{n+1})$. The first term does zero work against $e^{n+1}$. For the second, the strong form uses divergence-free $e^n$ to cancel the center term. An outgoing or incoming face average of $e^n_j$ has $L^2_h$ norm at most $\|e_j^n\|_{2,h}$. Thus $\|C_h(e^n,U^{n+1})\|_{2,h}\le L_n E_n$. Testing against $e^{n+1}$ and dropping nonnegative diffusion gives

$$E_{n+1}^2-E_n E_{n+1}\le\tau(L_n E_n+\|R^{n+1}\|_{2,h})E_{n+1}.$$

The same estimate holds with an additional residual norm for an inexact solenoidal numerical step. The zero-error case is immediate. Otherwise divide by $E_{n+1}$ and iterate. For $L=\max_{n<m}L_n$ the result is

$$E_m\le e^{L t_m}\Big(E_0+\tau\sum_{n<m}\|R^{n+1}\|_{2,h}\Big).$$

Lean verifies the scalar one-step implication and the finite weighted residual recursion. For a supplied rational solenoidal reference, residuals and the squared cell norms can be evaluated in exact arithmetic. Any certified upper bounds for the norms and the exponential suffice to evaluate the estimate. This yields an a posteriori velocity error bound on a fixed mesh. A certificate for continuum smoothness also needs consistency and a continuation norm.

**Smooth-reference consistency.** Let $u,p$ be a smooth solution on $[0,T]\times\mathbb T^3$ with viscosity $\nu>0$. Use $U^n=P_h S_h u(t_n)$, with $S_h$ sampling grid points. The Fourier projection has symbol $P_h(k)$ orthogonal to $s_h(k)=h^{-1}\sin(2\pi hk)$ componentwise, and is the identity when $s_h=0$. On $0<|k|\le N/4$, the centered sine symbol differs from $2\pi k$ by $O(h^2|k|^3)$ and has length bounded below by a positive multiple of $|k|$. On that set $\|P_h(k)-P(k)\|\le C h^2|k|^2$, where $P(k)$ projects onto the continuum solenoidal plane. For each continuum Fourier term there is a projected trigonometric term with the same original wavevector and grid coefficients projected by $P_h([k])$, where $[k]$ is its grid alias. The resulting smooth trigonometric-series extension of $P_h S_h u$ differs from $u$ by $O(h^2)$ in any fixed spatial derivative norm used here, also after time differentiation. To see this, sum the low-mode bound with the absolutely summable weighted Fourier coefficients of the smooth solution. For $|k|>N/4$, use the projection bound of one and smooth Fourier decay with enough derivatives to give an $O(h^2)$ tail even after multiplying by the derivative weights. The zero mode is preserved. This handles Nyquist modes and aliases without estimating a grid derivative by $h^{-1}$ times a pointwise error.

For projected samples, Taylor expansion of the six face fluxes gives $C_h(U^n,U^{n+1})=S_h\operatorname{div}(u(t_{n+1})\otimes u(t_{n+1}))+O(\tau+h^2)$ in grid $L^2$. Each arithmetic face average is its midpoint value plus a term of order $h^2$ with bounded first spatial derivative. Subtracting adjacent face fluxes and dividing by $h$ therefore preserves second-order consistency. This estimate uses the smooth extensions above and first-order temporal replacement of the advector. The backward time quotient, forward-gradient Laplacian and centered pressure gradient have local defects $O(\tau)$, $O(h^2)$ and $O(h^2)$, respectively. The discrete pressure gradient is orthogonal to the projected test space. Projecting the sampled continuum equation and using contraction of $P_h$ yields

$$\max_n\|R^{n+1}\|_{2,h}\le C_{u,p,T}(\tau+h^2+|\nu_h-\nu|).$$

In particular the whole discrete velocity family converges on every known smooth interval, for general smooth data on that interval, with matched-grid estimate

$$\max_{t_n\le T}\|u_h^n-P_h S_hu(t_n)\|_{2,h}\le C_{u,p,T}\big(E_0+\tau+h^2+|\nu_h-\nu|\big).$$

The same order compares against unprojected nodal samples because the projection error is $O(h^2)$. Piecewise-constant spatial reconstruction adds an $O(h)$ interpolation error in continuum $L^2$. The constants depend on the smooth reference. The estimate therefore gives numerical convergence wherever that reference exists and has bounded derivatives. General global regularity still requires estimates that control those intervals from the initial data.
