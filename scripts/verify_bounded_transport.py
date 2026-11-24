#!/usr/bin/env python3
"""Exercise bounded-scale rational NS updates with exact centered constraints."""
from functools import cache
from itertools import product
from math import gcd

from verify_centered_inverse import curl, green_kernel, raw_curl_green
from verify_cram_mock_theta import coefficients, normalized_weights
from verify_exact_transport import Q, Torus, velocity_step

SAFE_BASIS = (2, 3, 5, 7, 11, 13)


@cache
def potential_kernel(n):
    return green_kernel(n)


def centered_integer(value):
    shifted = value + Q(1, 2)
    return shifted.numerator // shifted.denominator


def round_harmonics(values):
    """Round parity means in fixed scale while preserving their sum."""
    total = sum(values)
    if total.denominator != 1:
        raise ValueError("the mean is outside the chosen harmonic scale")
    lower = [v.numerator // v.denominator for v in values]
    missing = int(total) - sum(lower)
    order = sorted(range(len(values)), key=lambda j: values[j] - lower[j], reverse=True)
    for j in order[:missing]:
        lower[j] += 1
    assert sum(lower) == total
    return lower


def encode_velocity(grid, field, scale):
    """Encode divergence-free velocity with common scale and exact total mean."""
    if scale <= 0 or any(grid.div(field)):
        raise ValueError("the input must be divergence-free and the scale positive")
    periods = gcd(2, grid.n)
    keys = list(product(range(periods), repeat=3))
    classes = {p: [] for p in keys}
    field_at = {}
    for i, x in enumerate(grid.sites):
        classes[tuple(v % periods for v in x)].append(x)
        field_at[x] = tuple(f[i] for f in field)
    potential, den = raw_curl_green(field_at, grid.n, potential_kernel(grid.n))
    encoded_potential = {x: tuple(centered_integer(Q(scale*z, den)) for z in v)
                         for x, v in potential.items()}
    rotational = curl(encoded_potential, grid.n)
    exact_rotational = curl(potential, grid.n)
    integers = {}
    for component in range(3):
        means = [Q(sum(field_at[x][component] for x in cells), len(cells))
                 for cells in classes.values()]
        levels = round_harmonics([scale*v for v in means])
        for cells, mean, level in zip(classes.values(), means, levels):
            assert abs(mean - Q(level, scale)) < Q(1, scale)
            for x in cells:
                assert Q(exact_rotational[x][component], den) + mean == field_at[x][component]
                integers.setdefault(x, []).append(rotational[x][component] + level)
    result = [[integers[x][d] for x in grid.sites] for d in range(3)]
    assert not any(grid.div(result))
    assert all(sum(f) == Q(sum(z), scale) for f, z in zip(field, result))
    squared_error = sum((a-Q(b, scale))**2 for f,z in zip(field,result) for a,b in zip(f,z)) / grid.cells
    assert squared_error <= Q(15, scale**2)
    return result, squared_error


def matches_residues(candidate, readings, moduli, bound):
    """Check the signed representative without CRT reconstruction."""
    modulus = 1
    for d in moduli:
        if d <= 1 or gcd(modulus, d) != 1:
            raise ValueError("the lane moduli must be pairwise coprime")
        modulus *= d
    if bound < 0 or 2*bound >= modulus:
        raise ValueError("the signed range exceeds the recovery window")
    if len(readings) != len(moduli):
        raise ValueError("each lane needs a reading")
    return abs(candidate) <= bound and all(candidate % d == r for d,r in zip(moduli,readings))


def as_field(numerators, scale):
    return [[Q(v, scale) for v in f] for f in numerators]


def sample(grid):
    seed = [((j*j+2*j+1) % 7)-2 for j in range(grid.n)]
    v = [[Q(seed[y]+seed[z],3) for x,y,z in grid.sites],
         [Q(seed[x],2) for x,y,z in grid.sites], [Q(seed[y],5) for x,y,z in grid.sites]]
    return [[x-Q(sum(f),grid.cells) for x in f] for f in v]


def run():
    scale = 2**14
    lane_powers = [p**3 for p in SAFE_BASIS]
    for n in (3,4,6,8):
        grid = Torus(n)
        v = sample(grid)
        stored, _ = encode_velocity(grid,v,scale)
        print(f'N={n}: bounded encoding preserves divergence and mean; error bound verified')
        if n > 4:
            continue
        options = {}
        flow = 'nearest-neighbour viscosity'
        if n == 3:
            options['viscous_weights'] = normalized_weights(coefficients(2), Q(1, 2))
            flow = 'mock theta viscosity'
        for step in range(1,4):
            v, exact_denominator = velocity_step(grid,as_field(stored,scale),Q(1,8),Q(1,16),**options)
            stored, error2 = encode_velocity(grid,v,scale)
            assert error2 > 0
            velocity_at = {x: tuple(f[i] for f in stored) for i,x in enumerate(grid.sites)}
            vorticity = curl(velocity_at,n)
            fields = [*velocity_at.values(), *vorticity.values()]
            signed_bound = max(abs(x) for f in fields for x in f)
            for f in fields:
                for x in f:
                    residues = tuple(x % d for d in lane_powers)
                    assert matches_residues(x, residues, lane_powers, signed_bound)
                    other = x-1 if x == signed_bound else x+1
                    assert not matches_residues(other, residues, lane_powers, signed_bound)
            print(f'N={n}, {flow}, step {step}: solve scale {exact_denominator.bit_length()} bits, '
                  f'stored scale {scale.bit_length()} bits, numerator bound {signed_bound}')
    try:
        matches_residues(0,(0,0),(2,3),3)
    except ValueError:
        pass
    else:
        raise AssertionError('ambiguous window was accepted')


if __name__ == '__main__':
    run()
