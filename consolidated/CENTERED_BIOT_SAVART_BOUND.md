# A uniform Biot–Savart velocity bound for the even periodic centered grid

Let $N=2m\ge4$ on the unit three-torus, $b=\lfloor m/2\rfloor$, and $D_j u(x)=m[u(x+e_j)-u(x-e_j)]$. The norm below is the largest absolute component over all sites. On the range of the centered curl, choose the real divergence-free inverse with zero projection onto the eight parity classes' constant modes. Then
$$
 \|u\|_\infty\le \frac{2b(m-b)}{m^2}\|\omega\|_\infty
 \le \frac12\|\omega\|_\infty.
$$
The same $1/2$ velocity bound holds for the mean-zero continuum Biot–Savart inverse with smooth data on the unit torus. For the repository's **unscaled integer** centered curl at $N=8$, there exists a divergence-free integer right inverse on the representable domain with $\|u\|_\infty\le 2\|\omega\|_\infty$. Below are written proofs and an exact reference algorithm. Lean checks the minimum-norm selector, the 64 parity-grid Green equations at $N=8$, all three derivative budgets and the universal bound on the sum of two weighted rows. The all-resolution heat argument and its assembly into the integer right inverse are written proofs.

## The one-dimensional kernel

Set $A=-\sum_j D_j^2$. Its restriction to each parity class is the ordinary positive periodic graph Laplacian on an $m^3$ grid multiplied by $m^2$. In one coordinate let $p_t(r)$ be the probability kernel on $\mathbb Z/m$ for the continuous-time walk with generator $m^2(p(r+1)+p(r-1)-2p(r))$, starting at zero. Symmetry gives $p_t(r)=p_t(-r)$. On $0\le r<b$, set $q_r=p_t(r)-p_t(r+1)$. It satisfies the same tridiagonal heat equation with ghost values $q_{-1}=-q_0$ and either $q_b=0$ for odd $m$, or $q_b=-q_{b-1}$ for even $m$. After substitution, the system has nonnegative off-diagonal entries. Its exponential is entrywise nonnegative, as follows by adding a sufficiently large multiple of the identity before expanding the exponential series. The initial differences are nonnegative, so all $q_r\ge0$. Hence
$$
 \sum_{r\bmod m}|p_t(r+1)-p_t(r)|=2[p_t(0)-p_t(b)].
$$
The walk converges exponentially to $1/m$ because the connected finite cycle Laplacian has a positive spectral gap off constants. Its centered Green function is $H(r)=\int_0^\infty(p_t(r)-1/m)\,dt$. It satisfies
$$
 m^2[2H(r)-H(r+1)-H(r-1)]={\bf1}_{r=0}-1/m.
$$
Substituting the quadratic $H(r)=H(0)-r(m-r)/(2m^3)$ verifies the equation at each node, including the periodic join. Uniqueness modulo constants follows from the sum-of-squared-edge-differences identity. Consequently
$$
 \int_0^\infty m\sum_r |p_t(r+1)-p_t(r)|\,dt
 =2m[H(0)-H(b)]=\frac{b(m-b)}{m^2}\le\frac14.
$$

## The three-dimensional inverse

Let $P$ average each scalar field separately over each parity class. The semigroup $e^{-tA}$ on each class is the product of three one-dimensional probability kernels, and $P$ is its equilibrium projection. A centered derivative on the $N$-grid subtracts two adjacent entries of the **same** input parity class. The separation between those entries is $1/m$. The other two kernel factors have mass one. The infinity operator norm of $D_j e^{-tA}$ is therefore bounded by the one-dimensional derivative row sum just integrated.

Define $u=\int_0^\infty {\rm curl}_h e^{-tA}\omega\,dt$. The derivative annihilates $P$. Each output component is a difference of two derivative contractions. The triangle inequality gives the claimed $2b(m-b)/m^2$ bound for any input. In Fourier coordinates the multiplier is $i\,s\times/|s|^2$ for $s_j=N\sin(2\pi k_j/N)$ and $s\ne0$, and zero on $s=0$. It follows that $\mathrm{div}_h u=0$ and $\mathrm{curl}_h u=\omega$ provided $P\omega=0$ and $\mathrm{div}_h\omega=0$. The kernel of simultaneous centered curl and divergence consists exactly of the vector fields constant on each parity class. An integer right inverse has an additional lattice condition below.

For a trigonometric polynomial on the continuum torus, sufficiently fine grids distinguish all its frequencies. On each fixed nonzero frequency $s_j(k)\to 2\pi k_j$, so the sampled inverse converges uniformly at the nodes to the sampled mean-zero continuum inverse. The nodal component maxima converge to the continuum maxima as the grids become dense. The common $1/2$ bound passes to this limit. Smooth periodic fields follow by Fourier truncation, since both the field and its order-minus-one inverse have absolutely convergent series. This identifies the continuum **linear inverse** on smooth data. Vortex stretching contains a derivative of velocity; this velocity estimate alone gives no stretching bound.

## An integer inverse at the original scale

Write $D_j^{\rm raw}u=u(x+e_j)-u(x-e_j)$. Since $D_j=mD_j^{\rm raw}$, the real gauge-fixed inverse $v$ of a raw vorticity $w$ satisfies
$$
 \|v\|_\infty \le \frac{2b(m-b)}{m}\|w\|_\infty .
$$
At $N=8$, $m=4$ and this coefficient equals two. If an integer solenoidal preimage $u_0$ exists, $u_0-v$ has zero curl and divergence and hence is constant on each parity class. Subtract $v(x_0)$ from $v(x)$ in the same parity class and component. Every resulting $z(x)$ is integral. Its range diameter is at most $2\|v\|_\infty$. Subtract the integer $\lfloor(\max z+\min z)/2\rfloor$ on that class. The result has component maximum at most $\lceil(\max z-\min z)/2\rceil$. Since $2\|w\|_\infty$ is an integer, this yields an integer velocity with $\|u\|_\infty\le2\|w\|_\infty$ at $N=8$. All subtractions were by parity constants, which preserve raw curl and divergence. The minimum-norm selector inherits this bound from this witness. Extending the inverse by zero outside the integer range gives a total operator satisfying the finite range, divergence and norm contracts. Equivalence with the named opaque Lean operator or the Rust NTT requires an implementation-identification theorem.

The raw constant two at $N=8$ is optimal. On the eight indices take $F=(0,0,1,0,2,0,1,0)$ and $u=(F(y)+F(z),0,0)$. It is divergence-free and its raw curl has component maximum one. On the all-even parity class the x-velocity has values zero and four. Every other solenoidal preimage differs by a constant on this class, so every inverse has component maximum at least two for these data. The reference implementation attains two. The Lean lemma `center_integer_class` checks the signed-integer midpoint argument for arbitrary endpoints.

The algorithm is constructive. Solve the rational Poisson equation on one parity grid. Apply its kernel componentwise, take the centered curl, check relative integrality, center as above, and validate the output's curl and divergence. These checks are necessary and sufficient for an integer solenoidal preimage. For example, on $N=8$ set $p=(0,0,{\bf1}_{x=0\bmod4,\ y=0\bmod4})$, $u=\mathrm{curl}_{\rm raw}p$, $w=\mathrm{curl}_{\rm raw}u/2$. The components of $w$ are integers. The real preimage $u/2$ has a half-integer velocity difference between $(0,1,0)$ and $(2,1,0)$ in the same parity class. No admissible change of gauge can remove that difference.

The [Lean N8 certificate](../lean/NSRegularity/NS/CenteredKernel8.lean) uses a Green numerator of denominator 1920. On the $4^3$ parity grid it verifies $L g=1920\delta_0-30$, where $L$ is the unscaled positive seven-point Laplacian. Each derivative row has absolute sum 1920, so the two-row curl numerator is bounded by $3840M$ for any integer inputs of magnitude at most $M$. Dividing by 1920 gives the raw factor two.

The exact rational reference checks every Green equation and derivative row norm for $N=4,6,8,12,16,32$. At $N=8$ it also checks constructive curl recovery, the raw infinity bound, constant and checkerboard obstructions, and the fractional-preimage example. Run at the project root with Python 3 using **scripts/verify_centered_inverse.py**. This is a reference realization for verification. The uniform proof above applies to every even $N\ge4$.

The same estimate applies to the Coulomb-gauge vector potential of a periodic solenoidal magnetic field on this stencil: replace vorticity by the magnetic field and velocity by its vector potential, with the same range and gauge conditions.

The finite torus heat semigroup and its product construction are standard background. The normalization, parity reduction and integer gauge are specified here for the repository operators. [Discrete torus heat kernels, Chinta, Jorgenson and Karlsson](https://www.unige.ch/~karlssob/cjktori.pdf) · [Original centered curl and divergence](../lean/NSRegularity/NS/BiotSavart.lean)
