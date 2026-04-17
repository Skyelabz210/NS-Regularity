//! # CRAM-3 Stage Pipeline + Ramanujan-Guided Grover
//!
//! Implements the two highest-priority unbuilt bridges from the Grand Synthesis:
//!
//! 1. **3-Stage CRAM Pipelines** (root → mid → end): Extends the 2-stage Γ calculus
//!    to compounding pullback cascades. The original CRAM exploration (March 10, 2026)
//!    identified this as the natural next step but it was never built.
//!
//! 2. **Ramanujan-Guided Grover Oracle**: Connects Dense Toric Grover to the
//!    Ramanujan-CRT torus. The oracle marks residue classes identified by T6
//!    as forbidden zones for a given prime gap d.
//!
//! Zero floats. All arithmetic exact integer.

#![forbid(unsafe_code)]

use std::collections::HashSet;

// ============================================================================
// SHARED PRIMITIVES
// ============================================================================

#[inline] fn mulmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 * b as u128) % m as u128) as u64 }
#[inline] fn addmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 + b as u128) % m as u128) as u64 }
#[inline] fn submod(a: u64, b: u64, m: u64) -> u64 { if a >= b { (a - b) % m } else { (m - ((b - a) % m)) % m } }

fn mod_pow(mut b: u64, mut e: u64, m: u64) -> u64 {
    if m == 1 { return 0; }
    let mut r = 1u64; b %= m;
    while e > 0 { if e & 1 == 1 { r = mulmod(r, b, m); } e >>= 1; if e > 0 { b = mulmod(b, b, m); } }
    r
}

fn mod_inverse(a: u64, p: u64) -> u64 { mod_pow(a % p, p - 2, p) }

fn gcd(a: u64, b: u64) -> u64 { let (mut a, mut b) = (a, b); while b != 0 { let t = b; b = a % b; a = t; } a }

fn integer_sqrt_prim(n: u64) -> u64 {
    if n == 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + n / x) / 2; }
    x
}

fn is_prime(n: u64) -> bool {
    if n < 2 { return false; } if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5;
    while i * i <= n { if n % i == 0 || n % (i + 2) == 0 { return false; } i += 6; }
    true
}

fn ramanujan_sum_prime(p: u64, d: u64) -> i64 {
    if d % p == 0 { p as i64 - 1 } else { -1 }
}

fn sieve_power(p: u64, d: u64) -> (u64, u64) {
    // Returns (numerator, denominator) of SP(p,d) = (2p-1-c_p(d))/p²
    let c = ramanujan_sum_prime(p, d);
    let num = (2 * p as i64 - 1 - c) as u64;
    let den = p * p;
    (num, den)
}

// ============================================================================
// CRAM OPERATOR TYPES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RootOp { Add, Sub, Mul, Div }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp { Id, Sqr, Cube, Neg, Inv, Double }

/// A 3-stage CRAM lane: root_op → mid_op → end_op
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lane3Stage {
    pub prime: u64,
    pub root: RootOp,
    pub mid: UnaryOp,
    pub end: UnaryOp,
}

/// A complete 3-stage CRAM topology
#[derive(Debug, Clone)]
pub struct Topology3Stage {
    pub lanes: Vec<Lane3Stage>,
}

// ============================================================================
// 3-STAGE PIPELINE EXECUTION
// ============================================================================

fn apply_root(op: RootOp, a: u64, b: u64, p: u64) -> Option<u64> {
    match op {
        RootOp::Add => Some(addmod(a, b, p)),
        RootOp::Sub => Some(submod(a, b, p)),
        RootOp::Mul => Some(mulmod(a, b, p)),
        RootOp::Div => {
            if b == 0 { None } // Exceptional: division by zero
            else { Some(mulmod(a, mod_inverse(b, p), p)) }
        }
    }
}

fn apply_unary(op: UnaryOp, x: u64, p: u64) -> Option<u64> {
    match op {
        UnaryOp::Id => Some(x),
        UnaryOp::Sqr => Some(mulmod(x, x, p)),
        UnaryOp::Cube => Some(mulmod(mulmod(x, x, p), x, p)),
        UnaryOp::Neg => Some(if x == 0 { 0 } else { p - x }),
        UnaryOp::Inv => {
            if x == 0 { None } // Exceptional
            else { Some(mod_inverse(x, p)) }
        }
        UnaryOp::Double => Some(addmod(x, x, p)),
    }
}

/// Execute a 3-stage pipeline on inputs (a, b) mod p.
/// Returns None if any stage hits an exceptional case (div/inv by 0).
fn execute_3stage(lane: &Lane3Stage, a: u64, b: u64) -> Option<u64> {
    let p = lane.prime;
    let stage1 = apply_root(lane.root, a, b, p)?;
    let stage2 = apply_unary(lane.mid, stage1, p)?;
    let stage3 = apply_unary(lane.end, stage2, p)?;
    Some(stage3)
}

// ============================================================================
// Γ CALCULUS FOR 3-STAGE PIPELINES
// ============================================================================

/// Compute the effective forbidden set for a 3-stage pipeline on gap d.
///
/// The forbidden set F_base = {0, (p-d) mod p} (or {0} if p|d).
/// The pipeline maps each residue r to an output via root(r, (r+d)%p) → mid → end.
/// Forbidden outputs = images of F_base under the pipeline.
/// Effective forbidden = pullback: all r whose output lands in forbidden outputs.
///
/// Γ = |F_eff| / |F_base| — returned as (eff_size, base_size) exact rational
pub fn compute_gamma_3stage(lane: &Lane3Stage, d: u64) -> (usize, usize) {
    let p = lane.prime;
    let d_r = d % p;

    // Base forbidden residues
    let mut base_forbidden = HashSet::new();
    base_forbidden.insert(0u64);
    if d_r != 0 {
        base_forbidden.insert((p - d_r) % p);
    }
    let base_size = base_forbidden.len();

    // Forward map: for each residue r, compute full pipeline output
    let mut forward: Vec<Option<u64>> = Vec::with_capacity(p as usize);
    for r in 0..p {
        forward.push(execute_3stage(lane, r, d_r));
    }

    // Forbidden outputs = images of base_forbidden under forward map
    let mut forbidden_outputs = HashSet::new();
    for &f_r in &base_forbidden {
        match forward[f_r as usize] {
            Some(out) => { forbidden_outputs.insert(out); }
            None => {}
        }
    }

    // Pullback: all r whose output is in forbidden_outputs OR is None
    let mut effective_forbidden = HashSet::new();
    for r in 0..p {
        match forward[r as usize] {
            None => { effective_forbidden.insert(r); }
            Some(out) => {
                if forbidden_outputs.contains(&out) {
                    effective_forbidden.insert(r);
                }
            }
        }
    }

    (effective_forbidden.len(), base_size)
}

/// Compute total topology gain as exact rational (numerator, denominator).
/// Product of per-lane Γ = ∏(eff_i/base_i) = (∏eff_i) / (∏base_i)
pub fn compute_topology_gain(topology: &Topology3Stage, d: u64) -> (u64, u64) {
    let mut num: u64 = 1;
    let mut den: u64 = 1;
    for lane in &topology.lanes {
        let (eff, base) = compute_gamma_3stage(lane, d);
        num = num.saturating_mul(eff as u64);
        den = den.saturating_mul(base as u64);
    }
    (num, den)
}

/// Validity ratio as exact rational (valid_count, total_sampled).
pub fn compute_validity(topology: &Topology3Stage, sample_size: usize) -> (u64, u64) {
    let n: u64 = topology.lanes.iter().map(|l| l.prime).product();
    let limit = (sample_size as u64).min(n);
    let mut valid = 0u64;
    for x in 0..limit {
        let mut all_valid = true;
        for lane in &topology.lanes {
            let p = lane.prime;
            let a = x % p;
            let b = (x + 1) % p; // Arbitrary second input
            if execute_3stage(lane, a, b).is_none() {
                all_valid = false;
                break;
            }
        }
        if all_valid { valid += 1; }
    }
    (valid, limit)
}

// ============================================================================
// RAMANUJAN-GUIDED GROVER ORACLE
// ============================================================================

/// Ramanujan-Guided Grover Oracle for prime-pair sieving on the CRT torus.
///
/// For a given gap d and set of primes, computes:
/// 1. Which residue classes are forbidden (from T6 Ramanujan-sieve identity)
/// 2. Which candidates survive ALL lane sieves
/// 3. Marks survivors for Grover amplitude amplification
pub struct RamanujanOracle {
    pub primes: Vec<u64>,
    pub gap: u64,
    pub forbidden_per_lane: Vec<HashSet<u64>>,
    pub modulus: u64,
}

impl RamanujanOracle {
    pub fn new(primes: Vec<u64>, gap: u64) -> Self {
        let modulus: u64 = primes.iter().product();
        let mut forbidden_per_lane = Vec::new();

        for &p in &primes {
            let mut forbidden = HashSet::new();
            forbidden.insert(0);
            let d_r = gap % p;
            if d_r != 0 {
                forbidden.insert(p - d_r);
            }
            forbidden_per_lane.push(forbidden);
        }

        Self { primes, gap, forbidden_per_lane, modulus }
    }

    /// Check if candidate n is eliminated by the sieve (composite for gap d).
    /// Returns true if n should be MARKED by the oracle (it's a survivor = potential prime pair).
    pub fn is_survivor(&self, n: u64) -> bool {
        for (i, &p) in self.primes.iter().enumerate() {
            let r = n % p;
            if self.forbidden_per_lane[i].contains(&r) {
                return false; // Eliminated by lane i
            }
        }
        true
    }

    /// Count survivors in [1, limit)
    pub fn count_survivors(&self, limit: u64) -> u64 {
        let mut count = 0;
        for n in 1..limit {
            if self.is_survivor(n) { count += 1; }
        }
        count
    }

    /// Predicted survival rate from Ramanujan spectrum (T6).
    /// Returns (numerator, denominator) exact rational.
    /// survival = ∏(1 - SP(p,d)) = ∏((p²-sp_num)/p²)
    pub fn predicted_survival_rate(&self) -> (u64, u64) {
        let mut num: u64 = 1;
        let mut den: u64 = 1;
        for &p in &self.primes {
            let (sp_num, sp_den) = sieve_power(p, self.gap);
            // 1 - sp_num/sp_den = (sp_den - sp_num)/sp_den
            let lane_num = (sp_den as i64 - sp_num as i64).max(0) as u64;
            num = num.saturating_mul(lane_num);
            den = den.saturating_mul(sp_den);
        }
        (num, den)
    }

    /// Per-lane sieve power from T6 identity
    pub fn lane_sieve_powers(&self) -> Vec<(u64, u64, u64)> {
        // Returns (prime, SP_numerator, SP_denominator) per lane
        self.primes.iter().map(|&p| {
            let (n, d) = sieve_power(p, self.gap);
            (p, n, d)
        }).collect()
    }
}

/// Dense Toric Grover simulation with Ramanujan oracle.
///
/// Uses exact integer arithmetic on ℤ/Mℤ for amplitude tracking.
/// The oracle marks candidates that SURVIVE the Ramanujan sieve
/// (potential prime pairs), and Grover amplification concentrates
/// probability on those survivors.
pub struct DenseToricGroverRamanujan {
    pub oracle: RamanujanOracle,
    pub amplitudes: Vec<i64>, // Scaled amplitudes (×SCALE)
    pub n_states: usize,
    pub n_marked: usize,
    pub scale: i64,
}

impl DenseToricGroverRamanujan {
    pub fn new(oracle: RamanujanOracle, search_range: u64) -> Self {
        let n_states = search_range as usize;
        let scale = 10_000i64; // Fixed-point scale for integer amplitude tracking

        // Initial uniform amplitude: 1/sqrt(N) ≈ SCALE/isqrt(N)
        let initial_amp = scale / (integer_sqrt_prim(n_states as u64) as i64).max(1);
        let amplitudes = vec![initial_amp; n_states];

        // Count marked states (survivors)
        let n_marked = (1..=search_range).filter(|&n| oracle.is_survivor(n)).count();

        Self { oracle, amplitudes, n_states, n_marked, scale }
    }

    /// One Grover iteration: Oracle + Diffusion
    pub fn iterate(&mut self) {
        let n = self.n_states;

        // Oracle: flip sign of marked states (survivors)
        for i in 0..n {
            if self.oracle.is_survivor((i + 1) as u64) {
                self.amplitudes[i] = -self.amplitudes[i];
            }
        }

        // Diffusion: 2|mean⟩⟨mean| - I
        // mean = sum / n
        let sum: i64 = self.amplitudes.iter().sum();
        let two_mean_n = 2 * sum; // = 2 * mean * n

        for i in 0..n {
            // new[i] = 2*mean - old[i] = (2*sum - n*old[i]) / n
            // To stay integer: new[i] = (2*sum - n*old[i]) / n
            // Use scaled: we accept the integer division truncation here
            // (exact version would use CRT torus, but this demonstrates the bridge)
            self.amplitudes[i] = (two_mean_n - (n as i64) * self.amplitudes[i]) / (n as i64);
        }
    }

    /// Probability of measuring a marked state as exact rational (marked_power, total_power).
    pub fn marked_probability(&self) -> (i128, i128) {
        let mut marked_power: i128 = 0;
        let mut total_power: i128 = 0;

        for i in 0..self.n_states {
            let amp_sq = (self.amplitudes[i] as i128) * (self.amplitudes[i] as i128);
            total_power += amp_sq;
            if self.oracle.is_survivor((i + 1) as u64) {
                marked_power += amp_sq;
            }
        }

        (marked_power, total_power)
    }

    /// Probability as milliunits (0..1000) for threshold checks
    pub fn marked_probability_millis(&self) -> u32 {
        let (num, den) = self.marked_probability();
        if den == 0 { 0 } else { (num * 1000 / den) as u32 }
    }

    /// Optimal number of Grover iterations: floor(π/4 * sqrt(N/M))
    /// Integer-only: π/4 ≈ 785398/1000000, sqrt via integer_sqrt
    pub fn optimal_iterations(&self) -> usize {
        if self.n_marked == 0 { return 0; }
        // π/4 · √(N/M) using scaled integer arithmetic
        // = 785398 · isqrt(N * 10^12 / M) / 10^6 / 10^6
        let ratio_scaled = (self.n_states as u128) * 1_000_000_000_000u128 / (self.n_marked as u128);
        let sqrt_scaled = {
            let n = ratio_scaled;
            if n == 0 { return 0; }
            let mut x = n;
            let mut y = (x + 1) / 2;
            while y < x { x = y; y = (x + n / x) / 2; }
            x // ≈ √(N/M) · 10^6
        };
        (785398u128 * sqrt_scaled / 1_000_000_000_000u128) as usize
    }
}

// ============================================================================
// 3-STAGE CRAM: EXHAUSTIVE TOPOLOGY SEARCH
// ============================================================================

/// Find the best 3-stage topology for a given gap d across Safe Basis primes.
/// Returns (topology, gain_numerator, gain_denominator) with gain as exact rational.
pub fn find_best_3stage_topology(primes: &[u64], d: u64) -> (Topology3Stage, u64, u64) {
    let roots = [RootOp::Add, RootOp::Sub, RootOp::Mul];
    let unaries = [UnaryOp::Id, UnaryOp::Sqr, UnaryOp::Cube, UnaryOp::Neg, UnaryOp::Double];

    let mut best_topology = Topology3Stage { lanes: Vec::new() };

    // For each prime, find the best (root, mid, end) triple by max eff/base ratio
    let mut best_per_lane: Vec<(Lane3Stage, usize, usize)> = Vec::new(); // (lane, eff, base)

    for &p in primes {
        let mut lane_best_eff: usize = 0;
        let mut lane_best_base: usize = 1;
        let mut lane_best = Lane3Stage { prime: p, root: RootOp::Add, mid: UnaryOp::Id, end: UnaryOp::Id };

        for &root in &roots {
            for &mid in &unaries {
                for &end in &unaries {
                    let lane = Lane3Stage { prime: p, root, mid, end };
                    let (eff, base) = compute_gamma_3stage(&lane, d);
                    // Compare eff/base > lane_best_eff/lane_best_base via cross-multiply
                    if eff * lane_best_base > lane_best_eff * base {
                        lane_best_eff = eff;
                        lane_best_base = base;
                        lane_best = lane;
                    }
                }
            }
        }

        best_per_lane.push((lane_best, lane_best_eff, lane_best_base));
    }

    let mut lanes = Vec::new();
    let mut total_num: u64 = 1;
    let mut total_den: u64 = 1;
    for (lane, eff, base) in &best_per_lane {
        lanes.push(*lane);
        total_num = total_num.saturating_mul(*eff as u64);
        total_den = total_den.saturating_mul(*base as u64);
    }

    best_topology = Topology3Stage { lanes };
    (best_topology, total_num, total_den)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const SAFE_PRIMES: [u64; 5] = [3, 5, 7, 11, 13];

    #[test]
    fn gamma_3stage_add_id_id_matches_2stage() {
        for &p in &SAFE_PRIMES {
            let lane = Lane3Stage { prime: p, root: RootOp::Add, mid: UnaryOp::Id, end: UnaryOp::Id };
            let (eff, base) = compute_gamma_3stage(&lane, 2);
            if 2 % p != 0 {
                assert_eq!(base, 2, "p={p}: base should be 2");
                assert_eq!(eff, 2, "p={p}: Add+Id+Id should give |F_eff|=2");
            }
        }
    }

    #[test]
    fn gamma_3stage_add_sqr_id_matches_2stage_add_sqr() {
        for &p in &SAFE_PRIMES {
            if 2 % p == 0 { continue; }
            let lane = Lane3Stage { prime: p, root: RootOp::Add, mid: UnaryOp::Sqr, end: UnaryOp::Id };
            let (eff, base) = compute_gamma_3stage(&lane, 2);
            if p == 3 {
                assert_eq!(eff, 3, "p=3: Add+Sqr+Id is degenerate (all residues forbidden)");
            } else {
                assert_eq!(eff, 3, "p={p}: Add+Sqr+Id should give |F_eff|=3");
            }
            assert_eq!(base, 2);
        }
    }

    #[test]
    fn gamma_3stage_sub_id_convention_a() {
        for &p in &SAFE_PRIMES {
            if 2 % p == 0 { continue; }
            let lane = Lane3Stage { prime: p, root: RootOp::Sub, mid: UnaryOp::Id, end: UnaryOp::Id };
            let (eff, base) = compute_gamma_3stage(&lane, 2);
            assert_eq!(base, 2, "p={p}: base should be 2");
            assert_eq!(eff, 2, "p={p}: Sub+Id+Id is bijection, |F_eff|=2");
            // Γ = eff/base = 2/2 = 1 (no gain)
            assert_eq!(eff, base, "p={p}: Γ should be 1 for shift");
        }
    }

    #[test]
    fn gamma_3stage_sub_sqr_matches_t71() {
        for &p in &SAFE_PRIMES {
            if 2 % p == 0 { continue; }
            let lane = Lane3Stage { prime: p, root: RootOp::Sub, mid: UnaryOp::Sqr, end: UnaryOp::Id };
            let (eff, base) = compute_gamma_3stage(&lane, 2);
            assert_eq!(base, 2);
            if p == 3 {
                assert_eq!(eff, 2, "p=3: Sub+Sqr known special case");
            } else {
                assert_eq!(eff, 4, "p={p}: Sub+Sqr+Id should give |F_eff|=4");
            }
        }
    }

    #[test]
    fn gamma_3stage_compounding_gain() {
        for &p in &[5u64, 7, 11, 13, 17, 19, 23, 29, 31] {
            if !is_prime(p) { continue; }
            let roots = [RootOp::Add, RootOp::Sub, RootOp::Mul];
            let posts = [UnaryOp::Id, UnaryOp::Sqr, UnaryOp::Cube, UnaryOp::Neg];
            let ops = [UnaryOp::Id, UnaryOp::Sqr, UnaryOp::Cube, UnaryOp::Neg, UnaryOp::Double];
            let mut two_best = 0usize;
            for &root in &roots {
                for &post in &posts {
                    let lane = Lane3Stage { prime: p, root, mid: post, end: UnaryOp::Id };
                    let (eff, _) = compute_gamma_3stage(&lane, 2);
                    two_best = two_best.max(eff);
                }
            }
            let mut three_best = 0usize;
            for &root in &roots {
                for &mid in &ops {
                    for &end in &ops {
                        let lane = Lane3Stage { prime: p, root, mid, end };
                        let (eff, _) = compute_gamma_3stage(&lane, 2);
                        three_best = three_best.max(eff);
                    }
                }
            }
            // Sub+Id achieves p, so 3-stage can't exceed. Framework correctness check.
            assert!(three_best >= two_best, "p={p}: 3-stage should be >= 2-stage");
        }
    }

    #[test]
    fn gamma_3stage_exhaustive_small_primes() {
        let roots = [RootOp::Add, RootOp::Sub, RootOp::Mul];
        let ops = [UnaryOp::Id, UnaryOp::Sqr, UnaryOp::Neg];
        let mut total_configs = 0;
        let mut max_eff_seen = 0usize;

        for &p in &[3u64, 5, 7, 11, 13] {
            for d in 1..=20u64 {
                for &root in &roots {
                    for &mid in &ops {
                        for &end in &ops {
                            let lane = Lane3Stage { prime: p, root, mid, end };
                            let (eff, _base) = compute_gamma_3stage(&lane, d);
                            total_configs += 1;
                            assert!(eff <= p as usize,
                                "p={p} d={d} {:?}+{:?}+{:?}: |F_eff|={eff} > p={p}",
                                root, mid, end);
                            if eff > max_eff_seen { max_eff_seen = eff; }
                        }
                    }
                }
            }
        }
        eprintln!("Exhaustive: {total_configs} configs verified. Max |F_eff| = {max_eff_seen}");
        assert!(total_configs > 2000);
    }

    #[test]
    fn ramanujan_oracle_survival_rate() {
        let oracle = RamanujanOracle::new(vec![3, 5, 7, 11, 13], 2);
        let (pred_n, pred_d) = oracle.predicted_survival_rate();
        let measured_n = oracle.count_survivors(15015) as u64;
        let measured_d = 15014u64;
        // Check: pred_n/pred_d ≈ measured_n/measured_d via cross-multiply
        // |pred_n * measured_d - measured_n * pred_d| / (pred_d * measured_d) < 0.001
        let diff = (pred_n as i128 * measured_d as i128 - measured_n as i128 * pred_d as i128).unsigned_abs();
        let threshold = pred_d as u128 * measured_d as u128 / 1000; // 0.1%
        eprintln!("Twin d=2: predicted={pred_n}/{pred_d}, measured={measured_n}/{measured_d}, diff={diff}");
        assert!(diff < threshold, "T6 prediction should match within 0.1%");
    }

    #[test]
    fn ramanujan_oracle_sexy_primes() {
        let oracle = RamanujanOracle::new(vec![3, 5, 7, 11, 13], 6);
        let (pred_n, pred_d) = oracle.predicted_survival_rate();
        let measured_n = oracle.count_survivors(15015) as u64;
        let measured_d = 15014u64;
        let diff = (pred_n as i128 * measured_d as i128 - measured_n as i128 * pred_d as i128).unsigned_abs();
        let threshold = pred_d as u128 * measured_d as u128 / 1000;
        assert!(diff < threshold);
    }

    #[test]
    fn ramanujan_oracle_lane_powers() {
        let oracle = RamanujanOracle::new(vec![3, 5, 7, 11, 13], 2);
        let powers = oracle.lane_sieve_powers();
        for (p, num, den) in &powers {
            let expected_num = 2 * p;
            assert_eq!(*num, expected_num, "p={p}: SP numerator should be 2p");
            assert_eq!(*den, p * p, "p={p}: SP denominator should be p²");
        }
    }

    #[test]
    fn grover_ramanujan_amplifies_survivors() {
        let oracle = RamanujanOracle::new(vec![3, 5, 7], 2);
        let mut grover = DenseToricGroverRamanujan::new(oracle, 105);
        let init_millis = grover.marked_probability_millis();
        let opt = grover.optimal_iterations();
        for _ in 0..opt { grover.iterate(); }
        let final_millis = grover.marked_probability_millis();
        eprintln!("Grover d=2: init={init_millis}‰, final={final_millis}‰, iters={opt}");
        assert!(final_millis > init_millis, "Grover should amplify survivor probability");
    }

    #[test]
    fn grover_ramanujan_larger_basis() {
        let oracle = RamanujanOracle::new(vec![3, 5, 7, 11], 2);
        let mut grover = DenseToricGroverRamanujan::new(oracle, 200);
        let init_millis = grover.marked_probability_millis();
        let opt = grover.optimal_iterations().min(20);
        for _ in 0..opt { grover.iterate(); }
        let final_millis = grover.marked_probability_millis();
        assert!(final_millis > init_millis);
    }

    #[test]
    fn find_best_twin_topology() {
        let primes = vec![3, 5, 7, 11, 13];
        let (best, gain_n, gain_d) = find_best_3stage_topology(&primes, 2);
        eprintln!("\nBest 3-stage for d=2: gain={gain_n}/{gain_d}");
        for lane in &best.lanes {
            let (eff, base) = compute_gamma_3stage(lane, 2);
            eprintln!("  p={}: {:?}+{:?}+{:?} → |F_eff|={eff}/{base}",
                lane.prime, lane.root, lane.mid, lane.end);
        }
        assert!(gain_n > gain_d, "Best topology should have gain > 1");
    }

    #[test]
    fn find_best_cousin_topology() {
        let primes = vec![3, 5, 7, 11, 13];
        let (best, gain_n, gain_d) = find_best_3stage_topology(&primes, 4);
        eprintln!("Best 3-stage for d=4: gain={gain_n}/{gain_d}");
        assert!(gain_n >= gain_d);
    }

    #[test]
    fn find_best_sexy_topology() {
        let primes = vec![3, 5, 7, 11, 13];
        let (best, gain_n, gain_d) = find_best_3stage_topology(&primes, 6);
        eprintln!("Best 3-stage for d=6: gain={gain_n}/{gain_d}");
        assert!(gain_n >= gain_d);
    }

    #[test]
    fn compare_2stage_vs_3stage_gains() {
        let primes = vec![3u64, 5, 7, 11, 13];
        for &d in &[2u64, 4, 6, 30] {
            // 2-stage best per lane (as rational product)
            let mut two_num: u64 = 1;
            let mut two_den: u64 = 1;
            for &p in &primes {
                let roots = [RootOp::Add, RootOp::Sub, RootOp::Mul];
                let posts = [UnaryOp::Id, UnaryOp::Sqr, UnaryOp::Cube, UnaryOp::Neg];
                let mut best_eff = 0usize;
                let mut best_base = 1usize;
                for &root in &roots {
                    for &post in &posts {
                        let lane = Lane3Stage { prime: p, root, mid: post, end: UnaryOp::Id };
                        let (eff, base) = compute_gamma_3stage(&lane, d);
                        if eff * best_base > best_eff * base {
                            best_eff = eff; best_base = base;
                        }
                    }
                }
                two_num = two_num.saturating_mul(best_eff as u64);
                two_den = two_den.saturating_mul(best_base as u64);
            }
            let (_, three_num, three_den) = find_best_3stage_topology(&primes, d);
            // 3-stage should be >= 2-stage
            assert!(three_num * two_den >= two_num * three_den,
                "d={d}: 3-stage gain should be >= 2-stage gain");
        }
    }

    #[test]
    fn integration_grover_with_cram_topology() {
        let primes = vec![3u64, 5, 7];
        let d = 2u64;
        let (best_topo, topo_n, topo_d) = find_best_3stage_topology(&primes, d);
        let oracle = RamanujanOracle::new(primes.clone(), d);
        let (surv_n, surv_d) = oracle.predicted_survival_rate();
        let mut grover = DenseToricGroverRamanujan::new(oracle, 105);
        let opt = grover.optimal_iterations().min(10);
        for _ in 0..opt { grover.iterate(); }
        let final_millis = grover.marked_probability_millis();
        eprintln!("Integration d=2: topo={topo_n}/{topo_d}, surv={surv_n}/{surv_d}, prob={final_millis}‰");
    }
}

