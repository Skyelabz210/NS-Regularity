#!/usr/bin/env python3
"""
TRANSIT ENGINE — Remastered Astrology
======================================
Implements pseudocode spec sections 6-22.
Depends on: astrology_engine.py, gear_manifold_v3.py, pyswisseph
No float in computation. Swiss Ephemeris floats converted at boundary.
"""
from __future__ import annotations
from typing import Dict, List, Tuple, Optional, Callable
from math import gcd
from astrology_engine import (
    PlanetManifold, CodexManifold, HouseTopology, PairRelation,
    BoundaryState, ChartFingerprint,
    ARCSEC_PER_DEG, FULL_CIRCLE, HALF_CIRCLE, ARCSEC_PER_SIGN,
    SAFE_BASIS, FULL_BASIS, CODEX_CYCLES, ASPECT_TABLE, ASPECT_TONES,
    CARRIER_ROLES, S_R_PRIMES, SIGN_NAMES, _fp, _fo,
)

MAYA_EPOCH_JD = 584283
PLANET_IDS = {
    'Sun': 0, 'Moon': 1, 'Mercury': 2, 'Venus': 3, 'Mars': 4,
    'Jupiter': 5, 'Saturn': 6, 'Uranus': 7, 'Neptune': 8, 'Pluto': 9,
}
THRESHOLD_LOCK = 350
THRESHOLD_LANE_SCORE = 5

def ephemeris_for_day(day_offset, asc_arcsec=0):
    import swisseph as swe
    jd = MAYA_EPOCH_JD + day_offset
    result = {}
    for name, pid in PLANET_IDS.items():
        calc = swe.calc_ut(jd, pid, 256)
        lon_as = int(round(calc[0][0] * ARCSEC_PER_DEG))
        vel_as = int(round(calc[0][3] * ARCSEC_PER_DEG))
        house_as = (lon_as - asc_arcsec) % FULL_CIRCLE
        result[name] = (lon_as, vel_as, house_as)
    swe.close()
    return result

class TransitEngine:
    def __init__(self, natal_day, natal_data, houses=None):
        self.natal_day = natal_day
        self.natal_codex = CodexManifold(natal_day)
        self.natal_planets = {n: PlanetManifold(n, *d) for n, d in natal_data.items()}
        self.houses = houses

    def evaluate_day(self, transit_day, transit_data):
        tc = CodexManifold(transit_day)
        tps = {n: PlanetManifold(f"t_{n}", *d) for n, d in transit_data.items()}
        rels = []
        for tn, tp in tps.items():
            for nn, np in self.natal_planets.items():
                rels.append(PairRelation(tp, np, tc))
        return rels

    def scan_window(self, start, end, ephem_fn, threshold=350):
        results = {}
        for day in range(start, end + 1):
            td = ephem_fn(day)
            rels = self.evaluate_day(day, td)
            active = [r for r in rels if r.lock_score >= threshold]
            if active:
                results[day] = active
        return results

def detect_return(tp, np, codex):
    diff = tuple((tp.r_full()[i] - np.r_full()[i]) % FULL_BASIS[i] for i in range(6))
    ds = diff[:5]
    safe_ret = all(d == 0 for d in ds)
    full_ret = all(d == 0 for d in diff)
    ang = abs(tp._lon - np._lon) % FULL_CIRCLE
    if ang > HALF_CIRCLE: ang = FULL_CIRCLE - ang
    non_ret = [SAFE_BASIS[i] for i in range(5) if ds[i] != 0]
    return {
        'planet': np.name, 'is_safe_return': safe_ret, 'is_full_return': full_ret,
        'angular_distance': ang, 'diff_safe': ds, 'non_return_lanes': non_ret,
        'sr_closure': codex.sr_closure(), 'boundary_clock': codex.r('819') % 13,
    }

def progressed_day(natal_day, age_years): return natal_day + age_years

def solar_arc(natal_sun, prog_sun): return (prog_sun - natal_sun) % FULL_CIRCLE

def directed_planet(natal_lon, arc): return (natal_lon + arc) % FULL_CIRCLE

def build_synastry(chart_a, chart_b, day_offset):
    codex = CodexManifold(day_offset)
    pa = {n: PlanetManifold(f"A_{n}", *d) for n, d in chart_a.items()}
    pb = {n: PlanetManifold(f"B_{n}", *d) for n, d in chart_b.items()}
    return [PairRelation(ap, bp, codex) for ap in pa.values() for bp in pb.values()]

def transit_is_active(rel, lock_t=THRESHOLD_LOCK, lane_t=THRESHOLD_LANE_SCORE):
    if rel.lock_score < lock_t: return False
    asp = rel.classify_aspect()
    if asp is None:
        return rel.shadow.strength >= 300 or rel.boundary.strength >= 200
    return asp.get('lane_score', 0) >= lane_t

def run_tests():
    failures = 0
    def chk(name, cond):
        nonlocal failures
        if not cond: print(f"  FAIL: {name}"); failures += 1
    print("── Test Suite ──")
    for n, a in [('conj',0),('sext',60),('sq',90),('tri',120),('opp',180)]:
        r = tuple((a * ARCSEC_PER_DEG) % p for p in SAFE_BASIS)
        chk(f"{n} mod2=0", r[0]==0); chk(f"{n} mod3=0", r[1]==0); chk(f"{n} mod5=0", r[2]==0)
    chk("opp mod11=1", (180*ARCSEC_PER_DEG)%11==1)
    chk("819 bnd clock", CodexManifold(819).r('819')%13==0)
    chk("eclipse shd clock", CodexManifold(11960).r('eclipse')%11==0)
    for n, e in [('conjunction',1),('square',2),('opposition',3),('sextile',6),('trine',11)]:
        chk(f"{n} tone={e}", ASPECT_TONES.get(n,0)==e)
    chk("Xultun", 1195740==4*9*5*7*13*73)
    chk("Mercury", 116==4*29 and 116%11!=0)
    chk("lcm(260,365)", 260*365//gcd(260,365)==18980)
    chk("gcd(11960,93)", gcd(11960,93)==1)
    chk("LC mod CR", 1872000%18980==11960)
    chk("Δ_Mars", 11960%780==260); chk("Δ_Venus", 11960%584==280)
    chk("Δ_Saturn", 11960%378==242); chk("242=2×11²", 242==2*121)
    print(f"  {'All passed' if failures==0 else f'{failures} FAILED'}."); return failures

def render_transit_report(engine, tday, tdata):
    L = []
    rels = sorted(engine.evaluate_day(tday, tdata), key=lambda r: r.lock_score, reverse=True)
    tc = CodexManifold(tday)
    L.append("="*70)
    L.append("  TRANSIT REPORT — REMASTERED")
    L.append(f"  Transit: day {tday} | Natal: day {engine.natal_day}")
    L.append(f"  Delta: {tday - engine.natal_day} days ({(tday - engine.natal_day)//365} yrs)")
    L.append("="*70)
    L.append(f"\n  Codex: {tc.tone()} {tc.glyph()}")
    wk, wr = tc.covering()
    L.append(f"  Covering: K={wk} r={wr}")
    for p, cl in sorted(tc.sr_closure().items()):
        L.append(f"  S_R[{p}]: {'CLOSED' if cl else 'OPEN'}")
    L.append(f"\n  Shadow clocks:")
    for ck in ('eclipse','saturn','819','double_cr'):
        m = tc.r(ck)%11; at = m==0 and tc.r(ck)>0
        L.append(f"    {ck:12s}  {'▶ AT' if at else f'  {11-m} to next'}")
    bclk = BoundaryState.boundary_clock(tc)
    L.append(f"  Boundary clocks:")
    for ck, info in bclk.items():
        tn = info['to_next']
        L.append(f"    {ck:12s}  {'▶ AT' if info['at_boundary'] else f'  {tn} to next'}")
    active = [r for r in rels if transit_is_active(r)]
    shd_only = [r for r in active if r.shadow.active and not r._has_aspect]
    L.append(f"\n{'─'*70}")
    L.append(f"  ACTIVE TRANSITS: {len(active)} (shadow-only: {len(shd_only)})")
    L.append(f"{'─'*70}")
    L.append(f"  {'Transit→Natal':30s}  {'Aspect':13s}  {'Orb':>6s}  {'Lock':>4s}  {'SB':>2s}  {'T':>2s}")
    L.append(f"  {'─'*30}  {'─'*13}  {'─'*6}  {'─'*4}  {'─'*2}  {'─'*2}")
    for r in active[:35]:
        pair = f"{r.name_a}→{r.name_b}"
        asp = r.classify_aspect()
        if asp:
            L.append(f"  {pair:30s}  {asp['name']:13s}  {_fo(asp['orb']):>6s}  "
                     f"{r.lock_score:4d}  {'●' if r.shadow.active else '·'}"
                     f"{'●' if r.boundary.active else '·'}  {asp.get('tone',0):2d}")
        else:
            tag = "—shadow—" if r.shadow.active else "—boundary—"
            L.append(f"  {pair:30s}  {tag:13s}  {'—':>6s}  "
                     f"{r.lock_score:4d}  {'●' if r.shadow.active else '·'}"
                     f"{'●' if r.boundary.active else '·'}   ·")
    # Returns
    L.append(f"\n{'─'*70}")
    L.append(f"  RETURN ANALYSIS")
    L.append(f"{'─'*70}")
    tps = {n: PlanetManifold(n, *d) for n, d in tdata.items()}
    for name in engine.natal_planets:
        if name in tps:
            ret = detect_return(tps[name], engine.natal_planets[name], tc)
            if ret['angular_distance'] < 15 * ARCSEC_PER_DEG:
                st = "EXACT" if ret['is_safe_return'] else "APPROACHING"
                L.append(f"  {name:10s} {st}  dist={ret['angular_distance']//ARCSEC_PER_DEG}°  "
                         f"gaps={ret['non_return_lanes']}")
    L.append("="*70)
    return "\n".join(L)

if __name__ == '__main__':
    print("="*70)
    print("  TRANSIT ENGINE — REMASTERED ASTROLOGY")
    print("="*70)
    run_tests()
    NATAL_DAY = 1860251
    NATAL = {
        'Sun':(751107,3581,727288),'Moon':(1810,54173,1273991),
        'Mercury':(827399,540,803580),'Venus':(611243,4272,587424),
        'Mars':(888870,2604,865051),'Jupiter':(644043,724,620224),
        'Saturn':(661345,419,637526),'Uranus':(843177,207,819358),
        'Neptune':(938127,93,914308),'Pluto':(727106,143,703287),
    }
    HOUSES = HouseTopology(
        [23819,160400,256749,337956,421275,524762,
         671819,808400,904749,985956,1069275,1172762], 23819, 985956)
    engine = TransitEngine(NATAL_DAY, NATAL, HOUSES)
    TODAY = 1876816  # ~Mar 24, 2026
    print(f"\n  Age: {(TODAY-NATAL_DAY)//365} years")
    try:
        td = ephemeris_for_day(TODAY, HOUSES.asc)
        print(render_transit_report(engine, TODAY, td))
    except Exception as e:
        print(f"  Ephemeris error: {e}")
        print("  Using synthetic transits...")
        syn = {}
        speeds = {'Sun':3548,'Moon':47800,'Mercury':5400,'Venus':4200,'Mars':2300,
                  'Jupiter':830,'Saturn':400,'Uranus':170,'Neptune':90,'Pluto':50}
        for n,(l,v,h) in NATAL.items():
            syn[n] = ((l + speeds[n]*(TODAY-NATAL_DAY))%FULL_CIRCLE, speeds[n], h)
        print(render_transit_report(engine, TODAY, syn))
