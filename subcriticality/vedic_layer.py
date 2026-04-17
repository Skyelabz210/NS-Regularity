#!/usr/bin/env python3
"""
VEDIC LAYER — Sidereal Positions, Nakshatras, Dasha Timing
=============================================================

Lanes are PARALLEL, not merged.
Tropical lane and sidereal lane produce INDEPENDENT outputs.
Aspects/returns/houses computed PER LANE separately.

Sidereal = Tropical - Ayanamsa (integer arcseconds)

Nakshatras: 27 lunar mansions at 13°20' = 48,000 arcsec each.
  27 = 3³ — fits cleanly into CRT framework.
  Coprime to {2, 5, 7, 11, 13}.

Vimsottari Dasha: 120-year planetary period cycle.
  120 = 2³ × 3 × 5 — CRT-native.
  Based on Moon's nakshatra at birth.
  Planetary rulers in fixed sequence:
    Ketu-7, Venus-20, Sun-6, Moon-10, Mars-7,
    Rahu-18, Jupiter-16, Saturn-19, Mercury-17 = 120 years

True Shadow Inversion: S(t+T) = -S(t)
  Now testable: compare natal diff residues to transit diff residues.
  If transit_diff ≡ p - natal_diff (mod p) for 3+ lanes → shadow inversion.

No float in computation. Ayanamsa is integer arcseconds.

Author: HackFate Research Laboratory
Date: 2026-03-25
"""

from __future__ import annotations
from typing import Dict, List, Tuple, Optional

from astrology_engine import (
    PlanetManifold, CodexManifold, PairRelation, HouseTopology,
    ARCSEC_PER_DEG, FULL_CIRCLE, HALF_CIRCLE, ARCSEC_PER_SIGN,
    SAFE_BASIS, FULL_BASIS, SIGN_NAMES,
)


# ═══════════════════════════════════════════════════════════════════════════
# AYANAMSA — precession offset (integer arcseconds)
# ═══════════════════════════════════════════════════════════════════════════

# Lahiri ayanamsa at J2000.0 = 23.8338° = 85802 arcsec
# Annual precession rate = 50.29 arcsec/year
# Both stored as integers in milliarcseconds for exact computation.

LAHIRI_J2000_MAS = 85801800    # 23.8338° × 3,600,000 mas/deg
PRECESSION_RATE_MAS = 50290    # 50.29 arcsec/year × 1000 mas/arcsec
J2000_MAYA_DAY = 1867262       # J2000.0 as Maya day offset


def ayanamsa_arcsec(maya_day: int) -> int:
    """Compute Lahiri ayanamsa in arcseconds for a given Maya day offset.

    Uses integer arithmetic throughout. The milliarcsecond intermediate
    prevents accumulated rounding error over centuries.
    """
    # Days from J2000
    days_from_j2000 = maya_day - J2000_MAYA_DAY
    # Years from J2000 (Julian year = 365.25 days, but use integer: ×4 trick)
    # years × 4 = days_from_j2000 × 4 / 1461
    years_x4 = days_from_j2000 * 4
    # Ayanamsa in milliarcseconds
    ayanamsa_mas = LAHIRI_J2000_MAS + (PRECESSION_RATE_MAS * years_x4) // 1461
    # Convert to arcseconds (integer division)
    return ayanamsa_mas // 1000


def sidereal_longitude(tropical_lon: int, maya_day: int) -> int:
    """Convert tropical longitude to sidereal (Lahiri).
    Both in arcseconds. Pure integer subtraction.
    """
    ayanamsa = ayanamsa_arcsec(maya_day)
    return (tropical_lon - ayanamsa) % FULL_CIRCLE


# ═══════════════════════════════════════════════════════════════════════════
# SIDEREAL SIGN NAMES
# ═══════════════════════════════════════════════════════════════════════════

SIDEREAL_SIGNS = [
    'Mesha', 'Vrishabha', 'Mithuna', 'Karka',
    'Simha', 'Kanya', 'Tula', 'Vrischika',
    'Dhanu', 'Makara', 'Kumbha', 'Meena',
]

SIDEREAL_SIGN_WESTERN = [
    'Aries', 'Taurus', 'Gemini', 'Cancer',
    'Leo', 'Virgo', 'Libra', 'Scorpio',
    'Sagittarius', 'Capricorn', 'Aquarius', 'Pisces',
]


# ═══════════════════════════════════════════════════════════════════════════
# NAKSHATRAS — 27 lunar mansions
# 27 = 3³, coprime to {2, 5, 7, 11, 13}
# Each nakshatra = 13°20' = 48,000 arcseconds
# ═══════════════════════════════════════════════════════════════════════════

NAKSHATRA_SPAN = 48000  # 13°20' in arcseconds
assert FULL_CIRCLE == 27 * NAKSHATRA_SPAN  # 1,296,000 = 27 × 48,000 ✓

NAKSHATRA_NAMES = [
    'Ashvini',     'Bharani',    'Krittika',   'Rohini',
    'Mrigashira',  'Ardra',      'Punarvasu',  'Pushya',
    'Ashlesha',    'Magha',      'Purva Phalguni', 'Uttara Phalguni',
    'Hasta',       'Chitra',     'Swati',      'Vishakha',
    'Anuradha',    'Jyeshtha',   'Mula',       'Purva Ashadha',
    'Uttara Ashadha', 'Shravana', 'Dhanishta', 'Shatabhisha',
    'Purva Bhadrapada', 'Uttara Bhadrapada', 'Revati',
]

# Each nakshatra is ruled by a planet (for Dasha calculation)
NAKSHATRA_RULERS = [
    'Ketu', 'Venus', 'Sun', 'Moon', 'Mars', 'Rahu', 'Jupiter', 'Saturn', 'Mercury',  # 1-9
    'Ketu', 'Venus', 'Sun', 'Moon', 'Mars', 'Rahu', 'Jupiter', 'Saturn', 'Mercury',  # 10-18
    'Ketu', 'Venus', 'Sun', 'Moon', 'Mars', 'Rahu', 'Jupiter', 'Saturn', 'Mercury',  # 19-27
]


def nakshatra_index(sidereal_lon: int) -> int:
    """Which nakshatra does this sidereal longitude fall in? (0-26)"""
    return sidereal_lon // NAKSHATRA_SPAN


def nakshatra_name(sidereal_lon: int) -> str:
    return NAKSHATRA_NAMES[nakshatra_index(sidereal_lon)]


def nakshatra_ruler(sidereal_lon: int) -> str:
    return NAKSHATRA_RULERS[nakshatra_index(sidereal_lon)]


def nakshatra_position(sidereal_lon: int) -> Tuple[int, int, int]:
    """Returns (nakshatra_index, pada, position_in_nakshatra_arcsec).
    Each nakshatra has 4 padas of 3°20' = 12,000 arcsec.
    """
    nak = nakshatra_index(sidereal_lon)
    pos_in_nak = sidereal_lon % NAKSHATRA_SPAN
    pada = pos_in_nak // 12000 + 1  # 1-4
    return (nak, pada, pos_in_nak)


# ═══════════════════════════════════════════════════════════════════════════
# VIMSOTTARI DASHA — 120-year planetary period cycle
# 120 = 2³ × 3 × 5 — CRT-native
# ═══════════════════════════════════════════════════════════════════════════

DASHA_SEQUENCE = [
    ('Ketu',    7),
    ('Venus',  20),
    ('Sun',     6),
    ('Moon',   10),
    ('Mars',    7),
    ('Rahu',   18),
    ('Jupiter', 16),
    ('Saturn', 19),
    ('Mercury', 17),
]

DASHA_TOTAL = 120  # years
assert sum(d[1] for d in DASHA_SEQUENCE) == DASHA_TOTAL

# Days per year (for Dasha computation)
DAYS_PER_YEAR = 365


def compute_dasha(moon_sidereal_lon: int, birth_day: int,
                  target_day: int) -> Dict:
    """Compute which Dasha period is active at target_day.

    The Moon's nakshatra at birth determines the starting Dasha lord.
    The balance (how far through the first Dasha) is determined by
    the Moon's position within its nakshatra.

    Returns dict with:
      - mahadasha: current major period planet
      - mahadasha_years: length of this period
      - elapsed_in_mahadasha: years elapsed in current period
      - antardasha: current sub-period planet
      - elapsed_in_antardasha: years elapsed in sub-period
    """
    # Find birth nakshatra ruler (starting Dasha lord)
    nak_idx = nakshatra_index(moon_sidereal_lon)
    ruler = NAKSHATRA_RULERS[nak_idx]

    # Find position in Dasha sequence
    start_idx = next(i for i, (name, _) in enumerate(DASHA_SEQUENCE)
                     if name == ruler)

    # Balance: how much of the first Dasha is remaining at birth
    pos_in_nak = moon_sidereal_lon % NAKSHATRA_SPAN
    fraction_elapsed = pos_in_nak * 1000 // NAKSHATRA_SPAN  # per-mille
    first_dasha_years = DASHA_SEQUENCE[start_idx][1]
    # Remaining years in birth dasha = total × (1 - fraction_elapsed/1000)
    remaining_first = first_dasha_years * (1000 - fraction_elapsed) // 1000

    # Total elapsed days since birth
    elapsed_days = target_day - birth_day
    if elapsed_days < 0:
        return {'error': 'target_day before birth'}

    elapsed_years = elapsed_days // DAYS_PER_YEAR

    # Walk through Dasha sequence
    years_consumed = 0
    dasha_idx = start_idx

    # First (partial) Dasha
    if elapsed_years < remaining_first:
        # Still in birth Dasha
        md_name, md_years = DASHA_SEQUENCE[dasha_idx]
        elapsed_in_md = first_dasha_years - remaining_first + elapsed_years
    else:
        years_consumed = remaining_first
        dasha_idx = (dasha_idx + 1) % 9

        # Walk full Dashas
        while True:
            md_name, md_years = DASHA_SEQUENCE[dasha_idx]
            if years_consumed + md_years > elapsed_years:
                elapsed_in_md = elapsed_years - years_consumed
                break
            years_consumed += md_years
            dasha_idx = (dasha_idx + 1) % 9

        md_name, md_years = DASHA_SEQUENCE[dasha_idx]

    # Antardasha (sub-period) within the Mahadasha
    # Sub-periods follow the same sequence starting from the Mahadasha lord
    ad_start = dasha_idx
    ad_elapsed_years = elapsed_in_md
    ad_consumed = 0

    for i in range(9):
        ad_idx = (ad_start + i) % 9
        ad_name, ad_full_years = DASHA_SEQUENCE[ad_idx]
        # Sub-period length = (md_years × ad_full_years) / 120
        ad_length = md_years * ad_full_years  # in 120ths of years
        ad_length_years = ad_length // DASHA_TOTAL

        if ad_consumed + ad_length_years > ad_elapsed_years:
            return {
                'mahadasha': md_name,
                'mahadasha_years': md_years,
                'elapsed_in_mahadasha': elapsed_in_md,
                'remaining_in_mahadasha': md_years - elapsed_in_md,
                'antardasha': ad_name,
                'age_years': elapsed_years,
            }
        ad_consumed += ad_length_years

    # Fallback (shouldn't reach here)
    return {
        'mahadasha': md_name,
        'mahadasha_years': md_years,
        'elapsed_in_mahadasha': elapsed_in_md,
        'antardasha': DASHA_SEQUENCE[dasha_idx][0],
        'age_years': elapsed_years,
    }


def dasha_timeline(moon_sidereal_lon: int, birth_day: int,
                   years: int = 80) -> List[Dict]:
    """Generate complete Dasha timeline from birth to age `years`."""
    nak_idx = nakshatra_index(moon_sidereal_lon)
    ruler = NAKSHATRA_RULERS[nak_idx]
    start_idx = next(i for i, (n, _) in enumerate(DASHA_SEQUENCE) if n == ruler)

    pos_in_nak = moon_sidereal_lon % NAKSHATRA_SPAN
    frac = pos_in_nak * 1000 // NAKSHATRA_SPAN
    first_years = DASHA_SEQUENCE[start_idx][1]
    remaining = first_years * (1000 - frac) // 1000

    timeline = []
    age = 0
    idx = start_idx

    # First partial
    timeline.append({
        'planet': DASHA_SEQUENCE[idx][0],
        'start_age': 0,
        'end_age': remaining,
        'years': remaining,
    })
    age = remaining
    idx = (idx + 1) % 9

    # Full cycles
    while age < years:
        name, dur = DASHA_SEQUENCE[idx]
        end = min(age + dur, years)
        timeline.append({
            'planet': name,
            'start_age': age,
            'end_age': end,
            'years': dur,
        })
        age += dur
        idx = (idx + 1) % 9

    return timeline


# ═══════════════════════════════════════════════════════════════════════════
# TRUE SHADOW INVERSION — S(t+T) = -S(t)
# Now testable with transit engine
# ═══════════════════════════════════════════════════════════════════════════

def test_shadow_inversion(natal_diff: Tuple[int, ...],
                          transit_diff: Tuple[int, ...]) -> Dict:
    """Test whether a transit-natal relation is the TRUE MODULAR NEGATION
    of the natal relation.

    For each safe-basis lane p:
      natal_diff[i] + transit_diff[i] ≡ 0 (mod p)
      equivalently: transit_diff[i] ≡ p - natal_diff[i] (mod p)
      equivalently: transit_diff[i] ≡ -natal_diff[i] (mod p)

    This is the REAL S(t+T) = -S(t) test. Not a heuristic.
    Requires both natal and transit PairRelation diff tuples.
    """
    negation_lanes = 0
    identity_lanes = 0
    details = []

    for i, p in enumerate(SAFE_BASIS):
        n = natal_diff[i]
        t = transit_diff[i]

        # True negation: n + t ≡ 0 (mod p)
        is_negation = (n + t) % p == 0
        # Identity: n ≡ t (mod p)
        is_identity = n == t

        if is_negation and n != 0:  # exclude zero (both negation and identity)
            negation_lanes += 1
            details.append(f"lane {p}: NEG (n={n}, t={t}, sum={n+t}≡0)")
        elif is_identity:
            identity_lanes += 1
            details.append(f"lane {p}: IDENTITY (n=t={n})")
        else:
            details.append(f"lane {p}: other (n={n}, t={t})")

    return {
        'negation_lanes': negation_lanes,
        'identity_lanes': identity_lanes,
        'is_shadow_inversion': negation_lanes >= 3,
        'is_shadow_return': identity_lanes >= 4,
        'details': details,
    }


def scan_transit_inversions(natal_relations: List[PairRelation],
                            transit_relations: List[PairRelation]
                            ) -> List[Dict]:
    """Scan all transit relations against all natal relations for
    true shadow inversions.

    Returns list of inversions found.
    """
    inversions = []

    for nr in natal_relations:
        for tr in transit_relations:
            # Match by planet pair (transit planet → natal planet)
            natal_a = nr.name_a
            natal_b = nr.name_b
            transit_a = tr.name_a.replace('t_', '').replace('A_', '').replace('B_', '')
            transit_b = tr.name_b

            # Check if this transit relation involves the same planet pair
            if (transit_a == natal_a and transit_b == natal_b) or \
               (transit_a == natal_b and transit_b == natal_a):
                result = test_shadow_inversion(nr.diff_safe, tr.diff_safe)
                if result['is_shadow_inversion']:
                    inversions.append({
                        'natal_pair': f"{natal_a}-{natal_b}",
                        'transit_pair': f"{tr.name_a}-{tr.name_b}",
                        'natal_diff': nr.diff_safe,
                        'transit_diff': tr.diff_safe,
                        **result,
                    })

    return inversions


# ═══════════════════════════════════════════════════════════════════════════
# DUAL-LANE PLANET — tropical + sidereal in parallel
# ═══════════════════════════════════════════════════════════════════════════

def dual_lane_positions(planet_data: Dict[str, Tuple[int, int, int]],
                        maya_day: int) -> Dict[str, Dict]:
    """Compute dual-lane positions for all planets.
    Returns {name: {tropical, sidereal, nakshatra, pada, sign_trop, sign_sid}}.
    """
    positions = {}
    ayanamsa = ayanamsa_arcsec(maya_day)

    for name, (lon_trop, vel, house) in planet_data.items():
        lon_sid = (lon_trop - ayanamsa) % FULL_CIRCLE
        nak_idx, pada, pos_in_nak = nakshatra_position(lon_sid)

        trop_sign = (lon_trop // ARCSEC_PER_SIGN) % 12
        sid_sign = (lon_sid // ARCSEC_PER_SIGN) % 12
        trop_deg = (lon_trop % ARCSEC_PER_SIGN) // ARCSEC_PER_DEG
        sid_deg = (lon_sid % ARCSEC_PER_SIGN) // ARCSEC_PER_DEG

        positions[name] = {
            'tropical': lon_trop,
            'sidereal': lon_sid,
            'trop_sign': SIGN_NAMES[trop_sign],
            'sid_sign': SIDEREAL_SIGNS[sid_sign],
            'trop_deg': trop_deg,
            'sid_deg': sid_deg,
            'nakshatra': NAKSHATRA_NAMES[nak_idx],
            'nakshatra_idx': nak_idx,
            'pada': pada,
            'nakshatra_ruler': NAKSHATRA_RULERS[nak_idx],
        }

    return positions


# ═══════════════════════════════════════════════════════════════════════════
# RENDER
# ═══════════════════════════════════════════════════════════════════════════

def render_vedic(planet_data: Dict[str, Tuple[int, int, int]],
                 maya_day: int, birth_day: int) -> str:
    L = []
    L.append("=" * 70)
    L.append("  VEDIC LAYER — Sidereal Positions + Nakshatras + Dasha")
    L.append(f"  Ayanamsa (Lahiri): {ayanamsa_arcsec(maya_day) // ARCSEC_PER_DEG}°"
             f"{(ayanamsa_arcsec(maya_day) % ARCSEC_PER_DEG) // 60:02d}'")
    L.append("  Lanes are PARALLEL — tropical and sidereal are independent outputs")
    L.append("=" * 70)

    positions = dual_lane_positions(planet_data, maya_day)

    L.append(f"\n  {'Planet':10s}  {'Tropical':14s}  {'Sidereal':14s}  {'Nakshatra':18s}  {'Pada':>4s}")
    L.append(f"  {'─'*10}  {'─'*14}  {'─'*14}  {'─'*18}  {'─'*4}")

    for name in ('Sun', 'Moon', 'Mercury', 'Venus', 'Mars',
                 'Jupiter', 'Saturn', 'Uranus', 'Neptune', 'Pluto'):
        if name not in positions:
            continue
        p = positions[name]
        trop = f"{p['trop_deg']:2d}° {p['trop_sign']:13s}"
        sid = f"{p['sid_deg']:2d}° {p['sid_sign']:13s}"
        nak = f"{p['nakshatra']:18s}"
        L.append(f"  {name:10s}  {trop}  {sid}  {nak}  {p['pada']:4d}")

    # Moon's nakshatra determines Dasha
    if 'Moon' in positions:
        moon_sid = positions['Moon']['sidereal']
        L.append(f"\n  Moon Nakshatra: {nakshatra_name(moon_sid)} "
                 f"(ruler: {nakshatra_ruler(moon_sid)})")

        # Dasha timeline
        L.append(f"\n{'─'*70}")
        L.append(f"  VIMSOTTARI DASHA TIMELINE (120-year cycle)")
        L.append(f"{'─'*70}")

        timeline = dasha_timeline(moon_sid, birth_day)
        for entry in timeline:
            active = ""
            if entry['start_age'] <= (maya_day - birth_day) // DAYS_PER_YEAR < entry['end_age']:
                active = " ◄ CURRENT"
            L.append(f"  age {entry['start_age']:3d}–{entry['end_age']:3d}  "
                     f"{entry['planet']:10s}  ({entry['years']} years){active}")

        # Current Dasha detail
        current = compute_dasha(moon_sid, birth_day, maya_day)
        L.append(f"\n  CURRENT: {current['mahadasha']} Mahadasha"
                 f" (year {current['elapsed_in_mahadasha']}/{current['mahadasha_years']})")
        L.append(f"  Sub-period: {current['antardasha']} Antardasha")

    L.append("\n" + "=" * 70)
    return "\n".join(L)


# ═══════════════════════════════════════════════════════════════════════════
# DEMO
# ═══════════════════════════════════════════════════════════════════════════

if __name__ == '__main__':
    # ── Anthony Diaz ──
    print("\n" + "=" * 70)
    print("  CHART 1: Anthony Diaz — Oct 21, 1980")
    print("=" * 70)
    NATAL_DAY_1 = 1860251
    NATAL_1 = {
        'Sun': (751107, 3581, 727288), 'Moon': (1810, 54173, 1273991),
        'Mercury': (827399, 540, 803580), 'Venus': (611243, 4272, 587424),
        'Mars': (888870, 2604, 865051), 'Jupiter': (644043, 724, 620224),
        'Saturn': (661345, 419, 637526), 'Uranus': (843177, 207, 819358),
        'Neptune': (938127, 93, 914308), 'Pluto': (727106, 143, 703287),
    }
    print(render_vedic(NATAL_1, NATAL_DAY_1, NATAL_DAY_1))

    # Current transits
    TODAY = 1876816
    print(f"\n  Dasha at age 45:")
    moon_sid = sidereal_longitude(NATAL_1['Moon'][0], NATAL_DAY_1)
    current = compute_dasha(moon_sid, NATAL_DAY_1, TODAY)
    print(f"  {current}")

    # ── Test Chart — Sep 18, 1981 ──
    print("\n" + "=" * 70)
    print("  CHART 2: Test — Sep 18, 1981")
    print("=" * 70)
    NATAL_DAY_2 = 1860583
    NATAL_2 = {
        'Sun': (632958, 3515, 912528), 'Moon': (215033, 51939, 494603),
        'Mercury': (725604, 4136, 1005174), 'Venus': (780777, 4184, 1060347),
        'Mars': (469962, 2235, 749532), 'Jupiter': (703120, 756, 982690),
        'Saturn': (686594, 432, 966164), 'Uranus': (852920, 134, 1132490),
        'Neptune': (943745, 30, 1223315), 'Pluto': (731333, 131, 1010903),
    }
    print(render_vedic(NATAL_2, NATAL_DAY_2, NATAL_DAY_2))

    # ── True shadow inversion test ──
    print("\n" + "=" * 70)
    print("  TRUE SHADOW INVERSION TEST — S(t+T) = -S(t)")
    print("=" * 70)

    # Example: test natal Neptune-Sun diff against itself (identity check)
    natal_diff = (0, 0, 0, 4, 0)  # Neptune-Sun from Anthony's chart
    transit_diff_identity = (0, 0, 0, 4, 0)  # same = identity
    transit_diff_negation = (0, 0, 0, 3, 0)  # 4+3=7≡0(mod7), but others need checking

    result_id = test_shadow_inversion(natal_diff, transit_diff_identity)
    print(f"\n  Identity test: {result_id['identity_lanes']} identity lanes, "
          f"{result_id['negation_lanes']} negation lanes")

    # Construct a true negation: for each lane, t = p - n
    true_neg = tuple((p - natal_diff[i]) % p for i, p in enumerate(SAFE_BASIS))
    print(f"\n  Natal diff:  {natal_diff}")
    print(f"  True -S(t):  {true_neg}")
    result_neg = test_shadow_inversion(natal_diff, true_neg)
    print(f"  Negation test: {result_neg['negation_lanes']} negation lanes")
    print(f"  Is shadow inversion: {result_neg['is_shadow_inversion']}")
    for d in result_neg['details']:
        print(f"    {d}")
