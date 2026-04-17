#!/usr/bin/env python3
"""
T2.2: Lane-by-Lane Structure Detection
========================================
Run the commutativity projector against all 10 generator classes.
Build signature table: generator_class → lane_pass_pattern.

Output:
  - projector_signatures.md (human-readable table)
  - projector_signatures.json (machine-readable results)
"""

from __future__ import annotations
import sys
import json
import time
from dataclasses import dataclass, asdict
from typing import List, Dict, Any

# Import generators from harness
sys.path.insert(0, '/home/claude')
from cram_harness_v2 import (
    SimpleLCG,
    FullPeriodLCG,
    ConstantAdditive,
    MultiplicativeLCG,
    ReelMappedLCG,
    TruncatedLCG,
    LFSR16,
    CompoundLCG,
    XorShift32,
    MersenneTwister,
    SAFE_BASIS,
)

from projector_slot import check_commutativity, ProjectorResult

# Compute SAFE_M
from functools import reduce
SAFE_M = reduce(lambda a, b: a * b, SAFE_BASIS)  # 15,015

# ═══════════════════════════════════════════════════════════════
# Generator Inventory
# ═══════════════════════════════════════════════════════════════

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


# ═══════════════════════════════════════════════════════════════
# Run Projector on All Generators
# ═══════════════════════════════════════════════════════════════

def run_t2_2(
    num_spins: int = 200,
    calibration_window: int = 20,
) -> Dict[str, Any]:
    """
    Run projector on all generators.
    
    Returns:
      {
        'timestamp': ISO timestamp,
        'total_generators': int,
        'generators': [
          {
            'name': str,
            'projector_result': ProjectorResult as dict,
            'generation_time_ms': float,
          },
          ...
        ],
        'summary': {...}
      }
    """
    from datetime import datetime
    
    print(f"T2.2: Lane-by-Lane Structure Detection")
    print(f"{'='*70}")
    print(f"Generators: {len(GENERATORS)}")
    print(f"Spins per generator: {num_spins}")
    print(f"Calibration window: {calibration_window}")
    print(f"{'='*70}\n")
    
    results = {
        'timestamp': datetime.now().isoformat(),
        'total_generators': len(GENERATORS),
        'generators': [],
        'summary': {}
    }
    
    commutativity_scores = []
    commutative_count = 0
    
    for gen_name, gen_instance in GENERATORS:
        print(f"[*] {gen_name:40s} ", end='', flush=True)
        
        t0 = time.time()
        
        # Generate spins
        spins = []
        for _ in range(num_spins):
            spins.append(gen_instance.next())
        
        t1 = time.time()
        gen_time_ms = (t1 - t0) * 1000
        
        # Run projector
        projector_result = check_commutativity(
            spins,
            gen_name,
            basis=SAFE_BASIS,
            calibration_window=calibration_window
        )
        
        results['generators'].append({
            'name': gen_name,
            'projector_result': projector_result.to_dict(),
            'generation_time_ms': gen_time_ms,
        })
        
        # Track summary
        is_commutative = projector_result.fully_commutative
        commutativity_scores.append(projector_result.commutativity_score)
        if is_commutative:
            commutative_count += 1
        
        status = "✓ FULLY COMMUTATIVE" if is_commutative else f"✗ PARTIAL ({projector_result.commutativity_score*100:.0f}%)"
        print(f"{status}")
    
    # Summary
    results['summary'] = {
        'fully_commutative_count': commutative_count,
        'total_generators': len(GENERATORS),
        'avg_commutativity_score': sum(commutativity_scores) / len(commutativity_scores),
        'max_commutativity_score': max(commutativity_scores),
        'min_commutativity_score': min(commutativity_scores),
    }
    
    print(f"\n{'='*70}")
    print(f"SUMMARY:")
    print(f"  Fully commutative: {commutative_count}/{len(GENERATORS)}")
    print(f"  Avg commutativity score: {results['summary']['avg_commutativity_score']*100:.1f}%")
    print(f"{'='*70}\n")
    
    return results


# ═══════════════════════════════════════════════════════════════
# Format Results
# ═══════════════════════════════════════════════════════════════

def format_signatures_markdown(results: Dict[str, Any]) -> str:
    """Format results as human-readable Markdown table."""
    
    lines = [
        "# Projector Signatures — Lane-by-Lane Structure Detection",
        "",
        "**T2.2 Output: Generator Class → Lane Commutativity Pattern**",
        "",
        "## Summary",
        "",
        f"- **Fully Commutative:** {results['summary']['fully_commutative_count']}/{results['summary']['total_generators']} generators",
        f"- **Avg Commutativity Score:** {results['summary']['avg_commutativity_score']*100:.1f}%",
        f"- **Range:** {results['summary']['min_commutativity_score']*100:.1f}% – {results['summary']['max_commutativity_score']*100:.1f}%",
        "",
        "## Signature Table",
        "",
        "| # | Generator | Commutative | Score | Lane Pattern (Mod 3,5,7,11,13) |",
        "|---|-----------|-------------|-------|--------------------------------|",
    ]
    
    for i, gen_data in enumerate(results['generators'], 1):
        name = gen_data['name']
        proj = gen_data['projector_result']
        
        commutative = "✅ YES" if proj['fully_commutative'] else "❌ NO"
        score = f"{proj['commutativity_score']*100:.0f}%"
        
        lane_pattern = []
        for lane_result in proj['lane_results']:
            if lane_result['is_linear']:
                lane_pattern.append(f"✓{lane_result['modulus']}")
            else:
                consistency_pct = int(lane_result['consistency_score']*100)
                lane_pattern.append(f"✗{lane_result['modulus']}({consistency_pct}%)")
        
        lane_str = ", ".join(lane_pattern)
        lines.append(f"| {i:2d} | {name:40s} | {commutative:10s} | {score:6s} | {lane_str} |")
    
    lines.extend([
        "",
        "## Detailed Lane Breakdown",
        "",
    ])
    
    for i, gen_data in enumerate(results['generators'], 1):
        name = gen_data['name']
        proj = gen_data['projector_result']
        
        lines.append(f"### Generator {i}: {name}")
        lines.append("")
        lines.append(f"**Commutativity: {proj['commutativity_score']*100:.1f}% ({proj['commutative_lanes']}/{proj['total_lanes']} lanes)**")
        lines.append("")
        
        for lane_result in proj['lane_results']:
            m = lane_result['modulus']
            is_lin = "✓ LINEAR" if lane_result['is_linear'] else "✗ NON-LINEAR"
            a = lane_result['a_recovered']
            c = lane_result['c_recovered']
            consistency = f"{lane_result['consistency_score']*100:.1f}%"
            first_fail = lane_result['first_failure_idx']
            
            lines.append(f"- **Lane {lane_result['lane_number']} (mod {m}):** {is_lin}")
            lines.append(f"  - Recovered: a={a}, c={c}")
            lines.append(f"  - Consistency: {consistency}")
            if first_fail is not None:
                lines.append(f"  - First failure at transition {first_fail}→{first_fail+1}")
        
        lines.append("")
    
    return "\n".join(lines)


if __name__ == "__main__":
    # Run T2.2
    results = run_t2_2(num_spins=200, calibration_window=20)
    
    # Save JSON
    with open('/home/claude/projector_signatures.json', 'w') as f:
        json.dump(results, f, indent=2)
    print("✓ Saved: projector_signatures.json\n")
    
    # Save Markdown
    markdown = format_signatures_markdown(results)
    with open('/home/claude/projector_signatures.md', 'w') as f:
        f.write(markdown)
    print("✓ Saved: projector_signatures.md\n")
    
    # Print to stdout
    print(markdown)
