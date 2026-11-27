#!/usr/bin/env python3
"""Exact rational witnesses for a continuum Navier–Stokes continuation test."""

from dataclasses import dataclass
from fractions import Fraction as Q
from functools import cache


@dataclass(frozen=True)
class GaussianRational:
    re: Q = Q(0)
    im: Q = Q(0)

    def __add__(self, other):
        return GaussianRational(self.re + other.re, self.im + other.im)

    def scale(self, factor):
        return GaussianRational(factor * self.re, factor * self.im)

    def mul(self, other):
        return GaussianRational(self.re * other.re - self.im * other.im, self.re * other.im + self.im * other.re)

    def times_i(self):
        return GaussianRational(-self.im, self.re)

    def upper_abs(self):
        return abs(self.re) + abs(self.im)

    def abs_squared(self):
        return self.re**2 + self.im**2


ZERO = (GaussianRational(), GaussianRational(), GaussianRational())


def require(condition, message):
    if not condition:
        raise ArithmeticError(message)


def add_term(poly, degree, k, vector):
    key = degree, k
    combined = tuple(a + b for a, b in zip(poly.get(key, ZERO), vector))
    if combined == ZERO:
        poly.pop(key, None)
    else:
        poly[key] = combined


def dot(k, vector):
    out = GaussianRational()
    for component, coeff in zip(k, vector):
        out = out + coeff.scale(component)
    return out


def norm_squared(k):
    return sum(a*a for a in k)


def weight(k):
    return (1 + sum(map(abs, k)))**3


def scale_vector(vector, factor):
    return tuple(x.scale(factor) for x in vector)


def majorant(poly):
    # Local scaled time belongs to [0, 1], so every monomial has magnitude <= 1.
    return sum(weight(k) * sum(x.upper_abs() for x in v) for (_, k), v in poly.items())


def pressure_projection(k, vector):
    square = norm_squared(k)
    if square == 0:
        return vector
    longitudinal = dot(k, vector).scale(Q(1, square))
    return tuple(x + longitudinal.scale(-component) for component, x in zip(k, vector))


def nonlinear(poly, *, max_degree=None):
    # Multiplier i k for spatial derivative, with the common 2π left out.
    coefficients = {}
    for (d, p), a in poly.items():
        for (e, q), b in poly.items():
            if max_degree is not None and d + e > max_degree:
                continue
            k = tuple(x + y for x, y in zip(p, q))
            factor = dot(k, a).times_i()
            if factor == GaussianRational():
                continue
            add_term(coefficients, d + e, k, tuple(x.mul(factor) for x in b))
    return {key: pressure_projection(key[1], v) for key, v in coefficients.items()}


def positive_laplacian(poly):
    # Multiplier |k|², with the common (2π)² left out.
    return {key: scale_vector(v, norm_squared(key[1])) for key, v in poly.items()}


def add_scaled(target, source, factor, degree_offset=0):
    for (d, k), v in source.items():
        add_term(target, d + degree_offset, k, scale_vector(v, factor))


def validate_poly(poly):
    for (d, k), v in poly.items():
        if (type(d) is not int or d < 0 or len(k) != 3 or len(v) != 3
                or any(type(component) is not int for component in k)
                or any(not isinstance(z.re, (int, Q)) or not isinstance(z.im, (int, Q)) for z in v)
                or dot(k, v) != GaussianRational()):
            raise ValueError('Expected a solenoidal time-polynomial Fourier field')
        reverse = poly.get((d, tuple(-x for x in k)), ZERO)
        if reverse != tuple(GaussianRational(x.re, -x.im) for x in v):
            raise ValueError('Expected conjugate-symmetric Fourier coefficients')


def arctan_bounds(inv, terms):
    partial = sum(Q((-1)**j, (2*j + 1) * inv**(2*j + 1)) for j in range(terms))
    adjacent = partial + Q((-1)**terms, (2*terms + 1) * inv**(2*terms + 1))
    return min(partial, adjacent), max(partial, adjacent)


@cache
def two_pi_bounds():
    # Machin's identity 2π = 32 arctan(1/5) - 8 arctan(1/239).
    a, b = arctan_bounds(5, 12)
    c, d = arctan_bounds(239, 5)
    lo, hi = 32*a - 8*d, 32*b - 8*c
    q = Q(6283185307179586, 10**15)
    eps = Q(1, 10**15)
    require(0 < q - eps <= lo <= hi <= q + eps, "Invalid rational enclosure of 2π")
    return q, eps


def residual_majorant(poly, viscosity, step):
    q, eps = two_pi_bounds()
    projected_convection = nonlinear(poly)
    lap = positive_laplacian(poly)
    residual = {}
    for (d, k), v in poly.items():
        if d > 0:
            add_term(residual, d-1, k, scale_vector(v, Q(d) / step))
    add_scaled(residual, projected_convection, q)
    add_scaled(residual, lap, viscosity*q*q)
    diff_square = eps*(2*q + eps)
    return majorant(residual) + eps*majorant(projected_convection) + viscosity*diff_square*majorant(lap)


def slab_report(poly, viscosity, step, eta, a, radius):
    """Return recomputed norm bounds only after all interval checks pass."""
    validate_poly(poly)
    if any(not isinstance(v, (int, Q)) for v in (viscosity, step, eta, a, radius)):
        return None
    if not (viscosity > 0 and step > 0 and a >= 0 and eta >= 0 and radius > 0 and a*a*viscosity >= step):
        return None
    u = majorant(poly)
    r = residual_majorant(poly, viscosity, step)
    image = eta + step*r + a*(2*u*radius + radius*radius)
    return (u, r) if image <= radius and 2*a*(u+radius) < 1 else None


def check_slab(poly, viscosity, step, eta, a, radius):
    return slab_report(poly, viscosity, step, eta, a, radius) is not None


def euler_candidate(initial, viscosity, step, q):
    poly = dict(initial)
    add_scaled(poly, positive_laplacian(initial), -step*viscosity*q*q, degree_offset=1)
    resolved = {key: v for key, v in nonlinear(initial).items() if sum(map(abs, key[1])) <= 2}
    add_scaled(poly, resolved, -step*q, degree_offset=1)
    return poly


def end_field(poly):
    result = {}
    for (_, k), v in poly.items():
        add_term(result, 0, k, v)
    return result


def round_up(value, denominator):
    scaled = value * denominator
    return Q(-(-scaled.numerator // scaled.denominator), denominator)


def gradient_squared_without_2pi(poly):
    require(all(d == 0 for d, _ in poly), "Expected a single time slice")
    return sum(norm_squared(k) * sum(x.abs_squared() for x in v) for (_, k), v in poly.items())


def check_terminal(poly, viscosity, radius):
    validate_poly(poly)
    if (any(not isinstance(v, (int, Q)) for v in (viscosity, radius))
            or viscosity <= 0 or radius < 0 or any(d != 0 for d, _ in poly)):
        return False
    q, eps = two_pi_bounds()
    two_pi_upper = q + eps
    allowed = Q(2, 9)*viscosity - two_pi_upper*radius/8
    return allowed > 0 and two_pi_upper**2 * gradient_squared_without_2pi(poly) < allowed**2


def main():
    q, eps = two_pi_bounds()
    nu, step, a = Q(1), Q(1, 4096), Q(1, 64)
    amplitude_half = Q(1, 66)
    initial = {
        (0, (1, 0, 0)): (GaussianRational(), GaussianRational(im=-amplitude_half), GaussianRational()),
        (0, (-1, 0, 0)): (GaussianRational(), GaussianRational(im=amplitude_half), GaussianRational()),
        (0, (0, 1, 0)): (GaussianRational(), GaussianRational(), GaussianRational(im=-amplitude_half)),
        (0, (0, -1, 0)): (GaussianRational(), GaussianRational(), GaussianRational(im=amplitude_half)),
        (0, (0, 0, 1)): (GaussianRational(im=-amplitude_half), GaussianRational(), GaussianRational()),
        (0, (0, 0, -1)): (GaussianRational(im=amplitude_half), GaussianRational(), GaussianRational()),
    }
    # Genuine nonlinearity and initial data above the direct small-data threshold.
    require(majorant(nonlinear(initial)) > 0, "Expected genuine convection")
    require((q - eps)**2 * gradient_squared_without_2pi(initial) > (2*nu/9)**2,
            "Fixture must start above the small-data threshold")
    require(not check_terminal(initial, nu, Q(0)), "Unexpected initial terminal acceptance")

    radius = Q(0)
    for slab in range(1, 9):
        poly = euler_candidate(initial, nu, step, q)
        require({key: v for key, v in poly.items() if key[0] == 0} == initial, "Candidate jump")
        eta = radius
        r = residual_majorant(poly, nu, step)
        radius = round_up(Q(33, 32)*(eta + step*r), 2**40)
        require(check_slab(poly, nu, step, eta, a, radius), f"Certificate failed on slab {slab}")
        require(not check_slab(poly, nu, step, eta, a/2, radius), "Accepted an invalid smoothing bound")
        initial = end_field(poly)
        print(f'slab {slab}: upper residual {round_up(r, 10**6)}, error radius {round_up(radius, 10**6)}')

    require(check_terminal(initial, nu, radius), "Global continuation certificate failed")
    require(not check_terminal(initial, nu, Q(1)), "Accepted an excessive terminal radius")
    print(f'global continuation accepted after {8*step}; terminal gradient strictly below {2*nu/9}')


if __name__ == '__main__':
    main()
