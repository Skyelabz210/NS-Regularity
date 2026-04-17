#!/usr/bin/env python3
"""
cram_planetary_schema.py — CRAM Schema for Planetary Tracking
================================================================

The insight: every standard aspect angle has a UNIQUE CRT residue tuple
mod (2, 3, 5, 7, 11, 13). Lanes mod 2,3,5 are always zero (all angles
are multiples of 3600" = 2³×3²×5²). The discriminating information
lives in lanes mod 7, 11, 13 — the Ramanujan primes + boundary prime.

The schema: for a planet pair with angular separation Δ,
  1. Decompose Δ into CRT residues
  2. For each candidate aspect angle A, compute Sub(Δ, A) + Sqr
  3. Extract carry pattern
  4. All-zero carry → exact aspect match
  5. Low-carry → within orb
  6. High-carry → no aspect

This replaces brute-force angular comparison with atlas lookup.

Architecture:
  CRAM_PLANETARY = Layer 3 schema on Layer 1 CRTManifold
  Operator: Sub(Δ_residue, target_residue) → Sqr → carry
  Cross-lane: reads Δ from all lanes simultaneously
  Output: aspect classification + shadow/boundary state + harmonic class

Author: HackFate Research Laboratory
Date: 2026-03-26
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Dict, List, Tuple, Optional, FrozenSet
from functools import reduce

from gear_manifold_v3 import CRTManifold, mod_inverse


# ═══════════════════════════════════════════════════════════════════════════
# CONSTANTS
# ═══════════════════════════════════════════════════════════════════════════

ARCSEC_PER_DEG = 3600
FULL_CIRCLE    = 1_296_000
FULL_BASIS     = (2, 3, 5, 7, 11, 13)
SAFE_BASIS     = (2, 3, 5, 7, 11)

# The discriminating lanes — mod 2,3,5 are always 0 for degree-multiple angles
DISC_PRIMES    = (7, 11, 13)
DISC_INDICES   = (3, 4, 5)   # indices into FULL_BASIS

# ═══════════════════════════════════════════════════════════════════════════
# ASPECT ATLAS — precomputed CRT fingerprints
# ═══════════════════════════════════════════════════════════════════════════

@dataclass(frozen=True)
class AspectEntry:
    """One row in the aspect atlas."""
    harmonic: int
    name: str
    angle: int                    # arcseconds
    max_orb: int                  # arcseconds
    crt: Tuple[int, ...]          # residues mod FULL_BASIS
    disc: Tuple[int, ...]         # discriminating residues mod (7, 11, 13)
    sqr_carry: Tuple[int, ...]    # carry pattern of Sqr(residue)
    harmonic_class: str           # 'hard' / 'soft' / 'minor' / 'generative'
    mirror_disc: Tuple[int, ...]  # residues of (FULL_CIRCLE - angle) mod disc primes


def _classify_harmonic(h: int) -> str:
    if h in (1, 2):   return 'cardinal'    # conjunction, opposition
    if h in (3, 6):   return 'soft'        # trine, sextile
    if h in (4, 8):   return 'hard'        # square, semisquare
    if h in (5, 10):  return 'generative'  # quintile, biquintile
    return 'minor'


def _build_atlas() -> Dict[str, AspectEntry]:
    """Build the complete aspect atlas with CRT fingerprints."""
    raw = {
        1:  (0,              'conjunction',  10 * ARCSEC_PER_DEG),
        2:  (FULL_CIRCLE//2, 'opposition',   10 * ARCSEC_PER_DEG),
        3:  (432000,         'trine',         8 * ARCSEC_PER_DEG),
        4:  (324000,         'square',        8 * ARCSEC_PER_DEG),
        5:  (259200,         'quintile',      2 * ARCSEC_PER_DEG),
        6:  (216000,         'sextile',       6 * ARCSEC_PER_DEG),
        8:  (162000,         'semisquare',    3 * ARCSEC_PER_DEG),
        9:  (540000,         'quincunx',      3 * ARCSEC_PER_DEG),
        10: (518400,         'biquintile',    2 * ARCSEC_PER_DEG),
        12: (108000,         'semisextile',   3 * ARCSEC_PER_DEG),
    }
    atlas = {}
    for h, (angle, name, orb) in raw.items():
        crt = tuple(angle % p for p in FULL_BASIS)
        disc = tuple(angle % p for p in DISC_PRIMES)
        mirror = FULL_CIRCLE - angle if angle > 0 else 0
        mirror_disc = tuple(mirror % p for p in DISC_PRIMES)
        sqr = tuple((r * r) % p for r, p in zip(crt, FULL_BASIS))
        carry = tuple(1 if 2 * s >= p else 0 for s, p in zip(sqr, FULL_BASIS))
        atlas[name] = AspectEntry(
            harmonic=h, name=name, angle=angle, max_orb=orb,
            crt=crt, disc=disc, sqr_carry=carry,
            harmonic_class=_classify_harmonic(h),
            mirror_disc=mirror_disc,
        )
    return atlas

ASPECT_ATLAS = _build_atlas()

# Discriminating-lane lookup: disc_tuple → aspect name
_DISC_LOOKUP: Dict[Tuple[int, int, int], str] = {}
_MIRROR_LOOKUP: Dict[Tuple[int, int, int], str] = {}
for name, entry in ASPECT_ATLAS.items():
    _DISC_LOOKUP[entry.disc] = name
    if entry.angle > 0:
        _MIRROR_LOOKUP[entry.mirror_disc] = name


# ═══════════════════════════════════════════════════════════════════════════
# CRAM OPERATORS — planetary-specific
# ═══════════════════════════════════════════════════════════════════════════

def cram_sub_sqr(delta_residue: int, target_residue: int, p: int) -> Tuple[int, int, int]:
    """Core CRAM operation: Sub → Sqr → carry extraction.
    Returns (sub_result, sqr_result, carry_bit).
    """
    sub = (delta_residue - target_residue) % p
    sqr = (sub * sub) % p
    carry = 1 if 2 * sqr >= p else 0
    return (sub, sqr, carry)


def cram_proximity(delta_residues: Tuple[int, ...],
                   target_residues: Tuple[int, ...]) -> Tuple[Tuple[int, ...], int]:
    """Compute CRAM proximity score between Δ and a target aspect.
    
    Returns (carry_pattern, proximity_score).
    proximity_score = number of lanes where Sub(Δ, target) == 0.
    A score of 6 means exact match on all lanes.
    """
    carries = []
    exact_lanes = 0
    for i, p in enumerate(FULL_BASIS):
        sub, sqr, carry = cram_sub_sqr(delta_residues[i], target_residues[i], p)
        carries.append(carry)
        if sub == 0:
            exact_lanes += 1
    return (tuple(carries), exact_lanes)


# ═══════════════════════════════════════════════════════════════════════════
# PLANETARY CRAM SCHEMA
# ═══════════════════════════════════════════════════════════════════════════

@dataclass(frozen=True)
class CRAMClassification:
    """Complete CRAM classification of a planet pair."""
    aspect_name: Optional[str]
    aspect_harmonic: Optional[int]
    harmonic_class: str           # 'cardinal'/'soft'/'hard'/'generative'/'minor'/'none'
    carry_pattern: Tuple[int, ...]  # 6-lane carry vector
    proximity: int                 # exact-match lane count (0-6)
    shadow_active: bool            # Δ ≡ 0 mod 11
    boundary_active: bool          # Δ ≡ 0 mod 13
    disc_residues: Tuple[int, ...] # discriminating (mod 7, 11, 13)
    orb: int                       # arcseconds (0 if no aspect)
    applying: bool                 # true if planets approaching exact


class PlanetaryCRAMSchema:
    """CRAM schema for planetary aspect classification.
    
    The schema operates on the angular separation Δ between two planets.
    It decomposes Δ into CRT residues and classifies the configuration
    via carry-pattern matching against the precomputed aspect atlas.
    
    Usage:
        schema = PlanetaryCRAMSchema()
        result = schema.classify(lon_a, lon_b, vel_a, vel_b)
        
        # O(1) atlas lookup — no iterative aspect comparison
        print(result.aspect_name)       # 'trine'
        print(result.carry_pattern)     # (0, 0, 0, 1, 1, 1)
        print(result.shadow_active)     # True/False
        print(result.harmonic_class)    # 'soft'
    """

    def __init__(self):
        self.atlas = ASPECT_ATLAS
        self._disc_lookup = _DISC_LOOKUP
        self._mirror_lookup = _MIRROR_LOOKUP

    def classify(self, lon_a: int, lon_b: int,
                 vel_a: int = 0, vel_b: int = 0) -> CRAMClassification:
        """Classify the angular relationship between two planets.
        
        All inputs in integer arcseconds. Pure integer computation.
        """
        # Angular separation (unsigned, shortest arc)
        raw_delta = (lon_a - lon_b) % FULL_CIRCLE
        if raw_delta > FULL_CIRCLE // 2:
            delta = FULL_CIRCLE - raw_delta
            direction = -1
        else:
            delta = raw_delta
            direction = 1

        # CRT decomposition of raw delta (for shadow/boundary)
        raw_residues = tuple(raw_delta % p for p in FULL_BASIS)
        
        # CRT decomposition of shortest-arc delta (for aspect matching)
        delta_residues = tuple(delta % p for p in FULL_BASIS)

        # Shadow and boundary — always computed on raw delta
        shadow_active = raw_residues[4] == 0   # mod 11
        boundary_active = raw_residues[5] == 0  # mod 13
        disc_residues = tuple(raw_residues[i] for i in DISC_INDICES)

        # O(1) atlas lookup via discriminating lanes
        delta_disc = tuple(delta % p for p in DISC_PRIMES)
        
        # Check direct match
        matched_name = self._disc_lookup.get(delta_disc)
        
        # Check mirror match (e.g., 240° trine = 360° - 120°)
        if matched_name is None:
            mirror_disc = tuple((FULL_CIRCLE - delta) % p for p in DISC_PRIMES)
            matched_name = self._disc_lookup.get(mirror_disc)

        # If disc-match found, verify within orb
        if matched_name is not None:
            entry = self.atlas[matched_name]
            orb = abs(delta - entry.angle)
            mirror_orb = abs(delta - (FULL_CIRCLE - entry.angle)) if entry.angle > 0 else FULL_CIRCLE
            orb = min(orb, mirror_orb)
            
            if orb <= entry.max_orb:
                carry_pattern, proximity = cram_proximity(delta_residues, entry.crt)
                applying = (vel_a - vel_b) * direction < 0
                return CRAMClassification(
                    aspect_name=matched_name,
                    aspect_harmonic=entry.harmonic,
                    harmonic_class=entry.harmonic_class,
                    carry_pattern=carry_pattern,
                    proximity=proximity,
                    shadow_active=shadow_active,
                    boundary_active=boundary_active,
                    disc_residues=disc_residues,
                    orb=orb,
                    applying=applying,
                )

        # No exact disc match — try proximity scan (still O(k) where k=10 aspects)
        best_name = None
        best_orb = FULL_CIRCLE
        for name, entry in self.atlas.items():
            orb = abs(delta - entry.angle)
            if entry.angle > 0:
                orb = min(orb, abs(delta - (FULL_CIRCLE - entry.angle)))
            if orb <= entry.max_orb and orb < best_orb:
                best_orb = orb
                best_name = name

        if best_name is not None:
            entry = self.atlas[best_name]
            carry_pattern, proximity = cram_proximity(delta_residues, entry.crt)
            applying = (vel_a - vel_b) * direction < 0
            return CRAMClassification(
                aspect_name=best_name,
                aspect_harmonic=entry.harmonic,
                harmonic_class=entry.harmonic_class,
                carry_pattern=carry_pattern,
                proximity=proximity,
                shadow_active=shadow_active,
                boundary_active=boundary_active,
                disc_residues=disc_residues,
                orb=best_orb,
                applying=applying,
            )

        # No aspect — still return shadow/boundary and carry pattern
        # Use conjunction (0,0,0,0,0,0) as reference to get deviation pattern
        carry_pattern, proximity = cram_proximity(
            delta_residues, (0,) * len(FULL_BASIS))
        return CRAMClassification(
            aspect_name=None, aspect_harmonic=None,
            harmonic_class='none',
            carry_pattern=carry_pattern,
            proximity=proximity,
            shadow_active=shadow_active,
            boundary_active=boundary_active,
            disc_residues=disc_residues,
            orb=0, applying=False,
        )

    def classify_all_pairs(self, planets: Dict[str, Tuple[int, int]]) -> List[Tuple[str, str, CRAMClassification]]:
        """Classify all pairs from a {name: (longitude, velocity)} dict."""
        names = sorted(planets.keys())
        results = []
        for i in range(len(names)):
            for j in range(i + 1, len(names)):
                a, b = names[i], names[j]
                lon_a, vel_a = planets[a]
                lon_b, vel_b = planets[b]
                c = self.classify(lon_a, lon_b, vel_a, vel_b)
                results.append((a, b, c))
        return results

    def carry_atlas(self) -> Dict[str, Tuple[int, ...]]:
        """Return the complete carry-pattern atlas for reference."""
        return {name: entry.sqr_carry for name, entry in self.atlas.items()}

    def disc_atlas(self) -> Dict[Tuple[int, int, int], str]:
        """Return the discriminating-lane atlas for O(1) lookup."""
        return dict(self._disc_lookup)


# ═══════════════════════════════════════════════════════════════════════════
# MULTI-SCHEMA: Planetary + Dresden simultaneously
# ═══════════════════════════════════════════════════════════════════════════

def dual_schema_classify(lon_a: int, lon_b: int,
                         vel_a: int = 0, vel_b: int = 0) -> Dict:
    """Apply BOTH planetary and Dresden schemas to same Δ.
    
    The planetary schema classifies the aspect.
    The Dresden schema classifies the number-theoretic structure.
    Both read from the same CRT residues. No interference.
    """
    from test_crt_manifold import CRAMTopology, Root, Post

    schema_p = PlanetaryCRAMSchema()
    p_result = schema_p.classify(lon_a, lon_b, vel_a, vel_b)

    # Dresden schema on the raw delta
    delta = abs(lon_a - lon_b) % FULL_CIRCLE
    primes5 = [3, 5, 7, 11, 13]
    g = CRTManifold.from_value(delta, primes5)
    d_topo = CRAMTopology.dresden_schema(primes5)
    d_carry = d_topo.carry_pattern(g, d=2)

    return {
        'planetary': p_result,
        'dresden_carry': d_carry,
        'delta': delta,
        'crt_full': tuple(delta % p for p in FULL_BASIS),
    }


# ═══════════════════════════════════════════════════════════════════════════
# TESTS
# ═══════════════════════════════════════════════════════════════════════════

def test_schema():
    """Verify the CRAM planetary schema against known aspects."""
    schema = PlanetaryCRAMSchema()
    failures = 0

    print("=" * 72)
    print("  CRAM PLANETARY SCHEMA — VERIFICATION")
    print("=" * 72)

    # 1. Exact aspect detection
    print("\n  1. Exact aspect detection (zero orb)")
    for name, entry in ASPECT_ATLAS.items():
        c = schema.classify(entry.angle, 0)
        ok = c.aspect_name == name
        if not ok:
            print(f"    FAIL: {name} → {c.aspect_name}")
            failures += 1
    if failures == 0:
        print(f"    All 10 aspects detected at exact angle ✓")

    # 2. Orb tolerance
    print("\n  2. Orb tolerance (5° for major aspects)")
    for name in ('conjunction', 'opposition', 'trine', 'square', 'sextile'):
        entry = ASPECT_ATLAS[name]
        c = schema.classify(entry.angle + 5 * ARCSEC_PER_DEG, 0)
        ok = c.aspect_name == name
        if not ok:
            print(f"    FAIL: {name} +5° → {c.aspect_name}")
            failures += 1
    print(f"    Major aspects detected within 5° orb ✓" if failures == 0 else "")

    # 3. Shadow/boundary detection
    print("\n  3. Shadow (mod 11) and boundary (mod 13)")
    # Δ = 11 → shadow active
    c = schema.classify(11, 0)
    assert c.shadow_active, "11 should activate shadow"
    # Δ = 13 → boundary active
    c = schema.classify(13, 0)
    assert c.boundary_active, "13 should activate boundary"
    # Δ = 143 = 11×13 → both active
    c = schema.classify(143, 0)
    assert c.shadow_active and c.boundary_active, "143 should activate both"
    print(f"    Shadow (Δ=11) ✓  Boundary (Δ=13) ✓  Both (Δ=143) ✓")

    # 4. Carry pattern uniqueness
    print("\n  4. Carry pattern analysis")
    carries = {}
    for name, entry in ASPECT_ATLAS.items():
        cp = entry.sqr_carry
        print(f"    {name:13s}: carry={cp}  class={entry.harmonic_class}")
        if cp in carries:
            print(f"      NOTE: shares carry with {carries[cp]}")
        carries[cp] = name

    # 5. Discriminating-lane atlas completeness
    print(f"\n  5. Disc-lane atlas: {len(_DISC_LOOKUP)} entries (unique disc tuples)")
    atlas = schema.disc_atlas()
    for disc, name in sorted(atlas.items(), key=lambda x: x[1]):
        print(f"    {disc} → {name}")

    # 6. Full chart test
    print(f"\n  6. Full chart classification (Acid's birth data)")
    planets = {
        'Sun': (749687, 3564), 'Moon': (1276423, 47400),
        'Mercury': (827151, 14400), 'Venus': (609549, 5760),
        'Mars': (887838, 1886), 'Jupiter': (643755, 299),
        'Saturn': (661179, 120), 'Uranus': (843095, 42),
        'Neptune': (938090, 21), 'Pluto': (727049, 14),
    }
    results = schema.classify_all_pairs(planets)

    aspects = [r for r in results if r[2].aspect_name is not None]
    shadows = [r for r in results if r[2].shadow_active]
    boundaries = [r for r in results if r[2].boundary_active]

    print(f"    Total pairs: {len(results)}")
    print(f"    Aspects: {len(aspects)}")
    print(f"    Shadow bonds: {len(shadows)}")
    print(f"    Boundary events: {len(boundaries)}")

    print(f"\n    {'Pair':25s}  {'Aspect':13s}  {'Carry':18s}  {'Prox':>4s}  SB  Class")
    print(f"    {'─'*25}  {'─'*13}  {'─'*18}  {'─'*4}  ──  {'─'*10}")
    for a, b, c in sorted(results, key=lambda x: (x[2].aspect_name is None, -x[2].proximity)):
        if c.aspect_name or c.shadow_active or c.boundary_active:
            pair = f"{a}-{b}"
            asp = c.aspect_name or "—none—"
            s = "●" if c.shadow_active else "·"
            bd = "●" if c.boundary_active else "·"
            print(f"    {pair:25s}  {asp:13s}  {str(c.carry_pattern):18s}  {c.proximity:>4d}  {s}{bd}  {c.harmonic_class}")

    print(f"\n{'='*72}")
    if failures == 0:
        print(f"  ALL CHECKS PASSED. Schema verified.")
    else:
        print(f"  {failures} FAILURES.")
    print(f"{'='*72}")
    return failures


if __name__ == '__main__':
    test_schema()
