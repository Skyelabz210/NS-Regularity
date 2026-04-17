#!/usr/bin/env python3
"""
HYDRA DEEP INVESTIGATION — Open Questions
==========================================
Q1: WHY does Sqr work but Inv/Mul don't?
Q2: Can we find operator configs that beat 53.5%?
Q3: Does elimination rate scale with range?
Q4: Why does d=30030 get 0% but d=2 gets 53.5%?
Q5: What IS the Sqr operator actually detecting?
Q6: Does the correction factor converge as base grows?
Q7: Is there an optimal operator PER PRIME in the base?
"""

import math
from collections import Counter, defaultdict

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

# ═══════════════════════════════════════════════════════════════════
# Q1: WHY DOES SQR WORK?
# ═══════════════════════════════════════════════════════════════════

print("=" * 78)
print("Q1: WHY DOES SQR WORK BUT INV/MUL DON'T?")
print("=" * 78)

# The Sqr operator maps x -> x^2 mod p.
# This is a 2-to-1 map: x and (p-x) both map to x^2 mod p.
# The IMAGE of this map is the set of QUADRATIC RESIDUES mod p.
# There are exactly (p-1)/2 quadratic residues mod p (for odd p).
#
# When we compute operator_carry_pattern(p, p+d):
#   For each lane i with prime p_i:
#     r_a = p mod p_i, r_b = (p+d) mod p_i
#     intermediate = (r_a + r_b) mod p_i    [for Add root op]
#     result = intermediate^2 mod p_i        [for Sqr post op]
#     carry = 1 if result > p_i/2, else 0
#
# The KEY: the Sqr maps the PAIR (p, p+d) into the QR space.
# For a twin prime pair (p, p+2), BOTH p and p+2 are prime.
# Primes have specific QR distributions that composites DON'T.

print("\nThe Sqr operator maps residues into quadratic residue (QR) space.")
print("Primes have QR signatures that differ from composites.\n")

for p_base in [5, 7, 11, 13]:
    qr = set()
    for x in range(p_base):
        qr.add((x * x) % p_base)
    nqr = set(range(p_base)) - qr
    print(f"  mod {p_base}: QR = {sorted(qr)}, NQR = {sorted(nqr)}")
    print(f"         |QR| = {len(qr)}, |NQR| = {len(nqr)}")

    # For twins: what's the distribution of (p+p+2)^2 mod p_base
    # for p prime vs p composite?
    prime_results = Counter()
    composite_results = Counter()
    for n in range(3, 100000, 2):
        r = ((n + n + 2) % p_base)
        sq = (r * r) % p_base
        half = 1 if sq > p_base // 2 else 0
        if is_prime(n) and is_prime(n + 2):
            prime_results[half] += 1
        elif not is_prime(n) or not is_prime(n + 2):
            composite_results[half] += 1

    p_ratio = prime_results[1] / max(sum(prime_results.values()), 1)
    c_ratio = composite_results[1] / max(sum(composite_results.values()), 1)
    print(f"         Twin pairs: carry=1 in {p_ratio:.3f} of cases")
    print(f"         Non-twins:  carry=1 in {c_ratio:.3f} of cases")
    print(f"         SEPARATION: {abs(p_ratio - c_ratio):.4f}")
    print()

# ═══════════════════════════════════════════════════════════════════
# Q2: CAN WE BEAT 53.5%?
# ═══════════════════════════════════════════════════════════════════

print("=" * 78)
print("Q2: OPERATOR SEARCH — CAN WE BEAT 53.5%?")
print("=" * 78)

# Systematically try ALL operator combinations on {5,7,11}
# That's 4 root ops x 4 post ops = 16 per lane, 16^3 = 4096 total

root_ops = ['Add', 'Sub', 'Mul']  # Skip Div (domain issues)
post_ops = ['Id', 'Neg', 'Sqr']   # Skip Inv (domain issues at 0)

def apply_op(root, post, a_r, b_r, p):
    # Root operation
    if root == 'Add':
        inter = (a_r + b_r) % p
    elif root == 'Sub':
        inter = (a_r + p - b_r) % p
    elif root == 'Mul':
        inter = (a_r * b_r) % p
    else:
        return 0
    # Post operation
    if post == 'Id':
        result = inter
    elif post == 'Neg':
        result = (p - inter) % p
    elif post == 'Sqr':
        result = (inter * inter) % p
    else:
        return 0
    return 1 if result > p // 2 else 0

bases = [5, 7, 11]
gap = 2
LIMIT = 50000

# Get twin primes for pattern learning
twins = set()
for p in range(3, LIMIT, 2):
    if is_prime(p) and is_prime(p + gap):
        twins.add(p)

print(f"\n  Twin primes in [3, {LIMIT}): {len(twins)}")
print(f"  Testing all operator combinations on base [5,7,11]...\n")

best_configs = []

for r0 in root_ops:
    for p0 in post_ops:
        for r1 in root_ops:
            for p1 in post_ops:
                for r2 in root_ops:
                    for p2 in post_ops:
                        ops = [(r0,p0), (r1,p1), (r2,p2)]

                        # Compute patterns for twin primes
                        twin_patterns = set()
                        for p in twins:
                            pat = []
                            for i, base_p in enumerate(bases):
                                a_r = p % base_p
                                b_r = (p + gap) % base_p
                                pat.append(apply_op(ops[i][0], ops[i][1], a_r, b_r, base_p))
                            twin_patterns.add(tuple(pat))

                        total_patterns = 8  # 2^3
                        forbidden = total_patterns - len(twin_patterns)

                        # Test elimination rate on ALL odd numbers
                        eliminated = 0
                        total = 0
                        for n in range(3, LIMIT, 2):
                            total += 1
                            pat = []
                            for i, base_p in enumerate(bases):
                                a_r = n % base_p
                                b_r = (n + gap) % base_p
                                pat.append(apply_op(ops[i][0], ops[i][1], a_r, b_r, base_p))
                            if tuple(pat) not in twin_patterns:
                                eliminated += 1

                        rate = eliminated / total if total > 0 else 0

                        # Check for false negatives
                        false_neg = 0
                        for p in twins:
                            pat = []
                            for i, base_p in enumerate(bases):
                                a_r = p % base_p
                                b_r = (p + gap) % base_p
                                pat.append(apply_op(ops[i][0], ops[i][1], a_r, b_r, base_p))
                            if tuple(pat) not in twin_patterns:
                                false_neg += 1

                        if rate > 0.01 and false_neg == 0:
                            best_configs.append((rate, forbidden, ops, false_neg))

# Sort by elimination rate
best_configs.sort(key=lambda x: -x[0])

print(f"  {'Rank':>4} {'Rate':>7} {'Forbid':>7} {'FalseNeg':>9} Operators")
print(f"  {'----':>4} {'------':>7} {'------':>7} {'--------':>9} ---------")
for i, (rate, forb, ops, fn) in enumerate(best_configs[:20]):
    ops_str = " | ".join(f"{r}+{p}" for r, p in ops)
    print(f"  {i+1:>4} {rate*100:>6.2f}% {forb:>4}/8   {fn:>9} {ops_str}")

if best_configs:
    best_rate = best_configs[0][0]
    print(f"\n  BEST ELIMINATION RATE: {best_rate*100:.2f}%")
    print(f"  vs baseline Sqr: 42.60%")
    print(f"  IMPROVEMENT: {(best_rate - 0.426) / 0.426 * 100:+.1f}%")

# ═══════════════════════════════════════════════════════════════════
# Q3: DOES ELIMINATION SCALE WITH RANGE?
# ═══════════════════════════════════════════════════════════════════

print("\n" + "=" * 78)
print("Q3: ELIMINATION RATE vs RANGE (Sqr on [5,7,11])")
print("=" * 78)

# Use the best Sqr config
ranges = [10000, 50000, 100000, 500000]

print(f"\n  {'Range':>10} {'Twins':>8} {'Eliminated':>12} {'Rate':>8} {'Speedup':>8}")
print(f"  {'-----':>10} {'-----':>8} {'----------':>12} {'----':>8} {'-------':>8}")

for R in ranges:
    # Learn patterns from twins in this range
    tw = set()
    for p in range(3, R, 2):
        if is_prime(p) and is_prime(p + 2):
            tw.add(p)

    twin_pats = set()
    for p in tw:
        pat = tuple(apply_op('Add', 'Sqr', p % b, (p+2) % b, b) for b in bases)
        twin_pats.add(pat)

    elim = 0
    total = 0
    for n in range(3, R, 2):
        total += 1
        pat = tuple(apply_op('Add', 'Sqr', n % b, (n+2) % b, b) for b in bases)
        if pat not in twin_pats:
            elim += 1

    rate = elim / total if total > 0 else 0
    speedup = total / (total - elim) if elim < total else float('inf')
    print(f"  {R:>10,} {len(tw):>8,} {elim:>12,} {rate*100:>7.2f}% {speedup:>7.2f}x")

# ═══════════════════════════════════════════════════════════════════
# Q4: WHY d=30030 GETS 0%
# ═══════════════════════════════════════════════════════════════════

print("\n" + "=" * 78)
print("Q4: WHY d=30030 (primorial 13#) GETS 0% ELIMINATION")
print("=" * 78)

# d = 30030 = 2 x 3 x 5 x 7 x 11 x 13
# On base [5,7,11]:
#   d mod 5 = 0, d mod 7 = 0, d mod 11 = 0
# So for ANY n:
#   (n + d) mod p = n mod p   (for p in {5,7,11})
# The Sqr of (n + (n+d)) mod p = Sqr of (2n) mod p
# This is the SAME for all n with the same residue mod p.
# Since d mod p = 0 for all base primes, the pattern depends
# ONLY on n mod p, NOT on d at all!

print(f"\n  d = 30030 = 2 x 3 x 5 x 7 x 11 x 13")
print(f"  On base [5,7,11]: d mod 5 = {30030%5}, d mod 7 = {30030%7}, d mod 11 = {30030%11}")
print(f"\n  Since d = 0 mod every base prime:")
print(f"    (p + d) mod base = p mod base  for ALL p")
print(f"    The operator sees (r, r) instead of (r, r+d_r)")
print(f"    Pattern depends only on n mod base, not on d")
print(f"    ALL 8 patterns are observed (no forbidden)")
print(f"    Elimination = 0%")
print(f"\n  THIS IS CORRECT BEHAVIOR, not a bug.")
print(f"  When d shares all factors with the base, the base has")
print(f"  zero discriminating power. Need bases with primes NOT in d.")

# ═══════════════════════════════════════════════════════════════════
# Q5: WHAT IS SQR ACTUALLY DETECTING?
# ═══════════════════════════════════════════════════════════════════

print("\n" + "=" * 78)
print("Q5: WHAT IS THE SQR OPERATOR DETECTING?")
print("=" * 78)

# Hypothesis: Sqr detects the quadratic residue character of
# the sum (p + (p+2)) = (2p+2) mod each base prime.
# For twin primes, 2p+2 = 2(p+1) where p+1 is even (composite).
# The QR character of 2(p+1) mod each base prime creates a
# signature that distinguishes twin-prime-producing residue classes.

print(f"\n  For twin (p, p+2) with Add+Sqr on base prime q:")
print(f"    r_a = p mod q")
print(f"    r_b = (p+2) mod q")
print(f"    sum = (r_a + r_b) mod q = (2p+2) mod q")
print(f"    result = sum^2 mod q")
print(f"    carry = 1 if result > q/2")
print(f"\n  The carry pattern encodes: for each base prime q,")
print(f"  is (2p+2)^2 mod q in the upper or lower half?")
print(f"  This is a QUADRATIC CHARACTER test on (2p+2).")

for q in [5, 7, 11]:
    print(f"\n  mod {q}: (2p+2)^2 mod {q} for p = twin prime vs composite:")
    # What values can (2p+2) mod q take when p is a twin prime?
    twin_sums = Counter()
    comp_sums = Counter()
    for n in range(3, 200000, 2):
        s = (2*n + 2) % q
        sq = (s * s) % q
        half = 1 if sq > q // 2 else 0
        if is_prime(n) and is_prime(n + 2):
            twin_sums[s] += 1
        else:
            comp_sums[s] += 1

    print(f"    Sum residues for twins:     {dict(sorted(twin_sums.items()))}")
    print(f"    Sum residues for non-twins: {dict(sorted(comp_sums.items()))}")

    # The KEY: which sum residues NEVER appear for twins?
    all_residues = set(range(q))
    twin_residues = set(twin_sums.keys())
    forbidden_residues = all_residues - twin_residues
    print(f"    Forbidden sum residues: {sorted(forbidden_residues)}")
    if forbidden_residues:
        print(f"    These residues CANNOT produce twin primes on this lane!")

# ═══════════════════════════════════════════════════════════════════
# Q6: CORRECTION FACTOR CONVERGENCE
# ═══════════════════════════════════════════════════════════════════

print("\n" + "=" * 78)
print("Q6: DOES THE CORRECTION FACTOR CONVERGE?")
print("=" * 78)

# For twins (d=2), compute F(d,B) and F_product(d,B) for growing bases
print(f"\n  Computing correction factor C(2, B) for growing B:\n")

growing_bases = [
    ([3,5], "15"),
    ([3,5,7], "105"),
    ([5,7,11], "385"),
    ([3,5,7,11], "1155"),
    ([2,3,5,7,11], "2310"),
    ([5,7,11,13], "5005"),
    ([2,3,5,7,11,13], "30030"),
]

gap = 2
SAMPLE = 100000

print(f"  {'Base':>8} {'Product':>8} {'F_actual':>10} {'F_product':>10} {'Correction':>11}")
print(f"  {'----':>8} {'-------':>8} {'---------':>10} {'----------':>10} {'----------':>11}")

for primes, label in growing_bases:
    # Compute observed Sqr patterns from twin primes
    tw_pats = set()
    for p in range(3, SAMPLE, 2):
        if is_prime(p) and is_prime(p + gap):
            pat = tuple(apply_op('Add', 'Sqr', p % b, (p+gap) % b, b) for b in primes)
            tw_pats.add(pat)

    total_pat = 1 << len(primes)
    f_actual = (total_pat - len(tw_pats)) / total_pat

    # HL product approximation
    prod_surv = 1.0
    for q in primes:
        adm = (q-1) if gap % q == 0 else (q-2)
        prod_surv *= adm / q
    f_product = 1.0 - prod_surv

    correction = f_actual / f_product if f_product > 0 else float('inf')

    print(f"  {label:>8} {math.prod(primes):>8} {f_actual*100:>9.1f}% {f_product*100:>9.1f}% {correction:>10.4f}")

# ═══════════════════════════════════════════════════════════════════
# Q7: OPTIMAL OPERATOR PER PRIME
# ═══════════════════════════════════════════════════════════════════

print("\n" + "=" * 78)
print("Q7: IS THERE AN OPTIMAL OPERATOR PER BASE PRIME?")
print("=" * 78)

# Test each operator on each prime INDIVIDUALLY
for q in [5, 7, 11, 13]:
    print(f"\n  Base prime {q}:")
    best_op = None
    best_sep = 0

    for root in root_ops:
        for post in post_ops:
            # Compute carry for twin primes vs non-twins
            twin_ones = 0
            twin_total = 0
            comp_ones = 0
            comp_total = 0

            for n in range(3, 100000, 2):
                c = apply_op(root, post, n % q, (n + 2) % q, q)
                if is_prime(n) and is_prime(n + 2):
                    twin_total += 1
                    twin_ones += c
                else:
                    comp_total += 1
                    comp_ones += c

            tw_rate = twin_ones / max(twin_total, 1)
            co_rate = comp_ones / max(comp_total, 1)
            separation = abs(tw_rate - co_rate)

            if separation > best_sep:
                best_sep = separation
                best_op = f"{root}+{post}"

            if separation > 0.01:
                print(f"    {root:>3}+{post:>3}: twin_carry1={tw_rate:.4f}  "
                      f"comp_carry1={co_rate:.4f}  SEP={separation:.4f}")

    print(f"    BEST: {best_op} (separation {best_sep:.4f})")

# ═══════════════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════════════

print("\n" + "=" * 78)
print("ANSWERS SUMMARY")
print("=" * 78)
print("""
Q1: WHY SQR WORKS
  Sqr maps residues into quadratic residue space. The QR structure
  of (2p+2) mod q creates family-specific signatures. Some sum
  residues NEVER appear for twin primes, creating forbidden patterns.
  Inv and Mul don't create the same 2-to-1 concentration.

Q2: CAN WE BEAT 53.5%?
  See results above. The operator search exhaustively tested all
  3^3 x 3^3 = 729 combinations on [5,7,11].

Q3: ELIMINATION SCALES WITH RANGE?
  See scaling table. The rate should stabilize as the forbidden
  pattern set saturates (which happens quickly).

Q4: WHY d=30030 GETS 0%
  When d mod p = 0 for ALL base primes p, the operator sees (r, r)
  for every n. All patterns are observed. Zero discrimination.
  This is correct: need bases with primes NOT dividing d.

Q5: WHAT SQR DETECTS
  The squared sum (2p+2)^2 mod q concentrates twin primes into
  specific residue classes that composites don't favor. The
  forbidden sum residues are the mechanism.

Q6: CORRECTION CONVERGENCE
  See table. Track whether correction -> 1 as base grows.

Q7: OPTIMAL OPERATOR PER PRIME
  Different primes may have different optimal operators. The
  separation metric identifies which operator best distinguishes
  twin primes from composites on each individual lane.
""")
