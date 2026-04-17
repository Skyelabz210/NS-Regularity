#!/usr/bin/env python3
"""
Compute optimal CRAM operator per prime for each family.
Output: exact operator tables to wire into Rust.
"""
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

def apply_op(root, post, a_r, b_r, p):
    if root == 'Add': inter = (a_r + b_r) % p
    elif root == 'Sub': inter = (a_r + p - b_r) % p
    elif root == 'Mul': inter = (a_r * b_r) % p
    else: return 0
    if post == 'Id': result = inter
    elif post == 'Neg': result = (p - inter) % p
    elif post == 'Sqr': result = (inter * inter) % p
    else: result = inter
    return 1 if result > p // 2 else 0

ROOT_OPS = ['Add', 'Sub', 'Mul']
POST_OPS = ['Id', 'Neg', 'Sqr']
LIMIT = 200_000
PRIMES = [2, 3, 5, 7, 11, 13]

families = [
    ("Twin", 2, lambda p,d: is_prime(p) and is_prime(p+d)),
    ("Cousin", 4, lambda p,d: is_prime(p) and is_prime(p+d)),
    ("Sexy", 6, lambda p,d: is_prime(p) and is_prime(p+d)),
    ("d=30", 30, lambda p,d: is_prime(p) and is_prime(p+d)),
    ("d=210", 210, lambda p,d: is_prime(p) and is_prime(p+d)),
    ("d=360", 360, lambda p,d: is_prime(p) and is_prime(p+d)),
]

print("Computing optimal operators per prime per family...")
print(f"Range: [3, {LIMIT}), Primes in base: {PRIMES}\n")

# Results table: family -> prime -> (best_root, best_post, separation)
all_results = {}

for fname, gap, pair_test in families:
    t0 = time.time()
    # Precompute pair set
    pair_set = set()
    for n in range(3, LIMIT, 2):
        if pair_test(n, gap):
            pair_set.add(n)

    family_ops = {}
    for q in PRIMES:
        best_sep = -1
        best_op = ('Add', 'Sqr')
        for root in ROOT_OPS:
            for post in POST_OPS:
                pair_ones = 0; pair_total = 0
                comp_ones = 0; comp_total = 0
                for n in range(3, LIMIT, 2):
                    c = apply_op(root, post, n % q, (n + gap) % q, q)
                    if n in pair_set:
                        pair_total += 1; pair_ones += c
                    else:
                        comp_total += 1; comp_ones += c
                tw_r = pair_ones / max(pair_total, 1)
                co_r = comp_ones / max(comp_total, 1)
                sep = abs(tw_r - co_r)
                if sep > best_sep:
                    best_sep = sep
                    best_op = (root, post)
        family_ops[q] = (best_op, best_sep)

    all_results[fname] = family_ops
    elapsed = time.time() - t0
    print(f"  {fname:8s} ({len(pair_set):>5} pairs, {elapsed:.1f}s):")
    for q in PRIMES:
        op, sep = family_ops[q]
        print(f"    p={q:>2}: {op[0]:>3}+{op[1]:>3}  sep={sep:.4f}")

# Now compute elimination rates for hetero vs uniform on each family
print("\n" + "=" * 70)
print("ELIMINATION RATES: HETERO OPTIMAL vs UNIFORM SQR")
print("=" * 70)

print(f"\n  {'Family':10s} {'Uniform Sqr':>12} {'Hetero Opt':>12} {'Gain':>8}")
print(f"  {'-'*10} {'-'*12} {'-'*12} {'-'*8}")

for fname, gap, pair_test in families:
    pair_set = set()
    for n in range(3, LIMIT, 2):
        if pair_test(n, gap):
            pair_set.add(n)

    # Uniform Sqr
    def get_rate(ops_list):
        pair_pats = set()
        for p in pair_set:
            pat = tuple(apply_op(ops_list[i][0], ops_list[i][1],
                       p % PRIMES[i], (p+gap) % PRIMES[i], PRIMES[i])
                       for i in range(len(PRIMES)))
            pair_pats.add(pat)
        elim = 0; total = 0; fn = 0
        for n in range(3, LIMIT, 2):
            total += 1
            pat = tuple(apply_op(ops_list[i][0], ops_list[i][1],
                       n % PRIMES[i], (n+gap) % PRIMES[i], PRIMES[i])
                       for i in range(len(PRIMES)))
            if pat not in pair_pats:
                elim += 1
                if n in pair_set: fn += 1
        return elim / total if total > 0 else 0, fn

    uniform_ops = [('Add','Sqr')]*6
    hetero_ops = [all_results[fname][q][0] for q in PRIMES]

    u_rate, u_fn = get_rate(uniform_ops)
    h_rate, h_fn = get_rate(hetero_ops)
    gain = h_rate - u_rate

    print(f"  {fname:10s} {u_rate*100:>11.2f}% {h_rate*100:>11.2f}% {gain*100:>+7.2f}%  "
          f"FN={u_fn},{h_fn}")

# Generate Rust code
print("\n" + "=" * 70)
print("RUST OPERATOR TABLES (copy into hydra-sieve)")
print("=" * 70)

rust_map = {'Add': 'RootOp::Add', 'Sub': 'RootOp::Sub', 'Mul': 'RootOp::Mul'}
rust_post = {'Id': 'PostOp::Id', 'Neg': 'PostOp::Neg', 'Sqr': 'PostOp::Sqr'}

for fname, gap, _ in families:
    ops = all_results[fname]
    print(f"\n    // {fname} (d={gap}) — heterogeneous optimal")
    print(f"    // Primes: {PRIMES}")
    ops_str = ", ".join(f"({rust_map[ops[q][0][0]]}, {rust_post[ops[q][0][1]]})"
                       for q in PRIMES)
    seps = ", ".join(f"{ops[q][1]:.3f}" for q in PRIMES)
    print(f"    // Separations: [{seps}]")
    print(f"    vec![{ops_str}]")
