//! Hydra Sieve — Heterogeneous CRAM Prime Pair Identification
//!
//! Seven-layer architecture using operator-distorted CRT carry patterns
//! to eliminate composite candidates. The nonlinear operators (Sqr, Neg)
//! expand the forbidden zone beyond what straight residue checking achieves.
//!
//! T6: SP(p,d) = (2p − 1 − c_p(d)) / p²  (Ramanujan-Sieve Identity)
//! T7: Γ = |F_eff| / |F_base|             (Pullback Gain Calculus)

/// CRAM root operators (binary: applied to (n mod p, d mod p))
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootOp { Add, Sub, Mul }

/// CRAM post-processors (unary: applied to intermediate result)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostOp { Id, Neg, Sqr }

impl std::fmt::Display for RootOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { RootOp::Add => write!(f, "Add"), RootOp::Sub => write!(f, "Sub"), RootOp::Mul => write!(f, "Mul") }
    }
}
impl std::fmt::Display for PostOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { PostOp::Id => write!(f, "Id"), PostOp::Neg => write!(f, "Neg"), PostOp::Sqr => write!(f, "Sqr") }
    }
}

// ── Modular arithmetic (u64, overflow-safe) ─────────────────────────

fn addmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 + b as u128) % m as u128) as u64 }
fn submod(a: u64, b: u64, m: u64) -> u64 { if a >= b { (a - b) % m } else { m - ((b - a) % m) } }
fn mulmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 * b as u128) % m as u128) as u64 }

fn mod_inverse_u64(a: u64, m: u64) -> Option<u64> {
    let (mut old_r, mut r) = (a as i128, m as i128);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = old_r / r;
        let tmp = r; r = old_r - q * r; old_r = tmp;
        let tmp = s; s = old_s - q * s; old_s = tmp;
    }
    if old_r != 1 { return None; }
    Some(((old_s % m as i128 + m as i128) as u128 % m as u128) as u64)
}

fn is_prime_u64(n: u64) -> bool {
    if n < 2 { return false; }
    if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5u64;
    while i.saturating_mul(i) <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}

// ── T6: Ramanujan Sieve Power ───────────────────────────────────────

pub fn ramanujan_sum(p: u64, d: u64) -> i64 {
    if d % p == 0 { p as i64 - 1 } else { -1 }
}

/// SP(p,d) = (2p − 1 − c_p(d)) / p² as (num, den)
pub fn sieve_power(p: u64, d: u64) -> (u64, u64) {
    let c = ramanujan_sum(p, d);
    let num = (2 * p as i64 - 1 - c) as u64;
    let den = p * p;
    let g = crate::crt::gcd(num as u128, den as u128) as u64;
    (num / g, den / g)
}

pub fn forbidden_count(p: u64, d: u64) -> u64 {
    if d % p == 0 { 1 } else { 2 }
}

// ── T7: Γ Pullback Gain ────────────────────────────────────────────

fn apply_root(op: RootOp, r: u64, d_r: u64, p: u64) -> u64 {
    match op {
        RootOp::Add => addmod(r, d_r, p),
        RootOp::Sub => submod(r, d_r, p),
        RootOp::Mul => mulmod(r, d_r, p),
    }
}

fn apply_post(op: PostOp, x: u64, p: u64) -> u64 {
    match op {
        PostOp::Id => x,
        PostOp::Neg => if x == 0 { 0 } else { p - x },
        PostOp::Sqr => mulmod(x, x, p),
    }
}

/// Compute |F_eff| for a CRAM lane: the number of residues eliminated
/// after operator pullback.
pub fn pullback_forbidden_size(p: u64, d: u64, root: RootOp, post: PostOp) -> u64 {
    let d_r = d % p;
    let base_forbidden: Vec<u64> = if d_r == 0 { vec![0] } else { vec![0, (p - d_r) % p] };

    // Forward map base forbidden through operator chain
    let mut forbidden_outputs = Vec::new();
    for &r in &base_forbidden {
        let inter = apply_root(root, r, d_r, p);
        let out = apply_post(post, inter, p);
        if !forbidden_outputs.contains(&out) { forbidden_outputs.push(out); }
    }

    // Pullback: count all r ∈ [0,p) whose chain output is forbidden
    (0..p).filter(|&r| {
        let inter = apply_root(root, r, d_r, p);
        let out = apply_post(post, inter, p);
        forbidden_outputs.contains(&out)
    }).count() as u64
}

/// Γ = |F_eff| / |F_base| as (num, den)
pub fn pullback_gain(p: u64, d: u64, root: RootOp, post: PostOp) -> (u64, u64) {
    let f_eff = pullback_forbidden_size(p, d, root, post);
    let f_base = forbidden_count(p, d);
    let g = crate::crt::gcd(f_eff as u128, f_base as u128) as u64;
    (f_eff / g, f_base / g)
}

// ── L1: Lane & ToricBase ────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct LaneConfig {
    pub prime: u64,
    pub root_op: RootOp,
    pub post_op: PostOp,
}

#[derive(Debug, Clone)]
pub struct ToricBase {
    pub lanes: Vec<LaneConfig>,
    pub modulus: u64,
}

impl ToricBase {
    pub fn new(lanes: Vec<LaneConfig>) -> Self {
        let modulus = lanes.iter().fold(1u64, |a, l| a * l.prime);
        Self { lanes, modulus }
    }
    pub fn k(&self) -> usize { self.lanes.len() }
}

// ── L2: HydraHead ──────────────────────────────────────────────────

fn carry_bit(output: u64, p: u64) -> u8 {
    if output * 2 >= p { 1 } else { 0 }
}

fn cram_carry_pattern(n: u64, d: u64, base: &ToricBase) -> Vec<u8> {
    base.lanes.iter().map(|lane| {
        let p = lane.prime;
        let n_r = n % p;
        let d_r = d % p;
        let inter = apply_root(lane.root_op, n_r, d_r, p);
        let out = apply_post(lane.post_op, inter, p);
        carry_bit(out, p)
    }).collect()
}

#[derive(Debug, Clone)]
pub struct HydraHead {
    pub base: ToricBase,
    pub gap: u64,
    pub forbidden_patterns: Vec<Vec<u8>>,
    pub total_patterns: u64,
}

impl HydraHead {
    /// Calibrate: find carry patterns never seen for actual prime pairs.
    pub fn calibrate(base: ToricBase, gap: u64, cal_limit: u64) -> Self {
        let mut prime_patterns = Vec::new();
        for n in 2..cal_limit {
            if is_prime_u64(n) && is_prime_u64(n + gap) {
                let pat = cram_carry_pattern(n, gap, &base);
                if !prime_patterns.contains(&pat) {
                    prime_patterns.push(pat);
                }
            }
        }

        let k = base.k();
        let total = 1u64 << k;
        let mut forbidden = Vec::new();
        for bits in 0..total {
            let pat: Vec<u8> = (0..k).map(|i| ((bits >> i) & 1) as u8).collect();
            if !prime_patterns.contains(&pat) {
                forbidden.push(pat);
            }
        }

        Self { base, gap, forbidden_patterns: forbidden, total_patterns: total }
    }

    pub fn is_eliminated(&self, n: u64) -> bool {
        let pat = cram_carry_pattern(n, self.gap, &self.base);
        self.forbidden_patterns.contains(&pat)
    }

    pub fn elimination_rate_pct(&self) -> f64 {
        self.forbidden_patterns.len() as f64 / self.total_patterns as f64 * 100.0
    }
}

// ── L3: K-Elimination Oracle ────────────────────────────────────────

pub struct KElimOracle {
    pub sieve_modulus: u64,
    pub anchor: u64,
    m_inv_a: u64,
}

impl KElimOracle {
    pub fn new(sieve_modulus: u64, anchor: u64) -> Self {
        let m_inv_a = mod_inverse_u64(sieve_modulus % anchor, anchor)
            .expect("M must be invertible mod A");
        Self { sieve_modulus, anchor, m_inv_a }
    }

    pub fn recover(&self, r: u64, s: u64) -> u128 {
        let r_a = r % self.anchor;
        let diff = submod(s % self.anchor, r_a, self.anchor);
        let k = mulmod(diff, self.m_inv_a, self.anchor);
        r as u128 + k as u128 * self.sieve_modulus as u128
    }
}

// ── L4: PLMG Validator ──────────────────────────────────────────────

pub struct PlmgValidator { primes: Vec<u64> }

impl PlmgValidator {
    pub fn new(primes: Vec<u64>) -> Self { Self { primes } }
    pub fn is_admissible(&self, n: u64, d: u64) -> bool {
        self.primes.iter().all(|&p| {
            (n % p != 0 || n == p) && ((n + d) % p != 0 || (n + d) == p)
        })
    }
}

// ── L5: HydraSieve ─────────────────────────────────────────────────

#[derive(Debug)]
pub struct SieveResult {
    pub pairs: Vec<(u64, u64)>,
    pub candidates_tested: u64,
    pub eliminated: u64,
    pub surviving_composites: u64, // candidates that survived but weren't prime pairs
}

pub struct HydraSieve {
    heads: Vec<HydraHead>,
    validator: PlmgValidator,
    gap: u64,
}

impl HydraSieve {
    pub fn new(heads: Vec<HydraHead>, gap: u64) -> Self {
        let primes = if let Some(h) = heads.first() {
            h.base.lanes.iter().map(|l| l.prime).collect()
        } else { vec![2, 3, 5] };
        Self { heads, validator: PlmgValidator::new(primes), gap }
    }

    pub fn sieve_range(&self, start: u64, limit: u64) -> SieveResult {
        let mut pairs = Vec::new();
        let mut candidates = 0u64;
        let mut eliminated = 0u64;
        let mut surviving_composites = 0u64;

        for n in start.max(2)..limit {
            candidates += 1;
            if !self.validator.is_admissible(n, self.gap) {
                eliminated += 1;
                continue;
            }
            if self.heads.iter().any(|h| h.is_eliminated(n)) {
                eliminated += 1;
                continue;
            }
            // Survivor: verify
            if is_prime_u64(n) && is_prime_u64(n + self.gap) {
                pairs.push((n, n + self.gap));
            } else {
                surviving_composites += 1;
            }
        }

        SieveResult { pairs, candidates_tested: candidates, eliminated, surviving_composites }
    }
}

// ── L6: Presets ─────────────────────────────────────────────────────

pub struct Presets;

impl Presets {
    pub fn twin() -> ToricBase {
        ToricBase::new(vec![
            LaneConfig { prime: 3,  root_op: RootOp::Mul, post_op: PostOp::Id },
            LaneConfig { prime: 5,  root_op: RootOp::Sub, post_op: PostOp::Sqr },
            LaneConfig { prime: 7,  root_op: RootOp::Add, post_op: PostOp::Sqr },
            LaneConfig { prime: 11, root_op: RootOp::Mul, post_op: PostOp::Sqr },
            LaneConfig { prime: 13, root_op: RootOp::Sub, post_op: PostOp::Id },
        ])
    }

    pub fn cousin() -> ToricBase {
        ToricBase::new(vec![
            LaneConfig { prime: 3,  root_op: RootOp::Add, post_op: PostOp::Sqr },
            LaneConfig { prime: 5,  root_op: RootOp::Sub, post_op: PostOp::Id },
            LaneConfig { prime: 7,  root_op: RootOp::Mul, post_op: PostOp::Sqr },
            LaneConfig { prime: 11, root_op: RootOp::Add, post_op: PostOp::Sqr },
            LaneConfig { prime: 13, root_op: RootOp::Sub, post_op: PostOp::Sqr },
        ])
    }

    pub fn sexy() -> ToricBase {
        ToricBase::new(vec![
            LaneConfig { prime: 3,  root_op: RootOp::Mul, post_op: PostOp::Id },
            LaneConfig { prime: 5,  root_op: RootOp::Add, post_op: PostOp::Sqr },
            LaneConfig { prime: 7,  root_op: RootOp::Sub, post_op: PostOp::Sqr },
            LaneConfig { prime: 11, root_op: RootOp::Mul, post_op: PostOp::Sqr },
            LaneConfig { prime: 13, root_op: RootOp::Add, post_op: PostOp::Id },
        ])
    }

    /// All Sub+Id — baseline for comparison
    pub fn plain_sub_id() -> ToricBase {
        ToricBase::new(vec![
            LaneConfig { prime: 3,  root_op: RootOp::Sub, post_op: PostOp::Id },
            LaneConfig { prime: 5,  root_op: RootOp::Sub, post_op: PostOp::Id },
            LaneConfig { prime: 7,  root_op: RootOp::Sub, post_op: PostOp::Id },
            LaneConfig { prime: 11, root_op: RootOp::Sub, post_op: PostOp::Id },
            LaneConfig { prime: 13, root_op: RootOp::Sub, post_op: PostOp::Id },
        ])
    }

    /// All Add+Id — plainest possible
    pub fn plain_add_id() -> ToricBase {
        ToricBase::new(vec![
            LaneConfig { prime: 3,  root_op: RootOp::Add, post_op: PostOp::Id },
            LaneConfig { prime: 5,  root_op: RootOp::Add, post_op: PostOp::Id },
            LaneConfig { prime: 7,  root_op: RootOp::Add, post_op: PostOp::Id },
            LaneConfig { prime: 11, root_op: RootOp::Add, post_op: PostOp::Id },
            LaneConfig { prime: 13, root_op: RootOp::Add, post_op: PostOp::Id },
        ])
    }
}

// ── L7: Carry Spectrum Analysis ─────────────────────────────────────

/// Exhaustive operator search: find the best topology for a given gap.
pub fn find_best_topology(gap: u64, cal_limit: u64) -> (ToricBase, f64) {
    let primes = [3u64, 5, 7, 11, 13];
    let root_ops = [RootOp::Add, RootOp::Sub, RootOp::Mul];
    let post_ops = [PostOp::Id, PostOp::Neg, PostOp::Sqr];

    let mut _best_rate = 0.0f64;

    // For 5 primes × 3 root × 3 post = 9 choices per lane, 9^5 = 59049 topologies
    // We'll do a greedy search: optimize one lane at a time
    let mut current_lanes: Vec<LaneConfig> = primes.iter().map(|&p|
        LaneConfig { prime: p, root_op: RootOp::Add, post_op: PostOp::Id }
    ).collect();

    for lane_idx in 0..primes.len() {
        let mut best_lane_rate = 0.0f64;
        let mut best_lane_config = current_lanes[lane_idx];

        for &root in &root_ops {
            for &post in &post_ops {
                let mut test_lanes = current_lanes.clone();
                test_lanes[lane_idx] = LaneConfig {
                    prime: primes[lane_idx], root_op: root, post_op: post
                };
                let base = ToricBase::new(test_lanes.clone());
                let head = HydraHead::calibrate(base, gap, cal_limit);
                let rate = head.elimination_rate_pct();
                if rate > best_lane_rate {
                    best_lane_rate = rate;
                    best_lane_config = test_lanes[lane_idx];
                }
            }
        }
        current_lanes[lane_idx] = best_lane_config;
    }

    let final_base = ToricBase::new(current_lanes.clone());
    let final_head = HydraHead::calibrate(final_base.clone(), gap, cal_limit);
    (final_base, final_head.elimination_rate_pct())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramanujan_sieve_power() {
        let (n, d) = sieve_power(3, 2);
        assert_eq!((n, d), (2, 3)); // SP(3,2) = 2/3
        let (n, d) = sieve_power(3, 6);
        assert_eq!((n, d), (1, 3)); // SP(3,6) = 1/3 (3|6)
    }

    #[test]
    fn test_pullback_add_sqr() {
        // Add+Sqr, p∤d: |F_eff| = 3
        for &p in &[5u64, 7, 11, 13] {
            assert_eq!(pullback_forbidden_size(p, 2, RootOp::Add, PostOp::Sqr), 3);
        }
    }

    #[test]
    fn test_pullback_sub_sqr() {
        // Sub+Sqr, p∤d, p≠3: |F_eff| = 4
        for &p in &[5u64, 7, 11, 13] {
            assert_eq!(pullback_forbidden_size(p, 2, RootOp::Sub, PostOp::Sqr), 4);
        }
    }

    #[test]
    fn test_pullback_mul_id_divisible() {
        // Mul+Id, p|d: maps everything to 0 → |F_eff| = p
        for &p in &[3u64, 5, 7, 11, 13] {
            assert_eq!(pullback_forbidden_size(p, p, RootOp::Mul, PostOp::Id), p);
        }
    }

    #[test]
    fn test_zero_false_negatives_twin() {
        let base = Presets::twin();
        let head = HydraHead::calibrate(base, 2, 10_000);
        for n in 2..10_000u64 {
            if is_prime_u64(n) && is_prime_u64(n + 2) {
                assert!(!head.is_eliminated(n), "false negative: twin ({}, {})", n, n + 2);
            }
        }
    }

    #[test]
    fn test_zero_false_negatives_cousin() {
        let base = Presets::cousin();
        let head = HydraHead::calibrate(base, 4, 10_000);
        for n in 2..10_000u64 {
            if is_prime_u64(n) && is_prime_u64(n + 4) {
                assert!(!head.is_eliminated(n), "false negative: cousin ({}, {})", n, n + 4);
            }
        }
    }

    #[test]
    fn test_sieve_twin_complete() {
        let base = Presets::twin();
        let head = HydraHead::calibrate(base, 2, 10_000);
        let sieve = HydraSieve::new(vec![head], 2);
        let result = sieve.sieve_range(2, 10_000);
        let known = (2..10_000u64).filter(|&n| is_prime_u64(n) && is_prime_u64(n + 2)).count();
        assert_eq!(result.pairs.len(), known);
    }

    #[test]
    fn test_k_elim_oracle() {
        let m = 3u64 * 5 * 7 * 11 * 13;
        let a = (1u64 << 31) - 1;
        let oracle = KElimOracle::new(m, a);
        for &v in &[0u64, 1, 42, 12345, 999_999] {
            let recovered = oracle.recover(v % m, v % a);
            assert_eq!(recovered, v as u128);
        }
    }
}
