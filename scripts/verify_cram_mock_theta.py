#!/usr/bin/env python3
"""Exact coefficient, residue and Navier–Stokes checks for the omega stencil."""
from fractions import Fraction as Q
from math import prod

from verify_exact_transport import Torus, diffusion_faces, velocity_step


def coefficients(degree, *, omega=True):
    """Expand omega or f through a fixed degree, with no evaluation of q."""
    total = [0] * (degree + 1)
    term = [1] + [0] * degree
    m = 0
    while (start := 2*m*(m+1) if omega else m*m) <= degree:
        factor = 2*m+1 if omega else m
        if factor > 0:
            sign = 1 if omega else -1
            term = [sum((r+1)*sign**r*term[k-r*factor] for r in range(k//factor+1))
                    for k in range(degree + 1)]
        for k in range(start, degree + 1):
            total[k] += term[k-start]
        m += 1
    return total


def psi_coefficients(degree):
    """Expand the third-order psi from Ramanujan's January 12, 1920 letter."""
    total = [0] * (degree + 1)
    term = [1] + [0] * degree
    for m in range(1, degree+1):
        start = m*m
        if start > degree:
            break
        factor = 2*m-1
        for k in range(factor, degree+1):
            term[k] += term[k-factor]
        for k in range(start, degree+1):
            total[k] += term[k-start]
    return total


def normalized_weights(a, q):
    weights = [value * q**j for j, value in enumerate(a) if j]
    moment = sum(j*j*weight for j, weight in enumerate(weights, 1))
    return tuple((j, weight/moment) for j, weight in enumerate(weights, 1))


def numerator(g, field, p, b):
    """Exact omega heat numerator at nu*dt/h^2 = p/b, scale multiplier 16*b."""
    return ([(16*b-42*p)*f[i] + p*sum(4*f[s[i]]+3*f[s[s[i]]]
             for s in g.plus+g.minus) for i in range(g.cells)] for f in field)


def sample_solenoidal(g):
    potential = [[(d+1)*(x*y + 2*y*z) + (d+3)*z*x*x for x,y,z in g.sites] for d in range(3)]
    potential = [[(v % 11)-5 for v in f] for f in potential]
    return g.curl(potential)


def check_numerators(weights):
    basis = (2, 3, 5, 7, 11, 13)
    modulus = prod(basis)
    for n in (4, 5, 8, 16, 32):
        g = Torus(n)
        u = sample_solenoidal(g)
        w = g.curl(u)
        hn_u = list(numerator(g, u, 1, 8))
        hn_w = list(numerator(g, w, 1, 8))
        assert g.curl(hn_u) == hn_w and not any(g.div(hn_u))
        assert all(sum(after) == 128*sum(before) for before, after in zip(w, hn_w))
        assert all(128*min(before) <= min(after) <= max(after) <= 128*max(before)
                   for before, after in zip(w, hn_w))
        assert 2*max(abs(v) for f in hn_w for v in f) < modulus
        for prime in basis:
            residues = [[v % prime for v in f] for f in w]
            modular = [[v % prime for v in f] for f in numerator(g, residues, 1, 8)]
            assert modular == [[v % prime for v in f] for f in hn_w]
        if n <= 16:
            def diss(test_weights):
                return sum(weight * sum((f[j]-f[i])**2 for f in u)
                           for i, j, weight in diffusion_faces(g, test_weights))
            nn = diss(((1, 1),))
            omega = diss(weights)
            assert nn / Q(4) <= omega <= nn
        if n % 4 == 0:
            # This mode has exact nearest-neighbour jump ratios and distance-two
            # jump ratio. Nonzero coefficients of the mode alternate ±1.
            values = [((1, 0, -1, 0)[x%4]) for x,y,z in g.sites]
            assert list(numerator(g, [values], 1, 8))[0] == [108*v for v in values]
        print(f'N={n}: exact omega stencil, centered curl and six primary residue lanes checked')


def check_implicit(weights):
    for n in (3, 4):
        g = Torus(n)
        a = [[Q(v, 8) for v in f] for f in sample_solenoidal(g)]
        nu, tau = Q(1, 8), Q(1, 16)
        b, denominator = velocity_step(g, a, nu, tau, viscous_weights=weights)
        heat, _ = velocity_step(g, a, nu, tau, advect=False, viscous_weights=weights)
        nearest, _ = velocity_step(g, a, nu, tau)
        assert b != heat and b != nearest
        print(f'N={n}: full nonlinear omega step, exact momentum, energy and curl checked ({denominator.bit_length()} denominator bits)')


def main():
    a = coefficients(12)
    assert a[:5] == [1, 2, 3, 4, 6] and all(c > 0 for c in a)
    assert coefficients(4, omega=False)[:4] == [1, 1, -2, 3]
    weights = normalized_weights(a[:3], Q(1, 2))
    assert weights == ((1, Q(1, 4)), (2, Q(3, 16)))
    assert sum(j**4*w for j,w in weights)/12 == Q(13, 48)
    check_numerators(weights)
    check_implicit(weights)
    psi = psi_coefficients(12)
    assert psi[:5] == [0, 1, 1, 1, 2] and all(c > 0 for c in psi[1:])
    letter_weights = normalized_weights(psi[:3], Q(1, 2))
    assert letter_weights == ((1, Q(1, 3)), (2, Q(1, 6)))
    assert sum(j**4*w for j,w in letter_weights)/12 == Q(1, 4)
    g = Torus(3)
    a = [[Q(v, 8) for v in f] for f in sample_solenoidal(g)]
    nu, tau = Q(1, 8), Q(1, 16)
    b, _ = velocity_step(g, a, nu, tau, viscous_weights=letter_weights)
    heat, _ = velocity_step(g, a, nu, tau, advect=False, viscous_weights=letter_weights)
    assert b != heat
    print('N=3: full nonlinear psi step from Ramanujan’s 1920 letter checked exactly')


if __name__ == '__main__':
    main()
