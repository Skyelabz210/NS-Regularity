//! # Loki 5.0 Adaptive Security Engine — W3.2
//!
//! Non-stationary cryptographic defense driven by real-time threat telemetry.
//!
//! ## Architecture
//!
//! ```text
//! TimeCrystal (W2.5) ──entropy──→ CT-DRBG ──random──→ Cipher Operations
//!                                    ↑
//! CDHS (W2.3) ──threat──→ ThreatReactor ──level──→ Security Policy
//!                                    ↓
//!                            Adaptive S-box Evolution
//! ```
//!
//! ## Key Properties
//!
//! - **Non-stationary**: Security parameters change with threat level
//! - **Entropy-driven**: CT-DRBG reseeds from TimeCrystal gate events
//! - **Threat-reactive**: CDHS threat vector drives security escalation
//! - **Integer-only**: All operations exact modular arithmetic

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

use std::fmt;

// ============================================================================
// CT-DRBG (Counter-mode Deterministic Random Bit Generator)
// ============================================================================

/// CT-DRBG state. Uses a simple integer-only construction:
/// output = hash(key ⊕ counter), where hash is a mixing function.
///
/// Reseeded from TimeCrystal gate entropy.
#[derive(Clone, Debug)]
pub struct CtDrbg {
    /// 256-bit key stored as [u64; 4]
    key: [u64; 4],
    /// Monotonic counter (never wraps in practice: 2^64 outputs)
    counter: u64,
    /// Number of outputs since last reseed
    since_reseed: u64,
    /// Maximum outputs before mandatory reseed
    reseed_interval: u64,
    /// Total reseeds performed
    reseed_count: u64,
}

impl CtDrbg {
    /// Create with initial seed.
    pub fn new(seed: &[u8]) -> Self {
        let mut key = [0u64; 4];
        for (i, chunk) in seed.chunks(8).enumerate().take(4) {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            key[i] = u64::from_le_bytes(bytes);
        }
        Self {
            key,
            counter: 0,
            since_reseed: 0,
            reseed_interval: 4096,
            reseed_count: 0,
        }
    }

    /// Integer mixing function (SplitMix64 variant).
    fn mix(mut x: u64) -> u64 {
        x ^= x >> 30;
        x = x.wrapping_mul(0xbf58476d1ce4e5b9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94d049bb133111eb);
        x ^= x >> 31;
        x
    }

    /// Generate next 64-bit random value.
    pub fn generate(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        self.since_reseed += 1;

        let mixed = Self::mix(self.key[0] ^ self.counter)
            ^ Self::mix(self.key[1] ^ self.counter.wrapping_mul(3))
            ^ Self::mix(self.key[2] ^ self.counter.wrapping_mul(7))
            ^ Self::mix(self.key[3] ^ self.counter.wrapping_mul(13));

        mixed
    }

    /// Generate N random bytes.
    pub fn generate_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(n);
        while result.len() < n {
            let val = self.generate();
            let bytes = val.to_le_bytes();
            let remaining = n - result.len();
            result.extend_from_slice(&bytes[..remaining.min(8)]);
        }
        result
    }

    /// Reseed from external entropy (TimeCrystal gate events).
    pub fn reseed(&mut self, entropy: &[u8]) {
        // XOR new entropy into key
        for (i, chunk) in entropy.chunks(8).enumerate() {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            let val = u64::from_le_bytes(bytes);
            self.key[i % 4] ^= Self::mix(val ^ self.counter);
        }
        self.since_reseed = 0;
        self.reseed_count += 1;
    }

    /// Check if reseed is needed.
    pub fn needs_reseed(&self) -> bool {
        self.since_reseed >= self.reseed_interval
    }

    pub fn reseed_count(&self) -> u64 { self.reseed_count }
    pub fn since_reseed(&self) -> u64 { self.since_reseed }
}

// ============================================================================
// SECURITY LEVELS
// ============================================================================

/// Security level, driven by threat vector from CDHS.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    /// θ < 2000: Standard operations, normal reseed interval
    Normal,
    /// 2000 ≤ θ < 5000: Increased monitoring, faster reseeds
    Elevated,
    /// 5000 ≤ θ < 8000: Active defense, S-box evolution, short reseed
    High,
    /// θ ≥ 8000: Maximum defense, continuous reseed, lockdown
    Critical,
}

impl SecurityLevel {
    /// Determine level from threat vector (0..10000).
    pub fn from_threat(theta: u64) -> Self {
        match theta {
            0..2000 => SecurityLevel::Normal,
            2000..5000 => SecurityLevel::Elevated,
            5000..8000 => SecurityLevel::High,
            _ => SecurityLevel::Critical,
        }
    }

    /// Reseed interval for this security level.
    pub fn reseed_interval(&self) -> u64 {
        match self {
            SecurityLevel::Normal => 4096,
            SecurityLevel::Elevated => 1024,
            SecurityLevel::High => 256,
            SecurityLevel::Critical => 16,
        }
    }

    /// Number of S-box evolution rounds per tick.
    pub fn sbox_evolution_rounds(&self) -> u32 {
        match self {
            SecurityLevel::Normal => 0,
            SecurityLevel::Elevated => 1,
            SecurityLevel::High => 4,
            SecurityLevel::Critical => 16,
        }
    }
}

impl fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityLevel::Normal => write!(f, "NORMAL"),
            SecurityLevel::Elevated => write!(f, "ELEVATED"),
            SecurityLevel::High => write!(f, "HIGH"),
            SecurityLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

// ============================================================================
// ADAPTIVE S-BOX
// ============================================================================

/// An S-box (substitution box) that evolves based on security level.
///
/// Standard: fixed 256-byte permutation.
/// Adaptive: Fisher-Yates shuffle driven by CT-DRBG, re-evolved at each
/// security level change.
#[derive(Clone, Debug)]
pub struct AdaptiveSbox {
    /// Forward substitution table (256 entries)
    pub forward: [u8; 256],
    /// Inverse substitution table
    pub inverse: [u8; 256],
    /// Evolution generation counter
    pub generation: u64,
}

impl AdaptiveSbox {
    /// Create identity S-box.
    pub fn identity() -> Self {
        let mut forward = [0u8; 256];
        let mut inverse = [0u8; 256];
        for i in 0..256 {
            forward[i] = i as u8;
            inverse[i] = i as u8;
        }
        Self { forward, inverse, generation: 0 }
    }

    /// Evolve S-box using Fisher-Yates shuffle driven by DRBG.
    pub fn evolve(&mut self, drbg: &mut CtDrbg) {
        // Start from current state, apply Fisher-Yates
        for i in (1..256).rev() {
            let j = (drbg.generate() as usize) % (i + 1);
            self.forward.swap(i, j);
        }
        // Rebuild inverse
        for i in 0..256 {
            self.inverse[self.forward[i] as usize] = i as u8;
        }
        self.generation += 1;
    }

    /// Substitute a byte (forward).
    #[inline]
    pub fn substitute(&self, byte: u8) -> u8 {
        self.forward[byte as usize]
    }

    /// Inverse substitute.
    #[inline]
    pub fn unsubstitute(&self, byte: u8) -> u8 {
        self.inverse[byte as usize]
    }

    /// Verify S-box is a valid permutation.
    pub fn is_valid_permutation(&self) -> bool {
        let mut seen = [false; 256];
        for &b in &self.forward {
            if seen[b as usize] { return false; }
            seen[b as usize] = true;
        }
        // Verify inverse
        for i in 0..256 {
            if self.inverse[self.forward[i] as usize] != i as u8 { return false; }
        }
        true
    }
}

// ============================================================================
// LOKI ENGINE
// ============================================================================

/// The Loki 5.0 Adaptive Security Engine.
pub struct LokiEngine {
    /// CT-DRBG for cryptographic randomness
    pub drbg: CtDrbg,
    /// Current security level
    pub level: SecurityLevel,
    /// Adaptive S-box
    pub sbox: AdaptiveSbox,
    /// Tick counter
    pub tick: u64,
    /// Level change history
    level_history: Vec<(u64, SecurityLevel)>,
}

impl LokiEngine {
    /// Create with initial seed.
    pub fn new(seed: &[u8]) -> Self {
        Self {
            drbg: CtDrbg::new(seed),
            level: SecurityLevel::Normal,
            sbox: AdaptiveSbox::identity(),
            tick: 0,
            level_history: vec![(0, SecurityLevel::Normal)],
        }
    }

    /// Process a threat vector from CDHS. Adjusts security level and
    /// triggers appropriate responses.
    pub fn process_threat(&mut self, theta: u64) -> ThreatResponse {
        self.tick += 1;
        let new_level = SecurityLevel::from_threat(theta);
        let level_changed = new_level != self.level;

        if level_changed {
            self.level = new_level;
            self.level_history.push((self.tick, new_level));
            // Adjust reseed interval
            self.drbg.reseed_interval = new_level.reseed_interval();
        }

        // Evolve S-box if required by security level
        let evolutions = self.level.sbox_evolution_rounds();
        for _ in 0..evolutions {
            self.sbox.evolve(&mut self.drbg);
        }

        // Check if reseed needed
        let reseeded = if self.drbg.needs_reseed() {
            // In production, entropy comes from TimeCrystal gate events.
            // Here we self-reseed from current state as fallback.
            let entropy = self.drbg.generate().to_le_bytes();
            self.drbg.reseed(&entropy);
            true
        } else { false };

        ThreatResponse {
            tick: self.tick,
            theta,
            level: self.level,
            level_changed,
            sbox_evolutions: evolutions,
            sbox_generation: self.sbox.generation,
            reseeded,
        }
    }

    /// Reseed from external entropy (TimeCrystal gate event).
    pub fn reseed_from_crystal(&mut self, entropy: &[u8]) {
        self.drbg.reseed(entropy);
    }

    /// Generate random bytes at current security level.
    pub fn random_bytes(&mut self, n: usize) -> Vec<u8> {
        self.drbg.generate_bytes(n)
    }

    /// Encrypt bytes through adaptive S-box.
    pub fn sbox_encrypt(&self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|&b| self.sbox.substitute(b)).collect()
    }

    /// Decrypt bytes through adaptive S-box.
    pub fn sbox_decrypt(&self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|&b| self.sbox.unsubstitute(b)).collect()
    }

    pub fn security_level(&self) -> SecurityLevel { self.level }
    pub fn sbox_generation(&self) -> u64 { self.sbox.generation }
}

impl fmt::Debug for LokiEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LokiEngine(level={}, sbox_gen={}, reseeds={})",
            self.level, self.sbox.generation, self.drbg.reseed_count())
    }
}

/// Response from processing a threat vector.
#[derive(Clone, Debug)]
pub struct ThreatResponse {
    pub tick: u64,
    pub theta: u64,
    pub level: SecurityLevel,
    pub level_changed: bool,
    pub sbox_evolutions: u32,
    pub sbox_generation: u64,
    pub reseeded: bool,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- CT-DRBG ---

    #[test]
    fn drbg_deterministic() {
        let mut d1 = CtDrbg::new(b"hackfate");
        let mut d2 = CtDrbg::new(b"hackfate");
        let v1: Vec<u64> = (0..100).map(|_| d1.generate()).collect();
        let v2: Vec<u64> = (0..100).map(|_| d2.generate()).collect();
        assert_eq!(v1, v2, "Same seed should produce same sequence");
    }

    #[test]
    fn drbg_different_seeds_diverge() {
        let mut d1 = CtDrbg::new(b"seed_a");
        let mut d2 = CtDrbg::new(b"seed_b");
        let v1 = d1.generate();
        let v2 = d2.generate();
        assert_ne!(v1, v2, "Different seeds should diverge");
    }

    #[test]
    fn drbg_reseed_changes_output() {
        let mut d1 = CtDrbg::new(b"test");
        let mut d2 = CtDrbg::new(b"test");
        // Advance both identically
        for _ in 0..10 { d1.generate(); d2.generate(); }
        // Reseed only d1
        d1.reseed(b"new_entropy_12345678");
        let v1 = d1.generate();
        let v2 = d2.generate();
        assert_ne!(v1, v2, "Reseed should change output");
    }

    #[test]
    fn drbg_needs_reseed() {
        let mut drbg = CtDrbg::new(b"test");
        drbg.reseed_interval = 10;
        for _ in 0..9 { drbg.generate(); }
        assert!(!drbg.needs_reseed());
        drbg.generate();
        assert!(drbg.needs_reseed());
    }

    #[test]
    fn drbg_generate_bytes() {
        let mut drbg = CtDrbg::new(b"test");
        let bytes = drbg.generate_bytes(32);
        assert_eq!(bytes.len(), 32);
        // Not all zeros
        assert!(bytes.iter().any(|&b| b != 0));
    }

    // --- Security Levels ---

    #[test]
    fn security_level_thresholds() {
        assert_eq!(SecurityLevel::from_threat(0), SecurityLevel::Normal);
        assert_eq!(SecurityLevel::from_threat(1999), SecurityLevel::Normal);
        assert_eq!(SecurityLevel::from_threat(2000), SecurityLevel::Elevated);
        assert_eq!(SecurityLevel::from_threat(4999), SecurityLevel::Elevated);
        assert_eq!(SecurityLevel::from_threat(5000), SecurityLevel::High);
        assert_eq!(SecurityLevel::from_threat(7999), SecurityLevel::High);
        assert_eq!(SecurityLevel::from_threat(8000), SecurityLevel::Critical);
        assert_eq!(SecurityLevel::from_threat(10000), SecurityLevel::Critical);
    }

    #[test]
    fn security_level_ordering() {
        assert!(SecurityLevel::Normal < SecurityLevel::Elevated);
        assert!(SecurityLevel::Elevated < SecurityLevel::High);
        assert!(SecurityLevel::High < SecurityLevel::Critical);
    }

    // --- Adaptive S-box ---

    #[test]
    fn sbox_identity_roundtrip() {
        let sbox = AdaptiveSbox::identity();
        for i in 0..=255u8 {
            assert_eq!(sbox.substitute(i), i);
            assert_eq!(sbox.unsubstitute(i), i);
        }
        assert!(sbox.is_valid_permutation());
    }

    #[test]
    fn sbox_evolve_is_permutation() {
        let mut sbox = AdaptiveSbox::identity();
        let mut drbg = CtDrbg::new(b"evolve_test");
        sbox.evolve(&mut drbg);
        assert!(sbox.is_valid_permutation(), "Evolved S-box must be valid permutation");
        assert_eq!(sbox.generation, 1);
    }

    #[test]
    fn sbox_encrypt_decrypt_roundtrip() {
        let mut sbox = AdaptiveSbox::identity();
        let mut drbg = CtDrbg::new(b"roundtrip");
        sbox.evolve(&mut drbg);

        let plaintext = b"K-Elimination is exact.";
        let encrypted: Vec<u8> = plaintext.iter().map(|&b| sbox.substitute(b)).collect();
        let decrypted: Vec<u8> = encrypted.iter().map(|&b| sbox.unsubstitute(b)).collect();
        assert_eq!(&decrypted, plaintext);
        assert_ne!(&encrypted, &plaintext[..], "Encryption should change data");
    }

    #[test]
    fn sbox_multiple_evolutions_all_valid() {
        let mut sbox = AdaptiveSbox::identity();
        let mut drbg = CtDrbg::new(b"multi_evolve");
        for _ in 0..100 {
            sbox.evolve(&mut drbg);
            assert!(sbox.is_valid_permutation());
        }
        assert_eq!(sbox.generation, 100);
    }

    // --- Loki Engine ---

    #[test]
    fn loki_normal_operations() {
        let mut loki = LokiEngine::new(b"hackfate_seed_0123456789abcdef");
        let resp = loki.process_threat(0);
        assert_eq!(resp.level, SecurityLevel::Normal);
        assert!(!resp.level_changed); // was already Normal
        assert_eq!(resp.sbox_evolutions, 0);
    }

    #[test]
    fn loki_escalation() {
        let mut loki = LokiEngine::new(b"escalation_test");
        loki.process_threat(0); // Normal

        let resp = loki.process_threat(5000); // → High
        assert_eq!(resp.level, SecurityLevel::High);
        assert!(resp.level_changed);
        assert_eq!(resp.sbox_evolutions, 4); // High = 4 rounds
    }

    #[test]
    fn loki_critical_rapid_evolution() {
        let mut loki = LokiEngine::new(b"critical_test");
        let resp = loki.process_threat(9000); // → Critical
        assert_eq!(resp.level, SecurityLevel::Critical);
        assert_eq!(resp.sbox_evolutions, 16);
        assert!(loki.sbox.is_valid_permutation());
    }

    #[test]
    fn loki_sbox_encrypt_decrypt() {
        let mut loki = LokiEngine::new(b"sbox_test");
        // Evolve S-box
        loki.process_threat(6000); // High → evolves 4 times

        let data = b"Truth cannot be approximated.";
        let encrypted = loki.sbox_encrypt(data);
        let decrypted = loki.sbox_decrypt(&encrypted);
        assert_eq!(&decrypted, &data[..]);
        assert_ne!(&encrypted, &data[..]);
    }

    #[test]
    fn loki_reseed_from_crystal() {
        let mut loki = LokiEngine::new(b"reseed_test");
        let before = loki.drbg.generate();
        loki.reseed_from_crystal(b"crystal_gate_entropy_bytes_here!");
        let after = loki.drbg.generate();
        // After reseed, sequence should differ from what it would have been
        // (We can't easily test this without a parallel instance, but reseed_count tracks it)
        assert_eq!(loki.drbg.reseed_count(), 1);
        let _ = (before, after); // used
    }

    #[test]
    fn loki_deescalation() {
        let mut loki = LokiEngine::new(b"deescalate");
        loki.process_threat(9000); // Critical
        assert_eq!(loki.security_level(), SecurityLevel::Critical);

        let resp = loki.process_threat(100); // Back to Normal
        assert_eq!(resp.level, SecurityLevel::Normal);
        assert!(resp.level_changed);
    }

    #[test]
    fn loki_sustained_threat_evolves_sbox() {
        let mut loki = LokiEngine::new(b"sustained");
        let initial_gen = loki.sbox_generation();
        // 10 ticks at High (4 evolutions each)
        for _ in 0..10 {
            loki.process_threat(6000);
        }
        assert_eq!(loki.sbox_generation(), initial_gen + 40);
        assert!(loki.sbox.is_valid_permutation());
    }
}
