#!/usr/bin/env python3
"""
T2.1: Commutativity Projector — Slot Context Port
====================================================
Adapted from qmnf_ws/crates/cram_explorer/src/projector.rs

Given an observed spin sequence from an unknown generator,
check per-lane commutativity: does each lane satisfy a linear
recurrence x_{n+1} ≡ a_m * x_n + c_m (mod m)?

LCG-class (tractable) generators should have commutative (fully linear) lanes.
Non-linear generators should show lane-level commutativity failures.

Output: detailed per-lane verdict + failure diagnostics for structure detection.
"""

from __future__ import annotations
import json
from dataclasses import dataclass, asdict
from typing import Optional, Tuple, List
from math import gcd
from functools import reduce

# ═══════════════════════════════════════════════════════════════
# CRAM Basis (from harness)
# ═══════════════════════════════════════════════════════════════

SAFE_BASIS   = [3, 5, 7, 11, 13]
SAFE_M       = reduce(lambda a, b: a * b, SAFE_BASIS)  # 15,015


def mod_inverse(a: int, m: int) -> Optional[int]:
    """Extended GCD: return a^-1 mod m, or None if gcd(a,m) ≠ 1."""
    if gcd(a % m, m) != 1:
        return None
    old_r, r = a % m, m
    old_s, s = 1, 0
    while r:
        q = old_r // r
        old_r, r = r, old_r - q * r
        old_s, s = s, old_s - q * s
    return old_s % m


# ═══════════════════════════════════════════════════════════════
# LANE LINEARITY CHECKER
# ═══════════════════════════════════════════════════════════════

@dataclass
class LaneLinearityResult:
    """Per-lane commutativity verdict."""
    modulus: int
    lane_number: int
    is_linear: bool
    a_recovered: Optional[int]
    c_recovered: Optional[int]
    consistency_score: float  # 0.0 to 1.0: fraction of transitions matching linear model
    first_failure_idx: Optional[int]  # Index of first inconsistency, or None if all pass
    notes: str = ""

    def to_dict(self):
        return asdict(self)


def check_lane_linearity(
    lane_values: List[int],
    modulus: int,
    lane_number: int,
    min_calibration_window: int = 3
) -> LaneLinearityResult:
    """
    Check whether a lane satisfies linear recurrence:
      x_{n+1} ≡ a * x_n + c (mod m)

    Steps:
    1. Attempt to recover (a, c) from first calibration_window values
    2. Verify recovered (a, c) against remaining lane values
    3. Return consistency score and first failure point
    """
    
    if len(lane_values) < min_calibration_window:
        return LaneLinearityResult(
            modulus=modulus,
            lane_number=lane_number,
            is_linear=False,
            a_recovered=None,
            c_recovered=None,
            consistency_score=0.0,
            first_failure_idx=None,
            notes=f"Insufficient data: {len(lane_values)} < {min_calibration_window}"
        )

    # ── Step 1: Calibrate (a, c) from first window ──
    a_m = None
    c_m = None
    
    # Try to solve via diff-pair method
    for k in range(len(lane_values) - 2):
        r1 = lane_values[k] % modulus
        r2 = lane_values[k+1] % modulus
        r3 = lane_values[k+2] % modulus
        
        diff_r = (r2 - r1) % modulus
        diff_next = (r3 - r2) % modulus
        
        if gcd(diff_r, modulus) == 1:
            inv = mod_inverse(diff_r, modulus)
            a_m = (diff_next * inv) % modulus
            c_m = (r2 - a_m * r1) % modulus
            break
    
    # ── Degenerate lane: all values constant ──
    if a_m is None:
        residues = [v % modulus for v in lane_values]
        if len(set(residues)) == 1:
            # Constant lane: any (a, c) with a*v + c ≡ v (mod m) works
            # Simplest: a=1, c=0
            a_m = 1
            c_m = 0
        else:
            # Non-constant but no linear pair found
            return LaneLinearityResult(
                modulus=modulus,
                lane_number=lane_number,
                is_linear=False,
                a_recovered=None,
                c_recovered=None,
                consistency_score=0.0,
                first_failure_idx=0,
                notes="No invertible diff-pair found (composite modulus edge case)"
            )
    
    # ── Step 2: Verify (a_m, c_m) against all transitions ──
    matches = 0
    first_failure = None
    
    for i in range(len(lane_values) - 1):
        x_i = lane_values[i] % modulus
        x_next = lane_values[i+1] % modulus
        predicted = (a_m * x_i + c_m) % modulus
        
        if predicted == x_next:
            matches += 1
        elif first_failure is None:
            first_failure = i
    
    consistency = matches / (len(lane_values) - 1) if len(lane_values) > 1 else 0.0
    is_linear = (consistency == 1.0)
    
    return LaneLinearityResult(
        modulus=modulus,
        lane_number=lane_number,
        is_linear=is_linear,
        a_recovered=a_m,
        c_recovered=c_m,
        consistency_score=consistency,
        first_failure_idx=first_failure,
        notes=""
    )


# ═══════════════════════════════════════════════════════════════
# FULL PROJECTOR: Multi-Lane Consistency Check
# ═══════════════════════════════════════════════════════════════

@dataclass
class ProjectorResult:
    """Full multi-lane commutativity verdict."""
    generator_name: str
    basis: List[int]
    M: int
    total_observations: int
    calibration_window: int
    
    # Per-lane results
    lane_results: List[LaneLinearityResult]
    
    # Summary
    fully_commutative: bool  # All lanes linear
    commutative_lanes: int   # Count of linear lanes
    total_lanes: int
    commutativity_score: float  # Fraction of commutative lanes
    
    def to_dict(self):
        return {
            'generator_name': self.generator_name,
            'basis': self.basis,
            'M': self.M,
            'total_observations': self.total_observations,
            'calibration_window': self.calibration_window,
            'lane_results': [r.to_dict() for r in self.lane_results],
            'fully_commutative': self.fully_commutative,
            'commutative_lanes': self.commutative_lanes,
            'total_lanes': self.total_lanes,
            'commutativity_score': self.commutativity_score,
        }


def check_commutativity(
    spins: List[int],
    generator_name: str,
    basis: Optional[List[int]] = None,
    calibration_window: int = 20
) -> ProjectorResult:
    """
    Multi-lane commutativity check.

    Args:
        spins: Observed output sequence
        generator_name: Name for reporting
        basis: Lane moduli (default: SAFE_BASIS)
        calibration_window: Min observations per lane for calibration

    Returns:
        ProjectorResult with per-lane verdicts
    """
    if basis is None:
        basis = SAFE_BASIS
    
    M = reduce(lambda a, b: a * b, basis)
    
    # Extract per-lane values
    lane_results = []
    for lane_num, modulus in enumerate(basis):
        lane_values = [s % modulus for s in spins]
        result = check_lane_linearity(
            lane_values, 
            modulus, 
            lane_num,
            min_calibration_window=calibration_window
        )
        lane_results.append(result)
    
    # Summary
    commutative_lanes = sum(1 for r in lane_results if r.is_linear)
    total_lanes = len(basis)
    commutativity_score = commutative_lanes / total_lanes if total_lanes > 0 else 0.0
    fully_commutative = (commutative_lanes == total_lanes)
    
    return ProjectorResult(
        generator_name=generator_name,
        basis=basis,
        M=M,
        total_observations=len(spins),
        calibration_window=calibration_window,
        lane_results=lane_results,
        fully_commutative=fully_commutative,
        commutative_lanes=commutative_lanes,
        total_lanes=total_lanes,
        commutativity_score=commutativity_score,
    )


# ═══════════════════════════════════════════════════════════════
# REPORTING
# ═══════════════════════════════════════════════════════════════

def format_lane_report(lane_result: LaneLinearityResult) -> str:
    """Format a single lane result for human reading."""
    status = "✓ LINEAR" if lane_result.is_linear else "✗ NON-LINEAR"
    return (
        f"  Lane {lane_result.lane_number} (mod {lane_result.modulus}): {status}\n"
        f"    Recovered: a={lane_result.a_recovered}, c={lane_result.c_recovered}\n"
        f"    Consistency: {lane_result.consistency_score*100:.1f}%\n"
        f"    First failure: {lane_result.first_failure_idx if lane_result.first_failure_idx is not None else 'None'}\n"
        f"    Notes: {lane_result.notes if lane_result.notes else '(none)'}"
    )


def format_projector_report(result: ProjectorResult) -> str:
    """Format full projector result for human reading."""
    lines = [
        f"{'='*70}",
        f"PROJECTOR REPORT: {result.generator_name}",
        f"{'='*70}",
        f"Observations: {result.total_observations}",
        f"Basis: {result.basis}",
        f"M: {result.M}",
        f"",
        f"COMMUTATIVITY: {result.commutativity_score*100:.1f}% ({result.commutative_lanes}/{result.total_lanes} lanes)",
        f"Verdict: {'FULLY COMMUTATIVE (LCG-class)' if result.fully_commutative else 'NON-COMMUTATIVE (non-linear)'}",
        f"",
        f"LANE-BY-LANE BREAKDOWN:",
    ]
    
    for lane_result in result.lane_results:
        lines.append(format_lane_report(lane_result))
    
    lines.append(f"{'='*70}")
    return "\n".join(lines)


if __name__ == "__main__":
    # Unit test: simple LCG
    print("Testing projector with synthetic LCG...")
    
    a, c, m = 5, 3, 15015  # LCG parameters
    x = 12345
    spins = []
    for _ in range(100):
        x = (a * x + c) % m
        spins.append(x)
    
    result = check_commutativity(spins, "Synthetic LCG (a=5, c=3)")
    print(format_projector_report(result))
    print()
    print("JSON dump:")
    print(json.dumps(result.to_dict(), indent=2))
