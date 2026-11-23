#!/usr/bin/env python3
"""Exact rational centered-curl kernel and integer range check on even tori."""
from fractions import Fraction
from itertools import product
from math import lcm

from verify_rounded_ns_step import check, shift


def solve(matrix):
    """Gauss elimination for a nonsingular augmented rational matrix."""
    size = len(matrix)
    a = [[Fraction(v) for v in row] for row in matrix]
    for j in range(size):
        pivot = next(i for i in range(j, size) if a[i][j])
        a[j], a[pivot] = a[pivot], a[j]
        a[j] = [v / a[j][j] for v in a[j]]
        for i in range(size):
            if i != j and a[i][j]:
                factor = a[i][j]
                a[i] = [v-factor*w for v, w in zip(a[i], a[j])]
    return [row[-1] for row in a]


def green_kernel(n):
    """Green function of minus the unscaled centered derivative squared.

    Its nonzero offsets are on the even parity subgrid. Fix the value at
    the origin; the additive constant disappears in every derivative.
    """
    check(n >= 4 and n % 2 == 0, "even grid")
    m = n // 2
    sites = list(product(range(m), repeat=3))

    def orbit(x):
        return tuple(sorted(min(i, m-i) for i in x))

    reps = sorted({orbit(x) for x in sites})
    index = {key: i for i, key in enumerate(reps)}
    a = [[0]*(len(reps)+1) for x in reps]
    for i, x in enumerate(reps):
        if x == (0, 0, 0):
            a[i][i] = 1
            continue
        a[i][i] = 6
        for axis in range(3):
            for sign in (1, -1):
                a[i][index[orbit(shift(x, axis, sign, m))]] -= 1
        a[i][-1] = -1
    coeff = solve(a)
    g = {x: coeff[index[orbit(x)]] / m**3 for x in sites}
    for x in sites:
        lap = 6*g[x] - sum(g[shift(x, a, s, m)] for a in range(3) for s in (1, -1))
        check(lap == int(x == (0, 0, 0)) - Fraction(1, m**3), "Poisson identity")
    b = m//2
    for axis in range(3):
        row_norm = sum(abs(g[shift(x, axis, 1, m)]-g[x]) for x in sites)
        check(row_norm == Fraction(b*(m-b), m), "unscaled derivative row norm")
    return g


def curl(v, n):
    def derivative(x, c, axis):
        return v[shift(x, axis, 1, n)][c] - v[shift(x, axis, -1, n)][c]
    return {x: tuple(derivative(x, (a+2)%3, (a+1)%3) - derivative(x, (a+1)%3, (a+2)%3)
                     for a in range(3)) for x in v}


def solenoidal(v, n):
    return all(sum(v[shift(x, a, 1, n)][a] - v[shift(x, a, -1, n)][a]
                   for a in range(3)) == 0 for x in v)


def integer_inverse(w, n, g):
    """Integer-valued, divergence-free right inverse on the exact range."""
    den = lcm(*(v.denominator for v in g.values()))
    kernel = [(tuple(2*j for j in y), int(v*den)) for y, v in g.items() if v]
    potential = {}
    for x in w:
        total = [0, 0, 0]
        for y, weight in kernel:
            source = w[tuple((i-j)%n for i, j in zip(x, y))]
            for a in range(3):
                total[a] += weight*source[a]
        potential[x] = tuple(total)
    numerator = curl(potential, n)
    out = {}
    for parity in product(range(2), repeat=3):
        coords = list(product(*(range(p, n, 2) for p in parity)))
        relative = {}
        for x in coords:
            v = tuple(u-b for u, b in zip(numerator[x], numerator[parity]))
            if any(a % den for a in v):
                return None
            relative[x] = tuple(a//den for a in v)
        mid = tuple((min(v[a] for v in relative.values()) +
                     max(v[a] for v in relative.values()))//2 for a in range(3))
        out.update({x: tuple(a-b for a, b in zip(v, mid)) for x, v in relative.items()})
    return out if solenoidal(out, n) and curl(out, n) == w else None


def main():
    for n in (4, 6, 8, 12, 16, 32):
        g = green_kernel(n)
        m, b = n//2, n//4
        print(f"N={n}: physical curl row bound {Fraction(2*b*(m-b), m*m)}")
        if n == 8:
            g8 = g
    n = 8
    sites = list(product(range(n), repeat=3))
    potential = {x: tuple((x[a]*x[(a+1)%3] + 3*x[(a+2)%3]**2) % 7 - 3
                          for a in range(3)) for x in sites}
    u = curl(potential, n)
    w = curl(u, n)
    recovered = integer_inverse(w, n, g8)
    check(recovered is not None and curl(recovered, n) == w, "integer right inverse")
    norm = lambda field: max(abs(a) for v in field.values() for a in v)
    check(norm(recovered) <= 2*norm(w), "raw N8 infinity estimate")
    # The norm-2 constant is attained on integer curl-image data.
    tent = (0, 0, 1, 0, 2, 0, 1, 0)
    velocity = {x: (tent[x[1]]+tent[x[2]], 0, 0) for x in sites}
    vorticity = curl(velocity, n)
    optimal = integer_inverse(vorticity, n, g8)
    check(solenoidal(velocity, n) and norm(vorticity) == 1, "extremizing source")
    check(velocity[(0, 4, 4)][0] - velocity[(0, 0, 0)][0] == 4,
          "diameter four on one parity class")
    check(optimal is not None and norm(optimal) == 2, "sharp raw constant")
    for bad in ({x: (1, 0, 0) for x in sites},
                {x: ((-1)**x[1], 0, 0) for x in sites}):
        check(integer_inverse(bad, n, g8) is None, "blind-mode rejection")

    # A real curl whose unique velocity modulo parity constants has a
    # half-integer difference inside a parity class.
    pulse = {x: (0, 0, int(x[0]%4 == 0 and x[1]%4 == 0)) for x in sites}
    u = curl(pulse, n)
    w = curl(u, n)
    check(all(a%2 == 0 for v in w.values() for a in v), "integral half-curl")
    check((u[(0, 1, 0)][0]-u[(2, 1, 0)][0])%2 == 1, "fractional relative velocity")
    half = {x: tuple(a//2 for a in v) for x, v in w.items()}
    check(integer_inverse(half, n, g8) is None, "integer range stricter than real range")
    print("N=8: integer right inverse, sharp norm bound, blind modes and fractional obstruction verified")


if __name__ == "__main__":
    main()
