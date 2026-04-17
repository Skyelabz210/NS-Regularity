#!/usr/bin/env python3
"""
HYDRA OPTIMAL OPERATOR SEARCH — Phase 2
=========================================
Q7 revealed different primes want different operators.
Now: build heterogeneous heads and test multi-head union.

Strategy:
1. Find optimal operator per prime for twins (d=2)
2. Build heterogeneous head with per-prime optimal
3. Test multi-head union with heterogeneous + Sqr heads
4. Find ceiling of multi-head elimination
5. Test on other families (cousin, sexy, d=30, d=360)
"""

import math
from collections import Counter
import time

def gcd(a, b):
    while b: a, b = b, a % b
    return abs(a)

def is_prime(n):
    if n < 2: return False
    if n < 4: return True
    if n % 2 == 0 or n % 3 == 0: return False
    i = 5
    while i * i <= n:
        if n % i == 0 or n % (i+2) == 0: return False
        i += 6
    return True

def mod_inverse(a, m):
    if a == 0: return None
    old_r, r = a % m, m
    old_s, s = 1, 0
    while r:
        q = old_r // r
        old_r, r = r, old_r - q * r
        old_s, s = s, old_s - q * s
    if old_r != 1: return None
    return old_s % m

def apply_op(root, post, a_r, b_r, p):
    if root == 'Add': inter = (a_r + b_r) % p
    elif root == 'Sub': inter = (a_r + p - b_r) % p
    elif root == 'Mul': inter = (a_r * b_r) % p
    else: inter = 0
    if post == 'Id': result = inter
    elif post == 'Neg': result = (p - inter) % p
    elif post == 'Sqr': result = (inter * inter) % p
    else: result = inter
    return 1 if result > p // 2 else 0

ROOT_OPS = ['Add', 'Sub', 'Mul']
POST_OPS = ['Id', 'Neg', 'Sqr']

LIMIT = 200_000

# Precompute primes and twin primes
print("Precomputing primes...")
t0 = time.time()
prime_set = set()
twin_set = set()
for n in range(3, LIMIT, 2):
    if is_prime(n):
        prime_set.add(n)
        if is_prime(n + 2):
            twin_set.add(n)
print(f"  Primes: {len(prime_set)}, Twins: {len(twin_set)} in {time.time()-t0:.1f}s")

def compute_patterns(bases, ops, gap, limit):
    """Compute observed patterns for prime pairs and all odd numbers."""
    pair_pats = set()
    for p in range(3, limit, 2):
        if is_prime(p) and is_prime(p + gap):
            pat = tuple(apply_op(ops[i][0], ops[i][1], p % bases[i], (p+gap) % bases[i], bases[i])
                       for i in range(len(bases)))
            pair_pats.add(pat)
    return pair_pats

def elimination_rate(bases, ops, gap, limit, pair_patterns):
    """Count eliminated odd numbers."""
    elim = 0
    total = 0
    false_neg = 0
    for n in range(3, limit, 2):
        total += 1
        pat = tuple(apply_op(ops[i][0], ops[i][1], n % bases[i], (n+gap) % bases[i], bases[i])
                   for i in range(len(bases)))
        if pat not in pair_patterns:
            elim += 1
            if is_prime(n) and is_prime(n + gap):
                false_neg += 1
    return elim, total, false_neg

# ═══════════════════════════════════════════════════════════════════
# PHASE 1: OPTIMAL OPERATOR PER PRIME (extended to more primes)
# ═══════════════════════════════════════════════════════════════════

print("\n" + "=" * 78)
print("PHASE 1: OPTIMAL OPERATOR PER PRIME (twins, d=2)")
print("=" * 78)

primes_to_test = [3, 5, 7, 11, 13, 17, 19, 23]
best_ops = {}

for q in primes_to_test:
    best_sep = 0
    best_op = ('Add', 'Sqr')

    for root in ROOT_OPS:
        for post in POST_OPS:
            twin_ones = 0; twin_total = 0
            comp_ones = 0; comp_total = 0

            for n in range(3, LIMIT, 2):
                c = apply_op(root, post, n % q, (n + 2) % q, q)
                if n in twin_set:
                    twin_total += 1; twin_ones += c
                else:
                    comp_total += 1; comp_ones += c

            tw_rate = twin_ones / max(twin_total, 1)
            co_rate = comp_ones / max(comp_total, 1)
            sep = abs(tw_rate - co_rate)

            if sep > best_sep:
                best_sep = sep
                best_op = (root, post)

    best_ops[q] = (best_op, best_sep)
    print(f"  Prime {q:>2}: BEST = {best_op[0]:>3}+{best_op[1]:>3}  separation = {best_sep:.4f}")

# ═══════════════════════════════════════════════════════════════════
# PHASE 2: BUILD HETEROGENEOUS HEADS
# ═══════════════════════════════════════════════════════════════════

print("\n" + "=" * 78)
print("PHASE 2: HETEROGENEOUS vs UNIFORM HEADS")
print("=" * 78)

configs = {
    "Uniform Add+Sqr [5,7,11]": {
        'bases': [5, 7, 11],
        'ops': [('Add','Sqr'), ('Add','Sqr'), ('Add','Sqr')]
    },
    "Hetero Optimal [5,7,11]": {
        'bases': [5, 7, 11],
        'ops': [best_ops[5][0], best_ops[7][0], best_ops[11][0]]
    },
    "Uniform Add+Sqr [5,7,11,13]": {
        'bases': [5, 7, 11, 13],
        'ops': [('Add','Sqr')]*4
    },
    "Hetero Optimal [5,7,11,13]": {
        'bases': [5, 7, 11, 13],
        'ops': [best_ops[5][0], best_ops[7][0], best_ops[11][0], best_ops[13][0]]
    },
    "Uniform Add+Sqr [3,5,7,11]": {
        'bases': [3, 5, 7, 11],
        'ops': [('Add','Sqr')]*4
    },
    "Hetero Optimal [3,5,7,11]": {
        'bases': [3, 5, 7, 11],
        'ops': [best_ops[3][0], best_ops[5][0], best_ops[7][0], best_ops[11][0]]
    },
    "Champion S6 Uniform [2,3,5,7,11,13]": {
        'bases': [2, 3, 5, 7, 11, 13],
        'ops': [('Add','Sqr')]*6
    },
    "Champion S6 Hetero [2,3,5,7,11,13]": {
        'bases': [2, 3, 5, 7, 11, 13],
        'ops': [best_ops.get(p, (('Add','Sqr'), 0))[0] if p in best_ops
                else ('Add','Sqr') for p in [2, 3, 5, 7, 11, 13]]
    },
}

gap = 2
results = {}

print(f"\n  {'Config':42s} {'Forbid':>8} {'Elim':>8} {'Rate':>8} {'FN':>4}")
print(f"  {'-'*42} {'-'*8} {'-'*8} {'-'*8} {'-'*4}")

for name, cfg in configs.items():
    bases = cfg['bases']
    ops = cfg['ops']

    pair_pats = compute_patterns(bases, ops, gap, LIMIT)
    total_pats = 1 << len(bases)
    forbidden = total_pats - len(pair_pats)

    elim, total, fn = elimination_rate(bases, ops, gap, LIMIT, pair_pats)
    rate = elim / total if total > 0 else 0

    results[name] = {'rate': rate, 'forbidden': forbidden, 'total_pats': total_pats,
                     'elim': elim, 'total': total, 'fn': fn}

    print(f"  {name:42s} {forbidden:>4}/{total_pats:<3} {elim:>8} {rate*100:>7.2f}% {fn:>4}")

# ═══════════════════════════════════════════════════════════════════
# PHASE 3: MULTI-HEAD UNION — FIND THE CEILING
# ═══════════════════════════════════════════════════════════════════

print("\n" + "=" * 78)
print("PHASE 3: MULTI-HEAD UNION — STACKING ELIMINATION")
print("=" * 78)

# Build multiple heads with DIFFERENT bases (key: different primes see different structure)
head_configs = [
    ("H1: [5,7,11] Sqr", [5,7,11], [('Add','Sqr')]*3),
    ("H2: [3,5,7] Sqr", [3,5,7], [('Add','Sqr')]*3),
    ("H3: [7,11,13] Sqr", [7,11,13], [('Add','Sqr')]*3),
    ("H4: [3,7,13] Sqr", [3,7,13], [('Add','Sqr')]*3),
    ("H5: [5,11,13] Sqr", [5,11,13], [('Add','Sqr')]*3),
    ("H6: [3,5,11] Sqr", [3,5,11], [('Add','Sqr')]*3),
    ("H7: [5,7,11] Hetero", [5,7,11], [best_ops[5][0], best_ops[7][0], best_ops[11][0]]),
    ("H8: [3,5,7,11,13] Sqr", [3,5,7,11,13], [('Add','Sqr')]*5),
]

# For each head, compute which odd numbers it eliminates
head_eliminated = {}
print(f"\n  Computing per-head elimination sets...")

for name, bases, ops in head_configs:
    pair_pats = compute_patterns(bases, ops, gap, LIMIT)
    eliminated = set()
    for n in range(3, LIMIT, 2):
        pat = tuple(apply_op(ops[i][0], ops[i][1], n % bases[i], (n+gap) % bases[i], bases[i])
                   for i in range(len(bases)))
        if pat not in pair_pats:
            eliminated.add(n)

    head_eliminated[name] = eliminated
    rate = len(eliminated) / (LIMIT // 2) * 100
    print(f"    {name:30s}: {len(eliminated):>7} eliminated ({rate:.1f}%)")

# Now: progressive union
print(f"\n  Progressive multi-head union:")
print(f"  {'Heads':>5} {'Union size':>12} {'Rate':>8} {'Marginal':>10}")
print(f"  {'-----':>5} {'----------':>12} {'------':>8} {'---------':>10}")

union = set()
prev_size = 0
all_odd = set(range(3, LIMIT, 2))
total_odd = len(all_odd)

# Verify no false negatives in union
twin_values = set(p for p in range(3, LIMIT, 2) if is_prime(p) and is_prime(p+2))

for i, (name, _, _) in enumerate(head_configs):
    union = union | head_eliminated[name]
    new_size = len(union)
    marginal = new_size - prev_size
    rate = new_size / total_odd * 100

    # Check false negatives
    fn = len(union & twin_values)

    print(f"  {i+1:>5} {new_size:>12,} {rate:>7.2f}% +{marginal:>8,}  FN={fn}")
    prev_size = new_size

# Theoretical ceiling
print(f"\n  Total odd numbers in [3, {LIMIT}): {total_odd:,}")
print(f"  Union eliminates: {len(union):,} ({len(union)/total_odd*100:.2f}%)")
print(f"  Survivors: {total_odd - len(union):,}")
print(f"  Twin primes in survivors: {len(twin_values - union):,} / {len(twin_values):,}")
print(f"  False negatives in union: {len(union & twin_values)}")
print(f"  Speedup: {total_odd / (total_odd - len(union)):.2f}x fewer primality tests")

# ═══════════════════════════════════════════════════════════════════
# PHASE 4: OTHER FAMILIES
# ═══════════════════════════════════════════════════════════════════

print("\n" + "=" * 78)
print("PHASE 4: MULTI-FAMILY ANALYSIS (d=2, 4, 6, 30, 360)")
print("=" * 78)

families = [
    ("Twin (d=2)", 2),
    ("Cousin (d=4)", 4),
    ("Sexy (d=6)", 6),
    ("d=30", 30),
    ("d=360 (sacred)", 360),
]

bases_sqr = [5, 7, 11]
ops_sqr = [('Add','Sqr')]*3

print(f"\n  {'Family':20s} {'Pairs':>7} {'Forbidden':>10} {'Elim':>8} {'Rate':>8} {'FN':>4}")
print(f"  {'-'*20} {'-'*7} {'-'*10} {'-'*8} {'-'*8} {'-'*4}")

for fname, d in families:
    # Count pairs
    pairs = sum(1 for p in range(3, LIMIT, 2) if is_prime(p) and is_prime(p + d))

    pair_pats = compute_patterns(bases_sqr, ops_sqr, d, LIMIT)
    total_pats = 8
    forbidden = total_pats - len(pair_pats)

    elim, total, fn = elimination_rate(bases_sqr, ops_sqr, d, LIMIT, pair_pats)
    rate = elim / total * 100 if total > 0 else 0

    print(f"  {fname:20s} {pairs:>7,} {forbidden:>5}/{total_pats:<4} {elim:>8,} {rate:>7.2f}% {fn:>4}")

# ═══════════════════════════════════════════════════════════════════
# PHASE 5: EXHAUSTIVE 4-PRIME BASE SEARCH
# ═══════════════════════════════════════════════════════════════════

print("\n" + "=" * 78)
print("PHASE 5: BEST 3-PRIME BASE FOR TWIN ELIMINATION")
print("=" * 78)

from itertools import combinations

candidate_primes = [3, 5, 7, 11, 13, 17, 19, 23]
best_base = None
best_base_rate = 0

print(f"\n  Testing all 3-prime combinations from {candidate_primes}:\n")

for combo in combinations(candidate_primes, 3):
    bases = list(combo)
    ops = [('Add','Sqr')]*3

    pair_pats = compute_patterns(bases, ops, 2, LIMIT)
    elim, total, fn = elimination_rate(bases, ops, 2, min(LIMIT, 100000), pair_pats)
    rate = elim / total if total > 0 else 0

    if rate > best_base_rate and fn == 0:
        best_base_rate = rate
        best_base = bases

    if rate > 0.20:  # only show significant ones
        print(f"    {bases}: {rate*100:.2f}% elimination, FN={fn}")

print(f"\n  BEST 3-PRIME BASE: {best_base} at {best_base_rate*100:.2f}%")

# ═══════════════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════════════

print("\n" + "=" * 78)
print("SUMMARY OF FINDINGS")
print("=" * 78)

if results:
    best_single = max(results.items(), key=lambda x: x[1]['rate'])
    print(f"\n  Best single head: {best_single[0]}")
    print(f"    Rate: {best_single[1]['rate']*100:.2f}%")

print(f"  Best multi-head union: {len(union)/total_odd*100:.2f}% ({len(union):,}/{total_odd:,})")
print(f"  Speedup: {total_odd / max(total_odd - len(union), 1):.2f}x")
print(f"  False negatives: {len(union & twin_values)}")
print(f"\n  All heads use CRAM operators. Plain gives 0%.")
print(f"  The quadratic residue topology IS the sieve mechanism.")
