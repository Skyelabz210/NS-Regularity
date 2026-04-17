//! Intrinsic CRAM Address — The Number's Position Outside of Time
//!
//! Every integer n has a fixed address on the multi-head CRAM torus.
//! This address is INTRINSIC — it depends only on n, not on any gap,
//! range, scanning order, or calibration window.
//!
//! Each Hydra head probes n from a different angle:
//!   Head A: root(n mod p, 1) — additive unit probe (shift by 1)
//!   Head B: root(n mod p, n mod p) — self-interaction (quadratic character)
//!   Head C: root(n mod p, p-1) — boundary probe (distance to modular edge)
//!
//! The multi-head CRAM address IS the number's position in the coprime
//! lattice. Primes live at specific addresses. That distribution is
//! a static mathematical object — an atlas computed once, valid forever.
//!
//! The atlas is OUTSIDE OF TIME: it doesn't depend on which numbers
//! you've seen, what range you're searching, or what order you scan.
//! It's a property of the integers themselves under CRAM projection.
//!
//! The DIMENSION is the number of (head × lane × post) coordinates.
//! Each coordinate is a carry bit. More heads = higher dimension =
//! finer resolution of the coprime lattice.

use crate::coprime_families::{CramTopology, Lane, Root, Post};
use crate::crt;

// ── Modular arithmetic ──────────────────────────────────────────────

fn addmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 + b as u128) % m as u128) as u64 }
fn submod(a: u64, b: u64, m: u64) -> u64 { if a >= b { (a-b) % m } else { m - ((b-a) % m) } }
fn mulmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 * b as u128) % m as u128) as u64 }

fn is_prime_u64(n: u64) -> bool {
    if n < 2 { return false; } if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5u64;
    while i.saturating_mul(i) <= n { if n % i == 0 || n % (i+2) == 0 { return false; } i += 6; }
    true
}

// ── Intrinsic Probe Types ───────────────────────────────────────────

/// What does the root operator's second argument come from?
/// This is what makes the address INTRINSIC — no external gap needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Probe {
    /// root(n mod p, 1) — unit shift: how does n respond to +1, -1, ×1?
    Unit,
    /// root(n mod p, n mod p) — self-interaction: n's quadratic character
    Self_,
    /// root(n mod p, p-1 mod p) — edge probe: n's distance from modular boundary
    Edge,
    /// root(n mod p, 2 mod p) — parity probe: n's relationship to 2
    Parity,
    /// Pure residue: just n mod p, no root operator, post only
    Residue,
}

impl std::fmt::Display for Probe {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Probe::Unit => write!(f, "Unit"),
            Probe::Self_ => write!(f, "Self"),
            Probe::Edge => write!(f, "Edge"),
            Probe::Parity => write!(f, "Par"),
            Probe::Residue => write!(f, "Res"),
        }
    }
}

/// One dimension of the intrinsic address: (prime, probe, root_op, post_op) → carry bit
#[derive(Debug, Clone, Copy)]
pub struct Dimension {
    pub prime: u64,
    pub probe: Probe,
    pub root: Root,
    pub post: Post,
}

impl Dimension {
    /// Compute this dimension's value for integer n. Returns carry bit (0 or 1).
    pub fn evaluate(&self, n: u64) -> u8 {
        let p = self.prime;
        let n_r = n % p;

        // Determine second operand based on probe type
        let b_r = match self.probe {
            Probe::Unit    => 1u64 % p,
            Probe::Self_   => n_r,
            Probe::Edge    => (p - 1) % p,
            Probe::Parity  => 2u64 % p,
            Probe::Residue => 0, // unused for Residue probe
        };

        // Apply root operator (or skip for Residue probe)
        let inter = if self.probe == Probe::Residue {
            n_r
        } else {
            match self.root {
                Root::Add => addmod(n_r, b_r, p),
                Root::Sub => submod(n_r, b_r, p),
                Root::Mul => mulmod(n_r, b_r, p),
                Root::Div => {
                    if b_r == 0 { 0 }
                    else { crate::coprime_families::mod_inv_pub(b_r, p).map(|inv| mulmod(n_r, inv, p)).unwrap_or(0) }
                }
            }
        };

        // Apply post-processor
        let out = match self.post {
            Post::Id  => inter,
            Post::Neg => if inter == 0 { 0 } else { p - inter },
            Post::Sqr => mulmod(inter, inter, p),
            Post::Inv => {
                if inter == 0 { 0 }
                else { crate::coprime_families::mod_inv_pub(inter, p).unwrap_or(0) }
            }
        };

        // Carry bit: threshold at p/2
        if out * 2 >= p { 1 } else { 0 }
    }
}

// ── Intrinsic Address ───────────────────────────────────────────────

/// The full CRAM address of a number: a binary vector across all dimensions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address {
    pub bits: Vec<u8>,
}

impl Address {
    pub fn to_id(&self) -> u64 {
        let mut id = 0u64;
        for (i, &bit) in self.bits.iter().enumerate() {
            if i >= 64 { break; }
            id |= (bit as u64) << i;
        }
        id
    }

    pub fn dim(&self) -> usize { self.bits.len() }
}

// ── Address Space: The Multi-Dimensional CRAM Torus ─────────────────

/// The address space: a set of dimensions that define the CRAM torus.
/// Every integer has a unique address in this space.
pub struct AddressSpace {
    pub dimensions: Vec<Dimension>,
    pub modulus: u64, // LCM of all primes (defines the period)
}

impl AddressSpace {
    pub fn new(dimensions: Vec<Dimension>) -> Self {
        let mut modulus = 1u64;
        let mut seen_primes = Vec::new();
        for dim in &dimensions {
            if !seen_primes.contains(&dim.prime) {
                modulus *= dim.prime;
                seen_primes.push(dim.prime);
            }
        }
        Self { dimensions, modulus }
    }

    pub fn ndim(&self) -> usize { self.dimensions.len() }

    /// Compute the intrinsic address of n. O(ndim) integer operations.
    pub fn address_of(&self, n: u64) -> Address {
        Address {
            bits: self.dimensions.iter().map(|d| d.evaluate(n)).collect()
        }
    }

    /// Check if n is coprime to the modulus (in the coprime lattice).
    pub fn is_coprime(&self, n: u64) -> bool {
        crt::gcd(n as u128, self.modulus as u128) == 1
    }

    /// The address is PERIODIC with period = modulus.
    /// address_of(n) == address_of(n + k * modulus) for all k.
    /// This is the "outside of time" property.
    pub fn period(&self) -> u64 { self.modulus }
}

// ── Pre-built Address Spaces ────────────────────────────────────────

/// Build a rich multi-probe address space on the safe basis {3,5,7,11,13}.
pub fn build_full_address_space() -> AddressSpace {
    let primes = [3u64, 5, 7, 11, 13];
    let probes = [Probe::Unit, Probe::Self_, Probe::Edge, Probe::Residue];
    let posts = [Post::Id, Post::Sqr];

    let mut dims = Vec::new();
    for &p in &primes {
        for &probe in &probes {
            let root = match probe {
                Probe::Residue => Root::Add, // unused but needed
                Probe::Self_   => Root::Mul, // n × n = Sqr
                _              => Root::Add, // default
            };
            for &post in &posts {
                dims.push(Dimension { prime: p, probe, root, post });
            }
        }
    }
    // 5 primes × 4 probes × 2 posts = 40 dimensions → 2^40 addresses (1 trillion)
    // Too many — trim to the most discriminative
    // Use: Unit+Id, Unit+Sqr, Self+Sqr, Residue+Id, Residue+Sqr per prime = 25 dims
    let mut trimmed = Vec::new();
    for &p in &primes {
        trimmed.push(Dimension { prime: p, probe: Probe::Unit, root: Root::Add, post: Post::Id });
        trimmed.push(Dimension { prime: p, probe: Probe::Unit, root: Root::Sub, post: Post::Sqr });
        trimmed.push(Dimension { prime: p, probe: Probe::Self_, root: Root::Mul, post: Post::Sqr });
        trimmed.push(Dimension { prime: p, probe: Probe::Residue, root: Root::Add, post: Post::Id });
        trimmed.push(Dimension { prime: p, probe: Probe::Edge, root: Root::Add, post: Post::Sqr });
    }
    AddressSpace::new(trimmed) // 25 dimensions
}

/// Compact space: fewer dimensions, manageable stratum count.
pub fn build_compact_address_space() -> AddressSpace {
    let primes = [3u64, 5, 7, 11, 13];
    let mut dims = Vec::new();
    for &p in &primes {
        // 3 dimensions per prime: Residue+Id, Unit+Sqr, Self+Sqr
        dims.push(Dimension { prime: p, probe: Probe::Residue, root: Root::Add, post: Post::Id });
        dims.push(Dimension { prime: p, probe: Probe::Unit, root: Root::Sub, post: Post::Sqr });
        dims.push(Dimension { prime: p, probe: Probe::Self_, root: Root::Mul, post: Post::Sqr });
    }
    AddressSpace::new(dims) // 15 dimensions → 32768 addresses
}

// ── Address Atlas: The Eternal Map ──────────────────────────────────

/// The atlas: prime density at every address, computed once.
/// Since addresses are periodic (period = modulus), the atlas is finite.
/// Every coprime residue class mod M has a fixed address.
/// The atlas maps address → prime density (measured or predicted).
#[derive(Debug)]
pub struct AddressAtlas {
    pub space_ndim: usize,
    pub modulus: u64,
    pub entries: std::collections::HashMap<u64, AtlasEntry>,
    pub total_addresses: u32,
    pub prime_rich_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct AtlasEntry {
    pub address_id: u64,
    pub coprime_count: u32,
    pub prime_count: u32,
    pub density: f64,
    pub is_prime_rich: bool,
}

impl AddressAtlas {
    /// Build the atlas by scanning one full period [2, modulus + sample_range).
    /// Since addresses repeat with period = modulus, scanning one period
    /// gives complete coverage. Additional range adds statistics.
    pub fn build(space: &AddressSpace, scan_limit: u64, threshold_percentile: f64) -> Self {
        let mut entries: std::collections::HashMap<u64, (u32, u32)> = std::collections::HashMap::new();

        for n in 2..scan_limit {
            if !space.is_coprime(n) { continue; }
            let addr = space.address_of(n);
            let id = addr.to_id();
            let entry = entries.entry(id).or_insert((0, 0));
            entry.0 += 1;
            if is_prime_u64(n) { entry.1 += 1; }
        }

        // Compute densities
        let mut atlas_entries: Vec<(u64, f64)> = entries.iter()
            .map(|(&id, &(cc, pc))| (id, if cc > 0 { pc as f64 / cc as f64 } else { 0.0 }))
            .collect();
        atlas_entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Threshold: capture top percentile of primes
        let total_primes: u32 = entries.values().map(|v| v.1).sum();
        let target = (total_primes as f64 * threshold_percentile) as u32;
        let mut captured = 0u32;
        let mut threshold = 0.0f64;
        let mut rich_ids = std::collections::HashSet::new();

        for &(id, density) in &atlas_entries {
            let pc = entries[&id].1;
            if captured < target || threshold_percentile >= 0.999 {
                captured += pc;
                threshold = density;
                rich_ids.insert(id);
            }
        }

        let final_entries = entries.into_iter().map(|(id, (cc, pc))| {
            let density = if cc > 0 { pc as f64 / cc as f64 } else { 0.0 };
            (id, AtlasEntry {
                address_id: id,
                coprime_count: cc,
                prime_count: pc,
                density,
                is_prime_rich: rich_ids.contains(&id),
            })
        }).collect();

        AddressAtlas {
            space_ndim: space.ndim(),
            modulus: space.period(),
            entries: final_entries,
            total_addresses: atlas_entries.len() as u32,
            prime_rich_threshold: threshold,
        }
    }

    /// Query: is this address prime-rich?
    pub fn is_prime_rich(&self, addr_id: u64) -> bool {
        self.entries.get(&addr_id).map(|e| e.is_prime_rich).unwrap_or(true)
        // Unknown addresses default to "test" (safe)
    }

    pub fn prime_rich_count(&self) -> u32 {
        self.entries.values().filter(|e| e.is_prime_rich).count() as u32
    }

    pub fn prime_poor_count(&self) -> u32 {
        self.entries.values().filter(|e| !e.is_prime_rich).count() as u32
    }

    /// Gini coefficient of prime density across addresses
    pub fn gini(&self) -> f64 {
        let densities: Vec<f64> = self.entries.values()
            .filter(|e| e.coprime_count > 0)
            .map(|e| e.density)
            .collect();
        if densities.is_empty() { return 0.0; }
        let n = densities.len() as f64;
        let mean = densities.iter().sum::<f64>() / n;
        if mean == 0.0 { return 0.0; }
        let abs_diff: f64 = densities.iter()
            .flat_map(|&a| densities.iter().map(move |&b| (a - b).abs()))
            .sum();
        abs_diff / (2.0 * n * n * mean)
    }
}

/// The eternal hunt: compute address → check atlas → test or skip.
/// Works for ANY n, in ANY order, with NO calibration per range.
pub fn eternal_hunt(
    space: &AddressSpace,
    atlas: &AddressAtlas,
    start: u64,
    limit: u64,
) -> EternalHuntResult {
    let mut primes_found = Vec::new();
    let mut tested = 0u64;
    let mut skipped = 0u64;
    let mut false_negatives = 0u64;
    let mut coprimes = 0u64;

    for n in start.max(2)..limit {
        if !space.is_coprime(n) { continue; }
        coprimes += 1;

        let addr = space.address_of(n);
        let addr_id = addr.to_id();

        if atlas.is_prime_rich(addr_id) {
            tested += 1;
            if is_prime_u64(n) { primes_found.push(n); }
        } else {
            skipped += 1;
            if is_prime_u64(n) { false_negatives += 1; }
        }
    }

    let actual = primes_found.len() as u64 + false_negatives;
    EternalHuntResult {
        primes_found,
        coprimes,
        tested,
        skipped,
        false_negatives,
        recall: if actual > 0 { (actual - false_negatives) as f64 / actual as f64 } else { 1.0 },
        speedup: if tested > 0 { coprimes as f64 / tested as f64 } else { 1.0 },
    }
}

#[derive(Debug)]
pub struct EternalHuntResult {
    pub primes_found: Vec<u64>,
    pub coprimes: u64,
    pub tested: u64,
    pub skipped: u64,
    pub false_negatives: u64,
    pub recall: f64,
    pub speedup: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_periodicity() {
        let space = build_compact_address_space();
        let m = space.period();
        // Address of n must equal address of n + m
        for n in [17u64, 29, 97, 101, 1009] {
            let a1 = space.address_of(n);
            let a2 = space.address_of(n + m);
            assert_eq!(a1, a2, "Address not periodic for n={}", n);
        }
    }

    #[test]
    fn test_address_deterministic() {
        let space = build_compact_address_space();
        // Same n always gives same address
        let a1 = space.address_of(97);
        let a2 = space.address_of(97);
        assert_eq!(a1, a2);
    }

    #[test]
    fn test_atlas_build() {
        let space = build_compact_address_space();
        let atlas = AddressAtlas::build(&space, 10_000, 0.95);
        assert!(atlas.total_addresses > 0);
        assert!(atlas.prime_rich_count() > 0);
    }

    #[test]
    fn test_eternal_hunt_finds_primes() {
        let space = build_compact_address_space();
        let atlas = AddressAtlas::build(&space, 10_000, 1.00);
        let result = eternal_hunt(&space, &atlas, 10_000, 20_000);
        assert!(!result.primes_found.is_empty());
        for &p in &result.primes_found {
            assert!(is_prime_u64(p));
        }
    }
}
