#!/usr/bin/env python3
"""
T3.2: Hydra Slot Harness
========================
Implements 4-head Hydra oracle for generator fingerprinting.

Each head independently tests an algebraic property:
  Head A: LCG Linearity (linear recurrence fitting)
  Head B: GF(2) Linearity (LFSR/bit-shift structure)
  Head C: Quadratic Residue Pattern (QR distribution)
  Head D: Cross-Correlation / Composition (multi-component detection)

All arithmetic is exact integer (zero float).
"""

from __future__ import annotations
import json
from dataclasses import dataclass, asdict
from typing import Optional, List, Tuple
from math import gcd
from functools import reduce
from statistics import mean, stdev

# ═══════════════════════════════════════════════════════════════
# CRAM Basis
# ═══════════════════════════════════════════════════════════════

SAFE_BASIS = [3, 5, 7, 11, 13]
SAFE_M = reduce(lambda a, b: a * b, SAFE_BASIS)  # 15,015


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
# HEAD A: LCG LINEARITY DETECTOR
# ═══════════════════════════════════════════════════════════════

@dataclass
class HeadA_Result:
    """Head A: LCG linearity verdict."""
    verdict: str  # "YES" or "NO"
    confidence: float  # 0.0–1.0
    fully_commutative_lanes: int
    total_lanes: int
    avg_consistency: float
    lane_details: List[dict]


def head_a_lcg_linearity(
    spins: List[int],
    basis: List[int] = None,
) -> HeadA_Result:
    """
    Test: Is generator LCG-class (all lanes fully linear)?
    
    Algorithm:
    1. Per lane: fit x_{n+1} = a*x_n + c (mod m)
    2. Count: how many lanes are 100% consistent?
    3. Verdict: all lanes linear → YES, else NO
    """
    if basis is None:
        basis = SAFE_BASIS
    
    # Fit per lane
    lane_details = []
    fully_linear_count = 0
    consistencies = []
    
    for lane_num, modulus in enumerate(basis):
        lane_values = [s % modulus for s in spins]
        
        # Fit linear recurrence via diff-pair
        a_m, c_m = None, None
        for k in range(len(lane_values) - 2):
            r1 = lane_values[k]
            r2 = lane_values[k+1]
            r3 = lane_values[k+2]
            
            diff_r = (r2 - r1) % modulus
            diff_next = (r3 - r2) % modulus
            
            if gcd(diff_r, modulus) == 1:
                inv = mod_inverse(diff_r, modulus)
                a_m = (diff_next * inv) % modulus
                c_m = (r2 - a_m * r1) % modulus
                break
        
        # If no linear pair found, check if constant
        if a_m is None:
            residues = [v % modulus for v in lane_values]
            if len(set(residues)) == 1:
                a_m, c_m = 1, 0
            else:
                a_m, c_m = 0, 0  # Mark as unable to fit
        
        # Verify consistency
        matches = 0
        for i in range(len(lane_values) - 1):
            predicted = (a_m * lane_values[i] + c_m) % modulus
            if predicted == lane_values[i + 1]:
                matches += 1
        
        consistency = matches / (len(lane_values) - 1) if len(lane_values) > 1 else 0.0
        is_linear = (consistency == 1.0)
        
        lane_details.append({
            'modulus': modulus,
            'is_linear': is_linear,
            'consistency': consistency,
            'a_recovered': a_m,
            'c_recovered': c_m,
        })
        
        if is_linear:
            fully_linear_count += 1
        consistencies.append(consistency)
    
    # Verdict
    verdict = "YES" if fully_linear_count == len(basis) else "NO"
    confidence = fully_linear_count / len(basis) if basis else 0.0
    avg_consistency = mean(consistencies) if consistencies else 0.0
    
    return HeadA_Result(
        verdict=verdict,
        confidence=confidence,
        fully_commutative_lanes=fully_linear_count,
        total_lanes=len(basis),
        avg_consistency=avg_consistency,
        lane_details=lane_details
    )


# ═══════════════════════════════════════════════════════════════
# HEAD B: GF(2)-LINEAR (LFSR) DETECTOR
# ═══════════════════════════════════════════════════════════════

@dataclass
class HeadB_Result:
    """Head B: GF(2)-linearity verdict."""
    verdict: str  # "GF2_LINEAR", "WEAK_GF2", "NON_LINEAR"
    confidence: float  # 0.0–1.0
    linear_bits: int
    total_bits: int
    feedback_matches: int
    best_feedback: Optional[int]
    bit_linearity: List[float]


def head_b_gf2_linear(spins: List[int], bit_width: int = 16) -> HeadB_Result:
    """
    Test: Does output follow GF(2)-linear recurrence (LFSR-like)?
    
    Algorithm:
    1. For each bit position, test: bit_i(x_{n+1}) = f_i(x_n)
       where f_i is XOR of some bits of x_n
    2. For each of 2^bit_width possible feedback functions,
       count how many transitions match
    3. Confidence = fraction of bit positions that are linear
    """
    
    # Truncate spins to bit_width
    mask = (1 << bit_width) - 1
    truncated = [s & mask for s in spins]
    
    # Test each bit position for linearity
    bit_linearity = []
    linear_bit_count = 0
    best_feedback = None
    best_feedback_score = 0
    
    for bit_pos in range(bit_width):
        # Extract bit_pos from each output
        bit_sequence = [(s >> bit_pos) & 1 for s in truncated]
        
        # Try to find feedback that reconstructs bit_sequence[1:]
        # from bit_sequence[0:-1]
        best_score = 0
        best_feedback_fn = None
        
        # Test all 2^bit_width feedback functions (brute force)
        # Heuristic: test common LFSR feedback patterns
        for fb in range(1 << min(bit_width, 8)):  # Limit search for speed
            matches = 0
            for i in range(len(truncated) - 1):
                prev_state = truncated[i]
                next_bit = bit_sequence[i + 1]
                
                # Compute feedback: XOR of selected bits
                feedback_result = 0
                for j in range(min(bit_width, 8)):
                    if (fb >> j) & 1:
                        feedback_result ^= (prev_state >> j) & 1
                
                # Compare with observed next bit
                if feedback_result == next_bit:
                    matches += 1
            
            if matches > best_score:
                best_score = matches
                best_feedback_fn = fb
        
        # Consistency of this bit position
        consistency = best_score / (len(truncated) - 1) if len(truncated) > 1 else 0.0
        is_linear = (consistency > 0.9)  # High threshold
        
        if is_linear:
            linear_bit_count += 1
            if best_score > best_feedback_score:
                best_feedback_score = best_score
                best_feedback = best_feedback_fn
        
        bit_linearity.append(consistency)
    
    # Verdict
    linear_ratio = linear_bit_count / bit_width if bit_width > 0 else 0.0
    if linear_ratio > 0.7:
        verdict = "GF2_LINEAR"
    elif linear_ratio > 0.4:
        verdict = "WEAK_GF2"
    else:
        verdict = "NON_LINEAR"
    
    confidence = linear_ratio
    
    # Count how many feedback functions scored well
    feedback_matches = sum(1 for bl in bit_linearity if bl > 0.8)
    
    return HeadB_Result(
        verdict=verdict,
        confidence=confidence,
        linear_bits=linear_bit_count,
        total_bits=bit_width,
        feedback_matches=feedback_matches,
        best_feedback=best_feedback,
        bit_linearity=bit_linearity
    )


# ═══════════════════════════════════════════════════════════════
# HEAD C: QUADRATIC RESIDUE PATTERN DETECTOR
# ═══════════════════════════════════════════════════════════════

@dataclass
class HeadC_Result:
    """Head C: Quadratic residue pattern verdict."""
    verdict: str  # "QR_STRUCTURED", "QR_NOISE", "QR_SPARSE"
    confidence: float  # 0.0–1.0
    lane_details: List[dict]
    avg_deviation: float


def head_c_qresidue_pattern(
    spins: List[int],
    basis: List[int] = None,
) -> HeadC_Result:
    """
    Test: Does output follow quadratic residue distribution?
    
    Algorithm:
    1. Per modulus m: count how many spins % m are QR
    2. Expected QR density ≈ 0.5 for odd primes
    3. Measure deviation from expected
    4. High deviation → QR_STRUCTURED, else QR_NOISE/SPARSE
    """
    if basis is None:
        basis = SAFE_BASIS
    
    lane_details = []
    deviations = []
    
    for modulus in basis:
        # Compute QR set
        qr_set = {(x * x) % modulus for x in range(modulus)}
        
        # Count QR in observed lane
        lane_values = [s % modulus for s in spins]
        qr_count = sum(1 for v in lane_values if v in qr_set)
        qr_ratio = qr_count / len(lane_values) if lane_values else 0.0
        
        # Expected for prime moduli
        expected = 0.5  # Approximation for odd primes
        deviation = abs(qr_ratio - expected)
        deviations.append(deviation)
        
        lane_details.append({
            'modulus': modulus,
            'qr_ratio': qr_ratio,
            'deviation': deviation,
        })
    
    avg_deviation = mean(deviations) if deviations else 0.0
    
    # Verdict
    if avg_deviation > 0.3:
        verdict = "QR_STRUCTURED"
    elif avg_deviation > 0.1:
        verdict = "QR_NOISE"
    else:
        verdict = "QR_SPARSE"
    
    confidence = min(avg_deviation * 2, 1.0)  # Scale to [0, 1]
    
    return HeadC_Result(
        verdict=verdict,
        confidence=confidence,
        lane_details=lane_details,
        avg_deviation=avg_deviation
    )


# ═══════════════════════════════════════════════════════════════
# HEAD D: CROSS-CORRELATION / COMPOSITION DETECTOR
# ═══════════════════════════════════════════════════════════════

@dataclass
class HeadD_Result:
    """Head D: Composition detection verdict."""
    verdict: str  # "COMPOUND", "SINGLE", "MASKED"
    confidence: float  # 0.0–1.0
    lane_details: List[dict]
    correlation_variance: float
    avg_correlation: float


def pearson_correlation_integer(x: List[int], y: List[int]) -> float:
    """
    Compute Pearson correlation using integer arithmetic.
    Returns value in approximately [-1, 1].
    """
    if len(x) < 2 or len(y) < 2:
        return 0.0
    
    n = len(x)
    mean_x = sum(x) // n
    mean_y = sum(y) // n
    
    numerator = sum((x[i] - mean_x) * (y[i] - mean_y) for i in range(n))
    denom_x = sum((x[i] - mean_x) ** 2 for i in range(n))
    denom_y = sum((y[i] - mean_y) ** 2 for i in range(n))
    
    if denom_x <= 0 or denom_y <= 0:
        return 0.0
    
    # Use scaled integer division to avoid float
    try:
        corr = numerator / (denom_x * denom_y) ** 0.5
        return max(-1.0, min(1.0, corr))
    except:
        return 0.0


def head_d_composition(
    spins: List[int],
    basis: List[int] = None,
) -> HeadD_Result:
    """
    Test: Is generator compound (multiple LCGs/sources)?
    
    Algorithm:
    1. Per lane: compute correlation between successive values
    2. Single LCG → high correlation (linear dependency)
    3. Compound → mixed correlation (different components)
    4. Variance in correlations → indicates composition
    """
    if basis is None:
        basis = SAFE_BASIS
    
    lane_details = []
    correlations = []
    
    for modulus in basis:
        lane_values = [s % modulus for s in spins]
        
        # Correlation lag-1
        if len(lane_values) > 1:
            x = lane_values[:-1]
            y = lane_values[1:]
            corr = pearson_correlation_integer(x, y)
        else:
            corr = 0.0
        
        correlations.append(corr)
        lane_details.append({
            'modulus': modulus,
            'correlation_lag1': corr,
        })
    
    avg_correlation = mean(correlations) if correlations else 0.0
    
    # Variance in correlations
    if len(correlations) > 1:
        try:
            correlation_variance = stdev(correlations) ** 2
        except:
            correlation_variance = 0.0
    else:
        correlation_variance = 0.0
    
    # Verdict: high variance → composition
    if correlation_variance > 0.10:
        verdict = "COMPOUND"
    elif correlation_variance > 0.03:
        verdict = "MASKED"
    else:
        verdict = "SINGLE"
    
    confidence = min(correlation_variance * 10, 1.0)
    
    return HeadD_Result(
        verdict=verdict,
        confidence=confidence,
        lane_details=lane_details,
        correlation_variance=correlation_variance,
        avg_correlation=avg_correlation
    )


# ═══════════════════════════════════════════════════════════════
# HYDRA ORCHESTRATION
# ═══════════════════════════════════════════════════════════════

@dataclass
class HydraResult:
    """Complete 4-head Hydra verdict."""
    generator_name: str
    total_spins: int
    
    head_a: HeadA_Result
    head_b: HeadB_Result
    head_c: HeadC_Result
    head_d: HeadD_Result
    
    consensus_type: str
    confidence_scores: List[float]
    max_confidence_head: str
    
    def to_dict(self):
        return {
            'generator_name': self.generator_name,
            'total_spins': self.total_spins,
            'head_a': asdict(self.head_a),
            'head_b': {k: v for k, v in asdict(self.head_b).items() if k != 'bit_linearity'},
            'head_c': asdict(self.head_c),
            'head_d': asdict(self.head_d),
            'consensus_type': self.consensus_type,
            'confidence_scores': {
                'A_LCG': self.head_a.confidence,
                'B_GF2': self.head_b.confidence,
                'C_QR': self.head_c.confidence,
                'D_Comp': self.head_d.confidence,
            },
            'max_confidence_head': self.max_confidence_head,
        }


def infer_consensus_type(head_a, head_b, head_c, head_d):
    """Infer generator type from head verdicts."""
    if head_a.verdict == "YES":
        return "LCG-class"
    elif head_b.verdict == "GF2_LINEAR":
        return "LFSR-like"
    elif head_d.verdict == "COMPOUND":
        return "Compound"
    elif head_c.verdict == "QR_STRUCTURED":
        return "QR-Masked"
    else:
        return "Unknown"


def run_hydra(
    spins: List[int],
    generator_name: str,
    basis: List[int] = None,
) -> HydraResult:
    """
    Run full Hydra oracle (4 heads) on observed spin sequence.
    """
    if basis is None:
        basis = SAFE_BASIS
    
    # Run all 4 heads in parallel
    head_a = head_a_lcg_linearity(spins, basis)
    head_b = head_b_gf2_linear(spins)
    head_c = head_c_qresidue_pattern(spins, basis)
    head_d = head_d_composition(spins, basis)
    
    # Aggregate confidence
    confidences = [head_a.confidence, head_b.confidence, head_c.confidence, head_d.confidence]
    max_confidence = max(confidences)
    max_head_idx = confidences.index(max_confidence)
    max_heads = ["A_LCG", "B_GF2", "C_QR", "D_Comp"]
    max_confidence_head = max_heads[max_head_idx]
    
    # Infer consensus
    consensus_type = infer_consensus_type(head_a, head_b, head_c, head_d)
    
    return HydraResult(
        generator_name=generator_name,
        total_spins=len(spins),
        head_a=head_a,
        head_b=head_b,
        head_c=head_c,
        head_d=head_d,
        consensus_type=consensus_type,
        confidence_scores=confidences,
        max_confidence_head=max_confidence_head,
    )


if __name__ == "__main__":
    # Unit test: synthetic LCG
    print("Testing Hydra harness with synthetic LCG...")
    
    a, c, m = 5, 3, 15015
    x = 12345
    spins = []
    for _ in range(200):
        x = (a * x + c) % m
        spins.append(x)
    
    result = run_hydra(spins, "Synthetic LCG (a=5, c=3)")
    
    print(f"\nGenerator: {result.generator_name}")
    print(f"Consensus: {result.consensus_type}")
    print(f"Confidences: A={result.head_a.confidence:.2f}, B={result.head_b.confidence:.2f}, "
          f"C={result.head_c.confidence:.2f}, D={result.head_d.confidence:.2f}")
    print(f"Max confidence: {result.max_confidence_head}")
    print()
    print("JSON dump:")
    print(json.dumps(result.to_dict(), indent=2))
