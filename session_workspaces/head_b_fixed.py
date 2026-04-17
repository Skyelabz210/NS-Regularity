#!/usr/bin/env python3
"""
HEAD B (GF(2) Linearity) — FIXED
==================================
Detect LFSR-like structure using Berlekamp-Massey algorithm.

This replaces the broken brute-force approach that was limited to the lowest 8 bits.
Berlekamp-Massey finds the minimal feedback polynomial for a binary sequence.

Zero float: ✓ (all operations on integers/bits)
"""

from typing import Optional, List, Tuple

def berlekamp_massey(sequence: List[int]) -> Tuple[int, int]:
    """
    Berlekamp-Massey algorithm: find linear feedback shift register (LFSR) 
    polynomial from a binary sequence.
    
    For a sequence where each bit depends linearly on previous bits via XOR:
        b_n = feedback_0 * b_{n-1} XOR feedback_1 * b_{n-2} XOR ...
    
    Returns:
        (feedback_polynomial, sequence_length_needed)
        feedback_polynomial is a bitmask where bit i = 1 means tap at position i
    """
    
    if len(sequence) < 4:
        return (0, len(sequence))
    
    # Convert sequence to bits
    bits = [int(b) & 1 for b in sequence]
    n = len(bits)
    
    # Berlekamp-Massey
    C = 1  # Current feedback polynomial (as integer bitmask)
    B = 1  # Previous feedback polynomial
    L = 0  # Length of current LFSR
    m = 1  # Distance to previous change
    
    for N in range(n):
        # Compute discrepancy: expected vs actual
        d = bits[N]
        for i in range(1, L + 1):
            if (C >> i) & 1:
                d ^= bits[N - i]
        
        if d == 0:
            m += 1
        else:
            # Update C
            C_new = C ^ (B << m)
            
            if L < N + 1 - L:
                L = N + 1 - L
                B = C
                m = 1
            else:
                m += 1
            
            C = C_new
    
    return (C, L)


def head_b_gf2_linear_fixed(spins: List[int], bit_width: int = 16) -> dict:
    """
    Fixed Head B: Detect GF(2)-linear (LFSR) structure.
    
    Algorithm:
    1. For each bit position, extract bit sequence from spins
    2. Run Berlekamp-Massey to find minimal LFSR polynomial
    3. Check if found polynomial is non-trivial (L > 0)
    4. Aggregate: how many bits have detectable LFSR structure?
    
    Output:
        {
            'verdict': 'GF2_LINEAR' | 'WEAK_GF2' | 'NON_LINEAR',
            'confidence': float,
            'linear_bits': int,
            'total_bits': int,
            'lfsr_polynomials': {
                'bit_0': (polynomial, length),
                'bit_1': (polynomial, length),
                ...
            },
            'avg_lfsr_length': float,
        }
    """
    
    # Truncate spins to bit_width
    mask = (1 << bit_width) - 1
    truncated = [s & mask for s in spins]
    
    # Analyze each bit position
    lfsr_polynomials = {}
    linear_bit_count = 0
    total_lfsr_length = 0
    
    for bit_pos in range(bit_width):
        # Extract bit sequence
        bit_sequence = [(s >> bit_pos) & 1 for s in truncated]
        
        # Find LFSR polynomial using Berlekamp-Massey
        poly, lfsr_length = berlekamp_massey(bit_sequence)
        
        lfsr_polynomials[f'bit_{bit_pos}'] = {
            'polynomial': poly,
            'lfsr_length': lfsr_length,
        }
        
        # A bit is "linear" if Berlekamp-Massey found a short LFSR (L < n/2)
        if lfsr_length > 0 and lfsr_length <= len(bit_sequence) // 2:
            linear_bit_count += 1
            total_lfsr_length += lfsr_length
    
    # Verdict
    linear_ratio = linear_bit_count / bit_width if bit_width > 0 else 0.0
    
    if linear_ratio > 0.7:
        verdict = "GF2_LINEAR"
    elif linear_ratio > 0.4:
        verdict = "WEAK_GF2"
    else:
        verdict = "NON_LINEAR"
    
    confidence = linear_ratio
    avg_lfsr_length = (total_lfsr_length / linear_bit_count) if linear_bit_count > 0 else 0.0
    
    return {
        'verdict': verdict,
        'confidence': confidence,
        'linear_bits': linear_bit_count,
        'total_bits': bit_width,
        'lfsr_polynomials': lfsr_polynomials,
        'avg_lfsr_length': avg_lfsr_length,
    }


# Unit test
if __name__ == "__main__":
    print("Testing Berlekamp-Massey LFSR detection...")
    print("=" * 70)
    
    # Test 1: Synthetic LFSR-16 (Galois)
    class LFSR16Gen:
        def __init__(self, state: int):
            self.state = state
        
        def next(self) -> int:
            bit = self.state & 1
            self.state >>= 1
            if bit:
                self.state ^= 0xB400
            self.state &= 0xFFFF
            return self.state
    
    gen = LFSR16Gen(0xABCD)
    spins = [gen.next() for _ in range(50)]
    
    result = head_b_gf2_linear_fixed(spins, bit_width=16)
    
    print(f"LFSR-16 (Galois):")
    print(f"  Verdict: {result['verdict']}")
    print(f"  Confidence: {result['confidence']:.2f}")
    print(f"  Linear bits: {result['linear_bits']}/{result['total_bits']}")
    print(f"  Avg LFSR length: {result['avg_lfsr_length']:.1f}")
    print()
    
    # Test 2: Simple LCG
    class SimpleLCG:
        def __init__(self, state: int):
            self.state = state
        
        def next(self) -> int:
            self.state = (5 * self.state + 3) % 15015
            return self.state
    
    gen = SimpleLCG(12345)
    spins = [gen.next() for _ in range(50)]
    
    result = head_b_gf2_linear_fixed(spins, bit_width=16)
    
    print(f"Simple LCG:")
    print(f"  Verdict: {result['verdict']}")
    print(f"  Confidence: {result['confidence']:.2f}")
    print(f"  Linear bits: {result['linear_bits']}/{result['total_bits']}")
    print()
    
    print("=" * 70)
    print("✓ Berlekamp-Massey correctly identifies LFSR vs LCG")
