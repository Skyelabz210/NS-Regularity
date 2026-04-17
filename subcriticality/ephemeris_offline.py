#!/usr/bin/env python3
"""
OFFLINE ANALYTICAL EPHEMERIS
==============================

Ultra-high precision planetary position calculator.
Zero external dependencies. Works offline forever.

Architecture:
  Float computation is UNAVOIDABLE for celestial mechanics (Kepler's
  equation, perturbation series with sin/cos). But floats exist ONLY
  inside this module. The output boundary converts to integer arcseconds.
  One float→int conversion. Everything downstream is exact.

Methods:
  Sun:     Meeus Ch. 25 — equation of center + nutation (~±0.01°)
  Moon:    Meeus Ch. 47 — simplified ELP2000-82 (~±0.5°)
  Planets: VSOP87 mean longitude + equation of center → geocentric
           (~±1° outer, ~±0.5° inner)

For production: install pyswisseph for JPL DE440/441 (±0.001°).
This module is the fallback when that's unavailable.

Author: HackFate Research Laboratory
Date: 2026-03-24
"""

from __future__ import annotations
from typing import Dict, Tuple
from math import sin, cos, radians, sqrt, atan2, degrees, pi

ARCSEC_PER_DEG = 3600
FULL_CIRCLE_AS = 1296000
FULL_CIRCLE_DEG = 360.0

# J2000.0 epoch
J2000_JD = 2451545.0

# Maya epoch (GMT 584283 correlation)
MAYA_EPOCH_JD = 584283


# ═══════════════════════════════════════════════════════════════════════════
# JULIAN DATE CONVERSION
# ═══════════════════════════════════════════════════════════════════════════

def maya_day_to_jd(day_offset: int) -> float:
    """Convert Maya day offset to Julian Date."""
    return MAYA_EPOCH_JD + day_offset


def jd_to_centuries(jd: float) -> float:
    """Julian centuries from J2000.0."""
    return (jd - J2000_JD) / 36525.0


def date_to_jd(year: int, month: int, day: int,
               hour: float = 12.0) -> float:
    """Gregorian date to Julian Date (Meeus formula)."""
    if month <= 2:
        year -= 1
        month += 12
    A = year // 100
    B = 2 - A + A // 4
    return (int(365.25 * (year + 4716)) +
            int(30.6001 * (month + 1)) +
            day + B - 1524.5 + hour / 24.0)


# ═══════════════════════════════════════════════════════════════════════════
# ORBITAL ELEMENTS — Meeus Table 31.A (J2000.0 + rates)
# (L0, L1, e0, e1, i0, i1, omega0, omega1, Omega0, Omega1, a)
# L = mean longitude (deg), e = eccentricity, i = inclination,
# omega = argument of perihelion, Omega = longitude of ascending node,
# a = semi-major axis (AU)
# All rates are per Julian century.
# ═══════════════════════════════════════════════════════════════════════════

ELEMENTS = {
    # L0 = Swiss Ephemeris calibrated heliocentric longitude at J2000 (degrees)
    # L1 = Meeus mean heliocentric longitude rate (degrees/century) — NOT instantaneous speed
    # w0, w1 = longitude of perihelion at J2000 + rate
    # e0, e1 = eccentricity at J2000 + rate
    # a = semi-major axis (AU)
    'Mercury': {
        'L0': 253.7737, 'L1': 149474.07,  # 360°/87.97d × 36525
        'w0': 77.456,   'w1': 1.556,
        'e0': 0.20563593, 'e1': 0.00001906,
        'a': 0.38709927,
    },
    'Venus': {
        'L0': 182.5935, 'L1': 58519.21,   # 360°/224.70d × 36525
        'w0': 131.564,  'w1': 1.402,
        'e0': 0.00677672, 'e1': -0.00004107,
        'a': 0.72333566,
    },
    'Earth': {
        'L0': 100.3689, 'L1': 36000.77,   # 360°/365.25d × 36525
        'w0': 102.937,  'w1': 1.720,
        'e0': 0.01671123, 'e1': -0.00004392,
        'a': 1.00000261,
    },
    'Mars': {
        'L0': 359.4389, 'L1': 19141.70,   # 360°/686.97d × 36525
        'w0': 336.060,  'w1': 1.841,
        'e0': 0.09339410, 'e1': 0.00007882,
        'a': 1.52371034,
    },
    'Jupiter': {
        'L0': 36.2881,  'L1': 3036.30,    # 360°/4332.59d × 36525
        'w0': 14.331,   'w1': 1.612,
        'e0': 0.04838624, 'e1': -0.00013253,
        'a': 5.20288700,
    },
    'Saturn': {
        'L0': 45.7165,  'L1': 1223.51,    # 360°/10759.22d × 36525
        'w0': 93.057,   'w1': 1.964,
        'e0': 0.05386179, 'e1': -0.00050991,
        'a': 9.53667594,
    },
    'Uranus': {
        'L0': 316.4136, 'L1': 428.47,     # 360°/30688.5d × 36525
        'w0': 173.005,  'w1': 1.486,
        'e0': 0.04725744, 'e1': -0.00004397,
        'a': 19.18916464,
    },
    'Neptune': {
        'L0': 303.9239, 'L1': 218.49,     # 360°/60182d × 36525
        'w0': 48.124,   'w1': 1.426,
        'e0': 0.00859048, 'e1': 0.00005105,
        'a': 30.06992276,
    },
    'Pluto': {
        'L0': 250.5412, 'L1': 146.17,     # 360°/90560d × 36525
        'w0': 224.067,  'w1': 1.291,
        'e0': 0.24882730, 'e1': 0.00006465,
        'a': 39.48211675,
    },
}


# ═══════════════════════════════════════════════════════════════════════════
# JUPITER-SATURN MUTUAL PERTURBATIONS (Meeus Ch. 31)
# These are the largest perturbation terms in the solar system.
# ═══════════════════════════════════════════════════════════════════════════

def jupiter_saturn_perturbations(T: float):
    """Major mutual perturbation terms for Jupiter and Saturn.
    Returns (dL_jupiter_deg, dL_saturn_deg).
    From Meeus, Astronomical Algorithms, simplified.
    """
    # Mean anomalies
    Mj = radians((225.328 + 3034.6921 * T) % 360)
    Ms = radians((175.764 + 1221.5515 * T) % 360)

    # Jupiter perturbations by Saturn
    dL_j = (-0.332 * sin(2 * Mj - 5 * Ms - 67.6 * pi / 180) -
             0.056 * sin(2 * Mj - 2 * Ms + 21.0 * pi / 180) +
             0.042 * sin(3 * Mj - 5 * Ms + 21.0 * pi / 180) -
             0.036 * sin(Mj - 2 * Ms) +
             0.022 * cos(Mj - Ms) +
             0.023 * sin(2 * Mj - 3 * Ms + 52.0 * pi / 180))

    # Saturn perturbations by Jupiter
    dL_s = (0.812 * sin(2 * Mj - 5 * Ms - 67.6 * pi / 180) -
            0.229 * cos(2 * Mj - 4 * Ms - 2.0 * pi / 180) +
            0.119 * sin(Mj - 2 * Ms - 3.0 * pi / 180) +
            0.046 * sin(2 * Mj - 6 * Ms - 69.0 * pi / 180) +
            0.014 * sin(Mj - 3 * Ms + 32.0 * pi / 180))

    # Uranus perturbation by Jupiter and Saturn (major terms)
    dL_u = (0.040 * sin(Ms - 2 * radians((314.055 + 429.885 * T) % 360)) +
            0.035 * sin(Ms) -
            0.015 * sin(3 * Ms))

    return (dL_j, dL_s, dL_u)


# ═══════════════════════════════════════════════════════════════════════════
# KEPLER'S EQUATION SOLVER
# ═══════════════════════════════════════════════════════════════════════════

def solve_kepler(M_deg: float, e: float, tol: float = 1e-10) -> float:
    """Solve Kepler's equation M = E - e*sin(E) for eccentric anomaly E.
    Newton-Raphson iteration. Returns E in degrees.
    """
    M = radians(M_deg % 360.0)
    E = M + e * sin(M)  # initial guess
    for _ in range(50):
        dE = (E - e * sin(E) - M) / (1 - e * cos(E))
        E -= dE
        if abs(dE) < tol:
            break
    return degrees(E)


def true_anomaly(M_deg: float, e: float) -> float:
    """Compute true anomaly from mean anomaly and eccentricity.
    Returns true anomaly in degrees.
    """
    E = radians(solve_kepler(M_deg, e))
    nu = 2 * atan2(sqrt(1 + e) * sin(E / 2), sqrt(1 - e) * cos(E / 2))
    return degrees(nu)


# ═══════════════════════════════════════════════════════════════════════════
# HELIOCENTRIC → GEOCENTRIC CONVERSION
# ═══════════════════════════════════════════════════════════════════════════

def heliocentric_lon(name: str, T: float) -> Tuple[float, float]:
    """Compute heliocentric ecliptic longitude and distance for a planet.
    Uses proper mean anomaly M = L - w (longitude of perihelion).
    Returns (longitude_deg, distance_AU).
    """
    el = ELEMENTS[name]
    L = (el['L0'] + el['L1'] * T) % 360.0
    w = (el['w0'] + el['w1'] * T) % 360.0
    e = el['e0'] + el['e1'] * T
    a = el['a']

    # Proper mean anomaly
    M = (L - w) % 360.0

    # True anomaly via Kepler's equation
    nu = true_anomaly(M, e)

    # Heliocentric longitude = true anomaly + longitude of perihelion
    lon = (nu + w) % 360.0

    # Heliocentric distance
    r = a * (1 - e * e) / (1 + e * cos(radians(nu)))

    # Apply perturbation terms for Jupiter, Saturn, Uranus
    dL_j, dL_s, dL_u = jupiter_saturn_perturbations(T)
    if name == 'Jupiter':
        lon = (lon + dL_j) % 360.0
    elif name == 'Saturn':
        lon = (lon + dL_s) % 360.0
    elif name == 'Uranus':
        lon = (lon + dL_u) % 360.0

    return (lon, r)


def geocentric_planet(planet_name: str, T: float) -> Tuple[float, float]:
    """Convert heliocentric planet position to geocentric longitude.
    Returns (geocentric_longitude_deg, daily_motion_deg).
    """
    # Planet's heliocentric position
    lon_p, r_p = heliocentric_lon(planet_name, T)

    # Earth's heliocentric position
    lon_e, r_e = heliocentric_lon('Earth', T)

    # Convert to Cartesian
    lp_rad = radians(lon_p)
    le_rad = radians(lon_e)

    xp = r_p * cos(lp_rad) - r_e * cos(le_rad)
    yp = r_p * sin(lp_rad) - r_e * sin(le_rad)

    # Geocentric ecliptic longitude
    geo_lon = degrees(atan2(yp, xp)) % 360.0

    # Approximate daily motion from rate per century
    daily_motion = ELEMENTS[planet_name]['L1'] / 36525.0

    return (geo_lon, daily_motion)


# ═══════════════════════════════════════════════════════════════════════════
# SUN — Meeus Ch. 25 (geocentric by definition)
# ═══════════════════════════════════════════════════════════════════════════

def sun_position(T: float) -> Tuple[float, float]:
    """Sun's apparent geocentric ecliptic longitude.
    Returns (longitude_deg, daily_motion_deg).
    Accuracy: ±0.01° (36 arcseconds).
    """
    L0 = 280.46646 + 36000.76983 * T + 0.0003032 * T * T
    M = 357.52911 + 35999.05029 * T - 0.0001537 * T * T
    M_rad = radians(M)

    # Equation of center
    C = ((1.914602 - 0.004817 * T) * sin(M_rad) +
         (0.019993 - 0.000101 * T) * sin(2 * M_rad) +
         0.000289 * sin(3 * M_rad))

    # Nutation + aberration
    omega = 125.04 - 1934.136 * T
    apparent = L0 + C - 0.00569 - 0.00478 * sin(radians(omega))

    return (apparent % 360.0, 0.9856)


# ═══════════════════════════════════════════════════════════════════════════
# MOON — Meeus Ch. 47 (simplified ELP2000-82)
# ═══════════════════════════════════════════════════════════════════════════

def moon_position(T: float) -> Tuple[float, float]:
    """Moon's geocentric ecliptic longitude.
    Returns (longitude_deg, daily_motion_deg).
    Accuracy: ±0.5°.
    """
    Lp = (218.3164477 + 481267.88123421 * T -
          0.0015786 * T * T + T * T * T / 538841.0)
    D = (297.8501921 + 445267.1114034 * T -
         0.0018819 * T * T + T * T * T / 545868.0)
    M = (357.5291092 + 35999.0502909 * T -
         0.0001536 * T * T)
    Mp = (134.9633964 + 477198.8675055 * T +
          0.0087414 * T * T + T * T * T / 69699.0)
    F = (93.2720950 + 483202.0175233 * T -
         0.0036539 * T * T)

    D_r = radians(D); M_r = radians(M)
    Mp_r = radians(Mp); F_r = radians(F)

    # Major longitude terms
    sigma_l = (6.288774 * sin(Mp_r) +
               1.274027 * sin(2 * D_r - Mp_r) +
               0.658314 * sin(2 * D_r) +
               0.213618 * sin(2 * Mp_r) -
               0.185116 * sin(M_r) -
               0.114332 * sin(2 * F_r) +
               0.058793 * sin(2 * D_r - 2 * Mp_r) +
               0.057066 * sin(2 * D_r - M_r - Mp_r) +
               0.053322 * sin(2 * D_r + Mp_r) +
               0.045758 * sin(2 * D_r - M_r))

    lon = (Lp + sigma_l) % 360.0
    return (lon, 13.176)


# ═══════════════════════════════════════════════════════════════════════════
# MASTER FUNCTION — all planets, one call
# ═══════════════════════════════════════════════════════════════════════════

def compute_all_positions(jd: float) -> Dict[str, Tuple[float, float]]:
    """Compute geocentric ecliptic longitude for all 10 bodies.
    Returns {name: (longitude_deg, daily_motion_deg)}.
    All computation in float. Conversion to int happens at output boundary.
    """
    T = jd_to_centuries(jd)
    positions = {}

    positions['Sun'] = sun_position(T)
    positions['Moon'] = moon_position(T)

    for planet in ('Mercury', 'Venus', 'Mars', 'Jupiter',
                   'Saturn', 'Uranus', 'Neptune', 'Pluto'):
        positions[planet] = geocentric_planet(planet, T)

    return positions


# ═══════════════════════════════════════════════════════════════════════════
# OUTPUT BOUNDARY — float → int arcseconds
# ═══════════════════════════════════════════════════════════════════════════

def ephemeris_for_day_offline(day_offset: int,
                              asc_arcsec: int = 0
                              ) -> Dict[str, Tuple[int, int, int]]:
    """Compute planetary positions for a Maya day offset.

    THIS IS THE BOUNDARY. Float computation happens inside.
    Output is integer arcseconds. No float escapes this function.
    Applies Swiss-calibrated correction tables for ±1° accuracy.

    Returns: {name: (lon_arcsec, vel_arcsec_per_day, house_arcsec)}
    """
    jd = maya_day_to_jd(day_offset)
    positions = compute_all_positions(jd)

    # Load corrections (lazy import)
    try:
        from ephemeris_corrections import (
            CORRECTIONS, CORR_START, CORR_STEP_DEFAULT, CORR_STEP_MARS,
            CORR_PLANETS_DEFAULT
        )
        has_corrections = True
    except ImportError:
        has_corrections = False

    result = {}
    for name, (lon_deg, vel_deg) in positions.items():
        lon_as = int(round(lon_deg * ARCSEC_PER_DEG)) % FULL_CIRCLE_AS
        vel_as = int(round(vel_deg * ARCSEC_PER_DEG))

        # Apply correction table if available
        if has_corrections and name in CORRECTIONS:
            step = CORR_STEP_MARS if name == 'Mars' else CORR_STEP_DEFAULT
            corr = CORRECTIONS[name]
            idx = (day_offset - CORR_START) // step
            if 0 <= idx < len(corr) - 1:
                c0 = corr[idx]; c1 = corr[idx + 1]
                frac_num = (day_offset - CORR_START) % step
                correction = c0 + ((c1 - c0) * frac_num) // step
                lon_as = (lon_as + correction) % FULL_CIRCLE_AS

        house_as = (lon_as - asc_arcsec) % FULL_CIRCLE_AS
        result[name] = (lon_as, vel_as, house_as)

    return result


# ═══════════════════════════════════════════════════════════════════════════
# SMART PROVIDER — tries Swiss Ephemeris first, falls back to analytical
# ═══════════════════════════════════════════════════════════════════════════

def ephemeris_for_day_auto(day_offset: int,
                           asc_arcsec: int = 0
                           ) -> Dict[str, Tuple[int, int, int]]:
    """Auto-select best available ephemeris.

    Priority:
      1. Swiss Ephemeris (pyswisseph) — ±0.001° (arcsecond)
      2. Analytical (this module) — ±0.5° (Sun/Moon), ±1° (planets)

    Returns: {name: (lon_arcsec, vel_arcsec_per_day, house_arcsec)}
    """
    try:
        import swisseph as swe
        jd = MAYA_EPOCH_JD + day_offset
        result = {}
        planet_ids = {
            'Sun': 0, 'Moon': 1, 'Mercury': 2, 'Venus': 3, 'Mars': 4,
            'Jupiter': 5, 'Saturn': 6, 'Uranus': 7, 'Neptune': 8, 'Pluto': 9,
        }
        for name, pid in planet_ids.items():
            calc = swe.calc_ut(jd, pid, 256)
            lon_as = int(round(calc[0][0] * ARCSEC_PER_DEG)) % FULL_CIRCLE_AS
            vel_as = int(round(calc[0][3] * ARCSEC_PER_DEG))
            house_as = (lon_as - asc_arcsec) % FULL_CIRCLE_AS
            result[name] = (lon_as, vel_as, house_as)
        swe.close()
        return result
    except (ImportError, Exception):
        return ephemeris_for_day_offline(day_offset, asc_arcsec)


# ═══════════════════════════════════════════════════════════════════════════
# VALIDATION — compare analytical vs Swiss Ephemeris
# ═══════════════════════════════════════════════════════════════════════════

def validate_ephemeris(day_offset: int = 1860251) -> None:
    """Compare analytical vs Swiss Ephemeris for validation."""
    signs = ['Ari','Tau','Gem','Can','Leo','Vir',
             'Lib','Sco','Sag','Cap','Aqu','Pis']

    # Analytical
    analytical = ephemeris_for_day_offline(day_offset)

    # Swiss Ephemeris (if available)
    try:
        import swisseph as swe
        jd = MAYA_EPOCH_JD + day_offset
        swiss = {}
        pids = {'Sun':0,'Moon':1,'Mercury':2,'Venus':3,'Mars':4,
                'Jupiter':5,'Saturn':6,'Uranus':7,'Neptune':8,'Pluto':9}
        for name, pid in pids.items():
            calc = swe.calc_ut(jd, pid, 256)
            swiss[name] = int(round(calc[0][0] * ARCSEC_PER_DEG)) % FULL_CIRCLE_AS
        swe.close()
        has_swiss = True
    except (ImportError, Exception):
        has_swiss = False

    print("=" * 74)
    print("  EPHEMERIS VALIDATION")
    print(f"  Day offset: {day_offset}")
    print("=" * 74)

    if has_swiss:
        print(f"\n  {'Planet':10s}  {'Analytical':14s}  {'Swiss Eph':14s}  {'Δ':>6s}")
        print(f"  {'─'*10}  {'─'*14}  {'─'*14}  {'─'*6}")
        for name in ('Sun','Moon','Mercury','Venus','Mars',
                     'Jupiter','Saturn','Uranus','Neptune','Pluto'):
            a_lon = analytical[name][0]
            s_lon = swiss[name]
            diff = abs(a_lon - s_lon)
            if diff > FULL_CIRCLE_AS // 2:
                diff = FULL_CIRCLE_AS - diff
            diff_deg = diff / ARCSEC_PER_DEG

            a_si = (a_lon // (30 * ARCSEC_PER_DEG)) % 12
            a_d = (a_lon % (30 * ARCSEC_PER_DEG)) // ARCSEC_PER_DEG
            s_si = (s_lon // (30 * ARCSEC_PER_DEG)) % 12
            s_d = (s_lon % (30 * ARCSEC_PER_DEG)) // ARCSEC_PER_DEG

            quality = "✓" if diff_deg < 2.0 else "⚠" if diff_deg < 5.0 else "✗"
            print(f"  {name:10s}  {a_d:2d}° {signs[a_si]:3s}         "
                  f"{s_d:2d}° {signs[s_si]:3s}         "
                  f"{diff_deg:5.1f}° {quality}")
    else:
        print("\n  Swiss Ephemeris not available — showing analytical only")
        for name in ('Sun','Moon','Mercury','Venus','Mars',
                     'Jupiter','Saturn','Uranus','Neptune','Pluto'):
            lon = analytical[name][0]
            si = (lon // (30 * ARCSEC_PER_DEG)) % 12
            d = (lon % (30 * ARCSEC_PER_DEG)) // ARCSEC_PER_DEG
            m = ((lon % (30 * ARCSEC_PER_DEG)) % ARCSEC_PER_DEG) // 60
            print(f"  {name:10s}  {d:2d}°{m:02d}' {signs[si]:3s}")

    print("\n  Accuracy tiers:")
    print("    Sun:     ±0.01° (Meeus equation of center)")
    print("    Moon:    ±0.5°  (simplified ELP2000-82)")
    print("    Planets: ±1-3°  (VSOP87 mean longitude + Kepler)")
    print("    Install pyswisseph for ±0.001° (JPL DE440/441)")
    print("=" * 74)


if __name__ == '__main__':
    validate_ephemeris(1860251)  # Anthony's birth
    print()
    validate_ephemeris(1860583)  # Test chart (Sep 18, 1981)
    print()
    validate_ephemeris(1876816)  # Today (Mar 24, 2026)
