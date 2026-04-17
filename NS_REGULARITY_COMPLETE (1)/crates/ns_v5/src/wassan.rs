//! # WASSAN Holographic Drive v2.0 — Structural Persistence
//!
//! ZERO FLOAT. Stores full RecumbentState, not flattened integers.
//!
//! ## Phase 4 Closures (Items 10-12)
//!
//! 10. STRUCTURAL STORAGE: stores RecumbentState bundles, not primitive ints.
//! 11. INGRESS/EGRESS RULES: defined for each tier entry/exit.
//! 12. DEEP PERSISTENCE INVARIANCE: tested via write/read cycles.
//!
//! ## Architecture
//!
//! WASSAN stores the COMPLETE tier geometry of each Recumbent value:
//!   - Which tier (Primary/Overflow/Hot)
//!   - The primary lane CRTBigInt value
//!   - The overflow lane CRTBigInt value (if any)
//!   - The hot lane CRTBigInt value (if any)
//!   - The overflow step counter
//!
//! Retrieval restores exact tier placement, not just numeric value.
//!
//! Active role: WASSAN belongs to the live exact carrier stack. Some of its
//! tests still touch archived 1D helpers because persistence regressions are
//! easiest to catch there, but that does not promote the 1D harness back into
//! the active theorem path.

use crate::recumbent::{Recumbent, RecumbentTier};
use qmnf_crtbigint::CRTBigInt;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════
// WASSAN RECORD — structural, not flattened
// ═══════════════════════════════════════════════════════════════════════

/// A WASSAN persistence record preserving the full lane geometry.
/// Item 10: "WASSAN should store recumbent states or CRTBigInt-backed
/// lane bundles, not just flattened primitive ints."
#[derive(Clone, Debug)]
pub struct WassanRecord {
    /// The tier this value was in when written.
    pub tier: RecumbentTier,
    /// Primary lane value (CRTBigInt).
    pub primary_value: CRTBigInt,
    /// Overflow lane value, if in OVERFLOW tier.
    pub overflow_value: Option<CRTBigInt>,
    /// Hot lane value, if in HOT tier.
    pub hot_value: Option<CRTBigInt>,
    /// Primary corridor radius.
    pub primary_radius: i64,
    /// Promotion threshold.
    pub promotion_threshold: i64,
    /// Cumulative overflow step count.
    pub overflow_steps: u64,
}

impl WassanRecord {
    /// Reconstruct the original Recumbent from stored structure.
    /// Item 11: "how retrieval restores exact tier placement."
    pub fn to_recumbent(&self) -> Recumbent {
        let mut r = Recumbent::new(0, self.primary_radius, self.promotion_threshold);

        // Restore exact tier structure, not just value
        r.primary.value = self.primary_value.clone();
        r.overflow = self.overflow_value.as_ref().map(|v| {
            crate::recumbent::OverflowLane {
                value: v.clone(),
                threshold: self.promotion_threshold,
            }
        });
        r.hot = self.hot_value.as_ref().map(|v| {
            crate::recumbent::HotLane { value: v.clone() }
        });
        r.overflow_steps = self.overflow_steps;
        r
    }

    /// Create record from a Recumbent.
    pub fn from_recumbent(rec: &Recumbent) -> Self {
        WassanRecord {
            tier: rec.tier(),
            primary_value: rec.primary.value.clone(),
            overflow_value: rec.overflow.as_ref().map(|o| o.value.clone()),
            hot_value: rec.hot.as_ref().map(|h| h.value.clone()),
            primary_radius: rec.primary_radius,
            promotion_threshold: rec.promotion_threshold,
            overflow_steps: rec.overflow_steps,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// WASSAN DRIVE — structural persistence fabric
// ═══════════════════════════════════════════════════════════════════════

/// WASSAN Holographic Drive: structural persistence.
///
/// Item 10: Stores WassanRecord (full tier geometry), not i64.
/// Item 11: Defined ingress/egress rules per tier.
/// Item 12: Deep persistence invariance tested.
pub struct WassanDrive {
    data: HashMap<String, WassanRecord>,
}

impl WassanDrive {
    pub fn new() -> Self {
        WassanDrive { data: HashMap::new() }
    }

    /// INGRESS RULE (Item 11):
    /// What enters WASSAN from each tier:
    ///   - PRIMARY: full primary lane CRTBigInt
    ///   - OVERFLOW: primary + overflow lane CRTBigInts
    ///   - HOT: primary + hot lane CRTBigInts
    /// All enter as structural WassanRecord.
    pub fn write(&mut self, key: &str, value: &Recumbent) {
        self.data.insert(key.to_string(), WassanRecord::from_recumbent(value));
    }

    /// EGRESS RULE (Item 11):
    /// How retrieval restores exact tier placement:
    ///   - Record tier is restored (PRIMARY/OVERFLOW/HOT)
    ///   - Lane values are restored as CRTBigInts
    ///   - Overflow step counter is preserved
    /// Persistence is STRUCTURAL, not just numeric.
    pub fn read(&self, key: &str) -> Option<Recumbent> {
        self.data.get(key).map(|record| record.to_recumbent())
    }

    /// Number of stored entries.
    pub fn depth(&self) -> usize { self.data.len() }

    /// Check if a key exists.
    pub fn contains(&self, key: &str) -> bool { self.data.contains_key(key) }

    /// Remove an entry.
    pub fn remove(&mut self, key: &str) -> Option<WassanRecord> {
        self.data.remove(key)
    }

    /// All stored keys.
    pub fn keys(&self) -> Vec<&String> {
        self.data.keys().collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS — Phase 4 Closure Verification
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Item 10: Structural storage ──

    #[test]
    fn wassan_stores_structural_state() {
        let mut drive = WassanDrive::new();

        // Store values in each tier
        let primary_val = Recumbent::from_ns(500);     // in corridor
        let overflow_val = Recumbent::from_ns(15000);  // in overflow
        let hot_val = Recumbent::from_ns(200_000_000); // in hot

        drive.write("primary", &primary_val);
        drive.write("overflow", &overflow_val);
        drive.write("hot", &hot_val);

        // Verify structural preservation
        let record_p = drive.data.get("primary").unwrap();
        assert_eq!(record_p.tier, RecumbentTier::Primary);
        assert!(record_p.overflow_value.is_none());
        assert!(record_p.hot_value.is_none());

        let record_o = drive.data.get("overflow").unwrap();
        assert_eq!(record_o.tier, RecumbentTier::Overflow);
        assert!(record_o.overflow_value.is_some());

        let record_h = drive.data.get("hot").unwrap();
        assert_eq!(record_h.tier, RecumbentTier::Hot);
        assert!(record_h.hot_value.is_some());
    }

    // ── Item 11: Ingress/egress rules ──

    #[test]
    fn wassan_roundtrip_preserves_value() {
        let mut drive = WassanDrive::new();
        let vals = [123i64, -456, 50000, -1_000_000, 0];
        for (i, &v) in vals.iter().enumerate() {
            drive.write(&format!("k{}", i), &Recumbent::from_ns(v));
        }
        assert_eq!(drive.depth(), 5);
        for (i, &v) in vals.iter().enumerate() {
            let r = drive.read(&format!("k{}", i)).unwrap();
            assert_eq!(r.reconstruct_exact(), v,
                "WASSAN roundtrip failed at {}", v);
        }
    }

    #[test]
    fn wassan_roundtrip_preserves_tier() {
        let mut drive = WassanDrive::new();

        let primary_val = Recumbent::from_ns(500);
        let overflow_val = Recumbent::from_ns(15000);

        drive.write("p", &primary_val);
        drive.write("o", &overflow_val);

        let rp = drive.read("p").unwrap();
        let ro = drive.read("o").unwrap();

        assert_eq!(rp.tier(), RecumbentTier::Primary, "tier must be preserved");
        assert_eq!(ro.tier(), RecumbentTier::Overflow, "tier must be preserved");
    }

    #[test]
    fn wassan_roundtrip_preserves_overflow_steps() {
        let mut drive = WassanDrive::new();
        let mut r = Recumbent::from_ns(0);
        r.push(20000); // go outside corridor
        r.push(-10000); // still outside
        assert!(r.overflow_steps > 0);
        let expected_steps = r.overflow_steps;

        drive.write("cell", &r);
        let restored = drive.read("cell").unwrap();
        assert_eq!(restored.overflow_steps, expected_steps,
            "overflow_steps must survive WASSAN roundtrip");
    }

    // ── Item 12: Deep persistence invariance ──
    //
    // These final tests intentionally include archived 1D steps because they are
    // cheap and brutal on serialization symmetry. They are regression pressure,
    // not theorem evidence.

    #[test]
    fn wassan_deep_write_read_cycles() {
        let mut drive = WassanDrive::new();
        let vals = [123i64, -456, 50000, -1_000_000, 0, 9999, -9999, 10001, -10001];

        // Write all
        for (i, &v) in vals.iter().enumerate() {
            drive.write(&format!("cell_{}", i), &Recumbent::from_ns(v));
        }

        // 100 write/read cycles
        for cycle in 0..100 {
            for (i, &v) in vals.iter().enumerate() {
                let key = format!("cell_{}", i);
                let r = drive.read(&key).unwrap();
                assert_eq!(r.reconstruct_exact(), v,
                    "deep cycle {}: value corruption at cell_{}", cycle, i);
                // Re-write after read
                drive.write(&key, &r);
            }
        }
    }

    #[test]
    fn wassan_deep_promotion_state_preserved() {
        let mut drive = WassanDrive::new();

        // Value that's in overflow tier
        let r = Recumbent::from_ns(25000);
        assert_eq!(r.tier(), RecumbentTier::Overflow);

        drive.write("excursion", &r);

        // 100 cycles: tier must not change
        for cycle in 0..100 {
            let restored = drive.read("excursion").unwrap();
            assert_eq!(restored.tier(), RecumbentTier::Overflow,
                "cycle {}: tier corruption", cycle);
            assert_eq!(restored.reconstruct_exact(), 25000,
                "cycle {}: value corruption", cycle);
            drive.write("excursion", &restored);
        }
    }

    #[test]
    fn wassan_deep_archival_reload_exactness() {
        let mut drive = WassanDrive::new();

        // Create a chain: evolve, archive, reload, evolve more
        let n = 16usize;
        let mut omega: Vec<Recumbent> = (0..n).map(|i| {
            Recumbent::from_ns(qmnf_crtbigint::integer_sine(i, n, 10_000))
        }).collect();

        // Evolve 10 steps
        for _ in 0..10 {
            omega = crate::recumbent::archived_recumbent_ns_step(&omega, n);
        }

        // Archive to WASSAN
        for (j, cell) in omega.iter().enumerate() {
            drive.write(&format!("omega_{}", j), cell);
        }

        // Reload from WASSAN
        let mut reloaded: Vec<Recumbent> = (0..n).map(|j| {
            drive.read(&format!("omega_{}", j)).unwrap()
        }).collect();

        // Values must match exactly
        for j in 0..n {
            assert_eq!(omega[j].reconstruct_exact(), reloaded[j].reconstruct_exact(),
                "archival/reload mismatch at cell {}", j);
        }

        // Evolve 10 more steps from reload
        for _ in 0..10 {
            reloaded = crate::recumbent::archived_recumbent_ns_step(&reloaded, n);
        }

        // All cells must still be finite and consistent
        for (j, cell) in reloaded.iter().enumerate() {
            let v = cell.reconstruct_exact();
            assert!(v.abs() < i64::MAX / 2,
                "post-reload evolution: blow-up at cell {}", j);
        }
    }
}
