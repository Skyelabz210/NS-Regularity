//! # RamanujanMemoryLane: Cross-Domain Memory via Number-Theoretic Resonance
//!
//! Encodes Ramanujan harmonic residues through a GearManifold accumulator
//! into a 144-band WASSAN substrate. The resulting winding state has
//! simultaneous valid interpretations across three domains:
//!
//! | Domain     | Zero state        | Resonance event        |
//! |------------|-------------------|------------------------|
//! | CRAM/CRT   | K_vort = 0        | Winding number changes |
//! | Fluid (NS) | Zero vorticity    | Stretching fires       |
//! | Quantum    | Ground state |0⟩  | Measurement collapses  |
//!
//! ## The Permanent Non-Resonance Theorem (integrated)
//!
//! For the N-doubling sequence N = N₀·2^k, the shadow lane (prime 11)
//! NEVER resonates with the NS transition coefficients c(N) because
//! gcd(N₀·2^k, 11) = 1 (since 11 ∤ 2 and gcd(N₀, 11) = 1 for N₀ = 8).
//! The shadow lane stays at MultidimensionalZero forever — not by accident
//! but by theorem. This structural absence IS the regularity signal.
//!
//! ## The Spectral Fingerprint Bridge
//!
//! The sieve power SP(p, d) = (2p - 1 - c_p(d)) / p² classifies both
//! prime gaps AND NS transitions via the same Ramanujan sum c_p(d).
//! When c_p(d) = -1 (non-resonant): SP = 2/p (maximum).
//! When c_p(d) = p-1 (resonant): SP = 1/p (reduced).
//! The Permanent Non-Resonance Theorem guarantees SP = 2/p always
//! for NS coefficients in the doubling sequence.

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

use qmnf_primitives::gear_manifold::GearManifold;

// ═══════════════════════════════════════════════════════════════
// RAMANUJAN SUM
// ═══════════════════════════════════════════════════════════════

/// Ramanujan sum c_q(n) for prime q: exact integer.
/// c_q(n) = q-1 if q|n, else -1.
pub fn ramanujan_sum_prime(q: u64, n: u64) -> i64 {
    if n == 0 { return q as i64 - 1; } // Convention: 0 is divisible by everything
    if n % q == 0 { q as i64 - 1 } else { -1 }
}

/// Sieve power SP(p, d) = (2p - 1 - c_p(d)) / p²
/// Returns (numerator, denominator) as exact integers.
/// Theorem T6-1 (Lean 4 sorry-free).
pub fn sieve_power(p: u64, d: u64) -> (u64, u64) {
    let c = ramanujan_sum_prime(p, d);
    let num = (2 * p as i64 - 1 - c) as u64;
    let den = p * p;
    (num, den)
}

/// Check if n resonates at prime q (q divides n)
pub fn resonates(q: u64, n: u64) -> bool {
    n > 0 && n % q == 0
}

// ═══════════════════════════════════════════════════════════════
// MULTIDIMENSIONAL ZERO
// ═══════════════════════════════════════════════════════════════

/// The multidimensional zero: an EMPTY winding vector.
/// This is distinct from a zero-filled vector (which has meaning:
/// "winding measured, found to be zero in each lane").
/// MDZero means "no winding has been measured at all."
pub type MultidimensionalZero = Vec<i128>;

/// Create the canonical multidimensional zero
pub fn mdzero() -> MultidimensionalZero { Vec::new() }

/// Check if a winding vector is the multidimensional zero
pub fn is_mdzero(v: &[i128]) -> bool { v.is_empty() }

// ═══════════════════════════════════════════════════════════════
// LANE ROLES
// ═══════════════════════════════════════════════════════════════

/// Role of a harmonic lane in the cross-domain bridge
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LaneRole {
    /// Active lane: writes winding at every step
    Active,
    /// Shadow anchor: holds MDZero until resonance fires
    /// Prime 11 in the QMNF taxonomy: "structure that exists
    /// but cannot be directly measured"
    ShadowAnchor,
}

impl LaneRole {
    pub fn is_shadow_anchor(&self) -> bool {
        matches!(self, LaneRole::ShadowAnchor)
    }
}

/// Harmonic operator applied to the Ramanujan sum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HarmonicOp { Add, Sub }

// ═══════════════════════════════════════════════════════════════
// HARMONIC LANE
// ═══════════════════════════════════════════════════════════════

/// A single lane tracking Ramanujan winding for a specific prime
#[derive(Debug, Clone)]
pub struct HarmonicLane {
    /// The prime driving this lane
    pub q: u64,
    /// The operator applied to the Ramanujan sum
    pub op: HarmonicOp,
    /// The role of this lane
    pub role: LaneRole,
    /// GearManifold accumulating winding
    pub gear: GearManifold,
}

impl HarmonicLane {
    pub fn new(q: u64, op: HarmonicOp, role: LaneRole, crt_moduli: &[u64]) -> Self {
        HarmonicLane {
            q, op, role,
            gear: GearManifold::new(crt_moduli),
        }
    }

    /// Advance by one step with input n
    pub fn step(&mut self, n: u64) -> i64 {
        let c = ramanujan_sum_prime(self.q, n);
        let delta = match self.op {
            HarmonicOp::Add => c,
            HarmonicOp::Sub => -c,
        };
        self.gear.add(delta as i128);
        delta
    }

    /// Does n resonate at this lane's prime?
    /// FIXED: reviewer bug #2 — use this instead of checking delta value
    pub fn resonance_at(&self, n: u64) -> bool {
        resonates(self.q, n)
    }

    /// Primary winding (fastest CRT lane)
    pub fn primary_winding(&self) -> i128 {
        self.gear.primary_winding()
    }

    /// Full winding vector
    pub fn winding_vector(&self) -> Vec<i128> {
        self.gear.magnitudes()
    }

    /// Reset to zero state
    pub fn reset(&mut self) {
        self.gear.reset();
    }
}

// ═══════════════════════════════════════════════════════════════
// WASSAN FIELD (144-band substrate)
// ═══════════════════════════════════════════════════════════════

/// 144-band WASSAN storage field
/// Each band holds a winding vector (or MDZero = empty)
#[derive(Debug, Clone)]
pub struct WassanField {
    pub bands: Vec<MultidimensionalZero>,
    pub num_bands: usize,
}

impl WassanField {
    pub fn new(num_bands: usize) -> Self {
        WassanField {
            bands: (0..num_bands).map(|_| mdzero()).collect(),
            num_bands,
        }
    }

    /// Write a winding vector to a band
    pub fn write_band(&mut self, band: usize, data: Vec<i128>) -> Result<(), &'static str> {
        if band >= self.num_bands { return Err("band index out of range"); }
        self.bands[band] = data;
        Ok(())
    }

    /// Read a band
    pub fn read_band(&self, band: usize) -> Option<&[i128]> {
        self.bands.get(band).map(|v| v.as_slice())
    }

    /// Check if a band is at MDZero
    pub fn is_zero_at(&self, band: usize) -> bool {
        self.bands.get(band).map_or(true, |v| is_mdzero(v))
    }

    /// Count non-zero bands
    pub fn occupied_bands(&self) -> usize {
        self.bands.iter().filter(|b| !is_mdzero(b)).count()
    }
}

/// Compute band index for a prime at a given step
/// 28 bands per prime, distributed across 144 bands
pub fn band_index(prime_idx: usize, step: usize) -> usize {
    (prime_idx * 28 + (step % 28)) % 144
}

// ═══════════════════════════════════════════════════════════════
// RAMANUJAN MEMORY LANE (the bridge)
// ═══════════════════════════════════════════════════════════════

/// The cross-domain memory bridge
#[derive(Debug, Clone)]
pub struct RamanujanMemoryLane {
    /// Harmonic lanes (one per prime)
    pub lanes: Vec<HarmonicLane>,
    /// WASSAN storage
    pub field: WassanField,
    /// Index of the shadow lane (prime 11)
    pub shadow_lane_idx: Option<usize>,
    /// Step counter
    pub step_count: usize,
}

impl RamanujanMemoryLane {
    /// Create a standard memory lane with Safe Basis primes
    /// Shadow anchor = prime 11
    pub fn new(crt_moduli: &[u64]) -> Self {
        let specs: Vec<(u64, HarmonicOp, LaneRole)> = vec![
            (3,  HarmonicOp::Add, LaneRole::Active),
            (5,  HarmonicOp::Add, LaneRole::Active),
            (7,  HarmonicOp::Add, LaneRole::Active),
            (11, HarmonicOp::Add, LaneRole::ShadowAnchor),
            (13, HarmonicOp::Sub, LaneRole::Active),
        ];

        let mut shadow_idx = None;
        let lanes: Vec<HarmonicLane> = specs.iter().enumerate().map(|(i, (q, op, role))| {
            if role.is_shadow_anchor() { shadow_idx = Some(i); }
            HarmonicLane::new(*q, *op, *role, crt_moduli)
        }).collect();

        RamanujanMemoryLane {
            lanes,
            field: WassanField::new(144),
            shadow_lane_idx: shadow_idx,
            step_count: 0,
        }
    }

    /// Advance by one step with input n
    /// Shadow lane: writes ONLY on resonance (reviewer bug #2 FIXED)
    pub fn step(&mut self, n: u64) {
        let mut deltas = Vec::new();
        for lane in &mut self.lanes {
            let d = lane.step(n);
            deltas.push(d);
        }

        // Write winding vectors to WASSAN
        for (p_idx, lane) in self.lanes.iter().enumerate() {
            let band = band_index(p_idx, self.step_count);

            // FIXED (reviewer bug #2): check resonance, not delta value
            let should_write = if lane.role.is_shadow_anchor() {
                lane.resonance_at(n)  // Only write when prime divides n
            } else {
                true  // Active lanes always write
            };

            if should_write {
                let winding = lane.winding_vector();
                let _ = self.field.write_band(band, winding);
            }
        }

        self.step_count += 1;
    }

    /// Get the shadow lane's current state
    pub fn shadow_state(&self) -> Option<Vec<i128>> {
        self.shadow_lane_idx.map(|idx| self.lanes[idx].winding_vector())
    }

    /// Check if shadow lane has ever been written to WASSAN
    pub fn shadow_has_fired(&self) -> bool {
        if let Some(idx) = self.shadow_lane_idx {
            // Check all bands that could belong to this lane
            for step in 0..self.step_count.min(28) {
                let band = band_index(idx, step);
                if !self.field.is_zero_at(band) {
                    return true;
                }
            }
        }
        false
    }

    /// Spectral fingerprint of the current state
    /// Returns sieve power SP(p, n) for the last input n
    pub fn spectral_fingerprint(&self, n: u64) -> Vec<(u64, u64, u64)> {
        self.lanes.iter().map(|lane| {
            let (num, den) = sieve_power(lane.q, n);
            (lane.q, num, den)
        }).collect()
    }

    /// Reset all lanes and clear WASSAN
    pub fn reset(&mut self) {
        for lane in &mut self.lanes {
            lane.reset();
        }
        self.field = WassanField::new(144);
        self.step_count = 0;
    }

    /// Winding matrix: all lanes × all CRT moduli
    pub fn winding_matrix(&self) -> Vec<Vec<i128>> {
        self.lanes.iter().map(|l| l.winding_vector()).collect()
    }
}

// ═══════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramanujan_sum_correctness() {
        // c_7(14) = 6 (7 | 14)
        assert_eq!(ramanujan_sum_prime(7, 14), 6);
        // c_7(15) = -1 (7 ∤ 15)
        assert_eq!(ramanujan_sum_prime(7, 15), -1);
        // c_11(33) = 10 (11 | 33)
        assert_eq!(ramanujan_sum_prime(11, 33), 10);
        // c_11(34) = -1 (11 ∤ 34)
        assert_eq!(ramanujan_sum_prime(11, 34), -1);
        // c_13(26) = 12 (13 | 26)
        assert_eq!(ramanujan_sum_prime(13, 26), 12);
    }

    #[test]
    fn test_sieve_power_t6_1() {
        // T6-1: SP(p, d) = (2p - 1 - c_p(d)) / p²
        // Non-resonant (p ∤ d): SP = 2p/p² = 2/p
        let (num, den) = sieve_power(7, 5);
        assert_eq!(num * 7, 2 * den, "SP(7, 5) = 2/7");

        // Resonant (p | d): SP = p/p² = 1/p
        let (num, den) = sieve_power(7, 14);
        assert_eq!(num * 7, den, "SP(7, 14) = 1/7");

        // T6-3: difference = 1/p
        let (num_nr, den_nr) = sieve_power(11, 5);
        let (num_r, den_r) = sieve_power(11, 22);
        assert_eq!(den_nr, den_r, "same denominator");
        assert_eq!((num_nr - num_r) * 11, den_nr, "difference = 1/11");
    }

    #[test]
    fn test_shadow_lane_stays_zero_without_resonance() {
        let moduli = [3u64, 7, 11, 13];
        let mut mem = RamanujanMemoryLane::new(&moduli);

        // Feed numbers NOT divisible by 11
        for n in [1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
            mem.step(n);
        }

        // Shadow lane (11) should NOT have fired
        assert!(!mem.shadow_has_fired(),
            "Shadow lane should not fire for n=1..10 (none divisible by 11)");
    }

    #[test]
    fn test_shadow_lane_fires_on_resonance() {
        let moduli = [3u64, 7, 11, 13];
        let mut mem = RamanujanMemoryLane::new(&moduli);

        // Feed 10 non-resonant, then one resonant (n=11)
        for n in 1..=10 {
            mem.step(n);
        }
        assert!(!mem.shadow_has_fired(), "Pre-resonance: shadow quiet");

        mem.step(11); // RESONANCE: 11 | 11
        assert!(mem.shadow_has_fired(), "Post-resonance: shadow fired");
    }

    #[test]
    fn test_shadow_with_sub_operator() {
        // Reviewer suggestion: test Sub operator on shadow lane
        let moduli = [3u64, 7, 11, 13];
        let specs: Vec<(u64, HarmonicOp, LaneRole)> = vec![
            (7,  HarmonicOp::Add, LaneRole::Active),
            (11, HarmonicOp::Sub, LaneRole::ShadowAnchor), // Sub, not Add
            (13, HarmonicOp::Add, LaneRole::Active),
        ];

        let mut shadow_idx = None;
        let lanes: Vec<HarmonicLane> = specs.iter().enumerate().map(|(i, (q, op, role))| {
            if role.is_shadow_anchor() { shadow_idx = Some(i); }
            HarmonicLane::new(*q, *op, *role, &moduli)
        }).collect();

        let mut mem = RamanujanMemoryLane {
            lanes,
            field: WassanField::new(144),
            shadow_lane_idx: shadow_idx,
            step_count: 0,
        };

        // Non-resonant steps
        for n in 1..=10 { mem.step(n); }
        assert!(!mem.shadow_has_fired(), "Sub shadow: quiet before resonance");

        // Resonant step
        mem.step(22); // 11 | 22
        assert!(mem.shadow_has_fired(), "Sub shadow: fires on resonance");
    }

    #[test]
    fn test_permanent_non_resonance_ns() {
        // The Permanent Non-Resonance Theorem:
        // For N = 8·2^k, the NS coefficient c(N) = dt·ν·N²·SCALE⁻¹ mod p
        // is NEVER divisible by any admissible prime p ∈ {7, 11, 13}
        let scale: u64 = 10000;
        let dt: u64 = 100;
        let nu: u64 = 6;
        let primes = [7u64, 11, 13];

        for &p in &primes {
            let scale_inv = mod_pow(scale % p, p - 2, p);
            let mut n: u64 = 8;
            for _ in 0..30 { // Full 30-step cycle
                let c = {
                    let a = (dt % p) * (nu % p) % p;
                    let b = (n % p) * (n % p) % p;
                    a * b % p * scale_inv % p
                };
                assert_ne!(c, 0,
                    "Permanent non-resonance VIOLATED: c({}) ≡ 0 mod {}", n, p);

                // Also verify Ramanujan sum = -1 (non-resonant)
                assert_eq!(ramanujan_sum_prime(p, c), -1,
                    "c({}) mod {} = {} should be non-resonant", n, p, c);

                // Verify SP = 2/p (maximum)
                let (num, den) = sieve_power(p, c);
                assert_eq!(num * p, 2 * den,
                    "SP({}, c({})) should be 2/{}", p, n, p);

                n *= 2;
            }
        }
    }

    #[test]
    fn test_erasure_in_cycle() {
        // Erasure: 6c ≡ 1 (mod p) → self-coefficient = 0
        // First occurrence: p=7 at N=32 (c=6), p=11 at N=64 (c=2), p=13 at N=32 (c=11)
        let scale: u64 = 10000;
        let dt: u64 = 100;
        let nu: u64 = 6;

        for &(p, expected_first_n) in &[(7u64, 32u64), (11, 64), (13, 32)] {
            let scale_inv = mod_pow(scale % p, p - 2, p);
            let mut n: u64 = 8;
            let mut first_erasure: Option<u64> = None;
            for _ in 0..30 {
                let c = {
                    let a = (dt % p) * (nu % p) % p;
                    let b = (n % p) * (n % p) % p;
                    a * b % p * scale_inv % p
                };
                let six_c = (6 * c) % p;
                if six_c == 1 && first_erasure.is_none() {
                    first_erasure = Some(n);
                }
                n = n.saturating_mul(2);
            }
            assert_eq!(first_erasure, Some(expected_first_n),
                "First erasure for p={}: expected N={}, got {:?}",
                p, expected_first_n, first_erasure);
        }
    }

    #[test]
    fn test_active_lanes_write_every_step() {
        let moduli = [3u64, 7, 11, 13];
        let mut mem = RamanujanMemoryLane::new(&moduli);

        for n in 1..=28 {
            mem.step(n);
        }

        // Active lanes (idx 0,1,2,4) should have written to their bands
        // Shadow lane (idx 3 = prime 11) should only have written at n=11,22
        let active_occupied = (0..28).filter(|&s| {
            !mem.field.is_zero_at(band_index(0, s)) // lane 0 (prime 3)
        }).count();

        assert_eq!(active_occupied, 28,
            "Active lane should write every step");
    }

    #[test]
    fn test_spectral_fingerprint_ns_bridge() {
        // The spectral fingerprint of NS coefficients should be
        // constant at SP = 2/p for all N in the doubling sequence
        let scale: u64 = 10000;
        let dt: u64 = 100;
        let nu: u64 = 6;
        let primes = [3u64, 5, 7, 11, 13];

        let mut n: u64 = 8;
        for _ in 0..30 {
            for &p in &[7u64, 11, 13] { // Admissible primes only
                let scale_inv = mod_pow(scale % p, p - 2, p);
                let c = {
                    let a = (dt % p) * (nu % p) % p;
                    let b = (n % p) * (n % p) % p;
                    a * b % p * scale_inv % p
                };
                let (num, den) = sieve_power(p, c);
                // SP should be 2/p = 2p/p² → num = 2p
                assert_eq!(num, 2 * p,
                    "SP({}, c({})) numerator should be 2×{} = {}, got {}",
                    p, n, p, 2*p, num);
            }
            n *= 2;
        }
    }

    #[test]
    fn test_winding_boundedness() {
        let moduli = [3u64, 7, 11, 13];
        let mut mem = RamanujanMemoryLane::new(&moduli);

        // Run 100 steps
        for n in 1..=100 {
            mem.step(n);
        }

        // All windings should be bounded (not growing unboundedly)
        let matrix = mem.winding_matrix();
        for (i, row) in matrix.iter().enumerate() {
            let max_w = row.iter().map(|w| w.abs()).max().unwrap_or(0);
            assert!(max_w < 1000,
                "Lane {} winding {} is too large after 100 steps", i, max_w);
        }
    }

    /// Modular exponentiation for tests
    fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
        let mut result = 1u64;
        base %= modulus;
        while exp > 0 {
            if exp % 2 == 1 { result = result * base % modulus; }
            exp /= 2;
            base = base * base % modulus;
        }
        result
    }
}
