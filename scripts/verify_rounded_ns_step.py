#!/usr/bin/env python3
"""Exact fixtures for the periodic centered Lean split step and its observations."""

from fractions import Fraction
from itertools import product


def check(condition, message):
    if not condition:
        raise AssertionError(message)


def shift(site, axis, amount, n):
    pos = list(site)
    pos[axis] = (pos[axis] + amount) % n
    return tuple(pos)


def diff(f, x, axis, n):
    return f(shift(x, axis, 1, n)) - f(shift(x, axis, -1, n))


def lap(f, x, n):
    return sum(f(shift(x, axis, sign, n)) for axis in range(3)
               for sign in (1, -1)) - 6 * f(x)


def adv(v, f, x, n):
    return sum(v[a](x) * diff(f, x, a, n) for a in range(3))


def curl(v, n):
    def component(axis, x):
        b, c = (axis + 1) % 3, (axis + 2) % 3
        return diff(v[c], x, b, n) - diff(v[b], x, c, n)
    return tuple(lambda x, a=a: component(a, x) for a in range(3))


def step(velocity, times_viscosity, dt, scale, n):
    sites = list(product(range(n), repeat=3))
    w = curl(velocity, n)
    den = n**2 * scale**2
    check(0 < 6 * times_viscosity <= den, "positive diffusion CFL")
    check(all(sum(diff(velocity[a], x, a, n) for a in range(3)) == 0
              for x in sites), "velocity divergence")
    before, after, defects, remainders = [], [], [], []
    for a, f in enumerate(w):
        q = {x: divmod(times_viscosity * lap(f, x, n), den) for x in sites}
        g = {x: f(x) + q[x][0] for x in sites}
        values = [f(x) for x in sites]
        check(min(values) <= min(g.values()) <= max(g.values()) <= max(values),
              "rounded diffusion interval")
        loss = sum(g.values()) - sum(values)
        rem = sum(r for _, r in q.values())
        check(den * loss == -rem, "summed Euclidean division")
        # Advection of g and stretching from the initial (w,u) pair.
        h = {x: g[x] - dt * adv(velocity, g.__getitem__, x, n) // (2*n*scale)
             + dt * adv(w, velocity[a], x, n) // (2*n*scale) for x in sites}
        before.append(max(map(abs, values)))
        after.append(h)
        defects.append(loss)
        remainders.append(rem)
    return max(before), after, defects, remainders


def verify_witnesses():
    n = 8
    wave = (0, 1, 0, -1)
    zero = lambda x: 0
    layered = (lambda x: 32 * wave[x[1] % 4], lambda x: 32 * wave[x[2] % 4], zero)
    start, end, defects, _ = step(layered, 1, 1, 1, n)
    peak = max(abs(v) for field in end for v in field.values())
    check((start, peak, end[1][(0, 0, 0)], defects) == (64, 256, -256, [0, 0, 0]),
          "growth at positive diffusion with exact diffusion quotients")
    print("N=8 split step: max 64 -> 256; stretching produces -256 at the origin")

    shear = (zero, zero, lambda x: int(x[1] == 0))
    start, end, defects, remainders = step(shear, 1, 1, 1, n)
    check(sum(end[0].values()) == defects[0] < 0, "curl range lost in one step")
    print(f"N=8 shear: total x-vorticity 0 -> {defects[0]}; summed remainders {remainders[0]}")

    # A representable high-frequency shear with positive diffusion; the first
    # step erases the mode exactly. Its continuation uses velocity zero at zero.
    plane = (zero, zero, lambda x: wave[x[1] % 4] * (-1)**x[2])
    start, end, defects, _ = step(plane, 96, 1, 3, n)
    check(start == 2 and all(v == 0 for field in end for v in field.values())
          and defects == [0, 0, 0], "positive diffusion orbit")
    _, end0, _, _ = step((zero, zero, zero), 96, 1, 3, n)
    check(all(v == 0 for field in end0 for v in field.values()), "zero continuation")
    print("N=8 representable shear: max 2 -> 0 -> 0, positive viscosity")


def verify_face_flux():
    """A separate conservative scalar stencil with antisymmetric face flux."""
    for n in (4, 8, 16, 32):
        sites = list(product(range(n), repeat=3))
        # Signed values and partial quotients exercise both signs of a face.
        f = {x: (13*x[0] + 17*x[1]**2 - 23*x[2]) % 37 - 18 for x in sites}
        for c, d in ((1, 8), (3, 18)):
            g = f.copy()
            for x in sites:
                for axis in range(3):
                    y = shift(x, axis, 1, n)
                    raw = c * (f[y] - f[x])
                    flux = (1 if raw >= 0 else -1) * (abs(raw) // d)
                    g[x] += flux
                    g[y] -= flux
            check(sum(g.values()) == sum(f.values()), "antisymmetric face mass")
            check(min(f.values()) <= min(g.values()) <= max(g.values()) <= max(f.values()),
                  "face-flux maximum principle")
    print("Conservative face flux: mass and interval verified at N=4,8,16,32")


def verify_observations():
    # Each pair is generated from the same signed numerator. The quotient
    # is derived only on request. No stored winding is used.
    for d in (8, 64, 576):
        for x in (-d*(d+1)-1, -d-1, -1, 0, 1, d+1, d*(d+1)+1):
            r, a = x % d, x % (d+1)
            check((r-a) % (d+1) == (x//d) % (d+1), "adjacent quotient class")
            alias = x + d*(d+1)
            check((alias % d, alias % (d+1)) == (r, a) and alias//d != x//d,
                  "ambiguous unbounded quotient")
    # No external Fourier/transcendental functions are needed for the
    # maximum principle: exhaustive scalar tests on the attainable sums.
    count = 0
    for d in range(1, 33):
        for c in range(d//6 + 1):
            for x in range(-2, 3):
                for neighbours in range(-12, 13):
                    exact = Fraction(d*x+c*(neighbours-6*x), d)
                    rounded = x + c*(neighbours-6*x)//d
                    check(-2 <= rounded <= 2 and rounded <= exact < rounded+1, "interval")
                    count += 1
    print(f"{count} integer stencil cases; adjacent quotient and period aliases verified")


if __name__ == "__main__":
    verify_witnesses()
    verify_observations()
    verify_face_flux()
