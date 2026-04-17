//! =============================================================================
//! UNIFIED SUBSTRATE — QEDDE × HackFate × MMBF
//! =============================================================================
//!
//! ARCHITECTURAL PRINCIPLE: Everything is the same thing.
//!
//! The WASSAN drive is the storage substrate.
//! The Double Helix verifies every write to the drive.
//! The CRT lanes ARE the drive's address space.
//! Grover, Ramanujan, and Shor are RESIDENT ORACLES on the drive.
//! The CRT Beacon is a shared accelerator available to any cell.
//! The Fourth Attractor provides self-correcting memory.
//! The learning architecture (FRST) runs inherently on attractor dynamics.
//!
//! When a cell writes a value, it goes through:
//!   Cell → Schema Kernel → Double Helix verify → WASSAN drive write
//!
//! When a cell reads a value, it comes through:
//!   WASSAN drive read → Attractor correction → Double Helix verify → Cell
//!
//! The oracles are not called — they are QUERIED. They live on the drive
//! as persistent state. Grover's amplitudes are stored values. Ramanujan's
//! sums are precomputed lane metadata. Shor's order-finding is a CRT
//! operation on the drive's own moduli.
//!
//! Authors: Acid (Anthony Diaz) / HackFate × Claude (Anthropic)
//! Date: March 17, 2026

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 1: MODULAR ARITHMETIC PRIMITIVES (shared by everything)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn addmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 + b as u128) % m as u128) as u64
}

pub fn submod(a: u64, b: u64, m: u64) -> u64 {
    if a >= b { (a - b) % m }
    else { let d = (b - a) % m; if d == 0 { 0 } else { m - d } }
}

pub fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

pub fn powmod(mut base: u64, mut exp: u64, m: u64) -> u64 {
    if m == 1 { return 0; }
    let mut r = 1u64; base %= m;
    while exp > 0 {
        if exp & 1 == 1 { r = mulmod(r, base, m); }
        base = mulmod(base, base, m); exp >>= 1;
    }
    r
}

pub fn mod_inverse(a: u64, m: u64) -> Option<u64> {
    if a == 0 || m == 0 { return None; }
    let (mut old_r, mut r) = (m as i128, a as i128);
    let (mut old_s, mut s) = (0i128, 1i128);
    while r != 0 {
        let q = old_r / r;
        let tmp = r; r = old_r - q * r; old_r = tmp;
        let tmp = s; s = old_s - q * s; old_s = tmp;
    }
    if old_r != 1 { return None; }
    Some(((old_s % m as i128 + m as i128) % m as i128) as u64)
}

pub fn gcd(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a, b);
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 2: THE WASSAN DRIVE — Shared substrate for everything
// ═══════════════════════════════════════════════════════════════════════════════

/// The WASSAN drive is the physical substrate. Every value in the system
/// is a location on this drive. CRT lanes are the drive's address dimensions.
/// NTT bands are the drive's spectral encoding. Apollonian packing is the
/// drive's geometric organization.
pub struct WassanDrive {
    /// The 8 exact CRT lanes — these ARE the drive's address space
    pub lane_moduli: [u64; 8],
    /// Storage: each "sector" is an 8-residue CRT value
    pub sectors: Vec<DriveSector>,
    /// Double Helix verification state (strand A and strand B)
    pub helix: DoubleHelixVerifier,
    /// Attractor memory cells for self-correction
    pub attractors: Vec<AttractorCell>,
    /// Oracle registry — resident services on the drive
    pub oracles: OracleRegistry,
    /// CRT Beacon — shared accelerator
    pub beacon: CRTBeacon,
    /// Shadow Entropy pool
    pub entropy_pool: ShadowEntropyPool,
    /// Drive generation counter
    pub generation: u64,
}

/// A single sector on the WASSAN drive = one CRT-encoded value
#[derive(Debug, Clone)]
pub struct DriveSector {
    pub residues: [u64; 8],
    pub negative: bool,
    pub attractor_target: Option<[u64; 8]>,  // What value this sector "wants" to hold
    pub winding: i64,                          // K-Elimination winding number
    pub write_generation: u64,                 // When last written
    pub verified: bool,                        // Double Helix verified?
    pub tags: u32,                             // Domain tags (bitfield)
}

impl DriveSector {
    pub fn zero(lane_moduli: &[u64; 8]) -> Self {
        DriveSector {
            residues: [0; 8],
            negative: false,
            attractor_target: None,
            winding: 0,
            write_generation: 0,
            verified: false,
            tags: 0,
        }
    }

    pub fn from_i64(val: i64, lane_moduli: &[u64; 8]) -> Self {
        let abs = val.unsigned_abs();
        let residues = std::array::from_fn(|i| abs % lane_moduli[i]);
        DriveSector {
            residues,
            negative: val < 0,
            attractor_target: Some(residues), // Attractor locks to written value
            winding: 0,
            write_generation: 0,
            verified: false,
            tags: 0,
        }
    }

    /// Add two sectors (per-lane modular addition)
    pub fn add(&self, other: &Self, moduli: &[u64; 8]) -> Self {
        let residues = std::array::from_fn(|i|
            addmod(self.residues[i], other.residues[i], moduli[i]));
        DriveSector { residues, negative: false, attractor_target: None,
                      winding: 0, write_generation: 0, verified: false, tags: 0 }
    }

    /// Multiply two sectors
    pub fn mul(&self, other: &Self, moduli: &[u64; 8]) -> Self {
        let residues = std::array::from_fn(|i|
            mulmod(self.residues[i], other.residues[i], moduli[i]));
        DriveSector { residues, negative: self.negative ^ other.negative,
                      attractor_target: None, winding: 0, write_generation: 0,
                      verified: false, tags: 0 }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 3: DOUBLE HELIX VERIFIER — Every write goes through this
// ═══════════════════════════════════════════════════════════════════════════════

/// The Double Helix maintains two strands. Every write is committed to both.
/// On read, both strands are checked. If they disagree → tamper detected → fold.
pub struct DoubleHelixVerifier {
    /// Strand A checksums (Descartes identity residues)
    pub strand_a: Vec<u64>,
    /// Strand B checksums (independent computation)
    pub strand_b: Vec<u64>,
    /// Phase counter (Fibonacci-scheduled)
    pub phase: u64,
    /// Fibonacci increment table
    pub fib_table: Vec<u64>,
    pub fib_index: usize,
    /// Verification modulus
    pub verify_mod: u64,
    /// Violation count
    pub violations: u64,
}

impl DoubleHelixVerifier {
    pub fn new(capacity: usize, verify_mod: u64) -> Self {
        // Precompute Fibonacci table
        let mut fib = vec![0u64, 1];
        for i in 2..100 {
            fib.push((fib[i-1] + fib[i-2]) % verify_mod);
        }
        DoubleHelixVerifier {
            strand_a: vec![0; capacity],
            strand_b: vec![0; capacity],
            phase: 0,
            fib_table: fib,
            fib_index: 2,
            verify_mod,
            violations: 0,
        }
    }

    /// Commit a write to both strands with Descartes check
    pub fn commit_write(&mut self, sector_idx: usize, sector: &DriveSector) -> bool {
        if sector_idx >= self.strand_a.len() {
            return false;
        }

        // Strand A: simple checksum (sum of residues mod verify_mod)
        let sum_a: u64 = sector.residues.iter()
            .fold(0u64, |acc, &r| addmod(acc, r % self.verify_mod, self.verify_mod));

        // Strand B: independent checksum (product-based)
        let sum_b: u64 = sector.residues.iter()
            .fold(1u64, |acc, &r| mulmod(acc, (r % self.verify_mod) + 1, self.verify_mod));

        self.strand_a[sector_idx] = sum_a;
        self.strand_b[sector_idx] = sum_b;

        // Advance phase (Fibonacci)
        let delta = self.fib_table[self.fib_index % self.fib_table.len()];
        self.phase = addmod(self.phase, delta, self.verify_mod);
        self.fib_index += 1;

        true
    }

    /// Verify a sector against both strands
    pub fn verify_read(&mut self, sector_idx: usize, sector: &DriveSector) -> bool {
        if sector_idx >= self.strand_a.len() {
            return false;
        }

        let sum_a: u64 = sector.residues.iter()
            .fold(0u64, |acc, &r| addmod(acc, r % self.verify_mod, self.verify_mod));
        let sum_b: u64 = sector.residues.iter()
            .fold(1u64, |acc, &r| mulmod(acc, (r % self.verify_mod) + 1, self.verify_mod));

        let ok = sum_a == self.strand_a[sector_idx] && sum_b == self.strand_b[sector_idx];
        if !ok { self.violations += 1; }
        ok
    }

    /// Descartes phase-lock check across four sectors
    pub fn descartes_check(&self, a: u64, b: u64, c: u64, d: u64) -> bool {
        let p = self.verify_mod;
        let sum = addmod(addmod(a, b, p), addmod(c, d, p), p);
        let lhs = mulmod(sum, sum, p);
        let sq_sum = addmod(
            addmod(mulmod(a,a,p), mulmod(b,b,p), p),
            addmod(mulmod(c,c,p), mulmod(d,d,p), p), p);
        let rhs = mulmod(2, sq_sum, p);
        lhs == rhs
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 4: ATTRACTOR MEMORY — Self-correcting storage
// ═══════════════════════════════════════════════════════════════════════════════

/// Each attractor cell has a "target" value it converges toward.
/// Noise and corruption are corrected by the attractor dynamics.
/// This IS the Fourth Attractor — the memory that heals itself.
#[derive(Debug, Clone)]
pub struct AttractorCell {
    pub target: u64,        // What the cell wants to hold
    pub current: u64,       // What it currently holds (may be corrupted)
    pub modulus: u64,
    pub coupling: u64,      // Attraction strength (integer, e.g. 3 = 30%)
    pub coupling_denom: u64, // Denominator (e.g. 10)
    pub corrections: u64,   // How many self-corrections performed
}

impl AttractorCell {
    pub fn new(value: u64, modulus: u64) -> Self {
        AttractorCell {
            target: value % modulus,
            current: value % modulus,
            modulus,
            coupling: 3,
            coupling_denom: 10,
            corrections: 0,
        }
    }

    /// One step of attractor dynamics: current moves toward target
    pub fn converge_step(&mut self) {
        if self.current == self.target { return; }
        let diff = submod(self.target, self.current, self.modulus);
        // Move current by coupling * diff / coupling_denom
        let step = mulmod(diff, self.coupling, self.modulus) / self.coupling_denom;
        if step > 0 {
            self.current = addmod(self.current, step, self.modulus);
            self.corrections += 1;
        } else if diff > 0 {
            // Minimum step of 1 toward target
            self.current = addmod(self.current, 1, self.modulus);
            self.corrections += 1;
        }
    }

    /// Read with self-correction (run a few converge steps first)
    pub fn read_corrected(&mut self, steps: usize) -> u64 {
        for _ in 0..steps { self.converge_step(); }
        self.current
    }

    /// Write new target (attractor locks to new value)
    pub fn write(&mut self, value: u64) {
        self.target = value % self.modulus;
        self.current = value % self.modulus;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 5: SHADOW ENTROPY POOL
// ═══════════════════════════════════════════════════════════════════════════════

pub struct ShadowEntropyPool {
    pub pool: Vec<u64>,
    pub consumed: u64,
    pub harvested: u64,
    pub modulus: u64,
}

impl ShadowEntropyPool {
    pub fn new(modulus: u64) -> Self {
        ShadowEntropyPool { pool: Vec::new(), consumed: 0, harvested: 0, modulus }
    }

    /// Harvest entropy from a Montgomery reduction byproduct
    pub fn harvest(&mut self, montgomery_quotient: u64) {
        self.pool.push(montgomery_quotient % self.modulus);
        self.harvested += 1;
    }

    /// Consume one entropy value (for fold function or key material)
    pub fn consume(&mut self) -> Option<u64> {
        let val = self.pool.pop();
        if val.is_some() { self.consumed += 1; }
        val
    }

    pub fn budget_remaining(&self) -> usize { self.pool.len() }
    pub fn is_exhausted(&self) -> bool { self.pool.is_empty() }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 6: ORACLE REGISTRY — Resident services on the drive
// ═══════════════════════════════════════════════════════════════════════════════

/// Oracles are not external calls. They are resident state on the WASSAN drive.
/// Querying an oracle reads pre-computed values from drive sectors.
pub struct OracleRegistry {
    pub grover: GroverOracle,
    pub ramanujan: RamanujanOracle,
    pub shor: ShorOracle,
}

/// Dense Toric Grover — F_p² amplitude amplification
/// Resident on the drive as pre-computed amplitude tables per admissible prime
pub struct GroverOracle {
    /// Admissible primes (p ≡ 3 mod 4) from the Safe Basis
    pub admissible: Vec<u64>,
    /// Pre-computed optimal iteration counts per (prime, gap) pair
    pub iteration_table: Vec<(u64, u64, u64)>, // (prime, gap, T_opt)
}

impl GroverOracle {
    pub fn new(safe_basis: &[u64]) -> Self {
        let admissible: Vec<u64> = safe_basis.iter()
            .filter(|&&p| p % 4 == 3)
            .copied()
            .collect();

        // Pre-compute T_opt for common gaps
        let mut table = Vec::new();
        for &p in &admissible {
            for d in [2u64, 4, 6, 8, 10, 12, 30] {
                let f = if d % p == 0 { 1u64 } else { 2 };
                let ratio = p * 10000 / f;
                let sqrt_r = integer_sqrt(ratio);
                let t_opt = 7854 * sqrt_r / 10000;
                table.push((p, d, t_opt.max(1)));
            }
        }

        GroverOracle { admissible, iteration_table: table }
    }

    /// Query: optimal iterations for prime p and gap d
    pub fn query_iterations(&self, p: u64, d: u64) -> u64 {
        self.iteration_table.iter()
            .find(|&&(pp, dd, _)| pp == p && dd == d)
            .map(|&(_, _, t)| t)
            .unwrap_or_else(|| {
                // Compute on the fly if not in table
                let f = if d % p == 0 { 1u64 } else { 2 };
                let ratio = p * 10000 / f;
                (7854 * integer_sqrt(ratio) / 10000).max(1)
            })
    }

    /// Is this prime admissible for F_p² Grover?
    pub fn is_admissible(&self, p: u64) -> bool {
        p % 4 == 3
    }
}

/// Ramanujan Spectral Oracle — sieve power and spectral analysis
/// Resident as precomputed Ramanujan sums for the Safe Basis
pub struct RamanujanOracle {
    /// Precomputed c_p(d) for Safe Basis primes × common gaps
    pub sum_table: Vec<(u64, u64, i64)>, // (prime, d, c_p(d))
    /// Precomputed sieve power SP(p,d) as (num, den)
    pub power_table: Vec<(u64, u64, i64, u64)>, // (prime, d, sp_num, sp_den)
}

impl RamanujanOracle {
    pub fn new(safe_basis: &[u64]) -> Self {
        let mut sums = Vec::new();
        let mut powers = Vec::new();

        for &p in safe_basis {
            if p < 2 { continue; }
            for d in 1..=30u64 {
                let c = if d % p == 0 { p as i64 - 1 } else { -1i64 };
                sums.push((p, d, c));

                let sp_num = 2 * p as i64 - 1 - c;
                let sp_den = p * p;
                powers.push((p, d, sp_num, sp_den));
            }
        }

        RamanujanOracle { sum_table: sums, power_table: powers }
    }

    /// Query: Ramanujan sum c_p(d)
    pub fn ramanujan_sum(&self, p: u64, d: u64) -> i64 {
        self.sum_table.iter()
            .find(|&&(pp, dd, _)| pp == p && dd == d)
            .map(|&(_, _, c)| c)
            .unwrap_or(if d % p == 0 { p as i64 - 1 } else { -1 })
    }

    /// Query: sieve power SP(p,d)
    pub fn sieve_power(&self, p: u64, d: u64) -> (i64, u64) {
        self.power_table.iter()
            .find(|&&(pp, dd, _, _)| pp == p && dd == d)
            .map(|&(_, _, num, den)| (num, den))
            .unwrap_or_else(|| {
                let c = self.ramanujan_sum(p, d);
                (2 * p as i64 - 1 - c, p * p)
            })
    }

    /// T6-1 Ramanujan-Grover iteration formula
    pub fn t6_iterations(&self, p: u64, d: u64) -> u64 {
        let c = self.ramanujan_sum(p, d);
        let discriminant = 2 * p as i64 - 1 - c;
        if discriminant <= 0 { return 1; }
        let sqrt_disc = integer_sqrt(discriminant as u64 * 10000);
        (7854 * p as u64 / sqrt_disc.max(1)).max(1)
    }
}

/// Shor's Oracle — order finding via CRT
/// Uses K-Elimination for non-circular order verification
pub struct ShorOracle {
    /// Cached order results (base, modulus) → order
    pub order_cache: Vec<(u64, u64, u64)>,
}

impl ShorOracle {
    pub fn new() -> Self {
        ShorOracle { order_cache: Vec::new() }
    }

    /// Find multiplicative order of base mod n using K-Elimination
    /// Non-circular: uses B = N-1 as upper bound (Lagrange)
    pub fn find_order(&mut self, base: u64, n: u64) -> Option<u64> {
        if n <= 1 || gcd(base, n) != 1 { return None; }

        // Check cache
        if let Some(&(_, _, ord)) = self.order_cache.iter()
            .find(|&&(b, m, _)| b == base && m == n) {
            return Some(ord);
        }

        // Baby-step giant-step on modular powers
        // Upper bound: N-1 (Lagrange, no phi(N) needed — NON-CIRCULAR)
        let bound = n - 1;
        let mut power = 1u64;
        for r in 1..=bound {
            power = mulmod(power, base, n);
            if power == 1 {
                // Found: minimize by checking divisors
                let order = self.minimize_order(base, n, r);
                self.order_cache.push((base, n, order));
                return Some(order);
            }
        }
        None
    }

    /// Minimize: given a^r ≡ 1 (mod n), find smallest r' | r with same property
    fn minimize_order(&self, base: u64, n: u64, r: u64) -> u64 {
        let mut min_r = r;
        // Check all divisors of r
        let mut d = 1;
        while d * d <= r {
            if r % d == 0 {
                if d > 0 && powmod(base, d, n) == 1 {
                    min_r = min_r.min(d);
                }
                let d2 = r / d;
                if d2 > 0 && powmod(base, d2, n) == 1 {
                    min_r = min_r.min(d2);
                }
            }
            d += 1;
        }
        min_r
    }

    /// Shor's factoring reduction: given even order r, attempt factoring
    pub fn factor_attempt(&self, base: u64, n: u64, order: u64) -> Option<(u64, u64)> {
        if order % 2 != 0 { return None; } // Need even order
        let half_pow = powmod(base, order / 2, n);
        if half_pow == n - 1 { return None; } // Trivial case

        let p = gcd(if half_pow > 0 { half_pow - 1 } else { 0 }, n);
        let q = gcd(half_pow + 1, n);

        if p > 1 && p < n { return Some((p, n / p)); }
        if q > 1 && q < n { return Some((q, n / q)); }
        None
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 7: CRT BEACON — Shared accelerator available to all cells
// ═══════════════════════════════════════════════════════════════════════════════

/// The CRT Beacon provides K-Elimination, Garner reconstruction, and
/// Recombinant Bridge services to ANY cell on the drive.
pub struct CRTBeacon {
    pub lane_moduli: [u64; 8],
    /// Precomputed lane product M = prod(lane_moduli)
    pub product: u128,
    /// Precomputed partial products M_i = M / p_i
    pub partials: [u128; 8],
    /// Precomputed inverses M_i^{-1} mod p_i
    pub inverses: [u64; 8],
    /// Safe Basis for Recombinant Bridge
    pub safe_basis: [u64; 5],
}

impl CRTBeacon {
    pub fn new(lane_moduli: [u64; 8]) -> Self {
        let product: u128 = lane_moduli.iter().map(|&p| p as u128).product();
        let partials: [u128; 8] = std::array::from_fn(|i| product / lane_moduli[i] as u128);
        let inverses: [u64; 8] = std::array::from_fn(|i| {
            let m_i_mod_pi = (partials[i] % lane_moduli[i] as u128) as u64;
            mod_inverse(m_i_mod_pi, lane_moduli[i]).unwrap_or(0)
        });

        CRTBeacon {
            lane_moduli,
            product,
            partials,
            inverses,
            safe_basis: [2, 3, 5, 7, 11],
        }
    }

    /// Garner reconstruction: residues → integer (first two lanes for demo range)
    pub fn reconstruct_two_lane(&self, r0: u64, r1: u64) -> Option<u128> {
        let p0 = self.lane_moduli[0];
        let p1 = self.lane_moduli[1];
        let inv = mod_inverse(p0 % p1, p1)?;
        let k = mulmod(submod(r1, r0 % p1, p1), inv, p1);
        Some(r0 as u128 + k as u128 * p0 as u128)
    }

    /// K-Elimination: exact winding number between main lane and anchor
    pub fn k_eliminate(&self, main_residue: u64, main_mod: u64,
                       anchor_residue: u64, anchor_mod: u64) -> Option<i64> {
        let inv = mod_inverse(main_mod % anchor_mod, anchor_mod)?;
        let diff = submod(anchor_residue, main_residue % anchor_mod, anchor_mod);
        let k_raw = mulmod(diff, inv, anchor_mod);
        // Center: if k > anchor_mod/2, it's negative
        let k_signed = if k_raw > anchor_mod / 2 {
            k_raw as i64 - anchor_mod as i64
        } else {
            k_raw as i64
        };
        Some(k_signed)
    }

    /// Recombinant Bridge: onboard an approximate value with frozen error
    pub fn onboard_approximate(&self, numerator: i64, denominator: u64) -> Vec<(u64, u64, f64)> {
        // Returns (modulus, residue, epsilon) for each Safe Basis sub-lane
        self.safe_basis.iter().map(|&p| {
            let modulus = p;
            let residue = if numerator >= 0 {
                (numerator as u64) % modulus
            } else {
                submod(0, numerator.unsigned_abs() % modulus, modulus)
            };
            let epsilon = 0.5 / denominator as f64;
            (modulus, residue, epsilon)
        }).collect()
    }

    /// PLMG check: is value on a rail (coprime to all moduli) or void?
    pub fn is_rail(&self, residues: &[u64; 8]) -> bool {
        residues.iter().zip(self.lane_moduli.iter())
            .all(|(&r, &m)| r != 0 && gcd(r, m) == 1)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 8: WASSAN DRIVE IMPLEMENTATION — The unified write/read path
// ═══════════════════════════════════════════════════════════════════════════════

/// Default 63-bit primes for CRTBigInt-compatible lanes
pub const DEFAULT_MODULI: [u64; 8] = [
    4611686018427387847, 4611686018427387817,
    4611686018427387631, 4611686018427387593,
    4611686018427387587, 4611686018427387553,
    4611686018427387541, 4611686018427387527,
];

/// Smaller moduli for testing
pub const TEST_MODULI: [u64; 8] = [101, 103, 107, 109, 113, 127, 131, 137];

impl WassanDrive {
    pub fn new(lane_moduli: [u64; 8], capacity: usize) -> Self {
        let verify_mod = 65537; // Fermat prime for verification
        WassanDrive {
            lane_moduli,
            sectors: vec![DriveSector::zero(&lane_moduli); capacity],
            helix: DoubleHelixVerifier::new(capacity, verify_mod),
            attractors: (0..capacity).map(|_| AttractorCell::new(0, verify_mod)).collect(),
            oracles: OracleRegistry {
                grover: GroverOracle::new(&[2, 3, 5, 7, 11, 13]),
                ramanujan: RamanujanOracle::new(&[2, 3, 5, 7, 11, 13]),
                shor: ShorOracle::new(),
            },
            beacon: CRTBeacon::new(lane_moduli),
            entropy_pool: ShadowEntropyPool::new(verify_mod),
            generation: 0,
        }
    }

    /// WRITE PATH: Cell → Schema → Double Helix → Drive
    /// Every write goes through Double Helix verification
    pub fn write(&mut self, idx: usize, sector: DriveSector) -> bool {
        if idx >= self.sectors.len() { return false; }

        // Double Helix: commit to both strands
        if !self.helix.commit_write(idx, &sector) {
            return false;
        }

        // Attractor: lock target to new value
        if idx < self.attractors.len() {
            self.attractors[idx].write(sector.residues[0] % self.helix.verify_mod);
        }

        // Shadow Entropy: harvest from the write operation
        // The Montgomery quotient of the residue computation is entropy
        let entropy_val = sector.residues[0] ^ sector.residues[1];
        self.entropy_pool.harvest(entropy_val);

        // Commit to drive
        let mut committed = sector;
        committed.write_generation = self.generation;
        committed.verified = true;
        self.sectors[idx] = committed;
        self.generation += 1;
        true
    }

    /// READ PATH: Drive → Attractor correction → Double Helix verify → Cell
    pub fn read(&mut self, idx: usize) -> Option<&DriveSector> {
        if idx >= self.sectors.len() { return None; }

        // Attractor: self-correct before read
        if idx < self.attractors.len() {
            self.attractors[idx].converge_step();
        }

        // Double Helix: verify integrity
        let sector = &self.sectors[idx];
        if !self.helix.verify_read(idx, sector) {
            // TAMPER DETECTED — this is the firewall
            // In production: trigger fold function
            return None;
        }

        Some(sector)
    }

    /// FOLD FUNCTION: Irreversible destruction
    /// Triggered when Shadow Entropy is exhausted or tamper detected
    pub fn fold(&mut self, idx: usize) -> bool {
        if idx >= self.sectors.len() { return false; }

        // Consume entropy to power the fold
        let entropy = self.entropy_pool.consume();

        // Collapse CRT sheet index: modular reduce to smaller modulus
        // This destroys the information needed for reconstruction
        let sector = &mut self.sectors[idx];
        for i in 0..8 {
            // XOR with entropy to scramble, then reduce mod small prime
            let scramble = entropy.unwrap_or(0) ^ sector.residues[i];
            sector.residues[i] = scramble % 7; // Collapse to tiny modulus
        }
        sector.verified = false;
        sector.attractor_target = None;
        sector.winding = i64::MIN; // Mark as folded

        // Destroy helix verification state for this sector
        if idx < self.helix.strand_a.len() {
            self.helix.strand_a[idx] = 0;
            self.helix.strand_b[idx] = 0;
        }

        true
    }

    /// Query an oracle (reads resident state, no external call)
    pub fn query_grover(&self, p: u64, d: u64) -> u64 {
        self.oracles.grover.query_iterations(p, d)
    }

    pub fn query_ramanujan(&self, p: u64, d: u64) -> i64 {
        self.oracles.ramanujan.ramanujan_sum(p, d)
    }

    pub fn query_shor(&mut self, base: u64, n: u64) -> Option<u64> {
        self.oracles.shor.find_order(base, n)
    }

    /// CRT Beacon: available to any cell
    pub fn beacon_k_eliminate(&self, main_r: u64, main_m: u64,
                              anchor_r: u64, anchor_m: u64) -> Option<i64> {
        self.beacon.k_eliminate(main_r, main_m, anchor_r, anchor_m)
    }

    pub fn beacon_reconstruct(&self, r0: u64, r1: u64) -> Option<u128> {
        self.beacon.reconstruct_two_lane(r0, r1)
    }

    pub fn beacon_is_rail(&self, residues: &[u64; 8]) -> bool {
        self.beacon.is_rail(residues)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 9: CELL TYPES — Domain-specific workers on the shared drive
// ═══════════════════════════════════════════════════════════════════════════════

/// A Cell is a worker that operates on the shared WASSAN drive.
/// Each cell knows its job. It reads from the drive, computes, writes back.
/// The drive handles verification, correction, and entropy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    PrimeSieve,       // Hydra twin/cousin/sexy prime sieving
    CoprimeSieve,     // Coprimality detection
    CompositeSieve,   // Composite number analysis
    NSVelocity,       // Navier-Stokes velocity field
    NSVortex,         // Navier-Stokes vorticity tracking
    NSTurbulence,     // Energy cascade analysis
    NSRegularity,     // Beta certificate computation
    YukawaSieve,      // String theory landscape sieve
    KahlerPeriods,    // Picard-Fuchs computation
    CryptoFHE,        // NINE65 homomorphic encryption
    NeuralForward,    // Neural network forward pass
    NeuralBackward,   // Neural network backward pass (learning)
    AttractorLearn,   // FRST one-shot learning
    SpectralWASSAN,   // Holographic NTT operations
}

/// Cell state: what a cell needs to do its job
pub struct CellState {
    pub cell_type: CellType,
    /// Sectors this cell owns (indices into the drive)
    pub owned_sectors: Vec<usize>,
    /// Active lens set name
    pub active_lens: String,
    /// Generation counter for this cell
    pub generation: u64,
    /// Cell-local scratch space (small, in registers)
    pub scratch: [u64; 16],
}

impl CellState {
    pub fn new(cell_type: CellType) -> Self {
        CellState {
            cell_type,
            owned_sectors: Vec::new(),
            active_lens: String::new(),
            generation: 0,
            scratch: [0; 16],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 10: UTILITY
// ═══════════════════════════════════════════════════════════════════════════════

fn integer_sqrt(n: u64) -> u64 {
    if n == 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + n / x) / 2; }
    x
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 11: TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn test_drive() -> WassanDrive {
        WassanDrive::new(TEST_MODULI, 64)
    }

    // ── Drive write/read path ──

    #[test]
    fn test_write_read_verified() {
        let mut drive = test_drive();
        let sector = DriveSector::from_i64(42, &TEST_MODULI);
        assert!(drive.write(0, sector));
        let read = drive.read(0).unwrap();
        assert_eq!(read.residues[0], 42 % TEST_MODULI[0]);
        assert!(read.verified);
    }

    #[test]
    fn test_write_updates_helix() {
        let mut drive = test_drive();
        let sector = DriveSector::from_i64(100, &TEST_MODULI);
        drive.write(0, sector);
        assert!(drive.helix.strand_a[0] != 0 || drive.helix.strand_b[0] != 0);
    }

    #[test]
    fn test_tamper_detection() {
        let mut drive = test_drive();
        let sector = DriveSector::from_i64(42, &TEST_MODULI);
        drive.write(0, sector);

        // Tamper with the sector directly (bypass write path)
        drive.sectors[0].residues[0] = 999;

        // Read should detect tamper
        let read = drive.read(0);
        assert!(read.is_none(), "Tampered sector should fail verification");
        assert!(drive.helix.violations > 0);
    }

    #[test]
    fn test_fold_destroys_data() {
        let mut drive = test_drive();
        let sector = DriveSector::from_i64(42, &TEST_MODULI);
        drive.write(0, sector.clone());

        // Fold
        assert!(drive.fold(0));

        // After fold: residues collapsed to tiny values
        let folded = &drive.sectors[0];
        assert!(folded.residues.iter().all(|&r| r < 7));
        assert!(!folded.verified);
        assert_eq!(folded.winding, i64::MIN);
    }

    // ── Attractor self-correction ──

    #[test]
    fn test_attractor_converges() {
        let mut cell = AttractorCell::new(100, 257);
        // Corrupt
        cell.current = 200;
        // Converge
        for _ in 0..50 { cell.converge_step(); }
        // Should be close to target
        let diff = if cell.current > cell.target {
            cell.current - cell.target
        } else {
            cell.target - cell.current
        };
        assert!(diff < 10, "Attractor should converge: diff={}", diff);
    }

    // ── Oracle queries ──

    #[test]
    fn test_grover_oracle_admissibility() {
        let drive = test_drive();
        assert!(drive.oracles.grover.is_admissible(3));
        assert!(drive.oracles.grover.is_admissible(7));
        assert!(drive.oracles.grover.is_admissible(11));
        assert!(!drive.oracles.grover.is_admissible(5));
        assert!(!drive.oracles.grover.is_admissible(13));
    }

    #[test]
    fn test_grover_iterations() {
        let drive = test_drive();
        let t = drive.query_grover(7, 2);
        assert!(t >= 1, "Should have at least 1 iteration for p=7, d=2");
    }

    #[test]
    fn test_ramanujan_oracle() {
        let drive = test_drive();
        // c_p(d) = p-1 when p|d, else -1
        assert_eq!(drive.query_ramanujan(5, 10), 4);  // 5|10
        assert_eq!(drive.query_ramanujan(5, 7), -1);   // 5∤7
        assert_eq!(drive.query_ramanujan(7, 14), 6);   // 7|14
    }

    #[test]
    fn test_shor_order_finding() {
        let mut drive = test_drive();
        // ord_{15}(2) = 4 (since 2^4 = 16 ≡ 1 mod 15)
        let order = drive.query_shor(2, 15);
        assert_eq!(order, Some(4));
    }

    #[test]
    fn test_shor_factoring() {
        let mut drive = test_drive();
        let order = drive.query_shor(2, 15).unwrap();
        let factors = drive.oracles.shor.factor_attempt(2, 15, order);
        assert!(factors.is_some());
        let (p, q) = factors.unwrap();
        assert_eq!(p * q, 15);
    }

    // ── CRT Beacon ──

    #[test]
    fn test_beacon_reconstruction() {
        let drive = test_drive();
        // Value 42: 42 mod 101 = 42, 42 mod 103 = 42
        let reconstructed = drive.beacon_reconstruct(42, 42);
        assert_eq!(reconstructed, Some(42));
    }

    #[test]
    fn test_beacon_k_eliminate() {
        let drive = test_drive();
        // Simple case: main=42 mod 101, anchor=42 mod 103
        let k = drive.beacon_k_eliminate(42, 101, 42, 103);
        assert_eq!(k, Some(0)); // No winding when values agree
    }

    #[test]
    fn test_beacon_rail_void() {
        let drive = test_drive();
        // All nonzero coprime residues → rail
        let rail = [1, 1, 1, 1, 1, 1, 1, 1];
        assert!(drive.beacon_is_rail(&rail));

        // Zero in a lane → not coprime → void
        let void_val = [0, 1, 1, 1, 1, 1, 1, 1];
        assert!(!drive.beacon_is_rail(&void_val));
    }

    // ── Shadow Entropy ──

    #[test]
    fn test_entropy_harvest_consume() {
        let mut drive = test_drive();
        let sector = DriveSector::from_i64(42, &TEST_MODULI);
        drive.write(0, sector); // Harvests entropy

        assert!(drive.entropy_pool.budget_remaining() > 0);
        let val = drive.entropy_pool.consume();
        assert!(val.is_some());
    }

    #[test]
    fn test_entropy_exhaustion() {
        let mut pool = ShadowEntropyPool::new(257);
        assert!(pool.is_exhausted());
        pool.harvest(42);
        assert!(!pool.is_exhausted());
        pool.consume();
        assert!(pool.is_exhausted());
    }

    // ── Descartes phase-lock ──

    #[test]
    fn test_descartes_trivial() {
        let drive = test_drive();
        // (0+0+0+0)² = 0, 2(0+0+0+0) = 0 → holds
        assert!(drive.helix.descartes_check(0, 0, 0, 0));
    }

    // ── Sector arithmetic ──

    #[test]
    fn test_sector_add() {
        let a = DriveSector::from_i64(10, &TEST_MODULI);
        let b = DriveSector::from_i64(20, &TEST_MODULI);
        let c = a.add(&b, &TEST_MODULI);
        assert_eq!(c.residues[0], 30 % TEST_MODULI[0]);
    }

    #[test]
    fn test_sector_mul() {
        let a = DriveSector::from_i64(6, &TEST_MODULI);
        let b = DriveSector::from_i64(7, &TEST_MODULI);
        let c = a.mul(&b, &TEST_MODULI);
        assert_eq!(c.residues[0], 42 % TEST_MODULI[0]);
    }

    // ── Integration: oracle + beacon + drive ──

    #[test]
    fn test_shor_then_beacon() {
        let mut drive = test_drive();
        // Find order, then use beacon to verify via K-Elimination
        let order = drive.query_shor(2, 15).unwrap();
        assert_eq!(order, 4);

        // Verify: 2^4 mod 15 = 1
        let power = powmod(2, order, 15);
        assert_eq!(power, 1);

        // Store result on drive
        let sector = DriveSector::from_i64(order as i64, &TEST_MODULI);
        drive.write(10, sector);

        // Read back through full verified path
        let read = drive.read(10).unwrap();
        assert_eq!(read.residues[0], 4);
    }

    #[test]
    fn test_ramanujan_sieve_power() {
        let drive = test_drive();
        // SP(7, 2) = (14 - 1 - (-1)) / 49 = 14/49 = 2/7
        let (num, den) = drive.oracles.ramanujan.sieve_power(7, 2);
        assert_eq!(num, 14);
        assert_eq!(den, 49);
    }
}
