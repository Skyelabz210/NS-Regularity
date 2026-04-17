//! # Apollonian Reflections + CRAM Schema Oracle
//!
//! THE THESIS: Collatz inversion and AHOP orbit recovery are hard for the
//! same reason (non-commutative forward composition). Therefore they share
//! the same Achilles heel: knowing a SINGLE intermediate state collapses
//! exponential search to two polynomial halves.
//!
//! The shadow prime lane r(t) mod 11 IS that intermediate state.
//! A CRAM schema is the extraction mechanism.
//!
//! This module:
//! 1. Implements Apollonian reflections as a TorusMap
//! 2. Implements the CRAM schema oracle (shadow lane reader)
//! 3. Tests whether the shadow signal distinguishes orbits
//! 4. Measures the information content of the shadow lane

use crate::mod_arith::*;
use std::collections::HashMap;

// ============================================================
// APOLLONIAN REFLECTIONS ON Z/PZ
// ============================================================

/// A curvature 4-tuple in Z/PZ.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CurvatureTuple {
    pub k: [i64; 4],
}

impl CurvatureTuple {
    pub fn new(k0: i64, k1: i64, k2: i64, k3: i64) -> Self {
        CurvatureTuple { k: [k0, k1, k2, k3] }
    }

    /// Standard Apollonian seed: (-1, 2, 2, 3)
    pub fn standard_seed() -> Self {
        Self::new(-1, 2, 2, 3)
    }

    /// Reduce all components mod p
    pub fn reduce(&self, p: i64) -> Self {
        CurvatureTuple {
            k: [modd(self.k[0], p), modd(self.k[1], p),
                modd(self.k[2], p), modd(self.k[3], p)]
        }
    }

    /// Check Descartes relation: (Σk)² = 2Σ(k²)
    pub fn check_descartes(&self, p: i64) -> bool {
        let sum: i64 = self.k.iter().map(|&x| modd(x, p)).sum();
        let sum_sq = mulmod(sum, sum, p);
        let sq_sum: i64 = self.k.iter()
            .map(|&x| { let r = modd(x, p); mulmod(r, r, p) })
            .sum::<i64>();
        let two_sq_sum = modd(2 * sq_sum, p);
        modd(sum_sq, p) == modd(two_sq_sum, p)
    }

    /// Apply reflection G_i: replace k[i] with 2*(sum of others) - k[i]
    pub fn reflect(&self, i: usize, p: i64) -> Self {
        assert!(i < 4);
        let mut result = *self;
        let others_sum: i64 = (0..4)
            .filter(|&j| j != i)
            .map(|j| modd(self.k[j], p))
            .sum();
        result.k[i] = modd(2 * others_sum - self.k[i], p);
        result
    }

    /// Apply a word (sequence of reflection indices) to this tuple
    pub fn apply_word(&self, word: &[usize], p: i64) -> Self {
        let mut current = self.reduce(p);
        for &i in word {
            current = current.reflect(i, p);
        }
        current
    }

    /// Extract shadow lane: project to Z/11
    pub fn shadow_signal(&self) -> [i64; 4] {
        [modd(self.k[0], 11), modd(self.k[1], 11),
         modd(self.k[2], 11), modd(self.k[3], 11)]
    }

    /// Full CRT decomposition on Safe Basis
    pub fn crt_decompose(&self, primes: &[i64]) -> Vec<[i64; 4]> {
        primes.iter().map(|&p| {
            [modd(self.k[0], p), modd(self.k[1], p),
             modd(self.k[2], p), modd(self.k[3], p)]
        }).collect()
    }
}

impl std::fmt::Display for CurvatureTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({},{},{},{})", self.k[0], self.k[1], self.k[2], self.k[3])
    }
}

// ============================================================
// CRAM SCHEMA ORACLE — The Shadow Lane Reader
// ============================================================

/// A CRAM schema for extracting information from Apollonian orbits.
/// The schema defines which lane to read and what operator to apply.
#[derive(Clone, Debug)]
pub struct CRAMSchema {
    /// The shadow prime (lane to read)
    pub shadow_prime: i64,
    /// The basis primes for full decomposition
    pub basis: Vec<i64>,
    /// Name of this schema
    pub name: String,
}

impl CRAMSchema {
    /// Standard shadow schema: read lane 11 on Safe Basis
    pub fn shadow_standard() -> Self {
        CRAMSchema {
            shadow_prime: 11,
            basis: vec![3, 5, 7, 11, 13],
            name: "Shadow-11 Standard".to_string(),
        }
    }

    /// Read the shadow signal from a curvature tuple
    pub fn read_shadow(&self, tuple: &CurvatureTuple) -> [i64; 4] {
        let p = self.shadow_prime;
        [modd(tuple.k[0], p), modd(tuple.k[1], p),
         modd(tuple.k[2], p), modd(tuple.k[3], p)]
    }

    /// Read full CRT fingerprint
    pub fn read_full(&self, tuple: &CurvatureTuple) -> Vec<[i64; 4]> {
        tuple.crt_decompose(&self.basis)
    }

    /// Extract the shadow trace of an entire orbit (word applied step by step).
    /// Returns the shadow signal at each step.
    pub fn trace_orbit_shadow(
        &self,
        seed: &CurvatureTuple,
        word: &[usize],
        p: i64,
    ) -> Vec<[i64; 4]> {
        let mut current = seed.reduce(p);
        let mut trace = vec![self.read_shadow(&current)];
        for &i in word {
            current = current.reflect(i, p);
            trace.push(self.read_shadow(&current));
        }
        trace
    }
}

// ============================================================
// ACHILLES HEEL TEST — Can shadow signal distinguish words?
// ============================================================

/// Result of the Achilles heel test.
#[derive(Clone, Debug)]
pub struct AchillesResult {
    /// How many distinct words were tested
    pub words_tested: usize,
    /// How many distinct shadow traces were observed
    pub distinct_shadow_traces: usize,
    /// How many distinct endpoints were observed
    pub distinct_endpoints: usize,
    /// Whether shadow traces uniquely identify words (the heel exists)
    pub shadow_is_unique: bool,
    /// Collision count: pairs of different words with same shadow trace
    pub shadow_collisions: usize,
    /// Maximum number of words sharing the same shadow trace
    pub max_collision_class: usize,
    /// Information content: log2(distinct_traces) / log2(words_tested)
    pub information_ratio: f64, // one allowed float for reporting only
}

/// Test the Achilles heel hypothesis.
///
/// Generate all words of given length, apply them to the seed,
/// and check whether the shadow trace uniquely identifies the word.
///
/// If shadow traces are unique → the shadow lane IS the Achilles heel.
/// If collisions occur → measure how much information the shadow provides.
pub fn achilles_heel_test(
    seed: &CurvatureTuple,
    p: i64,
    word_length: usize,
    schema: &CRAMSchema,
) -> AchillesResult {
    // Generate all 4^word_length words (skip consecutive repeats since G_i² = id)
    let mut words: Vec<Vec<usize>> = Vec::new();
    generate_reduced_words(word_length, &mut words, &mut Vec::new(), 5); // 5 = no previous

    let mut shadow_map: HashMap<Vec<[i64; 4]>, Vec<Vec<usize>>> = HashMap::new();
    let mut endpoint_set: std::collections::HashSet<CurvatureTuple> = std::collections::HashSet::new();

    for word in &words {
        let trace = schema.trace_orbit_shadow(seed, word, p);
        let endpoint = seed.apply_word(word, p);
        endpoint_set.insert(endpoint);
        shadow_map.entry(trace).or_default().push(word.clone());
    }

    let distinct_traces = shadow_map.len();
    let collisions: usize = shadow_map.values()
        .filter(|v| v.len() > 1)
        .map(|v| v.len() * (v.len() - 1) / 2)
        .sum();
    let max_class = shadow_map.values().map(|v| v.len()).max().unwrap_or(0);

    let info_ratio = if words.len() > 1 {
        (distinct_traces as f64).log2() / (words.len() as f64).log2()
    } else { 1.0 };

    AchillesResult {
        words_tested: words.len(),
        distinct_shadow_traces: distinct_traces,
        distinct_endpoints: endpoint_set.len(),
        shadow_is_unique: collisions == 0,
        shadow_collisions: collisions,
        max_collision_class: max_class,
        information_ratio: info_ratio,
    }
}

/// Generate all reduced words (no consecutive same reflections, since G_i² = id).
fn generate_reduced_words(
    remaining: usize,
    results: &mut Vec<Vec<usize>>,
    current: &mut Vec<usize>,
    last: usize,
) {
    if remaining == 0 {
        results.push(current.clone());
        return;
    }
    for i in 0..4usize {
        if i == last { continue; } // Skip: G_i G_i = identity
        current.push(i);
        generate_reduced_words(remaining - 1, results, current, i);
        current.pop();
    }
}

// ============================================================
// SONAR DELAY — Ping/return between shadow bands
// ============================================================

/// Sonar result: delay statistics between band-0 and band-5 hits.
#[derive(Clone, Debug)]
pub struct SonarResult {
    pub ping_count: usize,        // band-0 hits
    pub return_count: usize,      // band-5 hits
    pub delays: Vec<usize>,       // step count between consecutive ping→return
    pub min_delay: usize,
    pub max_delay: usize,
    pub mean_delay_num: u64,      // exact numerator
    pub mean_delay_den: u64,      // exact denominator
    pub mode_delay: usize,        // most common delay
}

/// Compute sonar delays for a Syracuse torus orbit on Z/PZ.
pub fn sonar_analysis(
    start_r: i64,
    p_odd: i64,
    shadow_prime: i64,
    max_steps: u64,
) -> SonarResult {
    let syracuse_torus = |r: i64| -> i64 {
        if r <= 0 { return 0; }
        let mut m = r;
        while m % 2 == 0 && m > 0 { m /= 2; }
        if m <= 0 { return 0; }
        let val = 3 * m + 1;
        let mut result = val;
        while result % 2 == 0 { result /= 2; }
        ((result % p_odd) + p_odd) % p_odd
    };

    let mut current = start_r;
    let mut last_ping: Option<u64> = None;
    let mut delays = Vec::new();
    let mut ping_count = 0usize;
    let mut return_count = 0usize;

    let band_0 = 0i64; // membrane: r ≡ 0 mod shadow
    let band_half = shadow_prime / 2; // receive: r ≡ 5 mod 11

    for step in 0..max_steps {
        let band = modd(current, shadow_prime);

        if band == band_0 {
            ping_count += 1;
            last_ping = Some(step);
        }

        if band == band_half {
            return_count += 1;
            if let Some(ping_step) = last_ping {
                let delay = (step - ping_step) as usize;
                if delay > 0 {
                    delays.push(delay);
                }
                last_ping = None; // consume the ping
            }
        }

        if current == 1 { break; }
        current = syracuse_torus(current);
    }

    let min_d = delays.iter().copied().min().unwrap_or(0);
    let max_d = delays.iter().copied().max().unwrap_or(0);
    let sum: u64 = delays.iter().map(|&d| d as u64).sum();
    let count = delays.len() as u64;

    // Mode
    let mut freq: HashMap<usize, usize> = HashMap::new();
    for &d in &delays { *freq.entry(d).or_default() += 1; }
    let mode = freq.iter().max_by_key(|(_, v)| **v).map(|(k, _)| *k).unwrap_or(0);

    SonarResult {
        ping_count, return_count, delays,
        min_delay: min_d, max_delay: max_d,
        mean_delay_num: sum, mean_delay_den: count.max(1),
        mode_delay: mode,
    }
}

// ============================================================
// ORBIT WORD RECOVERY — Using shadow signal as the key
// ============================================================

/// Given a seed, target, and the shadow trace, attempt to recover
/// the word by searching only words consistent with the observed
/// shadow signal at each step.
///
/// This is the "Achilles heel exploit": instead of searching all 4^L
/// words, we only search words whose shadow projection matches.
pub fn shadow_guided_recovery(
    seed: &CurvatureTuple,
    target: &CurvatureTuple,
    shadow_trace: &[[i64; 4]],
    p: i64,
    schema: &CRAMSchema,
) -> Option<Vec<usize>> {
    let max_len = shadow_trace.len().saturating_sub(1);
    if max_len == 0 { return None; }

    // DFS with shadow pruning
    fn dfs(
        current: CurvatureTuple,
        target: &CurvatureTuple,
        shadow_trace: &[[i64; 4]],
        depth: usize,
        max_depth: usize,
        p: i64,
        schema: &CRAMSchema,
        path: &mut Vec<usize>,
        last: usize,
    ) -> bool {
        if depth == max_depth {
            return current == *target;
        }

        for i in 0..4usize {
            if i == last { continue; }
            let next = current.reflect(i, p);
            let next_shadow = schema.read_shadow(&next);

            // PRUNE: if shadow doesn't match, skip this branch
            if depth + 1 < shadow_trace.len() && next_shadow != shadow_trace[depth + 1] {
                continue;
            }

            path.push(i);
            if dfs(next, target, shadow_trace, depth + 1, max_depth, p, schema, path, i) {
                return true;
            }
            path.pop();
        }
        false
    }

    let mut path = Vec::new();
    if dfs(*seed, target, shadow_trace, 0, max_len, p, schema, &mut path, 5) {
        Some(path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descartes_standard_seed() {
        let seed = CurvatureTuple::standard_seed();
        // (-1+2+2+3)² = 36, 2(1+4+4+9) = 36 ✓
        // In Z/pZ for large p, this should still hold
        for p in [101i64, 1009, 15015] {
            let reduced = seed.reduce(p);
            assert!(reduced.check_descartes(p),
                "Descartes fails mod {}", p);
        }
    }

    #[test]
    fn test_reflection_preserves_descartes() {
        let seed = CurvatureTuple::standard_seed();
        let p = 15015i64;
        let reduced = seed.reduce(p);
        for i in 0..4 {
            let reflected = reduced.reflect(i, p);
            assert!(reflected.check_descartes(p),
                "Reflection G_{} breaks Descartes mod {}", i, p);
        }
    }

    #[test]
    fn test_reflection_involution() {
        let seed = CurvatureTuple::standard_seed();
        let p = 15015i64;
        let reduced = seed.reduce(p);
        for i in 0..4 {
            let once = reduced.reflect(i, p);
            let twice = once.reflect(i, p);
            assert_eq!(twice, reduced,
                "G_{} is not an involution mod {}", i, p);
        }
    }

    #[test]
    fn test_non_commutativity() {
        let seed = CurvatureTuple::standard_seed();
        let p = 15015i64;
        let reduced = seed.reduce(p);
        let g0_g1 = reduced.reflect(0, p).reflect(1, p);
        let g1_g0 = reduced.reflect(1, p).reflect(0, p);
        assert_ne!(g0_g1, g1_g0, "G_0 G_1 should not equal G_1 G_0");
    }

    #[test]
    fn test_shadow_signal_varies() {
        let seed = CurvatureTuple::standard_seed();
        let p = 15015i64;
        let reduced = seed.reduce(p);
        let schema = CRAMSchema::shadow_standard();

        let s0 = schema.read_shadow(&reduced);
        let s1 = schema.read_shadow(&reduced.reflect(0, p));
        let s2 = schema.read_shadow(&reduced.reflect(1, p));

        // At least two of the three should differ
        assert!(s0 != s1 || s0 != s2 || s1 != s2,
            "Shadow signal should vary across reflections");
    }

    #[test]
    fn test_achilles_heel_short() {
        let seed = CurvatureTuple::standard_seed().reduce(15015);
        let schema = CRAMSchema::shadow_standard();
        let result = achilles_heel_test(&seed, 15015, 3, &schema);

        println!("Achilles heel (L=3): {} words, {} traces, {} endpoints",
            result.words_tested, result.distinct_shadow_traces, result.distinct_endpoints);
        println!("  Unique: {}, collisions: {}, max_class: {}",
            result.shadow_is_unique, result.shadow_collisions, result.max_collision_class);
        println!("  Info ratio: {:.4}", result.information_ratio);

        // Shadow traces should provide SOME discrimination
        assert!(result.distinct_shadow_traces > 1,
            "Shadow must distinguish at least some words");
    }

    #[test]
    fn test_shadow_guided_recovery() {
        let seed = CurvatureTuple::standard_seed().reduce(15015);
        let schema = CRAMSchema::shadow_standard();
        let p = 15015i64;

        // Apply a known word
        let word = vec![0, 1, 2, 3, 0, 1];
        let target = seed.apply_word(&word, p);
        let trace = schema.trace_orbit_shadow(&seed, &word, p);

        // Recover using shadow guidance
        let recovered = shadow_guided_recovery(&seed, &target, &trace, p, &schema);
        assert!(recovered.is_some(), "Should recover the word");
        let rw = recovered.unwrap();
        // The recovered word should produce the same target
        let check = seed.apply_word(&rw, p);
        assert_eq!(check, target, "Recovered word must reach target");
    }
}
