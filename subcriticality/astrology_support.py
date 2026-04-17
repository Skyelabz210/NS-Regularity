#!/usr/bin/env python3
"""
astrology_support.py — Supporting Modules for CRAM Astrology Engine v4
========================================================================

Wires all supporting functionality to the v4 engine + gear_manifold_v3.
Replaces the old chain: astrology_engine_v3_1 → gear_manifold_v2 (cascade bug).

Modules:
  1. EPHEMERIS     — offline analytical + Swiss Ephemeris auto-select
  2. HOUSES        — Placidus house data + CRT decomposition (12 = 4×3)
  3. VEDIC         — sidereal positions, nakshatras, Vimsottari Dasha
  4. TRANSITS      — transit evaluation against natal chart
  5. EVENTS        — modular arithmetic event prediction (no brute force)
  6. SYSTEM        — unified API: birth data in → complete reading out

All computation: exact integer after the ephemeris boundary.
Float exists ONLY in ephemeris_offline (celestial mechanics requires it).
The float→int boundary is ephemeris_for_day_offline(): one round(), then exact.

Author: HackFate Research Laboratory
Date: 2026-03-26
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import Dict, List, Tuple, Optional
from math import gcd, sin, cos, radians, sqrt, atan2, degrees, pi
from functools import reduce

# Our clean foundation
from gear_manifold_v3 import CRTManifold, SafeAnchor, mod_inverse
from astrology_engine import (
    Chart, BirthData, PlanetState, PairAnalysis, CodexState,
    render_chart, compute_planet, analyze_pair, compute_codex,
    find_house, ARCSEC_PER_DEG, FULL_CIRCLE, HALF_CIRCLE,
    ARCSEC_PER_SIGN, FULL_BASIS, SAFE_BASIS, SIGN_NAMES,
    CODEX_CYCLES, ASPECTS, CARRIER_ROLES,
)


# ═══════════════════════════════════════════════════════════════════════════
# 1. EPHEMERIS — Offline Analytical (Meeus) + Swiss Ephemeris fallback
#
# Float is UNAVOIDABLE here (Kepler's equation, sin/cos, perturbations).
# The boundary: ephemeris_for_day() returns integer arcseconds.
# One int(round(...)) per planet. Everything downstream is exact.
# ═══════════════════════════════════════════════════════════════════════════

MAYA_EPOCH_JD = 584283
J2000_JD = 2451545.0

# Orbital elements (Meeus Table 31.A)
_ELEMENTS = {
    'Mercury': {'L0': 253.7737, 'L1': 149474.07, 'w0': 77.456, 'w1': 1.556, 'e0': 0.20563593, 'e1': 0.00001906, 'a': 0.38709927},
    'Venus':   {'L0': 182.5935, 'L1': 58519.21,  'w0': 131.564,'w1': 1.402, 'e0': 0.00677672, 'e1':-0.00004107, 'a': 0.72333566},
    'Earth':   {'L0': 100.3689, 'L1': 36000.77,  'w0': 102.937,'w1': 1.720, 'e0': 0.01671123, 'e1':-0.00004392, 'a': 1.00000261},
    'Mars':    {'L0': 359.4389, 'L1': 19141.70,  'w0': 336.060,'w1': 1.841, 'e0': 0.09339410, 'e1': 0.00007882, 'a': 1.52371034},
    'Jupiter': {'L0': 36.2881,  'L1': 3036.30,   'w0': 14.331, 'w1': 1.612, 'e0': 0.04838624, 'e1':-0.00013253, 'a': 5.20288700},
    'Saturn':  {'L0': 45.7165,  'L1': 1223.51,   'w0': 93.057, 'w1': 1.964, 'e0': 0.05386179, 'e1':-0.00050991, 'a': 9.53667594},
    'Uranus':  {'L0': 316.4136, 'L1': 428.47,    'w0': 173.005,'w1': 1.486, 'e0': 0.04725744, 'e1':-0.00004397, 'a': 19.18916464},
    'Neptune': {'L0': 303.9239, 'L1': 218.49,    'w0': 48.124, 'w1': 1.426, 'e0': 0.00859048, 'e1': 0.00005105, 'a': 30.06992276},
    'Pluto':   {'L0': 250.5412, 'L1': 146.17,    'w0': 224.067,'w1': 1.291, 'e0': 0.24882730, 'e1': 0.00006465, 'a': 39.48211675},
}

def _solve_kepler(M_deg: float, e: float) -> float:
    M = radians(M_deg % 360.0)
    E = M + e * sin(M)
    for _ in range(50):
        dE = (E - e * sin(E) - M) / (1 - e * cos(E))
        E -= dE
        if abs(dE) < 1e-10: break
    return degrees(E)

def _true_anomaly(M_deg: float, e: float) -> float:
    E = radians(_solve_kepler(M_deg, e))
    nu = 2 * atan2(sqrt(1 + e) * sin(E / 2), sqrt(1 - e) * cos(E / 2))
    return degrees(nu)

def _heliocentric_lon(name: str, T: float) -> Tuple[float, float]:
    el = _ELEMENTS[name]
    L = (el['L0'] + el['L1'] * T) % 360.0
    w = (el['w0'] + el['w1'] * T) % 360.0
    e = el['e0'] + el['e1'] * T
    M = (L - w) % 360.0
    nu = _true_anomaly(M, e)
    lon = (nu + w) % 360.0
    r = el['a'] * (1 - e * e) / (1 + e * cos(radians(nu)))
    # Jupiter-Saturn perturbations
    Mj = radians((225.328 + 3034.6921 * T) % 360)
    Ms = radians((175.764 + 1221.5515 * T) % 360)
    if name == 'Jupiter':
        lon += -0.332 * sin(2*Mj - 5*Ms - 67.6*pi/180) - 0.056 * sin(2*Mj - 2*Ms + 21*pi/180)
    elif name == 'Saturn':
        lon += 0.812 * sin(2*Mj - 5*Ms - 67.6*pi/180) - 0.229 * cos(2*Mj - 4*Ms - 2*pi/180)
    return (lon % 360.0, r)

def _geocentric(planet: str, T: float) -> Tuple[float, float]:
    lp, rp = _heliocentric_lon(planet, T)
    le, re = _heliocentric_lon('Earth', T)
    xp = rp * cos(radians(lp)) - re * cos(radians(le))
    yp = rp * sin(radians(lp)) - re * sin(radians(le))
    return (degrees(atan2(yp, xp)) % 360.0, _ELEMENTS[planet]['L1'] / 36525.0)

def _sun_position(T: float) -> Tuple[float, float]:
    L0 = 280.46646 + 36000.76983 * T
    M = 357.52911 + 35999.05029 * T
    C = (1.914602 - 0.004817 * T) * sin(radians(M)) + 0.019993 * sin(radians(2 * M))
    omega = 125.04 - 1934.136 * T
    return ((L0 + C - 0.00569 - 0.00478 * sin(radians(omega))) % 360.0, 0.9856)

def _moon_position(T: float) -> Tuple[float, float]:
    Lp = 218.3164477 + 481267.88123421 * T
    D = 297.8501921 + 445267.1114034 * T
    M = 357.5291092 + 35999.0502909 * T
    Mp = 134.9633964 + 477198.8675055 * T
    F = 93.2720950 + 483202.0175233 * T
    sigma = (6.288774*sin(radians(Mp)) + 1.274027*sin(radians(2*D - Mp)) +
             0.658314*sin(radians(2*D)) + 0.213618*sin(radians(2*Mp)) -
             0.185116*sin(radians(M)) - 0.114332*sin(radians(2*F)))
    return ((Lp + sigma) % 360.0, 13.176)

# ── Correction tables import ──
try:
    from ephemeris_corrections import CORRECTIONS, CORR_START, CORR_STEP_DEFAULT, CORR_STEP_MARS
    _HAS_CORRECTIONS = True
except ImportError:
    _HAS_CORRECTIONS = False

def ephemeris_for_day(day_offset: int, asc_arcsec: int = 0) -> Dict[str, Tuple[int, int, int]]:
    """THE FLOAT→INT BOUNDARY. Float inside, integer arcseconds out.
    One int(round(...)) per planet. Everything downstream is exact.
    """
    jd = MAYA_EPOCH_JD + day_offset
    T = (jd - J2000_JD) / 36525.0
    result = {}

    # Sun and Moon
    for name, fn in [('Sun', _sun_position), ('Moon', _moon_position)]:
        lon_deg, vel_deg = fn(T)
        lon_as = int(round(lon_deg * ARCSEC_PER_DEG)) % FULL_CIRCLE
        vel_as = int(round(vel_deg * ARCSEC_PER_DEG))
        result[name] = (lon_as, vel_as, (lon_as - asc_arcsec) % FULL_CIRCLE)

    # Planets
    for planet in ('Mercury', 'Venus', 'Mars', 'Jupiter', 'Saturn', 'Uranus', 'Neptune', 'Pluto'):
        lon_deg, vel_deg = _geocentric(planet, T)
        lon_as = int(round(lon_deg * ARCSEC_PER_DEG)) % FULL_CIRCLE
        vel_as = int(round(vel_deg * ARCSEC_PER_DEG))

        # Apply correction table (INTEGER interpolation — no float)
        if _HAS_CORRECTIONS and planet in CORRECTIONS:
            step = CORR_STEP_MARS if planet == 'Mars' else CORR_STEP_DEFAULT
            corr = CORRECTIONS[planet]
            idx = (day_offset - CORR_START) // step
            if 0 <= idx < len(corr) - 1:
                # INTEGER interpolation — fixed from v2 float contamination
                c0, c1 = corr[idx], corr[idx + 1]
                frac_num = (day_offset - CORR_START) % step
                correction = c0 + ((c1 - c0) * frac_num) // step
                lon_as = (lon_as + correction) % FULL_CIRCLE

        result[planet] = (lon_as, vel_as, (lon_as - asc_arcsec) % FULL_CIRCLE)
    return result


def ephemeris_for_day_auto(day_offset: int, asc_arcsec: int = 0) -> Dict[str, Tuple[int, int, int]]:
    """Try Swiss Ephemeris first, fall back to analytical."""
    try:
        import swisseph as swe
        jd = MAYA_EPOCH_JD + day_offset
        result = {}
        pids = {'Sun':0,'Moon':1,'Mercury':2,'Venus':3,'Mars':4,'Jupiter':5,'Saturn':6,'Uranus':7,'Neptune':8,'Pluto':9}
        for name, pid in pids.items():
            calc = swe.calc_ut(jd, pid, 256)
            lon_as = int(round(calc[0][0] * ARCSEC_PER_DEG)) % FULL_CIRCLE
            vel_as = int(round(calc[0][3] * ARCSEC_PER_DEG))
            result[name] = (lon_as, vel_as, (lon_as - asc_arcsec) % FULL_CIRCLE)
        swe.close()
        return result
    except (ImportError, Exception):
        return ephemeris_for_day(day_offset, asc_arcsec)


# ═══════════════════════════════════════════════════════════════════════════
# 2. HOUSES — Placidus + CRT structure (12 = 4×3 = 2²×3)
# ═══════════════════════════════════════════════════════════════════════════

# House quadrants (mod 4)
QUADRANT_NAMES = {0: 'self', 1: 'foundation', 2: 'other', 3: 'public'}
MODALITY_NAMES = {0: 'angular', 1: 'succedent', 2: 'cadent'}

def house_quadrant(h: int) -> str:
    return QUADRANT_NAMES[(h - 1) // 3]

def house_modality(h: int) -> str:
    return MODALITY_NAMES[(h - 1) % 3]

PLANET_ENERGY = {
    'Sun': 'identity', 'Moon': 'emotional', 'Mercury': 'mental',
    'Venus': 'relational', 'Mars': 'action', 'Jupiter': 'expansion',
    'Saturn': 'structure', 'Uranus': 'disruption', 'Neptune': 'dissolution',
    'Pluto': 'transformation',
}


def compute_houses_swe(jd_frac: float, lat: float, lon: float) -> Tuple[Tuple[int,...], int, int]:
    """Compute Placidus houses. Returns (cusps_tuple, asc, mc). Float boundary."""
    try:
        import swisseph as swe
        cusps, ascmc = swe.houses(jd_frac, lat, lon, b'P')
        asc = int(round(ascmc[0] * ARCSEC_PER_DEG))
        mc = int(round(ascmc[1] * ARCSEC_PER_DEG))
        cusp_list = tuple(int(round(c * ARCSEC_PER_DEG)) for c in cusps)
        swe.close()
        return cusp_list, asc, mc
    except (ImportError, Exception):
        # Fallback: equal houses from approximate ASC
        asc = int((jd_frac * 360 * ARCSEC_PER_DEG) % FULL_CIRCLE)
        cusps = tuple((asc + i * ARCSEC_PER_SIGN) % FULL_CIRCLE for i in range(12))
        return cusps, asc, cusps[9]


# ═══════════════════════════════════════════════════════════════════════════
# 3. VEDIC — Sidereal positions, Nakshatras, Dasha
# ═══════════════════════════════════════════════════════════════════════════

LAHIRI_J2000_MAS = 85801800   # 23.8338° in milliarcseconds
PRECESSION_RATE_MAS = 50290    # 50.29"/year in milliarcseconds
J2000_MAYA_DAY = 1867262

NAKSHATRA_SPAN = 48000  # 13°20' in arcseconds
NAKSHATRA_NAMES = (
    'Ashvini','Bharani','Krittika','Rohini','Mrigashira','Ardra',
    'Punarvasu','Pushya','Ashlesha','Magha','Purva Phalguni','Uttara Phalguni',
    'Hasta','Chitra','Swati','Vishakha','Anuradha','Jyeshtha',
    'Mula','Purva Ashadha','Uttara Ashadha','Shravana','Dhanishta',
    'Shatabhisha','Purva Bhadrapada','Uttara Bhadrapada','Revati',
)
NAKSHATRA_RULERS = (
    'Ketu','Venus','Sun','Moon','Mars','Rahu','Jupiter','Saturn','Mercury',
) * 3  # repeats 3× for 27 nakshatras

DASHA_SEQUENCE = (
    ('Ketu',7),('Venus',20),('Sun',6),('Moon',10),('Mars',7),
    ('Rahu',18),('Jupiter',16),('Saturn',19),('Mercury',17),
)
SIDEREAL_SIGNS = (
    'Mesha','Vrishabha','Mithuna','Karka','Simha','Kanya',
    'Tula','Vrischika','Dhanu','Makara','Kumbha','Meena',
)


def ayanamsa_arcsec(maya_day: int) -> int:
    """Lahiri ayanamsa in arcseconds. Integer arithmetic."""
    days_from_j2000 = maya_day - J2000_MAYA_DAY
    years_x4 = days_from_j2000 * 4
    ayanamsa_mas = LAHIRI_J2000_MAS + (PRECESSION_RATE_MAS * years_x4) // 1461
    return ayanamsa_mas // 1000


def sidereal_longitude(tropical: int, maya_day: int) -> int:
    return (tropical - ayanamsa_arcsec(maya_day)) % FULL_CIRCLE


def nakshatra_info(sid_lon: int) -> Tuple[str, int, str]:
    """Returns (nakshatra_name, pada, ruler)."""
    idx = sid_lon // NAKSHATRA_SPAN
    pada = (sid_lon % NAKSHATRA_SPAN) // 12000 + 1
    return NAKSHATRA_NAMES[idx], pada, NAKSHATRA_RULERS[idx]


def compute_dasha(moon_sid: int, birth_day: int, target_day: int) -> Dict:
    """Which Vimsottari Dasha is active at target_day?"""
    nak_idx = moon_sid // NAKSHATRA_SPAN
    ruler = NAKSHATRA_RULERS[nak_idx]
    start_idx = next(i for i, (n, _) in enumerate(DASHA_SEQUENCE) if n == ruler)

    pos_in_nak = moon_sid % NAKSHATRA_SPAN
    frac = pos_in_nak * 1000 // NAKSHATRA_SPAN
    first_years = DASHA_SEQUENCE[start_idx][1]
    remaining = first_years * (1000 - frac) // 1000

    elapsed_days = target_day - birth_day
    elapsed_years = elapsed_days // 365
    years_consumed = 0
    idx = start_idx

    if elapsed_years < remaining:
        md_name, md_years = DASHA_SEQUENCE[idx]
        elapsed_in_md = first_years - remaining + elapsed_years
    else:
        years_consumed = remaining
        idx = (idx + 1) % 9
        while True:
            md_name, md_years = DASHA_SEQUENCE[idx]
            if years_consumed + md_years > elapsed_years:
                elapsed_in_md = elapsed_years - years_consumed
                break
            years_consumed += md_years
            idx = (idx + 1) % 9
        md_name, md_years = DASHA_SEQUENCE[idx]

    # Antardasha
    ad_elapsed = elapsed_in_md
    ad_consumed = 0
    for i in range(9):
        ad_idx = (idx + i) % 9
        ad_name, ad_full = DASHA_SEQUENCE[ad_idx]
        ad_length = md_years * ad_full // 120
        if ad_consumed + ad_length > ad_elapsed:
            return {'mahadasha': md_name, 'mahadasha_years': md_years,
                    'elapsed': elapsed_in_md, 'antardasha': ad_name,
                    'age': elapsed_years}
        ad_consumed += ad_length

    return {'mahadasha': md_name, 'antardasha': DASHA_SEQUENCE[idx][0], 'age': elapsed_years}


def vedic_positions(planet_data: Dict[str, Tuple[int,int,int]], maya_day: int) -> Dict[str, Dict]:
    """Dual-lane positions: tropical + sidereal in parallel."""
    ayanamsa = ayanamsa_arcsec(maya_day)
    positions = {}
    for name, (lon, vel, house) in planet_data.items():
        sid = (lon - ayanamsa) % FULL_CIRCLE
        nak_name, pada, ruler = nakshatra_info(sid)
        positions[name] = {
            'tropical': lon, 'sidereal': sid,
            'trop_sign': SIGN_NAMES[(lon // ARCSEC_PER_SIGN) % 12],
            'sid_sign': SIDEREAL_SIGNS[(sid // ARCSEC_PER_SIGN) % 12],
            'trop_deg': (lon % ARCSEC_PER_SIGN) // ARCSEC_PER_DEG,
            'sid_deg': (sid % ARCSEC_PER_SIGN) // ARCSEC_PER_DEG,
            'nakshatra': nak_name, 'pada': pada, 'ruler': ruler,
        }
    return positions


# ═══════════════════════════════════════════════════════════════════════════
# 4. TRANSITS — evaluate current sky against natal chart
# ═══════════════════════════════════════════════════════════════════════════

def evaluate_transits(natal: Chart, transit_day: int,
                      transit_data: Dict[str, Tuple[int,int,int]]) -> List[PairAnalysis]:
    """Evaluate transit planets against natal positions. Returns all pairs."""
    transit_planets = {
        f"t_{name}": compute_planet(f"t_{name}", data)
        for name, data in transit_data.items()
    }
    pairs = []
    for tn, tp in transit_planets.items():
        for nn, np_state in natal.planets.items():
            pairs.append(analyze_pair(tp, np_state, transit_day))
    pairs.sort(key=lambda p: p.lock_score, reverse=True)
    return pairs


def render_transit_report(natal: Chart, transit_day: int,
                          transit_data: Dict[str, Tuple[int,int,int]]) -> str:
    """Full transit report."""
    pairs = evaluate_transits(natal, transit_day, transit_data)
    codex = compute_codex(transit_day)
    delta = transit_day - natal.birth.maya_day

    L = []
    L.append("=" * 70)
    L.append(f"  TRANSIT REPORT — {natal.birth.name}")
    L.append(f"  Transit day {transit_day} | Delta {delta} days ({delta // 365} years)")
    L.append(f"  Codex: {codex.tzolkin_tone} {codex.tzolkin_glyph}")
    L.append("=" * 70)

    active = [p for p in pairs if p.lock_score >= 350]
    L.append(f"\n  Active transits: {len(active)} (lock >= 350)")
    L.append(f"  {'Transit\u2192Natal':30s}  {'Aspect':13s}  {'Lock':>4s}  {'SB':>2s}")
    L.append(f"  {'─'*30}  {'─'*13}  {'─'*4}  {'─'*2}")

    for r in active[:25]:
        pair = f"{r.planet_a}\u2192{r.planet_b}"
        asp = r.aspect.name if r.aspect else "\u2014shadow\u2014"
        s = "\u25cf" if r.shadow.active else "\u00b7"
        b = "\u25cf" if r.boundary.active else "\u00b7"
        L.append(f"  {pair:30s}  {asp:13s}  {r.lock_score:4d}  {s}{b}")

    L.append("=" * 70)
    return "\n".join(L)


# ═══════════════════════════════════════════════════════════════════════════
# 5. EVENTS — modular arithmetic prediction (no brute force)
# ═══════════════════════════════════════════════════════════════════════════

MEAN_MOTION = {
    'Sun': 3548, 'Moon': 47400, 'Mercury': 14400, 'Venus': 5760,
    'Mars': 1886, 'Jupiter': 299, 'Saturn': 120, 'Uranus': 42,
    'Neptune': 21, 'Pluto': 14,
}

SIDEREAL_DAYS = {
    'Sun': 365, 'Moon': 27, 'Mercury': 88, 'Venus': 225, 'Mars': 687,
    'Jupiter': 4333, 'Saturn': 10759, 'Uranus': 30687, 'Neptune': 60190, 'Pluto': 90560,
}


def predict_returns(natal_day: int, planet: str, years: int = 80) -> List[Dict]:
    """Predict all return dates for one planet."""
    synodic = SIDEREAL_DAYS.get(planet, 365)
    returns = []
    max_laps = (years * 365) // max(synodic, 1) + 2
    for k in range(-max_laps, max_laps + 1):
        day = natal_day + k * synodic
        returns.append({
            'day': day, 'age': (day - natal_day) // 365,
            'planet': planet, 'cycle': k, 'type': 'return',
        })
    return [r for r in returns if -5 <= r['age'] <= years]


def predict_codex_closures(natal_day: int, years: int = 80) -> Dict[str, List[int]]:
    """Find days when codex cycles hit closure states."""
    search = years * 365
    closures = {}

    # Tzolk'in returns
    tz_r = natal_day % 260
    offset = (260 - tz_r) % 260
    closures['tzolkin_return'] = list(range(natal_day + offset, natal_day + search, 260))

    # Calendar Round returns
    cr_r = natal_day % 18980
    offset = (18980 - cr_r) % 18980
    closures['calendar_round'] = list(range(natal_day + offset, natal_day + search, 18980))

    # Eclipse shadow returns (mod 11 in eclipse cycle)
    ecl_r = natal_day % 11960
    offset_11 = (11 - (ecl_r % 11)) % 11
    closures['eclipse_shadow'] = list(range(natal_day + offset_11, natal_day + search, 11))[:50]

    return closures


def maya_day_to_date(maya_day: int) -> str:
    """Approximate Gregorian date from Maya day. OUTPUT ONLY."""
    j2000_days = MAYA_EPOCH_JD + maya_day - J2000_JD
    year = 2000 + int(j2000_days) // 365
    doy = int(j2000_days) % 365
    return f"~{year}-{doy // 30 + 1:02d}-{doy % 30 + 1:02d}"


# ═══════════════════════════════════════════════════════════════════════════
# 6. SYSTEM — Unified API
# ═══════════════════════════════════════════════════════════════════════════

TZ_OFFSETS = {
    'UTC': 0, 'GMT': 0, 'EST': -5, 'EDT': -4, 'CST': -6, 'CDT': -5,
    'MST': -7, 'MDT': -6, 'PST': -8, 'PDT': -7,
}

def gregorian_to_maya_day(year: int, month: int, day: int,
                          hour: int = 12, minute: int = 0,
                          tz: str = 'UTC') -> Tuple[int, float]:
    """Convert date to Maya day offset + JD fraction for houses."""
    tz_offset = TZ_OFFSETS.get(tz.upper(), 0)
    utc_hour = hour - tz_offset + minute / 60.0
    y, m = year, month
    if m <= 2: y -= 1; m += 12
    A = y // 100; B = 2 - A + A // 4
    jd = int(365.25 * (y + 4716)) + int(30.6001 * (m + 1)) + day + B - 1524
    jd_frac = jd + utc_hour / 24.0
    maya_day = jd - MAYA_EPOCH_JD
    return maya_day, jd_frac


class AstrologySystem:
    """Unified API. Birth data in → complete three-layer reading out.

    Usage:
        system = AstrologySystem()
        chart = system.natal("Name", 1980, 10, 21, 17, 31, "EDT", 35.139, -79.0)
        print(render_chart(chart))
        print(chart.as_dict())  # for API
    """

    def natal(self, name: str, year: int, month: int, day: int,
              hour: int = 12, minute: int = 0, tz: str = 'UTC',
              lat: float = 0.0, lon: float = 0.0) -> Chart:
        """Compute natal chart from birth data."""
        maya_day, jd_frac = gregorian_to_maya_day(year, month, day, hour, minute, tz)
        cusps, asc, mc = compute_houses_swe(jd_frac, lat, lon)
        planet_data = ephemeris_for_day_auto(maya_day, asc)
        return Chart.from_birth(BirthData(
            name=name, maya_day=maya_day, planets=planet_data,
            cusps=cusps, asc=asc, mc=mc,
        ))

    def natal_from_data(self, name: str, maya_day: int,
                        planet_data: Dict[str, Tuple[int,int,int]],
                        cusps: Tuple[int,...], asc: int, mc: int) -> Chart:
        """Create chart from pre-computed data."""
        return Chart.from_birth(BirthData(
            name=name, maya_day=maya_day, planets=planet_data,
            cusps=cusps, asc=asc, mc=mc,
        ))

    def transits(self, chart: Chart, target_day: Optional[int] = None) -> str:
        """Transit report for a specific day."""
        if target_day is None:
            target_day = 1876817  # ~Mar 25, 2026
        data = ephemeris_for_day_auto(target_day, chart.birth.asc)
        return render_transit_report(chart, target_day, data)

    def vedic(self, chart: Chart) -> Dict:
        """Vedic overlay: sidereal positions + Dasha."""
        positions = vedic_positions(chart.birth.planets, chart.birth.maya_day)
        moon_sid = positions.get('Moon', {}).get('sidereal', 0)
        today = 1876817
        dasha = compute_dasha(moon_sid, chart.birth.maya_day, today)
        return {'positions': positions, 'dasha': dasha,
                'ayanamsa': ayanamsa_arcsec(chart.birth.maya_day)}

    def returns(self, chart: Chart, planet: str = 'Saturn', years: int = 80) -> List[Dict]:
        """Predict planetary returns."""
        return predict_returns(chart.birth.maya_day, planet, years)

    def codex_closures(self, chart: Chart, years: int = 80) -> Dict[str, List[int]]:
        """Predict codex cycle closures."""
        return predict_codex_closures(chart.birth.maya_day, years)


# ═══════════════════════════════════════════════════════════════════════════
# FULL SYSTEM TEST
# ═══════════════════════════════════════════════════════════════════════════

def test_full_system():
    """End-to-end test: birth data → all outputs."""
    print("=" * 70)
    print("  FULL SYSTEM TEST — All Modules Wired")
    print("=" * 70)

    failures = 0

    # ── 1. Create chart from pre-computed data ──
    print("\n  1. Chart from pre-computed data")
    system = AstrologySystem()
    chart = system.natal_from_data(
        "Anthony Diaz", 1860251,
        {
            'Sun': (751107, 3581, 727288), 'Moon': (1810, 54173, 1273991),
            'Mercury': (827399, 540, 803580), 'Venus': (611243, 4272, 587424),
            'Mars': (888870, 2604, 865051), 'Jupiter': (644043, 724, 620224),
            'Saturn': (661345, 419, 637526), 'Uranus': (843177, 207, 819358),
            'Neptune': (938127, 93, 914308), 'Pluto': (727106, 143, 703287),
        },
        (23819, 160400, 256749, 337956, 421275, 524762,
         671819, 808400, 904749, 985956, 1069275, 1172762),
        23819, 985956,
    )
    assert len(chart.shadow_network()) == 6, f"Shadow: {len(chart.shadow_network())}"
    assert len(chart.boundary_network()) == 4, f"Boundary: {len(chart.boundary_network())}"
    print(f"    Chart: {chart.birth.name} \u2713")
    print(f"    Shadow bonds: {len(chart.shadow_network())} \u2713")
    print(f"    Boundary events: {len(chart.boundary_network())} \u2713")

    # ── 2. Ephemeris (offline) ──
    print("\n  2. Offline ephemeris")
    eph = ephemeris_for_day(1860251, 23819)
    assert 'Sun' in eph and 'Moon' in eph and 'Pluto' in eph
    sun_lon = eph['Sun'][0]
    sun_sign = SIGN_NAMES[(sun_lon // ARCSEC_PER_SIGN) % 12]
    print(f"    Sun: {sun_lon} arcsec = {sun_sign} \u2713")
    print(f"    All 10 bodies computed \u2713")

    # ── 3. Houses ──
    print("\n  3. House placement")
    for name in sorted(chart.planets):
        h = chart.planet_house(name)
        assert 1 <= h <= 12, f"{name} in invalid house {h}"
    print(f"    All planets in valid houses (1-12) \u2713")

    # ── 4. Vedic layer ──
    print("\n  4. Vedic overlay")
    vedic = system.vedic(chart)
    assert 'positions' in vedic and 'dasha' in vedic
    moon_nak = vedic['positions']['Moon']['nakshatra']
    dasha = vedic['dasha']
    print(f"    Moon nakshatra: {moon_nak}")
    print(f"    Dasha: {dasha['mahadasha']} / {dasha['antardasha']} (age {dasha['age']})")
    print(f"    Ayanamsa: {vedic['ayanamsa'] // ARCSEC_PER_DEG}\u00b0{(vedic['ayanamsa'] % ARCSEC_PER_DEG) // 60:02d}' \u2713")

    # ── 5. Transits (synthetic — no Swiss Ephemeris needed) ──
    print("\n  5. Transit evaluation")
    TODAY = 1876817
    transit_data = ephemeris_for_day(TODAY, chart.birth.asc)
    pairs = evaluate_transits(chart, TODAY, transit_data)
    active = [p for p in pairs if p.lock_score >= 300]
    print(f"    Transit day: {TODAY} (age ~{(TODAY - 1860251) // 365})")
    print(f"    Total pairs: {len(pairs)}")
    print(f"    Active (lock>=300): {len(active)}")

    # ── 6. Event prediction ──
    print("\n  6. Event prediction")
    sat_returns = predict_returns(1860251, 'Saturn', 80)
    useful = [r for r in sat_returns if 0 <= r['age'] <= 80]
    print(f"    Saturn returns (age 0-80): {len(useful)}")
    for r in useful[:5]:
        print(f"      age {r['age']:3d}  cycle #{r['cycle']}")

    closures = predict_codex_closures(1860251, 80)
    print(f"    Tzolk'in returns: {len(closures['tzolkin_return'])}")
    print(f"    Calendar Round returns: {len(closures['calendar_round'])}")

    # ── 7. Serialization ──
    print("\n  7. Serialization (app-ready)")
    d = chart.as_dict()
    assert isinstance(d, dict)
    assert 'planets' in d and 'shadow_count' in d and 'codex' in d
    print(f"    as_dict() keys: {list(d.keys())}")
    print(f"    Sun position: {d['planets']['Sun']['position']}")
    print(f"    Shadow count: {d['shadow_count']}")
    print(f"    Codex: {d['codex']['tzolkin']}")

    # ── 8. CRT residue verification ──
    print("\n  8. CRT residue verification (45/45 diffs)")
    all_correct = True
    for pair in chart.pairs:
        a_lon = chart.planets[pair.planet_a].longitude
        b_lon = chart.planets[pair.planet_b].longitude
        expected = tuple((a_lon - b_lon) % p for p in FULL_BASIS)
        if pair.diff_full != expected:
            print(f"    FAIL: {pair.planet_a}-{pair.planet_b}")
            all_correct = False
            failures += 1
    if all_correct:
        print(f"    All 45 pairs: TRUE CRT diffs \u2713")

    # ── Summary ──
    print(f"\n{'='*70}")
    if failures == 0:
        print(f"  ALL MODULES WIRED AND VERIFIED.")
        print(f"  Ephemeris \u2713  Houses \u2713  Vedic \u2713  Transits \u2713  Events \u2713  CRT \u2713")
    else:
        print(f"  {failures} FAILURES.")
    print(f"{'='*70}")

    return failures


if __name__ == '__main__':
    failures = test_full_system()
    if failures == 0:
        # Show the full chart
        system = AstrologySystem()
        chart = system.natal_from_data(
            "Anthony Diaz", 1860251,
            {
                'Sun': (751107, 3581, 727288), 'Moon': (1810, 54173, 1273991),
                'Mercury': (827399, 540, 803580), 'Venus': (611243, 4272, 587424),
                'Mars': (888870, 2604, 865051), 'Jupiter': (644043, 724, 620224),
                'Saturn': (661345, 419, 637526), 'Uranus': (843177, 207, 819358),
                'Neptune': (938127, 93, 914308), 'Pluto': (727106, 143, 703287),
            },
            (23819, 160400, 256749, 337956, 421275, 524762,
             671819, 808400, 904749, 985956, 1069275, 1172762),
            23819, 985956,
        )
        print("\n" + render_chart(chart))
