#!/usr/bin/env python3
"""
N3-STRUCTB — Exact rearrangement of the stretching pairing.
A1: ALL integer arithmetic. No float anywhere.

Periodic lattice (Z/NZ)^3, central differences, h absorbed (work with raw differences;
every identity is claimed and tested at the INTEGER level, the 1/(2h) factors cancel
identically on both sides).

Operators (integer, exact):
  d_d f[x]   = f[x+e_d] - f[x-e_d]              (2h * central difference)
  Dp_d f[x]  = f[x+e_d] - f[x]                  (h * forward difference)

CANDIDATE IDENTITY 1 (strain form — pure algebra):
  <S(w,u), w> = sum_x w^T Gsym w
  where S_c = sum_d w_d * d_d u_c   and  Gsym_cd = (d_d u_c + d_c u_d)
  (factor 2 absorbed: w^T G w = w^T Gsym w / ... tested exactly below)
  The antisymmetric part of the velocity-gradient annihilates the quadratic form
  IDENTICALLY — integer algebra, no analysis.

CANDIDATE IDENTITY 2 (flux form — summation by parts + discrete Leibniz defect):
  <S(w,u), w> = -<(w·∇_h)w, u> + Phi
  Phi = sum_{x,c,d} [ Dp_d w_c * Dp_d w_d * shifted u-difference ]  (exact cubic flux)
  Derivation:
    sum_x f * d_d g = -sum_x g * d_d f                  (exact on torus, reindex)
    d_d(fg) = f d_d g + g d_d f + L_d(f,g)              (discrete Leibniz defect)
    L_d(f,g)[x] = Dp(f)[x]Dp(g)[x] - Dp(f)[x-e_d]Dp(g)[x-e_d]   (backward diff of P)
    => sum_x L_d = 0 on torus, and sum_x u*L_d telescopes onto Dp(u).
"""
import itertools, random
random.seed(20260610)

N = 8
CELLS = list(itertools.product(range(N), repeat=3))

def zf():
    return [[[0]*N for _ in range(N)] for _ in range(N)]

def shift(f, d, s):
    """f shifted by s steps along axis d (periodic)."""
    g = zf()
    for i,j,k in CELLS:
        idx = [i,j,k]; idx[d] = (idx[d] + s) % N
        g[i][j][k] = f[idx[0]][idx[1]][idx[2]]
    return g

def dd(f, d):
    """central difference (times 2h)"""
    fp, fm = shift(f,d,1), shift(f,d,-1)
    g = zf()
    for i,j,k in CELLS: g[i][j][k] = fp[i][j][k] - fm[i][j][k]
    return g

def dp(f, d):
    """forward difference (times h)"""
    fp = shift(f,d,1)
    g = zf()
    for i,j,k in CELLS: g[i][j][k] = fp[i][j][k] - f[i][j][k]
    return g

def curl(psi):
    """w = curl psi: exactly divergence-free for central differences."""
    px,py,pz = psi
    wx = sub(dd(pz,1), dd(py,2))
    wy = sub(dd(px,2), dd(pz,0))
    wz = sub(dd(py,0), dd(px,1))
    return [wx,wy,wz]

def sub(a,b):
    g = zf()
    for i,j,k in CELLS: g[i][j][k] = a[i][j][k]-b[i][j][k]
    return g

def divg(w):
    g = zf()
    for d in range(3):
        dwd = dd(w[d], d)
        for i,j,k in CELLS: g[i][j][k] += dwd[i][j][k]
    return g

def rand_field(lo=-9, hi=9):
    f = zf()
    for i,j,k in CELLS: f[i][j][k] = random.randint(lo,hi)
    return f

def rand_solenoidal():
    return curl([rand_field(), rand_field(), rand_field()])

def stretching(w, u):
    """S_c = sum_d w_d * d_d u_c  (raw integer; (2h) factors implicit)."""
    S = [zf(), zf(), zf()]
    for c in range(3):
        for d in range(3):
            duc = dd(u[c], d)
            for i,j,k in CELLS:
                S[c][i][j][k] += w[d][i][j][k] * duc[i][j][k]
    return S

def pair(a, b):
    """<a,b> = sum_x sum_c a_c b_c, exact int."""
    t = 0
    for c in range(3):
        for i,j,k in CELLS: t += a[c][i][j][k]*b[c][i][j][k]
    return t

# ───────────────────────── IDENTITY 1: STRAIN FORM ─────────────────────────
def strain_quadratic(w, u):
    """sum_x w^T G w with G_cd = d_d u_c. Also the symmetrized version."""
    G = [[dd(u[c], d) for d in range(3)] for c in range(3)]  # G[c][d] = d_d u_c
    full = 0
    sym  = 0
    anti = 0
    for i,j,k in CELLS:
        for c in range(3):
            for d in range(3):
                wc, wd = w[c][i][j][k], w[d][i][j][k]
                g  = G[c][d][i][j][k]
                gt = G[d][c][i][j][k]
                full += wc*wd*g
                # symmetric/antisymmetric split times 2 to stay integer:
                sym  += wc*wd*(g+gt)   # = 2 * w^T E w
                anti += wc*wd*(g-gt)   # = 2 * w^T A w  (should be 0 identically)
    return full, sym, anti

# ───────────────────────── IDENTITY 2: FLUX FORM ─────────────────────────
def transport_pair(w, u):
    """<(w·∇_h)w, u> = sum_{x,c,d} u_c * w_d * d_d w_c."""
    t = 0
    for c in range(3):
        for d in range(3):
            dwc = dd(w[c], d)
            for i,j,k in CELLS:
                t += u[c][i][j][k]*w[d][i][j][k]*dwc[i][j][k]
    return t

def leibniz_flux(w, u):
    """
    Exact cubic flux from the discrete Leibniz defect after summation by parts.
    Derivation gives: <S,w> + <(w·∇)w, u> - (divergence-free terms = 0) = Phi
    where Phi is assembled from telescoped defect terms. We compute the candidate:
      Phi_candidate = sum_{x,c,d} Dp_d(w_c)*Dp_d(w_d) * u_c[x+e_d]   minus variants —
    rather than guess the exact shift structure, we MEASURE the defect
      R = <S,w> + <(w·∇)w, u>
    and then verify R equals an explicitly local cubic-difference functional.
    Central-difference Leibniz: d(fg) = f dg + g df + L, where
      L[x] = f[x+e]g[x+e] + f[x-e]g[x-e] - f[x](g[x+e]+g[x-e]) - g[x](f[x+e]+f[x-e]) + 2f[x]g[x]
    Let A = Dp f[x] = f[x+e]-f[x], B = Dp g[x], A' = f[x]-f[x-e] = Dp f[x-e], B' likewise.
      L[x] = A*B - A'*B'   (verify algebraically: expand)
    """
    # measure R
    return None

def defect_local(w, u):
    """
    R = <S,w> + <(w·∇_h)w, u>  should equal  sum_{x,c,d} u_c[x] * L_d(w_c, w_d)[x]... 
    Derive sign convention by direct construction:
      sum_x u_c d_d(w_c w_d) = -sum_x (w_c w_d) d_d u_c        [exact SBP]
      d_d(w_c w_d) = w_c d_d w_d + w_d d_d w_c + L_d(w_c,w_d)
      => sum_x u_c w_c d_d w_d + sum_x u_c w_d d_d w_c + sum_x u_c L_d = -<term of S>
    Summing over c,d:
      T1 = sum u_c w_c (div-type on w) ; T2 = transport ; T3 = defect-contracted
      => T1 + T2 + T3 = -<S,w>
    With div_h w = 0 (w = curl psi): T1 = sum_x (u·w) * 0? NO — T1 = sum_{c} u_c w_c * sum_d d_d w_d
    = sum_x (u·w) (div_h w) = 0 exactly. So:
      <S,w> = -T2 - T3 = -<(w·∇)w,u> - sum_{x,c,d} u_c L_d(w_c, w_d)
    Return T3 computed locally for verification.
    """
    t3 = 0
    for c in range(3):
        for d in range(3):
            wc, wd = w[c], w[d]
            wcp, wcm = shift(wc,d,1), shift(wc,d,-1)
            wdp, wdm = shift(wd,d,1), shift(wd,d,-1)
            for i,j,k in CELLS:
                A  = wcp[i][j][k] - wc[i][j][k]      # Dp w_c [x]
                B  = wdp[i][j][k] - wd[i][j][k]      # Dp w_d [x]
                Ap = wc[i][j][k]  - wcm[i][j][k]     # Dp w_c [x-e]
                Bp = wd[i][j][k]  - wdm[i][j][k]     # Dp w_d [x-e]
                L  = A*B - Ap*Bp
                t3 += u[c][i][j][k]*L
    return t3

def div_dot(w, u):
    """T1 = sum_x (u·w)(div_h w) — zero when w solenoidal."""
    dv = divg(w)
    t = 0
    for i,j,k in CELLS:
        s = sum(u[c][i][j][k]*w[c][i][j][k] for c in range(3))
        t += s*dv[i][j][k]
    return t

# ───────────────────────────────── RUN ─────────────────────────────────
print("=" * 76)
print("N3-STRUCTB — exact integer verification, N=8, periodic, central differences")
print("=" * 76)

trials = []
# structured seeds: TG-like integer, checkerboard, delta, axis mode, random
def tg_like(amp):
    f = [zf(),zf(),zf()]
    tbl = [0, 7, 10, 7, 0, -7, -10, -7]  # integer sine-ish, exact
    ctb = [tbl[(i+2)%N] for i in range(N)]
    for i,j,k in CELLS:
        f[0][i][j][k] =  amp*tbl[i]*ctb[j]
        f[1][i][j][k] = -amp*ctb[i]*tbl[j]
    return f
def checker(amp):
    f = [zf(),zf(),zf()]
    for i,j,k in CELLS:
        s = 1 if (i+j+k)%2==0 else -1
        f[0][i][j][k] = amp*s; f[1][i][j][k] = -amp*s; f[2][i][j][k] = amp*s
    return f
def delta(amp):
    f = [zf(),zf(),zf()]
    f[0][3][3][3] = amp; f[1][4][3][3] = -amp; f[2][3][4][3] = amp
    return f

seeds = [
    ("TG-like psi amp=3",        tg_like(3)),
    ("checkerboard psi amp=5",   checker(5)),
    ("delta psi amp=100",        delta(100)),
    ("random psi #1",            [rand_field(),rand_field(),rand_field()]),
    ("random psi #2",            [rand_field(-50,50),rand_field(-50,50),rand_field(-50,50)]),
    ("random psi #3 (large)",    [rand_field(-999,999),rand_field(-999,999),rand_field(-999,999)]),
]

all_ok_1 = True
all_ok_2 = True
print(f"\n{'seed':>26} | {'<S,w>':>16} | anti(=0?) | I2 residual (=0?) | T1(=0?)")
print("-"*92)
for name, psi in seeds:
    w = curl(psi)                       # exactly solenoidal vorticity
    u = curl([rand_field(-20,20) for _ in range(3)])   # exactly solenoidal independent velocity
    Sw   = pair(stretching(w,u), w)
    full, sym, anti = strain_quadratic(w, u)
    # IDENTITY 1: full == sym/2 exactly, anti == 0 exactly, and full == <S,w>
    id1_ok = (anti == 0) and (2*full == sym) and (full == Sw)
    T2 = transport_pair(w, u)
    T3 = defect_local(w, u)
    T1 = div_dot(w, u)
    # IDENTITY 2: <S,w> = -T2 - T3 - T1·(coupling)?  Per derivation: <S,w> + T2 + T3 + T1 = 0
    resid = Sw + T2 + T3 + T1
    id2_ok = (resid == 0)
    all_ok_1 &= id1_ok
    all_ok_2 &= id2_ok
    print(f"{name:>26} | {Sw:>16} | {anti:>9} | {resid:>17} | {T1:>7}")

print("-"*92)
print(f"IDENTITY 1 (strain form, antisym annihilation): {'EXACT — all seeds' if all_ok_1 else 'FAILED'}")
print(f"IDENTITY 2 (flux form, Leibniz-defect cocycle):  {'EXACT — all seeds' if all_ok_2 else 'FAILED'}")
