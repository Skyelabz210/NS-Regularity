#!/usr/bin/env python3
"""Exact integer checks: periodic centered-curl range obstructions at N=8."""
from itertools import product

N = 8
sites = list(product(range(N), repeat=3))

def derivative(field, axis, coordinate):
    plus, minus = list(coordinate), list(coordinate)
    plus[axis] = (coordinate[axis] + 1) % N
    minus[axis] = (coordinate[axis] - 1) % N
    return field(tuple(plus)) - field(tuple(minus))

constant = lambda p: 1
checkerboard = lambda p: 1 if sum(p) % 2 == 0 else -1

assert all(derivative(constant, 0, p) == 0 for p in sites)
assert sum(constant(p) for p in sites) == N**3
assert all(derivative(checkerboard, 0, p) == 0 for p in sites)
assert sum(checkerboard(p) for p in sites) == 0
assert sum(checkerboard(p)**2 for p in sites) == N**3

for seed in range(5):
    v_y = lambda p, s=seed: (17*p[0] + 29*p[1]**2 - 31*p[2] + 13*s) % 113
    v_z = lambda p, s=seed: (23*p[0]**2 + 19*p[1] - 37*p[2] + 7*s) % 127
    curl_x = lambda p: derivative(v_z, 1, p) - derivative(v_y, 2, p)
    assert sum(curl_x(p) for p in sites) == 0
    assert sum(checkerboard(p)*curl_x(p) for p in sites) == 0

print("PASS: zero-mean and checkerboard obstructions to periodic centered-curl inversion (exact N8)")
