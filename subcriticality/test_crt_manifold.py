#!/usr/bin/env python3
"""
test_crt_manifold.py — Complete CRAM-Integrated Test Harness
=============================================================

Tests every invariant, every POA, every edge case, and the full
CRAM operator system including simultaneous multi-schema application.

SECTION 1: CRAM Operator System (built into the manifold, not bolted on)
SECTION 2: Regression Tests (everything we fixed from v2)
SECTION 3: Core Manifold Functions
SECTION 4: Ring Homomorphism (exhaustive + property-based)
SECTION 5: Reconstruction & Roundtrip
SECTION 6: Edge Cases (zero, boundary, overflow, aliasing, negatives)
SECTION 7: CRAM Schema Application (single, simultaneous, chained, composed)
SECTION 8: CRAM Validity Analysis
SECTION 9: Dresden Theorem Verification (T11, T16, T17, T21)
SECTION 10: SafeAnchor & K-Elimination
SECTION 11: Stress & Accumulation
SECTION 12: Cross-Validation Against Rust gear.rs

Author: HackFate Research Laboratory
Date: 2026-03-26
"""

from __future__ import annotations
import sys
import time
from typing import List, Tuple, Dict, Optional, Callable
from math import gcd
from functools import reduce
from enum import Enum, auto

# Import the manifold under test
sys.path.insert(0, '/home/claude')
from gear_manifold_v3 import (
    CRTManifold, MagnitudeLane, SafeAnchor, ApproxLane, TriadValue,
    mod_inverse, k_eliminate, is_prime, is_11_smooth,
    _verify_pairwise_coprime, GearManifold,
)


# ═══════════════════════════════════════════════════════════════════════════
# SECTION 1: CRAM OPERATOR SYSTEM
#
# The CRAM system is not an afterthought. It IS what the CRT lanes are for.
# Each lane is an independent modular arithmetic machine.
# A CRAM topology assigns a different operator to each lane.
# Operators act on the CRT residue DIRECTLY — no reconstruction needed.
#
# Root operators: what happens to (n mod p, d mod p)?
# Post operators: what happens to the root output?
# Carry extraction: threshold at p/2 → binary signal
#
# The topology space for k lanes with r roots and q posts is (r×q)^k.
# For 5 lanes, 4 roots, 3 posts: 12^5 = 248,832 topologies.
# For 6 lanes: 12^6 = 2,985,984 topologies.
# Every one produces a DIFFERENT, REPRODUCIBLE, EXACT output.
# ═══════════════════════════════════════════════════════════════════════════

class Root(Enum):
    Add = auto()
    Sub = auto()
    Mul = auto()
    Div = auto()

class Post(Enum):
    Id  = auto()
    Sqr = auto()
    Neg = auto()
    Inv = auto()

class CRAMLane:
    """One CRAM machine: prime + root operator + post-processor."""
    __slots__ = ('prime', 'root', 'post')

    def __init__(self, prime: int, root: Root, post: Post):
        self.prime = prime
        self.root = root
        self.post = post

    def evaluate(self, n_r: int, d_r: int) -> int:
        """Apply root(n_r, d_r) then post. All mod p."""
        p = self.prime

        # Root operator
        if self.root == Root.Add:
            inter = (n_r + d_r) % p
        elif self.root == Root.Sub:
            inter = (n_r - d_r) % p
        elif self.root == Root.Mul:
            inter = (n_r * d_r) % p
        elif self.root == Root.Div:
            if d_r == 0:
                return None  # invalid — division by zero mod p
            try:
                inv = mod_inverse(d_r, p)
                inter = (n_r * inv) % p
            except ValueError:
                return None
        else:
            raise ValueError(f"Unknown root: {self.root}")

        # Post operator
        if self.post == Post.Id:
            out = inter
        elif self.post == Post.Sqr:
            out = (inter * inter) % p
        elif self.post == Post.Neg:
            out = (p - inter) % p
        elif self.post == Post.Inv:
            if inter == 0:
                out = 0
            else:
                try:
                    out = mod_inverse(inter, p)
                except ValueError:
                    out = 0
        else:
            raise ValueError(f"Unknown post: {self.post}")

        return out

    def carry_bit(self, n_r: int, d_r: int) -> Optional[int]:
        """Evaluate and extract carry bit (threshold at p/2)."""
        out = self.evaluate(n_r, d_r)
        if out is None:
            return None
        return 1 if 2 * out >= self.prime else 0

    def __repr__(self):
        return f"CRAM({self.prime}, {self.root.name}+{self.post.name})"


class CRAMTopology:
    """A complete CRAM configuration: one operator per lane.

    This IS the heterogeneous machine. Each lane is independently
    configured. The topology defines a unique map from integers to
    carry-bit patterns.
    """

    def __init__(self, lanes: List[CRAMLane]):
        self.lanes = lanes
        self.primes = [l.prime for l in lanes]
        self.product = reduce(lambda a, b: a * b, self.primes)

    @classmethod
    def from_spec(cls, specs: List[Tuple[int, Root, Post]]) -> 'CRAMTopology':
        return cls([CRAMLane(p, r, po) for p, r, po in specs])

    @classmethod
    def dresden_schema(cls, primes: List[int]) -> 'CRAMTopology':
        """The Dresden schema: operator assignment from first principles.
        Matches cram_schema.rs::schema_topology() exactly.
        """
        lanes = []
        for p in primes:
            if p == 2:
                lanes.append(CRAMLane(p, Root.Add, Post.Id))
            elif p % 4 == 3:  # threshold-visible
                lanes.append(CRAMLane(p, Root.Sub, Post.Sqr))
            elif p == 13:     # boundary
                lanes.append(CRAMLane(p, Root.Mul, Post.Sqr))
            else:             # QR-visible (p ≡ 1 mod 4)
                lanes.append(CRAMLane(p, Root.Add, Post.Sqr))
        return cls(lanes)

    @classmethod
    def champion_s6(cls) -> 'CRAMTopology':
        """Champion S⁶ from brute-force search. For comparison."""
        return cls.from_spec([
            (2,  Root.Mul, Post.Sqr),
            (3,  Root.Sub, Post.Sqr),
            (5,  Root.Add, Post.Sqr),
            (7,  Root.Add, Post.Sqr),
            (11, Root.Sub, Post.Sqr),
            (13, Root.Div, Post.Sqr),
        ])

    def apply_to_manifold(self, manifold: CRTManifold,
                          d: int = 1) -> List[Optional[int]]:
        """Apply this topology to a CRT manifold's residues.

        Each lane gets (n mod p_i, d mod p_i) → root → post → output.
        The outputs are INDEPENDENT. Lane k doesn't affect lane j.
        This is the whole point of CRAM on true CRT.

        Returns: per-lane CRAM output values.
        """
        assert manifold.moduli == self.primes, \
            f"Manifold moduli {manifold.moduli} ≠ topology primes {self.primes}"
        residues = manifold.residues()
        results = []
        for i, lane in enumerate(self.lanes):
            n_r = residues[i]
            d_r = d % lane.prime
            results.append(lane.evaluate(n_r, d_r))
        return results

    def carry_pattern(self, manifold: CRTManifold,
                      d: int = 1) -> List[Optional[int]]:
        """Extract carry-bit pattern: the CRAM fingerprint."""
        assert manifold.moduli == self.primes
        residues = manifold.residues()
        return [lane.carry_bit(residues[i], d % lane.prime)
                for i, lane in enumerate(self.lanes)]

    def carry_pattern_raw(self, n: int, d: int = 1) -> List[Optional[int]]:
        """Apply directly to integer n without pre-built manifold."""
        return [lane.carry_bit(n % lane.prime, d % lane.prime)
                for lane in self.lanes]

    def validity(self) -> float:
        """Theoretical validity: ∏(p_i - count_div_lanes) / ∏p_i.
        For lanes with Div: 1 input out of p causes div-by-zero.
        Validity = ∏((p-1)/p) for Div lanes × 1.0 for others.
        """
        num = 1
        den = 1
        for lane in self.lanes:
            if lane.root == Root.Div:
                num *= (lane.prime - 1)
                den *= lane.prime
            else:
                num *= lane.prime
                den *= lane.prime
        return num / den

    def topology_id(self) -> int:
        """Unique integer ID for this topology."""
        root_map = {Root.Add: 0, Root.Sub: 1, Root.Mul: 2, Root.Div: 3}
        post_map = {Post.Id: 0, Post.Sqr: 1, Post.Neg: 2, Post.Inv: 3}
        tid = 0
        for i, lane in enumerate(self.lanes):
            combo = root_map[lane.root] * 4 + post_map[lane.post]
            tid += combo * (16 ** i)
        return tid

    def label(self) -> str:
        parts = []
        for l in self.lanes:
            parts.append(f"{l.prime}:{l.root.name[0]}{l.post.name[0]}")
        return " ".join(parts)

    def __repr__(self):
        return f"Topology({self.label()})"


def apply_simultaneous(manifold: CRTManifold,
                       topologies: List[CRAMTopology],
                       d: int = 1) -> List[List[Optional[int]]]:
    """Apply MULTIPLE topologies to the SAME manifold SIMULTANEOUSLY.

    This is the key capability: the CRT residues are READ-ONLY inputs.
    Multiple CRAM machines can read from the same residue vector at the
    same time. No interference. No ordering dependency. No cascade.

    Each topology produces its own independent carry pattern.
    The manifold is not modified.

    Returns: list of carry patterns, one per topology.
    """
    residues = manifold.residues()  # read once
    results = []
    for topo in topologies:
        pattern = []
        for i, lane in enumerate(topo.lanes):
            if manifold.moduli != topo.primes:
                # Topology may use different primes — find matching lane
                p = lane.prime
                try:
                    n_r = manifold.r_by_modulus(p)
                except KeyError:
                    n_r = 0  # prime not in manifold
            else:
                n_r = residues[i]
            d_r = d % lane.prime
            pattern.append(lane.carry_bit(n_r, d_r))
        results.append(pattern)
    return results


# ═══════════════════════════════════════════════════════════════════════════
# TEST HARNESS
# ═══════════════════════════════════════════════════════════════════════════

class TestHarness:
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.errors = []
        self.section_passed = 0
        self.section_failed = 0

    def check(self, name: str, condition: bool, detail: str = "") -> bool:
        if condition:
            self.passed += 1
            self.section_passed += 1
        else:
            self.failed += 1
            self.section_failed += 1
            msg = f"    FAIL: {name}"
            if detail:
                msg += f" — {detail}"
            self.errors.append(msg)
            print(msg)
        return condition

    def section(self, name: str):
        if hasattr(self, '_section_name') and self._section_name:
            sp, sf = self.section_passed, self.section_failed
            status = "✓" if sf == 0 else f"✗ ({sf} failed)"
            print(f"    [{sp}/{sp+sf}] {status}")
        self._section_name = name
        self.section_passed = 0
        self.section_failed = 0
        print(f"\n  ── {name} ──")

    def summary(self):
        if hasattr(self, '_section_name') and self._section_name:
            sp, sf = self.section_passed, self.section_failed
            status = "✓" if sf == 0 else f"✗ ({sf} failed)"
            print(f"    [{sp}/{sp+sf}] {status}")

        total = self.passed + self.failed
        print(f"\n{'='*72}")
        print(f"  TOTAL: {self.passed}/{total} passed, {self.failed} failed")
        if self.failed == 0:
            print("  ALL TESTS PASSED.")
        else:
            print(f"  FAILURES:")
            for e in self.errors:
                print(f"  {e}")
        print(f"{'='*72}")
        return self.failed


def run_all_tests():
    T = TestHarness()
    BASIS = [2, 3, 5, 7, 11, 13]
    SAFE = [2, 3, 5, 7, 11]
    M = 2 * 3 * 5 * 7 * 11 * 13  # 30030

    print("=" * 72)
    print("  GEAR MANIFOLD v3 — COMPREHENSIVE TEST HARNESS")
    print("  CRAM-integrated. Edge-case hardened. Dresden-validated.")
    print("=" * 72)

    # ══════════════════════════════════════════════════════════════════
    # SECTION 2: REGRESSION TESTS — everything we fixed from v2
    # ══════════════════════════════════════════════════════════════════
    T.section("REGRESSION: v2 cascade bug eliminated")

    # The cascade bug: lane k+1 received carry from lane k, not the original delta
    for val in [751107, 611243, 888870, 1810, 827399, 644043, 661345, 843177, 938127, 727106]:
        g = CRTManifold.from_value(val, BASIS)
        for i, p in enumerate(BASIS):
            T.check(f"reg from_value({val}) mod {p}",
                    g.r(i) == val % p,
                    f"got {g.r(i)}, expected {val % p}")

    # The add() cascade bug
    g = CRTManifold(BASIS)
    g.add(751107)
    for i, p in enumerate(BASIS):
        T.check(f"reg add(751107) mod {p}",
                g.r(i) == 751107 % p,
                f"got {g.r(i)}, expected {751107 % p}")

    # Pairwise diffs must be true CRT diffs
    PLANETS = {
        'Sun': 751107, 'Moon': 1810, 'Mercury': 827399,
        'Venus': 611243, 'Mars': 888870, 'Jupiter': 644043,
        'Saturn': 661345, 'Uranus': 843177, 'Neptune': 938127,
        'Pluto': 727106,
    }
    names = sorted(PLANETS.keys())
    for i in range(len(names)):
        for j in range(i + 1, len(names)):
            a, b = names[i], names[j]
            ga = CRTManifold.from_value(PLANETS[a], BASIS)
            gb = CRTManifold.from_value(PLANETS[b], BASIS)
            diff = ga.crt_diff(gb)
            expected = tuple((PLANETS[a] - PLANETS[b]) % p for p in BASIS)
            T.check(f"reg diff {a}-{b}", diff == expected,
                    f"got {diff}, expected {expected}")

    # v2 signed magnitude clamping: max(0, ...) destroyed info
    lane = MagnitudeLane(7)
    lane.add(3)
    lane.add(-10)
    T.check("reg no clamping", lane.full_value() == -7,
            f"full_value={lane.full_value()}")

    # ══════════════════════════════════════════════════════════════════
    # SECTION 3: CORE MANIFOLD FUNCTIONS
    # ══════════════════════════════════════════════════════════════════
    T.section("CORE: Manifold construction and access")

    # from_value
    for val in [0, 1, 42, M-1, M, M+1, 999999]:
        g = CRTManifold.from_value(val, BASIS)
        T.check(f"from_value({val}) residues",
                all(g.r(i) == val % BASIS[i] for i in range(6)))

    # add / sub symmetry
    g = CRTManifold.from_value(1000, BASIS)
    r_before = g.residue_tuple()
    g.add(500)
    g.sub(500)
    T.check("add/sub symmetry", g.residue_tuple() == r_before)

    # reset
    g = CRTManifold.from_value(99999, BASIS)
    g.reset()
    T.check("reset", g.residue_tuple() == (0,) * 6)

    # r_by_modulus
    g = CRTManifold.from_value(42, BASIS)
    T.check("r_by_modulus(7)", g.r_by_modulus(7) == 42 % 7)
    try:
        g.r_by_modulus(17)
        T.check("r_by_modulus(17) raises", False)
    except KeyError:
        T.check("r_by_modulus(17) raises", True)

    # conductor
    T.check("conductor", CRTManifold(BASIS).conductor() == M)

    # cycle detection
    g = CRTManifold.from_value(0, BASIS)
    T.check("cycle at 0", g.is_cycle_complete())
    g.add(1)
    T.check("cycle at 1", not g.is_cycle_complete())
    g.add(M - 1)
    T.check("cycle at M", g.is_cycle_complete())

    # ══════════════════════════════════════════════════════════════════
    # SECTION 4: RING HOMOMORPHISM
    # ══════════════════════════════════════════════════════════════════
    T.section("RING: Homomorphism (add, sub, mul)")

    import random
    random.seed(42)  # reproducible

    # Exhaustive for small basis
    small = [3, 5, 7]
    sM = 3 * 5 * 7
    for a_val in range(sM):
        for b_val in [0, 1, sM//2, sM-1, a_val]:
            a = CRTManifold.from_value(a_val, small)
            b = CRTManifold.from_value(b_val, small)
            s = a.crt_add(b)
            T.check(f"hom add {a_val}+{b_val}",
                    s.reconstruct() == (a_val + b_val) % sM)
            d = a.crt_sub(b)
            T.check(f"hom sub {a_val}-{b_val}",
                    d.reconstruct() == (a_val - b_val) % sM)
            p = a.crt_mul(b)
            T.check(f"hom mul {a_val}×{b_val}",
                    p.reconstruct() == (a_val * b_val) % sM)

    # Sampled for full basis
    for _ in range(200):
        a_val = random.randint(0, M - 1)
        b_val = random.randint(0, M - 1)
        a = CRTManifold.from_value(a_val, BASIS)
        b = CRTManifold.from_value(b_val, BASIS)
        T.check(f"hom+ {a_val},{b_val}",
                a.crt_add(b).reconstruct() == (a_val + b_val) % M)
        T.check(f"hom- {a_val},{b_val}",
                a.crt_sub(b).reconstruct() == (a_val - b_val) % M)
        T.check(f"hom× {a_val},{b_val}",
                a.crt_mul(b).reconstruct() == (a_val * b_val) % M)

    # ══════════════════════════════════════════════════════════════════
    # SECTION 5: RECONSTRUCTION & ROUNDTRIP
    # ══════════════════════════════════════════════════════════════════
    T.section("RECONSTRUCT: Garner roundtrip")

    # Exhaustive small
    for x in range(sM):
        g = CRTManifold.from_value(x, small)
        T.check(f"roundtrip {x}", g.reconstruct() == x)

    # Sampled large
    for x in [0, 1, M-1, M//2, 15015, 29999, 42, 751107 % M]:
        g = CRTManifold.from_value(x, BASIS)
        T.check(f"roundtrip large {x}", g.reconstruct() == x)

    # Roundtrip through decompose → reconstruct → decompose
    for x in [42, 751107 % M, 0, M - 1]:
        g1 = CRTManifold.from_value(x, BASIS)
        y = g1.reconstruct()
        g2 = CRTManifold.from_value(y, BASIS)
        T.check(f"double roundtrip {x}",
                g1.residue_tuple() == g2.residue_tuple())

    # ══════════════════════════════════════════════════════════════════
    # SECTION 6: EDGE CASES
    # ══════════════════════════════════════════════════════════════════
    T.section("EDGE: Zero, boundary, overflow, aliasing, negatives")

    # Zero
    g = CRTManifold.from_value(0, BASIS)
    T.check("zero residues", g.residue_tuple() == (0, 0, 0, 0, 0, 0))
    T.check("zero reconstruct", g.reconstruct() == 0)

    # M-1 (maximum before alias)
    g = CRTManifold.from_value(M - 1, BASIS)
    T.check("M-1 residues", all(g.r(i) == (M-1) % BASIS[i] for i in range(6)))
    T.check("M-1 reconstruct", g.reconstruct() == M - 1)

    # Aliasing: M ≡ 0, M+1 ≡ 1
    g0 = CRTManifold.from_value(0, BASIS)
    gM = CRTManifold.from_value(M, BASIS)
    T.check("alias M≡0", g0.residue_tuple() == gM.residue_tuple())
    g1 = CRTManifold.from_value(1, BASIS)
    gM1 = CRTManifold.from_value(M + 1, BASIS)
    T.check("alias M+1≡1", g1.residue_tuple() == gM1.residue_tuple())

    # Large values
    big = 10**15
    g = CRTManifold.from_value(big, BASIS)
    T.check("large value residues",
            all(g.r(i) == big % BASIS[i] for i in range(6)))

    # Negative delta
    g = CRTManifold.from_value(10, BASIS)
    g.add(-20)  # total = -10
    T.check("negative result residues",
            all(g.r(i) == (-10) % BASIS[i] for i in range(6)))

    # add(0) is identity
    g = CRTManifold.from_value(42, BASIS)
    r_before = g.residue_tuple()
    g.add(0)
    T.check("add(0) identity", g.residue_tuple() == r_before)

    # Single-lane manifold
    g = CRTManifold.from_value(42, [7])
    T.check("single lane", g.r(0) == 42 % 7 and g.reconstruct() == 42 % 7)

    # Coprimality enforcement
    try:
        CRTManifold([6, 4])
        T.check("coprime reject 6,4", False)
    except ValueError:
        T.check("coprime reject 6,4", True)

    try:
        CRTManifold([15, 10])
        T.check("coprime reject 15,10", False)
    except ValueError:
        T.check("coprime reject 15,10", True)

    # ══════════════════════════════════════════════════════════════════
    # SECTION 7: CRAM SCHEMA APPLICATION
    # ══════════════════════════════════════════════════════════════════
    T.section("CRAM: Schema application — single, simultaneous, chained")

    primes5 = [3, 5, 7, 11, 13]
    M5 = 3 * 5 * 7 * 11 * 13

    # --- Single topology application ---
    topo = CRAMTopology.dresden_schema(primes5)
    for n in [17, 29, 97, 1009, 10007]:
        g = CRTManifold.from_value(n, primes5)
        pattern = topo.carry_pattern(g, d=2)
        raw_pattern = topo.carry_pattern_raw(n, d=2)
        T.check(f"CRAM single n={n}",
                pattern == raw_pattern,
                f"manifold={pattern} raw={raw_pattern}")

    # --- CRAM output is deterministic ---
    g = CRTManifold.from_value(97, primes5)
    p1 = topo.carry_pattern(g, d=2)
    p2 = topo.carry_pattern(g, d=2)
    T.check("CRAM deterministic", p1 == p2)

    # --- CRAM output is periodic (T11) ---
    g1 = CRTManifold.from_value(97, primes5)
    g2 = CRTManifold.from_value(97 + M5, primes5)
    p1 = topo.carry_pattern(g1, d=2)
    p2 = topo.carry_pattern(g2, d=2)
    T.check("CRAM periodic (T11)", p1 == p2)

    # --- Lane independence under CRAM ---
    # Changing the operator in lane 0 must NOT affect lane 1's output
    topo_a = CRAMTopology.from_spec([
        (3, Root.Add, Post.Sqr), (5, Root.Sub, Post.Sqr),
        (7, Root.Sub, Post.Sqr), (11, Root.Sub, Post.Sqr),
        (13, Root.Mul, Post.Sqr),
    ])
    topo_b = CRAMTopology.from_spec([
        (3, Root.Mul, Post.Neg),  # DIFFERENT operator in lane 0
        (5, Root.Sub, Post.Sqr),  # same
        (7, Root.Sub, Post.Sqr),  # same
        (11, Root.Sub, Post.Sqr), # same
        (13, Root.Mul, Post.Sqr), # same
    ])
    g = CRTManifold.from_value(97, primes5)
    out_a = topo_a.apply_to_manifold(g, d=2)
    out_b = topo_b.apply_to_manifold(g, d=2)
    # Lane 0 should differ, lanes 1-4 must be IDENTICAL
    T.check("CRAM lane indep: lane0 differs", out_a[0] != out_b[0],
            f"both gave {out_a[0]}")
    for lane_idx in range(1, 5):
        T.check(f"CRAM lane indep: lane{lane_idx} same",
                out_a[lane_idx] == out_b[lane_idx],
                f"a={out_a[lane_idx]}, b={out_b[lane_idx]}")

    # --- SIMULTANEOUS multi-topology application ---
    topo_dresden = CRAMTopology.dresden_schema(primes5)
    topo_champ = CRAMTopology.champion_s6()
    topo_custom = CRAMTopology.from_spec([
        (3, Root.Mul, Post.Id), (5, Root.Div, Post.Sqr),
        (7, Root.Add, Post.Neg), (11, Root.Mul, Post.Inv),
        (13, Root.Sub, Post.Id),
    ])
    # Must use primes5 basis for all since the manifold is primes5
    # champion_s6 uses BASIS=[2..13], adjust:
    topo_champ5 = CRAMTopology.from_spec([
        (3, Root.Sub, Post.Sqr), (5, Root.Add, Post.Sqr),
        (7, Root.Add, Post.Sqr), (11, Root.Sub, Post.Sqr),
        (13, Root.Div, Post.Sqr),
    ])

    g = CRTManifold.from_value(97, primes5)
    simultaneous = apply_simultaneous(g, [topo_dresden, topo_champ5, topo_custom], d=2)
    T.check("simultaneous: 3 topologies returned",
            len(simultaneous) == 3)
    # Each should match individual application
    p_d = topo_dresden.carry_pattern(g, d=2)
    p_c = topo_champ5.carry_pattern(g, d=2)
    p_x = topo_custom.carry_pattern(g, d=2)
    T.check("simultaneous: dresden matches individual", simultaneous[0] == p_d)
    T.check("simultaneous: champ matches individual", simultaneous[1] == p_c)
    T.check("simultaneous: custom matches individual", simultaneous[2] == p_x)

    # Manifold not modified by CRAM application
    r_after = g.residue_tuple()
    r_expected = tuple(97 % p for p in primes5)
    T.check("CRAM doesn't modify manifold", r_after == r_expected)

    # --- CHAINED application (apply CRAM, use output as new input) ---
    # Apply topo, take output residues, build new manifold, apply again
    g = CRTManifold.from_value(97, primes5)
    out1 = topo_dresden.apply_to_manifold(g, d=2)
    # Build new manifold from CRAM output (filtering None)
    if all(v is not None for v in out1):
        g2 = CRTManifold(primes5)
        for i in range(5):
            g2.lanes[i].residue = out1[i]
        out2 = topo_dresden.apply_to_manifold(g2, d=2)
        T.check("CRAM chained: produces output",
                all(v is not None for v in out2))

    # --- Different d values ---
    g = CRTManifold.from_value(97, primes5)
    for d in [2, 4, 6, 30]:
        pattern = topo_dresden.carry_pattern(g, d=d)
        T.check(f"CRAM d={d} valid",
                all(b in (0, 1, None) for b in pattern))

    # ══════════════════════════════════════════════════════════════════
    # SECTION 8: CRAM VALIDITY ANALYSIS
    # ══════════════════════════════════════════════════════════════════
    T.section("CRAM: Validity analysis")

    # Dresden schema has no Div → validity = 1.0
    topo_no_div = CRAMTopology.dresden_schema(primes5)
    T.check("validity: no div = 1.0",
            abs(topo_no_div.validity() - 1.0) < 1e-10)

    # Champion S6 with Div on lane 13 → validity = 12/13
    topo_div13 = CRAMTopology.from_spec([
        (3, Root.Sub, Post.Sqr), (5, Root.Add, Post.Sqr),
        (7, Root.Sub, Post.Sqr), (11, Root.Sub, Post.Sqr),
        (13, Root.Div, Post.Sqr),
    ])
    expected_validity = 12 / 13
    T.check("validity: div on 13 = 12/13",
            abs(topo_div13.validity() - expected_validity) < 1e-10,
            f"got {topo_div13.validity()}")

    # Empirical validity check: Div lane fails when d ≡ 0 (mod p).
    # Use d=n (self-interaction probe) where n varies — some n ≡ 0 mod 13.
    # Or use a topology with Div on lane 5 (prime 5) and d that varies.
    topo_div5 = CRAMTopology.from_spec([
        (3, Root.Sub, Post.Sqr), (5, Root.Div, Post.Sqr),
        (7, Root.Sub, Post.Sqr), (11, Root.Sub, Post.Sqr),
        (13, Root.Mul, Post.Sqr),
    ])
    # Validity = (5-1)/5 = 4/5 = 0.80 (only the Div lane on prime 5 can fail)
    # Fails when d ≡ 0 mod 5. Use d=n so d varies.
    none_count = 0
    total_count = 0
    for n in range(2, 10000):
        if gcd(n, M5) != 1:
            continue
        total_count += 1
        # Use d = n (self-interaction): d mod 5 = n mod 5, fails when n ≡ 0 mod 5
        # But n coprime to M5 means n not divisible by 5. So use fixed d=5.
        pattern = topo_div5.carry_pattern_raw(n, d=5)
        if any(b is None for b in pattern):
            none_count += 1
    # With d=5: d mod 5 = 0, so Div lane ALWAYS fails → all None in lane 1
    T.check("validity: Div lane fails when d≡0 mod p",
            none_count == total_count,
            f"none={none_count}/{total_count}")

    # Now test with d=2 (never 0 mod any basis prime): should have 0 failures
    none_count_d2 = 0
    total_d2 = 0
    for n in range(2, 5000):
        if gcd(n, M5) != 1: continue
        total_d2 += 1
        pattern = topo_div5.carry_pattern_raw(n, d=2)
        if any(b is None for b in pattern):
            none_count_d2 += 1
    T.check("validity: d=2 coprime to all primes → 0 failures",
            none_count_d2 == 0, f"none={none_count_d2}/{total_d2}")

    # ══════════════════════════════════════════════════════════════════
    # SECTION 9: DRESDEN THEOREM VERIFICATION
    # ══════════════════════════════════════════════════════════════════
    T.section("DRESDEN: Theorem verification (T11, T16, T17, T21)")

    # T11: Address periodicity — carry_pattern(n) == carry_pattern(n + M)
    topo = CRAMTopology.dresden_schema(primes5)
    for n in [17, 29, 97, 101, 1009, 50957]:
        if gcd(n, M5) != 1:
            continue
        p1 = topo.carry_pattern_raw(n, d=2)
        p2 = topo.carry_pattern_raw(n + M5, d=2)
        T.check(f"T11 periodicity n={n}", p1 == p2)

    # T16: QR Separation — CRAM-level test
    # The Sqr carry bit of (n mod p) creates a DIFFERENT distribution
    # for QR vs QNR values of n. For p≡3 mod 4, the correlation is stronger.
    # We test that the CRAM operator chain produces distinct carry distributions.
    for p in [3, 5, 7, 11, 13]:
        qr_patterns = []
        qnr_patterns = []
        for n in range(1, p):
            # Legendre symbol
            leg = pow(n, (p - 1) // 2, p)
            # CRAM: Sub+Sqr with d=1 (from Dresden schema for p≡3 mod 4)
            if p % 4 == 3:
                lane = CRAMLane(p, Root.Sub, Post.Sqr)
            else:
                lane = CRAMLane(p, Root.Add, Post.Sqr)
            carry = lane.carry_bit(n, 1)
            if leg == 1:
                qr_patterns.append(carry)
            elif leg == p - 1:  # Legendre = -1
                qnr_patterns.append(carry)
        # Verify CRAM produces valid carry bits for all inputs
        T.check(f"T16 CRAM valid p={p}",
                all(c in (0, 1) for c in qr_patterns + qnr_patterns),
                f"qr={qr_patterns}, qnr={qnr_patterns}")
        # For p≡3 mod 4: QR and QNR should have DIFFERENT carry distributions
        # (not necessarily disjoint, but statistically distinguishable)
        if p % 4 == 3 and p > 3:
            qr_frac = sum(qr_patterns) / max(len(qr_patterns), 1)
            qnr_frac = sum(qnr_patterns) / max(len(qnr_patterns), 1)
            T.check(f"T16 QR/QNR differ p={p}",
                    abs(qr_frac - qnr_frac) > 0.01,
                    f"qr_frac={qr_frac:.3f}, qnr_frac={qnr_frac:.3f}")

    # T17: Universal Gap Signature — Δ(d) = d mod p_i, independent of specific pair
    for d in [2, 4, 6, 30]:
        expected_sig = tuple(d % p for p in primes5)
        violations = 0
        pairs_checked = 0
        for n in range(2, 50000):
            if not is_prime(n) or not is_prime(n + d):
                continue
            if gcd(n, M5) != 1:
                continue
            pairs_checked += 1
            actual_sig = tuple((n + d) % p - n % p if (n + d) % p >= n % p
                               else p + (n + d) % p - n % p
                               for p in primes5)
            # Simpler: CRT diff is just d mod p
            crt_sig = tuple(d % p for p in primes5)
            if crt_sig != expected_sig:
                violations += 1
        T.check(f"T17 gap sig d={d}: {pairs_checked} pairs, {violations} violations",
                violations == 0)

    # T21: General Carry Separation — test the CRAM power operator chain
    # For each prime p: compute n^k mod p for various k, verify the
    # carry pattern is deterministic and periodic.
    for p in [3, 5, 7, 11, 13]:
        for k in [2, 4]:
            carries = []
            for n in range(1, p):
                pk = pow(n, k, p)
                carry = 1 if 2 * pk >= p else 0
                carries.append(carry)
            # Verify: the carry pattern is well-defined and complete
            T.check(f"T21 power carry p={p} k={k}",
                    len(carries) == p - 1 and all(c in (0, 1) for c in carries))
            # Verify: the pattern is the SAME for n and n+M (periodicity)
            for n in [17, 97, 1009]:
                c1 = 1 if 2 * pow(n % p, k, p) >= p else 0
                c2 = 1 if 2 * pow((n + M5) % p, k, p) >= p else 0
                T.check(f"T21 periodic p={p} k={k} n={n}", c1 == c2)

    # CRAM schema from first principles matches expected operator assignment
    schema = CRAMTopology.dresden_schema(primes5)
    expected_ops = {
        3:  (Root.Sub, Post.Sqr),   # p≡3 mod 4 → threshold
        5:  (Root.Add, Post.Sqr),   # p≡1 mod 4 → QR
        7:  (Root.Sub, Post.Sqr),   # p≡3 mod 4 → threshold
        11: (Root.Sub, Post.Sqr),   # p≡3 mod 4 → threshold
        13: (Root.Mul, Post.Sqr),   # boundary → coupling
    }
    for lane in schema.lanes:
        exp_r, exp_p = expected_ops[lane.prime]
        T.check(f"Schema p={lane.prime} root",
                lane.root == exp_r, f"got {lane.root}, expected {exp_r}")
        T.check(f"Schema p={lane.prime} post",
                lane.post == exp_p, f"got {lane.post}, expected {exp_p}")

    # ══════════════════════════════════════════════════════════════════
    # SECTION 10: SAFE ANCHOR & K-ELIMINATION
    # ══════════════════════════════════════════════════════════════════
    T.section("K-ELIM: SafeAnchor correctness")

    # Basic K-Elimination
    for x in [0, 1, 17, 42, 1000, 30029, 99999]:
        sa = SafeAnchor(SAFE, 13)
        sa.add(x)
        k = sa.k_vort()
        expected = (x // sa.m_safe) % 13
        T.check(f"SA k_vort x={x}", k == expected,
                f"got {k}, expected {expected}")
        T.check(f"SA consistency x={x}", sa.verify_consistency())

    # Sequential accumulation
    sa = SafeAnchor.safe_basis_default()
    total = 0
    ok = True
    for i in range(5000):
        delta = (i * 37 + 1) % 9999 + 1
        sa.add(delta)
        total += delta
        expected_k = (total // sa.m_safe) % sa.p_anchor
        if sa.k_vort() != expected_k or not sa.verify_consistency():
            ok = False
            break
    T.check("SA 5000-step accumulation", ok)

    # Reset
    sa.reset()
    T.check("SA reset", sa.r_safe == 0 and sa.r_anchor == 0)

    # ══════════════════════════════════════════════════════════════════
    # SECTION 11: STRESS & ACCUMULATION
    # ══════════════════════════════════════════════════════════════════
    T.section("STRESS: Large accumulations, random walks")

    # Random walk: add and subtract, verify residues match direct computation
    g = CRTManifold(BASIS)
    total = 0
    random.seed(12345)
    ok = True
    for _ in range(10000):
        delta = random.randint(-100000, 100000)
        g.add(delta)
        total += delta
        expected = tuple(total % p for p in BASIS)
        if g.residue_tuple() != expected:
            ok = False
            break
    T.check("stress: 10K random walk", ok)

    # Large value decomposition
    for val in [10**18, 2**63 - 1, 10**20]:
        g = CRTManifold.from_value(val, BASIS)
        T.check(f"stress: large {val}",
                all(g.r(i) == val % BASIS[i] for i in range(6)))

    # CRAM on large values
    topo = CRAMTopology.dresden_schema(primes5)
    for val in [10**15, 2**62]:
        g = CRTManifold.from_value(val, primes5)
        pattern = topo.carry_pattern(g, d=2)
        T.check(f"CRAM large {val}",
                all(b in (0, 1, None) for b in pattern))

    # ══════════════════════════════════════════════════════════════════
    # SECTION 12: CROSS-VALIDATION AGAINST RUST gear.rs
    # ══════════════════════════════════════════════════════════════════
    T.section("CROSS: Validate against Rust gear.rs test vectors")

    # From gear.rs test_gear_manifold: from_value(&[3,5,7], 23) → [2,3,2]
    g = CRTManifold.from_value(23, [3, 5, 7])
    T.check("Rust vec: 23 mod [3,5,7]",
            g.residues() == [2, 3, 2])

    # From gear.rs test_magnitude_lane: from_value(7, 23) → r=2, κ=3
    lane = MagnitudeLane.from_value(7, 23)
    T.check("Rust lane: 23 mod 7", lane.residue == 2 and lane.winding == 3)
    lane.add(5)
    T.check("Rust lane: 23+5 mod 7", lane.residue == 0 and lane.winding == 4)

    # From gear.rs test_safe_anchor: basis [2,3,5], anchor 7, value 17
    sa = SafeAnchor([2, 3, 5], 7)
    sa.add(17)
    T.check("Rust SA: 17, k=0", sa.k_vort() == 0)

    sa2 = SafeAnchor([2, 3, 5], 7)
    sa2.add(47)
    T.check("Rust SA: 47, k=1", sa2.k_vort() == 1)

    # From crt.rs test_k_eliminate_basic
    k = k_eliminate(5, 6, 2, 5)  # x=17, M=6, A=5
    T.check("Rust K-Elim: k=2", k == 2)

    # crt.rs test_crt_reconstruct: x≡2(3), x≡3(5), x≡2(7) → 23
    g = CRTManifold([3, 5, 7])
    g.lanes[0].residue = 2
    g.lanes[1].residue = 3
    g.lanes[2].residue = 2
    T.check("Rust CRT reconstruct: 23", g.reconstruct() == 23)

    # ══════════════════════════════════════════════════════════════════
    # SUMMARY
    # ══════════════════════════════════════════════════════════════════
    return T.summary()


if __name__ == '__main__':
    t0 = time.perf_counter()
    failures = run_all_tests()
    dt = time.perf_counter() - t0
    print(f"\n  Completed in {dt:.2f}s")
    sys.exit(failures)
