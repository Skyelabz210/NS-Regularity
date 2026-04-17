#!/usr/bin/env python3
"""
T3.3: Hydra Classification Atlas
================================
Run the 4-head Hydra oracle against all 10 generator classes.
Build comprehensive classification atlas.

Output:
  - hydra_atlas.md (human-readable)
  - hydra_atlas.json (machine-readable)
"""

import sys
import json
import time
from datetime import datetime
from dataclasses import asdict

sys.path.insert(0, '/home/claude')
from cram_harness_v2 import (
    SimpleLCG, FullPeriodLCG, ConstantAdditive,
    MultiplicativeLCG, ReelMappedLCG, TruncatedLCG,
    LFSR16, CompoundLCG, XorShift32, MersenneTwister,
    SAFE_BASIS,
)
from functools import reduce
from hydra_slot import run_hydra

SAFE_M = reduce(lambda a, b: a * b, SAFE_BASIS)

# Generator inventory
GENERATORS = [
    ("LCG (Simple)", SimpleLCG(SAFE_M)),
    ("LCG (Full Period / Hull-Dobell)", FullPeriodLCG(SAFE_M)),
    ("Constant Additive (a=1)", ConstantAdditive(SAFE_M)),
    ("LCG (Park-Miller, M=2^31-1)", MultiplicativeLCG(SAFE_M)),
    ("LCG (Reel-Mapped Output)", ReelMappedLCG(SAFE_M)),
    ("LCG (Truncated / Bit-Masked)", TruncatedLCG(SAFE_M)),
    ("Compound LCG (Dual-Interleaved)", CompoundLCG(SAFE_M)),
    ("LFSR-16 (Galois)", LFSR16(SAFE_M)),
    ("XorShift32", XorShift32(SAFE_M)),
    ("Mersenne Twister (MT19937)", MersenneTwister(SAFE_M)),
]


def run_t3_3(num_spins: int = 200):
    """
    Run Hydra fingerprinting on all generators.
    
    Returns:
      {
        'timestamp': ISO datetime,
        'total_generators': int,
        'results': [
          {
            'name': str,
            'hydra_result': HydraResult as dict,
            'generation_time_ms': float,
          },
          ...
        ],
        'summary': {...}
      }
    """
    
    print(f"T3.3: Hydra Classification Atlas")
    print(f"{'='*70}")
    print(f"Generators: {len(GENERATORS)}")
    print(f"Spins per generator: {num_spins}")
    print(f"{'='*70}\n")
    
    results = {
        'timestamp': datetime.now().isoformat(),
        'total_generators': len(GENERATORS),
        'results': [],
        'summary': {}
    }
    
    consensus_types = {}
    head_confidence_matrix = {}
    
    for gen_name, gen_instance in GENERATORS:
        print(f"[*] {gen_name:40s} ", end='', flush=True)
        
        t0 = time.time()
        
        # Generate spins
        spins = []
        for _ in range(num_spins):
            spins.append(gen_instance.next())
        
        t1 = time.time()
        gen_time_ms = (t1 - t0) * 1000
        
        # Run Hydra
        hydra_result = run_hydra(spins, gen_name, SAFE_BASIS)
        
        results['results'].append({
            'name': gen_name,
            'hydra_result': hydra_result.to_dict(),
            'generation_time_ms': gen_time_ms,
        })
        
        # Track summary
        consensus_type = hydra_result.consensus_type
        consensus_types.setdefault(consensus_type, []).append(gen_name)
        
        head_confidence_matrix[gen_name] = {
            'A_LCG': hydra_result.head_a.confidence,
            'B_GF2': hydra_result.head_b.confidence,
            'C_QR': hydra_result.head_c.confidence,
            'D_Comp': hydra_result.head_d.confidence,
        }
        
        max_conf = max(hydra_result.confidence_scores)
        print(f"{consensus_type:20s} (max conf: {max_conf:.2f})")
    
    # Summary
    results['summary'] = {
        'consensus_types': consensus_types,
        'head_confidence_matrix': head_confidence_matrix,
        'unique_consensus_types': len(consensus_types),
    }
    
    print(f"\n{'='*70}")
    print(f"SUMMARY:")
    print(f"  Unique consensus types: {len(consensus_types)}")
    for ct, gens in consensus_types.items():
        print(f"    {ct}: {len(gens)} generators")
    print(f"{'='*70}\n")
    
    return results


def format_atlas_markdown(results):
    """Format results as human-readable Markdown table."""
    
    lines = [
        "# Hydra Classification Atlas — T3.3 Output",
        "",
        "**4-Head Oracle Fingerprinting of All 10 Generator Classes**",
        "",
        "## Summary",
        "",
        f"- **Generators Tested:** {results['total_generators']}",
        f"- **Unique Consensus Types:** {results['summary']['unique_consensus_types']}",
        f"- **Timestamp:** {results['timestamp']}",
        "",
        "## Consensus Distribution",
        "",
    ]
    
    for ct, gens in sorted(results['summary']['consensus_types'].items()):
        lines.append(f"- **{ct}:** {len(gens)} generators")
        for gen in gens:
            lines.append(f"  - {gen}")
    
    lines.extend([
        "",
        "## Head Confidence Matrix",
        "",
        "Per-generator confidence scores for each head (0.0–1.0):",
        "",
        "| Generator | Head A (LCG) | Head B (GF2) | Head C (QR) | Head D (Comp) | Consensus |",
        "|-----------|---|---|---|---|---|",
    ])
    
    for result in results['results']:
        name = result['name']
        hydra = result['hydra_result']
        a_conf = hydra['head_a']['confidence']
        b_conf = hydra['head_b']['confidence']
        c_conf = hydra['head_c']['confidence']
        d_conf = hydra['head_d']['confidence']
        consensus = hydra['consensus_type']
        
        lines.append(
            f"| {name:40s} | {a_conf:5.2f} | {b_conf:5.2f} | {c_conf:5.2f} | "
            f"{d_conf:5.2f} | {consensus:20s} |"
        )
    
    lines.extend([
        "",
        "## Detailed Results",
        "",
    ])
    
    for i, result in enumerate(results['results'], 1):
        name = result['name']
        hydra = result['hydra_result']
        
        lines.append(f"### Generator {i}: {name}")
        lines.append("")
        lines.append(f"**Consensus Type:** {hydra['consensus_type']}")
        lines.append(f"**Confidences:** A={hydra['head_a']['confidence']:.2f}, "
                    f"B={hydra['head_b']['confidence']:.2f}, "
                    f"C={hydra['head_c']['confidence']:.2f}, "
                    f"D={hydra['head_d']['confidence']:.2f}")
        lines.append(f"**Max Confidence Head:** {hydra['max_confidence_head']}")
        lines.append("")
        
        lines.append("**Head A (LCG Linearity):**")
        lines.append(f"- Verdict: {hydra['head_a']['verdict']}")
        lines.append(f"- Fully Linear Lanes: {hydra['head_a']['fully_commutative_lanes']}/{hydra['head_a']['total_lanes']}")
        lines.append(f"- Avg Consistency: {hydra['head_a']['avg_consistency']:.1%}")
        lines.append("")
        
        lines.append("**Head B (GF(2) Linearity):**")
        lines.append(f"- Verdict: {hydra['head_b']['verdict']}")
        lines.append(f"- Linear Bits: {hydra['head_b']['linear_bits']}/{hydra['head_b']['total_bits']}")
        lines.append("")
        
        lines.append("**Head C (QResidue Pattern):**")
        lines.append(f"- Verdict: {hydra['head_c']['verdict']}")
        lines.append(f"- Avg Deviation: {hydra['head_c']['avg_deviation']:.3f}")
        lines.append("")
        
        lines.append("**Head D (Composition):**")
        lines.append(f"- Verdict: {hydra['head_d']['verdict']}")
        lines.append(f"- Avg Correlation: {hydra['head_d']['avg_correlation']:.2f}")
        lines.append(f"- Correlation Variance: {hydra['head_d']['correlation_variance']:.4f}")
        lines.append("")
    
    return "\n".join(lines)


if __name__ == "__main__":
    # Run T3.3
    results = run_t3_3(num_spins=200)
    
    # Save JSON
    with open('/home/claude/hydra_atlas.json', 'w') as f:
        json.dump(results, f, indent=2)
    print("✓ Saved: hydra_atlas.json\n")
    
    # Save Markdown
    markdown = format_atlas_markdown(results)
    with open('/home/claude/hydra_atlas.md', 'w') as f:
        f.write(markdown)
    print("✓ Saved: hydra_atlas.md\n")
    
    # Print to stdout
    print(markdown)
