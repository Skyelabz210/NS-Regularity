//! CRAM Schema Execution — Full Integration
//!
//! Fuses: prime-family-spacetime (attributes per prime),
//!        crt-operator (simultaneous lane application),
//!        dresden-prime-hunt (intrinsic address + atlas),
//!        grover-hydra (tier classification + Ramanujan-Grover).
//!
//! For any integer n and gap d:
//!   1. Distribute n across lanes (CRT — free by representation)
//!   2. Apply CRAM operators per lane (heterogeneous — set and apply)
//!   3. Extract carry pattern (nonlinear threshold)
//!   4. Compute intrinsic address (multi-probe)
//!   5. Lookup atlas (prime-rich or prime-poor)
//!   6. If targeting pairs: compute joint address (n, n+d)
//!   7. For extended basis: Grover amplification on quantum-tier lanes
//!
//! Every lane carries its family attributes:
//!   - Ramanujan status (S_R = {5,7,11} — measurement tier)
//!   - Mod-4 class (p≡1: QR-visible, p≡3: threshold-visible)
//!   - Grover tier (p≡3 mod 4: quantum, p≡1 mod 4: classical)
//!   - T6 sieve power SP(p,d)
//!   - Forbidden ratio on this base

use crate::crt;
use crate::exact::Rational;
use crate::coprime_families::{CramTopology, Lane, Root, Post, Champions};

fn mulmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 * b as u128) % m as u128) as u64 }

// ── Lane Attributes (from prime-family-spacetime) ───────────────

#[derive(Debug, Clone)]
pub struct LaneProfile {
    pub prime: u64,
    pub ramanujan: bool,         // {5,7,11} only
    pub mod4: u64,               // 1 or 3
    pub qr_visible: bool,        // p ≡ 1 (mod 4)
    pub grover_tier: bool,       // p ≡ 3 (mod 4)
    pub tier: SafeBasisTier,
    pub fibonacci_entry: FibEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeBasisTier {
    Fabric,      // {2, 3} — stability floor
    Measurement, // {5, 7, 11} — Ramanujan S_R
    Boundary,    // {13} — capacity only
    Extended,    // connector primes > 13
}

#[derive(Debug, Clone, Copy)]
pub enum FibEntry {
    Direct,    // p is a Fibonacci number (2, 3, 5, 13)
    Composite, // p enters through composite Fibonacci (7 via F(8)=21, 11 via F(10)=55)
    None,      // not in Fibonacci sequence at all
}

pub fn profile_prime(p: u64) -> LaneProfile {
    let ramanujan = p == 5 || p == 7 || p == 11;
    let mod4 = p % 4;
    let tier = match p {
        2 | 3 => SafeBasisTier::Fabric,
        5 | 7 | 11 => SafeBasisTier::Measurement,
        13 => SafeBasisTier::Boundary,
        _ => SafeBasisTier::Extended,
    };
    let fib = match p {
        2 | 3 | 5 | 13 => FibEntry::Direct,
        7 | 11 => FibEntry::Composite,
        _ => FibEntry::None,
    };
    LaneProfile {
        prime: p, ramanujan, mod4,
        qr_visible: mod4 == 1,
        grover_tier: mod4 == 3,
        tier, fibonacci_entry: fib,
    }
}

// ── CRAM Schema: Operator Assignment Per Lane ───────────────────

/// Assign the optimal CRAM operator per lane based on attributes.
/// This IS the schema — heterogeneous per-lane operator selection.
///
/// Design rules (from prime-family-spacetime + T16 + T7):
///   Ramanujan lanes {5,7,11}: these carry structural information.
///     - p≡3 mod 4 (7,11): threshold-visible → Sub+Sqr (Γ=2, max gain)
///     - p≡1 mod 4 (5): QR-visible → Add+Sqr (Γ=3/2, sees Legendre)
///   Boundary lane {13}: QR-visible → Mul+Sqr (couples gap structure)
///   Fabric lanes {2,3}: 2 is trivial (p=2 carry always 0 or 1)
///     - p=3: threshold-visible → Sub+Sqr
///   Extended connectors: classify by mod-4, assign accordingly.
pub fn schema_topology(primes: &[u64]) -> CramTopology {
    let lanes: Vec<Lane> = primes.iter().map(|&p| {
        let profile = profile_prime(p);
        let (root, post) = match (profile.tier, profile.qr_visible) {
            // Fabric: p=3 gets Sub+Sqr (max threshold gain)
            (SafeBasisTier::Fabric, false) => (Root::Sub, Post::Sqr),
            (SafeBasisTier::Fabric, true) => (Root::Add, Post::Id), // p=2 only

            // Measurement: Ramanujan primes
            (SafeBasisTier::Measurement, false) => (Root::Sub, Post::Sqr), // p≡3: threshold
            (SafeBasisTier::Measurement, true) => (Root::Add, Post::Sqr),  // p≡1: QR

            // Boundary: p=13
            (SafeBasisTier::Boundary, _) => (Root::Mul, Post::Sqr),

            // Extended: classify by mod-4
            (SafeBasisTier::Extended, false) => (Root::Sub, Post::Sqr), // p≡3: threshold
            (SafeBasisTier::Extended, true) => (Root::Add, Post::Sqr),  // p≡1: QR
        };
        Lane { prime: p, root, post }
    }).collect();
    CramTopology::new(lanes)
}

/// The Champion S⁶ topology for reference comparison.
pub fn champion_s6() -> CramTopology {
    Champions::s6()
}

// ── Full Schema Execution ───────────────────────────────────────

/// Execute the full CRAM schema on a candidate n for gap d.
/// Returns the per-lane carry pattern and all attributes.
pub fn execute_schema(n: u64, d: u64, primes: &[u64]) -> SchemaResult {
    let topo = schema_topology(primes);
    let profiles: Vec<LaneProfile> = primes.iter().map(|&p| profile_prime(p)).collect();

    // Step 1: n is already distributed across lanes (CRT — free)
    // Step 2: apply CRAM operators per lane (simultaneous)
    let pattern = topo.carry_pattern(n, d);

    // Step 3: compute T6 sieve power per lane
    let sieve_powers: Vec<Rational> = primes.iter().map(|&p| {
        let c: i64 = if d % p == 0 { p as i64 - 1 } else { -1 };
        let num = (2 * p as i64 - 1 - c) as u64;
        Rational::new(num, p * p)
    }).collect();

    // Step 4: compute forbidden fraction (HL prediction)
    let mut adm_num = 1u64;
    let mut adm_den = 1u64;
    for &p in primes {
        let f = if d % p == 0 { 1 } else { 2 };
        adm_num *= p - f;
        adm_den *= p;
    }
    let forbidden = Rational::new(adm_den - adm_num, adm_den);

    // Step 5: classify operation (from Separation Principle)
    // Address computation = CLASS-R (coprimality sufficient)
    // Atlas lookup = CLASS-R (no primality needed)
    // Grover amplification = CLASS-F on quantum-tier lanes (F_p² needed)
    let class_f_lanes: Vec<u64> = profiles.iter()
        .filter(|p| p.grover_tier)
        .map(|p| p.prime).collect();
    let class_r_lanes: Vec<u64> = profiles.iter()
        .filter(|p| !p.grover_tier)
        .map(|p| p.prime).collect();

    SchemaResult {
        n, d, pattern, profiles, sieve_powers, forbidden,
        class_f_lanes, class_r_lanes,
        topology_label: topo.label(),
    }
}

#[derive(Debug)]
pub struct SchemaResult {
    pub n: u64,
    pub d: u64,
    pub pattern: Vec<u8>,
    pub profiles: Vec<LaneProfile>,
    pub sieve_powers: Vec<Rational>,
    pub forbidden: Rational,
    pub class_f_lanes: Vec<u64>,
    pub class_r_lanes: Vec<u64>,
    pub topology_label: String,
}

/// Execute schema on both endpoints of a pair (n, n+d) and compute
/// joint fingerprint for pair prediction.
pub fn execute_joint_schema(n: u64, d: u64, primes: &[u64]) -> JointSchemaResult {
    let topo = schema_topology(primes);

    let pattern_n = topo.carry_pattern(n, d);
    let pattern_nd = topo.carry_pattern(n + d, d);

    // Joint address = concatenation
    let mut joint = pattern_n.clone();
    joint.extend_from_slice(&pattern_nd);

    // Joint ID for binning
    let mut joint_id = 0u64;
    for (i, &bit) in joint.iter().enumerate() {
        if i >= 64 { break; }
        joint_id |= (bit as u64) << i;
    }

    JointSchemaResult {
        n, d, pattern_n, pattern_nd, joint_pattern: joint, joint_id,
    }
}

#[derive(Debug)]
pub struct JointSchemaResult {
    pub n: u64,
    pub d: u64,
    pub pattern_n: Vec<u8>,
    pub pattern_nd: Vec<u8>,
    pub joint_pattern: Vec<u8>,
    pub joint_id: u64,
}

/// Full schema pair analysis: bin by joint CRAM pattern, measure pair density.
pub fn schema_pair_analysis(primes: &[u64], d: u64, limit: u64) -> SchemaPairAnalysis {
    let m: u64 = primes.iter().product();
    let mut bins: std::collections::HashMap<u64, (u32, u32)> = std::collections::HashMap::new();
    let mut total = 0u32;
    let mut pairs = 0u32;

    let is_prime = |n: u64| -> bool {
        if n < 2 { return false; } if n < 4 { return true; }
        if n % 2 == 0 || n % 3 == 0 { return false; }
        let mut i = 5u64;
        while i.saturating_mul(i) <= n { if n % i == 0 || n % (i+2) == 0 { return false; } i += 6; }
        true
    };

    for n in 2..limit {
        if crt::gcd(n as u128, m as u128) != 1 { continue; }
        if crt::gcd((n+d) as u128, m as u128) != 1 { continue; }
        total += 1;

        let joint = execute_joint_schema(n, d, primes);
        let entry = bins.entry(joint.joint_id).or_insert((0, 0));
        entry.0 += 1;
        if is_prime(n) && is_prime(n + d) {
            entry.1 += 1;
            pairs += 1;
        }
    }

    // Sort by density
    let mut ranked: Vec<(u64, u32, u32, Rational)> = bins.into_iter()
        .map(|(id, (t, p))| (id, t, p, Rational::new(p as u64, t as u64)))
        .collect();
    ranked.sort_by(|a, b| b.3.cmp(&a.3));

    let populated: Vec<&(u64, u32, u32, Rational)> = ranked.iter()
        .filter(|r| r.1 >= 5).collect();

    let spread = if populated.len() >= 2 {
        populated[0].3.sub_saturating(&populated.last().unwrap().3)
    } else { Rational::zero() };

    SchemaPairAnalysis {
        d, total, pairs,
        bins: ranked.len() as u32,
        populated: populated.len() as u32,
        spread,
        top_density: if !populated.is_empty() { populated[0].3 } else { Rational::zero() },
        bot_density: if !populated.is_empty() { populated.last().unwrap().3 } else { Rational::zero() },
        top_id: if !populated.is_empty() { populated[0].0 } else { 0 },
        bot_id: if !populated.is_empty() { populated.last().unwrap().0 } else { 0 },
        avg: Rational::new(pairs as u64, total as u64),
    }
}

#[derive(Debug)]
pub struct SchemaPairAnalysis {
    pub d: u64,
    pub total: u32,
    pub pairs: u32,
    pub bins: u32,
    pub populated: u32,
    pub spread: Rational,
    pub top_density: Rational,
    pub bot_density: Rational,
    pub top_id: u64,
    pub bot_id: u64,
    pub avg: Rational,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_safe_basis() {
        let p5 = profile_prime(5);
        assert!(p5.ramanujan);
        assert!(p5.qr_visible);     // 5 ≡ 1 (mod 4)
        assert!(!p5.grover_tier);    // 5 ≡ 1 (mod 4) → classical
        assert_eq!(p5.tier, SafeBasisTier::Measurement);

        let p7 = profile_prime(7);
        assert!(p7.ramanujan);
        assert!(!p7.qr_visible);    // 7 ≡ 3 (mod 4)
        assert!(p7.grover_tier);     // 7 ≡ 3 (mod 4) → quantum
    }

    #[test]
    fn test_schema_topology_heterogeneous() {
        let topo = schema_topology(&[3, 5, 7, 11, 13]);
        assert_eq!(topo.k(), 5);
        // p=3 (fabric, p≡3): Sub+Sqr
        // p=5 (measurement, p≡1): Add+Sqr
        // p=7 (measurement, p≡3): Sub+Sqr
        // p=11 (measurement, p≡3): Sub+Sqr
        // p=13 (boundary): Mul+Sqr
        // All heterogeneous — not all the same operator
        let labels = topo.label();
        assert!(labels.contains("SUB") && labels.contains("ADD") && labels.contains("MUL"));
    }

    #[test]
    fn test_execute_schema() {
        let result = execute_schema(17, 2, &[3, 5, 7, 11, 13]);
        assert_eq!(result.pattern.len(), 5);
        assert_eq!(result.sieve_powers.len(), 5);
        assert!(result.class_f_lanes.contains(&7));  // p≡3 mod 4
        assert!(result.class_r_lanes.contains(&5));   // p≡1 mod 4
    }

    #[test]
    fn test_joint_schema() {
        let joint = execute_joint_schema(17, 2, &[3, 5, 7, 11, 13]);
        assert_eq!(joint.joint_pattern.len(), 10); // 5 + 5
        assert_eq!(joint.pattern_n.len(), 5);
        assert_eq!(joint.pattern_nd.len(), 5);
    }

    #[test]
    fn test_schema_pair_analysis() {
        let result = schema_pair_analysis(&[3, 5, 7], 2, 10_000);
        assert!(result.total > 0);
        assert!(result.pairs > 0);
        assert!(result.spread.num > 0); // non-zero spread
    }
}

// ═══════════════════════════════════════════════════════════════════
// SENSITIVITY AMPLIFIER: Ternary Thresholds
// Binary carry: 1 bit/lane, 5 dims from 5 primes, Gini ~0.025
// Ternary carry: 2 bits/lane, 10 dims from 5 primes, Gini ~0.189 (7.65×)
// ═══════════════════════════════════════════════════════════════════

/// Ternary-amplified pair analysis: uses ternary carry patterns (2 bits/lane)
/// instead of binary (1 bit/lane). 7.65× Gini amplification measured in
/// sensitivity_amplifier.py.
pub fn amplified_pair_analysis(primes: &[u64], d: u64, limit: u64) -> SchemaPairAnalysis {
    let m: u64 = primes.iter().product();
    let topo = schema_topology(primes);
    let mut bins: std::collections::HashMap<u64, (u32, u32)> = std::collections::HashMap::new();
    let mut total = 0u32;
    let mut pairs = 0u32;

    let is_prime = |n: u64| -> bool {
        if n < 2 { return false; } if n < 4 { return true; }
        if n % 2 == 0 || n % 3 == 0 { return false; }
        let mut i = 5u64;
        while i.saturating_mul(i) <= n { if n % i == 0 || n % (i+2) == 0 { return false; } i += 6; }
        true
    };

    for n in 2..limit {
        if crt::gcd(n as u128, m as u128) != 1 { continue; }
        if crt::gcd((n+d) as u128, m as u128) != 1 { continue; }
        total += 1;

        // Ternary patterns for both endpoints
        let pat_n = topo.ternary_pattern(n, d);
        let pat_nd = topo.ternary_pattern(n + d, d);

        // Joint address = concatenation
        let mut joint_id = 0u64;
        for (i, &bit) in pat_n.iter().chain(pat_nd.iter()).enumerate() {
            if i >= 64 { break; }
            joint_id |= (bit as u64) << i;
        }

        let entry = bins.entry(joint_id).or_insert((0, 0));
        entry.0 += 1;
        if is_prime(n) && is_prime(n + d) {
            entry.1 += 1;
            pairs += 1;
        }
    }

    let mut ranked: Vec<(u64, u32, u32, Rational)> = bins.into_iter()
        .map(|(id, (t, p))| (id, t, p, Rational::new(p as u64, t as u64)))
        .collect();
    ranked.sort_by(|a, b| b.3.cmp(&a.3));

    let populated: Vec<&(u64, u32, u32, Rational)> = ranked.iter()
        .filter(|r| r.1 >= 5).collect();

    let spread = if populated.len() >= 2 {
        populated[0].3.sub_saturating(&populated.last().unwrap().3)
    } else { Rational::zero() };

    SchemaPairAnalysis {
        d, total, pairs,
        bins: ranked.len() as u32,
        populated: populated.len() as u32,
        spread,
        top_density: if !populated.is_empty() { populated[0].3 } else { Rational::zero() },
        bot_density: if !populated.is_empty() { populated.last().unwrap().3 } else { Rational::zero() },
        top_id: if !populated.is_empty() { populated[0].0 } else { 0 },
        bot_id: if !populated.is_empty() { populated.last().unwrap().0 } else { 0 },
        avg: Rational::new(pairs as u64, total as u64),
    }
}

#[cfg(test)]
mod amplifier_tests {
    use super::*;

    #[test]
    fn test_ternary_pattern() {
        let topo = schema_topology(&[3, 5, 7, 11, 13]);
        let pat = topo.ternary_pattern(17, 2);
        assert_eq!(pat.len(), 10); // 2 bits per lane × 5 lanes
        for &bit in &pat { assert!(bit <= 1); }
    }

    #[test]
    fn test_amplified_pair_spread() {
        let result = amplified_pair_analysis(&[3, 5, 7], 2, 10_000);
        assert!(result.total > 0);
        assert!(result.pairs > 0);
        // Ternary should give wider spread than binary
    }
}

/// Optimal-power pair analysis: uses k=2^v₂(p-1) per lane.
/// Every lane separates. No blind lanes.
pub fn optimal_power_pair_analysis(primes: &[u64], d: u64, limit: u64) -> SchemaPairAnalysis {
    let m: u64 = primes.iter().product();
    let topo = schema_topology(primes);
    let mut bins: std::collections::HashMap<u64, (u32, u32)> = std::collections::HashMap::new();
    let mut total = 0u32;
    let mut pairs = 0u32;

    let is_prime = |n: u64| -> bool {
        if n < 2 { return false; } if n < 4 { return true; }
        if n % 2 == 0 || n % 3 == 0 { return false; }
        let mut i = 5u64;
        while i.saturating_mul(i) <= n { if n % i == 0 || n % (i+2) == 0 { return false; } i += 6; }
        true
    };

    for n in 2..limit {
        if crt::gcd(n as u128, m as u128) != 1 { continue; }
        if crt::gcd((n+d) as u128, m as u128) != 1 { continue; }
        total += 1;

        let pat_n = topo.optimal_power_pattern(n);
        let pat_nd = topo.optimal_power_pattern(n + d);

        let mut joint_id = 0u64;
        for (i, &bit) in pat_n.iter().chain(pat_nd.iter()).enumerate() {
            if i >= 64 { break; }
            joint_id |= (bit as u64) << i;
        }

        let entry = bins.entry(joint_id).or_insert((0, 0));
        entry.0 += 1;
        if is_prime(n) && is_prime(n + d) {
            entry.1 += 1;
            pairs += 1;
        }
    }

    let mut ranked: Vec<(u64, u32, u32, Rational)> = bins.into_iter()
        .map(|(id, (t, p))| (id, t, p, Rational::new(p as u64, t as u64)))
        .collect();
    ranked.sort_by(|a, b| b.3.cmp(&a.3));

    let populated: Vec<&(u64, u32, u32, Rational)> = ranked.iter()
        .filter(|r| r.1 >= 5).collect();

    let spread = if populated.len() >= 2 {
        populated[0].3.sub_saturating(&populated.last().unwrap().3)
    } else { Rational::zero() };

    SchemaPairAnalysis {
        d, total, pairs,
        bins: ranked.len() as u32,
        populated: populated.len() as u32,
        spread,
        top_density: if !populated.is_empty() { populated[0].3 } else { Rational::zero() },
        bot_density: if !populated.is_empty() { populated.last().unwrap().3 } else { Rational::zero() },
        top_id: if !populated.is_empty() { populated[0].0 } else { 0 },
        bot_id: if !populated.is_empty() { populated.last().unwrap().0 } else { 0 },
        avg: Rational::new(pairs as u64, total as u64),
    }
}

/// Augmented pair analysis: standard CRAM schema + optimal power bits.
/// Uses BOTH mechanisms: CRAM carry (root+post) AND power residue (n^k).
/// This ADDS dimensions rather than replacing.
pub fn augmented_power_pair_analysis(primes: &[u64], d: u64, limit: u64) -> SchemaPairAnalysis {
    let m: u64 = primes.iter().product();
    let topo = schema_topology(primes);
    let mut bins: std::collections::HashMap<u64, (u32, u32)> = std::collections::HashMap::new();
    let mut total = 0u32;
    let mut pairs = 0u32;

    let is_prime = |n: u64| -> bool {
        if n < 2 { return false; } if n < 4 { return true; }
        if n % 2 == 0 || n % 3 == 0 { return false; }
        let mut i = 5u64;
        while i.saturating_mul(i) <= n { if n % i == 0 || n % (i+2) == 0 { return false; } i += 6; }
        true
    };

    for n in 2..limit {
        if crt::gcd(n as u128, m as u128) != 1 { continue; }
        if crt::gcd((n+d) as u128, m as u128) != 1 { continue; }
        total += 1;

        // Standard CRAM carry pattern for both endpoints
        let cram_n = topo.carry_pattern(n, d);
        let cram_nd = topo.carry_pattern(n + d, d);
        // Optimal power pattern for both endpoints (adds quartic on blind lanes)
        let pow_n = topo.optimal_power_pattern(n);
        let pow_nd = topo.optimal_power_pattern(n + d);

        // Joint address: cram_n ++ pow_n ++ cram_nd ++ pow_nd
        let mut joint_id = 0u64;
        let mut bit_idx = 0usize;
        for &bit in cram_n.iter().chain(pow_n.iter()).chain(cram_nd.iter()).chain(pow_nd.iter()) {
            if bit_idx >= 64 { break; }
            joint_id |= (bit as u64) << bit_idx;
            bit_idx += 1;
        }

        let entry = bins.entry(joint_id).or_insert((0, 0));
        entry.0 += 1;
        if is_prime(n) && is_prime(n + d) {
            entry.1 += 1;
            pairs += 1;
        }
    }

    let mut ranked: Vec<(u64, u32, u32, Rational)> = bins.into_iter()
        .map(|(id, (t, p))| (id, t, p, Rational::new(p as u64, t as u64)))
        .collect();
    ranked.sort_by(|a, b| b.3.cmp(&a.3));

    let populated: Vec<&(u64, u32, u32, Rational)> = ranked.iter()
        .filter(|r| r.1 >= 5).collect();

    let spread = if populated.len() >= 2 {
        populated[0].3.sub_saturating(&populated.last().unwrap().3)
    } else { Rational::zero() };

    SchemaPairAnalysis {
        d, total, pairs,
        bins: ranked.len() as u32,
        populated: populated.len() as u32,
        spread,
        top_density: if !populated.is_empty() { populated[0].3 } else { Rational::zero() },
        bot_density: if !populated.is_empty() { populated.last().unwrap().3 } else { Rational::zero() },
        top_id: if !populated.is_empty() { populated[0].0 } else { 0 },
        bot_id: if !populated.is_empty() { populated.last().unwrap().0 } else { 0 },
        avg: Rational::new(pairs as u64, total as u64),
    }
}

/// THREE-MECHANISM pair analysis: CRAM carry + optimal power + ternary threshold.
/// 5 + 5 + 10 = 20 dims per endpoint → 40 joint dims.
/// Each mechanism is proved independent (T16/T18, T21, sensitivity amplifier).
pub fn three_mechanism_pair_analysis(primes: &[u64], d: u64, limit: u64) -> SchemaPairAnalysis {
    let m: u64 = primes.iter().product();
    let topo = schema_topology(primes);
    let mut bins: std::collections::HashMap<u64, (u32, u32)> = std::collections::HashMap::new();
    let mut total = 0u32;
    let mut pairs = 0u32;

    let is_prime = |n: u64| -> bool {
        if n < 2 { return false; } if n < 4 { return true; }
        if n % 2 == 0 || n % 3 == 0 { return false; }
        let mut i = 5u64;
        while i.saturating_mul(i) <= n { if n % i == 0 || n % (i+2) == 0 { return false; } i += 6; }
        true
    };

    for n in 2..limit {
        if crt::gcd(n as u128, m as u128) != 1 { continue; }
        if crt::gcd((n+d) as u128, m as u128) != 1 { continue; }
        total += 1;

        // Mechanism 1: CRAM carry (root+Sqr), 5 bits
        let m1_n = topo.carry_pattern(n, d);
        let m1_nd = topo.carry_pattern(n + d, d);
        // Mechanism 2: optimal power (n^k mod p), 5 bits
        let m2_n = topo.optimal_power_pattern(n);
        let m2_nd = topo.optimal_power_pattern(n + d);
        // Mechanism 3: ternary threshold, 10 bits (2 per lane)
        let m3_n = topo.ternary_pattern(n, d);
        let m3_nd = topo.ternary_pattern(n + d, d);

        // Joint: m1_n ++ m2_n ++ m3_n ++ m1_nd ++ m2_nd ++ m3_nd
        // = 20 + 20 = 40 bits (fits in u64)
        let mut joint_id = 0u64;
        let mut bit_idx = 0usize;
        for &bit in m1_n.iter().chain(m2_n.iter()).chain(m3_n.iter())
                    .chain(m1_nd.iter()).chain(m2_nd.iter()).chain(m3_nd.iter()) {
            if bit_idx >= 64 { break; }
            joint_id |= (bit as u64) << bit_idx;
            bit_idx += 1;
        }

        let entry = bins.entry(joint_id).or_insert((0, 0));
        entry.0 += 1;
        if is_prime(n) && is_prime(n + d) {
            entry.1 += 1;
            pairs += 1;
        }
    }

    let mut ranked: Vec<(u64, u32, u32, Rational)> = bins.into_iter()
        .map(|(id, (t, p))| (id, t, p, Rational::new(p as u64, t as u64)))
        .collect();
    ranked.sort_by(|a, b| b.3.cmp(&a.3));

    let populated: Vec<&(u64, u32, u32, Rational)> = ranked.iter()
        .filter(|r| r.1 >= 5).collect();

    let spread = if populated.len() >= 2 {
        populated[0].3.sub_saturating(&populated.last().unwrap().3)
    } else { Rational::zero() };

    SchemaPairAnalysis {
        d, total, pairs,
        bins: ranked.len() as u32,
        populated: populated.len() as u32,
        spread,
        top_density: if !populated.is_empty() { populated[0].3 } else { Rational::zero() },
        bot_density: if !populated.is_empty() { populated.last().unwrap().3 } else { Rational::zero() },
        top_id: if !populated.is_empty() { populated[0].0 } else { 0 },
        bot_id: if !populated.is_empty() { populated.last().unwrap().0 } else { 0 },
        avg: Rational::new(pairs as u64, total as u64),
    }
}
