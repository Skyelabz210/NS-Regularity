#!/usr/bin/env python3
"""
astrology_engine.py — CRAM Astrology Engine v4.0
==================================================

Three-layer astrological computation on exact-integer CRT substrate.

Layer A: VISIBLE  — angular geometry, signs, houses, aspects (classical)
Layer B: SHADOW   — lane 11, recursive inversion, non-aspect bonds
Layer C: BOUNDARY — lane 13, incompleteness, threshold events

Architecture (app-ready):
  1. DATA MODELS   — immutable dataclasses, serialization-ready
  2. COMPUTATION   — pure functions, no side effects, testable
  3. RENDERING     — separate from computation, swappable

Depends on: gear_manifold_v3.py (CRTManifold, SafeAnchor)
No float in computation. All positions in integer arcseconds.

Wire-in: CRTManifold replaces GearManifold v2.
  PlanetState.crt → true CRT residues (not cascade digits)
  PairAnalysis.diff → true CRT diff (verified: 6 shadow, 4 boundary)

Author: HackFate Research Laboratory
Date: 2026-03-26
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Dict, List, Tuple, Optional, FrozenSet
from enum import Enum, auto
from functools import reduce
from math import gcd

from gear_manifold_v3 import (
    CRTManifold, MagnitudeLane, SafeAnchor, mod_inverse,
)


# ═══════════════════════════════════════════════════════════════════════════
# CONSTANTS — all exact integer
# ═══════════════════════════════════════════════════════════════════════════

ARCSEC_PER_DEG  = 3600
FULL_CIRCLE     = 360 * ARCSEC_PER_DEG      # 1,296,000
HALF_CIRCLE     = FULL_CIRCLE // 2           # 648,000
ARCSEC_PER_SIGN = 30 * ARCSEC_PER_DEG       # 108,000

SAFE_BASIS      = (2, 3, 5, 7, 11)
FULL_BASIS      = (2, 3, 5, 7, 11, 13)
SHADOW_PRIME    = 11
BOUNDARY_PRIME  = 13
SAFE_PRODUCT    = reduce(lambda a, b: a * b, SAFE_BASIS)
FULL_PRODUCT    = reduce(lambda a, b: a * b, FULL_BASIS)

# S_R = Ramanujan partition primes
S_R_PRIMES      = frozenset({5, 7, 11})
S_R_INDICES     = tuple(i for i, p in enumerate(SAFE_BASIS) if p in S_R_PRIMES)
NON_SR_INDICES  = tuple(i for i, p in enumerate(SAFE_BASIS) if p not in S_R_PRIMES)

SIGN_NAMES = (
    'Aries', 'Taurus', 'Gemini', 'Cancer', 'Leo', 'Virgo',
    'Libra', 'Scorpio', 'Sagittarius', 'Capricorn', 'Aquarius', 'Pisces',
)

TZOLKIN_GLYPHS = (
    'Imix', 'Ik', 'Akbal', 'Kan', 'Chicchan', 'Cimi', 'Manik', 'Lamat',
    'Muluc', 'Oc', 'Chuen', 'Eb', 'Ben', 'Ix', 'Men', 'Cib',
    'Caban', 'Etznab', 'Cauac', 'Ahau',
)

CODEX_CYCLES = {
    'tzolkin': 260, 'haab': 365, 'venus': 584, 'mars': 780,
    'jupiter': 399, 'saturn': 378, 'mercury': 116, '819': 819,
    'eclipse': 11960, 'double_cr': 37960,
    'xultun': 1195740,
}


# ═══════════════════════════════════════════════════════════════════════════
# ASPECT TABLE — all in arcseconds, precomputed
# ═══════════════════════════════════════════════════════════════════════════

@dataclass(frozen=True)
class AspectDef:
    harmonic: int
    exact_angle: int      # arcseconds
    name: str
    max_orb: int          # arcseconds
    tone: int             # Tzolk'in tone

def _build_aspects() -> Dict[int, AspectDef]:
    raw = {
        1:  (0,                    'conjunction', 10 * ARCSEC_PER_DEG),
        2:  (HALF_CIRCLE,          'opposition',  10 * ARCSEC_PER_DEG),
        3:  (120 * ARCSEC_PER_DEG, 'trine',        8 * ARCSEC_PER_DEG),
        4:  (90  * ARCSEC_PER_DEG, 'square',        8 * ARCSEC_PER_DEG),
        6:  (60  * ARCSEC_PER_DEG, 'sextile',       6 * ARCSEC_PER_DEG),
        12: (30  * ARCSEC_PER_DEG, 'semisextile',   3 * ARCSEC_PER_DEG),
        8:  (45  * ARCSEC_PER_DEG, 'semisquare',    3 * ARCSEC_PER_DEG),
        5:  (72  * ARCSEC_PER_DEG, 'quintile',      2 * ARCSEC_PER_DEG),
        10: (144 * ARCSEC_PER_DEG, 'biquintile',    2 * ARCSEC_PER_DEG),
        9:  (150 * ARCSEC_PER_DEG, 'quincunx',      3 * ARCSEC_PER_DEG),
    }
    return {
        h: AspectDef(h, exact, name, orb, (exact % 260) % 13 + 1)
        for h, (exact, name, orb) in raw.items()
    }

ASPECTS = _build_aspects()


# ═══════════════════════════════════════════════════════════════════════════
# CARRIER ROLES — from DPM-PRIME T10
# ═══════════════════════════════════════════════════════════════════════════

@dataclass(frozen=True)
class CarrierRole:
    sr_primes: FrozenSet[int]
    weight: int
    label: str

CARRIER_ROLES: Dict[str, CarrierRole] = {
    'Venus':   CarrierRole(frozenset({5, 7}),  120, 'exact-lock'),
    'Mars':    CarrierRole(frozenset({5, 13}), 100, 'boundary-bridge'),
    'Saturn':  CarrierRole(frozenset({11}),    150, 'shadow-carrier'),
    'Jupiter': CarrierRole(frozenset({5, 7}),  110, 'secondary-lock'),
}


# ═══════════════════════════════════════════════════════════════════════════
# DATA MODELS — immutable, serialization-ready
# ═══════════════════════════════════════════════════════════════════════════

@dataclass(frozen=True)
class BirthData:
    """Input: birth parameters. Immutable."""
    name: str
    maya_day: int
    planets: Dict[str, Tuple[int, int, int]]  # {name: (lon_as, vel_as, house_as)}
    cusps: Optional[Tuple[int, ...]] = None
    asc: int = 0
    mc: int = 0


@dataclass(frozen=True)
class PlanetState:
    """Computed state of one planet. Immutable after creation."""
    name: str
    longitude: int          # arcseconds, [0, FULL_CIRCLE)
    velocity: int           # arcseconds/day (signed)
    house_longitude: int    # arcseconds from ASC
    retrograde: bool
    sign_index: int
    sign_name: str
    degree: int
    minute: int
    second: int
    crt: Tuple[int, ...]    # TRUE CRT residues mod FULL_BASIS
    k_vort: int             # winding number via SafeAnchor

    @staticmethod
    def from_data(name: str, lon: int, vel: int, house: int) -> 'PlanetState':
        lw = lon % FULL_CIRCLE
        si = lw // ARCSEC_PER_SIGN
        pos = lw % ARCSEC_PER_SIGN
        # TRUE CRT — direct mod, no cascade
        crt = tuple(lw % p for p in FULL_BASIS)
        # SafeAnchor for K-Elimination
        anchor = SafeAnchor(list(SAFE_BASIS), BOUNDARY_PRIME)
        anchor.add(lw)
        return PlanetState(
            name=name, longitude=lw, velocity=vel,
            house_longitude=house % FULL_CIRCLE,
            retrograde=vel < 0,
            sign_index=si % 12, sign_name=SIGN_NAMES[si % 12],
            degree=pos // ARCSEC_PER_DEG,
            minute=(pos % ARCSEC_PER_DEG) // 60,
            second=pos % 60,
            crt=crt, k_vort=anchor.k_vort_fast(),
        )

    def fmt_position(self) -> str:
        rx = " \u211e" if self.retrograde else ""
        return f"{self.degree}\u00b0{self.minute:02d}'{self.second:02d}\" {self.sign_name}{rx}"

    def fmt_speed(self) -> str:
        mag = abs(self.velocity)
        sign = '+' if not self.retrograde else '-'
        return f"{sign}{mag // ARCSEC_PER_DEG}\u00b0{(mag % ARCSEC_PER_DEG) // 60:02d}'/d"


@dataclass(frozen=True)
class AspectResult:
    """Detected aspect between two planets."""
    harmonic: int
    name: str
    orb: int              # arcseconds
    applying: bool
    lane_score: int       # S_R-weighted harmonic match
    tone: int


@dataclass(frozen=True)
class ShadowAnalysis:
    """Layer B: shadow (lane 11) analysis for one pair."""
    active: bool                    # Δ ≡ 0 (mod 11)
    shadow_residue: int
    far_side_count: int             # residues in upper half of mod-p circle
    half_turn_count: int            # residues at geometric midpoint
    is_echo: bool                   # far_side ≥ 3 or half_turn ≥ 3
    tension: int                    # high lock + no classical aspect
    strength: int                   # composite metric (decomposed)
    active_clocks: Tuple[str, ...]  # which codex clocks are at shadow return


@dataclass(frozen=True)
class BoundaryAnalysis:
    """Layer C: boundary (lane 13) analysis for one pair."""
    active: bool                    # Δ ≡ 0 (mod 13)
    boundary_residue: int
    threshold_type: str             # stable / threshold_event / regime_shift / structural_fracture
    strength: int
    carrier_bridge: bool            # Mars involved


@dataclass(frozen=True)
class PairAnalysis:
    """Complete analysis of one planet pair. Immutable."""
    planet_a: str
    planet_b: str
    diff_full: Tuple[int, ...]      # TRUE CRT diff — (Δ mod 2, Δ mod 3, ..., Δ mod 13)
    diff_safe: Tuple[int, ...]      # first 5 lanes
    safe_zeros: int                 # count of zero diffs in safe basis
    lock_score: int
    aspect: Optional[AspectResult]
    shadow: ShadowAnalysis
    boundary: BoundaryAnalysis
    carrier_label: Optional[str]
    winding_diff: int


@dataclass(frozen=True)
class CodexState:
    """Maya calendar / codex cycle state. Immutable."""
    delta: int
    tzolkin_tone: int
    tzolkin_glyph: str
    haab_month: int
    haab_day: int
    long_count: Tuple[int, int, int, int, int]
    calendar_round_pos: int
    covering_k: int
    covering_r: int
    tier_819: Tuple[int, int, int]  # (stability, ramanujan, boundary)
    sr_closure: Dict[int, bool]
    shadow_power: int
    day_crt: Tuple[int, ...]


# ═══════════════════════════════════════════════════════════════════════════
# COMPUTATION — pure functions
# ═══════════════════════════════════════════════════════════════════════════

def compute_planet(name: str, data: Tuple[int, int, int]) -> PlanetState:
    """Pure function: birth data → planet state."""
    return PlanetState.from_data(name, *data)


def compute_crt_diff(a: PlanetState, b: PlanetState) -> Tuple[int, ...]:
    """TRUE CRT difference between two planets. Direct mod, no cascade."""
    return tuple((a.crt[i] - b.crt[i]) % FULL_BASIS[i]
                 for i in range(len(FULL_BASIS)))


def classify_aspect(lon_a: int, lon_b: int,
                    vel_a: int, vel_b: int,
                    diff_safe: Tuple[int, ...]) -> Optional[AspectResult]:
    """Detect the closest matching aspect (if any) from angular separation."""
    sep = abs((lon_a - lon_b) % FULL_CIRCLE)
    if sep > HALF_CIRCLE:
        sep = FULL_CIRCLE - sep
    vel_diff = vel_a - vel_b

    best: Optional[AspectResult] = None
    best_orb = FULL_CIRCLE

    for h, asp in ASPECTS.items():
        orb = abs(sep - asp.exact_angle)
        if asp.exact_angle > 0:
            orb = min(orb, abs(sep - (FULL_CIRCLE - asp.exact_angle)))
        if orb <= asp.max_orb and orb < best_orb:
            # S_R-weighted harmonic score
            lane_score = 0
            for i in range(5):
                exp = asp.exact_angle % SAFE_BASIS[i]
                mir = (FULL_CIRCLE - asp.exact_angle) % SAFE_BASIS[i]
                if diff_safe[i] == exp or diff_safe[i] == mir:
                    weight = 3 if i in S_R_INDICES else 1
                    lane_score += weight
            applying = (vel_diff < 0) if sep > asp.exact_angle else (vel_diff > 0)
            best_orb = orb
            best = AspectResult(h, asp.name, orb, applying, lane_score, asp.tone)

    return best


def compute_shadow(diff_full: Tuple[int, ...],
                   codex_delta: int,
                   names: Tuple[str, str],
                   has_aspect: bool,
                   base_lock: int) -> ShadowAnalysis:
    """Compute shadow (lane 11) analysis for one pair."""
    shadow_residue = diff_full[4]  # index 4 = prime 11
    active = shadow_residue == 0

    # Far-side detection
    far_side = sum(1 for i, p in enumerate(SAFE_BASIS)
                   if diff_full[i] > 0 and 2 * diff_full[i] > p)

    # Geometric half-turn
    half_turn = sum(1 for i, p in enumerate(SAFE_BASIS)
                    if diff_full[i] == (p + 1) // 2)

    is_echo = far_side >= 3 or half_turn >= 3
    tension = base_lock if not has_aspect and base_lock >= 200 else 0

    # Shadow strength (decomposed components)
    s_activation = 300 if active else 0
    s_carrier = 200 if 'Saturn' in names else 0
    s_tension = tension // 4
    s_far = far_side * 60
    s_half = half_turn * 40

    # Eclipse shadow power
    eclipse_r = codex_delta % CODEX_CYCLES['eclipse']
    codex_11_power = 0
    for T in (584, 780, 378, 399, 116):
        d = eclipse_r % T
        if d > 0:
            while d % 11 == 0:
                codex_11_power += 1
                d //= 11
    s_codex = codex_11_power * 100

    strength = s_activation + s_codex + s_carrier + s_tension + s_far + s_half

    # Shadow clocks
    active_clocks = []
    for clock_name in ('eclipse', 'saturn', '819', 'double_cr'):
        cycle = CODEX_CYCLES[clock_name]
        r = codex_delta % cycle
        if r > 0 and r % 11 == 0:
            active_clocks.append(clock_name)

    return ShadowAnalysis(
        active=active, shadow_residue=shadow_residue,
        far_side_count=far_side, half_turn_count=half_turn,
        is_echo=is_echo, tension=tension, strength=strength,
        active_clocks=tuple(active_clocks),
    )


def compute_boundary(diff_full: Tuple[int, ...],
                     names: Tuple[str, str]) -> BoundaryAnalysis:
    """Compute boundary (lane 13) analysis for one pair."""
    boundary_residue = diff_full[5]  # index 5 = prime 13
    active = boundary_residue == 0
    safe_zeros = sum(1 for d in diff_full[:5] if d == 0)

    if active and safe_zeros >= 3:
        threshold_type = 'structural_fracture'
    elif active and safe_zeros >= 1:
        threshold_type = 'regime_shift'
    elif active:
        threshold_type = 'threshold_event'
    else:
        threshold_type = 'stable'

    carrier_bridge = 'Mars' in names
    strength = (
        (200 if active else 0)
        + safe_zeros * 30
        + (100 if carrier_bridge and active else 0)
    )
    return BoundaryAnalysis(
        active=active, boundary_residue=boundary_residue,
        threshold_type=threshold_type, strength=strength,
        carrier_bridge=carrier_bridge,
    )


def analyze_pair(a: PlanetState, b: PlanetState,
                 codex_delta: int) -> PairAnalysis:
    """Complete three-layer analysis of one planet pair. Pure function."""
    names = (a.name, b.name)
    diff_full = compute_crt_diff(a, b)
    diff_safe = diff_full[:5]
    safe_zeros = sum(1 for d in diff_safe if d == 0)

    # Harmonic matching (S_R-weighted)
    best_h_score = 0
    for h, asp in ASPECTS.items():
        score = 0
        for i in range(5):
            exp = asp.exact_angle % SAFE_BASIS[i]
            mir = (FULL_CIRCLE - asp.exact_angle) % SAFE_BASIS[i]
            if diff_safe[i] == exp or diff_safe[i] == mir:
                weight = 3 if i in S_R_INDICES else 1
                score += weight
        best_h_score = max(best_h_score, score)

    # S_R displacement activation
    sr_activation = 0
    eclipse_r = codex_delta % CODEX_CYCLES['eclipse']
    syn_map = {'Venus': 584, 'Mars': 780, 'Jupiter': 399, 'Saturn': 378, 'Mercury': 116}
    for n in names:
        if n in syn_map:
            T = syn_map[n]
            disp = eclipse_r % T
            if disp > 0:
                for sp in (5, 7, 11):
                    if disp % sp == 0:
                        sr_activation += 1

    # Carrier bonus
    carrier_bonus = 0
    carrier_label = None
    for n in names:
        if n in CARRIER_ROLES:
            cr = CARRIER_ROLES[n]
            if cr.weight > carrier_bonus:
                carrier_bonus = cr.weight
                carrier_label = cr.label

    # Base lock score
    base_lock = best_h_score * 60 + safe_zeros * 60 + sr_activation * 40 + carrier_bonus

    # Aspect classification
    aspect = classify_aspect(a.longitude, b.longitude,
                             a.velocity, b.velocity, diff_safe)
    has_aspect = aspect is not None

    # Shadow & boundary
    shadow = compute_shadow(diff_full, codex_delta, names, has_aspect, base_lock)
    boundary = compute_boundary(diff_full, names)

    # Final lock score with shadow/boundary contributions
    lock_score = base_lock + shadow.strength // 3 + boundary.strength // 3

    return PairAnalysis(
        planet_a=a.name, planet_b=b.name,
        diff_full=diff_full, diff_safe=diff_safe,
        safe_zeros=safe_zeros, lock_score=lock_score,
        aspect=aspect, shadow=shadow, boundary=boundary,
        carrier_label=carrier_label,
        winding_diff=a.k_vort - b.k_vort,
    )


def compute_codex(delta: int) -> CodexState:
    """Compute Maya codex state. Pure function."""
    # Cycle residues via direct mod (true CRT, single lanes)
    tz_r = delta % 260
    haab_r = delta % 365
    eclipse_r = delta % CODEX_CYCLES['eclipse']

    # Long Count
    r = delta
    b = r // 144000; r %= 144000
    k = r // 7200; r %= 7200
    t = r // 360; r %= 360
    u = r // 20; ki = r % 20

    # 819-day tier
    r819 = delta % 819

    # S_R closure
    sr_closure = {}
    for p in (5, 7, 11):
        sr_closure[p] = any(
            (d := eclipse_r % T) > 0 and d % p == 0
            for T in (584, 780, 378, 399, 116)
        )

    # Shadow power
    shadow_power = 0
    for T in (584, 780, 378, 399, 116):
        d = eclipse_r % T
        if d > 0:
            while d % 11 == 0:
                shadow_power += 1
                d //= 11

    return CodexState(
        delta=delta,
        tzolkin_tone=(tz_r % 13) + 1,
        tzolkin_glyph=TZOLKIN_GLYPHS[tz_r % 20],
        haab_month=haab_r // 20, haab_day=haab_r % 20,
        long_count=(b, k, t, u, ki),
        calendar_round_pos=delta % 18980,
        covering_k=delta // 18980, covering_r=delta % 18980,
        tier_819=(r819 % 9, r819 % 7, r819 % 13),
        sr_closure=sr_closure,
        shadow_power=shadow_power,
        day_crt=tuple(delta % p for p in FULL_BASIS),
    )


def find_house(lon: int, cusps: Tuple[int, ...]) -> int:
    """Find which Placidus house a longitude falls in."""
    for h in range(12):
        s = cusps[h]
        e = cusps[(h + 1) % 12]
        if (e > s and s <= lon < e) or (e <= s and (lon >= s or lon < e)):
            return h + 1
    return 12


# ═══════════════════════════════════════════════════════════════════════════
# CHART — the complete computed chart object
# ═══════════════════════════════════════════════════════════════════════════

class Chart:
    """Complete three-layer chart. Immutable after construction.

    Usage:
        chart = Chart.from_birth(BirthData(...))
        print(chart.render())
        shadows = chart.shadow_network()
        boundaries = chart.boundary_network()
    """

    __slots__ = ('birth', 'planets', 'pairs', 'codex', '_cusps')

    def __init__(self, birth: BirthData):
        self.birth = birth
        self._cusps = birth.cusps

        # Compute planet states
        self.planets: Dict[str, PlanetState] = {
            name: compute_planet(name, data)
            for name, data in birth.planets.items()
        }

        # Compute codex
        self.codex = compute_codex(birth.maya_day)

        # Compute ALL pairwise relations
        names = sorted(self.planets.keys())
        self.pairs: List[PairAnalysis] = [
            analyze_pair(self.planets[names[i]],
                         self.planets[names[j]],
                         birth.maya_day)
            for i in range(len(names))
            for j in range(i + 1, len(names))
        ]

    @classmethod
    def from_birth(cls, birth: BirthData) -> 'Chart':
        return cls(birth)

    # ── Query methods ────────────────────────────────────────────

    def shadow_network(self) -> List[PairAnalysis]:
        """All pairs with active shadow (Δ ≡ 0 mod 11)."""
        return [p for p in self.pairs if p.shadow.active]

    def boundary_network(self) -> List[PairAnalysis]:
        """All pairs with active boundary (Δ ≡ 0 mod 13)."""
        return [p for p in self.pairs if p.boundary.active]

    def tension_network(self) -> List[PairAnalysis]:
        """High lock + no classical aspect."""
        return [p for p in self.pairs if p.shadow.tension > 0]

    def non_aspect_locks(self, threshold: int = 250) -> List[PairAnalysis]:
        """Classically invisible but CRT-locked."""
        return [p for p in self.pairs
                if p.lock_score >= threshold and p.aspect is None]

    def top_locks(self, n: int = 15) -> List[PairAnalysis]:
        return sorted(self.pairs, key=lambda p: p.lock_score, reverse=True)[:n]

    def planet_house(self, name: str) -> int:
        if self._cusps is None:
            return 0
        return find_house(self.planets[name].longitude, self._cusps)

    def fingerprint(self) -> Tuple[int, ...]:
        """CRT fingerprint: concatenated residues of all planets."""
        return tuple(r for name in sorted(self.planets)
                     for r in self.planets[name].crt)

    def as_dict(self) -> Dict:
        """Serialization-ready dict for API responses."""
        return {
            'name': self.birth.name,
            'maya_day': self.birth.maya_day,
            'planets': {
                name: {
                    'position': ps.fmt_position(),
                    'longitude': ps.longitude,
                    'velocity': ps.velocity,
                    'sign': ps.sign_name,
                    'degree': ps.degree,
                    'minute': ps.minute,
                    'retrograde': ps.retrograde,
                    'house': self.planet_house(name),
                    'crt': ps.crt,
                    'k_vort': ps.k_vort,
                }
                for name, ps in self.planets.items()
            },
            'shadow_count': len(self.shadow_network()),
            'boundary_count': len(self.boundary_network()),
            'codex': {
                'tzolkin': f"{self.codex.tzolkin_tone} {self.codex.tzolkin_glyph}",
                'long_count': '.'.join(str(x) for x in self.codex.long_count),
                'calendar_round': self.codex.calendar_round_pos,
            },
        }


# ═══════════════════════════════════════════════════════════════════════════
# RENDERING — separate from computation
# ═══════════════════════════════════════════════════════════════════════════

def _fmt_pos(arcsec: int) -> str:
    si = (arcsec // ARCSEC_PER_SIGN) % 12
    p = arcsec % ARCSEC_PER_SIGN
    return f"{p // ARCSEC_PER_DEG}\u00b0{(p % ARCSEC_PER_DEG) // 60:02d}'{p % 60:02d}\" {SIGN_NAMES[si]}"

def _fmt_orb(arcsec: int) -> str:
    return f"{arcsec // ARCSEC_PER_DEG}\u00b0{(arcsec % ARCSEC_PER_DEG) // 60:02d}'"


def render_chart(chart: Chart) -> str:
    """Render full three-layer chart as text. Separate from computation."""
    L = []
    L.append("=" * 70)
    L.append(f"  {chart.birth.name} \u2014 Three-Layer Chart")
    L.append(f"  Maya Day {chart.birth.maya_day} | {chart.codex.tzolkin_tone} {chart.codex.tzolkin_glyph}")
    L.append(f"  Engine v4.0 | CRT Substrate v3 | True CRT Residues")
    L.append("=" * 70)

    # ── Shadow network ──
    shd = chart.shadow_network()
    L.append(f"\n{'─'*70}")
    L.append(f"  SHADOW NETWORK (Lane 11) \u2014 {len(shd)} active bonds")
    L.append(f"{'─'*70}")

    for r in sorted(shd, key=lambda x: x.shadow.strength, reverse=True):
        pair = f"{r.planet_a}-{r.planet_b}"
        asp = r.aspect.name if r.aspect else "\u2014none\u2014"
        note = ""
        if r.carrier_label and 'shadow' in r.carrier_label:
            note = "Saturn shadow carrier"
        elif not r.aspect:
            note = "classically invisible"
        L.append(f"  {pair:25s}  str={r.shadow.strength:4d}  {asp:16s}  {note}")

    # ── Boundary network ──
    bnd = chart.boundary_network()
    L.append(f"\n{'─'*70}")
    L.append(f"  BOUNDARY NETWORK (Lane 13) \u2014 {len(bnd)} active events")
    L.append(f"{'─'*70}")

    for r in sorted(bnd, key=lambda x: x.boundary.strength, reverse=True):
        pair = f"{r.planet_a}-{r.planet_b}"
        asp = r.aspect.name if r.aspect else "\u2014none\u2014"
        L.append(f"  {pair:25s}  str={r.boundary.strength:4d}  type={r.boundary.threshold_type:20s}  {asp}")

    # ── Planets ──
    L.append(f"\n{'─'*70}")
    L.append(f"  PLANETS")
    L.append(f"{'─'*70}")
    L.append(f"  {'Planet':10s}  {'Position':22s}  {'H':>2s}  {'Speed':>9s}  {'CRT (2,3,5,7,11,13)'}")
    L.append(f"  {'─'*10}  {'─'*22}  {'─'*2}  {'─'*9}  {'─'*22}")

    for name in sorted(chart.planets):
        ps = chart.planets[name]
        h = chart.planet_house(name)
        L.append(f"  {name:10s}  {ps.fmt_position():22s}  {h:2d}  {ps.fmt_speed():>9s}  {ps.crt}")

    if chart._cusps:
        L.append(f"\n  ASC: {_fmt_pos(chart.birth.asc)}  |  MC: {_fmt_pos(chart.birth.mc)}")

    # ── Aspects ──
    L.append(f"\n  {'Pair':25s}  {'Aspect':13s}  {'Orb':>6s}  {'Dir':10s}  {'Lock':>4s}  {'S':>1s}{'B':>1s}")
    L.append(f"  {'─'*25}  {'─'*13}  {'─'*6}  {'─'*10}  {'─'*4}  {'─'*2}")

    for r in sorted(chart.pairs, key=lambda x: x.lock_score, reverse=True):
        if r.aspect is None:
            continue
        pair = f"{r.planet_a}-{r.planet_b}"
        s = "\u25cf" if r.shadow.active else "\u00b7"
        b = "\u25cf" if r.boundary.active else "\u00b7"
        d = "applying" if r.aspect.applying else "separating"
        L.append(f"  {pair:25s}  {r.aspect.name:13s}  {_fmt_orb(r.aspect.orb):>6s}  {d:10s}  {r.lock_score:4d}  {s}{b}")

    # ── Codex ──
    c = chart.codex
    lc = '.'.join(str(x) for x in c.long_count)
    L.append(f"\n{'─'*70}")
    L.append(f"  CODEX")
    L.append(f"{'─'*70}")
    L.append(f"  Tzolk'in: {c.tzolkin_tone} {c.tzolkin_glyph}  |  LC: {lc}  |  CR: {c.calendar_round_pos}/18980")
    s9, s7, s13 = c.tier_819
    L.append(f"  819 Tier: stab={s9} ram={s7} bnd={s13}")
    for p, cl in sorted(c.sr_closure.items()):
        L.append(f"  S_R[{p}]: {'CLOSED' if cl else 'OPEN'}")

    # ── Summary ──
    L.append(f"\n{'─'*70}")
    L.append(f"  SYNTHESIS")
    L.append(f"{'─'*70}")
    L.append(f"  Shadow bonds:     {len(shd)}")
    L.append(f"  Boundary events:  {len(bnd)}")
    L.append(f"  Tension nodes:    {len(chart.tension_network())}")
    L.append(f"  Non-aspect locks: {len(chart.non_aspect_locks())}")

    L.append("\n" + "=" * 70)
    L.append("  True CRT. Zero floats. Shadow is formal, not metaphorical.")
    L.append("=" * 70)
    return "\n".join(L)


# ═══════════════════════════════════════════════════════════════════════════
# VERIFICATION — proves correctness of the v3 wire-in
# ═══════════════════════════════════════════════════════════════════════════

def verify_wire_in() -> int:
    """Verify the engine produces correct CRT residues and networks."""
    failures = 0

    birth = BirthData(
        name="Anthony Diaz",
        maya_day=1860251,
        planets={
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
        },
        cusps=(23819, 160400, 256749, 337956, 421275, 524762,
               671819, 808400, 904749, 985956, 1069275, 1172762),
        asc=23819, mc=985956,
    )

    chart = Chart.from_birth(birth)

    print("=" * 70)
    print("  WIRE-IN VERIFICATION")
    print("=" * 70)

    # 1. CRT residues are TRUE (not cascade)
    print("\n  1. CRT Residues (must match direct mod)")
    for name, data in birth.planets.items():
        lon = data[0] % FULL_CIRCLE
        expected = tuple(lon % p for p in FULL_BASIS)
        actual = chart.planets[name].crt
        ok = actual == expected
        if not ok:
            print(f"    FAIL: {name} crt={actual}, expected={expected}")
            failures += 1
    if failures == 0:
        print(f"    All 10 planets: TRUE CRT \u2713")

    # 2. Shadow network count (must be 6, verified earlier)
    shadows = chart.shadow_network()
    expected_shadow = 6
    if len(shadows) != expected_shadow:
        print(f"\n  2. FAIL: shadow count = {len(shadows)}, expected {expected_shadow}")
        failures += 1
    else:
        print(f"\n  2. Shadow bonds: {len(shadows)} \u2713")
        for s in shadows:
            bnd = "B" if s.boundary.active else "\u00b7"
            print(f"    {s.planet_a:10s}-{s.planet_b:10s}  diff={s.diff_full}  {bnd}")

    # 3. Boundary network count (must be 4)
    boundaries = chart.boundary_network()
    expected_boundary = 4
    if len(boundaries) != expected_boundary:
        print(f"\n  3. FAIL: boundary count = {len(boundaries)}, expected {expected_boundary}")
        failures += 1
    else:
        print(f"\n  3. Boundary events: {len(boundaries)} \u2713")
        for b in boundaries:
            shd = "S" if b.shadow.active else "\u00b7"
            print(f"    {b.planet_a:10s}-{b.planet_b:10s}  diff={b.diff_full}  {shd}")

    # 4. Dual-locked pair (Moon-Pluto: shadow AND boundary)
    dual = [p for p in chart.pairs
            if p.shadow.active and p.boundary.active]
    if len(dual) != 1 or dual[0].planet_a != 'Moon' or dual[0].planet_b != 'Pluto':
        print(f"\n  4. FAIL: dual-locked pair not Moon-Pluto, got {[(d.planet_a, d.planet_b) for d in dual]}")
        failures += 1
    else:
        print(f"\n  4. Dual-locked (shadow+boundary): Moon-Pluto \u2713")

    # 5. Ring homomorphism: diff == (lon_a - lon_b) % p
    print(f"\n  5. Ring homomorphism on diffs")
    diff_ok = True
    for pair in chart.pairs:
        a_lon = chart.planets[pair.planet_a].longitude
        b_lon = chart.planets[pair.planet_b].longitude
        expected_diff = tuple((a_lon - b_lon) % p for p in FULL_BASIS)
        if pair.diff_full != expected_diff:
            print(f"    FAIL: {pair.planet_a}-{pair.planet_b}")
            diff_ok = False
            failures += 1
    if diff_ok:
        print(f"    All 45 pairs: correct \u2713")

    # 6. Codex
    print(f"\n  6. Codex: {chart.codex.tzolkin_tone} {chart.codex.tzolkin_glyph}")
    lc = '.'.join(str(x) for x in chart.codex.long_count)
    print(f"    LC: {lc}")

    # 7. Serialization
    d = chart.as_dict()
    assert 'planets' in d and 'shadow_count' in d and 'codex' in d
    print(f"\n  7. Serialization: as_dict() produces {len(d)} top-level keys \u2713")

    print(f"\n{'='*70}")
    if failures == 0:
        print(f"  ALL CHECKS PASSED. v3 wire-in verified.")
        print(f"  True CRT. 6 shadow. 4 boundary. 1 dual-lock. 45/45 diffs.")
    else:
        print(f"  {failures} FAILURES.")
    print(f"{'='*70}")

    return failures


# ═══════════════════════════════════════════════════════════════════════════
# ENTRY POINT
# ═══════════════════════════════════════════════════════════════════════════

if __name__ == '__main__':
    failures = verify_wire_in()

    if failures == 0:
        # Render the full chart
        birth = BirthData(
            name="Anthony Diaz",
            maya_day=1860251,
            planets={
                'Sun': (751107, 3581, 727288), 'Moon': (1810, 54173, 1273991),
                'Mercury': (827399, 540, 803580), 'Venus': (611243, 4272, 587424),
                'Mars': (888870, 2604, 865051), 'Jupiter': (644043, 724, 620224),
                'Saturn': (661345, 419, 637526), 'Uranus': (843177, 207, 819358),
                'Neptune': (938127, 93, 914308), 'Pluto': (727106, 143, 703287),
            },
            cusps=(23819, 160400, 256749, 337956, 421275, 524762,
                   671819, 808400, 904749, 985956, 1069275, 1172762),
            asc=23819, mc=985956,
        )
        chart = Chart.from_birth(birth)
        print("\n" + render_chart(chart))


# ═══════════════════════════════════════════════════════════════════════════
# BACKWARD COMPATIBILITY ALIASES
# The old v3_1 used different class/function names. Supporting modules
# (transit_engine, event_predictor, house_data, vedic_layer, astrology_system)
# import these names. These aliases let them work without rewriting.
# ═══════════════════════════════════════════════════════════════════════════

class HouseTopology:
    """Backward-compatible wrapper around cusps tuple + find_house()."""
    def __init__(self, cusps, asc, mc):
        self.cusps = tuple(cusps)
        self.asc = asc
        self.mc = mc
    def find(self, lon: int) -> int:
        return find_house(lon, self.cusps)


class PlanetManifold:
    """Backward-compatible wrapper around PlanetState.
    Old code used GearManifold internally. v4 uses direct CRT.
    """
    def __init__(self, name: str, lon: int, vel: int, house: int):
        self._state = PlanetState.from_data(name, lon, vel, house)
        self.name = name
        self._lon = self._state.longitude
        self._vel = vel
        self._house = house % FULL_CIRCLE
        self.vel_neg = vel < 0
    
    def r_full(self) -> Tuple[int, ...]:
        return self._state.crt
    
    def r_safe(self) -> Tuple[int, ...]:
        return self._state.crt[:5]
    
    def r_lane(self, p: int) -> int:
        idx = list(FULL_BASIS).index(p)
        return self._state.crt[idx]
    
    def r_shadow(self) -> int:
        return self.r_lane(SHADOW_PRIME)
    
    def r_boundary(self) -> int:
        return self.r_lane(BOUNDARY_PRIME)
    
    def k_vort(self) -> int:
        return self._state.k_vort
    
    def fmt_pos(self) -> str:
        return self._state.fmt_position()
    
    def fmt_speed(self) -> str:
        return self._state.fmt_speed()


# OmegaState → Chart (the old name)
OmegaState = Chart

# Old-style aspect table format {h: (exact, name, orb)}
ASPECT_TABLE = {
    asp.harmonic: (asp.exact_angle, asp.name, asp.max_orb)
    for asp in ASPECTS.values()
}

# Aspect tones as standalone dict
ASPECT_TONES = {asp.name: asp.tone for asp in ASPECTS.values()}

# Rendering aliases
_fp = _fmt_pos
_fo = _fmt_orb
render = render_chart

# Old class name aliases
ChartFingerprint = type('ChartFingerprint', (), {
    '__init__': lambda self, planets: setattr(self, 'residues',
        tuple(r for n in sorted(planets) for r in planets[n].r_full())),
    'similarity': lambda self, o: sum(1 for a, b in zip(self.residues, o.residues) if a == b),
})

# BoundaryState static method
class BoundaryState:
    @staticmethod
    def boundary_clock(codex_or_delta):
        if isinstance(codex_or_delta, int):
            delta = codex_or_delta
        elif hasattr(codex_or_delta, 'delta'):
            delta = codex_or_delta.delta
        else:
            delta = 0
        clocks = {}
        r819 = delta % 819
        r819_13 = r819 % 13
        clocks['819'] = {
            'at_boundary': r819_13 == 0 and r819 > 0,
            'residue': r819_13,
            'to_next': 0 if (r819_13 == 0 and r819 > 0) else (13 - r819_13),
        }
        recl = (delta % 11960) % 13
        clocks['eclipse'] = {
            'at_boundary': recl == 0 and (delta % 11960) > 0,
            'residue': recl,
            'to_next': 0 if (recl == 0 and (delta % 11960) > 0) else (13 - recl),
        }
        return clocks


# CodexManifold backward-compat wrapper
class CodexManifold:
    """Backward-compatible wrapper. Old code used MagnitudeLane per cycle."""
    def __init__(self, delta: int):
        self.delta = delta
        self._state = compute_codex(delta)
        self._cycle_residues = {name: delta % period for name, period in CODEX_CYCLES.items()}
    
    def r(self, cycle: str) -> int:
        return self._cycle_residues.get(cycle, 0)
    
    def w(self, cycle: str) -> int:
        period = CODEX_CYCLES.get(cycle, 1)
        return self.delta // period
    
    def tone(self) -> int: return self._state.tzolkin_tone
    def glyph(self) -> str: return self._state.tzolkin_glyph
    def cr_pos(self) -> int: return self._state.calendar_round_pos
    def covering(self) -> Tuple[int, int]: return (self._state.covering_k, self._state.covering_r)
    def lc(self) -> Tuple[int, int, int, int, int]: return self._state.long_count
    def tier_819(self) -> Tuple[int, int, int]: return self._state.tier_819
    def sr_closure(self) -> Dict[int, bool]: return self._state.sr_closure
    
    def shadow_eclipse_strength(self) -> int:
        return self._state.shadow_power


# Missing backward-compat aliases
PairRelation = PairAnalysis
FULL_PRODUCT = reduce(lambda a, b: a * b, FULL_BASIS)
SAFE_PRODUCT = reduce(lambda a, b: a * b, SAFE_BASIS)
