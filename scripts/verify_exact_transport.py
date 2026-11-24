#!/usr/bin/env python3
"""Exact arithmetic checks for the shared-scale heat and implicit velocity schemes."""
from fractions import Fraction as Q
from itertools import product
from math import gcd, lcm


class Torus:
    def __init__(self, n):
        self.n = n
        self.sites = list(product(range(n), repeat=3))
        index = {p: i for i, p in enumerate(self.sites)}
        self.cells = len(index)
        self.plus = [[index[tuple((v + (j == d)) % n for j, v in enumerate(p))]
                      for p in self.sites] for d in range(3)]
        self.minus = [[index[tuple((v - (j == d)) % n for j, v in enumerate(p))]
                       for p in self.sites] for d in range(3)]

    def diff(self, f, d, i):
        return f[self.plus[d][i]] - f[self.minus[d][i]]

    def curl(self, u):
        return [[self.diff(u[(d+2) % 3], (d+1) % 3, i) - self.diff(u[(d+1) % 3], (d+2) % 3, i)
                 for i in range(self.cells)] for d in range(3)]

    def div(self, u):
        return [sum(self.diff(u[d], d, i) for d in range(3)) for i in range(self.cells)]

    def heat(self, u, c, d):
        return [[(d-6*c)*f[i] + c*sum(f[s[i]] for s in self.plus+self.minus)
                 for i in range(self.cells)] for f in u]


def check_heat():
    for n in (4, 6, 8, 16, 32):
        g = Torus(n)
        potential = [[((d+1)*x*y + (d+2)*y*z + x*x*z) % 11 - 5 for x,y,z in g.sites]
                     for d in range(3)]
        u = g.curl(potential)
        w = g.curl(u)
        maximum = max(abs(v) for f in w for v in f)
        scale = 1
        for _ in range(3):
            u, w = g.heat(u, 1, 8), g.heat(w, 1, 8)
            scale *= 8
            assert g.curl(u) == w and not any(g.div(u))
            assert max(abs(v) for f in w for v in f) <= scale*maximum
            assert all(sum(f) == 0 for f in w)
    b = (0,0,3,3,0,0,0,0)
    f = [b[(j+1) % 8]-b[(j-1) % 8] for j in range(8)]
    def trunc_six(v):
        return (1 if v >= 0 else -1)*(abs(v)//6)
    flux = [trunc_six(f[(j+1) % 8]-f[j]) for j in range(8)]
    after = [f[j]+flux[j]-flux[j-1] for j in range(8)]
    assert sum(after) == 0 and sum(after[::2]) == -1 and sum(after[1::2]) == 1
    print('Shared-scale diffusion: curl, divergence and maximum verified over three steps at N=4,6,8,16,32')
    print('Face rounding counterexample: even/odd line sums -1 and +1')


def exact_solve(rows):
    """Row-wise denominator clearing and integer Bareiss elimination with pivoting."""
    m = []
    for row in rows:
        common = lcm(*(v.denominator for v in row))
        ints = [int(common*v) for v in row]
        divisor = gcd(*ints) or 1
        m.append([v//divisor for v in ints])
    n = len(m)
    previous = 1
    for k in range(n-1):
        pivot_row = next(i for i in range(k,n) if m[i][k])
        m[k], m[pivot_row] = m[pivot_row], m[k]
        pivot = m[k][k]
        pivot_tail = m[k][k+1:]
        for i in range(k+1,n):
            row = m[i]
            entry = row[k]
            if entry == 0 and pivot == previous:
                continue
            row[k] = 0
            vals = [pivot*x-entry*y for x,y in zip(row[k+1:],pivot_tail)]
            quotients = [divmod(x,previous) for x in vals]
            assert all(r == 0 for _,r in quotients)
            row[k+1:] = [q for q,_ in quotients]
        previous = pivot
    x = [Q(0)]*n
    for i in reversed(range(n)):
        row = m[i]
        x[i] = (row[-1]-sum(row[j]*x[j] for j in range(i+1,n) if row[j])) / Q(row[i])
    assert all(sum(a*v for a,v in zip(row[:-1],x) if a) == row[-1] for row in rows)
    return x


def diffusion_faces(g, weights):
    """Weighted forward edges, with periodic shifts measured in cells."""
    for jump, weight in weights:
        for plus in g.plus:
            destinations = range(g.cells)
            for _ in range(jump % g.n):
                destinations = [plus[i] for i in destinations]
            for i, j in enumerate(destinations):
                yield i, j, weight


def velocity_step(g, a, nu, tau, advect=True, *, viscous_weights=((1, Q(1)),)):
    for jump, weight in viscous_weights:
        if not isinstance(jump, int) or jump < 1 or not isinstance(weight, (int, Q)) or weight < 0:
            raise ValueError("viscosity requires positive integer jumps and nonnegative rational weights")
    if sum(jump**2 * weight for jump, weight in viscous_weights) != 1:
        raise ValueError("viscosity requires unit second moment")
    cells = g.cells
    # Remove one redundant pressure row for each centered-divergence null mode.
    step = gcd(2, g.n)
    anchors = set(product(range(step),repeat=3))
    independent = [i for i,p in enumerate(g.sites) if p not in anchors]
    size = 3*cells + len(independent)
    rows = [[0]*(size+1) for _ in range(size)]
    for d in range(3):
        for i in range(cells):
            idx = d*cells+i
            rows[idx][idx] = 1
            rows[idx][-1] = a[d][i]
    if advect:
        for k, plus in enumerate(g.plus):
            for i, j in enumerate(plus):
                alpha = tau*g.n*(a[k][i]+a[k][j])/Q(4)
                for d in range(3):
                    left, right = d*cells+i, d*cells+j
                    rows[left][left] += alpha
                    rows[left][right] += alpha
                    rows[right][left] -= alpha
                    rows[right][right] -= alpha
    faces = list(diffusion_faces(g, viscous_weights))
    for i, j, weight in faces:
        beta = tau*nu*g.n**2*weight
        for d in range(3):
            left, right = d*cells+i, d*cells+j
            rows[left][left] += beta
            rows[left][right] -= beta
            rows[right][left] -= beta
            rows[right][right] += beta
    for offset,i in enumerate(independent):
        row = 3*cells+offset
        for d in range(3):
            plus,minus = d*cells+g.plus[d][i], d*cells+g.minus[d][i]
            rows[row][plus] += 1
            rows[row][minus] -= 1
            rows[plus][row] += 1
            rows[minus][row] -= 1
    values = exact_solve(rows)
    b = [values[d*cells:(d+1)*cells] for d in range(3)]
    assert not any(g.div(b))
    energy = lambda u: sum(v*v for f in u for v in f)
    jump = sum((v-w)**2 for f,h in zip(b,a) for v,w in zip(f,h))
    diss = g.n**2*sum(weight*(f[j]-f[i])**2 for f in b for i,j,weight in faces)
    assert energy(a) == energy(b) + jump + 2*tau*nu*diss
    assert all(sum(b[d])==sum(a[d]) for d in range(3))
    den = lcm(*(v.denominator for f in b for v in f))
    integer_velocity = [[int(den*v) for v in f] for f in b]
    w = g.curl(integer_velocity)
    assert not any(g.div(integer_velocity))
    assert all(Q(v,den)==z for f,h in zip(w,g.curl(b)) for v,z in zip(f,h))
    return b, den



def check_enstrophy(g, a, b, nu, tau, small=False):
    """Test the original momentum equation against the negative grid Laplacian."""
    def grad2(u):
        return Q(g.n**2,g.cells)*sum((f[j]-f[i])**2 for f in u for plus in g.plus
                                      for i,j in enumerate(plus))
    minus_lap = [[g.n**2 * (6*f[i]-sum(f[s[i]] for s in g.plus+g.minus))
                  for i in range(g.cells)] for f in b]
    assert not any(g.div(minus_lap))
    y = sum(v*v for f in minus_lap for v in f)/g.cells
    work = Q(0)
    for k,plus in enumerate(g.plus):
        for i,j in enumerate(plus):
            alpha = (a[k][i]+a[k][j])*Q(g.n,4*g.cells)
            work += alpha*sum((b[d][i]+b[d][j])*(minus_lap[d][i]-minus_lap[d][j]) for d in range(3))
    jump = [[v-z for v,z in zip(f,h)] for f,h in zip(b,a)]
    old_e,new_e,jump_e = grad2(a),grad2(b),grad2(jump)
    assert new_e-old_e+jump_e+2*tau*nu*y == -2*tau*work
    if small:
        assert 108**2*old_e > nu**2
        assert 9**2*old_e < 4*nu**2
        assert new_e+jump_e+tau*nu*y <= old_e


def check_velocity():
    for n in (3,4):
        g = Torus(n)
        seed = [((j*j+2*j+1) % 7)-2 for j in range(n)]
        a = [[Q(seed[y]+seed[z],3) for x,y,z in g.sites],
             [Q(seed[x],2) for x,y,z in g.sites], [Q(seed[y],5) for x,y,z in g.sites]]
        assert not any(g.div(a))
        nu, tau = Q(1,8), Q(1,16)
        b, den = velocity_step(g,a,nu,tau)
        check_enstrophy(g,a,b,nu,tau)
        heat,_ = velocity_step(g,a,nu,tau,advect=False)
        assert b != heat
        print(f'N={n}: nonlinear rational solve, all momentum rows, mean, energy, enstrophy and curl checked; denominator bits {den.bit_length()}')
        if n == 3:
            small = [[(v-sum(f)/g.cells)/2**8 for v in f] for f in a]
            z,_ = velocity_step(g,small,nu,tau)
            heat_small = [[(value-sum(f)/g.cells)/2**8 for value in hf] for f,hf in zip(a,heat)]
            assert z != heat_small
            check_enstrophy(g,small,z,nu,tau,small=True)
            print('N=3: nonzero rational input between the earlier and refined thresholds; enstrophy absorption verified')


if __name__ == '__main__':
    check_heat()
    check_velocity()
