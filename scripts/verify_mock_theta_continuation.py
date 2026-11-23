#!/usr/bin/env python3
"""Check a continuum certificate sourced from the exact period-four omega step."""

from fractions import Fraction as Q
from itertools import product

from verify_bounded_transport import SAFE_BASIS, as_field, encode_velocity, matches_residues
from verify_continuation import (
    GaussianRational, ZERO, add_scaled, add_term, check_slab, check_terminal,
    end_field, gradient_squared_without_2pi, majorant, nonlinear,
    pressure_projection, require, residual_majorant, round_up, two_pi_bounds,
)
from verify_cram_mock_theta import coefficients, normalized_weights
from verify_exact_transport import Torus, velocity_step


def lift_n4(g, velocity):
    """Gaussian-rational DFT, low-mode selection, and continuum projection."""
    require(g.n == 4, 'The phase table requires a period-four lattice')
    phases = (GaussianRational(1), GaussianRational(im=-1),
              GaussianRational(-1), GaussianRational(im=1))
    field = {}
    for k in product((-1, 0, 1), repeat=3):
        if sum(map(abs, k)) > 2:
            continue
        sampled_phases = [phases[sum(a*b for a, b in zip(k, position)) % 4] for position in g.sites]
        vector = []
        for component in velocity:
            coefficient = GaussianRational()
            for value, phase in zip(component, sampled_phases):
                coefficient = coefficient + phase.scale(value)
            vector.append(coefficient.scale(Q(1, g.cells)))
        add_term(field, 0, k, pressure_projection(k, tuple(vector)))
    return field


def expected_initial(amplitude):
    field = {}
    for spatial_axis, velocity_axis in ((2, 0), (0, 1), (1, 2)):
        for sign in (-1, 1):
            k = tuple(sign if axis == spatial_axis else 0 for axis in range(3))
            coefficient = GaussianRational(im=-sign * amplitude/2)
            vector = tuple(coefficient if axis == velocity_axis else ZERO[axis] for axis in range(3))
            add_term(field, 0, k, vector)
    return field


def bounded_grid_output(grid, velocity):
    """Check the stored velocity and its curl against a fixed signed window."""
    scale = 2**24
    numerators, error_squared = encode_velocity(grid, velocity, scale)
    require(0 < error_squared <= Q(15, scale**2), 'Unexpected fixed-scale encoding error')
    require(not any(grid.div(numerators)), 'Encoding lost centered divergence')
    curl_numerators = grid.curl(numerators)
    moduli = tuple(p*p for p in SAFE_BASIS)
    bound = scale
    for modulus in moduli:
        velocity_readings = [[n % modulus for n in row] for row in numerators]
        curl_readings = [[n % modulus for n in row] for row in grid.curl(velocity_readings)]
        require(curl_readings == [[n % modulus for n in row] for row in curl_numerators],
                'The modular curl disagrees with the signed integer curl')
    for row in numerators + curl_numerators:
        for numerator in row:
            evidence = tuple(numerator % modulus for modulus in moduli)
            require(matches_residues(numerator, evidence, moduli, bound),
                    'Invalid signed-window residue evidence')
            alternative = numerator - 1 if numerator == bound else numerator + 1
            require(not matches_residues(alternative, evidence, moduli, bound),
                    'Residue evidence accepted a different representative')
    print(f'Fixed scale {scale}, squared error <= 15/{scale**2}, '
          f'signed velocity and curl inside [-{bound},{bound}] checked in all six squared lanes')
    return as_field(numerators, scale)


def main():
    grid = Torus(4)
    viscosity, lattice_step, physical_step = Q(1), Q(1, 256), Q(1, 512)
    amplitude = Q(1, 33)
    sine = (Q(0), amplitude, Q(0), -amplitude)
    u0 = [[sine[pos[axis]] for pos in grid.sites] for axis in (2, 0, 1)]
    weights = normalized_weights(coefficients(2), Q(1, 2))
    require(weights == ((1, Q(1, 4)), (2, Q(3, 16))), 'Unexpected omega viscosity weights')
    u1, denominator = velocity_step(grid, u0, viscosity, lattice_step, viscous_weights=weights)
    stored_u1 = bounded_grid_output(grid, u1)
    v0, v1 = lift_n4(grid, u0), lift_n4(grid, stored_u1)
    require(v0 == expected_initial(amplitude), 'The exact initial lift disagrees with continuum data')
    require(majorant(nonlinear(v0)) > 0, 'Expected genuine initial convection')
    require(any(sum(map(abs, k)) == 2 for _, k in lift_n4(grid, u1)),
            'The actual grid step must create nonlinear modes before encoding')
    q, eps = two_pi_bounds()
    require((q - eps)**2 * gradient_squared_without_2pi(v0) > (2*viscosity/9)**2,
            'The supplied datum must start above the direct small-data threshold')

    previous = v0
    radius = Q(0)
    pieces = 4
    for index in range(1, pieces+1):
        fraction = Q(index, pieces)
        endpoint = {}
        add_scaled(endpoint, v0, 1-fraction)
        add_scaled(endpoint, v1, fraction)
        candidate = dict(previous)
        add_scaled(candidate, previous, -1, degree_offset=1)
        add_scaled(candidate, endpoint, 1, degree_offset=1)
        require(end_field(candidate) == endpoint, 'Affine grid-lift endpoint mismatch')
        delta = physical_step/pieces
        residual = residual_majorant(candidate, viscosity, delta)
        next_radius = round_up(Q(17, 16) * (radius + delta*residual), 2**32)
        require(check_slab(candidate, viscosity, delta, radius, Q(1, 45), next_radius),
                f'Continuum comparison failed on subslab {index}')
        radius = next_radius
        previous = endpoint
        print(f'Omega lift subslab {index}: continuum residual <= {round_up(residual, 10**6)}, '
              f'A3 error <= {round_up(radius, 10**6)}')
    require(previous == v1, 'Last lift must be the recovered fixed-scale Fourier projection')
    require(check_terminal(v1, viscosity, radius), 'The grid-sourced terminal certificate failed')
    print(f'Global continuation accepted at {physical_step} from one exact N=4 omega step '
          f'({denominator.bit_length()} denominator bits)')


if __name__ == '__main__':
    main()
