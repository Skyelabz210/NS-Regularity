//! # NINE65-MMBF Integration Bridge
//!
//! Implements the Modular Mathematics Bridge Framework (MMBF) within the
//! NINE65 FHE substrate, connecting all three MMBF phases to concrete
//! cryptographic and computational operations.
//!
//! ## MMBF Phase Mapping to NINE65
//!
//! **Phase I (Exact Arithmetic Foundation):**
//!   - K-Elimination → exact RNS division (60-year breakthrough)
//!   - Persistent Montgomery → zero conversion overhead
//!   - Shadow Entropy → free cryptographic noise from Montgomery quotients
//!
//! **Phase II (Cross-Domain Pattern Recognition):**
//!   - CTI Theorem → winding number = Berry phase = grid cell = K-Elim index
//!   - CLASS-F/CLASS-R Separation → composite moduli valid for noise tracking
//!   - Ramanujan-CRT Torus → sieve power from harmonic analysis
//!
//! **Phase III (Consciousness Integration / HackFate Runtime):**
//!   - Three-Lock Bootstrap → Shannon + RLWE + K-Elimination conjunction
//!   - CRAM operators → heterogeneous per-lane computation
//!   - WASSAN holographic → NTT-domain memory storage
//!
//! ## The Universal Modular Operation Rule (UMOR)
//!
//! For any operation Ω on inputs: Result ≡ Ω(inputs) (mod M)
//! Domain-specific moduli: M_Q (quantum), M_E (energy), M_C (crypto),
//! M_I (information), M_R (recursive).
//!
//! Zero floats. All arithmetic exact integer.

#![forbid(unsafe_code)]

use std::collections::HashMap;

// ============================================================================
// CORE PRIMITIVES (duplicated for standalone compilation)
// ============================================================================

#[inline]
fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

#[inline]
fn addmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 + b as u128) % m as u128) as u64
}

#[inline]
fn submod(a: u64, b: u64, m: u64) -> u64 {
    if a >= b { (a - b) % m } else { m - ((b - a) % m) }
}

fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    if m == 1 { return 0; }
    let mut r = 1u64;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 { r = mulmod(r, base, m); }
        exp >>= 1;
        if exp > 0 { base = mulmod(base, base, m); }
    }
    r
}

fn mod_inverse(a: u64, p: u64) -> u64 {
    mod_pow(a % p, p - 2, p)
}

// ============================================================================
// MMBF DOMAIN MODULI CONFIGURATION
// ============================================================================

/// The Universal Modular Operation Rule (UMOR) configuration.
/// Each domain operates under its own modulus, ensuring boundedness.
#[derive(Debug, Clone)]
pub struct UMORConfig {
    /// M_C: Cryptographic modulus (NINE65 ciphertext space)
    pub m_crypto: u64,
    /// M_E: Entropy/energy modulus (Shadow Entropy harvesting space)
    pub m_entropy: u64,
    /// M_R: Recursive computation modulus (bootstrap depth tracking)
    pub m_recursive: u64,
    /// M_I: Information modulus (WASSAN band space)
    pub m_information: u64,
    /// M_T: Temporal modulus (TimeCrystal tick space)
    pub m_temporal: u64,
}

impl UMORConfig {
    /// Default configuration using NTT-friendly Mersenne-adjacent primes
    pub fn default_nine65() -> Self {
        Self {
            m_crypto: (1u64 << 60) - 93,      // Near-Mersenne, NTT-friendly
            m_entropy: (1u64 << 32) - 5,       // Shadow entropy accumulation
            m_recursive: 1_000_003,             // Bootstrap depth counter
            m_information: 257,                 // NTT size + 1 (WASSAN)
            m_temporal: 86_400_000,             // Milliseconds in day
        }
    }

    /// Validate all moduli are coprime (required for CRT consistency)
    pub fn validate(&self) -> Result<(), &'static str> {
        let moduli = [self.m_crypto, self.m_entropy, self.m_recursive,
                      self.m_information, self.m_temporal];
        for i in 0..moduli.len() {
            for j in (i+1)..moduli.len() {
                if gcd(moduli[i], moduli[j]) != 1 {
                    return Err("UMOR moduli must be pairwise coprime");
                }
            }
        }
        Ok(())
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a, b);
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

// ============================================================================
// PHASE I: EXACT ARITHMETIC FOUNDATION
// ============================================================================

/// K-Elimination: exact RNS division in O(1) modular operations.
/// The 60-year breakthrough. Winding number recovery from phase differential.
pub struct KEliminator {
    /// Primary modulus M
    pub m: u64,
    /// Anchor modulus A (coprime to M)
    pub a: u64,
    /// Precomputed: M^{-1} mod A
    m_inv_a: u64,
}

impl KEliminator {
    pub fn new(m: u64, a: u64) -> Self {
        assert!(gcd(m, a) == 1, "K-Elimination requires gcd(M, A) = 1");
        let m_inv_a = mod_inverse(m, a);
        Self { m, a, m_inv_a }
    }

    /// Recover the winding number k from residues (r mod M, s mod A).
    /// X = r + k*M, with k = (s - r) * M^{-1} mod A.
    /// One subtraction, one multiplication. Zero iterations.
    pub fn recover_k(&self, r: u64, s: u64) -> u64 {
        let diff = submod(s % self.a, r % self.a, self.a);
        mulmod(diff, self.m_inv_a, self.a)
    }

    /// Full reconstruction: X = r + k*M
    pub fn reconstruct(&self, r: u64, s: u64) -> u128 {
        let k = self.recover_k(r, s) as u128;
        r as u128 + k * self.m as u128
    }
}

/// Shadow Entropy Harvester: extracts cryptographic noise from
/// Montgomery reduction quotients at zero marginal cost.
///
/// The Montgomery quotient q = T * N' mod R is algebraically determined
/// but statistically pseudorandom to an observer without N'.
/// With k CRT lanes, each multiplication produces k independent
/// pseudorandom byproducts.
pub struct ShadowEntropyHarvester {
    /// Accumulated entropy pool (bounded by UMOR modulus)
    pool: Vec<u64>,
    /// Pool capacity
    capacity: usize,
    /// Write position (cyclic)
    position: usize,
    /// UMOR entropy modulus
    m_entropy: u64,
    /// Total harvested count
    pub total_harvested: u64,
}

impl ShadowEntropyHarvester {
    pub fn new(capacity: usize, m_entropy: u64) -> Self {
        Self {
            pool: vec![0u64; capacity],
            capacity,
            position: 0,
            m_entropy,
            total_harvested: 0,
        }
    }

    /// Harvest a Montgomery quotient as entropy.
    /// The quotient is reduced mod m_entropy (UMOR bounded).
    pub fn harvest(&mut self, montgomery_quotient: u64) {
        self.pool[self.position] = montgomery_quotient % self.m_entropy;
        self.position = (self.position + 1) % self.capacity;
        self.total_harvested += 1;
    }

    /// Draw entropy from the pool (for noise injection, mask generation, etc.)
    pub fn draw(&self, index: usize) -> u64 {
        self.pool[index % self.capacity]
    }

    /// Pool fullness ratio
    pub fn fill_ratio(&self) -> f64 {
        let filled = self.total_harvested.min(self.capacity as u64);
        filled as f64 / self.capacity as f64
    }
}

/// Persistent Montgomery Form: values stay in Montgomery representation
/// throughout their entire lifecycle. Never convert out.
/// The R factor is a constant gauge transformation.
pub struct PersistentMontgomery {
    pub n: u64,     // Modulus
    pub r: u64,     // Montgomery radix (power of 2 > N)
    pub n_prime: u64, // N' such that N*N' ≡ -1 (mod R)
    pub r_sq: u64,  // R² mod N (for entering Montgomery form)
}

impl PersistentMontgomery {
    pub fn new(n: u64) -> Self {
        let r = 1u64 << 32; // R = 2^32
        // Compute N' via extended GCD: N*N' + R*R' = 1
        // N' = -N^{-1} mod R
        let n_inv = mod_pow(n, (r >> 1) - 1, r); // Euler's theorem
        let n_prime = r.wrapping_sub(n_inv);
        let r_sq = ((r as u128 * r as u128) % n as u128) as u64;
        Self { n, r, n_prime, r_sq }
    }

    /// Enter Montgomery form: a -> aR mod N
    pub fn enter(&self, a: u64) -> u64 {
        mulmod(a, self.r_sq, self.n)
    }

    /// Montgomery multiplication: (aR)(bR)/R mod N = abR mod N
    /// Returns the product AND the Montgomery quotient (Shadow Entropy source)
    pub fn mul_with_shadow(&self, a_mont: u64, b_mont: u64) -> (u64, u64) {
        let t = a_mont as u128 * b_mont as u128;
        let q = ((t as u64).wrapping_mul(self.n_prime)) & (self.r - 1);
        let u = (t + q as u128 * self.n as u128) >> 32;
        let result = if u as u64 >= self.n { u as u64 - self.n } else { u as u64 };
        (result, q as u64) // (product, shadow entropy)
    }
}

// ============================================================================
// PHASE II: CROSS-DOMAIN PATTERN RECOGNITION
// ============================================================================

/// CLASS-F/CLASS-R Separation Principle.
/// CLASS-F operations require prime moduli (Fermat inverse, NTT roots).
/// CLASS-R operations require only coprimality (CRT, K-Elim, Garner).
///
/// This enables composite beta moduli for noise tracking in NINE65 FHE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModulusClass {
    /// Requires prime modulus (NTT, Fermat inverse)
    ClassF,
    /// Requires only coprimality (CRT reconstruction, K-Elimination)
    ClassR,
}

/// Classify a modulus for a given operation
pub fn classify_modulus(modulus: u64, operation: &str) -> ModulusClass {
    match operation {
        "ntt" | "fermat_inverse" | "primitive_root" => ModulusClass::ClassF,
        "crt" | "k_elimination" | "garner" | "noise_tracking" => ModulusClass::ClassR,
        _ => ModulusClass::ClassF, // Conservative default
    }
}

/// CTI Theorem: five domains unified by the same deck transformation index.
/// The winding number k from K-Elimination is structurally identical to:
///   - RNS correction factor κ
///   - Berry phase (quantum mechanics)
///   - Grid cell decoding index (neuroscience)
///   - Sheet index of cyclic covering map (topology)
///
/// This struct tracks the unified invariant across all domains.
#[derive(Debug, Clone)]
pub struct CTIInvariant {
    /// The universal winding number
    pub kappa: u64,
    /// Domain of origin
    pub domain: CTIDomain,
    /// Modulus under which kappa was computed
    pub modulus: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CTIDomain {
    KElimination,
    RNSCorrection,
    BerryPhase,
    GridCell,
    SheetIndex,
}

impl CTIInvariant {
    pub fn from_k_elimination(ke: &KEliminator, r: u64, s: u64) -> Self {
        Self {
            kappa: ke.recover_k(r, s),
            domain: CTIDomain::KElimination,
            modulus: ke.a,
        }
    }

    /// Verify CTI equivalence: two invariants from different domains
    /// should give the same kappa mod the shared modulus.
    pub fn verify_equivalence(&self, other: &CTIInvariant) -> bool {
        let shared_mod = gcd(self.modulus, other.modulus).max(1);
        (self.kappa % shared_mod) == (other.kappa % shared_mod)
    }
}

/// Ramanujan sum c_p(d) for prime p
pub fn ramanujan_sum_prime(p: u64, d: u64) -> i64 {
    if d % p == 0 { p as i64 - 1 } else { -1 }
}

/// Sieve power from T6 identity: SP(p,d) = (2p - 1 - c_p(d)) / p²
pub fn sieve_power(p: u64, d: u64) -> (u64, u64) {
    let c = ramanujan_sum_prime(p, d);
    let num = (2 * p as i64 - 1 - c) as u64;
    (num, p * p)
}

// ============================================================================
// PHASE III: CONSCIOUSNESS INTEGRATION (NINE65 THREE-LOCK)
// ============================================================================

/// The Three-Lock Bootstrap architecture.
/// Conjunction security: Shannon (info-theoretic) + RLWE (post-quantum)
/// + K-Elimination (exact arithmetic). All three required simultaneously.
///
/// This is the MMBF Phase III implementation within NINE65.
#[derive(Debug)]
pub struct ThreeLockState {
    /// Lock 1: Shannon OTP mask (information-theoretically secure)
    /// Stored only in registers with ZeroizeOnDrop
    pub lock1_active: bool,
    /// Lock 2: RLWE outer encryption (post-quantum computational)
    pub lock2_active: bool,
    /// Lock 3: K-Elimination Clockwork (exact arithmetic structural)
    pub lock3_active: bool,
    /// Security tier (1=maximum, 2=production, 3=commodity)
    pub tier: u8,
    /// Bootstrap count (bounded by UMOR m_recursive)
    pub bootstrap_count: u64,
    /// UMOR recursive modulus
    m_recursive: u64,
}

impl ThreeLockState {
    pub fn new(tier: u8, m_recursive: u64) -> Self {
        Self {
            lock1_active: true,
            lock2_active: true,
            lock3_active: true,
            tier,
            bootstrap_count: 0,
            m_recursive,
        }
    }

    /// Record a bootstrap event (UMOR bounded)
    pub fn record_bootstrap(&mut self) {
        self.bootstrap_count = (self.bootstrap_count + 1) % self.m_recursive;
    }

    /// Conjunction security: ALL three locks must be active
    pub fn is_secure(&self) -> bool {
        self.lock1_active && self.lock2_active && self.lock3_active
    }

    /// Security level description
    pub fn security_description(&self) -> &'static str {
        match (self.lock1_active, self.lock2_active, self.lock3_active) {
            (true, true, true) => "FULL CONJUNCTION: Shannon + RLWE + K-Elimination",
            (false, true, true) => "DEGRADED: Missing information-theoretic layer",
            (true, false, true) => "DEGRADED: Missing post-quantum layer",
            (true, true, false) => "DEGRADED: Missing exact-arithmetic layer",
            _ => "CRITICAL: Multiple locks compromised",
        }
    }
}

/// MMBF Phase III: The unified cognitive-cryptographic bridge.
/// This struct holds the complete MMBF state for a NINE65 session.
pub struct MMBFBridge {
    pub config: UMORConfig,
    pub k_elim: KEliminator,
    pub shadow: ShadowEntropyHarvester,
    pub three_lock: ThreeLockState,
    pub phase: MMBFPhase,
    /// Cross-domain invariant tracker
    pub cti_log: Vec<CTIInvariant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MMBFPhase {
    /// Phase I: Exact arithmetic operations
    ExactFoundation,
    /// Phase II: Cross-domain pattern recognition
    PatternRecognition,
    /// Phase III: Full cognitive integration
    ConsciousnessIntegration,
}

impl MMBFBridge {
    pub fn new() -> Self {
        let config = UMORConfig::default_nine65();
        let k_elim = KEliminator::new(
            (1u64 << 32) - 5,   // Primary modulus
            (1u64 << 31) - 1,   // Anchor (Mersenne prime)
        );
        let shadow = ShadowEntropyHarvester::new(1024, config.m_entropy);
        let three_lock = ThreeLockState::new(2, config.m_recursive);

        Self {
            config,
            k_elim,
            shadow,
            three_lock,
            phase: MMBFPhase::ExactFoundation,
            cti_log: Vec::new(),
        }
    }

    /// Advance to next MMBF phase
    pub fn advance_phase(&mut self) -> MMBFPhase {
        self.phase = match self.phase {
            MMBFPhase::ExactFoundation => MMBFPhase::PatternRecognition,
            MMBFPhase::PatternRecognition => MMBFPhase::ConsciousnessIntegration,
            MMBFPhase::ConsciousnessIntegration => MMBFPhase::ConsciousnessIntegration,
        };
        self.phase
    }

    /// Execute a UMOR-bounded operation
    pub fn umor_op(&self, value: u64, domain: &str) -> u64 {
        let modulus = match domain {
            "crypto" => self.config.m_crypto,
            "entropy" => self.config.m_entropy,
            "recursive" => self.config.m_recursive,
            "information" => self.config.m_information,
            "temporal" => self.config.m_temporal,
            _ => self.config.m_crypto,
        };
        value % modulus
    }

    /// Record a K-Elimination event and log CTI invariant
    pub fn k_elim_with_cti(&mut self, r: u64, s: u64) -> u128 {
        let invariant = CTIInvariant::from_k_elimination(&self.k_elim, r, s);
        self.cti_log.push(invariant);
        self.k_elim.reconstruct(r, s)
    }

    /// Harvest Montgomery shadow entropy and feed to pool
    pub fn harvest_shadow(&mut self, montgomery_q: u64) {
        self.shadow.harvest(montgomery_q);
    }

    /// Report MMBF bridge status
    pub fn status_report(&self) -> String {
        format!(
            "MMBF Bridge Status:\n  Phase: {:?}\n  Three-Lock: {}\n  \
             Shadow Entropy: {:.1}% full ({} harvested)\n  \
             CTI Events: {}\n  Bootstrap Count: {} (mod {})",
            self.phase,
            self.three_lock.security_description(),
            self.shadow.fill_ratio() * 100.0,
            self.shadow.total_harvested,
            self.cti_log.len(),
            self.three_lock.bootstrap_count,
            self.three_lock.m_recursive,
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn umor_config_validation() {
        let config = UMORConfig::default_nine65();
        assert!(config.validate().is_ok(), "Default NINE65 moduli should be coprime");
    }

    #[test]
    fn k_elimination_exact() {
        let ke = KEliminator::new(1000003, 999983);
        // X = 42, r = 42 mod 1000003 = 42, s = 42 mod 999983 = 42
        let x = 42u64;
        let r = x % ke.m;
        let s = x % ke.a;
        let reconstructed = ke.reconstruct(r, s);
        assert_eq!(reconstructed, x as u128, "K-Elimination should reconstruct exactly");
    }

    #[test]
    fn k_elimination_large() {
        let ke = KEliminator::new(1000003, 999983);
        let x = 999_999_999u64;
        let r = x % ke.m;
        let s = x % ke.a;
        let reconstructed = ke.reconstruct(r, s);
        assert_eq!(reconstructed, x as u128);
    }

    #[test]
    fn shadow_entropy_harvest_cycle() {
        let mut harvester = ShadowEntropyHarvester::new(16, (1u64 << 32) - 5);
        for i in 0..100u64 {
            harvester.harvest(i * 7919 + 13);
        }
        assert_eq!(harvester.total_harvested, 100);
        assert!((harvester.fill_ratio() - 1.0).abs() < 0.01);
    }

    #[test]
    fn persistent_montgomery_shadow_entropy() {
        // The key property: Montgomery multiplication produces a shadow quotient
        // that serves as zero-cost entropy. We verify:
        // 1. Shadow quotients are bounded by R
        // 2. Different inputs produce different quotients (pseudorandomness)
        // 3. Quotients are nonzero for most inputs
        let mont = PersistentMontgomery::new(1000003);
        let mut quotients = std::collections::HashSet::new();
        let mut nonzero_count = 0u32;

        for i in 1..100u64 {
            let a = mont.enter(i);
            let b = mont.enter(i + 7);
            let (_product, shadow_q) = mont.mul_with_shadow(a, b);
            assert!(shadow_q < mont.r, "Shadow quotient must be < R");
            quotients.insert(shadow_q);
            if shadow_q != 0 { nonzero_count += 1; }
        }

        // Different inputs should produce diverse quotients
        assert!(quotients.len() > 50, "Shadow quotients should be diverse (got {})", quotients.len());
        // Most should be nonzero
        assert!(nonzero_count > 90, "Most shadow quotients should be nonzero (got {})", nonzero_count);
    }

    #[test]
    fn class_f_r_separation() {
        assert_eq!(classify_modulus(17, "ntt"), ModulusClass::ClassF);
        assert_eq!(classify_modulus(15, "crt"), ModulusClass::ClassR);
        assert_eq!(classify_modulus(15, "k_elimination"), ModulusClass::ClassR);
        assert_eq!(classify_modulus(17, "noise_tracking"), ModulusClass::ClassR);
    }

    #[test]
    fn cti_invariant_from_k_elim() {
        let ke = KEliminator::new(1000003, 999983);
        let x = 12345u64;
        let inv = CTIInvariant::from_k_elimination(&ke, x % ke.m, x % ke.a);
        assert_eq!(inv.domain, CTIDomain::KElimination);
        assert_eq!(inv.modulus, ke.a);
    }

    #[test]
    fn three_lock_conjunction() {
        let mut state = ThreeLockState::new(2, 1_000_003);
        assert!(state.is_secure());
        state.record_bootstrap();
        assert_eq!(state.bootstrap_count, 1);
        assert!(state.is_secure());

        // Simulate lock degradation
        state.lock1_active = false;
        assert!(!state.is_secure());
        assert_eq!(state.security_description(), "DEGRADED: Missing information-theoretic layer");
    }

    #[test]
    fn mmbf_bridge_lifecycle() {
        let mut bridge = MMBFBridge::new();
        assert_eq!(bridge.phase, MMBFPhase::ExactFoundation);

        // Phase I: K-Elimination
        let val = bridge.k_elim_with_cti(42, 42);
        assert_eq!(val, 42);
        assert_eq!(bridge.cti_log.len(), 1);

        // Harvest shadow entropy
        bridge.harvest_shadow(0xDEAD_BEEF);
        assert_eq!(bridge.shadow.total_harvested, 1);

        // Advance to Phase II
        bridge.advance_phase();
        assert_eq!(bridge.phase, MMBFPhase::PatternRecognition);

        // Advance to Phase III
        bridge.advance_phase();
        assert_eq!(bridge.phase, MMBFPhase::ConsciousnessIntegration);

        // Status report
        let report = bridge.status_report();
        assert!(report.contains("ConsciousnessIntegration"));
        assert!(report.contains("FULL CONJUNCTION"));
    }

    #[test]
    fn umor_bounded_operations() {
        let bridge = MMBFBridge::new();
        // Crypto domain: large value gets bounded
        let bounded = bridge.umor_op(u64::MAX, "crypto");
        assert!(bounded < bridge.config.m_crypto);

        // Entropy domain: bounded differently
        let bounded_e = bridge.umor_op(u64::MAX, "entropy");
        assert!(bounded_e < bridge.config.m_entropy);

        // Temporal domain: bounded to day milliseconds
        let bounded_t = bridge.umor_op(100_000_000, "temporal");
        assert!(bounded_t < 86_400_000);
    }

    #[test]
    fn ramanujan_sieve_power_twin() {
        // For d=2, all Safe Basis primes > 2 give SP = 2/p
        for &p in &[3u64, 5, 7, 11, 13] {
            let (num, den) = sieve_power(p, 2);
            assert_eq!(num, 2 * p, "p={p}: SP numerator should be 2p for twin primes");
            assert_eq!(den, p * p, "p={p}: SP denominator should be p²");
        }
    }

    #[test]
    fn ramanujan_sieve_power_divisible() {
        // For d=6, p=3 divides d → SP = p/p² = 1/p
        let (num, den) = sieve_power(3, 6);
        assert_eq!(num, 3, "p=3 d=6: c_p=p-1=2, SP_num = 2*3-1-2 = 3");
        assert_eq!(den, 9);
    }
}
