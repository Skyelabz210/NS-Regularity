#!/usr/bin/env python3
"""Regression checks for initial binding, interval chaining and certificate replay."""

from copy import deepcopy
from fractions import Fraction as Q
import json
from pathlib import Path
from tempfile import TemporaryDirectory

from certify_fourier_flow import (
    decode_poly,
    encode_poly,
    field_distance,
    generate,
    initial_cyclic,
    load_certificate,
    save_certificate,
    sqrt_upper,
    verify_chain,
)
from verify_continuation import (
    GaussianRational,
    add_scaled,
    nonlinear,
    require,
    two_pi_bounds,
    gradient_squared_without_2pi,
)


def rejected(action, label):
    try:
        action()
    except (ArithmeticError, ValueError, TypeError):
        return
    raise ArithmeticError(f"An invalid certificate was accepted: {label}")


def main():
    initial = initial_cyclic(Q(1, 33))
    viscosity, initial_error = Q(1), Q(1, 10**8)
    q, eps = two_pi_bounds()
    require(
        (q - eps) ** 2 * gradient_squared_without_2pi(initial) > Q(2, 9) ** 2,
        "The source datum must exceed the direct small-data threshold",
    )
    intervals, status, time, radius = generate(
        initial,
        viscosity,
        Q(1, 1024),
        8,
        degree=3,
        cutoff=3,
        bits=80,
        initial_error=initial_error,
    )
    require(
        status == "global" and len(intervals) >= 2, "Expected a nontrivial global chain"
    )
    verified = verify_chain(initial, viscosity, intervals, initial_error)
    require(
        verified == {"global": True, "elapsed": time, "radius": radius},
        "Chain replay mismatch",
    )
    require(
        decode_poly(encode_poly(initial)) == initial, "Initial data did not round trip"
    )
    affine = dict(initial)
    add_scaled(affine, initial, 1, degree_offset=1)
    base_convection = nonlinear(initial)
    expected = dict(base_convection)
    add_scaled(expected, base_convection, 2, degree_offset=1)
    add_scaled(expected, base_convection, 1, degree_offset=2)
    require(
        nonlinear(affine) == expected, "Full residual convolution lost a time degree"
    )
    for degree in (0, 1, 2):
        require(
            nonlinear(affine, max_degree=degree)
            == {key: v for key, v in expected.items() if key[0] <= degree},
            "The predictor convolution limit changed a retained coefficient",
        )

    require(
        sqrt_upper(Q(2)) ** 2 >= 2 and sqrt_upper(Q(1, 4)) == Q(1, 2),
        "Invalid square root bound",
    )

    with TemporaryDirectory() as directory:
        path = Path(directory) / "certificate.json"
        save_certificate(path, initial, viscosity, intervals, initial_error)
        require(
            load_certificate(path, initial, viscosity, initial_error) == verified,
            "Serialized certificate did not replay",
        )
        rejected(
            lambda: load_certificate(
                path, initial_cyclic(Q(1, 32)), viscosity, initial_error
            ),
            "wrong initial data",
        )
        rejected(
            lambda: load_certificate(path, initial, Q(2), initial_error),
            "wrong viscosity",
        )
        rejected(
            lambda: load_certificate(path, initial, viscosity, 0), "wrong initial error"
        )
        document = json.loads(path.read_text())
        document["intervals"][0]["step"] = 0.001
        path.write_text(json.dumps(document))
        rejected(
            lambda: load_certificate(path, initial, viscosity, initial_error),
            "float step",
        )

    failed_radius = deepcopy(intervals)
    failed_radius[0]["radius"] = Q(0)
    rejected(
        lambda: verify_chain(initial, viscosity, failed_radius, initial_error),
        "zero error radius",
    )
    failed_smoothing = deepcopy(intervals)
    failed_smoothing[0]["smoothing"] /= 2
    rejected(
        lambda: verify_chain(initial, viscosity, failed_smoothing, initial_error),
        "small smoothing bound",
    )
    failed_jump = deepcopy(intervals)
    first = failed_jump[1]["poly"]
    key = next(key for key in first if key[0] == 0)
    opposite = (key[0], tuple(-k for k in key[1]))
    first[key] = tuple(z.scale(2) for z in first[key])
    first[opposite] = tuple(GaussianRational(z.re, -z.im) for z in first[key])
    require(
        field_distance(intervals[1]["poly"], first) > 0, "Jump mutation had no effect"
    )
    rejected(
        lambda: verify_chain(initial, viscosity, failed_jump, initial_error),
        "unpaid candidate jump",
    )
    bad_rows = encode_poly(initial)
    bad_rows[0]["mode"][0] = True
    rejected(lambda: decode_poly(bad_rows), "boolean Fourier index")
    rejected(lambda: decode_poly(encode_poly(initial) * 2), "duplicate Fourier index")
    print("Exact certificate replay passed with a nonzero initial neighborhood radius")
    print(
        "Source, viscosity, error-budget, float, radius, smoothing, jump and index controls rejected"
    )


if __name__ == "__main__":
    main()
