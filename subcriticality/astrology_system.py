#!/usr/bin/env python3
"""
ASTROLOGY SYSTEM — Unified Entry Point
========================================

Birth data in → complete three-layer reading out.
One API. All modules integrated.

Usage:
    system = AstrologySystem()

    # Natal chart
    chart = system.natal("Anthony Diaz", 1980, 10, 21, 17, 31, "EDT",
                         35.139, -79.000)
    print(chart.reading())

    # Current transits
    print(chart.transits_now())

    # Synastry
    other = system.natal("Person B", 1981, 9, 18, 15, 32, "CDT",
                         29.424, -98.494)
    print(system.synastry(chart, other))

    # Event timeline
    print(chart.timeline(years_forward=50))

    # Vedic overlay
    print(chart.vedic())

Depends on: all prior modules
Author: HackFate Research Laboratory
Date: 2026-03-25
"""

from __future__ import annotations
from typing import Dict, List, Tuple, Optional
from math import gcd

from astrology_engine import (
    PlanetManifold, CodexManifold, HouseTopology, PairRelation,
    BoundaryState, OmegaState, ChartFingerprint,
    ARCSEC_PER_DEG, FULL_CIRCLE, HALF_CIRCLE, ARCSEC_PER_SIGN,
    SAFE_BASIS, FULL_BASIS, SIGN_NAMES, CARRIER_ROLES, ASPECT_TONES,
    S_R_PRIMES, render as render_natal,
)
from transit_engine import (
    TransitEngine, detect_return, transit_is_active,
    build_synastry, render_transit_report, ephemeris_for_day,
    MAYA_EPOCH_JD,
)
from event_predictor import EventPredictor, maya_day_to_date
from house_data import (
    full_chart_placements, render_placements, placement_reading,
    transit_house_meaning, HOUSE_DATA, PLANET_ENERGY,
)
from vedic_layer import (
    dual_lane_positions, compute_dasha, dasha_timeline,
    render_vedic, test_shadow_inversion, scan_transit_inversions,
    ayanamsa_arcsec, sidereal_longitude, nakshatra_name,
    nakshatra_ruler, nakshatra_position,
)
from ephemeris_offline import ephemeris_for_day_auto


# ═══════════════════════════════════════════════════════════════════════════
# TIME CONVERSION — Gregorian to Maya day offset
# ═══════════════════════════════════════════════════════════════════════════

TZ_OFFSETS = {
    'UTC': 0, 'GMT': 0,
    'EST': -5, 'EDT': -4, 'CST': -6, 'CDT': -5,
    'MST': -7, 'MDT': -6, 'PST': -8, 'PDT': -7,
    'IST': 5,  # +5:30 approximated
}


def gregorian_to_maya_day(year: int, month: int, day: int,
                          hour: int = 12, minute: int = 0,
                          tz: str = 'UTC') -> int:
    """Convert Gregorian date+time to Maya day offset (integer).

    Uses Meeus JD formula → subtract Maya epoch.
    The hour/minute/timezone are used for house calculation JD
    but the Maya day offset is an integer (day resolution).
    """
    # UTC hour
    tz_offset = TZ_OFFSETS.get(tz.upper(), 0)
    utc_hour = hour - tz_offset + minute / 60

    # Meeus JD formula
    y, m = year, month
    if m <= 2:
        y -= 1
        m += 12
    A = y // 100
    B = 2 - A + A // 4
    jd = int(365.25 * (y + 4716)) + int(30.6001 * (m + 1)) + day + B - 1524
    # Add fractional day for house calculation
    jd_frac = utc_hour / 24.0

    # Maya day offset (integer)
    maya_day = jd - MAYA_EPOCH_JD
    return maya_day, jd + jd_frac


def compute_houses_swe(jd_frac: float, lat: float, lon: float) -> HouseTopology:
    """Compute Placidus houses via Swiss Ephemeris. Float boundary."""
    try:
        import swisseph as swe
        cusps, ascmc = swe.houses(jd_frac, lat, lon, b'P')
        asc = int(round(ascmc[0] * ARCSEC_PER_DEG))
        mc = int(round(ascmc[1] * ARCSEC_PER_DEG))
        cusp_list = [int(round(c * ARCSEC_PER_DEG)) for c in cusps]
        swe.close()
        return HouseTopology(cusp_list, asc, mc)
    except (ImportError, Exception):
        # Fallback: equal houses from approximate ASC
        # Very rough — production should always have swisseph
        asc = int((jd_frac * 360 * ARCSEC_PER_DEG) % FULL_CIRCLE)
        cusps = [(asc + i * ARCSEC_PER_SIGN) % FULL_CIRCLE for i in range(12)]
        mc = cusps[9]
        return HouseTopology(cusps, asc, mc)


# ═══════════════════════════════════════════════════════════════════════════
# NATAL CHART — computed once, queried forever
# ═══════════════════════════════════════════════════════════════════════════

class NatalChart:
    """A person's natal chart — all layers computed and cached."""

    def __init__(self, name: str, maya_day: int,
                 planet_data: Dict[str, Tuple[int, int, int]],
                 houses: HouseTopology,
                 jd_frac: float = 0):
        self.name = name
        self.maya_day = maya_day
        self.planet_data = planet_data
        self.houses = houses
        self.jd_frac = jd_frac

        # Core engine
        self.omega = OmegaState(maya_day, planet_data, houses)
        self.codex = self.omega.codex
        self.fingerprint = self.omega.fingerprint

        # Transit engine (for later use)
        self._transit_engine = TransitEngine(maya_day, planet_data, houses)

        # Event predictor
        self._predictor = EventPredictor(maya_day, planet_data)

        # Vedic positions
        self._vedic = dual_lane_positions(planet_data, maya_day)

        # Carrier and shadow classification
        self._shadow_planets = set()
        self._boundary_planets = set()
        for r in self.omega.shadow_network():
            self._shadow_planets.add(r.name_a)
            self._shadow_planets.add(r.name_b)
        for r in self.omega.boundary_network():
            self._boundary_planets.add(r.name_a)
            self._boundary_planets.add(r.name_b)
        self._retro = {n for n, p in self.omega.planets.items() if p.vel_neg}
        self._carrier_map = {n: CARRIER_ROLES[n]['label']
                             for n in self.omega.planets if n in CARRIER_ROLES}

    # ── Readings ──

    def residue_report(self) -> str:
        """Full three-layer residue-native report."""
        return render_natal(self.omega)

    def placements(self) -> str:
        """Three-layer house placement interpretations."""
        readings = full_chart_placements(
            self.planet_data, self.houses.find,
            self._shadow_planets, self._boundary_planets,
            self._carrier_map, self._retro)
        return render_placements(readings)

    def vedic(self) -> str:
        """Sidereal positions, nakshatras, Dasha timeline."""
        return render_vedic(self.planet_data, self.maya_day, self.maya_day)

    # ── Transits ──

    def transits_at(self, target_day: int) -> str:
        """Transit report for a specific day."""
        data = ephemeris_for_day_auto(target_day, self.houses.asc)
        return render_transit_report(self._transit_engine, target_day, data)

    def transits_now(self) -> str:
        """Transit report for today (approximate)."""
        # Today ~ March 25, 2026
        today = 1876817
        return self.transits_at(today)

    # ── Event prediction ──

    def timeline(self, years_forward: int = 50, years_back: int = 0) -> str:
        """All-at-once event timeline from modular arithmetic."""
        events = self._predictor.full_timeline(years_forward, years_back)
        lines = []
        lines.append(f"  EVENT TIMELINE — {self.name}")
        lines.append(f"  {len(events)} events predicted via modular arithmetic")
        lines.append(f"  {'─'*60}")

        prev_decade = -999
        for e in events:
            age = e.get('age', 0)
            if age < -years_back or age > years_forward:
                continue

            decade = age // 10
            if decade != prev_decade:
                lines.append(f"\n  ── Age {decade*10}s ──")
                prev_decade = decade

            etype = e.get('aspect', e.get('type', '?'))
            detail = ""
            if 'transit' in e and 'natal' in e:
                detail = f"{e['transit']}→{e['natal']}"
            elif 'planet' in e:
                detail = e['planet']

            date = maya_day_to_date(e['day'])
            lines.append(f"    age {age:3d}  {date}  {etype:16s}  {detail}")

        return "\n".join(lines)

    def events_at_age(self, age: int) -> List[Dict]:
        """What events fire around a specific age?"""
        return self._predictor.events_at_age(age)

    # ── Dasha ──

    def current_dasha(self, target_day: Optional[int] = None) -> Dict:
        """Current Vimsottari Dasha period."""
        if target_day is None:
            target_day = 1876817  # today
        moon_sid = self._vedic['Moon']['sidereal']
        return compute_dasha(moon_sid, self.maya_day, target_day)

    def dasha_tl(self, years: int = 80) -> List[Dict]:
        """Full Dasha timeline."""
        moon_sid = self._vedic['Moon']['sidereal']
        return dasha_timeline(moon_sid, self.maya_day, years)

    # ── Chart identity ──

    def similarity(self, other: NatalChart) -> int:
        """Fingerprint similarity score."""
        return self.fingerprint.similarity(other.fingerprint)


# ═══════════════════════════════════════════════════════════════════════════
# SYNASTRY REPORT
# ═══════════════════════════════════════════════════════════════════════════

class SynastryReport:
    """Complete synastry analysis between two charts."""

    def __init__(self, chart_a: NatalChart, chart_b: NatalChart):
        self.a = chart_a
        self.b = chart_b
        mid_day = (chart_a.maya_day + chart_b.maya_day) // 2
        self.relations = build_synastry(
            chart_a.planet_data, chart_b.planet_data, mid_day)
        self.relations.sort(key=lambda r: r.lock_score, reverse=True)

    def shadow_bonds(self) -> List[PairRelation]:
        return [r for r in self.relations if r.shadow.active]

    def boundary_events(self) -> List[PairRelation]:
        return [r for r in self.relations if r.boundary.active]

    def non_aspect_locks(self, threshold: int = 250) -> List[PairRelation]:
        return [r for r in self.relations
                if r.lock_score >= threshold and not r._has_aspect]

    def shadow_inversions(self) -> List[Dict]:
        """Scan for true S(t+T) = -S(t) between charts."""
        inversions = []
        for ra in self.a.omega.relations:
            for rb in self.b.omega.relations:
                if ra.name_a == rb.name_a and ra.name_b == rb.name_b:
                    result = test_shadow_inversion(ra.diff_safe, rb.diff_safe)
                    if result['is_shadow_inversion']:
                        inversions.append({
                            'pair': f"{ra.name_a}-{ra.name_b}",
                            'a_diff': ra.diff_safe,
                            'b_diff': rb.diff_safe,
                            **result,
                        })
        return inversions

    def top(self, n: int = 10) -> List[PairRelation]:
        return self.relations[:n]

    def summary(self) -> Dict:
        return {
            'total': len(self.relations),
            'shadow_bonds': len(self.shadow_bonds()),
            'boundary_events': len(self.boundary_events()),
            'non_aspect_locks': len(self.non_aspect_locks()),
            'inversions': len(self.shadow_inversions()),
            'fingerprint_similarity': self.a.similarity(self.b),
        }

    def render(self) -> str:
        """Full synastry report."""
        lines = []
        lines.append("=" * 70)
        lines.append(f"  SYNASTRY: {self.a.name} × {self.b.name}")
        lines.append(f"  {len(self.relations)} cross-relations")
        lines.append("=" * 70)

        s = self.summary()
        lines.append(f"\n  Shadow bonds:      {s['shadow_bonds']}")
        lines.append(f"  Boundary events:   {s['boundary_events']}")
        lines.append(f"  Non-aspect locks:  {s['non_aspect_locks']}")
        lines.append(f"  Shadow inversions: {s['inversions']}")
        lines.append(f"  Fingerprint match: {s['fingerprint_similarity']}/60")

        # Top connections
        lines.append(f"\n{'─'*70}")
        lines.append(f"  TOP 10 CONNECTIONS")
        lines.append(f"{'─'*70}")
        for r in self.top(10):
            asp = r.classify_aspect()
            an = asp['name'] if asp else "non-aspect"
            orb = f"{asp['orb']//ARCSEC_PER_DEG}°{(asp['orb']%ARCSEC_PER_DEG)//60:02d}'" if asp else "—"
            s_mark = "S" if r.shadow.active else "·"
            b_mark = "B" if r.boundary.active else "·"
            lines.append(f"  {r.name_a:14s}→{r.name_b:14s}  {an:14s}  "
                         f"{orb:>6s}  lock={r.lock_score:4d}  {s_mark}{b_mark}")

        # Shadow inversions
        inv = self.shadow_inversions()
        if inv:
            lines.append(f"\n{'─'*70}")
            lines.append(f"  SHADOW INVERSIONS — S(t+T) = -S(t)")
            lines.append(f"{'─'*70}")
            for i in inv:
                lines.append(f"  {i['pair']:20s}  neg={i['negation_lanes']}  "
                             f"A={i['a_diff']}  B={i['b_diff']}")

        lines.append("\n" + "=" * 70)
        return "\n".join(lines)


# ═══════════════════════════════════════════════════════════════════════════
# ASTROLOGY SYSTEM — the unified API
# ═══════════════════════════════════════════════════════════════════════════

class AstrologySystem:
    """Unified entry point for the residue-native astrology engine.

    Usage:
        system = AstrologySystem()
        chart = system.natal("Name", 1980, 10, 21, 17, 31, "EDT", 35.139, -79.0)
        print(chart.residue_report())
        print(chart.placements())
        print(chart.vedic())
        print(chart.transits_now())
        print(chart.timeline())
    """

    def natal(self, name: str,
              year: int, month: int, day: int,
              hour: int = 12, minute: int = 0,
              tz: str = 'UTC',
              lat: float = 0.0, lon: float = 0.0) -> NatalChart:
        """Compute a complete natal chart from birth data.

        Args:
            name: Person's name
            year, month, day: Birth date
            hour, minute: Birth time (local)
            tz: Timezone string (EST, EDT, CST, CDT, etc.)
            lat, lon: Birth latitude/longitude (decimal degrees)
        """
        maya_day, jd_frac = gregorian_to_maya_day(year, month, day, hour, minute, tz)

        # Houses
        houses = compute_houses_swe(jd_frac, lat, lon)

        # Planetary positions
        planet_data = ephemeris_for_day_auto(maya_day, houses.asc)

        return NatalChart(name, maya_day, planet_data, houses, jd_frac)

    def synastry(self, chart_a: NatalChart,
                 chart_b: NatalChart) -> SynastryReport:
        """Compute complete synastry between two charts."""
        return SynastryReport(chart_a, chart_b)

    def from_data(self, name: str, maya_day: int,
                  planet_data: Dict[str, Tuple[int, int, int]],
                  houses: HouseTopology) -> NatalChart:
        """Create chart from pre-computed data (no ephemeris needed)."""
        return NatalChart(name, maya_day, planet_data, houses)


# ═══════════════════════════════════════════════════════════════════════════
# DEMO
# ═══════════════════════════════════════════════════════════════════════════

if __name__ == '__main__':
    system = AstrologySystem()

    # ── Chart A: Anthony Diaz ──
    print("Computing Chart A...")
    a = system.natal("Anthony Diaz", 1980, 10, 21, 17, 31, "EDT", 35.139, -79.000)

    # ── Chart B: Test ──
    print("Computing Chart B...")
    b = system.natal("Chart B", 1981, 9, 18, 15, 32, "CDT", 29.424, -98.494)

    # ── Natal reports ──
    print(a.residue_report())
    print(a.placements())
    print(a.vedic())

    # ── Synastry ──
    syn = system.synastry(a, b)
    print(syn.render())

    # ── Current Dasha ──
    da = a.current_dasha()
    db = b.current_dasha()
    print(f"\n  Dasha A: {da['mahadasha']} / {da['antardasha']}")
    print(f"  Dasha B: {db['mahadasha']} / {db['antardasha']}")

    # ── Event timeline (short) ──
    events_a = a.events_at_age(45)
    if events_a:
        print(f"\n  Events at age 45 for {a.name}:")
        for e in events_a[:5]:
            print(f"    {e.get('aspect', e.get('type', '?'))}: "
                  f"{e.get('transit', '')}{' → ' + e.get('natal', '') if 'natal' in e else ''}")

    print("\n  System operational. All modules integrated.")
