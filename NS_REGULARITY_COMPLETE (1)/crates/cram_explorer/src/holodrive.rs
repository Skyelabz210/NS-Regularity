//! # WASSAN Holodrive — Frequency-Band Convergence Scanner
//!
//! The key insight: K_cap (2-adic valuation) is a FREE DIMENSION.
//! Every state (r, v2) with v2 > 0 is just v2 even steps away from (r, 0).
//! Even steps don't change the odd-part residue mod P_odd.
//!
//! So convergence of (r_odd, v2) = convergence of (r_odd, 0).
//! The K_cap multiplier EVAPORATES.
//!
//! Instead of enumerating P_odd × 2^k states, we enumerate P_odd
//! odd-part residues. The entire convergence map fits in a bitset
//! of P_odd/8 bytes — 592KB for P=4,849,845.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │  HOLODRIVE BITSET (1 bit per odd-part residue) │
//! │  [0][1][1][1][0][1][1][1][1][1][1][1]...    │
//! │   ↑                                          │
//! │   spurious (didn't reach 1 in P_odd steps)  │
//! └─────────────────────────────────────────────┘
//!          ↓
//! ┌─────────────────────────────────────────────┐
//! │  CURING PRIME DETECTOR                       │
//! │  For each spurious r, lift to r + j*P_odd    │
//! │  for j = 0..p_next-1. If ALL lifts converge  │
//! │  → p_next CURES the degeneracy.             │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! ## Generalized Boundary Prime Theorem
//!
//! Each primorial level has a "curing prime" that resolves its
//! spurious degeneracies:
//!   - Level {3,5,7,11}: curing prime = 13
//!   - Level {3,5,7,11,13,17,19}: curing prime = 23
//!
//! Spurious cycles are SIGNPOSTS pointing at the next necessary prime.

use crate::mod_arith::*;
use crate::torus::*;
use crate::k_elim::*;

// ============================================================
// BITSET — compact convergence tracker
// ============================================================

/// Compact bitset for convergence tracking.
/// One bit per odd-part residue. P_odd/8 bytes total.
pub struct HoloBitset {
    bits: Vec<u8>,
    size: usize,
}

impl HoloBitset {
    pub fn new(size: usize) -> Self {
        let bytes = (size + 7) / 8;
        HoloBitset { bits: vec![0u8; bytes], size }
    }

    #[inline(always)]
    pub fn set(&mut self, idx: usize) {
        debug_assert!(idx < self.size);
        self.bits[idx >> 3] |= 1 << (idx & 7);
    }

    #[inline(always)]
    pub fn get(&self, idx: usize) -> bool {
        debug_assert!(idx < self.size);
        (self.bits[idx >> 3] >> (idx & 7)) & 1 == 1
    }

    #[inline(always)]
    pub fn clear(&mut self, idx: usize) {
        debug_assert!(idx < self.size);
        self.bits[idx >> 3] &= !(1 << (idx & 7));
    }

    /// Count of set bits
    pub fn count_set(&self) -> u64 {
        self.bits.iter().map(|b| b.count_ones() as u64).sum()
    }

    /// Count of clear bits (within valid range)
    pub fn count_clear(&self) -> u64 {
        self.size as u64 - self.count_set()
    }

    /// Memory footprint in bytes
    pub fn memory_bytes(&self) -> usize {
        self.bits.len()
    }

    /// Iterate over indices of clear (non-converged) bits
    pub fn clear_indices(&self) -> Vec<usize> {
        (0..self.size).filter(|&i| !self.get(i)).collect()
    }
}

// ============================================================
// COLLATZ TRUE-INTEGER OPERATIONS
// ============================================================

/// True Collatz step on integers
#[inline(always)]
fn collatz_step(n: i64) -> i64 {
    if n <= 0 { 0 }
    else if n % 2 == 0 { n / 2 }
    else { 3 * n + 1 }
}

/// Syracuse function: compress even steps out.
/// For odd n > 1: S(n) = (3n+1) / 2^{v_2(3n+1)}
/// Returns (result, v2_removed) — both exact integers.
#[inline(always)]
fn syracuse(n: i64) -> (i64, u32) {
    debug_assert!(n > 0 && n % 2 == 1, "Syracuse requires odd positive n");
    let val = 3 * n + 1;
    let v = v2(val);
    (val >> v, v)
}

/// Run Collatz on integer n until it reaches 1 or exceeds max_steps.
/// Returns (converged, steps, peak).
fn collatz_run(mut n: i64, max_steps: u64) -> (bool, u64, i64) {
    let mut peak = n;
    for step in 0..max_steps {
        if n == 1 { return (true, step, peak); }
        if n <= 0 { return (false, step, peak); }
        n = collatz_step(n);
        if n > peak { peak = n; }
    }
    (false, max_steps, peak)
}

// ============================================================
// HOLODRIVE SCANNER
// ============================================================

/// Configuration for a holodrive scan.
#[derive(Clone, Debug)]
pub struct HoloConfig {
    /// The odd primes defining the primorial
    pub primes: Vec<i64>,
    /// P_odd = product of primes
    pub p_odd: i64,
    /// Maximum Collatz steps per residue (safety cutoff)
    pub max_steps: u64,
}

impl HoloConfig {
    pub fn new(primes: &[i64], max_steps: u64) -> Self {
        let p_odd: i64 = primes.iter().product();
        HoloConfig {
            primes: primes.to_vec(),
            p_odd,
            max_steps,
        }
    }

    /// Standard primorial levels
    pub fn level_3() -> Self { Self::new(&[3], 1000) }
    pub fn level_15() -> Self { Self::new(&[3, 5], 10_000) }
    pub fn level_105() -> Self { Self::new(&[3, 5, 7], 10_000) }
    pub fn level_1155() -> Self { Self::new(&[3, 5, 7, 11], 100_000) }
    pub fn level_15015() -> Self { Self::new(&[3, 5, 7, 11, 13], 500_000) }
    pub fn level_255255() -> Self { Self::new(&[3, 5, 7, 11, 13, 17], 1_000_000) }
    pub fn level_4849845() -> Self { Self::new(&[3, 5, 7, 11, 13, 17, 19], 2_000_000) }

    pub fn memory_estimate_bytes(&self) -> usize {
        (self.p_odd as usize + 7) / 8
    }
}

impl std::fmt::Display for HoloConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ps: Vec<String> = self.primes.iter().map(|p| p.to_string()).collect();
        write!(f, "P={} ({}), bitset={}KB, max_steps={}",
            self.p_odd, ps.join("×"),
            self.memory_estimate_bytes() / 1024,
            self.max_steps)
    }
}

/// Result of a holodrive scan.
#[derive(Clone, Debug)]
pub struct HoloResult {
    pub config: HoloConfig,
    /// Number of residues that converge to 1
    pub converged: u64,
    /// Number of spurious (non-converging) residues
    pub spurious_count: u64,
    /// The actual spurious residue values
    pub spurious_residues: Vec<i64>,
    /// Exact fraction converged (as Frac)
    pub converged_fraction: Frac,
    /// Peak value encountered across all orbits
    pub global_peak: i64,
    /// Actual memory used (bytes)
    pub memory_used: usize,
}

/// Run the holodrive scan using the SYRACUSE TORUS MAP.
///
/// The Syracuse map on Z/P_odd (odd modulus):
///   1. If r is even: extract odd part first (divide out 2s on the integer)
///   2. For odd r: compute 3r+1 (integer), extract odd part, reduce mod P_odd
///
/// The reduction mod P_odd is where spurious cycles form: two distinct
/// integers with different long-term Collatz behavior can collide to
/// the same residue, creating torus cycles that don't exist in Z.
///
/// These spurious cycles are the SIGNPOSTS for the Generalized
/// Boundary Prime Theorem — each one is cured by the next prime.
pub fn holodrive_scan(config: &HoloConfig) -> HoloResult {
    let p = config.p_odd;
    let mut bitset = HoloBitset::new(p as usize);
    let mut global_peak: i64 = 0;

    // Mark the target: residue 1 is convergent by definition
    if p > 1 { bitset.set(1); }

    // Syracuse torus step: for odd r, compute (3r+1)/2^v mod P_odd
    let syracuse_torus = |r: i64| -> i64 {
        if r <= 0 { return 0; }
        // Extract odd part if r is even
        let mut m = r;
        while m % 2 == 0 && m > 0 { m /= 2; }
        if m <= 0 { return 0; }
        // Now m is odd. Apply 3m+1, extract odd part, reduce mod P
        let val = 3 * m + 1;
        let mut result = val;
        while result % 2 == 0 { result /= 2; }
        modd(result, p)
    };

    // Scan all residues with path memoization
    for r in 0..p {
        if bitset.get(r as usize) { continue; }

        // Trace orbit through the Syracuse torus map
        let mut path: Vec<i64> = Vec::new();
        let mut visited_in_orbit = std::collections::HashSet::new();
        let mut current = r;
        let mut converged = false;

        // Handle r=0 specially (fixed point, never reaches 1)
        if r == 0 {
            continue; // leave as non-converged
        }

        for _ in 0..config.max_steps {
            // Check convergence
            if current == 1 || (current as usize) < bitset.size && bitset.get(current as usize) {
                converged = true;
                break;
            }

            // Check for torus cycle (revisit within this orbit)
            if visited_in_orbit.contains(&current) {
                // Spurious cycle detected — NOT convergent
                break;
            }

            visited_in_orbit.insert(current);
            path.push(current);

            if current > global_peak { global_peak = current; }

            // Apply Syracuse torus step
            current = syracuse_torus(current);
        }

        if converged {
            // Mark all residues along the path as convergent
            for &pr in &path {
                let idx = modd(pr, p) as usize;
                if idx < bitset.size {
                    bitset.set(idx);
                }
            }
        }
    }

    let converged_count = bitset.count_set();
    let spurious_residues = bitset.clear_indices()
        .into_iter()
        .map(|i| i as i64)
        .collect::<Vec<_>>();
    let spurious_count = spurious_residues.len() as u64;

    let total = p as u64;
    let g = gcd_u64(converged_count, total);

    HoloResult {
        config: config.clone(),
        converged: converged_count,
        spurious_count,
        spurious_residues,
        converged_fraction: Frac::new(
            (converged_count / g) as i64,
            (total / g) as i64,
        ),
        global_peak,
        memory_used: HoloBitset::new(p as usize).memory_bytes(),
    }
}

// ============================================================
// CURING PRIME DETECTOR
// ============================================================

/// Result of curing prime analysis for a specific spurious residue.
#[derive(Clone, Debug)]
pub struct CuringLiftResult {
    /// The spurious residue mod P_odd
    pub spurious_r: i64,
    /// The candidate curing prime
    pub curing_prime: i64,
    /// Number of CRT lifts tested (= curing_prime)
    pub lifts_tested: u64,
    /// Number of lifts that converge
    pub lifts_converged: u64,
    /// Whether ALL lifts converge (fully cured)
    pub fully_cured: bool,
    /// Any non-converging lift values (if not fully cured)
    pub non_converging_lifts: Vec<i64>,
}

/// Test whether a candidate prime cures a spurious residue.
///
/// For spurious r at level P_odd, lift to r + j*P_odd for j=0..p_next-1.
/// Run Collatz on each lift. If ALL converge, p_next cures r.
pub fn test_curing_prime(
    spurious_r: i64,
    p_odd: i64,
    curing_prime: i64,
    max_steps: u64,
) -> CuringLiftResult {
    let mut converged_count = 0u64;
    let mut non_converging = Vec::new();

    for j in 0..curing_prime {
        let lift = spurious_r + j * p_odd;
        let (conv, _, _) = collatz_run(lift, max_steps);
        if conv {
            converged_count += 1;
        } else {
            non_converging.push(lift);
        }
    }

    CuringLiftResult {
        spurious_r,
        curing_prime,
        lifts_tested: curing_prime as u64,
        lifts_converged: converged_count,
        fully_cured: non_converging.is_empty(),
        non_converging_lifts: non_converging,
    }
}

/// Scan ALL spurious residues against a candidate curing prime.
/// Excludes r=0 (trivial fixed point — Collatz(0)=0, uncurable).
/// Returns (all_cured, per_residue_results).
pub fn curing_prime_scan(
    holo_result: &HoloResult,
    curing_prime: i64,
    max_steps: u64,
) -> (bool, Vec<CuringLiftResult>) {
    let p_odd = holo_result.config.p_odd;
    let mut results = Vec::new();
    let mut all_cured = true;

    for &r in &holo_result.spurious_residues {
        // Skip trivial r=0 fixed point (uncurable by design)
        if r == 0 { continue; }

        let result = test_curing_prime(r, p_odd, curing_prime, max_steps);
        if !result.fully_cured {
            all_cured = false;
        }
        results.push(result);
    }

    (all_cured, results)
}

/// Auto-detect the curing prime by testing successive primes.
/// Returns the first prime that cures ALL spurious residues,
/// or None if no prime up to max_prime works.
pub fn detect_curing_prime(
    holo_result: &HoloResult,
    max_prime: i64,
    max_steps: u64,
) -> Option<(i64, Vec<CuringLiftResult>)> {
    if holo_result.spurious_count == 0 {
        return None; // Nothing to cure
    }

    // Generate primes not already in the basis
    let existing: std::collections::HashSet<i64> =
        holo_result.config.primes.iter().cloned().collect();
    let candidates: Vec<i64> = sieve_primes(max_prime)
        .into_iter()
        .filter(|p| !existing.contains(p) && *p > 2)
        .collect();

    for &p in &candidates {
        let (all_cured, results) = curing_prime_scan(holo_result, p, max_steps);
        if all_cured {
            return Some((p, results));
        }
    }

    None
}

/// Simple sieve of Eratosthenes up to max
fn sieve_primes(max: i64) -> Vec<i64> {
    if max < 2 { return vec![]; }
    let max = max as usize;
    let mut is_prime = vec![true; max + 1];
    is_prime[0] = false;
    if max >= 1 { is_prime[1] = false; }
    let mut i = 2;
    while i * i <= max {
        if is_prime[i] {
            let mut j = i * i;
            while j <= max {
                is_prime[j] = false;
                j += i;
            }
        }
        i += 1;
    }
    (2..=max).filter(|&i| is_prime[i]).map(|i| i as i64).collect()
}

// ============================================================
// MULTI-LEVEL SWEEP — The Generalized Boundary Prime Theorem test
// ============================================================

/// Result of a multi-level primorial sweep.
#[derive(Clone, Debug)]
pub struct PrimorialLevel {
    pub primes: Vec<i64>,
    pub p_odd: i64,
    pub converged: u64,
    pub spurious: u64,
    pub curing_prime: Option<i64>,
    pub status: String,
}

/// Sweep through primorial levels to test the Generalized Boundary
/// Prime Theorem: each level's spurious set is cured by the next prime.
pub fn primorial_sweep(
    max_primes: usize,
    max_steps: u64,
    cure_max_steps: u64,
) -> Vec<PrimorialLevel> {
    let all_primes = sieve_primes(100);
    let odd_primes: Vec<i64> = all_primes.into_iter().filter(|&p| p > 2).collect();
    let mut results = Vec::new();

    let mut current_primes: Vec<i64> = Vec::new();

    for (i, &p) in odd_primes.iter().enumerate().take(max_primes) {
        current_primes.push(p);
        let p_odd: i64 = current_primes.iter().product();

        // Skip if too large for exhaustive scan
        if p_odd > 50_000_000 {
            results.push(PrimorialLevel {
                primes: current_primes.clone(),
                p_odd,
                converged: 0,
                spurious: 0,
                curing_prime: None,
                status: format!("SKIPPED (P={} too large for exhaustive scan)", p_odd),
            });
            continue;
        }

        let config = HoloConfig::new(&current_primes, max_steps);
        let holo = holodrive_scan(&config);

        let curing = if holo.spurious_count > 0 {
            // Test the next prime as curing candidate
            if i + 1 < odd_primes.len() {
                let next_p = odd_primes[i + 1];
                let (cured, _) = curing_prime_scan(&holo, next_p, cure_max_steps);
                if cured { Some(next_p) } else { None }
            } else {
                None
            }
        } else {
            None
        };

        let status = if holo.spurious_count == 0 {
            "✓ CLEAN".to_string()
        } else if let Some(cp) = curing {
            format!("→ {} spurious, CURED by p={}", holo.spurious_count, cp)
        } else {
            format!("→ {} spurious, cure NOT found", holo.spurious_count)
        };

        results.push(PrimorialLevel {
            primes: current_primes.clone(),
            p_odd,
            converged: holo.converged,
            spurious: holo.spurious_count,
            curing_prime: curing,
            status,
        });
    }

    results
}

// ============================================================
// SPURIOUS CYCLE ANALYSIS
// ============================================================

/// Analyze a spurious residue: trace its Collatz orbit and find the cycle.
#[derive(Clone, Debug)]
pub struct SpuriousCycleInfo {
    pub anchor: i64,
    /// The cycle (as a sequence of values)
    pub cycle: Vec<i64>,
    /// Cycle length
    pub cycle_len: usize,
    /// Transient length (steps before entering cycle)
    pub transient: usize,
    /// CRT decomposition of the anchor
    pub crt_decomp: Vec<(i64, i64)>, // (residue, prime)
    /// Per-prime analysis: which primes "see" this cycle
    pub per_prime_residues: Vec<(i64, i64)>, // (prime, anchor mod prime)
}

/// Analyze spurious residues: find their cycles and CRT structure.
pub fn analyze_spurious(
    holo_result: &HoloResult,
    max_steps: u64,
) -> Vec<SpuriousCycleInfo> {
    let p_odd = holo_result.config.p_odd;
    let primes = &holo_result.config.primes;
    let mut results = Vec::new();

    // Syracuse torus step (same as in holodrive_scan)
    let syracuse_torus = |r: i64| -> i64 {
        if r <= 0 { return 0; }
        let mut m = r;
        while m % 2 == 0 && m > 0 { m /= 2; }
        if m <= 0 { return 0; }
        let val = 3 * m + 1;
        let mut result = val;
        while result % 2 == 0 { result /= 2; }
        modd(result, p_odd)
    };

    for &r in &holo_result.spurious_residues {
        // Trace orbit with Floyd's cycle detection ON THE TORUS MAP
        let mut tortoise = syracuse_torus(r);
        let mut hare = syracuse_torus(syracuse_torus(r));
        let mut found_cycle = false;

        for _ in 0..max_steps {
            if tortoise == hare {
                found_cycle = true;
                break;
            }
            tortoise = syracuse_torus(tortoise);
            hare = syracuse_torus(syracuse_torus(hare));
        }

        if !found_cycle {
            results.push(SpuriousCycleInfo {
                anchor: r,
                cycle: vec![],
                cycle_len: 0,
                transient: 0,
                crt_decomp: primes.iter().map(|&p| (modd(r, p), p)).collect(),
                per_prime_residues: primes.iter().map(|&p| (p, modd(r, p))).collect(),
            });
            continue;
        }

        // Find cycle start
        tortoise = r;
        let mut transient = 0usize;
        // Advance both from start and meeting point
        let mut t2 = tortoise;
        let mut h2 = hare;
        // Re-do Floyd properly: start tortoise from beginning
        t2 = r;
        for _ in 0..max_steps {
            if t2 == h2 { break; }
            t2 = syracuse_torus(t2);
            h2 = syracuse_torus(h2);
            transient += 1;
        }

        // Find cycle length
        let mut cycle = vec![t2];
        let mut runner = syracuse_torus(t2);
        while runner != t2 {
            cycle.push(runner);
            runner = syracuse_torus(runner);
        }

        results.push(SpuriousCycleInfo {
            anchor: r,
            cycle_len: cycle.len(),
            cycle,
            transient,
            crt_decomp: primes.iter().map(|&p| (modd(r, p), p)).collect(),
            per_prime_residues: primes.iter().map(|&p| (p, modd(r, p))).collect(),
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitset() {
        let mut bs = HoloBitset::new(100);
        assert!(!bs.get(42));
        bs.set(42);
        assert!(bs.get(42));
        bs.clear(42);
        assert!(!bs.get(42));
        assert_eq!(bs.count_set(), 0);
    }

    #[test]
    fn test_syracuse() {
        // S(3) = (3*3+1)/2^? = 10/2 = 5, v2=1
        let (r, v) = syracuse(3);
        assert_eq!(r, 5);
        assert_eq!(v, 1);
        // S(7) = (3*7+1)/2^? = 22/2 = 11, v2=1
        let (r, v) = syracuse(7);
        assert_eq!(r, 11);
        assert_eq!(v, 1);
        // S(5) = (3*5+1)/2^4 = 16/16 = 1, v2=4
        let (r, v) = syracuse(5);
        assert_eq!(r, 1);
        assert_eq!(v, 4);
    }

    #[test]
    fn test_sieve() {
        let primes = sieve_primes(30);
        assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
    }

    #[test]
    fn test_holo_level_3() {
        let config = HoloConfig::level_3();
        let result = holodrive_scan(&config);
        // Only r=0 is spurious (trivial fixed point)
        assert_eq!(result.spurious_count, 1, "P=3 should have only r=0 spurious");
        assert_eq!(result.spurious_residues, vec![0]);
    }

    #[test]
    fn test_holo_level_105() {
        let config = HoloConfig::level_105();
        let result = holodrive_scan(&config);
        // Only r=0 is spurious at P=105
        assert_eq!(result.spurious_count, 1,
            "P=105 should have only r=0 spurious, found {}: {:?}",
            result.spurious_count, &result.spurious_residues[..result.spurious_count.min(10) as usize]);
    }

    #[test]
    fn test_holo_level_15015() {
        let config = HoloConfig::level_15015();
        let result = holodrive_scan(&config);
        // P=15015 should have only r=0 spurious
        assert_eq!(result.spurious_count, 1,
            "P=15015 should have only r=0, found {} spurious: {:?}",
            result.spurious_count, &result.spurious_residues[..result.spurious_count.min(10) as usize]);
    }
}
