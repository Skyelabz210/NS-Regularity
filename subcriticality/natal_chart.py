#!/usr/bin/env python3
"""
NATAL CHART: Anthony Diaz
October 21, 1980 — 5:31 PM EDT
Fort Bragg, North Carolina (35.139°N, 79.000°W)

Rewired to: astrology_engine.py (v4) + gear_manifold_v3.py
"""
from astrology_engine import Chart, BirthData, render_chart

BIRTH = BirthData(
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

if __name__ == '__main__':
    chart = Chart.from_birth(BIRTH)
    print(render_chart(chart))
    
    # Vedic overlay
    from astrology_support import AstrologySystem
    system = AstrologySystem()
    vedic = system.vedic(chart)
    print(f"\n  Vedic: Moon in {vedic['positions']['Moon']['nakshatra']}")
    print(f"  Dasha: {vedic['dasha']['mahadasha']} / {vedic['dasha']['antardasha']}")
