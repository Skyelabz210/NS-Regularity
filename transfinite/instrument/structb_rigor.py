#!/usr/bin/env python3
"""
N3-STRUCTB rigor extension: N-independence (N=4,8,16) + pointwise Frobenius certificate.
A1: integers only.
"""
import itertools, random
random.seed(31337)

def make_ops(N):
    CELLS = list(itertools.product(range(N), repeat=3))
    def zf(): return [[[0]*N for _ in range(N)] for _ in range(N)]
    def shift(f,d,s):
        g = zf()
        for i,j,k in CELLS:
            idx=[i,j,k]; idx[d]=(idx[d]+s)%N
            g[i][j][k]=f[idx[0]][idx[1]][idx[2]]
        return g
    def dd(f,d):
        fp,fm = shift(f,d,1), shift(f,d,-1)
        g=zf()
        for i,j,k in CELLS: g[i][j][k]=fp[i][j][k]-fm[i][j][k]
        return g
    def sub(a,b):
        g=zf()
        for i,j,k in CELLS: g[i][j][k]=a[i][j][k]-b[i][j][k]
        return g
    def curl(psi):
        px,py,pz=psi
        return [sub(dd(pz,1),dd(py,2)), sub(dd(px,2),dd(pz,0)), sub(dd(py,0),dd(px,1))]
    def rand_field(lo,hi):
        f=zf()
        for i,j,k in CELLS: f[i][j][k]=random.randint(lo,hi)
        return f
    return CELLS, zf, shift, dd, curl, rand_field

def verify_at_N(N, n_seeds=3):
    CELLS, zf, shift, dd, curl, rand_field = make_ops(N)
    results = []
    for t in range(n_seeds):
        amp = [9, 99, 999][t % 3]
        w = curl([rand_field(-amp,amp) for _ in range(3)])
        u = curl([rand_field(-amp,amp) for _ in range(3)])
        # <S,w>
        Sw = 0
        G = [[dd(u[c],d) for d in range(3)] for c in range(3)]
        for i,j,k in CELLS:
            for c in range(3):
                for d in range(3):
                    Sw += w[c][i][j][k]*w[d][i][j][k]*G[c][d][i][j][k]
        # strain split
        anti = 0; sym = 0
        for i,j,k in CELLS:
            for c in range(3):
                for d in range(3):
                    g, gt = G[c][d][i][j][k], G[d][c][i][j][k]
                    sym  += w[c][i][j][k]*w[d][i][j][k]*(g+gt)
                    anti += w[c][i][j][k]*w[d][i][j][k]*(g-gt)
        # flux identity terms
        T2 = 0
        for c in range(3):
            for d in range(3):
                dwc = dd(w[c],d)
                for i,j,k in CELLS:
                    T2 += u[c][i][j][k]*w[d][i][j][k]*dwc[i][j][k]
        T3 = 0
        for c in range(3):
            for d in range(3):
                wc,wd = w[c],w[d]
                wcp,wcm = shift(wc,d,1),shift(wc,d,-1)
                wdp,wdm = shift(wd,d,1),shift(wd,d,-1)
                for i,j,k in CELLS:
                    A  = wcp[i][j][k]-wc[i][j][k]; B  = wdp[i][j][k]-wd[i][j][k]
                    Ap = wc[i][j][k]-wcm[i][j][k]; Bp = wd[i][j][k]-wdm[i][j][k]
                    T3 += u[c][i][j][k]*(A*B-Ap*Bp)
        # T1 = sum (u·w) div w  — w solenoidal => 0; verify anyway
        T1 = 0
        dv = zf()
        for d in range(3):
            dwd = dd(w[d],d)
            for i,j,k in CELLS: dv[i][j][k]+=dwd[i][j][k]
        for i,j,k in CELLS:
            s = sum(u[c][i][j][k]*w[c][i][j][k] for c in range(3))
            T1 += s*dv[i][j][k]
        id1 = (anti==0) and (2*Sw==sym)
        id2 = (Sw+T2+T3+T1==0)
        # Pointwise Frobenius certificate: (w^T E w)^2 <= ||2E||_F^2 |w|^4 / 4 ... integer form:
        # q := w^T (G+G^T) w  (= 2 w^T E w);  frob2 := sum (g+gt)^2;  |w|^2 = ww
        # certificate: q^2 <= frob2 * ww^2   (Cauchy-Schwarz |x^T M x| <= ||M||_F |x|^2)
        cert_ok = True
        worst_num, worst_den = 0, 1   # track max ratio q^2/(frob2*ww^2) as exact fraction
        for i,j,k in CELLS:
            q = 0; frob2 = 0; ww = 0
            for c in range(3):
                ww += w[c][i][j][k]*w[c][i][j][k]
                for d in range(3):
                    g, gt = G[c][d][i][j][k], G[d][c][i][j][k]
                    q += w[c][i][j][k]*w[d][i][j][k]*(g+gt)
                    frob2 += (g+gt)*(g+gt)
            lhs = q*q; rhs = frob2*ww*ww
            if lhs > rhs: cert_ok = False
            if rhs>0 and lhs*worst_den > worst_num*rhs:
                worst_num, worst_den = lhs, rhs
        results.append((id1, id2, cert_ok, T1, worst_num, worst_den))
    return results

print("="*78)
print("N-INDEPENDENCE + POINTWISE FROBENIUS CERTIFICATE (exact integers)")
print("="*78)
for N in [4, 8, 16]:
    res = verify_at_N(N, n_seeds=3 if N<16 else 2)
    for t,(id1,id2,cert,T1,wn,wd) in enumerate(res):
        from math import gcd
        g = gcd(wn,wd) if wn else 1
        print(f"N={N:>2} seed{t}: ID1={'EXACT' if id1 else 'FAIL'}  ID2={'EXACT' if id2 else 'FAIL'}  "
              f"FrobCert={'HOLDS' if cert else 'FAIL'}  T1={T1}  tightest q²/(F²|w|⁴)={wn//g}/{wd//g}")
print()
print("ID1: <S(w,u),w> = Σ w^T E_h w  (antisym annihilation, exact, all N)")
print("ID2: <S,w> + <(w·∇)w,u> + Φ_Leibniz + (u·w,div w) = 0  (exact cocycle, all N)")
