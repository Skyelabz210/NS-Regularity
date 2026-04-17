#!/usr/bin/env python3
"""
EXPEDITION FOLLOW-UP: Testable Predictions from the Breakdown Map
==================================================================

The expedition identified 7 fundamental breakdowns of the decomposition
principle. Each breakdown generates testable predictions about the QMNF/CRAM
system. This script tests the ones we can test NOW.

Prediction 1 (Hasse principle): Cubic post-processors should behave
qualitatively differently from quadratic ones. The Sqr post-processor
(degree 2) works because quadratic forms satisfy Hasse-Minkowski.
A Cube post-processor (degree 3) should show anomalous behavior
because cubic forms violate the Hasse principle.

Prediction 2 (Resurgence): The "dead" carry interaction content should
contain information about the non-perturbative sector of the sieve —
specifically, information about EXCEPTIONAL prime pairs that violate
the statistical prediction. Carry patterns that are "empty" at leading
order may correlate with rare prime pair configurations.

Prediction 3 (Phase transition / RG flow): The correction factor
convergence (C -> 1 as basis grows) should exhibit critical exponents
characteristic of a renormalization group fixed point.

Prediction 4 (Entanglement via phase-lock): Phase-locked CRT lanes
should be able to encode correlations that independent lanes cannot —
specifically, the phase-lock algebra should be rich enough to represent
Bell-type correlations.

Prediction 5 (Anderson localization): Introducing random perturbations
to operator assignments should cause a localization transition where
sieve power collapses above a critical disorder threshold.
"""

import math
import time
from collections import Counter

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
    else: inter = 0
    if post == 'Id': result = inter
    elif post == 'Neg': result = (p - inter) % p
    elif post == 'Sqr': result = (inter * inter) % p
    elif post == 'Cube': result = (inter * inter % p * inter) % p
    elif post == 'Quartic': result = pow(inter, 4, p)
    else: result = inter
    return 1 if result > p // 2 else 0

PRIMES = [2, 3, 5, 7, 11, 13]
LIMIT = 100_000

# ═══════════════════════════════════════════════════════════════
# PREDICTION 1: HASSE PRINCIPLE — Cubic vs Quadratic post-ops
# ═══════════════════════════════════════════════════════════════

print("=" * 70)
print("PREDICTION 1: HASSE PRINCIPLE TEST")
print("  Quadratic forms satisfy Hasse-Minkowski (local -> global works)")
print("  Cubic forms violate it (Selmer 1951)")
print("  Prediction: Cube post-op should show anomalous separation")
print("=" * 70)

# Precompute twin pairs
twin_set = set()
for p in range(3, LIMIT, 2):
    if is_prime(p) and is_prime(p + 2):
        twin_set.add(p)

print(f"\nTwin primes in [3, {LIMIT}): {len(twin_set)}")

# Test each post-op at each degree
post_ops = ['Id', 'Neg', 'Sqr', 'Cube', 'Quartic']
degrees = {'Id': 1, 'Neg': 1, 'Sqr': 2, 'Cube': 3, 'Quartic': 4}

print(f"\n  {'Post-Op':>8} {'Deg':>4}  ", end="")
for q in [3, 5, 7, 11, 13]:
    print(f"{'p='+str(q):>8}", end="")
print(f"  {'Avg Sep':>8}")
print(f"  {'-'*8} {'-'*4}  " + f"{'-'*8}" * 5 + f"  {'-'*8}")

for post in post_ops:
    deg = degrees[post]
    seps = []
    print(f"  {post:>8} {deg:>4}  ", end="")

    for q in [3, 5, 7, 11, 13]:
        twin_ones = 0; twin_total = 0
        comp_ones = 0; comp_total = 0

        for n in range(3, LIMIT, 2):
            c = apply_op('Add', post, n % q, (n + 2) % q, q)
            if n in twin_set:
                twin_total += 1; twin_ones += c
            else:
                comp_total += 1; comp_ones += c

        tw_rate = twin_ones / max(twin_total, 1)
        co_rate = comp_ones / max(comp_total, 1)
        sep = abs(tw_rate - co_rate)
        seps.append(sep)
        print(f"  {sep:>7.4f}", end="")

    avg = sum(seps) / len(seps)
    print(f"  {avg:>8.4f}")

# Now test with Mul root (our optimal for p=3)
print(f"\n  With Mul root (optimal for p=3):")
print(f"  {'Post-Op':>8} {'Deg':>4}  ", end="")
for q in [3, 5, 7, 11, 13]:
    print(f"{'p='+str(q):>8}", end="")
print(f"  {'Avg Sep':>8}")
print(f"  {'-'*8} {'-'*4}  " + f"{'-'*8}" * 5 + f"  {'-'*8}")

for post in post_ops:
    deg = degrees[post]
    seps = []
    print(f"  {post:>8} {deg:>4}  ", end="")

    for q in [3, 5, 7, 11, 13]:
        twin_ones = 0; twin_total = 0
        comp_ones = 0; comp_total = 0

        for n in range(3, LIMIT, 2):
            c = apply_op('Mul', post, n % q, (n + 2) % q, q)
            if n in twin_set:
                twin_total += 1; twin_ones += c
            else:
                comp_total += 1; comp_ones += c

        tw_rate = twin_ones / max(twin_total, 1)
        co_rate = comp_ones / max(comp_total, 1)
        sep = abs(tw_rate - co_rate)
        seps.append(sep)
        print(f"  {sep:>7.4f}", end="")

    avg = sum(seps) / len(seps)
    print(f"  {avg:>8.4f}")

# Check: does gcd(n, p-1) predict gain differently for n=2 vs n=3?
print(f"\n  T7-2 Prediction: |F_eff| depends on gcd(n, p-1)")
print(f"  {'Prime':>6} {'p-1':>6} {'gcd(2,p-1)':>12} {'gcd(3,p-1)':>12} {'gcd(4,p-1)':>12}")
for q in [3, 5, 7, 11, 13]:
    print(f"  {q:>6} {q-1:>6} {gcd(2,q-1):>12} {gcd(3,q-1):>12} {gcd(4,q-1):>12}")

# ═══════════════════════════════════════════════════════════════
# PREDICTION 3: RG FLOW — Correction factor critical exponents
# ═══════════════════════════════════════════════════════════════

print(f"\n{'='*70}")
print("PREDICTION 3: RENORMALIZATION GROUP FLOW TEST")
print("  If correction factor C(N) -> 1 is an RG fixed point approach,")
print("  then |1 - C(N)| should decay as a power law in N (or log N).")
print("=" * 70)

# Compute correction factors at increasing basis sizes
from itertools import combinations

bases_and_products = []
for k in range(2, 7):
    for combo in combinations([3, 5, 7, 11, 13], k):
        primes = list(combo)
        M = 1
        for p in primes: M *= p
        bases_and_products.append((primes, M))

# Sort by product
bases_and_products.sort(key=lambda x: x[1])

# For each basis, compute: theoretical forbidden fraction vs actual elimination
print(f"\n  {'Basis':>20} {'Product':>8} {'Theo Forb':>10} {'Actual Elim':>12} {'C = Act/Theo':>12} {'|1-C|':>8}")
print(f"  {'-'*20} {'-'*8} {'-'*10} {'-'*12} {'-'*12} {'-'*8}")

corrections = []
for primes, M in bases_and_products[:15]:
    # Theoretical: 1 - prod((p-2)/p) for p not dividing d=2
    prod_surv = 1.0
    for p in primes:
        adm = (p-1) if 2 % p == 0 else (p-2)
        prod_surv *= adm / p
    theo_forb = 1.0 - prod_surv

    # Actual: measure elimination with uniform Add+Sqr
    ops = [('Add', 'Sqr')] * len(primes)
    pair_pats = set()
    for n in range(3, 50000, 2):
        if is_prime(n) and is_prime(n + 2):
            pat = tuple(apply_op(ops[i][0], ops[i][1], n % primes[i],
                       (n+2) % primes[i], primes[i]) for i in range(len(primes)))
            pair_pats.add(pat)

    total_pats = 1 << len(primes)
    forbidden = total_pats - len(pair_pats)
    actual_forb = forbidden / total_pats

    # Measure actual elimination rate
    elim = 0; total = 0
    for n in range(3, 50000, 2):
        total += 1
        pat = tuple(apply_op(ops[i][0], ops[i][1], n % primes[i],
                   (n+2) % primes[i], primes[i]) for i in range(len(primes)))
        if pat not in pair_pats:
            elim += 1

    actual_rate = elim / total if total > 0 else 0
    C = actual_rate / theo_forb if theo_forb > 0 else 0
    residual = abs(1 - C)

    corrections.append((M, C, residual))
    label = str(primes)
    if len(label) > 20: label = label[:17] + "..."
    print(f"  {label:>20} {M:>8} {theo_forb:>10.4f} {actual_rate:>12.4f} {C:>12.4f} {residual:>8.4f}")

# Fit power law: |1-C| ~ M^(-alpha)
import math
if len(corrections) >= 3:
    # Simple log-log fit
    log_M = [math.log(c[0]) for c in corrections if c[2] > 0]
    log_r = [math.log(c[2]) for c in corrections if c[2] > 0]
    if len(log_M) >= 2:
        # Least squares: log_r = alpha * log_M + beta
        n = len(log_M)
        sx = sum(log_M); sy = sum(log_r)
        sxx = sum(x*x for x in log_M)
        sxy = sum(x*y for x, y in zip(log_M, log_r))
        alpha = (n * sxy - sx * sy) / (n * sxx - sx * sx) if (n * sxx - sx * sx) != 0 else 0
        beta = (sy - alpha * sx) / n
        print(f"\n  Power law fit: |1-C| ~ M^{alpha:.3f}")
        print(f"  If alpha < 0, C converges to 1 as M grows (RG fixed point)")
        print(f"  alpha = {alpha:.4f} ({'converges' if alpha < 0 else 'diverges or flat'})")

# ═══════════════════════════════════════════════════════════════
# PREDICTION 5: ANDERSON LOCALIZATION — Disorder kills sieve
# ═══════════════════════════════════════════════════════════════

print(f"\n{'='*70}")
print("PREDICTION 5: ANDERSON LOCALIZATION TEST")
print("  Introduce random operator perturbations to the optimal topology.")
print("  Prediction: above a critical disorder threshold, sieve power")
print("  collapses to zero (localization transition).")
print("=" * 70)

import random
random.seed(42)

OPTIMAL_TWIN = [('Add','Id'), ('Mul','Id'), ('Add','Sqr'), ('Add','Sqr'), ('Mul','Neg'), ('Mul','Id')]
ROOT_OPS = ['Add', 'Sub', 'Mul']
POST_OPS = ['Id', 'Neg', 'Sqr']

def perturb_topology(base_ops, disorder_prob):
    """With probability disorder_prob, replace each lane's operator with random."""
    result = []
    for root, post in base_ops:
        if random.random() < disorder_prob:
            root = random.choice(ROOT_OPS)
            post = random.choice(POST_OPS)
        result.append((root, post))
    return result

def measure_elimination(ops, gap, limit):
    pair_pats = set()
    for p in range(3, limit, 2):
        if is_prime(p) and is_prime(p + gap):
            pat = tuple(apply_op(ops[i][0], ops[i][1], p % PRIMES[i],
                       (p+gap) % PRIMES[i], PRIMES[i]) for i in range(len(PRIMES)))
            pair_pats.add(pat)
    elim = 0; total = 0
    for n in range(3, limit, 2):
        total += 1
        pat = tuple(apply_op(ops[i][0], ops[i][1], n % PRIMES[i],
                   (n+gap) % PRIMES[i], PRIMES[i]) for i in range(len(PRIMES)))
        if pat not in pair_pats:
            elim += 1
    return elim / total if total > 0 else 0

print(f"\n  {'Disorder':>10} {'Trials':>8} {'Mean Elim':>10} {'Std':>8} {'Min':>8} {'Max':>8}")
print(f"  {'-'*10} {'-'*8} {'-'*10} {'-'*8} {'-'*8} {'-'*8}")

disorder_levels = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.8, 1.0]
TRIALS = 10
MEAS_LIMIT = 50000

for d in disorder_levels:
    rates = []
    for _ in range(TRIALS):
        ops = perturb_topology(OPTIMAL_TWIN, d)
        rate = measure_elimination(ops, 2, MEAS_LIMIT)
        rates.append(rate)
    mean_r = sum(rates) / len(rates)
    std_r = (sum((r - mean_r)**2 for r in rates) / len(rates)) ** 0.5
    print(f"  {d:>10.1f} {TRIALS:>8} {mean_r*100:>9.2f}% {std_r*100:>7.2f}% {min(rates)*100:>7.2f}% {max(rates)*100:>7.2f}%")

# ═══════════════════════════════════════════════════════════════
# PREDICTION 4: PHASE-LOCK BELL CORRELATIONS
# ═══════════════════════════════════════════════════════════════

print(f"\n{'='*70}")
print("PREDICTION 4: PHASE-LOCK ENCODES CORRELATIONS")
print("  Test whether phase-locked lanes can encode correlations")
print("  that independent lanes cannot — Bell-like test.")
print("=" * 70)

# Bell-CHSH inequality: for classical (independent) systems,
# |E(a,b) - E(a,b') + E(a',b) + E(a',b')| <= 2
# Quantum violation: up to 2*sqrt(2) ~ 2.828

# Can CRT phase-locked lanes violate CHSH?
# Model: two CRT lanes (mod p1, mod p2), with phase-lock constraint
# that r1 + r2 = K mod lcm(p1, p2)

# "Measurement" = threshold test (carry bit) at different operator settings
# a, a' = two operator choices for lane 1
# b, b' = two operator choices for lane 2

def chsh_test(p1, p2, lock_const, ops_a, ops_a2, ops_b, ops_b2, samples=10000):
    """Test CHSH inequality on phase-locked CRT lanes."""
    def measure(r1, r2, op1, op2):
        # Apply operators and return +1 or -1
        v1 = apply_op(op1[0], op1[1], r1, r1, p1)  # self-interaction
        v2 = apply_op(op2[0], op2[1], r2, r2, p2)
        return (2*v1 - 1) * (2*v2 - 1)  # map {0,1} to {-1,+1}

    # Generate phase-locked pairs: r1 + r2 = K mod (p1*p2)
    pairs = []
    for r1 in range(p1):
        for r2 in range(p2):
            if (r1 + r2) % min(p1, p2) == lock_const % min(p1, p2):
                pairs.append((r1, r2))

    if not pairs:
        return 0.0

    E_ab = sum(measure(r1, r2, ops_a, ops_b) for r1, r2 in pairs) / len(pairs)
    E_ab2 = sum(measure(r1, r2, ops_a, ops_b2) for r1, r2 in pairs) / len(pairs)
    E_a2b = sum(measure(r1, r2, ops_a2, ops_b) for r1, r2 in pairs) / len(pairs)
    E_a2b2 = sum(measure(r1, r2, ops_a2, ops_b2) for r1, r2 in pairs) / len(pairs)

    S = abs(E_ab - E_ab2 + E_a2b + E_a2b2)
    return S

# Test various operator combinations on phase-locked lanes
print(f"\n  CHSH bound for classical (independent): S <= 2.0")
print(f"  Quantum violation: S up to 2*sqrt(2) = {2*math.sqrt(2):.4f}")
print(f"\n  {'p1':>4} {'p2':>4} {'Lock':>5} {'Ops':>35} {'S':>8} {'Violates?':>10}")
print(f"  {'-'*4} {'-'*4} {'-'*5} {'-'*35} {'-'*8} {'-'*10}")

op_combos = [
    (('Add','Id'), ('Add','Sqr'), ('Mul','Id'), ('Mul','Sqr')),
    (('Add','Sqr'), ('Sub','Sqr'), ('Mul','Id'), ('Mul','Neg')),
    (('Mul','Id'), ('Mul','Sqr'), ('Add','Id'), ('Add','Neg')),
    (('Sub','Id'), ('Sub','Sqr'), ('Mul','Neg'), ('Add','Sqr')),
]

max_S = 0
for p1, p2 in [(5,7), (7,11), (11,13), (5,13), (7,13)]:
    for lock in range(min(p1, p2)):
        for ops in op_combos:
            S = chsh_test(p1, p2, lock, ops[0], ops[1], ops[2], ops[3])
            if S > max_S:
                max_S = S
            if S > 1.8:  # only show interesting ones
                label = f"{ops[0][0][:1]}+{ops[0][1][:1]},{ops[1][0][:1]}+{ops[1][1][:1]}|{ops[2][0][:1]}+{ops[2][1][:1]},{ops[3][0][:1]}+{ops[3][1][:1]}"
                print(f"  {p1:>4} {p2:>4} {lock:>5} {label:>35} {S:>8.4f} {'YES' if S > 2.0 else 'no':>10}")

print(f"\n  Maximum S observed: {max_S:.4f}")
print(f"  Classical bound: 2.0000")
print(f"  {'PHASE-LOCKED LANES CAN VIOLATE CHSH' if max_S > 2.0 else 'No CHSH violation observed'}")
if max_S <= 2.0:
    print(f"  (Phase-lock creates correlations but may not reach quantum violation level)")
    print(f"  (This is an honest negative if confirmed — CRT correlations are classical)")

# ═══════════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════════

print(f"\n{'='*70}")
print("EXPEDITION FOLLOW-UP SUMMARY")
print("=" * 70)
print(f"""
  TESTED:
    P1 (Hasse): Cubic vs quadratic post-ops — separation table computed
    P3 (RG):    Correction factor power-law fit — alpha computed
    P4 (Bell):  CHSH test on phase-locked CRT lanes — max S computed
    P5 (Anderson): Disorder vs sieve power — transition mapped

  UNTESTED (requires deeper infrastructure):
    P2 (Resurgence): Carry patterns encoding non-perturbative content
    P6 (Holographic): K-Elim as dimensional reduction vs AdS/CFT
    P7 (Gödel): Self-referential statements about QMNF system

  Each result either confirms or honestly negates a prediction
  derived from the cross-disciplinary expedition. Both outcomes
  are valuable — the principle's boundaries are as informative
  as its successes.
""")
