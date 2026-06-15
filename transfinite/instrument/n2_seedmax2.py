#!/usr/bin/env python3
"""
N2-SEEDMAX v2 — exact production quotient catalog at N=8.
BS output carried in Q(sqrt2): v = a + b*sqrt2, a,b exact Fractions (Kill #128 confirmed:
zeta-form (a, b, 0, -b) <=> a + b*sqrt2 since zeta - zeta^3 = sqrt2).
Exact ordering in Q(sqrt2). ZERO FLOAT.
"""
from fractions import Fraction as Fr
import itertools, random, math
random.seed(8128)
N = 8
CELLS = list(itertools.product(range(N), repeat=3))

# ---------- zeta_8 machinery (verified exact) ----------
def zmul(a,b):
    c=[Fr(0)]*7
    for i in range(4):
        if a[i]==0: continue
        for j in range(4):
            if b[j]==0: continue
            c[i+j]+=a[i]*b[j]
    out=[c[0],c[1],c[2],c[3]]
    for t in range(4,7): out[t-4]-=c[t]
    return tuple(out)
def zadd(a,b): return (a[0]+b[0],a[1]+b[1],a[2]+b[2],a[3]+b[3])
def zscale(a,s): return (a[0]*s,a[1]*s,a[2]*s,a[3]*s)
ZZERO=(Fr(0),)*4
ZPOW=[]
for j in range(8):
    v=[Fr(0)]*4
    if j<4: v[j]=Fr(1)
    else: v[j-4]=Fr(-1)
    ZPOW.append(tuple(v))
I_UNIT=ZPOW[2]

def axis_dft(line, inverse=False):
    out=[]
    for k in range(8):
        acc=ZZERO
        for x in range(8):
            e=(k*x)%8
            tw=ZPOW[e] if inverse else ZPOW[(-e)%8]
            acc=zadd(acc,zmul(line[x],tw))
        out.append(acc)
    if inverse: out=[zscale(v,Fr(1,8)) for v in out]
    return out

def dft3(g, inverse=False):
    g=[[ [g[i][j][k] for k in range(N)] for j in range(N)] for i in range(N)]
    for i in range(N):
        for j in range(N):
            t=axis_dft([g[i][j][k] for k in range(N)],inverse)
            for k in range(N): g[i][j][k]=t[k]
    for i in range(N):
        for k in range(N):
            t=axis_dft([g[i][j][k] for j in range(N)],inverse)
            for j in range(N): g[i][j][k]=t[j]
    for j in range(N):
        for k in range(N):
            t=axis_dft([g[i][j][k] for i in range(N)],inverse)
            for i in range(N): g[i][j][k]=t[i]
    return g

def kappa(k): return k if k<=3 else k-8

def bs_flat_sqrt2(w):
    """Returns u as 3 fields of (a,b) pairs meaning a + b*sqrt2."""
    wh=[dft3([[[(Fr(w[c][i][j][k]),Fr(0),Fr(0),Fr(0)) for k in range(N)] for j in range(N)] for i in range(N)]) for c in range(3)]
    uh=[[[[ZZERO]*N for _ in range(N)] for _ in range(N)] for _ in range(3)]
    uh=[[[[ZZERO for k in range(N)] for j in range(N)] for i in range(N)] for c in range(3)]
    for i,j,k in CELLS:
        kx,ky,kz=kappa(i),kappa(j),kappa(k)
        if (kx,ky,kz)==(0,0,0) or kx==-4 or ky==-4 or kz==-4: continue
        k2=kx*kx+ky*ky+kz*kz
        wx,wy,wz=wh[0][i][j][k],wh[1][i][j][k],wh[2][i][j][k]
        cx=zadd(zscale(wz,Fr(ky)),zscale(wy,Fr(-kz)))
        cy=zadd(zscale(wx,Fr(kz)),zscale(wz,Fr(-kx)))
        cz=zadd(zscale(wy,Fr(kx)),zscale(wx,Fr(-ky)))
        inv=Fr(1,k2)
        uh[0][i][j][k]=zmul(I_UNIT,zscale(cx,inv))
        uh[1][i][j][k]=zmul(I_UNIT,zscale(cy,inv))
        uh[2][i][j][k]=zmul(I_UNIT,zscale(cz,inv))
    u=[]
    sqrt2_present = False
    for c in range(3):
        uc=dft3(uh[c],inverse=True)
        fc=[[[None]*N for _ in range(N)] for _ in range(N)]
        for i,j,k in CELLS:
            v=uc[i][j][k]
            # real subfield of Q(zeta_8): v = a + b(zeta - zeta^3) = a + b*sqrt2
            assert v[2]==0 and v[1]==-v[3], f"non-real BS output at {(i,j,k)}: {v}"
            a,b = v[0], v[1]
            if b != 0: sqrt2_present = True
            fc[i][j][k]=(a,b)
        u.append(fc)
    return u, sqrt2_present

# ---------- Q(sqrt2) exact arithmetic ----------
def q2add(x,y): return (x[0]+y[0], x[1]+y[1])
def q2subq(x,y): return (x[0]-y[0], x[1]-y[1])
def q2mulint(x,n): return (x[0]*n, x[1]*n)
def q2mul(x,y): return (x[0]*y[0]+2*x[1]*y[1], x[0]*y[1]+x[1]*y[0])
Q2ZERO=(Fr(0),Fr(0))
def q2_sign(x):
    a,b=x
    if a==0 and b==0: return 0
    if a>=0 and b>=0: return 1
    if a<=0 and b<=0: return -1
    # mixed signs: a + b*sqrt2 > 0 iff a>0,b<0: a^2 > 2b^2 ; a<0,b>0: 2b^2 > a^2
    if a>0:   # b<0
        return 1 if a*a > 2*b*b else (-1 if a*a < 2*b*b else 0)
    else:     # a<0, b>0
        return 1 if 2*b*b > a*a else (-1 if 2*b*b < a*a else 0)
def q2_lt(x,y): return q2_sign(q2subq(x,y))<0

# ---------- ops ----------
def six(i,j,k,d,s):
    idx=[i,j,k]; idx[d]=(idx[d]+s)%N
    return tuple(idx)

def stretch_pair_q2(w,u):
    t=Q2ZERO
    for c in range(3):
        for d in range(3):
            for i,j,k in CELLS:
                p=six(i,j,k,d,1); m=six(i,j,k,d,-1)
                duc=q2subq(u[c][p[0]][p[1]][p[2]], u[c][m[0]][m[1]][m[2]])
                t=q2add(t, q2mulint(duc, w[c][i][j][k]*w[d][i][j][k]))
    return t

def l2sq_int(w):
    return sum(w[c][i][j][k]**2 for c in range(3) for i,j,k in CELLS)

# ---------- seeds (same families) ----------
def zint(): return [[[0]*N for _ in range(N)] for _ in range(N)]
def curl_int(psi):
    def ddi(f,d):
        g=zint()
        for i,j,k in CELLS:
            p=six(i,j,k,d,1); m=six(i,j,k,d,-1)
            g[i][j][k]=f[p[0]][p[1]][p[2]]-f[m[0]][m[1]][m[2]]
        return g
    px,py,pz=psi
    a,b=ddi(pz,1),ddi(py,2); c,d=ddi(px,2),ddi(pz,0); e,f=ddi(py,0),ddi(px,1)
    wx,wy,wz=zint(),zint(),zint()
    for i,j,k in CELLS:
        wx[i][j][k]=a[i][j][k]-b[i][j][k]
        wy[i][j][k]=c[i][j][k]-d[i][j][k]
        wz[i][j][k]=e[i][j][k]-f[i][j][k]
    return [wx,wy,wz]

TBL=[0,7,10,7,0,-7,-10,-7]; CTB=[TBL[(i+2)%N] for i in range(N)]
def seed_tg(a):
    psi=[zint(),zint(),zint()]
    for i,j,k in CELLS: psi[2][i][j][k]=a*TBL[i]*TBL[j]
    return curl_int(psi)
def seed_abc(a):
    w=[zint(),zint(),zint()]
    for i,j,k in CELLS:
        w[0][i][j][k]=a*(TBL[k]+CTB[j]); w[1][i][j][k]=a*(TBL[i]+CTB[k]); w[2][i][j][k]=a*(TBL[j]+CTB[i])
    return w
def seed_stripe(p,a):
    psi=[zint(),zint(),zint()]
    for i,j,k in CELLS:
        if i%p==0: psi[2][i][j][k]=a
        if j%p==0: psi[0][i][j][k]=a
    return curl_int(psi)
def seed_checker(a):
    psi=[zint(),zint(),zint()]
    for i,j,k in CELLS:
        s=1 if (i+j+k)%2==0 else -1
        psi[0][i][j][k]=a*s; psi[1][i][j][k]=-a*s; psi[2][i][j][k]=a*s
    return curl_int(psi)
def seed_delta(a):
    psi=[zint(),zint(),zint()]
    psi[0][3][3][3]=a; psi[1][4][4][4]=-a; psi[2][2][5][3]=a
    return curl_int(psi)
def seed_random(a):
    psi=[[[ [random.randint(-a,a) for _ in range(N)] for _ in range(N)] for _ in range(N)] for _ in range(3)]
    return curl_int(psi)
def seed_mode(kv,a):
    psi=[zint(),zint(),zint()]
    kx,ky,kz=kv
    for i,j,k in CELLS: psi[2][i][j][k]=a*TBL[(kx*i+ky*j+kz*k)%N]
    return curl_int(psi)

seeds=[
    ("TG (psi_z=sin*sin)", seed_tg(1)), ("ABC Beltrami-like", seed_abc(1)),
    ("stripe mod 2", seed_stripe(2,50)), ("stripe mod 3", seed_stripe(3,50)),
    ("stripe mod 5", seed_stripe(5,50)), ("stripe mod 7", seed_stripe(7,50)),
    ("checkerboard", seed_checker(20)), ("delta (3-point)", seed_delta(500)),
    ("mode k=(1,0,0)", seed_mode((1,0,0),1)), ("mode k=(1,1,0)", seed_mode((1,1,0),1)),
    ("mode k=(1,1,1)", seed_mode((1,1,1),1)), ("mode k=(2,1,0)", seed_mode((2,1,0),1)),
    ("mode k=(3,1,1)", seed_mode((3,1,1),1)),
    ("random #1", seed_random(9)), ("random #2", seed_random(99)), ("random #3", seed_random(999)),
]

print("="*104)
print("N2-SEEDMAX v2 — exact catalog, N=8, flat BS in Q(zeta_8) -> real subfield Q(sqrt2)")
print("="*104)
rows=[]
any_sqrt2=False
for name,w in seeds:
    Z=l2sq_int(w)
    if Z==0:
        print(f"{name:>22}: zero field, skipped"); continue
    u,s2=bs_flat_sqrt2(w)
    any_sqrt2|=s2
    P=stretch_pair_q2(w,u)                 # (a,b) in Q(sqrt2)
    # ranking value: P^2 / Z^3 in Q(sqrt2), with sign of P
    P2=q2mul(P,P)
    Rsq=(P2[0]/Fr(Z)**3, P2[1]/Fr(Z)**3)
    rows.append((Rsq,P,Z,name,s2))

# exact sort by Rsq in Q(sqrt2), descending
rows.sort(key=lambda r: 0, reverse=False)  # placeholder; do exact insertion sort
sorted_rows=[]
for r in rows:
    placed=False
    for idx,sr in enumerate(sorted_rows):
        if q2_lt(sr[0], r[0]):   # r bigger -> insert before
            sorted_rows.insert(idx,r); placed=True; break
    if not placed: sorted_rows.append(r)

print(f"\n{'rank':>4} {'seed':>22} {'sign(P)':>8} {'sqrt2?':>6}  {'R_sq = a + b*sqrt2 (exact leading digits)':>52}")
print("-"*104)
def fr_str(f, n=14):
    s=str(f.numerator); t=str(f.denominator)
    return (s[:n]+('…' if len(s)>n else ''))+'/'+(t[:n]+('…' if len(t)>n else ''))
for rank,(Rsq,P,Z,name,s2) in enumerate(sorted_rows,1):
    sg=q2_sign(P)
    sgs='+' if sg>0 else ('-' if sg<0 else '0')
    # exact integer approx of Rsq for display: floor((a+b*sqrt2)*10^12) via integer sqrt
    a,b=Rsq
    # value*10^12 = a*10^12 + b*10^12*sqrt2 ; floor via isqrt of 2*(b*10^12)^2
    A=a*10**12; B=b*10**12
    # approx = A + sign(B)*isqrt(2*B^2) computed on integers of the fraction
    An=A.numerator; Ad=A.denominator; Bn=B.numerator; Bd=B.denominator
    val = An*Bd
    s2n = math.isqrt(2*Bn*Bn)* (1 if Bn>=0 else -1)
    val = val + s2n*Ad
    den = Ad*Bd
    approx = val//den if den else 0
    print(f"{rank:>4} {name:>22} {sgs:>8} {'Y' if s2 else 'N':>6}  a={fr_str(a)}  b={fr_str(b)}  ~{approx/10**12:.6e}")

print(f"\nsqrt2 content in BS output across catalog: {'PRESENT (Kill #128 reconfirmed)' if any_sqrt2 else 'absent'}")
print("Worst-case catalog = rows with sign(P)=+ from the top.")
