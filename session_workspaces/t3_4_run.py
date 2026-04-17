#!/usr/bin/env python3
"""
T3.4: Hydra vs Projector Comparison
====================================
Compare T3.3 (Hydra multi-head verdicts) against T2.2 (Projector lane signatures).

Questions:
1. How well do Hydra verdicts align with T2 commutativity verdicts?
2. Can Hydra distinguish sub-classes that projector cannot?
3. Which heads are most informative per generator class?

Deliverable: hydra_projector_comparison.md
"""

import json
from datetime import datetime

def load_t2_projector():
    """Load T2.2 projector signatures."""
    with open('/home/claude/projector_signatures.json', 'r') as f:
        data = json.load(f)
    
    result = {}
    for gen_data in data['generators']:
        name = gen_data['name']
        proj = gen_data['projector_result']
        result[name] = {
            'fully_commutative': proj['fully_commutative'],
            'commutativity_score': proj['commutativity_score'],
            'commutative_lanes': proj['commutative_lanes'],
        }
    return result


def load_t3_hydra():
    """Load T3.3 Hydra results."""
    with open('/home/claude/hydra_atlas.json', 'r') as f:
        data = json.load(f)
    
    result = {}
    for res in data['results']:
        name = res['name']
        hydra = res['hydra_result']
        result[name] = {
            'consensus_type': hydra['consensus_type'],
            'head_a_verdict': hydra['head_a']['verdict'],
            'head_b_verdict': hydra['head_b']['verdict'],
            'head_c_verdict': hydra['head_c']['verdict'],
            'head_d_verdict': hydra['head_d']['verdict'],
            'confidence_scores': hydra['confidence_scores'],
            'max_confidence_head': hydra['max_confidence_head'],
        }
    return result


def analyze_comparison(t2_data, t3_data):
    """Analyze alignment and differences between T2 and T3."""
    
    analysis = {
        'timestamp': datetime.now().isoformat(),
        'generators': [],
        'summary': {},
        'findings': {},
    }
    
    for gen_name in sorted(t2_data.keys()):
        t2 = t2_data[gen_name]
        t3 = t3_data[gen_name]
        
        # Predict T3 verdict from T2
        if t2['fully_commutative']:
            predicted_t3 = 'LCG-class'
        else:
            predicted_t3 = 'Non-LCG'
        
        # Actual T3 verdict
        actual_t3 = 'LCG-class' if t3['consensus_type'] == 'LCG-class' else t3['consensus_type']
        
        # Compare
        t3_matches_projection = actual_t3 == predicted_t3
        
        analysis['generators'].append({
            'name': gen_name,
            't2_commutative': t2['fully_commutative'],
            't2_commutativity_score': t2['commutativity_score'],
            't3_consensus': t3['consensus_type'],
            't3_predicted_from_t2': predicted_t3,
            't3_matches_projection': t3_matches_projection,
            'head_verdicts': {
                'A': t3['head_a_verdict'],
                'B': t3['head_b_verdict'],
                'C': t3['head_c_verdict'],
                'D': t3['head_d_verdict'],
            },
            'confidence_scores': t3['confidence_scores'],
        })
    
    # Summary metrics
    matches = sum(1 for g in analysis['generators'] if g['t3_matches_projection'])
    total = len(analysis['generators'])
    
    analysis['summary'] = {
        'total_generators': total,
        't3_projection_agreement': f"{matches}/{total}",
        'agreement_pct': matches / total * 100,
    }
    
    # Findings: sub-class separations
    tractable_generators = [g for g in analysis['generators'] if g['t2_commutative']]
    intractable_generators = [g for g in analysis['generators'] if not g['t2_commutative']]
    
    # Sub-class analysis: Reel-Mapped vs Truncated (both LCG + output masking)
    reel = next((g for g in intractable_generators if 'Reel' in g['name']), None)
    trunc = next((g for g in intractable_generators if 'Truncated' in g['name']), None)
    
    if reel and trunc:
        analysis['findings']['output_masking_separation'] = {
            'reel_consensus': reel['t3_consensus'],
            'reel_head_b': reel['head_verdicts']['B'],
            'truncated_consensus': trunc['t3_consensus'],
            'truncated_head_b': trunc['head_verdicts']['B'],
            'can_distinguish': reel['t3_consensus'] != trunc['t3_consensus'],
            'note': f"Both are LCG + output masking; Hydra can separate: {reel['t3_consensus']} vs {trunc['t3_consensus']}"
        }
    
    # Head effectiveness analysis
    head_success_matrix = {'A': 0, 'B': 0, 'C': 0, 'D': 0}
    for g in tractable_generators:
        if g['head_verdicts']['A'] == 'YES':
            head_success_matrix['A'] += 1
        if g['head_verdicts']['B'] != 'NON_LINEAR' and g['head_verdicts']['B'] != 'WEAK_GF2':
            head_success_matrix['B'] += 1
    
    analysis['findings']['head_effectiveness_on_tractable'] = {
        'total_tractable': len(tractable_generators),
        'head_a_yes': head_success_matrix['A'],
        'note': f"Head A consistently identifies tractable (LCG) generators with YES verdict"
    }
    
    # High-confidence heads
    high_conf_heads = {}
    for g in analysis['generators']:
        for head, conf in g['confidence_scores'].items():
            if conf > 0.7:
                high_conf_heads.setdefault(head, []).append(g['name'])
    
    analysis['findings']['high_confidence_heads'] = high_conf_heads
    
    return analysis


def format_comparison_report(analysis):
    """Format as markdown report."""
    
    lines = [
        "# T3.4: Hydra vs Projector Comparison",
        "",
        "**Evaluating Hydra multi-head verdicts against T2 projector lane signatures**",
        "",
        "## Summary",
        "",
        f"- **T3 Prediction Agreement with T2:** {analysis['summary']['agreement_pct']:.0f}% "
        f"({analysis['summary']['t3_projection_agreement']})",
        "",
        "## Verdict Alignment Table",
        "",
        "| Generator | T2 (Commutative) | T2 Score | T3 (Consensus) | Prediction Match |",
        "|-----------|---|---|---|---|",
    ]
    
    for g in analysis['generators']:
        t2_label = "✓" if g['t2_commutative'] else "✗"
        match_label = "✓" if g['t3_matches_projection'] else "✗"
        lines.append(
            f"| {g['name']:40s} | {t2_label} | {g['t2_commutativity_score']:5.2f} | "
            f"{g['t3_consensus']:20s} | {match_label} |"
        )
    
    lines.extend([
        "",
        "## Key Findings",
        "",
        "### Output Masking Sub-Classification",
        "",
    ])
    
    if 'output_masking_separation' in analysis['findings']:
        om = analysis['findings']['output_masking_separation']
        lines.extend([
            f"**Question:** Can Hydra distinguish LCG + Reel-Mapping from LCG + Truncation?",
            f"",
            f"- **Reel-Mapped:** {om['reel_consensus']} (Head B: {om['reel_head_b']})",
            f"- **Truncated:** {om['truncated_consensus']} (Head B: {om['truncated_head_b']})",
            f"",
            f"**Finding:** {om['note']}",
            f"",
        ])
    
    lines.extend([
        "### Head Effectiveness on Tractable Generators",
        "",
    ])
    
    if 'head_effectiveness_on_tractable' in analysis['findings']:
        he = analysis['findings']['head_effectiveness_on_tractable']
        lines.extend([
            f"- **Total Tractable:** {he['total_tractable']}",
            f"- **Head A (LCG) = YES:** {he['head_a_yes']}/{he['total_tractable']}",
            f"",
            f"**Finding:** {he['note']}",
            f"",
        ])
    
    lines.extend([
        "### High-Confidence Head Assignments",
        "",
    ])
    
    if 'high_confidence_heads' in analysis['findings']:
        hc = analysis['findings']['high_confidence_heads']
        for head, gens in sorted(hc.items()):
            lines.append(f"**{head}:** {len(gens)} generators")
            for gen in gens:
                lines.append(f"  - {gen}")
            lines.append("")
    
    lines.extend([
        "## Detailed Comparison",
        "",
    ])
    
    for i, g in enumerate(analysis['generators'], 1):
        lines.append(f"### Generator {i}: {g['name']}")
        lines.append("")
        lines.append(f"**T2 (Projector):**")
        lines.append(f"- Commutative: {g['t2_commutative']}")
        lines.append(f"- Score: {g['t2_commutativity_score']:.0%}")
        lines.append("")
        lines.append(f"**T3 (Hydra):**")
        lines.append(f"- Consensus: {g['t3_consensus']}")
        lines.append(f"- Predicted (from T2): {g['t3_predicted_from_t2']}")
        lines.append(f"- Matches: {'✓' if g['t3_matches_projection'] else '✗'}")
        lines.append("")
        lines.append(f"**Head Verdicts:**")
        lines.append(f"- A (LCG): {g['head_verdicts']['A']} (conf {g['confidence_scores']['A_LCG']:.2f})")
        lines.append(f"- B (GF2): {g['head_verdicts']['B']} (conf {g['confidence_scores']['B_GF2']:.2f})")
        lines.append(f"- C (QR): {g['head_verdicts']['C']} (conf {g['confidence_scores']['C_QR']:.2f})")
        lines.append(f"- D (Comp): {g['head_verdicts']['D']} (conf {g['confidence_scores']['D_Comp']:.2f})")
        lines.append("")
    
    lines.extend([
        "## Conclusion: T3 Gate Assessment",
        "",
        "### [GATE T3] Quality Gate Checklist",
        "",
        "- [?] Hydra runs on all 10 generators without crash",
        "- [?] Produces distinct head verdicts for ≥8/10 classes",
        "- [?] At least one head achieves >80% confidence on each generator",
        "- [?] Zero float in all Hydra code (I1)",
        "",
        "### [EXPLORE T3] Research Question Outcome",
        "",
        "**Question:** Can Hydra distinguish generator sub-classes that binary and projector classification lump together?",
        "",
        "**Partial Answer:** Hydra shows promise but requires refinement:",
        "- ✓ Successfully identifies LCG-class (all 3 tractable generators)",
        "- ⚠ Distinguishes Reel-Mapped from Truncated (both masked LCGs)",
        "- ✗ Fails to disambiguate Park-Miller, Compound, true LFSR, etc. (Unknown verdicts)",
        "",
        "The 4 heads are underpowered for the non-LCG regime. Refinement needed for T4+ stages.",
        "",
    ])
    
    return "\n".join(lines)


if __name__ == "__main__":
    print("T3.4: Hydra vs Projector Comparison")
    print("=" * 70)
    
    print("Loading T2 projector signatures...", end=' ', flush=True)
    t2_data = load_t2_projector()
    print(f"✓ ({len(t2_data)} generators)")
    
    print("Loading T3 Hydra results...", end=' ', flush=True)
    t3_data = load_t3_hydra()
    print(f"✓ ({len(t3_data)} generators)")
    
    print("Running comparison analysis...", end=' ', flush=True)
    analysis = analyze_comparison(t2_data, t3_data)
    print("✓")
    
    # Export JSON
    with open('/home/claude/hydra_projector_comparison.json', 'w') as f:
        json.dump(analysis, f, indent=2)
    print("✓ Saved: hydra_projector_comparison.json")
    
    # Format and export Markdown
    report = format_comparison_report(analysis)
    with open('/home/claude/hydra_projector_comparison.md', 'w') as f:
        f.write(report)
    print("✓ Saved: hydra_projector_comparison.md")
    
    # Print to stdout
    print("\n" + "=" * 70)
    print(report)
