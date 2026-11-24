#!/usr/bin/env python3
"""Exact rational reference for the curl-range-projected split on even tori."""
from fractions import Fraction
from itertools import product
from math import lcm

from verify_centered_inverse import curl, green_kernel, raw_curl_green, solenoidal
from verify_rounded_ns_step import adv, check, lap, step


def velocity(w, n, kernel, mean=(0, 0, 0)):
    numerator, denominator = raw_curl_green(w, n, kernel)
    return {x: tuple(Fraction(2*a, n*denominator) + mean[j] for j, a in enumerate(v))
            for x, v in numerator.items()}


def physical_curl(u, n):
    return {x: tuple(Fraction(n*a, 2) for a in row) for x, row in curl(u, n).items()}


def projection(w, n, kernel):
    return physical_curl(velocity(w, n, kernel), n)


def components(field):
    return tuple({x: v[j] for x, v in field.items()}.__getitem__ for j in range(3))


def unprojected_step(w, u, n, tau, nu):
    wc = components(w)
    uc = components(u)
    diffused = {x: tuple(f(x) + tau*nu*n*n*lap(f, x, n) for f in wc) for x in w}
    fc = components(diffused)
    return {x: tuple(fc[j](x) + tau*Fraction(n, 2)*(adv(wc, uc[j], x, n) - adv(uc, fc[j], x, n))
                     for j in range(3)) for x in w}


def projected_step(w, n, tau, nu, kernel, mean):
    u = velocity(w, n, kernel, mean)
    return projection(unprojected_step(w, u, n, tau, nu), n, kernel)


def integer_witness(w, n, kernel, mean):
    u = velocity(w, n, kernel, mean)
    scale_u = n * lcm(*(a.denominator for row in u.values() for a in row))
    scale_w = 2*(scale_u//n)
    uint = {x: tuple(int(scale_u*a) for a in values) for x, values in u.items()}
    wint = curl(uint, n)
    check(solenoidal(uint, n), "integer velocity witness must have zero divergence")
    check(all(tuple(Fraction(a, scale_w) for a in row) == w[x] for x, row in wint.items()),
          "integer velocity must recover the exact physical vorticity")


def verify():
    n = 8
    kernel = green_kernel(n)
    sites = list(product(range(n), repeat=3))
    wave = (0, 32, 0, -32)
    raw_velocity = {x: (wave[x[1] % 4], wave[x[2] % 4], 0) for x in sites}
    w = physical_curl(raw_velocity, n)
    # Here A, S, dt_latt, and nu_scaled are one, hence B=2/N.
    exact = unprojected_step(w, raw_velocity, n, Fraction(1, n*n), Fraction(1, n*n))
    _, original, _, _ = step(components(raw_velocity), 1, 1, 1, n)
    check(all(exact[x] == tuple(Fraction(n*field[x], 2) for field in original) for x in sites),
          "exact-division witness must match the legacy formula at every cell")
    check(not solenoidal(exact, n), "legacy result must violate the curl constraint")
    repaired = projection(exact, n, kernel)
    check(repaired != exact and projection(repaired, n, kernel) == repaired,
          "projection must remove the defect and fix its own output")
    removed = {x: tuple(a-b for a, b in zip(exact[x], repaired[x])) for x in sites}
    check(not any(a for row in curl(removed, n).values() for a in row),
          "the removed field must be orthogonal to centered curls")
    integer_witness(repaired, n, kernel, (0, 0, 0))
    print("N=8: exact legacy divergence defect and projected integer preimage verified")

    n = 4
    kernel = green_kernel(n)
    sites = list(product(range(n), repeat=3))
    potential = {x: tuple((x[j]**2 + x[(j+1)%3]*x[(j+2)%3]) % 7 - 3 for j in range(3)) for x in sites}
    w = physical_curl(curl(potential, n), n)
    mean = (Fraction(2, 3), Fraction(-1, 7), 0)
    tau, nu = Fraction(1, 13*n*n), Fraction(1, 3)
    zero = {x: (0, 0, 0) for x in sites}
    for _ in range(3):
        next_w = projected_step(w, n, tau, nu, kernel, mean)
        heat = unprojected_step(w, zero, n, tau, nu)
        check(next_w != heat, "fixture must have nonzero projected nonlinear transport")
        integer_witness(next_w, n, kernel, mean)
        w = next_w
    print("N=4: three rational projected nonlinear steps retained exact integer preimages")


if __name__ == "__main__":
    verify()
