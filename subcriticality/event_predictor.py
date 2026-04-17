#!/usr/bin/env python3
"""
EVENT PREDICTOR — All-At-Once Alignment Scanner
=================================================

Given one natal chart, predict ALL past and future significant dates.
No brute-force day scanning. Uses modular arithmetic to solve for
exact recurrence dates directly.

The insight: planetary motion is approximately linear over short periods.
If transit planet moves at v arcsec/day relative to natal position,
then the time to reach a specific residue pattern is a modular equation.

For exact dates: solve d ≡ target (mod period) for each channel.
For aspect dates: solve |Δλ(t) - aspect_angle| = 0.
For shadow dates: solve Δλ(t) ≡ 0 (mod 11).
For boundary dates: solve Δλ(t) ≡ 0 (mod 13).
For returns: solve Δλ(t) ≡ 0 (mod p) for all p in basis.

Then combine with CRT to find when multiple conditions fire simultaneously.

Depends on: astrology_engine.py, transit_engine.py
"""

from __future__ import annotations
from typing import Dict, List, Tuple, Optional
from math import gcd
from functools import reduce

from astrology_engine import (
    PlanetManifold, CodexManifold, PairRelation, BoundaryState,
    ARCSEC_PER_DEG, FULL_CIRCLE, HALF_CIRCLE, ARCSEC_PER_SIGN,
    SAFE_BASIS, FULL_BASIS, ASPECT_TABLE, ASPECT_TONES,
    S_R_PRIMES, SIGN_NAMES,
)
from transit_engine import (
    TransitEngine, detect_return, transit_is_active, ephemeris_for_day,
    MAYA_EPOCH_JD, PLANET_IDS,
)


# ═══════════════════════════════════════════════════════════════════════════
# SYNODIC PERIODS — mean daily motion (arcseconds/day, integer)
# These are the AVERAGE rates. Real motion varies, but these give
# the modular structure for prediction.
# ═══════════════════════════════════════════════════════════════════════════

# Mean daily motion in arcseconds (from astronomical constants)
MEAN_MOTION = {
    'Sun':     3548,    # ~0.986°/day
    'Moon':    47400,   # ~13.17°/day
    'Mercury': 14400,   # ~4.09°/day (mean, varies wildly)
    'Venus':   5760,    # ~1.60°/day
    'Mars':    1886,    # ~0.524°/day
    'Jupiter':  299,    # ~0.083°/day
    'Saturn':   120,    # ~0.033°/day
    'Uranus':    42,    # ~0.012°/day
    'Neptune':   21,    # ~0.006°/day
    'Pluto':     14,    # ~0.004°/day
}

# Sidereal (orbital) periods in days — for RETURN prediction
SIDEREAL_DAYS = {
    'Sun':      365,    # N/A but useful as year marker
    'Moon':      27,    # sidereal month
    'Mercury':   88,    # orbital period
    'Venus':    225,    # orbital period
    'Mars':     687,    # orbital period
    'Jupiter': 4333,    # ~11.86 years
    'Saturn':  10759,   # ~29.46 years
    'Uranus':  30687,   # ~84.01 years
    'Neptune': 60190,   # ~164.8 years
    'Pluto':   90560,   # ~247.9 years
}


# ═══════════════════════════════════════════════════════════════════════════
# MODULAR EVENT SOLVER — the core innovation
# ═══════════════════════════════════════════════════════════════════════════

def solve_aspect_dates(natal_lon: int, transit_speed: int,
                       aspect_angle: int, search_range_days: int,
                       natal_day: int) -> List[int]:
    """Solve: on which days does the transit planet form this exact aspect
    to the natal position?

    The transit planet moves at ~transit_speed arcsec/day.
    Aspect is exact when: (natal_lon + transit_speed * t) mod FULL_CIRCLE ≡ natal_lon ± aspect_angle

    This is a linear congruence: transit_speed * t ≡ aspect_angle (mod FULL_CIRCLE)

    Returns list of day offsets from natal_day.
    """
    if transit_speed == 0:
        return []

    results = []
    # Target separations: aspect_angle and FULL_CIRCLE - aspect_angle
    targets = [aspect_angle]
    if aspect_angle > 0 and aspect_angle < FULL_CIRCLE:
        targets.append(FULL_CIRCLE - aspect_angle)

    for target in targets:
        # Solve: speed * t ≡ target (mod FULL_CIRCLE)
        g = gcd(abs(transit_speed), FULL_CIRCLE)
        if target % g != 0:
            continue  # no solution

        # Reduced equation
        s_red = transit_speed // g
        t_red = target // g
        m_red = FULL_CIRCLE // g

        # Find modular inverse of s_red mod m_red
        try:
            inv = _mod_inverse(s_red % m_red, m_red)
        except ValueError:
            continue

        # Base solution
        t0 = (t_red * inv) % m_red

        # All solutions: t = t0 + k * m_red
        t = t0
        while t < search_range_days:
            if t > 0:  # future only (or past if negative search)
                results.append(natal_day + t)
            t += m_red

        # Also search backward
        t = t0 - m_red
        while t > -search_range_days:
            if t < 0:
                results.append(natal_day + t)
            t -= m_red

    results.sort()
    return results


def solve_shadow_dates(natal_lon: int, transit_speed: int,
                       search_range_days: int, natal_day: int) -> List[int]:
    """Solve: when does transit-natal difference hit zero in lane 11?

    Δλ(t) ≡ 0 (mod 11) when transit_speed * t ≡ 0 (mod 11)

    Period: 11 / gcd(speed, 11) days between shadow activations.
    """
    speed_mod_11 = abs(transit_speed) % 11
    if speed_mod_11 == 0:
        # Every day is shadow-active for this pair (speed is multiple of 11)
        return list(range(natal_day - search_range_days,
                          natal_day + search_range_days, 1))

    # Period between shadow activations
    period = 11  # since gcd(speed_mod_11, 11) = 1 for prime 11, period = 11

    # First activation: solve speed * t ≡ -natal_residue (mod 11)
    natal_r11 = natal_lon % 11
    # We need: (natal_lon + speed * t) % 11 == natal_lon % 11
    # i.e., speed * t ≡ 0 (mod 11)
    # First t: t = 11 / gcd(speed%11, 11) ... but 11 is prime, so period = 11
    # unless speed%11 == 0

    results = []
    # The shadow lane fires every `period` days from some offset
    # Find the offset by checking the first few days
    for start in range(period):
        if (transit_speed * start) % 11 == 0:
            t = start
            while t < search_range_days:
                results.append(natal_day + t)
                t += period
            t = start - period
            while t > -search_range_days:
                results.append(natal_day + t)
                t -= period
            break

    results.sort()
    return results


def solve_boundary_dates(natal_lon: int, transit_speed: int,
                         search_range_days: int, natal_day: int) -> List[int]:
    """Same as shadow but for lane 13."""
    speed_mod_13 = abs(transit_speed) % 13
    if speed_mod_13 == 0:
        return list(range(natal_day, natal_day + search_range_days, 1))

    period = 13
    results = []
    for start in range(period):
        if (transit_speed * start) % 13 == 0:
            t = start
            while t < search_range_days:
                results.append(natal_day + t)
                t += period
            t = start - period
            while t > -search_range_days:
                results.append(natal_day + t)
                t -= period
            break

    results.sort()
    return results


def solve_codex_closure_dates(natal_day: int,
                              search_range_days: int) -> Dict[str, List[int]]:
    """Find days when Codex cycles hit specific closure states.

    Eclipse shadow return: delta ≡ 0 (mod 11) in eclipse cycle
    Boundary return: delta ≡ 0 (mod 13) in 819 cycle
    Full cycle returns: delta ≡ 0 (mod cycle_length)
    """
    from astrology_engine import CODEX_CYCLES

    closures = {}

    # Shadow returns via eclipse cycle: (natal_day + t) % 11960 ≡ 0 (mod 11)
    eclipse_r = natal_day % 11960
    # Next: need (eclipse_r + t) % 11 == 0
    offset_11 = (11 - (eclipse_r % 11)) % 11
    eclipse_shadow_dates = list(range(natal_day + offset_11,
                                      natal_day + search_range_days, 11))
    closures['eclipse_shadow'] = eclipse_shadow_dates

    # Boundary returns via 819: (natal_day + t) % 819 ≡ 0 (mod 13)
    c819_r = natal_day % 819
    offset_13 = (13 - (c819_r % 13)) % 13
    boundary_dates = list(range(natal_day + offset_13,
                                natal_day + search_range_days, 13))
    closures['819_boundary'] = boundary_dates

    # Full Tzolk'in returns: delta ≡ 0 (mod 260)
    tz_r = natal_day % 260
    offset_tz = (260 - tz_r) % 260
    closures['tzolkin_return'] = list(range(natal_day + offset_tz,
                                            natal_day + search_range_days, 260))

    # Calendar Round returns: delta ≡ 0 (mod 18980)
    cr_r = natal_day % 18980
    offset_cr = (18980 - cr_r) % 18980
    closures['calendar_round'] = list(range(natal_day + offset_cr,
                                            natal_day + search_range_days, 18980))

    return closures


# ═══════════════════════════════════════════════════════════════════════════
# ALL-AT-ONCE SCANNER — the complete prediction engine
# ═══════════════════════════════════════════════════════════════════════════

class EventPredictor:
    """Given a natal chart, predict ALL significant alignment dates.

    Uses modular arithmetic — not brute-force scanning.
    One natal chart → complete event timeline.
    """

    def __init__(self, natal_day: int,
                 natal_data: Dict[str, Tuple[int, int, int]]):
        self.natal_day = natal_day
        self.natal_data = natal_data
        self.natal_planets = {n: PlanetManifold(n, *d)
                              for n, d in natal_data.items()}

    def predict_aspects(self, transit_planet: str, natal_planet: str,
                        years_forward: int = 100, years_back: int = 45
                        ) -> List[Dict]:
        """Predict all exact aspect dates between one transit-natal pair.

        Uses two methods:
        1. Linear approximation: t ≈ (target - initial_sep) / speed
        2. Modular solver when exact solution exists

        Returns candidate dates (approximate — ephemeris refines).
        """
        natal_lon = self.natal_data[natal_planet][0]
        transit_natal_lon = self.natal_data[transit_planet][0]
        speed = MEAN_MOTION.get(transit_planet, 1000)
        if speed == 0:
            return []

        # Initial separation: where transit planet starts relative to natal target
        init_sep = (natal_lon - transit_natal_lon) % FULL_CIRCLE

        events = []
        search_days = years_forward * 365
        search_back = years_back * 365

        for h, (exact_angle, name, max_orb) in ASPECT_TABLE.items():
            # Targets: init_sep + aspect_angle, init_sep - aspect_angle
            for sign in (1, -1):
                target = (init_sep + sign * exact_angle) % FULL_CIRCLE
                if target == 0 and exact_angle > 0 and sign == -1:
                    continue  # avoid duplicate conjunction

                # Linear approximation: t ≈ target / speed
                # But target wraps, so check multiple laps
                sidereal = SIDEREAL_DAYS.get(transit_planet, 36500)
                max_laps = (search_days + search_back) // max(sidereal, 1) + 2

                for lap in range(-max_laps, max_laps + 1):
                    t_approx = (target + lap * FULL_CIRCLE) // max(speed, 1)

                    if -search_back <= t_approx <= search_days:
                        day = self.natal_day + t_approx
                        age = t_approx // 365
                        events.append({
                            'day': day,
                            'delta_days': t_approx,
                            'age': age,
                            'transit': transit_planet,
                            'natal': natal_planet,
                            'aspect': name,
                            'harmonic': h,
                            'tone': ASPECT_TONES.get(name, 0),
                            'approx': True,  # candidate — needs ephemeris refinement
                        })

        # Deduplicate: events within 30 days of each other are the same event
        events.sort(key=lambda e: e['day'])
        deduped = []
        for e in events:
            if not deduped or abs(e['day'] - deduped[-1]['day']) > 30:
                deduped.append(e)

        return deduped

    def predict_shadow_activations(self, transit_planet: str,
                                   natal_planet: str,
                                   years: int = 100) -> List[int]:
        """Predict all shadow (lane-11) activation dates."""
        rel_speed = MEAN_MOTION.get(transit_planet, 1000)
        return solve_shadow_dates(
            self.natal_data[natal_planet][0], rel_speed,
            years * 365, self.natal_day)

    def predict_boundary_activations(self, transit_planet: str,
                                     natal_planet: str,
                                     years: int = 100) -> List[int]:
        """Predict all boundary (lane-13) activation dates."""
        rel_speed = MEAN_MOTION.get(transit_planet, 1000)
        return solve_boundary_dates(
            self.natal_data[natal_planet][0], rel_speed,
            years * 365, self.natal_day)

    def predict_codex_closures(self, years: int = 100) -> Dict[str, List[int]]:
        """Predict all Codex cycle closure dates."""
        return solve_codex_closure_dates(self.natal_day, years * 365)

    def predict_returns(self, planet: str,
                        years: int = 100) -> List[Dict]:
        """Predict all return dates for one planet.
        Return = transit planet returns to natal longitude.
        Period ≈ synodic period for outer planets, sidereal for inner.
        """
        synodic = SIDEREAL_DAYS.get(planet, 365)
        natal_lon = self.natal_data[planet][0]

        returns = []
        # Returns happen approximately every synodic period
        for k in range(-years * 365 // max(synodic, 1),
                       years * 365 // max(synodic, 1) + 1):
            approx_day = self.natal_day + k * synodic
            returns.append({
                'day': approx_day,
                'delta_days': approx_day - self.natal_day,
                'age': (approx_day - self.natal_day) // 365,
                'planet': planet,
                'cycle': k,
                'type': 'return',
            })

        return [r for r in returns if r['delta_days'] >= -years * 365]

    def full_timeline(self, years_forward: int = 80,
                      years_back: int = 45) -> List[Dict]:
        """Generate the complete event timeline for all planet pairs.

        This is the "all-at-once" scan.
        One natal chart → complete past + future timeline.
        """
        all_events = []

        # Planet pairs: slow movers transiting natal positions
        slow_movers = ['Saturn', 'Uranus', 'Neptune', 'Pluto', 'Jupiter']
        all_planets = list(self.natal_planets.keys())

        for transit in slow_movers:
            for natal in all_planets:
                aspects = self.predict_aspects(
                    transit, natal, years_forward, years_back)
                # Keep only major aspects for timeline
                major = [a for a in aspects
                         if a['harmonic'] in (1, 2, 3, 4, 6)]
                all_events.extend(major)

        # Returns for slow movers
        for planet in slow_movers:
            returns = self.predict_returns(planet, years_forward)
            all_events.extend(returns)

        # Codex closures
        closures = self.predict_codex_closures(years_forward)
        for closure_type, dates in closures.items():
            for day in dates[:20]:  # limit for display
                all_events.append({
                    'day': day,
                    'delta_days': day - self.natal_day,
                    'age': (day - self.natal_day) // 365,
                    'type': closure_type,
                })

        all_events.sort(key=lambda e: e['day'])
        return all_events

    def events_at_age(self, age: int, window_days: int = 30) -> List[Dict]:
        """What events fire around a specific age?"""
        center_day = self.natal_day + age * 365
        timeline = self.full_timeline(age + 1, 0)
        return [e for e in timeline
                if abs(e['day'] - center_day) <= window_days]


# ═══════════════════════════════════════════════════════════════════════════
# UTILITIES
# ═══════════════════════════════════════════════════════════════════════════

def _mod_inverse(a: int, m: int) -> int:
    if m == 1: return 0
    old_r, r = a % m, m
    old_s, s = 1, 0
    while r != 0:
        q = old_r // r
        old_r, r = r, old_r - q * r
        old_s, s = s, old_s - q * s
    if old_r != 1:
        raise ValueError(f"No inverse: gcd({a},{m})={old_r}")
    return old_s % m


def maya_day_to_date(maya_day: int) -> str:
    """Convert Maya day offset to approximate Gregorian date. OUTPUT ONLY."""
    # JD = MAYA_EPOCH_JD + maya_day
    # J2000 = JD 2451545.0
    # Gregorian approximation from J2000
    jd = MAYA_EPOCH_JD + maya_day
    j2000_days = jd - 2451545
    year = 2000 + j2000_days // 365
    day_of_year = j2000_days % 365
    month = day_of_year // 30 + 1
    day = day_of_year % 30 + 1
    return f"~{year}-{month:02d}-{day:02d}"


# ═══════════════════════════════════════════════════════════════════════════
# DEMO
# ═══════════════════════════════════════════════════════════════════════════

if __name__ == '__main__':
    NATAL_DAY = 1860251
    NATAL = {
        'Sun':     (751107,  3581,  727288),
        'Moon':    (  1810, 54173, 1273991),
        'Mercury': (827399,   540,  803580),
        'Venus':   (611243,  4272,  587424),
        'Mars':    (888870,  2604,  865051),
        'Jupiter': (644043,   724,  620224),
        'Saturn':  (661345,   419,  637526),
        'Uranus':  (843177,   207,  819358),
        'Neptune': (938127,    93,  914308),
        'Pluto':   (727106,   143,  703287),
    }

    predictor = EventPredictor(NATAL_DAY, NATAL)

    print("=" * 70)
    print("  EVENT PREDICTOR — All-At-Once Alignment Scanner")
    print("  One natal chart → complete event timeline")
    print("  No brute-force. Modular arithmetic solver.")
    print("=" * 70)

    # ── Saturn transits to natal Sun (most important slow transit) ──
    print(f"\n── Saturn → natal Sun: all major aspects (life timeline) ──")
    sat_sun = predictor.predict_aspects('Saturn', 'Sun', 80, 45)
    major = [e for e in sat_sun if e['harmonic'] in (1, 2, 3, 4)]
    for e in major[:30]:
        date = maya_day_to_date(e['day'])
        print(f"  age {e['age']:3d}  {date}  {e['aspect']:13s}  tone={e['tone']}")

    # ── Pluto transits to natal Sun ──
    print(f"\n── Pluto → natal Sun: all major aspects ──")
    plu_sun = predictor.predict_aspects('Pluto', 'Sun', 80, 45)
    major_p = [e for e in plu_sun if e['harmonic'] in (1, 2, 3, 4)]
    for e in major_p[:20]:
        date = maya_day_to_date(e['day'])
        print(f"  age {e['age']:3d}  {date}  {e['aspect']:13s}  tone={e['tone']}")

    # ── Saturn returns ──
    print(f"\n── Saturn returns (age 0-80) ──")
    sat_ret = predictor.predict_returns('Saturn', 80)
    for r in sat_ret:
        if 0 <= r['age'] <= 80:
            date = maya_day_to_date(r['day'])
            print(f"  age {r['age']:3d}  {date}  cycle #{r['cycle']}")

    # ── Codex closures ──
    print(f"\n── Codex cycle closures (next 20) ──")
    closures = predictor.predict_codex_closures(80)
    for ctype, dates in closures.items():
        print(f"\n  {ctype}:")
        for d in dates[:5]:
            date = maya_day_to_date(d)
            age = (d - NATAL_DAY) // 365
            print(f"    age {age:3d}  {date}")

    # ── Events at specific ages ──
    for age in [29, 30, 42, 45, 50]:
        events = predictor.events_at_age(age)
        if events:
            print(f"\n── Events at age {age} (±30 days) ──")
            for e in events[:10]:
                etype = e.get('aspect', e.get('type', '?'))
                detail = f"{e.get('transit','')}{' → ' + e.get('natal','') if 'natal' in e else ''}"
                print(f"    {etype:20s}  {detail}")

    print(f"\n{'='*70}")
    print(f"  Timeline computed from modular structure.")
    print(f"  No ephemeris needed for candidate dates.")
    print(f"  Ephemeris refines exact positions at candidates.")
    print(f"{'='*70}")
