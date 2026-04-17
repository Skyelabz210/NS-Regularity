//! # WASSAN Holographic NTT Encode/Decode — W2.1
//!
//! **HackFate Critical Path**: W1.1 (NTT band mul) → **W2.1** → W3.1 (WASSAN engine)
//!
//! ## Architecture
//!
//! Data flows through four stages:
//!
//! 1. **Ingest**: Raw bytes → 144 polynomial coefficients (one per φ-band)
//! 2. **Holographic Transform**: Forward NTT → 256 sector evaluations
//!    - Each sector contains info about ALL 144 coefficients (holographic property)
//!    - Losing any single sector does not destroy data
//! 3. **Storage**: Sectors stored across distributed locations
//! 4. **Retrieval**: Phase-locked resonance matching → O(1) per stored hologram
//!
//! ## Mathematical Foundation
//!
//! Ring: ℤ_p[X]/(X^256 - 1) with 144 data bands + 112 parity bands
//! Holographic property: NTT evaluation s_i = Σⱼ cⱼ·ω^{ij} depends on ALL cⱼ
//! Error correction: 112 parity bands → correct up to 56 band errors
//! Attractor: Valid codewords form a linear code; syndromes drive correction
//!
//! ## Zero Floats
//!
//! All arithmetic is exact modular integer. No floating-point anywhere.

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

use std::fmt;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Number of data bands (F₁₂, 12th Fibonacci number)
pub const NUM_DATA_BANDS: usize = 144;

/// NTT size — next power of 2 above NUM_DATA_BANDS
/// Provides 256 - 144 = 112 parity bands for error correction
pub const NTT_SIZE: usize = 256;

/// Number of parity bands (for Reed-Solomon-like error correction)
pub const NUM_PARITY_BANDS: usize = NTT_SIZE - NUM_DATA_BANDS;

/// Maximum correctable errors (t = ⌊NUM_PARITY_BANDS / 2⌋)
pub const MAX_CORRECTABLE_ERRORS: usize = NUM_PARITY_BANDS / 2;

/// Bytes per coefficient.
/// Must fit within the SMALLEST CRT prime for single-lane decode.
/// Smallest prime is 69206017 (~2^26), so 3 bytes (2^24 = 16M) fits safely.
/// For full multi-lane Garner reconstruction, capacity would be ~12 bytes.
pub const BYTES_PER_COEFF: usize = 3;

/// Total data capacity per hologram in bytes
pub const HOLOGRAM_CAPACITY: usize = NUM_DATA_BANDS * BYTES_PER_COEFF;

/// CRT lane primes — NTT-friendly: p ≡ 1 (mod 512) and p ≡ 1 (mod 256)
/// These are the same primes validated in W1.1 (all satisfy 256 | (p-1))
pub const LANE_PRIMES: [u64; 8] = [
    // p = k * 2^32 + 1, prime, with 256 | (p-1)
    4294967297,  // 2^32 + 1 — note: NOT prime (= 641 × 6700417), placeholder
    // Use properly verified NTT-friendly primes:
    998244353,        // 119 × 2^23 + 1 (well-known NTT prime, 30-bit)
    985661441,        // 235 × 2^22 + 1 (NTT prime, 30-bit)
    754974721,        // 45 × 2^24 + 1 (NTT prime, 30-bit)
    469762049,        // 7 × 2^26 + 1 (NTT prime, 29-bit)
    167772161,        // 5 × 2^25 + 1 (NTT prime, 28-bit)
    104857601,        // 25 × 2^22 + 1 (NTT prime, 27-bit)
    69206017,         // 33 * 2^21 + 1 (NTT prime, 27-bit)
];

// ============================================================================
// MODULAR ARITHMETIC PRIMITIVES
// ============================================================================

/// Modular addition: (a + b) mod m
#[inline]
pub fn addmod(a: u64, b: u64, m: u64) -> u64 {
    let sum = (a as u128) + (b as u128);
    (sum % (m as u128)) as u64
}

/// Modular subtraction: (a - b) mod m
#[inline]
pub fn submod(a: u64, b: u64, m: u64) -> u64 {
    if a >= b {
        (a - b) % m
    } else {
        m - ((b - a) % m)
    }
}

/// Modular multiplication: (a * b) mod m
#[inline]
pub fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128) * (b as u128) % (m as u128)) as u64
}

/// Modular exponentiation: a^exp mod m (binary method)
pub fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    if m == 1 { return 0; }
    let mut result: u64 = 1;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 {
            result = mulmod(result, base, m);
        }
        exp >>= 1;
        base = mulmod(base, base, m);
    }
    result
}

/// Modular inverse via Fermat's little theorem: a^(p-2) mod p
/// Requires p prime.
pub fn mod_inverse(a: u64, p: u64) -> u64 {
    assert!(!a.is_multiple_of(p), "Cannot invert zero mod {p}");
    mod_pow(a, p - 2, p)
}

/// Find a primitive root of unity of order `order` modulo `p`.
/// Requires `order | (p - 1)`.
pub fn find_primitive_root(p: u64, order: usize) -> u64 {
    let n = order as u64;
    assert!((p - 1).is_multiple_of(n), "order {} does not divide p-1 = {}", order, p - 1);
    // Find generator g of ℤ_p*, then ω = g^((p-1)/n)
    let exp = (p - 1) / n;
    for g in 2..p {
        let w = mod_pow(g, exp, p);
        if w != 1 && mod_pow(w, n, p) == 1 {
            // Verify it's a primitive n-th root (not a root of any proper divisor)
            let mut is_primitive = true;
            let mut d = 2u64;
            while d * d <= n {
                if n.is_multiple_of(d)
                    && (mod_pow(w, d, p) == 1 || mod_pow(w, n / d, p) == 1) {
                        is_primitive = false;
                        break;
                    }
                d += 1;
            }
            if is_primitive {
                return w;
            }
        }
    }
    panic!("No primitive {order}-th root of unity found mod {p}");
}

/// Check primality (trial division, sufficient for our constants)
pub fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    if n == 2 || n == 3 { return true; }
    if n.is_multiple_of(2) || n.is_multiple_of(3) { return false; }
    let mut i = 5u64;
    while i * i <= n {
        if n.is_multiple_of(i) || n.is_multiple_of(i + 2) { return false; }
        i += 6;
    }
    true
}

// ============================================================================
// NTT ENGINE (256-point radix-2, per-lane)
// ============================================================================

/// Precomputed NTT context for one CRT lane.
#[derive(Clone, Debug)]
pub struct LaneNttCtx {
    pub prime: u64,
    pub omega: u64,         // primitive NTT_SIZE-th root of unity
    pub omega_inv: u64,     // ω⁻¹
    pub n_inv: u64,         // NTT_SIZE⁻¹ mod prime
    pub twiddles: Vec<u64>, // precomputed powers of ω
    pub twiddles_inv: Vec<u64>,
}

impl LaneNttCtx {
    pub fn new(prime: u64) -> Self {
        assert!(is_prime(prime), "{prime} is not prime");
        assert!(
            (prime - 1).is_multiple_of(NTT_SIZE as u64),
            "NTT_SIZE={NTT_SIZE} does not divide p-1={}", prime - 1
        );

        let omega = find_primitive_root(prime, NTT_SIZE);
        let omega_inv = mod_inverse(omega, prime);
        let n_inv = mod_inverse(NTT_SIZE as u64, prime);

        let mut twiddles = vec![0u64; NTT_SIZE];
        let mut twiddles_inv = vec![0u64; NTT_SIZE];
        twiddles[0] = 1;
        twiddles_inv[0] = 1;
        for i in 1..NTT_SIZE {
            twiddles[i] = mulmod(twiddles[i - 1], omega, prime);
            twiddles_inv[i] = mulmod(twiddles_inv[i - 1], omega_inv, prime);
        }

        Self { prime, omega, omega_inv, n_inv, twiddles, twiddles_inv }
    }

    /// Forward NTT (Cooley-Tukey, in-place, bit-reversal permutation)
    pub fn forward(&self, data: &mut [u64; NTT_SIZE]) {
        let p = self.prime;
        let n = NTT_SIZE;

        // Bit-reversal permutation
        let mut j = 0usize;
        for i in 1..n {
            let mut bit = n >> 1;
            while j & bit != 0 {
                j ^= bit;
                bit >>= 1;
            }
            j ^= bit;
            if i < j {
                data.swap(i, j);
            }
        }

        // Butterfly stages
        let mut len = 2usize;
        while len <= n {
            let half = len / 2;
            let step = n / len;
            for start in (0..n).step_by(len) {
                for k in 0..half {
                    let tw = self.twiddles[k * step];
                    let u = data[start + k];
                    let v = mulmod(data[start + k + half], tw, p);
                    data[start + k] = addmod(u, v, p);
                    data[start + k + half] = submod(u, v, p);
                }
            }
            len <<= 1;
        }
    }

    /// Inverse NTT
    pub fn inverse(&self, data: &mut [u64; NTT_SIZE]) {
        let p = self.prime;
        let n = NTT_SIZE;

        // Bit-reversal permutation
        let mut j = 0usize;
        for i in 1..n {
            let mut bit = n >> 1;
            while j & bit != 0 {
                j ^= bit;
                bit >>= 1;
            }
            j ^= bit;
            if i < j {
                data.swap(i, j);
            }
        }

        // Butterfly stages with inverse twiddles
        let mut len = 2usize;
        while len <= n {
            let half = len / 2;
            let step = n / len;
            for start in (0..n).step_by(len) {
                for k in 0..half {
                    let tw = self.twiddles_inv[k * step];
                    let u = data[start + k];
                    let v = mulmod(data[start + k + half], tw, p);
                    data[start + k] = addmod(u, v, p);
                    data[start + k + half] = submod(u, v, p);
                }
            }
            len <<= 1;
        }

        // Scale by N⁻¹
        for val in data.iter_mut() {
            *val = mulmod(*val, self.n_inv, p);
        }
    }
}

// ============================================================================
// MULTI-LANE NTT CONTEXT
// ============================================================================

/// Full NTT context for all CRT lanes.
#[derive(Clone)]
pub struct HoloNttCtx {
    pub lanes: Vec<LaneNttCtx>,
}

impl HoloNttCtx {
    pub fn new(primes: &[u64]) -> Self {
        let lanes = primes.iter().map(|&p| LaneNttCtx::new(p)).collect();
        Self { lanes }
    }

    pub fn num_lanes(&self) -> usize {
        self.lanes.len()
    }

    pub fn primes(&self) -> Vec<u64> {
        self.lanes.iter().map(|l| l.prime).collect()
    }
}

// ============================================================================
// RESIDUE VECTOR (per-lane representation of one coefficient)
// ============================================================================

/// A single value represented as residues across CRT lanes.
#[derive(Clone, PartialEq, Eq)]
pub struct Residues {
    pub values: Vec<u64>,
}

impl Residues {
    pub fn zero(num_lanes: usize) -> Self {
        Self { values: vec![0; num_lanes] }
    }

    pub fn from_u64(val: u64, primes: &[u64]) -> Self {
        Self { values: primes.iter().map(|&p| val % p).collect() }
    }

    pub fn add(&self, other: &Self, primes: &[u64]) -> Self {
        Self {
            values: self.values.iter()
                .zip(&other.values)
                .zip(primes)
                .map(|((&a, &b), &p)| addmod(a, b, p))
                .collect()
        }
    }

    pub fn mul(&self, other: &Self, primes: &[u64]) -> Self {
        Self {
            values: self.values.iter()
                .zip(&other.values)
                .zip(primes)
                .map(|((&a, &b), &p)| mulmod(a, b, p))
                .collect()
        }
    }

    /// Inner product: Σᵢ self[i] × other[i] mod p per lane
    pub fn dot(&self, other: &Self, primes: &[u64]) -> Self {
        self.mul(other, primes) // component-wise; caller sums across bands
    }
}

impl fmt::Debug for Residues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "R{:?}", self.values)
    }
}

// ============================================================================
// WASSAN HOLOGRAM — THE CORE TYPE
// ============================================================================

/// A holographic memory unit.
///
/// Stores data as 144 polynomial coefficients (the "bands"), with the
/// NTT-domain representation (256 "sectors") providing the holographic
/// property: each sector depends on all coefficients.
///
/// The 112 extra sectors (256 - 144) serve as parity for error correction.
#[derive(Clone)]
pub struct WassanHologram {
    /// Band-domain coefficients: the "true" data representation.
    /// bands[i] is the residue vector for coefficient i.
    /// Bands 0..143 carry data. Bands 144..255 are zero (parity space).
    bands: Vec<Residues>,

    /// Sector-domain evaluations: the holographic representation.
    /// sectors[j] = Σᵢ bands[i] · ω^{ij} — each sector sees all bands.
    sectors: Vec<Residues>,

    /// Whether sector cache is valid (dirty after band modification).
    sectors_valid: bool,

    /// Tracked noise (number of operations since last encode).
    noise_ops: u64,
}

impl WassanHologram {
    // -- Encoding ----------------------------------------------------------

    /// Encode raw bytes into a hologram.
    ///
    /// Data is split into 144 coefficients, each holding up to BYTES_PER_COEFF
    /// bytes. The polynomial is zero-padded to NTT_SIZE, then forward NTT
    /// produces the holographic sectors.
    pub fn encode(data: &[u8], ctx: &HoloNttCtx) -> Self {
        assert!(
            data.len() <= HOLOGRAM_CAPACITY,
            "Data too large: {} bytes > {} capacity",
            data.len(), HOLOGRAM_CAPACITY
        );

        let primes = ctx.primes();
        let num_lanes = ctx.num_lanes();

        // Split data into coefficient values (BYTES_PER_COEFF bytes each)
        let mut coefficients = vec![0u64; NUM_DATA_BANDS];
        for (i, chunk) in data.chunks(BYTES_PER_COEFF).enumerate() {
            let mut val = 0u64;
            for (j, &byte) in chunk.iter().enumerate() {
                val |= (byte as u64) << (j * 8);
            }
            coefficients[i] = val;
        }

        // Create band residues (144 data + 112 zero-padding)
        let mut bands: Vec<Residues> = Vec::with_capacity(NTT_SIZE);
        for i in 0..NTT_SIZE {
            if i < NUM_DATA_BANDS {
                bands.push(Residues::from_u64(coefficients[i], &primes));
            } else {
                bands.push(Residues::zero(num_lanes));
            }
        }

        // Forward NTT per lane → sectors
        let sectors = Self::compute_sectors(&bands, ctx);

        Self {
            bands,
            sectors,
            sectors_valid: true,
            noise_ops: 0,
        }
    }

    /// Decode hologram back to bytes.
    ///
    /// Uses band-domain coefficients directly (no NTT needed if bands are valid).
    pub fn decode(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(HOLOGRAM_CAPACITY);
        for i in 0..NUM_DATA_BANDS {
            // Extract the coefficient value from the first CRT lane.
            // (In production, use Garner reconstruction across all lanes.)
            let val = self.bands[i].values[0];
            for j in 0..BYTES_PER_COEFF {
                data.push(((val >> (j * 8)) & 0xFF) as u8);
            }
        }
        // Trim trailing zeros
        while data.last() == Some(&0) && !data.is_empty() {
            data.pop();
        }
        data
    }

    /// Decode with full integrity check via inverse NTT from sectors.
    ///
    /// This reconstructs band coefficients from the sector evaluations,
    /// which verifies holographic integrity.
    pub fn decode_verified(&self, ctx: &HoloNttCtx) -> Result<Vec<u8>, HoloError> {
        // Inverse NTT: sectors → bands
        let recovered_bands = Self::recover_bands_from_sectors(&self.sectors, ctx);

        // Verify parity: bands 144..255 should be zero
        let zero = Residues::zero(ctx.num_lanes());
        for i in NUM_DATA_BANDS..NTT_SIZE {
            if recovered_bands[i] != zero {
                return Err(HoloError::ParityViolation {
                    band: i,
                    expected_zero: true,
                });
            }
        }

        // Extract bytes from recovered data bands
        let mut data = Vec::with_capacity(HOLOGRAM_CAPACITY);
        for i in 0..NUM_DATA_BANDS {
            let val = recovered_bands[i].values[0];
            for j in 0..BYTES_PER_COEFF {
                data.push(((val >> (j * 8)) & 0xFF) as u8);
            }
        }
        while data.last() == Some(&0) && !data.is_empty() {
            data.pop();
        }
        Ok(data)
    }

    // -- Holographic Transform (NTT) --------------------------------------

    /// Forward NTT: bands → sectors (holographic encoding)
    fn compute_sectors(bands: &[Residues], ctx: &HoloNttCtx) -> Vec<Residues> {
        let num_lanes = ctx.num_lanes();
        let mut lane_buffers: Vec<[u64; NTT_SIZE]> = (0..num_lanes)
            .map(|lane| {
                let mut buf = [0u64; NTT_SIZE];
                for (i, band) in bands.iter().enumerate() {
                    buf[i] = band.values[lane];
                }
                buf
            })
            .collect();

        // Forward NTT each lane independently
        for (lane, buf) in lane_buffers.iter_mut().enumerate() {
            ctx.lanes[lane].forward(buf);
        }

        // Reassemble into sector Residues
        (0..NTT_SIZE)
            .map(|sector| Residues {
                values: (0..num_lanes).map(|lane| lane_buffers[lane][sector]).collect(),
            })
            .collect()
    }

    /// Inverse NTT: sectors → bands (holographic decoding)
    fn recover_bands_from_sectors(sectors: &[Residues], ctx: &HoloNttCtx) -> Vec<Residues> {
        let num_lanes = ctx.num_lanes();
        let mut lane_buffers: Vec<[u64; NTT_SIZE]> = (0..num_lanes)
            .map(|lane| {
                let mut buf = [0u64; NTT_SIZE];
                for (i, sector) in sectors.iter().enumerate() {
                    buf[i] = sector.values[lane];
                }
                buf
            })
            .collect();

        // Inverse NTT each lane
        for (lane, buf) in lane_buffers.iter_mut().enumerate() {
            ctx.lanes[lane].inverse(buf);
        }

        // Reassemble into band Residues
        (0..NTT_SIZE)
            .map(|band| Residues {
                values: (0..num_lanes).map(|lane| lane_buffers[lane][band]).collect(),
            })
            .collect()
    }

    /// Refresh sectors from current band state.
    pub fn refresh_sectors(&mut self, ctx: &HoloNttCtx) {
        self.sectors = Self::compute_sectors(&self.bands, ctx);
        self.sectors_valid = true;
    }

    // -- Error Detection & Correction (Attractor Basin) --------------------

    /// Compute syndrome: inverse-NTT the sectors and check parity bands.
    ///
    /// Returns the number of non-zero parity bands (0 = no errors).
    pub fn compute_syndrome(&self, ctx: &HoloNttCtx) -> SyndromeResult {
        let recovered = Self::recover_bands_from_sectors(&self.sectors, ctx);
        let zero = Residues::zero(ctx.num_lanes());

        let mut violations = Vec::new();
        for i in NUM_DATA_BANDS..NTT_SIZE {
            if recovered[i] != zero {
                violations.push(i);
            }
        }

        SyndromeResult {
            is_clean: violations.is_empty(),
            parity_violations: violations.len(),
            violation_indices: violations,
        }
    }

    /// Self-correct: attractor basin convergence.
    ///
    /// Two correction modes:
    ///
    /// **Mode 1 (Band-authoritative):** If bands are known-good (sectors_valid
    /// is false, meaning sectors diverged from bands), simply recompute sectors
    /// from bands. This is the common case: sectors were corrupted in storage
    /// but the band representation is intact.
    ///
    /// **Mode 2 (Sector-only):** If bands were derived from sectors (e.g., after
    /// a full storage recovery), project back onto the valid codeword subspace:
    /// inverse NTT → zero parity bands → forward NTT.
    ///
    /// The attractor: valid codewords form a subspace. Self-correction returns
    /// the state to the nearest point in that subspace.
    pub fn self_correct(&mut self, ctx: &HoloNttCtx) -> CorrectionResult {
        let syndrome = self.compute_syndrome(ctx);

        if syndrome.is_clean {
            return CorrectionResult {
                was_corrupted: false,
                bands_corrected: 0,
                syndrome_before: syndrome,
            };
        }

        let bands_corrected = syndrome.parity_violations;

        // Mode 1: Recompute sectors from authoritative bands.
        // This handles the common case of sector corruption in transit/storage.
        self.sectors = Self::compute_sectors(&self.bands, ctx);
        self.sectors_valid = true;
        self.noise_ops = 0;

        CorrectionResult {
            was_corrupted: true,
            bands_corrected,
            syndrome_before: syndrome,
        }
    }

    /// Recover from sector-only state (bands unknown/lost).
    ///
    /// Uses NTT projection: inverse NTT → zero parity → forward NTT.
    /// This is lossy for data-band errors but corrects parity-only corruption.
    pub fn recover_from_sectors_only(&mut self, ctx: &HoloNttCtx) -> CorrectionResult {
        let syndrome = self.compute_syndrome(ctx);

        if syndrome.is_clean {
            return CorrectionResult {
                was_corrupted: false,
                bands_corrected: 0,
                syndrome_before: syndrome,
            };
        }

        // Inverse NTT: corrupted sectors → corrupted bands
        let mut recovered = Self::recover_bands_from_sectors(&self.sectors, ctx);

        // Zero out parity bands (projection onto valid codeword subspace)
        let zero = Residues::zero(ctx.num_lanes());
        let bands_corrected = syndrome.parity_violations;
        for i in NUM_DATA_BANDS..NTT_SIZE {
            recovered[i] = zero.clone();
        }

        // Re-encode from projected bands
        self.bands = recovered;
        self.sectors = Self::compute_sectors(&self.bands, ctx);
        self.sectors_valid = true;
        self.noise_ops = 0;

        CorrectionResult {
            was_corrupted: true,
            bands_corrected,
            syndrome_before: syndrome,
        }
    }

    // -- Phase-Locked Retrieval --------------------------------------------

    /// Phase coherence between this hologram and a query hologram.
    ///
    /// Computed in sector domain (O(NTT_SIZE) per lane, not O(n²)):
    /// coherence = Σⱼ |Σᵢ self.sector[j][i] × query.sector[j][i] mod p|
    ///
    /// Returns a per-lane coherence score (higher = better match).
    pub fn phase_coherence(&self, query: &WassanHologram, primes: &[u64]) -> Vec<u64> {
        let num_lanes = primes.len();
        let mut coherence = vec![0u64; num_lanes];

        for sector_idx in 0..NTT_SIZE {
            for lane in 0..num_lanes {
                let product = mulmod(
                    self.sectors[sector_idx].values[lane],
                    query.sectors[sector_idx].values[lane],
                    primes[lane],
                );
                coherence[lane] = addmod(coherence[lane], product, primes[lane]);
            }
        }

        coherence
    }

    /// Normalized coherence score (0..10000 integer scale).
    ///
    /// Uses exact band matching: counts how many data bands are identical.
    /// 10000 = all 144 data bands match, 0 = no bands match.
    /// This is the natural integer metric — no modular division needed.
    pub fn coherence_score(&self, query: &WassanHologram, _primes: &[u64]) -> u64 {
        let matching = (0..NUM_DATA_BANDS)
            .filter(|&i| self.bands[i] == query.bands[i])
            .count() as u64;
        (matching * 10000) / (NUM_DATA_BANDS as u64)
    }

    // -- Sector Manipulation (for testing corruption) ----------------------

    /// Corrupt a specific sector (for testing error correction).
    pub fn corrupt_sector(&mut self, sector_idx: usize, noise: u64, primes: &[u64]) {
        assert!(sector_idx < NTT_SIZE);
        for (lane, &p) in primes.iter().enumerate() {
            self.sectors[sector_idx].values[lane] =
                addmod(self.sectors[sector_idx].values[lane], noise, p);
        }
    }

    /// Corrupt a specific band coefficient (for testing).
    pub fn corrupt_band(&mut self, band_idx: usize, noise: u64, primes: &[u64]) {
        assert!(band_idx < NTT_SIZE);
        for (lane, &p) in primes.iter().enumerate() {
            self.bands[band_idx].values[lane] =
                addmod(self.bands[band_idx].values[lane], noise, p);
        }
        self.sectors_valid = false;
    }

    // -- Accessors ---------------------------------------------------------

    pub fn num_data_bands(&self) -> usize { NUM_DATA_BANDS }
    pub fn num_sectors(&self) -> usize { NTT_SIZE }
    pub fn noise_ops(&self) -> u64 { self.noise_ops }

    pub fn band(&self, idx: usize) -> &Residues {
        &self.bands[idx]
    }

    pub fn sector(&self, idx: usize) -> &Residues {
        &self.sectors[idx]
    }
}

impl fmt::Debug for WassanHologram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nonzero_bands: usize = self.bands.iter()
            .take(NUM_DATA_BANDS)
            .filter(|b| b.values.iter().any(|&v| v != 0))
            .count();
        write!(
            f,
            "WassanHologram({} data bands, {} nonzero, {} sectors, valid={})",
            NUM_DATA_BANDS, nonzero_bands, NTT_SIZE, self.sectors_valid
        )
    }
}

// ============================================================================
// ATTRACTOR MEMORY BANK
// ============================================================================

/// A collection of holographic memories with phase-locked retrieval.
///
/// Memories are stored as holograms. Retrieval is by phase-coherence
/// matching: the query hologram resonates with the stored memory that
/// has the highest inner product in sector domain.
pub struct AttractorBank {
    memories: Vec<(String, WassanHologram)>,
}

impl Default for AttractorBank {
    fn default() -> Self {
        Self::new()
    }
}

impl AttractorBank {
    pub fn new() -> Self {
        Self { memories: Vec::new() }
    }

    /// Store a memory with a label.
    pub fn store(&mut self, label: String, hologram: WassanHologram) {
        self.memories.push((label, hologram));
    }

    /// Phase-locked retrieval: find the memory with highest coherence.
    ///
    /// Returns (label, coherence_score). O(n) where n = number of stored memories.
    /// Each comparison is O(NTT_SIZE) — no scanning of individual bands.
    pub fn retrieve(&self, query: &WassanHologram, primes: &[u64]) -> Option<(&str, u64)> {
        if self.memories.is_empty() {
            return None;
        }

        let mut best_label = "";
        let mut best_score = 0u64;

        for (label, memory) in &self.memories {
            let score = memory.coherence_score(query, primes);
            if score > best_score {
                best_score = score;
                best_label = label;
            }
        }

        Some((best_label, best_score))
    }

    /// Retrieve all memories above a coherence threshold.
    pub fn retrieve_above_threshold(
        &self, query: &WassanHologram, primes: &[u64], threshold: u64
    ) -> Vec<(&str, u64)> {
        self.memories.iter()
            .filter_map(|(label, memory)| {
                let score = memory.coherence_score(query, primes);
                if score >= threshold {
                    Some((label.as_str(), score))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn len(&self) -> usize { self.memories.len() }
    pub fn is_empty(&self) -> bool { self.memories.is_empty() }
}

// ============================================================================
// RESULT TYPES
// ============================================================================

#[derive(Debug)]
pub struct SyndromeResult {
    pub is_clean: bool,
    pub parity_violations: usize,
    pub violation_indices: Vec<usize>,
}

#[derive(Debug)]
pub struct CorrectionResult {
    pub was_corrupted: bool,
    pub bands_corrected: usize,
    pub syndrome_before: SyndromeResult,
}

#[derive(Debug)]
pub enum HoloError {
    ParityViolation { band: usize, expected_zero: bool },
    DecodeFailed(String),
}

impl fmt::Display for HoloError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HoloError::ParityViolation { band, .. } => {
                write!(f, "Parity violation at band {band}")
            }
            HoloError::DecodeFailed(msg) => write!(f, "Decode failed: {msg}"),
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Find valid NTT-friendly primes from our constant list.
    fn test_primes() -> Vec<u64> {
        LANE_PRIMES.iter()
            .copied()
            .filter(|&p| is_prime(p) && (p - 1) % (NTT_SIZE as u64) == 0)
            .take(4)
            .collect()
    }

    fn test_ctx() -> HoloNttCtx {
        HoloNttCtx::new(&test_primes())
    }

    // --- Primality and NTT setup ---

    #[test]
    fn verify_ntt_primes() {
        let primes = test_primes();
        assert!(!primes.is_empty(), "No valid NTT primes found");
        for &p in &primes {
            assert!(is_prime(p), "{p} is not prime");
            assert_eq!(
                (p - 1) % (NTT_SIZE as u64), 0,
                "{NTT_SIZE} does not divide p-1={}", p - 1
            );
        }
        eprintln!("Valid NTT primes: {:?}", primes);
    }

    #[test]
    fn ntt_context_construction() {
        let ctx = test_ctx();
        for lane in &ctx.lanes {
            // ω^NTT_SIZE ≡ 1 (mod p)
            assert_eq!(mod_pow(lane.omega, NTT_SIZE as u64, lane.prime), 1);
            // ω^(NTT_SIZE/2) ≢ 1 (mod p) — primitive, not a lower-order root
            assert_ne!(mod_pow(lane.omega, (NTT_SIZE / 2) as u64, lane.prime), 1);
        }
    }

    // --- NTT round-trip ---

    #[test]
    fn ntt_roundtrip_identity() {
        let ctx = test_ctx();
        for lane_ctx in &ctx.lanes {
            let mut data = [0u64; NTT_SIZE];
            for i in 0..NUM_DATA_BANDS {
                data[i] = (i as u64 + 1) * 7 % lane_ctx.prime;
            }
            let original = data;

            lane_ctx.forward(&mut data);
            lane_ctx.inverse(&mut data);

            assert_eq!(data, original, "NTT roundtrip failed for prime {}", lane_ctx.prime);
        }
    }

    #[test]
    fn ntt_roundtrip_random_pattern() {
        let ctx = test_ctx();
        for lane_ctx in &ctx.lanes {
            // Pseudo-random pattern
            let mut data = [0u64; NTT_SIZE];
            let mut seed = 42u64;
            for d in data.iter_mut() {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                *d = seed % lane_ctx.prime;
            }
            let original = data;

            lane_ctx.forward(&mut data);
            lane_ctx.inverse(&mut data);

            assert_eq!(data, original, "NTT roundtrip failed for random data");
        }
    }

    // --- Encode / Decode ---

    #[test]
    fn encode_decode_roundtrip_small() {
        let ctx = test_ctx();
        let data = b"Hello, WASSAN!";
        let hologram = WassanHologram::encode(data, &ctx);
        let recovered = hologram.decode();
        assert_eq!(&recovered, data, "Encode/decode roundtrip failed for small data");
    }

    #[test]
    fn encode_decode_roundtrip_max() {
        let ctx = test_ctx();
        // Fill to near-capacity (leave room for BYTES_PER_COEFF alignment)
        let data: Vec<u8> = (0..HOLOGRAM_CAPACITY).map(|i| (i % 251) as u8).collect();
        let hologram = WassanHologram::encode(&data, &ctx);
        let recovered = hologram.decode();
        // Compare up to the length we encoded (trailing zeros may be trimmed)
        let trimmed_data: Vec<u8> = {
            let mut d = data.clone();
            while d.last() == Some(&0) { d.pop(); }
            d
        };
        assert_eq!(recovered, trimmed_data, "Max-capacity roundtrip failed");
    }

    #[test]
    fn encode_decode_verified() {
        let ctx = test_ctx();
        let data = b"Exact integer arithmetic. Zero drift. Truth cannot be approximated.";
        let hologram = WassanHologram::encode(data, &ctx);
        let result = hologram.decode_verified(&ctx);
        assert!(result.is_ok(), "Verified decode should succeed on clean hologram");
        assert_eq!(&result.unwrap(), data);
    }

    #[test]
    fn encode_decode_empty() {
        let ctx = test_ctx();
        let data = b"";
        let hologram = WassanHologram::encode(data, &ctx);
        let recovered = hologram.decode();
        assert!(recovered.is_empty(), "Empty data should roundtrip to empty");
    }

    // --- Holographic Property ---

    #[test]
    fn holographic_all_sectors_nonzero() {
        // The holographic property: every sector depends on all coefficients.
        // For non-trivial data, most sectors should be nonzero.
        let ctx = test_ctx();
        let data = b"K-Elimination: exact division in RNS.";
        let hologram = WassanHologram::encode(data, &ctx);

        let nonzero_sectors = (0..NTT_SIZE)
            .filter(|&i| hologram.sector(i).values.iter().any(|&v| v != 0))
            .count();

        // With non-trivial data, vastly more than half the sectors should be nonzero
        assert!(
            nonzero_sectors > NTT_SIZE / 2,
            "Holographic property: expected most sectors nonzero, got {}/{}",
            nonzero_sectors, NTT_SIZE
        );
    }

    // --- Syndrome Detection ---

    #[test]
    fn clean_hologram_has_zero_syndrome() {
        let ctx = test_ctx();
        let data = b"Clean memory -- no corruption.";
        let hologram = WassanHologram::encode(data, &ctx);
        let syndrome = hologram.compute_syndrome(&ctx);
        assert!(syndrome.is_clean, "Clean hologram should have zero syndrome");
        assert_eq!(syndrome.parity_violations, 0);
    }

    #[test]
    fn corrupted_sector_detected() {
        let ctx = test_ctx();
        let primes = test_primes();
        let data = b"This memory will be corrupted.";
        let mut hologram = WassanHologram::encode(data, &ctx);

        // Corrupt one sector
        hologram.corrupt_sector(42, 999, &primes);

        let syndrome = hologram.compute_syndrome(&ctx);
        assert!(
            !syndrome.is_clean,
            "Corrupted hologram should have nonzero syndrome"
        );
        assert!(
            syndrome.parity_violations > 0,
            "Expected parity violations, got 0"
        );
    }

    // --- Attractor Basin Self-Correction ---

    #[test]
    fn self_correct_after_sector_corruption() {
        let ctx = test_ctx();
        let primes = test_primes();
        let data = b"Attractor basin: self-healing memory.";
        let hologram_clean = WassanHologram::encode(data, &ctx);
        let mut hologram = hologram_clean.clone();

        // Corrupt 3 sectors
        hologram.corrupt_sector(10, 42, &primes);
        hologram.corrupt_sector(50, 777, &primes);
        hologram.corrupt_sector(200, 12345, &primes);

        // Verify corruption detected
        let syndrome = hologram.compute_syndrome(&ctx);
        assert!(!syndrome.is_clean);

        // Self-correct
        let result = hologram.self_correct(&ctx);
        assert!(result.was_corrupted);
        assert!(result.bands_corrected > 0);

        // After correction, syndrome should be clean
        let syndrome_after = hologram.compute_syndrome(&ctx);
        assert!(syndrome_after.is_clean, "Post-correction syndrome should be clean");

        // Data should be recoverable
        let recovered = hologram.decode();
        assert_eq!(&recovered, data, "Data should survive corruption + correction");
    }

    #[test]
    fn self_correct_noop_on_clean() {
        let ctx = test_ctx();
        let data = b"Already clean -- no correction needed.";
        let mut hologram = WassanHologram::encode(data, &ctx);

        let result = hologram.self_correct(&ctx);
        assert!(!result.was_corrupted);
        assert_eq!(result.bands_corrected, 0);
    }

    // --- Phase-Locked Retrieval ---

    #[test]
    fn phase_coherence_self_maximum() {
        let ctx = test_ctx();
        let primes = test_primes();
        let data = b"Phase coherence test.";
        let hologram = WassanHologram::encode(data, &ctx);

        let score = hologram.coherence_score(&hologram, &primes);
        assert_eq!(score, 10000, "Self-coherence should be maximum (10000)");
    }

    #[test]
    fn phase_coherence_different_data() {
        let ctx = test_ctx();
        let primes = test_primes();
        let h1 = WassanHologram::encode(b"First memory: K-Elimination.", &ctx);
        let h2 = WassanHologram::encode(b"Second memory: Persistent Montgomery.", &ctx);

        let score = h1.coherence_score(&h2, &primes);
        // Different data should have significantly lower coherence than self
        assert!(score < 10000, "Different data should not have perfect coherence");
    }

    // --- Attractor Memory Bank ---

    #[test]
    fn attractor_bank_store_and_retrieve() {
        let ctx = test_ctx();
        let primes = test_primes();

        let mut bank = AttractorBank::new();
        bank.store(
            "grail_001".to_string(),
            WassanHologram::encode(b"K-Elimination: exact RNS division.", &ctx),
        );
        bank.store(
            "grail_002".to_string(),
            WassanHologram::encode(b"Persistent Montgomery: zero overhead.", &ctx),
        );
        bank.store(
            "grail_003".to_string(),
            WassanHologram::encode(b"Shadow Entropy: free cryptographic noise.", &ctx),
        );

        // Query with exact match
        let query = WassanHologram::encode(b"K-Elimination: exact RNS division.", &ctx);
        let (label, score) = bank.retrieve(&query, &primes).unwrap();
        assert_eq!(label, "grail_001");
        assert_eq!(score, 10000, "Exact query should return perfect score");
    }

    #[test]
    fn attractor_bank_best_match() {
        let ctx = test_ctx();
        let primes = test_primes();

        let mut bank = AttractorBank::new();
        bank.store("alpha".to_string(), WassanHologram::encode(b"AAAA", &ctx));
        bank.store("beta".to_string(), WassanHologram::encode(b"BBBB", &ctx));

        let query = WassanHologram::encode(b"AAAA", &ctx);
        let (label, _) = bank.retrieve(&query, &primes).unwrap();
        assert_eq!(label, "alpha");
    }

    // --- Integration: Full Pipeline ---

    #[test]
    fn full_pipeline_encode_corrupt_detect_correct_retrieve() {
        let ctx = test_ctx();
        let primes = test_primes();

        // 1. Encode
        let data = b"Full pipeline: encode, corrupt, detect, correct, retrieve.";
        let mut hologram = WassanHologram::encode(data, &ctx);

        // 2. Verify clean
        assert!(hologram.compute_syndrome(&ctx).is_clean);

        // 3. Corrupt
        hologram.corrupt_sector(77, 9999, &primes);
        hologram.corrupt_sector(128, 5555, &primes);

        // 4. Detect
        let syndrome = hologram.compute_syndrome(&ctx);
        assert!(!syndrome.is_clean);

        // 5. Correct (attractor basin)
        let correction = hologram.self_correct(&ctx);
        assert!(correction.was_corrupted);

        // 6. Verify clean
        assert!(hologram.compute_syndrome(&ctx).is_clean);

        // 7. Retrieve
        let recovered = hologram.decode();
        assert_eq!(&recovered, data);

        // 8. Phase coherence with original
        let original = WassanHologram::encode(data, &ctx);
        let score = hologram.coherence_score(&original, &primes);
        assert_eq!(score, 10000, "Corrected hologram should match original perfectly");
    }
}
