//! # Digital Time Crystal — W2.5
//!
//! A phi-resonant integer oscillator with Fibonacci aperiodic gating.
//!
//! ## Purpose
//!
//! Provides the temporal heartbeat for HackFate:
//! - **Loki 5.0**: Drives CT-DRBG reseeding schedule (unpredictable to adversaries)
//! - **CDHS**: Phase readout for health monitoring
//! - **ASVE**: Cognitive tick clock
//!
//! ## Design Principles
//!
//! 1. **Phi-resonant**: Phase advances by Fibonacci ratios, maximally aperiodic
//! 2. **Integer-only**: All state is modular integer arithmetic
//! 3. **Aperiodic gating**: Gate events at Fibonacci-indexed ticks prevent
//!    adversarial phase-lock
//! 4. **Deterministic**: Same seed → same sequence (testable)
//! 5. **Entropy-rich**: Phase state at gate events provides high-quality
//!    deterministic entropy for CT-DRBG

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

// ============================================================================
// FIBONACCI CONSTANTS (precomputed, exact)
// ============================================================================

/// First 93 Fibonacci numbers (all fit in u64; F(93) = 12200160415121876738)
pub const FIB: [u64; 93] = {
    let mut f = [0u64; 93];
    f[0] = 0;
    f[1] = 1;
    let mut i = 2;
    while i < 93 {
        f[i] = f[i - 1] + f[i - 2];
        i += 1;
    }
    f
};

/// Golden ratio approximation as integer fraction: PHI_NUM/PHI_DEN
/// Uses F(46)/F(45) = 1836311903/1134903170, accurate to ~10^{-19}
pub const PHI_NUM: u64 = 1_836_311_903;
pub const PHI_DEN: u64 = 1_134_903_170;

// ============================================================================
// MODULAR ARITHMETIC
// ============================================================================

#[inline]
fn addmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 + b as u128) % m as u128) as u64
}

#[inline]
fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

// ============================================================================
// PHI OSCILLATOR
// ============================================================================

/// Crystal modulus — a large prime for phase space.
/// 2^61 - 1 (Mersenne prime M61, 19 digits)
pub const CRYSTAL_MODULUS: u64 = (1u64 << 61) - 1;

/// The phi-resonant oscillator.
///
/// Phase advances by phi-quantized steps on each tick.
/// The step size alternates between consecutive Fibonacci numbers,
/// producing a maximally aperiodic trajectory through phase space.
#[derive(Clone, Debug)]
pub struct PhiOscillator {
    /// Current phase in [0, CRYSTAL_MODULUS)
    phase: u64,
    /// Fibonacci index for current step size
    fib_index: usize,
    /// Total ticks since creation
    tick_count: u64,
    /// Modulus for phase arithmetic
    modulus: u64,
}

impl PhiOscillator {
    /// Create oscillator with seed determining initial phase.
    pub fn new(seed: u64) -> Self {
        // Hash seed into initial phase
        let phase = Self::hash_seed(seed);
        Self {
            phase,
            fib_index: 10, // Start at F(10) = 55
            tick_count: 0,
            modulus: CRYSTAL_MODULUS,
        }
    }

    /// Simple integer hash for seed → phase mapping.
    fn hash_seed(seed: u64) -> u64 {
        let mut h = seed;
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33;
        h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
        h ^= h >> 33;
        h % CRYSTAL_MODULUS
    }

    /// Advance one tick. Phase moves by F(fib_index) mod CRYSTAL_MODULUS.
    /// Fibonacci index advances cyclically through [10..80].
    pub fn tick(&mut self) {
        let step = FIB[self.fib_index];
        self.phase = addmod(self.phase, step, self.modulus);
        self.tick_count += 1;

        // Advance Fibonacci index (cycle through 10..80)
        self.fib_index += 1;
        if self.fib_index >= 80 {
            self.fib_index = 10;
        }
    }

    /// Advance N ticks.
    pub fn tick_n(&mut self, n: u64) {
        for _ in 0..n {
            self.tick();
        }
    }

    /// Current phase value.
    pub fn phase(&self) -> u64 {
        self.phase
    }

    /// Current tick count.
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    /// Extract entropy bytes from current phase state.
    /// Uses a mixing function to spread phase bits.
    pub fn extract_entropy(&self) -> [u8; 8] {
        let mixed = self.mix_phase();
        mixed.to_le_bytes()
    }

    /// Mix phase with tick count for high-quality entropy.
    fn mix_phase(&self) -> u64 {
        let mut h = self.phase;
        h ^= self.tick_count.wrapping_mul(PHI_NUM);
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33;
        h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
        h ^= h >> 33;
        h
    }
}

// ============================================================================
// FIBONACCI GATE
// ============================================================================

/// Aperiodic gate that fires at Fibonacci-indexed tick counts.
///
/// The gate fires when `tick_count` equals a Fibonacci number.
/// This produces an aperiodic firing pattern that no adversary
/// can predict without knowing the exact tick count.
#[derive(Clone, Debug)]
pub struct FibonacciGate {
    /// Index of next Fibonacci gate threshold
    next_gate_index: usize,
    /// Number of times the gate has fired
    fire_count: u64,
    /// Phase snapshot at each gate firing (ring buffer)
    snapshots: Vec<u64>,
    /// Max snapshots to retain
    max_snapshots: usize,
}

impl FibonacciGate {
    pub fn new() -> Self {
        Self {
            next_gate_index: 5, // First gate at F(5) = 5 ticks
            fire_count: 0,
            snapshots: Vec::new(),
            max_snapshots: 256,
        }
    }

    /// Check if the gate should fire at this tick count.
    /// Returns true if it fires, false otherwise.
    pub fn check(&mut self, tick_count: u64, phase: u64) -> bool {
        if self.next_gate_index >= 93 {
            // Wrapped past max Fibonacci — reset with offset
            self.next_gate_index = 10;
        }

        if tick_count >= FIB[self.next_gate_index] {
            self.fire_count += 1;
            self.snapshots.push(phase);
            if self.snapshots.len() > self.max_snapshots {
                self.snapshots.remove(0);
            }
            self.next_gate_index += 1;
            true
        } else {
            false
        }
    }

    pub fn fire_count(&self) -> u64 {
        self.fire_count
    }

    /// Get the last N phase snapshots (most recent first).
    pub fn recent_snapshots(&self, n: usize) -> Vec<u64> {
        self.snapshots.iter().rev().take(n).copied().collect()
    }

    /// Compute gate entropy: XOR of all recent snapshots.
    /// This is high-quality entropy because each snapshot captures
    /// phase at an unpredictable (Fibonacci-indexed) time.
    pub fn gate_entropy(&self) -> u64 {
        self.snapshots.iter().fold(0u64, |acc, &s| acc ^ s)
    }
}

impl Default for FibonacciGate {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// DIGITAL TIME CRYSTAL (Combined)
// ============================================================================

/// The complete Digital Time Crystal: oscillator + gate.
///
/// Provides:
/// - Deterministic phase trajectory (reproducible from seed)
/// - Aperiodic gate events at Fibonacci ticks
/// - High-quality entropy extraction at each gate event
/// - Phase readout for CDHS health monitoring
#[derive(Clone, Debug)]
pub struct TimeCrystal {
    oscillator: PhiOscillator,
    gate: FibonacciGate,
}

/// Event emitted when the gate fires.
#[derive(Clone, Debug)]
pub struct GateEvent {
    pub tick: u64,
    pub phase: u64,
    pub entropy: [u8; 8],
    pub fire_count: u64,
    pub cumulative_entropy: u64,
}

impl TimeCrystal {
    /// Create a new time crystal with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            oscillator: PhiOscillator::new(seed),
            gate: FibonacciGate::new(),
        }
    }

    /// Advance one tick. Returns Some(GateEvent) if the gate fires.
    pub fn tick(&mut self) -> Option<GateEvent> {
        self.oscillator.tick();

        let fired = self.gate.check(
            self.oscillator.tick_count(),
            self.oscillator.phase(),
        );

        if fired {
            Some(GateEvent {
                tick: self.oscillator.tick_count(),
                phase: self.oscillator.phase(),
                entropy: self.oscillator.extract_entropy(),
                fire_count: self.gate.fire_count(),
                cumulative_entropy: self.gate.gate_entropy(),
            })
        } else {
            None
        }
    }

    /// Advance N ticks, collecting all gate events.
    pub fn tick_n(&mut self, n: u64) -> Vec<GateEvent> {
        let mut events = Vec::new();
        for _ in 0..n {
            if let Some(event) = self.tick() {
                events.push(event);
            }
        }
        events
    }

    /// Run until the next gate event fires. Returns the event.
    /// Safety: will not run more than `max_ticks` to prevent infinite loop.
    pub fn tick_until_gate(&mut self, max_ticks: u64) -> Option<GateEvent> {
        for _ in 0..max_ticks {
            if let Some(event) = self.tick() {
                return Some(event);
            }
        }
        None
    }

    /// Current phase (for CDHS readout).
    pub fn phase(&self) -> u64 {
        self.oscillator.phase()
    }

    /// Current tick count.
    pub fn tick_count(&self) -> u64 {
        self.oscillator.tick_count()
    }

    /// Total gate fires.
    pub fn gate_fire_count(&self) -> u64 {
        self.gate.fire_count()
    }

    /// Cumulative gate entropy.
    pub fn cumulative_entropy(&self) -> u64 {
        self.gate.gate_entropy()
    }

    /// Extract entropy bytes from current oscillator state.
    pub fn extract_entropy(&self) -> [u8; 8] {
        self.oscillator.extract_entropy()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Fibonacci table ---

    #[test]
    fn fibonacci_table_correct() {
        assert_eq!(FIB[0], 0);
        assert_eq!(FIB[1], 1);
        assert_eq!(FIB[2], 1);
        assert_eq!(FIB[10], 55);
        assert_eq!(FIB[20], 6765);
        assert_eq!(FIB[45], PHI_DEN);
        assert_eq!(FIB[46], PHI_NUM);
        // Verify recurrence for all entries
        for i in 2..93 {
            assert_eq!(FIB[i], FIB[i - 1] + FIB[i - 2], "Fibonacci recurrence failed at {i}");
        }
    }

    // --- Oscillator ---

    #[test]
    fn oscillator_deterministic() {
        let mut osc1 = PhiOscillator::new(42);
        let mut osc2 = PhiOscillator::new(42);
        for _ in 0..1000 {
            osc1.tick();
            osc2.tick();
        }
        assert_eq!(osc1.phase(), osc2.phase(), "Same seed should give same phase");
        assert_eq!(osc1.tick_count(), osc2.tick_count());
    }

    #[test]
    fn oscillator_different_seeds_diverge() {
        let mut osc1 = PhiOscillator::new(1);
        let mut osc2 = PhiOscillator::new(2);
        for _ in 0..100 {
            osc1.tick();
            osc2.tick();
        }
        assert_ne!(osc1.phase(), osc2.phase(), "Different seeds should diverge");
    }

    #[test]
    fn oscillator_phase_bounded() {
        let mut osc = PhiOscillator::new(0);
        for _ in 0..10000 {
            osc.tick();
            assert!(osc.phase() < CRYSTAL_MODULUS, "Phase must stay within modulus");
        }
    }

    #[test]
    fn oscillator_entropy_varies() {
        let mut osc = PhiOscillator::new(42);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..100 {
            osc.tick();
            seen.insert(osc.extract_entropy());
        }
        assert!(seen.len() > 90, "Entropy should be highly varied, got {} unique / 100", seen.len());
    }

    // --- Fibonacci Gate ---

    #[test]
    fn gate_fires_at_fibonacci_ticks() {
        let mut gate = FibonacciGate::new();
        // First gate is at F(5) = 5
        // Tick 1..4 should not fire, tick 5 should
        for t in 1..5 {
            assert!(!gate.check(t, t * 100), "Gate should not fire at tick {t}");
        }
        assert!(gate.check(5, 500), "Gate should fire at tick 5 (F(5))");
        assert_eq!(gate.fire_count(), 1);
    }

    #[test]
    fn gate_fires_increase_monotonically() {
        let mut gate = FibonacciGate::new();
        let mut last_fire = 0u64;
        for t in 1..10000 {
            if gate.check(t, t) {
                assert!(t > last_fire, "Gate fires should be monotonically increasing");
                last_fire = t;
            }
        }
        assert!(gate.fire_count() > 5, "Should have multiple gate fires in 10000 ticks");
    }

    #[test]
    fn gate_snapshots_captured() {
        let mut gate = FibonacciGate::new();
        for t in 1..100 {
            gate.check(t, t * 7);
        }
        let snaps = gate.recent_snapshots(3);
        assert!(!snaps.is_empty(), "Should have captured snapshots");
    }

    // --- Time Crystal (Combined) ---

    #[test]
    fn crystal_deterministic() {
        let mut c1 = TimeCrystal::new(42);
        let mut c2 = TimeCrystal::new(42);

        let events1 = c1.tick_n(1000);
        let events2 = c2.tick_n(1000);

        assert_eq!(events1.len(), events2.len(), "Same seed should produce same gate events");
        for (e1, e2) in events1.iter().zip(&events2) {
            assert_eq!(e1.tick, e2.tick);
            assert_eq!(e1.phase, e2.phase);
            assert_eq!(e1.entropy, e2.entropy);
        }
    }

    #[test]
    fn crystal_produces_gate_events() {
        let mut crystal = TimeCrystal::new(0);
        let events = crystal.tick_n(10000);
        // With Fibonacci gating from F(5)=5 through ~F(20)=6765, should get many events
        assert!(events.len() >= 10, "Expected at least 10 gate events in 10000 ticks, got {}", events.len());
    }

    #[test]
    fn crystal_tick_until_gate() {
        let mut crystal = TimeCrystal::new(99);
        let event = crystal.tick_until_gate(10000);
        assert!(event.is_some(), "Should find a gate event within 10000 ticks");
        let e = event.unwrap();
        assert!(e.tick > 0);
        assert!(e.fire_count == 1);
    }

    #[test]
    fn crystal_entropy_nonzero() {
        let mut crystal = TimeCrystal::new(42);
        crystal.tick_n(1000);
        let entropy = crystal.cumulative_entropy();
        assert_ne!(entropy, 0, "Cumulative entropy should be nonzero after gate events");
    }

    #[test]
    fn crystal_aperiodic_gate_spacing() {
        let mut crystal = TimeCrystal::new(7);
        let events = crystal.tick_n(10000);

        // Gate spacings should NOT be constant (aperiodic)
        if events.len() >= 3 {
            let spacings: Vec<u64> = events.windows(2)
                .map(|w| w[1].tick - w[0].tick)
                .collect();
            let all_same = spacings.windows(2).all(|w| w[0] == w[1]);
            assert!(!all_same, "Gate spacings should be aperiodic (Fibonacci), not constant");
        }
    }

    #[test]
    fn crystal_phase_readout_for_cdhs() {
        let mut crystal = TimeCrystal::new(42);
        let initial_phase = crystal.phase();
        crystal.tick_n(100);
        let after_phase = crystal.phase();
        assert_ne!(initial_phase, after_phase, "Phase should change after ticking");
        assert!(crystal.tick_count() == 100);
    }
}
