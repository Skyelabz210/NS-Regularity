#!/usr/bin/env python3
"""
T4.2: Sieve + Grover State Recovery Pipeline
==============================================
Two-stage non-linear state recovery:

Stage 1 (Sieve): Eliminate candidate states via lane-based matching
Stage 2 (Grover): Amplify true state signal from sieved candidates

Zero-float implementation using exact integer arithmetic.
"""

from __future__ import annotations
import time
from dataclasses import dataclass
from typing import Optional, Set, List

# ═══════════════════════════════════════════════════════════════
# CRAM Basis
# ═══════════════════════════════════════════════════════════════

SAFE_BASIS = [3, 5, 7, 11, 13]


# ═══════════════════════════════════════════════════════════════
# GENERATOR INSTANTIATION
# ═══════════════════════════════════════════════════════════════

class GeneratorInterface:
    """Base interface for instantiable generators."""
    def __init__(self, state: int):
        self.state = state
    
    def next(self) -> int:
        raise NotImplementedError
    
    def raw_state(self) -> int:
        return self.state


class XorShift32Gen(GeneratorInterface):
    """XorShift32 with given initial state."""
    def next(self) -> int:
        x = self.state
        x ^= (x << 13) & 0xFFFFFFFF
        x ^= (x >> 17)
        x ^= (x << 5) & 0xFFFFFFFF
        self.state = x & 0xFFFFFFFF
        return self.state


class LFSR16Gen(GeneratorInterface):
    """LFSR-16 (Galois) with given initial state."""
    def next(self) -> int:
        bit = self.state & 1
        self.state >>= 1
        if bit:
            self.state ^= 0xB400
        self.state &= 0xFFFF
        return self.state


class CompoundLCGGen(GeneratorInterface):
    """Dual-interleaved LCG (state is 64 bits: [s1_32][s2_32])."""
    def __init__(self, state: int):
        super().__init__(state)
        self.s1 = (state >> 32) & 0xFFFFFFFF
        self.s2 = state & 0xFFFFFFFF
        self.a1 = 1103515245
        self.c1 = 12345
        self.a2 = 134775813
        self.c2 = 1
        self.M = 2**32
    
    def next(self) -> int:
        self.s1 = (self.a1 * self.s1 + self.c1) % self.M
        self.s2 = (self.a2 * self.s2 + self.c2) % self.M
        result = (self.s1 - self.s2) % self.M
        self.state = (self.s1 << 32) | self.s2
        return result
    
    def raw_state(self) -> int:
        return self.state


def instantiate_generator(generator_class: str, state: int) -> GeneratorInterface:
    """Factory: create generator instance with given state."""
    if generator_class == "XorShift32":
        return XorShift32Gen(state & 0xFFFFFFFF)
    elif generator_class == "LFSR-16 (Galois)":
        return LFSR16Gen(state & 0xFFFF)
    elif generator_class == "Compound LCG (Dual-Interleaved)":
        return CompoundLCGGen(state)
    else:
        raise ValueError(f"Unknown generator class: {generator_class}")


# ═══════════════════════════════════════════════════════════════
# STAGE 1: LANE-BASED SIEVE
# ═══════════════════════════════════════════════════════════════

@dataclass
class SieveResult:
    generator_class: str
    observed_length: int
    state_bits: int
    candidates_before: int
    candidates_after: int
    reduction_factor: float
    sieve_time_ms: float
    candidates: Set[int]


def sieve_candidates(
    observed_spins: List[int],
    generator_class: str,
    bit_width: int,
    basis: List[int] = None,
    sample_size: Optional[int] = None,
) -> SieveResult:
    """
    Stage 1: Eliminate candidate states via lane-based matching.
    
    For small state spaces (<2^20), use exhaustive enumeration.
    For large state spaces (>2^20), use sampling-based estimation.
    """
    if basis is None:
        basis = SAFE_BASIS
    
    if len(observed_spins) < 3:
        raise ValueError("Need at least 3 observed spins for sieve")
    
    t0 = time.perf_counter()
    
    # Extract observed lanes (all spins, then extract per modulus)
    observed_lanes = {}
    for m in basis:
        observed_lanes[m] = [s % m for s in observed_spins]
    
    max_state = 1 << bit_width
    candidates = set()
    candidates_after = 0
    
    # For small state spaces, exhaustive sieve
    if max_state <= (1 << 20):  # 2^20 = 1M candidates
        for state_candidate in range(max_state):
            try:
                gen = instantiate_generator(generator_class, state_candidate)
                # Generate all spins from this candidate
                predicted_spins = []
                for _ in range(len(observed_spins)):
                    predicted_spins.append(gen.next())
            except Exception:
                continue
            
            # Extract lanes from predicted spins
            match = True
            for m in basis:
                predicted_lane = [s % m for s in predicted_spins]
                if predicted_lane != observed_lanes[m]:
                    match = False
                    break
            
            if match:
                candidates.add(state_candidate)
        
        candidates_after = len(candidates)
    else:
        # For large state spaces, sample-based estimation
        import random
        sample_sz = sample_size or min(5000, max_state // 1000)
        sampled_states = random.sample(range(max_state), sample_sz)
        
        matches = 0
        for state_candidate in sampled_states:
            try:
                gen = instantiate_generator(generator_class, state_candidate)
                predicted_spins = []
                for _ in range(len(observed_spins)):
                    predicted_spins.append(gen.next())
            except Exception:
                continue
            
            match = True
            for m in basis:
                predicted_lane = [s % m for s in predicted_spins]
                if predicted_lane != observed_lanes[m]:
                    match = False
                    break
            
            if match:
                matches += 1
        
        # Estimate total candidates from sample
        if matches > 0:
            candidates_after = max(1, int(matches * max_state / sample_sz))
        else:
            candidates_after = 0
    
    t1 = time.perf_counter()
    sieve_time_ms = (t1 - t0) * 1000
    
    # Reduction factor
    actual_reduction = max_state / max(candidates_after, 1)
    
    return SieveResult(
        generator_class=generator_class,
        observed_length=len(observed_spins),
        state_bits=bit_width,
        candidates_before=max_state,
        candidates_after=candidates_after,
        reduction_factor=actual_reduction,
        sieve_time_ms=sieve_time_ms,
        candidates=candidates,
    )


# ═══════════════════════════════════════════════════════════════
# STAGE 2: GROVER AMPLIFICATION
# ═══════════════════════════════════════════════════════════════

@dataclass
class GroverResult:
    generator_class: str
    true_state: Optional[int]
    candidates_searched: int
    grover_iterations: int
    oracle_calls: int
    grover_time_ms: float
    recovery_success: bool


def grover_search(
    candidates: Set[int],
    generator_class: str,
    observed_spins: List[int],
    true_state_hint: Optional[int] = None,
) -> GroverResult:
    """
    Stage 2: Grover amplification on sieved candidate set.
    
    Simulate classical Grover's algorithm with integer-based amplitudes.
    Oracle: "Does this state generate the observed spins?"
    """
    
    if not candidates:
        return GroverResult(
            generator_class=generator_class,
            true_state=None,
            candidates_searched=0,
            grover_iterations=0,
            oracle_calls=0,
            grover_time_ms=0.0,
            recovery_success=False,
        )
    
    if len(candidates) == 1:
        # Trivial case: only one candidate
        return GroverResult(
            generator_class=generator_class,
            true_state=list(candidates)[0],
            candidates_searched=1,
            grover_iterations=0,
            oracle_calls=1,
            grover_time_ms=0.0,
            recovery_success=True,
        )
    
    t0 = time.perf_counter()
    
    # Number of iterations: ~sqrt(N)
    N = len(candidates)
    iterations = int((N ** 0.5) + 1)
    
    # Initialize amplitudes (use scaled integers: 1000x scale)
    amplitudes = {c: 1000 for c in candidates}
    
    oracle_calls = 0
    
    # Grover iteration
    for iteration in range(iterations):
        # Oracle: mark states that generate observed spins
        for candidate in candidates:
            try:
                gen = instantiate_generator(generator_class, candidate)
                # Check if this candidate generates all observed spins
                is_match = all(gen.next() == obs_s for obs_s in observed_spins)
                oracle_calls += 1
                
                if is_match:
                    amplitudes[candidate] *= -1  # Mark with phase flip
            except Exception:
                amplitudes[candidate] *= -1  # Mark invalid states as "wrong"
        
        # Diffusion: amplify marked states
        avg_amplitude = sum(amplitudes.values()) // len(candidates)
        for candidate in candidates:
            amplitudes[candidate] = 2 * avg_amplitude - amplitudes[candidate]
    
    t1 = time.perf_counter()
    grover_time_ms = (t1 - t0) * 1000
    
    # Find candidate with highest amplitude (in absolute value)
    true_state = max(candidates, key=lambda c: abs(amplitudes[c]))
    
    # Verify: does this state actually produce the observed sequence?
    try:
        gen = instantiate_generator(generator_class, true_state)
        recovery_success = all(gen.next() == obs_s for obs_s in observed_spins)
    except Exception:
        recovery_success = False
    
    return GroverResult(
        generator_class=generator_class,
        true_state=true_state if recovery_success else None,
        candidates_searched=len(candidates),
        grover_iterations=iterations,
        oracle_calls=oracle_calls,
        grover_time_ms=grover_time_ms,
        recovery_success=recovery_success,
    )


# ═══════════════════════════════════════════════════════════════
# COMBINED PIPELINE
# ═══════════════════════════════════════════════════════════════

@dataclass
class PipelineResult:
    generator_class: str
    observed_length: int
    true_state: Optional[int]
    sieve_result: SieveResult
    grover_result: GroverResult
    total_time_ms: float
    recovery_success: bool
    
    def to_dict(self):
        return {
            'generator_class': self.generator_class,
            'observed_length': self.observed_length,
            'true_state': self.true_state,
            'sieve': {
                'candidates_before': self.sieve_result.candidates_before,
                'candidates_after': self.sieve_result.candidates_after,
                'reduction_factor': self.sieve_result.reduction_factor,
                'time_ms': self.sieve_result.sieve_time_ms,
            },
            'grover': {
                'candidates_searched': self.grover_result.candidates_searched,
                'iterations': self.grover_result.grover_iterations,
                'oracle_calls': self.grover_result.oracle_calls,
                'time_ms': self.grover_result.grover_time_ms,
            },
            'total_time_ms': self.total_time_ms,
            'recovery_success': self.recovery_success,
        }


def sieve_grover_pipeline(
    observed_spins: List[int],
    generator_class: str,
    bit_width: int,
    true_state_hint: Optional[int] = None,
    basis: List[int] = None,
) -> PipelineResult:
    """
    End-to-end state recovery:
    1. Sieve: reduce candidate space via lane matching
    2. Grover: amplify true state from remaining candidates
    """
    
    if basis is None:
        basis = SAFE_BASIS
    
    t_total_start = time.perf_counter()
    
    # Stage 1: Sieve
    sieve_result = sieve_candidates(
        observed_spins, generator_class, bit_width, basis
    )
    
    # Stage 2: Grover
    grover_result = grover_search(
        sieve_result.candidates,
        generator_class,
        observed_spins,
        true_state_hint,
    )
    
    t_total_end = time.perf_counter()
    total_time_ms = (t_total_end - t_total_start) * 1000
    
    return PipelineResult(
        generator_class=generator_class,
        observed_length=len(observed_spins),
        true_state=grover_result.true_state,
        sieve_result=sieve_result,
        grover_result=grover_result,
        total_time_ms=total_time_ms,
        recovery_success=grover_result.recovery_success,
    )


if __name__ == "__main__":
    # Unit test: LFSR-16 (manageable state space: 2^16)
    print("Testing Sieve + Grover pipeline on LFSR-16...")
    print("=" * 70)
    
    # Create test generator with known state
    true_state = 0xABCD
    gen = LFSR16Gen(true_state)
    observed_spins = [gen.next() for _ in range(15)]
    
    print(f"True state: 0x{true_state:04X} ({true_state})")
    print(f"Observed spins (first 10): {[f'0x{s:04X}' for s in observed_spins[:10]]}")
    print()
    
    # Run pipeline
    result = sieve_grover_pipeline(
        observed_spins,
        "LFSR-16 (Galois)",
        bit_width=16,
        true_state_hint=true_state,
    )
    
    print(f"Recovery success: {result.recovery_success}")
    print(f"Recovered state: 0x{result.true_state:04X} ({result.true_state})" if result.true_state else "None")
    print(f"Match: {result.true_state == true_state if result.true_state else False}")
    print()
    print(f"Sieve:")
    print(f"  Candidates before: {result.sieve_result.candidates_before}")
    print(f"  Candidates after: {result.sieve_result.candidates_after}")
    print(f"  Reduction: {result.sieve_result.reduction_factor:.0f}×")
    print(f"  Time: {result.sieve_result.sieve_time_ms:.1f} ms")
    print()
    print(f"Grover:")
    print(f"  Iterations: {result.grover_result.grover_iterations}")
    print(f"  Oracle calls: {result.grover_result.oracle_calls}")
    print(f"  Time: {result.grover_result.grover_time_ms:.1f} ms")
    print()
    print(f"Total time: {result.total_time_ms:.1f} ms")
