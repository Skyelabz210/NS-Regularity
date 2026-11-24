#!/usr/bin/env python3
"""Generate and independently replay exact Fourier continuation certificates."""

import argparse
import json
from fractions import Fraction as Q
from math import isqrt
from pathlib import Path

from verify_continuation import (
    GaussianRational,
    ZERO,
    add_scaled,
    add_term,
    check_terminal,
    end_field,
    majorant,
    nonlinear,
    positive_laplacian,
    require,
    pressure_projection,
    residual_majorant,
    round_up,
    slab_report,
    two_pi_bounds,
    validate_poly,
)


def initial_cyclic(amplitude):
    field = {}
    for direction, component in ((2, 0), (0, 1), (1, 2)):
        for sign in (-1, 1):
            mode = tuple(sign if j == direction else 0 for j in range(3))
            vector = list(ZERO)
            vector[component] = GaussianRational(im=-sign * amplitude / 2)
            add_term(field, 0, mode, tuple(vector))
    return field


def start_field(poly):
    return {
        key: vector for key, vector in poly.items() if key[0] == 0 and vector != ZERO
    }


def field_distance(left, right):
    difference = dict(left)
    add_scaled(difference, right, -1)
    return majorant(difference)


def dyadic_round(value, bits):
    scaled = value * (1 << bits)
    nearest = (2 * scaled.numerator + scaled.denominator) // (2 * scaled.denominator)
    return Q(nearest, 1 << bits)


def round_field(field, bits):
    # Project after rounding to retain exact solenoidality. Conjugate pairs
    # are rounded together, so midpoint ties cannot break reality.
    out = {}
    for _, mode in sorted(field):
        opposite = tuple(-v for v in mode)
        if mode < opposite:
            continue
        vector = field[0, mode]
        rounded = tuple(
            GaussianRational(dyadic_round(z.re, bits), dyadic_round(z.im, bits))
            for z in vector
        )
        projected = pressure_projection(mode, rounded)
        add_term(out, 0, mode, projected)
        if opposite != mode:
            add_term(
                out,
                0,
                opposite,
                tuple(GaussianRational(z.re, -z.im) for z in projected),
            )
    validate_poly(out)
    return out


def taylor_candidate(initial, viscosity, step, degree, cutoff):
    """Finite Fourier Taylor predictor, with discarded modes charged in the residual."""
    q, _ = two_pi_bounds()
    candidate = dict(initial)
    for order in range(degree):
        rhs = {}
        lap = {
            key: v
            for key, v in positive_laplacian(candidate).items()
            if key[0] == order
        }
        conv = {
            key: v
            for key, v in nonlinear(candidate, max_degree=order).items()
            if key[0] == order and sum(map(abs, key[1])) <= cutoff
        }
        add_scaled(rhs, lap, -viscosity * q * q * step / Q(order + 1))
        add_scaled(rhs, conv, -q * step / Q(order + 1))
        for (_, mode), vector in rhs.items():
            add_term(candidate, order + 1, mode, vector)
    validate_poly(candidate)
    return candidate


def sqrt_upper(value, bits=40):
    denominator = 1 << bits
    root = isqrt((value.numerator * denominator**2) // value.denominator)
    if Q(root, denominator) ** 2 < value:
        root += 1
    return Q(root, denominator)


def choose_radius(poly, viscosity, step, eta, bits):
    smoothing = sqrt_upper(step / viscosity)
    size = majorant(poly)
    residual = residual_majorant(poly, viscosity, step)
    linear_margin = 1 - 2 * smoothing * size
    if linear_margin <= 0:
        return None
    source = eta + step * residual
    scale = 1 << bits
    # The self-map quadratic is increasing up to its vertex. Search that
    # interval on a fixed rational lattice using exact comparisons.
    vertex = linear_margin / (2 * smoothing)
    high = (vertex * scale).numerator // (vertex * scale).denominator

    def accepts(integer):
        radius = Q(integer, scale)
        return (
            radius > 0
            and source + smoothing * (2 * size * radius + radius * radius) <= radius
        )

    if high < 1 or not accepts(high):
        return None
    low = 1
    while low < high:
        midpoint = (low + high) // 2
        if accepts(midpoint):
            high = midpoint
        else:
            low = midpoint + 1
    radius = Q(low, scale)
    if 2 * smoothing * (size + radius) >= 1:
        return None
    return smoothing, radius, residual


def endpoint_error(
    poly, viscosity, step, eta, smoothing, radius, residual, initial, initial_error
):
    """Use the heat spectral gap for nonconstant initial error modes."""
    mean = initial.get((0, (0, 0, 0)), ZERO)
    candidate_mean = poly.get((0, (0, 0, 0)), ZERO)
    mean_difference = sum(
        abs(a.re - b.re) + abs(a.im - b.im) for a, b in zip(mean, candidate_mean)
    )
    mean_bound = min(eta, initial_error + mean_difference)
    q, eps = two_pi_bounds()
    decay = 1 / (1 + viscosity * (q - eps) ** 2 * step)
    bound = (
        decay * eta
        + (1 - decay) * mean_bound
        + step * residual
        + smoothing * (2 * majorant(poly) * radius + radius**2)
    )
    return min(radius, round_up(bound, 1 << 96))


def generate(
    initial,
    viscosity,
    step,
    max_steps,
    degree=3,
    cutoff=3,
    bits=96,
    initial_error=Q(0),
    heat_decay=False,
):
    validate_poly(initial)
    require(
        all(order == 0 for order, _ in initial),
        "Initial data must have time degree zero",
    )
    require(
        viscosity > 0 and step > 0 and max_steps >= 0 and initial_error >= 0,
        "Invalid integration parameters",
    )
    require(degree >= 1 and cutoff >= 1 and bits >= 8, "Invalid predictor parameters")
    require(type(heat_decay) is bool, "Invalid endpoint policy")
    require(
        all(sum(map(abs, mode)) <= cutoff for _, mode in initial),
        "The cutoff must contain the prescribed initial support",
    )
    current = dict(initial)
    radius = initial_error
    intervals = []
    elapsed = Q(0)
    if check_terminal(current, viscosity, radius):
        return intervals, "global", elapsed, radius
    for index in range(max_steps):
        anchor = round_field(current, bits)
        jump = field_distance(current, anchor)
        candidate = taylor_candidate(anchor, viscosity, step, degree, cutoff)
        eta = radius + jump
        selected = choose_radius(candidate, viscosity, step, eta, bits)
        if selected is None:
            return intervals, "slab_inconclusive", elapsed, radius
        smoothing, radius, residual = selected
        intervals.append(
            {"poly": candidate, "step": step, "smoothing": smoothing, "radius": radius}
        )
        if heat_decay:
            radius = endpoint_error(
                candidate,
                viscosity,
                step,
                eta,
                smoothing,
                radius,
                residual,
                initial,
                initial_error,
            )
        elapsed += step
        current = end_field(candidate)
        if index % 8 == 0:
            print(
                f"interval {index+1}, time {elapsed}, radius <= {round_up(radius, 10**8)}, "
                f"residual <= {round_up(residual, 10**8)}",
                flush=True,
            )
        if check_terminal(current, viscosity, radius):
            return intervals, "global", elapsed, radius
    return intervals, "horizon_certified", elapsed, radius


def verify_chain(initial, viscosity, intervals, initial_error=Q(0), heat_decay=False):
    """Replay proof obligations, recomputing every jump and residual."""
    validate_poly(initial)
    require(
        all(order == 0 for order, _ in initial),
        "Initial data must be a single time slice",
    )
    require(isinstance(viscosity, (int, Q)) and viscosity > 0, "Invalid viscosity")
    require(
        isinstance(initial_error, (int, Q)) and initial_error >= 0,
        "Invalid initial error",
    )
    require(type(heat_decay) is bool, "Invalid endpoint policy")
    current = dict(initial)
    radius = initial_error
    elapsed = Q(0)
    for index, interval in enumerate(intervals):
        candidate = interval["poly"]
        step, smoothing, next_radius = (
            interval[key] for key in ("step", "smoothing", "radius")
        )
        eta = radius + field_distance(current, start_field(candidate))
        bounds = slab_report(candidate, viscosity, step, eta, smoothing, next_radius)
        require(bounds is not None, f"Unverified continuation interval {index+1}")
        elapsed += step
        radius = next_radius
        if heat_decay:
            radius = endpoint_error(
                candidate,
                viscosity,
                step,
                eta,
                smoothing,
                radius,
                bounds[1],
                initial,
                initial_error,
            )
        current = end_field(candidate)
    return {
        "global": check_terminal(current, viscosity, radius),
        "elapsed": elapsed,
        "radius": radius,
    }


def encode_poly(poly):
    return [
        {
            "degree": degree,
            "mode": list(mode),
            "real": [str(Q(z.re)) for z in vector],
            "imag": [str(Q(z.im)) for z in vector],
        }
        for (degree, mode), vector in sorted(poly.items())
    ]


def decode_poly(rows):
    out = {}
    for row in rows:
        degree, mode = row["degree"], tuple(row["mode"])
        require(
            type(degree) is int
            and len(mode) == 3
            and all(type(k) is int for k in mode),
            "Invalid Fourier index",
        )
        require(
            len(row["real"]) == 3 and len(row["imag"]) == 3, "Invalid vector dimensions"
        )
        require(
            all(type(v) is str for v in row["real"] + row["imag"]),
            "Rationals must be strings",
        )
        require((degree, mode) not in out, "Repeated Fourier index")
        out[degree, mode] = tuple(
            GaussianRational(Q(a), Q(b)) for a, b in zip(row["real"], row["imag"])
        )
    validate_poly(out)
    return {key: value for key, value in out.items() if value != ZERO}


def save_certificate(
    path, initial, viscosity, intervals, initial_error=Q(0), heat_decay=False
):
    document = {
        "format": "ns-fourier-continuation-v1",
        "viscosity": str(viscosity),
        "initial": encode_poly(initial),
        "initial_error": str(initial_error),
        "intervals": [
            {
                "poly": encode_poly(slab["poly"]),
                **{key: str(slab[key]) for key in ("step", "smoothing", "radius")},
            }
            for slab in intervals
        ],
    }
    if heat_decay:
        document["endpoint_decay"] = True
    path.write_text(json.dumps(document, indent=2) + "\n")


def load_certificate(
    path, expected_initial, expected_viscosity, expected_initial_error=Q(0)
):
    document = json.loads(path.read_text())
    require(
        document.get("format") == "ns-fourier-continuation-v1",
        "Unknown certificate format",
    )
    require(
        type(document["viscosity"]) is str, "Viscosity must be an exact rational string"
    )
    viscosity = Q(document["viscosity"])
    initial = decode_poly(document["initial"])
    require(
        initial == expected_initial and viscosity == expected_viscosity,
        "Initial source mismatch",
    )
    require(
        type(document["initial_error"]) is str,
        "Initial error must be an exact rational string",
    )
    initial_error = Q(document["initial_error"])
    require(initial_error == expected_initial_error, "Initial error budget mismatch")
    intervals = []
    for row in document["intervals"]:
        require(
            all(type(row[key]) is str for key in ("step", "smoothing", "radius")),
            "Interval bounds must be exact rational strings",
        )
        intervals.append(
            {
                "poly": decode_poly(row["poly"]),
                **{key: Q(row[key]) for key in ("step", "smoothing", "radius")},
            }
        )
    return verify_chain(
        initial,
        viscosity,
        intervals,
        initial_error,
        document.get("endpoint_decay", False),
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--amplitude", type=Q, default=Q(1, 33))
    parser.add_argument("--viscosity", type=Q, default=Q(1))
    parser.add_argument(
        "--initial", type=Path, help="JSON array of exact Fourier coefficients"
    )
    parser.add_argument("--step", type=Q, default=Q(1, 2048))
    parser.add_argument("--max-steps", type=int, default=128)
    parser.add_argument(
        "--initial-error", type=Q, default=Q(0), help="Declared A3 neighborhood radius"
    )
    parser.add_argument("--degree", type=int, default=3)
    parser.add_argument("--cutoff", type=int, default=3)
    parser.add_argument("--bits", type=int, default=96)
    parser.add_argument(
        "--heat-decay",
        action="store_true",
        help="Sharpen interval endpoint error with viscous decay",
    )
    parser.add_argument("--output", type=Path)
    parser.add_argument("--verify", type=Path)
    args = parser.parse_args()
    initial = (
        decode_poly(json.loads(args.initial.read_text()))
        if args.initial
        else initial_cyclic(args.amplitude)
    )
    if args.verify:
        print(
            load_certificate(args.verify, initial, args.viscosity, args.initial_error)
        )
        return
    intervals, status, elapsed, radius = generate(
        initial,
        args.viscosity,
        args.step,
        args.max_steps,
        args.degree,
        args.cutoff,
        args.bits,
        args.initial_error,
        args.heat_decay,
    )
    replay = verify_chain(
        initial, args.viscosity, intervals, args.initial_error, args.heat_decay
    )
    require(
        replay["elapsed"] == elapsed
        and replay["radius"] == radius
        and replay["global"] == (status == "global"),
        "Replay result mismatch",
    )
    if args.output:
        save_certificate(
            args.output,
            initial,
            args.viscosity,
            intervals,
            args.initial_error,
            args.heat_decay,
        )
        require(
            load_certificate(args.output, initial, args.viscosity, args.initial_error)
            == replay,
            "Serialized replay mismatch",
        )
    print(
        f"{status} at {elapsed} from {len(intervals)} independently verified intervals; "
        f"radius <= {round_up(radius,10**8)}"
    )
    if status == "slab_inconclusive":
        print(
            "The next self-map or contraction inequality is inconclusive at these parameters."
        )
        raise SystemExit(2)


if __name__ == "__main__":
    main()
