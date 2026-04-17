#!/usr/bin/env python3
"""
HOUSE DATA — Astrological House Meanings & Placement Interpretation
====================================================================

Houses are a mod-12 partition of the ecliptic relative to the observer.
12 = 2² × 3, which is CRT-native:
  - mod 4 = angular quadrant (ASC/IC/DSC/MC axes)
  - mod 3 = modality (angular/succedent/cadent)
  - mod 2 = hemisphere (above/below horizon)

This module provides:
  1. What each house governs (life domains)
  2. What each planet means in each house
  3. CRT-native house classification
  4. Three-layer integration (visible + shadow + boundary meanings)

No float. All data is structural. Interpretations are grounded in
the residue architecture, not arbitrary keyword assignment.

Author: HackFate Research Laboratory
Date: 2026-03-24
"""

from __future__ import annotations
from typing import Dict, List, Tuple, Optional


# ═══════════════════════════════════════════════════════════════════════════
# CRT STRUCTURE OF HOUSES — 12 = 4 × 3
# ═══════════════════════════════════════════════════════════════════════════

# Quadrant (mod 4): which axis does the house belong to?
# H1-3 = Q1 (ASC, self), H4-6 = Q2 (IC, foundation),
# H7-9 = Q3 (DSC, other), H10-12 = Q4 (MC, public)
QUADRANT = {
    1: 0, 2: 0, 3: 0,      # Q1: Self / Identity / Personal
    4: 1, 5: 1, 6: 1,      # Q2: Foundation / Home / Service
    7: 2, 8: 2, 9: 2,      # Q3: Other / Partnership / Expansion
    10: 3, 11: 3, 12: 3,   # Q4: Public / Community / Transcendence
}

QUADRANT_NAMES = {
    0: 'self',        # ASC axis — personal identity
    1: 'foundation',  # IC axis — roots, security
    2: 'other',       # DSC axis — relationships, projection
    3: 'public',      # MC axis — career, legacy
}

# Modality (mod 3): angular / succedent / cadent
# Angular (0): H1, H4, H7, H10 — initiation, action, power
# Succedent (1): H2, H5, H8, H11 — consolidation, resources
# Cadent (2): H3, H6, H9, H12 — adaptation, processing
MODALITY = {
    1: 0, 4: 0, 7: 0, 10: 0,   # Angular: initiating
    2: 1, 5: 1, 8: 1, 11: 1,   # Succedent: stabilizing
    3: 2, 6: 2, 9: 2, 12: 2,   # Cadent: distributing
}

MODALITY_NAMES = {
    0: 'angular',    # Power positions — direct action
    1: 'succedent',  # Resource positions — consolidation
    2: 'cadent',     # Processing positions — adaptation
}

# Hemisphere (mod 2)
HEMISPHERE = {h: 0 if h <= 6 else 1 for h in range(1, 13)}
HEMISPHERE_NAMES = {0: 'below_horizon', 1: 'above_horizon'}


# ═══════════════════════════════════════════════════════════════════════════
# HOUSE MEANINGS — what each house governs
# ═══════════════════════════════════════════════════════════════════════════

HOUSE_DATA = {
    1: {
        'name': 'House of Self',
        'domain': 'identity, appearance, vitality, first impression',
        'keywords': ['self-image', 'body', 'persona', 'approach to life'],
        'question': 'Who am I? How do I present myself?',
        'axis': 'ASC — the mask and the emergence',
        'shadow_meaning': 'compulsive self-projection, identity rigidity',
        'boundary_meaning': 'identity crisis, forced reinvention',
    },
    2: {
        'name': 'House of Resources',
        'domain': 'money, possessions, values, self-worth',
        'keywords': ['income', 'material security', 'talents', 'what I value'],
        'question': 'What do I have? What do I value?',
        'axis': 'resource axis (2/8)',
        'shadow_meaning': 'hidden attachment to material security',
        'boundary_meaning': 'financial collapse/restructuring, values crisis',
    },
    3: {
        'name': 'House of Communication',
        'domain': 'mind, siblings, local travel, daily environment',
        'keywords': ['speech', 'writing', 'learning', 'neighbors', 'short journeys'],
        'question': 'How do I think and communicate?',
        'axis': 'knowledge axis (3/9)',
        'shadow_meaning': 'obsessive mental patterns, communication blocks',
        'boundary_meaning': 'cognitive breakthrough or breakdown',
    },
    4: {
        'name': 'House of Foundation',
        'domain': 'home, family, roots, inner security, parent',
        'keywords': ['ancestry', 'property', 'emotional base', 'private life'],
        'question': 'Where do I come from? What is my foundation?',
        'axis': 'IC — the root and the hidden ground',
        'shadow_meaning': 'unresolved family patterns, inherited wounds',
        'boundary_meaning': 'uprooting, family structure collapse/rebuild',
    },
    5: {
        'name': 'House of Creation',
        'domain': 'creativity, children, romance, pleasure, self-expression',
        'keywords': ['play', 'art', 'love affairs', 'gambling', 'joy'],
        'question': 'What do I create? What gives me joy?',
        'axis': 'creative axis (5/11)',
        'shadow_meaning': 'creative blocks, compulsive pleasure-seeking',
        'boundary_meaning': 'creative crisis, fertility turning point',
    },
    6: {
        'name': 'House of Service',
        'domain': 'health, work, daily routine, duty, self-improvement',
        'keywords': ['employment', 'habits', 'illness', 'service', 'craft'],
        'question': 'How do I serve? How do I maintain myself?',
        'axis': 'service axis (6/12)',
        'shadow_meaning': 'workaholism, health denial, perfectionism',
        'boundary_meaning': 'health crisis, work restructuring',
    },
    7: {
        'name': 'House of Partnership',
        'domain': 'marriage, business partners, open enemies, contracts',
        'keywords': ['spouse', 'equal relationships', 'projection', 'the other'],
        'question': 'Who do I partner with? What do I project onto others?',
        'axis': 'DSC — the mirror and the other',
        'shadow_meaning': 'codependency, projection, losing self in other',
        'boundary_meaning': 'relationship rupture, forced independence',
    },
    8: {
        'name': 'House of Transformation',
        'domain': 'death, rebirth, shared resources, intimacy, occult',
        'keywords': ['inheritance', 'debt', 'sex', 'power', 'crisis', 'regeneration'],
        'question': 'What must I release? What transforms me?',
        'axis': 'resource axis (2/8) — other people\'s resources',
        'shadow_meaning': 'control issues, fear of vulnerability, obsession',
        'boundary_meaning': 'ego death, total transformation, power shift',
    },
    9: {
        'name': 'House of Expansion',
        'domain': 'philosophy, higher education, travel, law, belief',
        'keywords': ['religion', 'foreign cultures', 'publishing', 'meaning'],
        'question': 'What do I believe? How do I expand my world?',
        'axis': 'knowledge axis (3/9) — higher knowledge',
        'shadow_meaning': 'dogmatism, escapism through ideology',
        'boundary_meaning': 'worldview collapse, paradigm shift',
    },
    10: {
        'name': 'House of Vocation',
        'domain': 'career, public reputation, authority, achievement',
        'keywords': ['status', 'ambition', 'boss', 'legacy', 'contribution'],
        'question': 'What is my calling? How am I seen publicly?',
        'axis': 'MC — the summit and the visible achievement',
        'shadow_meaning': 'obsessive ambition, public persona vs private self',
        'boundary_meaning': 'career collapse/breakthrough, authority crisis',
    },
    11: {
        'name': 'House of Community',
        'domain': 'friends, groups, hopes, social networks, humanitarian',
        'keywords': ['allies', 'aspirations', 'collective', 'innovation'],
        'question': 'What is my tribe? What do I hope for?',
        'axis': 'creative axis (5/11) — collective creation',
        'shadow_meaning': 'social alienation, misplaced idealism',
        'boundary_meaning': 'community exile/transformation, dream death/rebirth',
    },
    12: {
        'name': 'House of Dissolution',
        'domain': 'unconscious, hidden enemies, isolation, spirituality',
        'keywords': ['secrets', 'karma', 'institutions', 'self-undoing', 'dreams'],
        'question': 'What is hidden? What must I surrender?',
        'axis': 'service axis (6/12) — transcendent service',
        'shadow_meaning': 'self-sabotage, addiction, martyrdom',
        'boundary_meaning': 'spiritual breakthrough, dissolution of ego structure',
    },
}


# ═══════════════════════════════════════════════════════════════════════════
# PLANET-IN-HOUSE INTERPRETATIONS
# ═══════════════════════════════════════════════════════════════════════════

# Planet keywords (what each planet BRINGS to a house)
PLANET_ENERGY = {
    'Sun':     {'drive': 'identity',     'gives': 'vitality, purpose, ego focus',
                'shadow': 'ego inflation, need for recognition'},
    'Moon':    {'drive': 'emotional',    'gives': 'nurture, instinct, fluctuation',
                'shadow': 'emotional dependency, moodiness'},
    'Mercury': {'drive': 'mental',       'gives': 'analysis, communication, adaptability',
                'shadow': 'overthinking, nervous energy, duplicity'},
    'Venus':   {'drive': 'relational',   'gives': 'beauty, harmony, attraction, values',
                'shadow': 'vanity, avoidance of conflict, indulgence'},
    'Mars':    {'drive': 'action',       'gives': 'energy, courage, assertion, competition',
                'shadow': 'aggression, impatience, conflict-seeking'},
    'Jupiter': {'drive': 'expansion',    'gives': 'growth, optimism, wisdom, opportunity',
                'shadow': 'excess, overconfidence, dogmatism'},
    'Saturn':  {'drive': 'structure',    'gives': 'discipline, mastery, responsibility, limits',
                'shadow': 'rigidity, fear, depression, delay'},
    'Uranus':  {'drive': 'disruption',   'gives': 'innovation, freedom, sudden change',
                'shadow': 'instability, rebellion for its own sake'},
    'Neptune': {'drive': 'dissolution',  'gives': 'imagination, spirituality, compassion',
                'shadow': 'confusion, escapism, deception'},
    'Pluto':   {'drive': 'transformation', 'gives': 'power, depth, regeneration, control',
                'shadow': 'obsession, manipulation, destruction'},
}

# Planet + House → specific interpretation
# Format: (visible_meaning, shadow_meaning, boundary_meaning)
PLACEMENT = {}

def _build_placement(planet: str, house: int) -> Dict:
    """Generate placement interpretation from planet energy + house domain."""
    pe = PLANET_ENERGY[planet]
    hd = HOUSE_DATA[house]
    quad = QUADRANT_NAMES[QUADRANT[house]]
    mod = MODALITY_NAMES[MODALITY[house]]

    return {
        'planet': planet,
        'house': house,
        'house_name': hd['name'],
        'domain': hd['domain'],
        # What this placement does
        'visible': f"{pe['drive']} energy directed toward {hd['domain']}. "
                   f"{pe['gives']} expressed through the {hd['name'].lower()}.",
        # Shadow layer
        'shadow': f"Shadow pattern: {pe['shadow']} combined with {hd['shadow_meaning']}. "
                  f"Lane-11 activation here means hidden {pe['drive']} dynamics "
                  f"in the {quad} quadrant.",
        # Boundary layer
        'boundary': f"Boundary event: {hd['boundary_meaning']}. "
                    f"Lane-13 activation here marks a threshold in "
                    f"{hd['domain']} — the {pe['drive']} function "
                    f"crosses its completeness edge.",
        # Structural classification
        'quadrant': quad,
        'modality': mod,
        'hemisphere': HEMISPHERE_NAMES[HEMISPHERE[house]],
        # CRT decomposition
        'house_mod_4': house % 4 if house % 4 != 0 else 4,
        'house_mod_3': house % 3 if house % 3 != 0 else 3,
    }

# Build all 120 placements (10 planets × 12 houses)
for _p in PLANET_ENERGY:
    for _h in range(1, 13):
        PLACEMENT[(_p, _h)] = _build_placement(_p, _h)


# ═══════════════════════════════════════════════════════════════════════════
# CARRIER ROLE HOUSE INTEGRATION
# ═══════════════════════════════════════════════════════════════════════════

CARRIER_HOUSE_NOTES = {
    'exact-lock': (
        "As the exact-lock carrier (S_R {5,7}), this planet's house placement "
        "shows WHERE visible harmonic patterns crystallize in the life. "
        "The house domain is where exact-lane closures manifest as "
        "tangible, recognizable experiences."
    ),
    'boundary-bridge': (
        "As the boundary bridge (S_R {5} + boundary {13}), this planet's "
        "house placement shows WHERE threshold events occur. The house "
        "domain is the arena of regime shifts and structural transitions."
    ),
    'shadow-carrier': (
        "As the shadow carrier (S_R {11}), this planet's house placement "
        "shows WHERE the hidden architecture operates. The house domain "
        "is where invisible patterns surface, where the 'missing channel' "
        "becomes tangible. Saturn's house is the shadow's primary address."
    ),
    'secondary-lock (provisional)': (
        "As the secondary lock carrier, this planet's house placement "
        "provides a supporting harmonic anchor in its house domain."
    ),
}


# ═══════════════════════════════════════════════════════════════════════════
# READING GENERATOR — produces placement interpretations
# ═══════════════════════════════════════════════════════════════════════════

def placement_reading(planet: str, house: int,
                      shadow_active: bool = False,
                      boundary_active: bool = False,
                      carrier_label: Optional[str] = None,
                      is_retrograde: bool = False) -> Dict:
    """Generate a complete three-layer reading for one placement.

    Args:
        planet: Planet name
        house: House number (1-12)
        shadow_active: Is lane-11 active for this planet?
        boundary_active: Is lane-13 active for this planet?
        carrier_label: DPM-PRIME carrier role (if any)
        is_retrograde: Is the planet retrograde?
    """
    key = (planet, house)
    if key not in PLACEMENT:
        return {'error': f'Unknown placement: {planet} in H{house}'}

    p = PLACEMENT[key]
    reading = {
        'summary': p['visible'],
        'domain': p['domain'],
        'quadrant': p['quadrant'],
        'modality': p['modality'],
    }

    # Layer B: Shadow
    if shadow_active:
        reading['shadow'] = p['shadow']
    else:
        reading['shadow'] = None

    # Layer C: Boundary
    if boundary_active:
        reading['boundary'] = p['boundary']
    else:
        reading['boundary'] = None

    # Carrier role
    if carrier_label and carrier_label in CARRIER_HOUSE_NOTES:
        reading['carrier'] = CARRIER_HOUSE_NOTES[carrier_label]
    else:
        reading['carrier'] = None

    # Retrograde
    if is_retrograde:
        reading['retrograde'] = (
            f"{planet} retrograde in {p['house_name']}: the {PLANET_ENERGY[planet]['drive']} "
            f"function turns inward in this domain. Internal processing "
            f"replaces external expression. Delays and revision in "
            f"{p['domain']}."
        )
    else:
        reading['retrograde'] = None

    return reading


def full_chart_placements(planet_data: Dict[str, Tuple],
                          house_fn,
                          shadow_planets: set = None,
                          boundary_planets: set = None,
                          carrier_map: Dict[str, str] = None,
                          retro_planets: set = None) -> List[Dict]:
    """Generate placement readings for all planets in a chart.

    Args:
        planet_data: {name: (lon_arcsec, vel_arcsec, house_arcsec)}
        house_fn: function(lon_arcsec) -> house_number
        shadow_planets: set of planet names with lane-11 activation
        boundary_planets: set of planet names with lane-13 activation
        carrier_map: {planet_name: carrier_label}
        retro_planets: set of retrograde planet names
    """
    if shadow_planets is None: shadow_planets = set()
    if boundary_planets is None: boundary_planets = set()
    if carrier_map is None: carrier_map = {}
    if retro_planets is None: retro_planets = set()

    readings = []
    for name, (lon, vel, house_as) in planet_data.items():
        h = house_fn(lon)
        r = placement_reading(
            planet=name,
            house=h,
            shadow_active=name in shadow_planets,
            boundary_active=name in boundary_planets,
            carrier_label=carrier_map.get(name),
            is_retrograde=name in retro_planets,
        )
        r['planet'] = name
        r['house'] = h
        r['longitude'] = lon
        readings.append(r)

    return readings


# ═══════════════════════════════════════════════════════════════════════════
# TRANSIT-IN-HOUSE — what a transit means for a specific house
# ═══════════════════════════════════════════════════════════════════════════

def transit_house_meaning(transit_planet: str, natal_house: int,
                          aspect_name: Optional[str] = None,
                          shadow_active: bool = False,
                          boundary_active: bool = False) -> Dict:
    """What does a transit planet passing through a natal house mean?"""
    pe = PLANET_ENERGY.get(transit_planet, {})
    hd = HOUSE_DATA.get(natal_house, {})

    if not pe or not hd:
        return {}

    reading = {
        'transit': transit_planet,
        'house': natal_house,
        'meaning': (
            f"Transit {transit_planet} activates your {hd['name']}. "
            f"{pe['gives']} enters the domain of {hd['domain']}."
        ),
    }

    if aspect_name:
        reading['aspect_context'] = (
            f"The {aspect_name} aspect channels this activation as "
            f"{'harmonious flow' if aspect_name in ('trine', 'sextile', 'conjunction') else 'structural tension'}."
        )

    if shadow_active:
        reading['shadow'] = (
            f"Shadow transit: {transit_planet}'s {pe['drive']} energy "
            f"operates BELOW visibility in your {hd['name'].lower()}. "
            f"{hd['shadow_meaning']}."
        )

    if boundary_active:
        reading['boundary'] = (
            f"Boundary transit: {hd['boundary_meaning']}. "
            f"{transit_planet} pushes {hd['domain']} past its "
            f"structural limit."
        )

    return reading


# ═══════════════════════════════════════════════════════════════════════════
# RENDER
# ═══════════════════════════════════════════════════════════════════════════

def render_placements(readings: List[Dict]) -> str:
    """Render placement readings as text."""
    lines = []
    lines.append("─" * 70)
    lines.append("  PLANETARY PLACEMENTS — Three-Layer Interpretation")
    lines.append("─" * 70)

    for r in sorted(readings, key=lambda x: x.get('house', 0)):
        name = r.get('planet', '?')
        house = r.get('house', 0)
        hdata = HOUSE_DATA.get(house, {})
        quad = r.get('quadrant', '')
        mod = r.get('modality', '')

        lines.append(f"\n  {name} in House {house} ({hdata.get('name', '')})")
        lines.append(f"  Quadrant: {quad} | Modality: {mod}")
        lines.append(f"  ► {r.get('summary', '')}")

        if r.get('shadow'):
            lines.append(f"  ◐ SHADOW: {r['shadow']}")
        if r.get('boundary'):
            lines.append(f"  ◈ BOUNDARY: {r['boundary']}")
        if r.get('carrier'):
            lines.append(f"  ★ CARRIER: {r['carrier']}")
        if r.get('retrograde'):
            lines.append(f"  ℞ RETROGRADE: {r['retrograde']}")

    return "\n".join(lines)


# ═══════════════════════════════════════════════════════════════════════════
# DEMO
# ═══════════════════════════════════════════════════════════════════════════

if __name__ == '__main__':
    from astrology_engine import HouseTopology

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
    HOUSES = HouseTopology(
        [23819, 160400, 256749, 337956, 421275, 524762,
         671819, 808400, 904749, 985956, 1069275, 1172762],
        asc=23819, mc=985956)

    # Shadow and boundary planets from v3.1 natal analysis
    shadow_planets = {'Neptune', 'Mercury', 'Jupiter', 'Moon', 'Mars'}
    boundary_planets = {'Saturn', 'Moon'}
    carrier_map = {
        'Venus': 'exact-lock',
        'Mars': 'boundary-bridge',
        'Saturn': 'shadow-carrier',
        'Jupiter': 'secondary-lock (provisional)',
    }
    retro_planets = set()  # none at birth

    readings = full_chart_placements(
        NATAL, HOUSES.find,
        shadow_planets, boundary_planets,
        carrier_map, retro_planets)

    print("=" * 70)
    print("  HOUSE PLACEMENTS — Anthony Diaz")
    print("  Oct 21, 1980 | 5:31 PM EDT | Fort Bragg, NC")
    print("=" * 70)
    print(render_placements(readings))
    print("=" * 70)
