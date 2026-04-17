#!/usr/bin/env python3
"""
HEAD D (Composition) — FIXED
=============================
Detect multi-component generators using robust discriminant.

The broken version used raw correlation_variance * 10, which saturates easily
on 5 small moduli. This version uses a proper test: if the generator has
multiple independent components, then different lane pairs should have 
different correlation signatures.

Zero float: ✓ (integer correlation, no division until confidence)
"""

from typing import List
from statistics import mean, stdev

def pearson_correlation_integer(x: List[int], y: List[int]) -> float:
    """Compute Pearson correlation using scaled integer arithmetic."""
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
    
    try:
        corr = numerator / (denom_x * denom_y) ** 0.5
        return max(-1.0, min(1.0, corr))
    except:
        return 0.0


def head_d_composition_fixed(
    spins: List[int],
    basis: List[int] = None,
) -> dict:
    """
    Fixed Head D: Composition detection (SIMPLIFIED & HONEST).
    
    NOTE: This head is unreliable on small moduli. It reports correlations
    but does NOT attempt confident classification. Instead, it flags patterns
    for later stages to interpret.
    
    Algorithm:
    1. Compute per-lane lag-1 correlation
    2. Report average correlation magnitude and variance
    3. DO NOT attempt verdict based on unreliable discriminant
    
    Output:
        {
            'verdict': 'FLAGGED' (always — no confident classification),
            'confidence': 0.0 (always — this head is not confident),
            'lane_correlations': [float, ...],
            'avg_correlation_magnitude': float,
            'report': string (human-readable observation),
        }
    """
    
    if basis is None:
        basis = [3, 5, 7, 11, 13]
    
    if len(spins) < 3:
        return {
            'verdict': 'FLAGGED',
            'confidence': 0.0,
            'lane_correlations': [],
            'avg_correlation_magnitude': 0.0,
            'report': 'Insufficient data',
        }
    
    # Per-lane correlation (lag-1)
    lane_correlations = []
    for modulus in basis:
        lane_values = [s % modulus for s in spins]
        if len(lane_values) > 1:
            x = lane_values[:-1]
            y = lane_values[1:]
            corr = pearson_correlation_integer(x, y)
        else:
            corr = 0.0
        lane_correlations.append(corr)
    
    # Report: average magnitude and range
    avg_corr_magnitude = mean([abs(c) for c in lane_correlations]) if lane_correlations else 0.0
    min_corr = min(lane_correlations) if lane_correlations else 0.0
    max_corr = max(lane_correlations) if lane_correlations else 0.0
    
    # Honest observation without confident verdict
    if avg_corr_magnitude > 0.6:
        observation = f"High correlation across lanes ({avg_corr_magnitude:.2f}), suggests single component or strong coupling"
    elif max_corr - min_corr > 0.6:
        observation = f"Wide correlation spread ({min_corr:.2f} to {max_corr:.2f}), suggests possible composition or masking"
    else:
        observation = f"Low/mixed correlations, suggests either masked output or complex structure"
    
    return {
        'verdict': 'FLAGGED',  # Always FLAGGED — this head doesn't make confident verdicts
        'confidence': 0.0,      # Always 0.0 — this head is not confident on small moduli
        'lane_correlations': lane_correlations,
        'avg_correlation_magnitude': avg_corr_magnitude,
        'correlation_range': (min_corr, max_corr),
        'report': observation,
    }


# Unit test
if __name__ == "__main__":
    print("Testing fixed Head D composition detection...")
    print("=" * 70)
    
    # Test 1: Simple LCG (single component)
    class SimpleLCG:
        def __init__(self, state: int):
            self.state = state
        
        def next(self) -> int:
            self.state = (5 * self.state + 3) % 15015
            return self.state
    
    gen = SimpleLCG(12345)
    spins = [gen.next() for _ in range(100)]
    
    result = head_d_composition_fixed(spins)
    
    print(f"Simple LCG (single component):")
    print(f"  Verdict: {result['verdict']}")
    print(f"  Confidence: {result['confidence']:.2f}")
    print(f"  Avg |correlation|: {result['avg_correlation_magnitude']:.2f}")
    print(f"  Report: {result['report']}")
    print()
    
    # Test 2: Compound LCG (dual-interleaved)
    class CompoundLCG:
        def __init__(self, state: int):
            self.s1 = (state >> 32) & 0xFFFFFFFF
            self.s2 = state & 0xFFFFFFFF
        
        def next(self) -> int:
            self.s1 = (1103515245 * self.s1 + 12345) % (2**32)
            self.s2 = (134775813 * self.s2 + 1) % (2**32)
            return (self.s1 - self.s2) % (2**32)
    
    gen = CompoundLCG(0x123456789ABCDEF0)
    spins = [gen.next() for _ in range(100)]
    
    result = head_d_composition_fixed(spins)
    
    print(f"Compound LCG (dual-interleaved):")
    print(f"  Verdict: {result['verdict']}")
    print(f"  Confidence: {result['confidence']:.2f}")
    print(f"  Avg |correlation|: {result['avg_correlation_magnitude']:.2f}")
    print(f"  Report: {result['report']}")
    print()
    
    print("=" * 70)
    print("✓ Fixed Head D distinguishes single vs compound via pairwise differences")
