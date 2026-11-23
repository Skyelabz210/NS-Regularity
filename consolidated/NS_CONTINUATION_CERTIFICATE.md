# Exact residual certificates for continuum Navier–Stokes

The rational velocity scheme can supply a candidate trajectory on the unit periodic three-torus. The certificate below checks a continuum trajectory constructed from that candidate. It proves existence, uniqueness and an error bound on each accepted interval without assuming a smooth continuum reference there. If the final gradient enclosure meets the [global small-data criterion](NS_GAP_ANALYSIS.md#refined-small-data-condition), it certifies global smoothness for that initial datum. The exact fixture starts above the direct small-data threshold, evolves a genuinely nonlinear datum, and certifies a final gradient below that threshold.

## Fourier norms and continuum residual

Let $k\in\mathbb Z^3$ index the basis $e^{2\pi i k\cdot x}$. Write $A_s$ for the space with norm

$$
\|f\|_{A_s}=\sum_k (1+|k|_1)^s\,|\widehat f(k)|_2.
$$

Use $s=3$, normalized torus measure, and the Euclidean norm for each complex vector coefficient. The Leray projector has matrix $P_k=I-kk^\top/|k|_2^2$ when $k\ne0$ and $P_0=I$. Define $B(a,b)=P\nabla\cdot(a\otimes b)$, with Fourier coefficients

$$
\widehat{B(a,b)}(k)=2\pi i P_k\sum_{p+q=k} (k\cdot\widehat a(p))\widehat b(q).
$$

On solenoidal fields this represents $P((a\cdot\nabla)b)$. For a time-polynomial solenoidal Fourier candidate $v$, compute the continuum residual

$$
r=v_t-\nu\Delta v+B(v,v).
$$

Retain every convolution frequency $p+q$, including modes absent from the candidate. This accounts for discarded modes and aliasing without requiring a discrete Fourier stability assumption. The exact continuum pressure projection differs from the centered grid projection. In lifting a trajectory from the rational solver, select symmetric Fourier modes, project each coefficient with $P_k$, and rationally enclose any conversions involving sample roots of unity. The initial error bound must compare the lifted field with the prescribed continuum datum. On each slab the following proof uses the continuum residual of the lift, so the choice of discrete solver, sampling precision, and interpolation affects only those measured defects.

## Slab test

Let $v$ be smooth in space and continuously differentiable in time on a slab $[t_0,t_0+\delta]$ with $\delta>0$, or continuous and piecewise continuously differentiable with the residual bound holding almost everywhere. Assume

$$
\|u(t_0)-v(t_0)\|_{A_3}\le\eta,\qquad
\sup_t\|v(t)\|_{A_3}\le U,\qquad
\operatorname*{ess\,sup}_t\|r(t)\|_{A_3}\le R.
$$

At the first slab $u(t_0)$ is the given smooth datum. Choose $a,\rho>0$ and check the three scalar inequalities

$$
 a^2\nu\ge\delta,\qquad
 \eta+\delta R+a(2U\rho+\rho^2)\le\rho,\qquad
 2a(U+\rho)<1.\tag{1}
$$

If (1) holds, the exact solenoidal solution exists on the whole slab and remains within $\rho$ of $v$ in $A_3$. The second and third inequalities are rational tests when the input bounds are rational.

To prove this, let $S(t)=e^{\nu t\Delta}$. The Fourier weight is submultiplicative because $1+|p+q|_1\le(1+|p|_1)(1+|q|_1)$. The projector is contractive at each mode. The Fourier multiplier for $S(t)B$ is therefore bounded by $2\pi|k|_2e^{-\nu t(2\pi|k|_2)^2}$. Its supremum over radial frequencies is $(2e\nu t)^{-1/2}\le(2\sqrt{\nu t})^{-1}$. Convolution and integration yield

$$
\|S(t)B(f,g)\|_{A_3}\le\frac{\|f\|_{A_3}\|g\|_{A_3}}{2\sqrt{\nu t}},\qquad
\int_0^\delta\frac{dt}{2\sqrt{\nu t}}=\sqrt{\delta/\nu}\le a.
$$

The mild equation for the error $w=u-v$ is

$$
w(t) = S(t-t_0)w(t_0)-\int_{t_0}^t S(t-s)\left(r+B(v,w)+B(w,v)+B(w,w)\right)(s)\,ds.
$$

Use the closed radius-$\rho$ ball in $C([t_0,t_0+\delta];A_3)$ of real solenoidal fields. The residual is integrable and the nonlinear kernel is integrable at zero. Strong continuity of the heat operator and truncation of the final part of the convolution integral give continuity of the operator on this ball. Heat contractivity, the bilinear estimate, and the middle inequality in (1) map the ball into itself. For two errors $w,z$, expand $B(w,w)-B(z,z)=B(w-z,w)+B(z,w-z)$. Their images differ by at most $2a(U+\rho)$ times the supremum of their distance. The last inequality in (1) proves contraction. Its fixed point yields the mild Navier–Stokes solution and the error enclosure.

The space $A_3$ embeds continuously into $H^3$, and the enclosed gradient is bounded in space and time. Standard local strong theory and uniqueness identify this solution from the smooth initial datum. Weak–strong uniqueness also follows from the difference energy estimate with $\|\nabla u\|_\infty$ as the coefficient in Grönwall. The mild solution smooths on the open time interval. One way to see the spatial smoothing is to map $B$ from $A_s\times A_s$ to $A_{s-1}$ for $s\ge1$ and gain $1+\alpha$ derivatives by heat smoothing with the integrable kernel $t^{-(1+\alpha)/2}$, for any $0<\alpha<1$. Restart the mild formula at successively later interior points to bootstrap all spatial derivatives on compact subintervals. The projected equation gives temporal regularity. Standard local theory at the final $H^3$ state extends through the endpoint.

## Slab chaining and the terminal test

If consecutive candidates agree at their common time, use the previous radius as the next $\eta$. If they differ, add a certified $A_3$ bound on the jump. A finite chain with all tests accepted certifies the union of the slabs.

The weighted Fourier error also bounds the physical gradient. For $m=|k|_1\ge1$, $(1+m)^3\ge8m$, since $(1+m)^2\ge4m$ and $1+m\ge2$. Thus

$$
 \|\nabla(u-v)\|_2\le \sum_k 2\pi |k|_2 |\widehat{u-v}(k)|_2\le \frac{2\pi}{8}\rho.
$$

Consequently any certified upper bound $G$ on $\|\nabla v(T)\|_2$ gives global continuation as soon as $9(G+2\pi\rho/8)<2\nu$. The written small-data result applies from the certified smooth endpoint. For a finite Fourier candidate, Parseval computes $G^2/(2\pi)^2=\sum_k|k|_2^2|\widehat v(T)(k)|_2^2$ exactly, and the terminal comparison can use a rational upper bound on $2\pi$, a positive slack, and a squared inequality. An inconclusive test only leaves that datum or interval uncertified.

## Scope of a successful certificate chain

If a smooth solution exists on a fixed compact time interval, rational trigonometric time-polynomial candidates can approximate it with arbitrarily small initial defect and residual in $A_3$. Fourier truncate the solution uniformly in $C^1_t A_5$ and approximate the finitely many time coefficients and their first derivatives with rational polynomials. Project each coefficient with $P_k$ and impose Fourier conjugacy. The projection preserves rationality. The residual tends to zero because the time derivative converges in $A_3$, $Delta$ is bounded from $A_5$ to $A_3$, and $B$ is bounded from $A_4\times A_4$ to $A_3$ by submultiplicativity and the one-derivative Fourier multiplier. A common polynomial makes candidates continuous at the slab endpoints.

Choose finitely many short slabs and a common upper radius $\rho_* > 0$ such that $a(2U+2\rho_*)\le1/4$ on each slab. This is possible because $a$ can be taken arbitrarily close to $\sqrt{\delta/\nu}$ and the candidate norm is uniformly bounded. If there are $m$ slabs, set $\rho_j=2^{j-m}\rho_*$ for $1\le j\le m$. Require the initial defect and each $\delta R$ to be at most $\rho_1/8$. The nonlinear contribution is at most $\rho_j/4$. Each inherited defect is at most $\rho_j/2$. These strict margins imply (1), and certified rational bounds can be obtained by polynomial interval subdivision and by tightening the enclosure of $2\pi$. This argument describes existence of a finite certificate on any known smooth interval. The supplied verifier implements one explicit chain.

If a smooth unforced solution exists globally, the energy inequality gives a finite integral of $\|\nabla u\|_2^2$ over positive time. There is a finite time where $\|\nabla u\|_2<2\nu/9$ with strict slack. Choose the error radii and terminal gradient approximation to fit in that slack. Hence global smoothness for a specified smooth datum is equivalent to the existence of a finite accepted chain reaching the terminal condition. The chain for the fixture gives this finite witness. Constructing such a witness for every smooth datum remains open.

## Exact fixture

Run `python3 scripts/verify_continuation.py`. The verifier uses Gaussian rational time-polynomial coefficients in local time $\theta=(t-t_0)/\delta\in[0,1]$. It checks Fourier conjugacy and exact solenoidality. For every time monomial it uses $|\theta^j|\le1$ and bounds each vector coefficient by the sum of absolute real and imaginary components. All estimates accumulate over the full product support. The rational approximation $q$ to $2\pi$ with error $\epsilon$ is certified using Machin's identity and alternating rational arctangent sums. The residual calculation at $q$ adds $\epsilon$ times the normalized convection bound and $\nu\epsilon(2q+\epsilon)$ times the normalized Laplacian bound. This keeps the small residual cancellation before accounting for the uncertainty in $2\pi$.

The datum is

$$
 u_0(x,y,z)=\frac1{33}\left(\sin(2\pi z),\sin(2\pi x),\sin(2\pi y)\right),\qquad \nu=1.
$$

All three components participate in nonzero projected convection. The initial gradient norm is $(2\pi/33)\sqrt{3/2}>2/9$, which exceeds the direct small-data threshold. Eight piecewise-linear Fourier candidates use a rational forward Euler slope truncated to $|k|_1\le2$. Every slab uses the exact continuum residual verification above, including all frequencies generated by convection. The final test encloses the error in $A_3$ within $2/3125$, proves $\|\nabla u(1/512)\|_2<2/9$, and certifies global smoothness for this specific datum. The fixture is an explicit low-mode validation case. The result for arbitrary initial data requires a certificate for each chosen datum, with no presumption that any particular chain succeeds.

The general a posteriori method has precedents in [Chernyshenko, Constantin, Robinson and Titi](https://arxiv.org/abs/math/0607181) and [Morosi and Pizzocchero](https://arxiv.org/abs/1104.3832). This proof uses a weighted Fourier norm, viscosity in the mild equation, and rational interval bounds for the specific benchmark.

## Certificate from the actual mock theta grid step

The same cyclic continuum datum also has a certificate whose candidate comes from the [rational mock theta scheme](CRAM_MOCK_THETA_TRANSFER.md). On the period-four grid, sample $u_0$ exactly and take one positive-viscosity nonlinear solve with the two weights $(1/4,3/16)$ at lattice step $1/256$. The solver checks its rational momentum equations, divergence and energy identity. Its output has a 34-bit common denominator and nonzero modes with $|k|_1=2$ generated by transport.

Encode the resulting velocity at the fixed physical scale $A=2^{24}$ with the [solenoidal encoder](BOUNDED_TRANSPORT.md). It preserves the mean and centered divergence, and its squared discrete $L^2$ error is bounded by $15/A^2$. Verify the velocity numerators and their raw integer curl against a signed window $[-A,A]$ and all six squared safe-basis lanes. The effective modulus $30030^2$ exceeds $2A$. The lane-wise curl matches the integer curl and an altered representative fails validation. The initial endpoint uses the exact datum. The final endpoint uses the stored, checked field.

Apply the Gaussian-rational discrete Fourier transform at both endpoints. Retain $k\in\{-1,0,1\}^3$ with $|k|_1\le2$, apply the continuum projector, and interpolate the two lifts linearly over physical time $[0,1/512]$. The initial lift is exactly $u_0$. The coarse clock choice uses the retained-mode viscosity symbol $\lambda_{\omega}(k)=20|k|_2^2$ at $N=4$. Mapping the lattice step to half its width in physical time makes the semidiscrete rate $40|k|_2^2$, close to $(2\pi)^2|k|_2^2$. The continuum residual test uses the actual projected lift, time slope, convection including its full convolution support, and exact enclosures for $2\pi$. It accounts for the discrepancy introduced by the clock calibration, projection, fixed-scale encoding and omitted modes.

Four equal scalar certificate subslabs, with no further grid solves, bound the final $A_3$ error by $1823/500000$. The terminal inequality proves $\|\nabla u(1/512)\|_2<2/9$. Thus the global continuation test passes from a candidate generated by the actual nonlinear mock theta solver and then encoded in fixed-scale CRAM, starting from a datum above the direct small-data bound. This is a certified result for one explicit datum and calibrated interval. The finite-slab theorem sets the conditions for checking other candidates.

Run `python3 scripts/verify_mock_theta_continuation.py` from the repository root. The computation uses fractions and Gaussian-rational coefficients throughout.
