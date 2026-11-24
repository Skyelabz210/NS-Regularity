# Legacy split-step scaling, curl-range obstruction, and projected refinement

The integer split step in `NS/FullStep.lean` can leave the centered-curl range even when every division in the step is exact. Its continuum local defect can nevertheless be made second order in space and first order in time. An exact curl-range projection produces a related split method with a quantitative convergence theorem on any interval where there is a smooth periodic solution. The projection changes the update and supplies the invariant needed to apply the centered velocity inverse at the next step.

## Finite obstruction with exact division

Use the N8 layered input already defined in `NS/RoundedStep.lean`. Put $f(y)=g(y)$ equal to the period-four sequence $(0,32,0,-32)$ and set $U=(f(y),g(z),0)$. Let $D_i$ be the unnormalized centered difference and $L$ the unnormalized nearest-neighbor Laplacian. The initial vorticity is $W=\operatorname{curl}_{\rm raw}U=(-D_z g,0,-D_y f)$. Both $U$ and $W$ are centered divergence-free. Set `ns_scale`, `nu_scaled`, and `dt_latt` equal to one. The diffusion CFL is $6\le64$.

Every value of $W$ is a multiple of 64, hence every diffusion division by 64 is exact. All values of $U$ and $W$ are multiples of 16, hence all advection and stretching divisions by 16 are exact. Write $G=(I+L/64)W$. The advective term has only a z-component, $g(z)D_y G_z$, and the stretching term has only a y-component, $-D_y f D_z g$. Because $L$ commutes with centered divergence, the exact update satisfies

$$\operatorname{div}_{\rm raw}W^{\rm new}=\frac{1}{16\cdot64}(D_zg)(D_y^2Lf).$$

In this identity $Lf=-2f$. At $(0,1,0)$ the factors are 64 and 256, giving divergence $16$. A centered curl has zero centered divergence, so the output has no centered-curl preimage. The same defect occurs with an arbitrary integer solenoidal right inverse of this input. The difference from $U$ is constant on each parity class by the [centered-kernel calculation](CENTERED_BIOT_SAVART_BOUND.md). Such a difference has zero centered derivatives and leaves stretching unchanged. Its x-advection change is a parity-constant component times a function of z, and its z-advection change is a parity-constant component times a function of y. The corresponding x and z derivatives vanish even after floor division of these changes. To split off those floors, use exactness of the original advection numerators divided by 16. The y-advection term vanishes identically. This obstruction is independent of the previously established mass defect due to rounding.

[Lean exact-division and divergence fixture](../lean/NSRegularity/NS/LegacySplit.lean)

## Physically matched scales and local defect

On the unit torus set $h=1/N$ and use centered $D_h^0$ and the nearest-neighbor $\Delta_h$ scaled by $1/(2h)$ and $1/h^2$. Write $A>0$ for the integer velocity scale and $B=2hA$ for the vorticity scale. Then $u_h=U/A$, $w_h=W/B$, and the literal integer relation $W=\operatorname{curl}_{\rm raw}U$ gives $w_h=\operatorname{curl}_h^0u_h$. Write $S=\texttt{ns_scale}$. The physical parameters for the actual denominators in the source are

$$\tau=\frac{\texttt{dt_latt}\,A}{N^2 S},\qquad \nu_h=\frac{\texttt{nu_scaled}}{A N^2 S}.$$

Indeed, the raw nonlinear numerator divided by $B$ carries a factor $2hA$, whereas $L=h^2\Delta_h$. With exact quotients or rational evaluation, one normalized full step is

$$F_h(w,u)=(I-\tau A_h(u))(I+\nu_h\tau\Delta_h)w+\tau A_h(w)u,\qquad A_h(a)b=\sum_j a_j D_{h,j}^0b.$$

For integer data let $M=\max_j\|u_j\|_\infty$. Each normalized vorticity floor error in diffusion and stretching lies in $(-1/B,0]$. The subtracted advection floor adds an error in $[0,1/B)$. The centered difference of the diffusion error is bounded by $1/(2hB)$. The full normalized step therefore differs from $F_h(w,u)$ by at most $(2+3\tau M/(2h))/B$ at each component. This bound keeps the sign of each rounding error and includes the propagation of the diffusion error into advection.

For a smooth periodic solution on a fixed finite interval, write $u=\bar u+\operatorname{curl}\psi$ with a smooth periodic vector potential $\psi$ and constant spatial mean $\bar u$. At each time let $u_h^*=\bar u+\operatorname{curl}_h^0\psi$ and $w_h^*=\operatorname{curl}_h^0u_h^*$, sampled at the grid nodes. The finite differences define smooth translated functions before sampling. Their spatial derivatives of any fixed order and their time derivatives through order two approximate the matching continuum derivatives to order $h^2$ by Taylor's theorem. In particular, the defect in the normalized unrounded split is bounded by

$$\|F_h(w_h^*(t),u_h^*(t))-w_h^*(t+\tau)\|_\infty \le C\tau(h^2+\tau+|\nu_h-\nu|).$$

To see this, expand $F_h$ into one time step of vorticity diffusion, centered advection and centered stretching, plus $-\nu_h\tau^2 A_h(u_h^*)\Delta_h w_h^*$. Smoothness bounds the latter divided by $\tau^2$ uniformly, because the difference operators commute on translated smooth functions. The time difference of $w_h^*$ approximates $\partial_t w_h^*$ with error $O(\tau)$. The continuum equation then cancels the leading terms. This proof uses data in the image of the precise discrete curl rather than assuming that sampling continuum vorticity gives such data.

An exactly representable integer sample is obtained by quantizing $A\psi/(2h)$ componentwise, taking its raw centered curl, and adding an integer approximation to $A\bar u$. Call the result $U$. Its centered divergence vanishes. Set $W=\operatorname{curl}_{\rm raw}U$. Then $U/A$ and $W/B$ approximate the respective samples above with errors $O(1/A)$ and $O(1/(Ah))$. Under the diffusion CFL, $I+\nu_h\tau\Delta_h$ has nonnegative weights summing to one, and $\|D_h^0e\|_\infty\le\|e\|_\infty/h$. With $\tau=O(h^2)$ these bounds propagate the input quantization errors through $F_h$ by at most $C/(Ah)$. Together with the floor bound and the error at $t+\tau$, the normalized defect divided by $\tau$ is bounded by

$$C\left(h^2+\tau+|\nu_h-\nu|+\frac{1}{Ah\tau}\right).$$

For a concrete refining family take $N$ even, $A=N^5$, $S=mA$, `dt_latt` equal to one and `nu_scaled` the nearest integer to $\nu mA^2N^2$, with fixed integer $m>12\nu$. Then $\tau=h^2/m$, $\nu_h\to \nu$, and the displayed defect per unit time is $O(h^2)$. This is a one-step consistency result for the original floor formulas on compatible smooth reference samples. The divergence fixture shows why it cannot be iterated with the legacy inverse on its certified domain alone.

[Integer step definitions](../lean/NSRegularity/NS/FullStep.lean) · [Diffusion numerator and CFL](../lean/NSRegularity/NS/Defs.lean)

## A projected split that converges on smooth intervals

Let $\mathcal R_h=\operatorname{range}(\operatorname{curl}_h^0)$ and let $P_h$ be the orthogonal projection onto this range in the physical grid $L^2$ inner product $h^3\sum_x a(x)\cdot b(x)$. Put $u=B_hw+\bar u$, where $B_h$ is the centered divergence-free inverse with zero parity means and $\bar u$ is a prescribed constant mean. The repaired scheme is

$$w^{n+1}=P_h F_h(w^n,B_hw^n+\bar u).$$

All matrices of the difference and raw-curl operators are rational, including $h$. The range has a rational basis. Orthogonal projection can be computed by a rational Gram system and the parity-gauged inverse is uniquely specified by a rational linear system. Hence every state is rational for rational initial data, time step, viscosity and mean. A shared denominator can be chosen after each step so that $U$ is integer and $W$ is exactly its raw curl. For example, if a rational $u$ has denominator $q$, choose $A=Nq$, then $B=2q$. The computation never needs to apply the inverse to data outside $\mathcal R_h$.

The reference fields above have $B_h w_h^*=u_h^*-\bar u$ since the curl-generated part has zero parity means. If the prescribed mean is replaced in the code by a rational mean $\bar u_h$, use $B_hw_h^*+\bar u_h$ as the comparison velocity. The mean change contributes at most $C\tau|\bar u_h-\bar u|$ to the local residual and has zero centered derivative. The local residual of the projected step has the same $L^2$ bound by the contraction of $P_h$ and $P_h w_h^*(t+\tau)=w_h^*(t+\tau)$. If an inexact but range-preserving rational implementation is used, denote its projected one-step error in grid $L^2$ by $\epsilon_n$.

**Stability relative to a smooth reference.** Let $e=w-\widetilde w$ where $\widetilde w=w_h^*$ at the current time. Write $u=B_hw+\bar u$, $\widetilde u=B_h\widetilde w+\bar u$, $H_h=I+\nu_h\tau\Delta_h$, and $d=H_h e$. Subtract the two unprojected steps to obtain

$$F_h(w,u)-F_h(\widetilde w,\widetilde u)=d-\tau A_h(u)d+\tau G,$$

$$G=-A_h(B_he)H_h\widetilde w+A_h(w)B_he+A_h(e)\widetilde u.$$

Use weighted grid $L^2$ norms summed over components and derivative directions, and pointwise Euclidean velocity norms in $L^\infty$. The centered inverse satisfies $\|B_he\|_2\le\|e\|_2/4$ and $\|D_h^0 B_he\|_2=\|e\|_2$ on $\mathcal R_h$, by its Fourier symbol. The gradients of $\widetilde u$ and $H_h\widetilde w$ are uniformly bounded in $L^\infty$. Hence $\|G\|_2\le C\|e\|_2$ whenever $\|e\|_\infty\le1$. The [uniform kernel bound](CENTERED_BIOT_SAVART_BOUND.md) also gives an upper bound $M$ for $\|u\|_\infty$ independent of $h$ on this same region.

Assume $12\nu_h\tau/h^2\le 1$. The nonnegative multiplier of $H_h$ is $1-\nu_h\tau\lambda_h(k)$ with $0\le\lambda_h\le12/h^2$. Thus

$$\|d\|_2^2\le\|e\|_2^2-\nu_h\tau\|D_h^+e\|_2^2,\qquad \|D_h^0d\|_2\le\|D_h^+e\|_2.$$

Here $D_h^+$ is the forward difference. The first inequality follows from $(1-z)^2\le1-z$ for $0\le z\le1$. To estimate $d-\tau A_h(u)d$, expand its squared norm and use $\|A_h(u)d\|_2\le M\|D_h^0d\|_2$ and Young's inequality. Explicitly,

$$\|d-\tau A_h(u)d\|_2^2\le (1+2\tau M^2/\nu_h)\|e\|_2^2-(\nu_h\tau/2-\tau^2 M^2)\|D_h^+e\|_2^2.$$

For sufficiently fine meshes, $\tau M^2\le\nu_h/2$. Dropping the last term and using the bound on $G$ proves a one-step amplification of at most $1+C\tau$ in $L^2$ on the stated region. It does not require control of the computed velocity gradient in $L^\infty$.

**Convergence.** Set $E_n=\|w^n-w_h^*(t_n)\|_2$. Projection, consistency and the optional inexact-step error give

$$E_{n+1}\le (1+C\tau)E_n+C\tau(h^2+\tau+|\nu_h-\nu|)+\epsilon_n$$

while the preceding error is at most one in $L^\infty$. Choose initial data and parameters with $E_0=O(h^2)$, $|\nu_h-\nu|+|\bar u_h-\bar u|=O(h^2)$ and $\tau=O(h^2)$ under the stated CFL. Require $\sum_{n\tau\le T}\epsilon_n=O(h^2)$ if using inexact steps. Discrete Grönwall under the region assumption gives $E_n\le C_T h^2$. On the physical grid $\|e\|_\infty\le h^{-3/2}\|e\|_2$. The bound is therefore at most $C_T h^{1/2}<1$ for sufficiently fine meshes. Induction closes the region assumption at every step of the fixed smooth interval. The velocity converges at the same order in grid $L^2$ by the inverse estimate, and both reference fields approximate the continuum nodal samples at order $h^2$.

Run `python3 scripts/verify_legacy_split.py` for the exact rational reference. It matches the N8 legacy formulas at every cell, checks that the projection removes their divergence defect, and verifies the integer velocity witness after projection. At N4 it checks three nontrivial nonlinear projected steps with a nonzero rational mean.

The physical projection, stability and continuum estimates above are written proofs. The finite counterexample is checked in Lean. The projected split is a distinct method from the original floor trajectory and the semi-implicit rational velocity scheme. Its estimate assumes a smooth reference interval and does not establish arbitrary-data smoothness or a weak limit for this projected vorticity method.
