# Recombinant CRT Bridge v3 - Constraints Reference

## Hard Mathematical Constraints

### C1: Prime Separation
Exact lane primes (> 2^62) must be distinct from Safe Basis primes (<= 13)
and from anchor primes. Guaranteed by construction (size difference).

### C2: Sub-Lane Coprimality
All sub-lanes within a TriadValue must have pairwise coprime moduli.
Since each sub-lane uses a distinct prime base from {2,3,5,7,11,13},
this is automatic. Violation occurs only if the SAME prime base appears
in both exact lanes and approx sub-lanes (e.g., exact mod-2 + approx mod-4).
Resolution: use large-prime exact lanes (standard CRTBigInt).

### C3: Range Bound
Reconstructed value X = r + k*m must satisfy 0 <= X < m*a for K-Elimination
to return a unique result. The anchor provides range verification: k < P.

### C4: Odd Modulus for Montgomery
Montgomery reduction requires odd modulus. All Safe Basis primes except 2
are odd. The mod-2 sub-lane uses direct reduction, not Montgomery.

### C5: Quantization Irreversibility
Once an approximation is quantized into sub-lanes, the original float
cannot be recovered. The frozen epsilon records the quantization bound.
This is by design: the value is now an exact CRT resident.

### C6: Anchor Primality
The Safe Anchor P must be prime (not just coprime to M_safe). Primality
guarantees coprimality with ALL Safe Basis composites regardless of
exponent configuration. A composite anchor could share a factor with
a sub-lane if the configuration changes.

## The Prime-13 Boundary

Empirical Phase 3c data (N <= 500,000):

| Configuration | Cells | Coverage | Status |
|---------------|-------|----------|--------|
| 3x5x7x11x13 quintuple | 5,760 | 100.00% | SATURATED |
| All quads from {3,5,7,11,13} | 480-2,880 | 100.00% | SATURATED |
| 7x11x13x17 (with 17) | 11,520 | 99.32% | NEAR |
| 5x7x11x13x17 (no mod-3) | 46,080 | 69.97% | INCOMPLETE |

Saturation criterion: pi(N) >> product(p_i - 1).
5-prime basis: threshold N ~ 24,000, margin 21.2x at N=500k.
6-prime basis: threshold N ~ 363,000, margin 1.4x at N=500k.
Recommendation: extend testing to N = 5,000,000 for 12x margin.

## Engineering Constraints

### E1: Garner Overflow
Garner reconstruction across all 8 exact lanes produces values up to ~2^504.
The u128 round-trip handles values up to ~2^127. For larger reconstructions,
use the InfiniteScaffold cascading K-Elimination instead.

### E2: Sign Handling
CRTBigInt stores sign as i8 (-1, 0, 1) with unsigned residues.
Negative values: residues = modulus - |value| mod modulus.
K-Elimination operates on unsigned residues; sign tracked separately.

### E3: Duplicate Sub-Lane Bases
If two absorptions produce sub-lanes with the same prime base (e.g., two
denominators both containing factor 2), the exponents should be merged
via LCM. Current implementation: user responsibility. Future: auto-merge.

### E4: Error Bound Conservatism
Multiplication error bound (eps_a*|b| + eps_b*|a| + eps_a*eps_b) is
worst-case. For small epsilon relative to operands, the cross-term
eps_a*eps_b is negligible.

### E5: WASSAN Band Width
Band bits = ceil(total_bits / 144). For non-power-of-2 totals, the last
band may be partially filled. Reconstruction handles this correctly.
Band-domain mod requires base = 2^band_bits mod M, computed once.

## Open Problems

### O1: Formal Phase-Lock Proof for 6-Prime Basis
Status: EMPIRICAL (N=500k). Path: Bombieri-Vinogradov or Hardy-Littlewood.

### O2: NS Discrete Vector Calculus on CRTBigInt Grid
Status: NOT STARTED. Requires exact div/grad/curl/Laplacian on integer grid.

### O3: Continuum Limit Correspondence
Status: THEORETICAL GAP. Discrete-to-continuous CRT bridge theorem needed.

### O4: Automatic Sub-Lane Merging
Status: ENGINEERING. LCM-based merge in TriadValue::absorb.

### O5: WASSAN Band-Domain Multiplication
Status: PARTIAL. Convolution across bands works; NTT optimization needed.

### O6: Double Helix Integration with Recombinant Bridge
Status: ARCHITECTURAL. Wire helix strands to WASSAN bands for per-band
dual verification.

### O7: Extended Summit K-Elimination Testing
Status: VERIFIED at 0.20s for value < M_safe. Stress test with k > 0 needed
for values exceeding the Safe Basis composite capacity.
