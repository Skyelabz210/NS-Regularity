//! Dresden–Codex Bridge: Intrinsic CRAM Address on Codex Gear Manifold
//!
//! Wires the Dresden Prime Hunt (T11-T20, March 2026) directly onto the
//! production Codex Gear Manifold and DualCodexSiblings architecture.
//!
//! MAPPING:
//!
//! CodexGear.modulus  → lane prime pᵢ
//! CodexGear.residue  → n mod pᵢ
//! carry_bit(gear)    → ⌊2 × residue / modulus⌋
//! IntrinsicAddress   → concatenation of carry bits across gears
//! Atlas              → HashMap<address_id, (coprimes, primes)>
//!
//! DualCodexSiblings:
//!   Primary (Grover tier)    → gears with p ≡ 3 (mod 4): {3, 7, 11}
//!   Secondary (Classical)    → gears with p ≡ 1 (mod 4): {5, 13}
//!   Shared anchors           → K-Elimination bridge between siblings
//!
//! Grover-Hydra:
//!   Each primary gear       → F_p² amplitude substrate, Dense Toric Grover
//!   Each secondary gear     → classical carry bit, QR separation (T16)
//!   Atlas oracle            → marked states = prime-rich strata
//!   Ramanujan-Grover formula → T_opt per gear from T6 c_p(d)
//!   WASSAN                  → amplitude storage (144 φ-bands, 7 NTT lanes)
//!   K-Elimination           → candidate extraction from shared anchors
//!
//! INTEGRATION LAYERS:
//!
//! Layer 0: CodexGear (modular primitives) — existing
//! Layer 1: CodexManifold (CRT lanes) — existing
//! Layer 2: CRAM operators (heterogeneous per-lane) — existing via cram-core
//! Layer 3: DualCodexSiblings (Grover/Classical split) — existing
//! Layer 4: WASSAN (amplitude storage) — existing
//! Layer 5: PLMG Rails/Voids (prime exclusion zone) — existing
//! Layer 6: Double Helix (dual-strand compute+verify) — existing
//! Layer 7: Ramanujan Spectral (T6 sieve power) — existing
//! Layer 8: ** INTRINSIC CRAM ADDRESS (this bridge) ** — NEW
//!            ├─ Carry bit extraction from gear residues
//!            ├─ Multi-probe intrinsic dimensions
//!            ├─ Atlas construction and lookup
//!            ├─ Grover-Hydra tier classification
//!            ├─ Ramanujan-Grover iteration count
//!            └─ Joint pair prediction (CRAM schema)
//!
//! Layer 8 is a READ-ONLY observer on Layers 0-7.
//! It computes addresses and atlas lookups from existing gear state.
//! It does NOT modify any gear residue or manifold structure.

// ═══════════════════════════════════════════════════════════════════
// GEAR → CARRY BIT (the fundamental observation)
// ═══════════════════════════════════════════════════════════════════

/// Extract carry bit from a CodexGear's current state.
/// carry = 1 if residue × 2 ≥ modulus, else 0.
/// This is the CRAM threshold that creates nonlinear structure.
pub fn gear_carry_bit(residue: u64, modulus: u64) -> u8 {
    if residue * 2 >= modulus { 1 } else { 0 }
}

/// Apply CRAM operator to gear residue, then extract carry bit.
/// root_op(residue, probe_value) → intermediate → post_op → carry threshold
pub fn cram_gear_carry(
    residue: u64,
    probe_value: u64,
    modulus: u64,
    root: CramRoot,
    post: CramPost,
) -> u8 {
    let r = residue % modulus;
    let b = probe_value % modulus;

    // Root operator
    let inter = match root {
        CramRoot::Add => (r as u128 + b as u128) as u64 % modulus,
        CramRoot::Sub => if r >= b { r - b } else { modulus - (b - r) },
        CramRoot::Mul => ((r as u128 * b as u128) % modulus as u128) as u64,
        CramRoot::Id  => r,
    };

    // Post-processor
    let out = match post {
        CramPost::Id  => inter,
        CramPost::Neg => if inter == 0 { 0 } else { modulus - inter },
        CramPost::Sqr => ((inter as u128 * inter as u128) % modulus as u128) as u64,
    };

    // Carry threshold
    if out * 2 >= modulus { 1 } else { 0 }
}

#[derive(Debug, Clone, Copy)]
pub enum CramRoot { Add, Sub, Mul, Id }

#[derive(Debug, Clone, Copy)]
pub enum CramPost { Id, Neg, Sqr }

// ═══════════════════════════════════════════════════════════════════
// INTRINSIC ADDRESS ON GEAR MANIFOLD (Layer 8)
// ═══════════════════════════════════════════════════════════════════

/// One dimension of the intrinsic address space.
/// Maps to a specific gear + probe + operator combination.
#[derive(Debug, Clone, Copy)]
pub struct GearDimension {
    pub gear_index: usize,   // which gear in the manifold
    pub prime: u64,          // gear modulus (must match gear.modulus)
    pub probe: GearProbe,    // what second operand to use
    pub root: CramRoot,      // root operator
    pub post: CramPost,      // post-processor
}

#[derive(Debug, Clone, Copy)]
pub enum GearProbe {
    Residue,  // just the raw residue → post only
    Unit,     // probe with 1
    SelfMul,  // probe with residue itself (quadratic character)
    Edge,     // probe with modulus - 1
}

impl GearDimension {
    /// Evaluate this dimension for a given gear residue.
    pub fn evaluate(&self, residue: u64) -> u8 {
        let probe_val = match self.probe {
            GearProbe::Residue => return gear_carry_bit(
                match self.post {
                    CramPost::Sqr => ((residue as u128 * residue as u128) % self.prime as u128) as u64,
                    CramPost::Neg => if residue == 0 { 0 } else { self.prime - residue },
                    CramPost::Id  => residue % self.prime,
                },
                self.prime
            ),
            GearProbe::Unit    => 1,
            GearProbe::SelfMul => residue % self.prime,
            GearProbe::Edge    => self.prime - 1,
        };
        cram_gear_carry(residue, probe_val, self.prime, self.root, self.post)
    }
}

/// The address space: a set of gear dimensions.
/// Maps directly onto a CodexManifold's gear set.
pub struct GearAddressSpace {
    pub dimensions: Vec<GearDimension>,
    pub period: u64,  // LCM of gear moduli
}

impl GearAddressSpace {
    /// Build address space from a manifold's gear moduli.
    /// 3 dimensions per gear: Residue+Id, Unit+Sub+Sqr, Self+Mul+Sqr
    pub fn from_moduli(moduli: &[u64]) -> Self {
        let mut dims = Vec::new();
        for (i, &p) in moduli.iter().enumerate() {
            if p <= 2 { continue; } // skip p=2 (trivial carry)
            dims.push(GearDimension {
                gear_index: i, prime: p,
                probe: GearProbe::Residue, root: CramRoot::Id, post: CramPost::Id,
            });
            dims.push(GearDimension {
                gear_index: i, prime: p,
                probe: GearProbe::Unit, root: CramRoot::Sub, post: CramPost::Sqr,
            });
            dims.push(GearDimension {
                gear_index: i, prime: p,
                probe: GearProbe::SelfMul, root: CramRoot::Mul, post: CramPost::Sqr,
            });
        }
        let period = moduli.iter().fold(1u64, |a, &m| a * m);
        Self { dimensions: dims, period }
    }

    /// Compute intrinsic address from gear residues.
    pub fn address_from_residues(&self, residues: &[u64]) -> u64 {
        let mut addr = 0u64;
        for (i, dim) in self.dimensions.iter().enumerate() {
            if i >= 64 { break; }
            let r = residues[dim.gear_index] % dim.prime;
            let bit = dim.evaluate(r);
            addr |= (bit as u64) << i;
        }
        addr
    }

    pub fn ndim(&self) -> usize { self.dimensions.len() }
}

// ═══════════════════════════════════════════════════════════════════
// GROVER-HYDRA TIER CLASSIFICATION ON GEAR MANIFOLD
// ═══════════════════════════════════════════════════════════════════

/// Classify each gear as Grover-tier or Classical-tier.
/// Maps to DualCodexSiblings: primary = Grover, secondary = Classical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GearTier {
    Grover,    // p ≡ 3 (mod 4), F_p² exists → Dense Toric Grover
    Classical, // p ≡ 1 (mod 4), QR separation via Sqr
}

pub fn classify_gear(modulus: u64) -> GearTier {
    if modulus % 4 == 3 { GearTier::Grover } else { GearTier::Classical }
}

/// Split a manifold's gears into Grover and Classical siblings.
/// Returns (grover_indices, classical_indices).
pub fn split_dual_siblings(moduli: &[u64]) -> (Vec<usize>, Vec<usize>) {
    let mut grover = Vec::new();
    let mut classical = Vec::new();
    for (i, &m) in moduli.iter().enumerate() {
        match classify_gear(m) {
            GearTier::Grover => grover.push(i),
            GearTier::Classical => classical.push(i),
        }
    }
    (grover, classical)
}

// ═══════════════════════════════════════════════════════════════════
// ATLAS ON GEAR MANIFOLD
// ═══════════════════════════════════════════════════════════════════

/// Atlas entry: prime density at a specific address.
#[derive(Debug, Clone)]
pub struct GearAtlasEntry {
    pub address: u64,
    pub coprime_count: u32,
    pub prime_count: u32,
}

impl GearAtlasEntry {
    pub fn density_permille(&self) -> u64 {
        if self.coprime_count == 0 { return 0; }
        (self.prime_count as u64 * 1000) / self.coprime_count as u64
    }
}

/// The atlas: maps gear addresses to prime density.
/// Built once, applied forever (T14: range-stable transfer).
pub struct GearAtlas {
    pub entries: Vec<GearAtlasEntry>,
    pub prime_rich_ids: Vec<u64>,  // sorted for binary search
    pub threshold_permille: u64,
}

impl GearAtlas {
    pub fn is_prime_rich(&self, addr: u64) -> bool {
        self.prime_rich_ids.binary_search(&addr).is_ok()
    }
}

// ═══════════════════════════════════════════════════════════════════
// RAMANUJAN-GROVER ON GEAR MANIFOLD
// ═══════════════════════════════════════════════════════════════════

/// Compute Ramanujan-Grover iteration count for a gear.
/// T_opt(p,d) = ⌊π/4 · p / √(2p-1-c_p(d))⌋
pub fn gear_grover_t_opt(modulus: u64, gap: u64) -> u64 {
    let c: i64 = if gap % modulus == 0 { modulus as i64 - 1 } else { -1 };
    let fw = (2 * modulus as i64 - 1 - c) as u64;
    let isqrt = {
        let mut x = fw; let mut y = (x + 1) / 2;
        while y < x { x = y; y = (x + fw / x) / 2; }
        x
    };
    if isqrt == 0 { return modulus; }
    (785 * modulus) / (1000 * isqrt)
}

// ═══════════════════════════════════════════════════════════════════
// COMPLETE LAYER 8 INTEGRATION SPEC
// ═══════════════════════════════════════════════════════════════════

/// The full Layer 8 integration: all Dresden discoveries on gear manifold.
///
/// Usage:
///   1. Build GearAddressSpace from manifold moduli
///   2. Build GearAtlas by scanning calibration range
///   3. Classify gears → split DualCodexSiblings
///   4. For each candidate n:
///      a. Encode n into manifold (manifold.encode(n))
///      b. Read gear residues
///      c. Compute address = address_from_residues(residues)
///      d. Lookup atlas.is_prime_rich(address)
///      e. If rich: primality test. If poor: skip.
///   5. For connector primes (Infinite Scaffold):
///      a. Classify tier (Grover or Classical)
///      b. Compute T_opt via Ramanujan-Grover formula
///      c. Run Dense Toric Grover on Grover-tier gears
///      d. K-Eliminate across sibling anchors
///      e. WASSAN stores amplitudes
///
/// All operations are Layer 8 observations on Layer 0-7 state.
/// Layer 8 does NOT modify any gear.
pub struct Layer8 {
    pub address_space: GearAddressSpace,
    pub grover_indices: Vec<usize>,
    pub classical_indices: Vec<usize>,
}

impl Layer8 {
    pub fn new(moduli: &[u64]) -> Self {
        let address_space = GearAddressSpace::from_moduli(moduli);
        let (grover, classical) = split_dual_siblings(moduli);
        Self { address_space, grover_indices: grover, classical_indices: classical }
    }

    pub fn address(&self, residues: &[u64]) -> u64 {
        self.address_space.address_from_residues(residues)
    }

    pub fn ndim(&self) -> usize { self.address_space.ndim() }
    pub fn period(&self) -> u64 { self.address_space.period }
}
