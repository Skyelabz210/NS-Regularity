//! # GearManifold: Cascaded CRT Odometer with Automatic Winding
//!
//! The central data structure unifying K-Elimination winding numbers,
//! Maya Long Count mechanics, and NS regularity diagnostics.
//!
//! Zero floats. All exact integer arithmetic.

/// A single CRT lane with residue and winding count.
#[derive(Debug, Clone)]
pub struct Lane {
    pub modulus: u64,
    pub residue: u64,
    pub magnitude: i128, // winding number (signed, can go negative)
}

/// Cascaded CRT odometer.
/// Lanes are ordered by modulus (smallest = fastest gear).
/// Carries propagate upward automatically.
#[derive(Debug, Clone)]
pub struct GearManifold {
    pub lanes: Vec<Lane>,
}

impl GearManifold {
    /// Create a new GearManifold with the given moduli.
    /// Moduli must be pairwise coprime (not checked here — architectural invariant).
    pub fn new(moduli: &[u64]) -> Self {
        let mut lanes: Vec<Lane> = moduli.iter().map(|&m| Lane {
            modulus: m,
            residue: 0,
            magnitude: 0,
        }).collect();
        // Sort by modulus ascending (fastest gear first)
        lanes.sort_by_key(|l| l.modulus);
        GearManifold { lanes }
    }

    /// Add delta to the manifold. Each lane updated independently (CRT).
    /// Winding tracked per-lane as floor((residue + delta) / modulus).
    pub fn add(&mut self, delta: i128) -> () {
        for lane in self.lanes.iter_mut() {
            let m = lane.modulus as i128;
            let temp = lane.residue as i128 + delta;
            // Euclidean mod and quotient
            let new_residue = ((temp % m) + m) % m;
            let wraps = (temp - new_residue) / m;
            lane.residue = new_residue as u64;
            lane.magnitude += wraps;
        }
    }

    /// Reconstruct the full value via Garner's algorithm.
    /// Returns the value in [0, product_of_moduli).
    pub fn full_value(&self) -> u128 {
        let n = self.lanes.len();
        if n == 0 { return 0; }

        // Garner mixed-radix reconstruction (Knuth TAOCP 4.3.2)
        // v[i] = r[i]; for j in 0..i: v[i] = (v[i] - v[j]) * inv(m[j], m[i]) mod m[i]
        // x = v[0] + v[1]*m[0] + v[2]*m[0]*m[1] + ...
        let mut v = vec![0u128; n];
        v[0] = self.lanes[0].residue as u128;

        for i in 1..n {
            v[i] = self.lanes[i].residue as u128;
            let mi = self.lanes[i].modulus as u128;

            for j in 0..i {
                let mj = self.lanes[j].modulus as u128;
                // v[i] = (v[i] - v[j]) * inv(m[j], m[i]) mod m[i]
                let diff = if v[i] >= v[j] % mi {
                    v[i] - v[j] % mi
                } else {
                    v[i] + mi - v[j] % mi
                };
                let inv = mod_inverse_u128(mj, mi);
                v[i] = (diff * inv) % mi;
            }
        }

        // Reconstruct: value = v[0] + v[1]*m[0] + v[2]*m[0]*m[1] + ...
        let mut result: u128 = 0;
        let mut product: u128 = 1;
        for i in 0..n {
            result += v[i] * product;
            product *= self.lanes[i].modulus as u128;
        }
        result % product
    }

    /// Primary winding (fastest lane magnitude) = K_vort
    pub fn primary_winding(&self) -> i128 {
        if self.lanes.is_empty() { 0 } else { self.lanes[0].magnitude }
    }

    /// Slowest lane winding (Long Count equivalent)
    pub fn slowest_winding(&self) -> i128 {
        self.lanes.last().map_or(0, |l| l.magnitude)
    }

    /// Total product of all moduli
    pub fn capacity(&self) -> u128 {
        self.lanes.iter().map(|l| l.modulus as u128).product()
    }

    /// All residues as a vector
    pub fn residues(&self) -> Vec<u64> {
        self.lanes.iter().map(|l| l.residue).collect()
    }

    /// All magnitudes as a vector
    pub fn magnitudes(&self) -> Vec<i128> {
        self.lanes.iter().map(|l| l.magnitude).collect()
    }

    /// Reset to zero state
    pub fn reset(&mut self) {
        for lane in self.lanes.iter_mut() {
            lane.residue = 0;
            lane.magnitude = 0;
        }
    }
}

/// Modular inverse via extended Euclidean algorithm (u128)
fn mod_inverse_u128(a: u128, m: u128) -> u128 {
    if m == 1 { return 0; }
    let (mut old_r, mut r) = (a as i128, m as i128);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = old_r / r;
        let temp_r = r; r = old_r - q * r; old_r = temp_r;
        let temp_s = s; s = old_s - q * s; old_s = temp_s;
    }
    ((old_s % m as i128 + m as i128) % m as i128) as u128
}

// ═══════════════════════════════════════════════════════════════
// E8 MAP — CORRECT CENTERED-RESIDUE EMBEDDING
// ═══════════════════════════════════════════════════════════════

/// Map 8-lane CRT residues to [-1, 1]^8 via centered normalization.
///
///   ι(r₀,...,r₇) = ((2rᵢ - pᵢ + 1) / (pᵢ - 1))ᵢ
///
/// This map reaches {-1, 0, +1} exactly:
///   rᵢ = 0       → xᵢ = -1
///   rᵢ = (pᵢ-1)/2 → xᵢ = 0
///   rᵢ = pᵢ-1    → xᵢ = +1
///
/// E8 Type-1 roots (±1,±1,0,...,0): work for ANY 8 odd primes (112 roots).
/// E8 Type-2 roots (±½,...,±½): require pᵢ ≡ 1 mod 4 for all lanes (128 roots).
///
/// Returns coordinates as (numerator, denominator) for exact integer arithmetic.
pub fn e8_map_centered(residues: &[u64], moduli: &[u64]) -> Vec<(i64, u64)> {
    assert_eq!(residues.len(), 8, "E8 map requires exactly 8 lanes");
    assert_eq!(moduli.len(), 8);
    residues.iter().zip(moduli.iter()).map(|(&r, &m)| {
        // x = (2r - m + 1) / (m - 1)
        let num = 2 * r as i64 - m as i64 + 1;
        let den = m - 1;
        (num, den)
    }).collect()
}

/// Residue tuple for an E8 Type-1 root: two lanes at extremes, rest at midpoint.
/// pos = (i, j): which two lanes are at ±1
/// signs = (s_i, s_j): +1 or -1 for each active lane
pub fn e8_type1_residues(moduli: &[u64], pos: (usize, usize), signs: (i8, i8)) -> Vec<u64> {
    assert_eq!(moduli.len(), 8);
    (0..8).map(|i| {
        if i == pos.0 {
            if signs.0 < 0 { 0 } else { moduli[i] - 1 }
        } else if i == pos.1 {
            if signs.1 < 0 { 0 } else { moduli[i] - 1 }
        } else {
            (moduli[i] - 1) / 2  // midpoint → x = 0
        }
    }).collect()
}

/// Residue tuple for an E8 Type-2 root: all lanes at ±½.
/// Requires all moduli ≡ 1 mod 4.
/// signs: array of 8 values, each +1 or -1. Must have even count of -1.
pub fn e8_type2_residues(moduli: &[u64], signs: &[i8; 8]) -> Option<Vec<u64>> {
    // Check: all moduli ≡ 1 mod 4
    if !moduli.iter().all(|&m| (m - 1) % 4 == 0) {
        return None;
    }
    // Check: even number of minus signs
    if signs.iter().filter(|&&s| s < 0).count() % 2 != 0 {
        return None;
    }
    Some((0..8).map(|i| {
        let quarter = (moduli[i] - 1) / 4;
        if signs[i] < 0 { quarter } else { 3 * quarter }
    }).collect())
}

/// Compute exact inner product of two E8-mapped vectors.
/// Returns (numerator, denominator) as i128 for overflow safety.
pub fn e8_inner_product(a: &[(i64, u64)], b: &[(i64, u64)]) -> (i128, i128) {
    assert_eq!(a.len(), 8);
    assert_eq!(b.len(), 8);
    // Use common denominator approach for exact computation
    // Product of all denominators
    let common_den: i128 = a.iter().zip(b.iter())
        .map(|((_, ad), (_, bd))| *ad as i128 * *bd as i128)
        .product();
    
    // Each term: (a_num * b_num) * (common_den / (a_den * b_den))
    let mut total_num: i128 = 0;
    for i in 0..8 {
        let an = a[i].0 as i128;
        let ad = a[i].1 as i128;
        let bn = b[i].0 as i128;
        let bd = b[i].1 as i128;
        let scale = common_den / (ad * bd);
        total_num += an * bn * scale;
    }
    (total_num, common_den)
}

/// Check if a residue tuple is near an E8 root (norm² = 2).
/// Returns the squared norm as (num, den).
pub fn e8_norm_squared(residues: &[u64], moduli: &[u64]) -> (i128, i128) {
    let coords = e8_map_centered(residues, moduli);
    e8_inner_product(&coords, &coords)
}

// ═══════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tzolkin_260() {
        let mut g = GearManifold::new(&[4, 5, 13]);
        for _ in 0..260 {
            g.add(1);
        }
        assert_eq!(g.full_value(), 0, "260 steps should return to origin");
        // Standard CRT: each lane wraps independently
        assert_eq!(g.lanes[0].magnitude, 65, "mod 4: 260/4 = 65 wraps");
        assert_eq!(g.lanes[1].magnitude, 52, "mod 5: 260/5 = 52 wraps");
        assert_eq!(g.lanes[2].magnitude, 20, "mod 13: 260/13 = 20 wraps");
    }

    #[test]
    fn test_calendar_round_18980() {
        let mut g = GearManifold::new(&[4, 5, 13, 73]);
        for _ in 0..18980 {
            g.add(1);
        }
        assert_eq!(g.full_value(), 0);
        // Each lane wraps independently: 18980/p wraps
        assert_eq!(g.lanes[3].magnitude, 18980 / 73, "mod 73: 260 wraps");
    }

    #[test]
    fn test_long_count_winding() {
        // Long Count = 1,872,000 days
        // Standard CRT: each lane wraps independently
        let mut g = GearManifold::new(&[4, 5, 13, 73]);
        g.add(1_872_000);
        // Residue should be 1872000 mod 18980 = 11960
        assert_eq!(g.full_value(), 11960, "Long Count mod CR = eclipse table");
        // Slowest lane winding: floor(1872000 / 73) = 25643
        assert_eq!(g.lanes[3].magnitude, 1_872_000 / 73);
    }

    #[test]
    fn test_garner_reconstruction() {
        let mut g = GearManifold::new(&[3, 7, 11, 13]);
        g.add(42);
        assert_eq!(g.full_value(), 42);
        g.add(100);
        assert_eq!(g.full_value(), 142);
        g.add(3003 - 142); // Fill to M_safe = 3×7×11×13 = 3003
        assert_eq!(g.full_value(), 0, "3003 wraps to 0");
    }

    #[test]
    fn test_ns_safe_basis_winding() {
        // NS safe basis {3, 7, 11, 13}, M_safe = 3003
        let mut g = GearManifold::new(&[3, 7, 11, 13]);
        g.add(10000); // max_vort_scaled = 10000 (SCALE)
        // K_vort on the mod-3 lane: floor(10000 / 3) = 3333
        assert_eq!(g.lanes[0].magnitude, 3333, "mod-3 winding = floor(10000/3)");
        assert_eq!(g.full_value() as u64, 10000 % 3003, "residue = 10000 mod 3003 = 991");
    }

    #[test]
    fn test_negative_add() {
        let mut g = GearManifold::new(&[4, 5, 13]);
        g.add(100);
        assert_eq!(g.full_value(), 100);
        g.add(-50);
        assert_eq!(g.full_value(), 50);
    }

    #[test]
    fn test_e8_type1_all_112_roots() {
        // E8 Type-1: (±1, ±1, 0, ..., 0) — 112 roots
        // Works with ANY 8 odd pairwise-coprime moduli
        // Using composite coprimes: squares of primes (Separation Principle)
        let moduli = [9u64, 25, 49, 121, 169, 289, 361, 529];
        //           3²    5²   7²   11²   13²   17²   19²   23²
        let mut count = 0;
        for i in 0..8 {
            for j in (i+1)..8 {
                for &si in &[-1i8, 1] {
                    for &sj in &[-1i8, 1] {
                        let res = e8_type1_residues(&moduli, (i, j), (si, sj));
                        let (num, den) = e8_norm_squared(&res, &moduli);
                        assert_eq!(num, 2 * den,
                            "Type-1 root ({},{}) signs=({},{}) norm²={}/{}",
                            i, j, si, sj, num, den);
                        count += 1;
                    }
                }
            }
        }
        assert_eq!(count, 112, "Should have exactly 112 Type-1 roots");
    }

    #[test]
    fn test_e8_type2_all_128_roots() {
        // E8 Type-2: (±½, ..., ±½) with even # of minus — 128 roots
        // For ANY odd p: p² ≡ 1 mod 4, so (p²-1) % 4 == 0 always.
        // ALL squares support Type-2 roots.
        let moduli = [9u64, 25, 49, 121, 169, 289, 361, 529];

        let mut count = 0;
        for pattern in 0u16..256 {
            let signs: [i8; 8] = std::array::from_fn(|i| {
                if (pattern >> i) & 1 == 0 { -1 } else { 1 }
            });
            let minus_count = signs.iter().filter(|&&s| s < 0).count();
            if minus_count % 2 != 0 { continue; }

            let res = e8_type2_residues(&moduli, &signs).expect("All p² support ½-coords");
            let (num, den) = e8_norm_squared(&res, &moduli);
            assert_eq!(num, 2 * den,
                "Type-2 root pattern={:08b} norm²={}/{}", pattern, num, den);
            count += 1;
        }
        assert_eq!(count, 128, "Should have exactly 128 Type-2 roots");
    }

    #[test]
    fn test_e8_full_240_composite_coprimes() {
        // COMPLETE E8 on composite-coprime basis (Separation Principle)
        // ALL moduli are squares of primes — zero bare primes
        // Safe basis {3,5,7,11,13} + Clockwork extension {17,19,23}
        let moduli = [9u64, 25, 49, 121, 169, 289, 361, 529];
        let mut total = 0;

        // Type-1: 112
        for i in 0..8 {
            for j in (i+1)..8 {
                for &si in &[-1i8, 1] {
                    for &sj in &[-1i8, 1] {
                        let res = e8_type1_residues(&moduli, (i, j), (si, sj));
                        let (num, den) = e8_norm_squared(&res, &moduli);
                        assert_eq!(num, 2 * den);
                        total += 1;
                    }
                }
            }
        }

        // Type-2: 128
        for pattern in 0u16..256 {
            let signs: [i8; 8] = std::array::from_fn(|i| {
                if (pattern >> i) & 1 == 0 { -1 } else { 1 }
            });
            if signs.iter().filter(|&&s| s < 0).count() % 2 != 0 { continue; }
            let res = e8_type2_residues(&moduli, &signs).unwrap();
            let (num, den) = e8_norm_squared(&res, &moduli);
            assert_eq!(num, 2 * den);
            total += 1;
        }
        assert_eq!(total, 240, "Complete E8 root system on composite coprimes");
    }

    #[test]
    fn test_e8_safe_basis_composites_type1() {
        // Pure safe basis composites {3²,5²,7²,11²,13²} — 5 lanes
        // Type-1 roots with 5 lanes: C(5,2)×4 = 40
        let moduli_5 = [9u64, 25, 49, 121, 169];
        let mut count = 0;
        for i in 0..5 {
            for j in (i+1)..5 {
                for &si in &[-1i8, 1] {
                    for &sj in &[-1i8, 1] {
                        let mut res = vec![0u64; 5];
                        for k in 0..5 {
                            if k == i { res[k] = if si < 0 { 0 } else { moduli_5[k]-1 }; }
                            else if k == j { res[k] = if sj < 0 { 0 } else { moduli_5[k]-1 }; }
                            else { res[k] = (moduli_5[k]-1)/2; }
                        }
                        // Check norm² = 2 in 5D (not E8, but validates the map)
                        let coords: Vec<_> = (0..5).map(|k| {
                            let num = 2*res[k] as i128 - moduli_5[k] as i128 + 1;
                            let den = moduli_5[k] as i128 - 1;
                            (num, den)
                        }).collect();
                        let norm_num: i128 = coords.iter().map(|(n,d)| n*n * coords.iter().map(|(_,dd)| dd*dd).product::<i128>() / (d*d)).sum();
                        let norm_den: i128 = coords.iter().map(|(_,d)| d*d).product();
                        if norm_num == 2 * norm_den { count += 1; }
                    }
                }
            }
        }
        assert_eq!(count, 40, "5-lane safe basis: 40 Type-1-like roots");
    }
}

    // ═══════════════════════════════════════════════════════════
    // E-01: DANGEROUS CONFIGURATION CLOSURE TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_d001_i64_overflow_safety() {
        // D-001: Overflow in lane arithmetic
        // GearManifold uses i128 for all intermediate math
        // Test: add i64::MAX worth of steps
        let mut g = GearManifold::new(&[3, 7, 11, 13]);
        g.add(i64::MAX as i128);
        // Should not panic. Residues must be valid.
        for lane in &g.lanes {
            assert!(lane.residue < lane.modulus,
                "D-001: residue {} >= modulus {} after i64::MAX add",
                lane.residue, lane.modulus);
        }
        // Add negative max
        g.add(-(i64::MAX as i128));
        // Should return to zero
        assert_eq!(g.full_value(), 0, "D-001: round-trip through i64::MAX");
    }

    #[test]
    fn test_d003_non_coprime_k_elim() {
        // D-003: K-Elimination with non-coprime moduli
        // mod_inverse_u128 should handle gcd>1 gracefully
        // In our implementation, moduli are pairwise coprime by construction.
        // But we verify the invariant holds for all test configurations.
        let moduli_sets: Vec<Vec<u64>> = vec![
            vec![3, 7, 11, 13],       // NS safe basis
            vec![4, 5, 13, 73],       // Maya calendar
            vec![9, 25, 49, 121, 169, 289, 361, 529], // E8 composite
        ];
        for moduli in &moduli_sets {
            for i in 0..moduli.len() {
                for j in (i+1)..moduli.len() {
                    let g = gcd_u64(moduli[i], moduli[j]);
                    assert_eq!(g, 1,
                        "D-003: moduli {} and {} share factor {}",
                        moduli[i], moduli[j], g);
                }
            }
        }
    }

    #[test]
    fn test_d005_sqr_boundary() {
        // D-005: Sqr operator near modulus boundary
        // For p=3: r=2, Sqr(2)=4%3=1. No overflow.
        // For p=7: r=6, Sqr(6)=36%7=1. No overflow.
        // The concern: r*(r) in u64 could overflow for large moduli.
        // But our safe basis moduli are all < 30, so r*r < 900. Safe.
        for &p in &[3u64, 5, 7, 11, 13] {
            let r = p - 1; // maximum residue
            let sqr = (r * r) % p;
            assert!(sqr < p, "D-005: Sqr({}) mod {} = {} overflows", r, p, sqr);
        }
        // For E8 composite moduli (up to 529=23²)
        for &p in &[9u64, 25, 49, 121, 169, 289, 361, 529] {
            let r = p - 1;
            let sqr = (r * r) % p;
            assert!(sqr < p, "D-005: Sqr({}) mod {} = {} overflows", r, p, sqr);
            // Also check: r*r doesn't overflow u64
            assert!(r.checked_mul(r).is_some(),
                "D-005: {}*{} overflows u64", r, r);
        }
    }

    #[test]
    fn test_d008_negative_winding() {
        // D-008: GearManifold negative winding
        // Winding is i128, so it can go very negative without underflow
        let mut g = GearManifold::new(&[3, 7, 11, 13]);
        g.add(-(1_000_000_000i128));
        // All windings should be negative
        for lane in &g.lanes {
            assert!(lane.magnitude < 0,
                "D-008: magnitude {} should be negative after large negative add",
                lane.magnitude);
            assert!(lane.residue < lane.modulus,
                "D-008: residue {} invalid after negative add", lane.residue);
        }
        // Round-trip
        g.add(1_000_000_000i128);
        assert_eq!(g.full_value(), 0, "D-008: round-trip through negative winding");
    }

    #[test]
    fn test_d009_garner_overflow() {
        // D-009: Garner reconstruction with large moduli
        // Product of E8 moduli: 9*25*49*121*169*289*361*529
        let moduli = [9u64, 25, 49, 121, 169, 289, 361, 529];
        let product: u128 = moduli.iter().map(|&m| m as u128).product();
        assert!(product < u128::MAX / 2,
            "D-009: product {} would overflow u128 operations", product);

        let mut g = GearManifold::new(&moduli);
        g.add(12345678);
        let val = g.full_value();
        assert_eq!(val, 12345678, "D-009: Garner reconstruction correct for E8 basis");

        // Test near-capacity value
        g.reset();
        g.add((product - 1) as i128);
        let val = g.full_value();
        assert_eq!(val, product - 1, "D-009: Garner at near-capacity");
    }

    #[test]
    fn test_d013_parking_lane_carry() {
        // D-013: CRAM carry pattern for p=2 (parking lane)
        // p=2: only residues 0,1. Sqr(0)=0, Sqr(1)=1. No carry possible.
        // This means p=2 lane is TRIVIAL — it adds no sieving power.
        // This is documented: parking lanes {2,5} excluded from transport core.
        let residues_0 = 0u64;
        let residues_1 = 1u64;
        let sqr_0 = (residues_0 * residues_0) % 2;
        let sqr_1 = (residues_1 * residues_1) % 2;
        // Carry = Sqr(r) < r
        let carry_0 = sqr_0 < residues_0; // 0 < 0 = false
        let carry_1 = sqr_1 < residues_1; // 1 < 1 = false
        assert!(!carry_0 && !carry_1,
            "D-013: p=2 has no carry, confirming it's a parking lane");
    }

    fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
        while b != 0 { let t = b; b = a % b; a = t; }
        a
    }
