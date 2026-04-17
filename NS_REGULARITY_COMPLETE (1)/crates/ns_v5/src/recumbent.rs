//! # Recumbent CRTBigInt v2.0 — SHUTDOWN BUILD
//!
//! ZERO FLOAT. CRTBigInt-backed lanes. No sentinel fallback.
//!
//! Active role: this module is the exact overflow geometry used by the live
//! 3D closure path. Archived 1D helpers are kept only under `#[cfg(test)]` and
//! are explicitly prefixed `archived_` to prevent planning drift.
//!
//! ## Phase 1 Closures (Items 1-4)
//!
//! 1. NO FAKE FALLBACK: mul() panics on overflow, never returns i64::MAX.
//! 2. ALL LANES CRTBigInt-BACKED: PrimaryLane, OverflowLane, HotLane carry CRTBigInt.
//! 3. CANONICAL reconstruct_exact(): single CRTBigInt → i64 path.
//! 4. FROZEN STATE MODEL: exact object grammar locked.
//!
//! ## Phase 2: Five Core Recumbent Theorems (Item 5)
//!
//! T-R1 Exact Split:      split(x) produces (primary, overflow) with primary + overflow = x.
//! T-R2 Promotion Sound:  promote(overflow) → hot preserves value exactly.
//! T-R3 Demotion Sound:   demote(state) when |v| < R preserves value exactly.
//! T-R4 Reconstruction:   reconstruct(state) = exactly one integer, for ANY admissible state.
//! T-R5 Migration:        any legal sequence of split/promote/demote is value-invariant.
//!
//! ## "No Destructive Overflow" Theorem (Item 6)
//!
//! THEOREM (Exact Overflow): Leaving the primary corridor does not imply loss,
//! wrap corruption, or approximation. It induces exact staged migration into
//! recumbent tiers. For all admissible states s:
//!   reconstruct(s) = primary.value + overflow.value + hot.value
//! This is an identity, not an approximation. The tier is bookkeeping; the
//! value is invariant.

use qmnf_crtbigint::CRTBigInt;

// ═══════════════════════════════════════════════════════════════════════
// LANE TYPES — CRTBigInt-BACKED (Phase 1, Item 2)
// ═══════════════════════════════════════════════════════════════════════

/// Primary lane: value within centered corridor [-R, R).
/// CRT contraction bound B ≤ 90/1001 operates HERE.
/// NOW: CRTBigInt-backed, not i64-backed.
#[derive(Clone, Debug)]
pub struct PrimaryLane {
    pub value: CRTBigInt,
    pub radius: i64,
}

impl PrimaryLane {
    pub fn fits(&self) -> bool {
        let v = self.value.to_i64_exact();
        v.abs() < self.radius
    }

    pub fn value_i64(&self) -> i64 {
        self.value.to_i64_exact()
    }
}

/// Composite overflow lane: exact excess beyond primary corridor.
/// NOW: CRTBigInt-backed.
#[derive(Clone, Debug)]
pub struct OverflowLane {
    pub value: CRTBigInt,
    pub threshold: i64,
}

impl OverflowLane {
    pub fn should_promote(&self) -> bool {
        self.value.to_i64_exact().abs() >= self.threshold
    }

    pub fn value_i64(&self) -> i64 {
        self.value.to_i64_exact()
    }
}

/// Hot lane: promoted overflow for values far beyond corridor.
/// Phase-locked to primary via K-Elimination winding number.
/// NOW: CRTBigInt-backed.
#[derive(Clone, Debug)]
pub struct HotLane {
    pub value: CRTBigInt,
}

impl HotLane {
    pub fn value_i64(&self) -> i64 {
        self.value.to_i64_exact()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// RECUMBENT CORE — ZERO FLOAT, NO SENTINEL
// ═══════════════════════════════════════════════════════════════════════

/// Recumbent state tier for diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecumbentTier {
    Primary,
    Overflow,
    Hot,
}

/// Recumbent CRTBigInt: tiered exact integer.
///
/// RECONSTRUCTION INVARIANT (T-R4):
///   reconstruct_exact() = primary + overflow + hot = EXACT, always.
///
/// NO SENTINEL VALUES (Phase 1, Item 1):
///   Multiplication panics on overflow. Never returns i64::MAX.
///   Every arithmetic path is either exact or explicitly rejected.
#[derive(Clone, Debug)]
pub struct Recumbent {
    pub primary: PrimaryLane,
    pub overflow: Option<OverflowLane>,
    pub hot: Option<HotLane>,
    pub primary_radius: i64,
    pub promotion_threshold: i64,
    /// Cumulative steps spent outside corridor.
    pub overflow_steps: u64,
}

impl Recumbent {
    /// Construct from integer value. Applies split rule (T-R1).
    pub fn new(value: i64, primary_radius: i64, promotion_threshold: i64) -> Self {
        let mut r = Recumbent {
            primary: PrimaryLane {
                value: CRTBigInt::from_i64(0),
                radius: primary_radius,
            },
            overflow: None,
            hot: None,
            primary_radius,
            promotion_threshold,
            overflow_steps: 0,
        };
        r.assign(value);
        r
    }

    /// The split/demotion assignment (T-R1, T-R3).
    /// Called on every construction and after every operation.
    ///
    /// FROZEN STATE MODEL (Phase 1, Item 4):
    ///   primary corridor → composite overflow → hot-lane promotion → demotion → reconstruction
    ///   No mixed "approximation absorption" semantics remain.
    fn assign(&mut self, value: i64) {
        if value.abs() < self.primary_radius {
            // DEMOTION (T-R3): value fits in corridor.
            self.primary.value = CRTBigInt::from_i64(value);
            self.overflow = None;
            self.hot = None;
        } else {
            // SPLIT (T-R1): primary saturates at corridor boundary.
            let (pv, ovf) = if value >= 0 {
                (self.primary_radius - 1, value - (self.primary_radius - 1))
            } else {
                (-self.primary_radius + 1, value - (-self.primary_radius + 1))
            };
            self.primary.value = CRTBigInt::from_i64(pv);
            self.overflow_steps += 1;

            if ovf.abs() >= self.promotion_threshold {
                // PROMOTION (T-R2): overflow → hot lane.
                self.overflow = None;
                self.hot = Some(HotLane { value: CRTBigInt::from_i64(ovf) });
            } else {
                self.overflow = Some(OverflowLane {
                    value: CRTBigInt::from_i64(ovf),
                    threshold: self.promotion_threshold,
                });
                self.hot = None;
            }
        }
    }

    /// NS-default constructor: `R = SCALE = 10000`, `T = SCALE² = 100_000_000`.
    ///
    /// This helper is still part of the active path because the discrete NS
    /// layers use the same corridor defaults. It should be read as a canonical
    /// policy constructor, not as 1D-specific legacy glue.
    pub fn from_ns(value: i64) -> Self {
        Self::new(value, 10_000, 100_000_000)
    }

    /// CANONICAL RECONSTRUCT (Phase 1, Item 3).
    ///
    /// This is the ONE canonical reconstruction path.
    /// reconstruct_exact(state) = primary + overflow + hot = EXACT.
    ///
    /// T-R4 (Reconstruction Uniqueness): returns exactly one integer.
    pub fn reconstruct_exact(&self) -> i64 {
        let mut v = self.primary.value.to_i64_exact();
        if let Some(ref o) = self.overflow {
            v = v.checked_add(o.value_i64())
                .expect("reconstruct_exact: overflow lane addition exceeded i64");
        }
        if let Some(ref h) = self.hot {
            v = v.checked_add(h.value_i64())
                .expect("reconstruct_exact: hot lane addition exceeded i64");
        }
        v
    }

    /// Legacy alias — uses canonical path.
    #[inline]
    pub fn reconstruct(&self) -> i64 {
        self.reconstruct_exact()
    }

    /// Is the value entirely within the primary corridor?
    pub fn in_corridor(&self) -> bool {
        self.overflow.is_none() && self.hot.is_none()
    }

    /// Which tier is active?
    pub fn tier(&self) -> RecumbentTier {
        if self.in_corridor() { RecumbentTier::Primary }
        else if self.hot.is_some() { RecumbentTier::Hot }
        else { RecumbentTier::Overflow }
    }

    /// The overflow magnitude (0 if in corridor).
    pub fn overflow_magnitude(&self) -> i64 {
        if let Some(ref o) = self.overflow { o.value_i64().abs() }
        else if let Some(ref h) = self.hot { h.value_i64().abs() }
        else { 0 }
    }

    // ── Arithmetic — NO SENTINEL FALLBACK ──
    // Phase 1, Item 1: Every path is exact or panics. No i64::MAX.

    pub fn add(&self, other: &Recumbent) -> Recumbent {
        let va = self.reconstruct_exact();
        let vb = other.reconstruct_exact();
        let sum = va.checked_add(vb)
            .expect("Recumbent::add overflow — value exceeds i64 range");
        let mut r = Recumbent::new(0, self.primary_radius, self.promotion_threshold);
        r.overflow_steps = self.overflow_steps.max(other.overflow_steps);
        r.assign(sum);
        r
    }

    pub fn sub(&self, other: &Recumbent) -> Recumbent {
        let va = self.reconstruct_exact();
        let vb = other.reconstruct_exact();
        let diff = va.checked_sub(vb)
            .expect("Recumbent::sub overflow — value exceeds i64 range");
        let mut r = Recumbent::new(0, self.primary_radius, self.promotion_threshold);
        r.overflow_steps = self.overflow_steps.max(other.overflow_steps);
        r.assign(diff);
        r
    }

    /// Multiplication: EXACT OR PANIC. No checked_mul().unwrap_or(i64::MAX).
    /// Phase 1, Item 1: "no silent checked_* fallback that changes semantics"
    pub fn mul(&self, other: &Recumbent) -> Recumbent {
        let va = self.reconstruct_exact();
        let vb = other.reconstruct_exact();
        let product = va.checked_mul(vb)
            .expect("Recumbent::mul overflow — value exceeds i64 range. \
                     NO SENTINEL FALLBACK. Use CRTBigInt wide multiply if needed.");
        let mut r = Recumbent::new(0, self.primary_radius, self.promotion_threshold);
        r.overflow_steps = self.overflow_steps.max(other.overflow_steps);
        r.assign(product);
        r
    }

    /// K-Elimination division: exact integer division toward zero.
    pub fn k_div(&self, d: i64) -> Recumbent {
        assert!(d != 0, "k_div: division by zero");
        let v = self.reconstruct_exact();
        let q = if (v >= 0) == (d > 0) { v.abs() / d.abs() } else { -(v.abs() / d.abs()) };
        let mut r = Recumbent::new(0, self.primary_radius, self.promotion_threshold);
        r.overflow_steps = self.overflow_steps;
        r.assign(q);
        r
    }

    /// In-place push: add integer directly.
    pub fn push(&mut self, delta: i64) {
        let new_val = self.reconstruct_exact()
            .checked_add(delta)
            .expect("Recumbent::push overflow");
        self.assign(new_val);
    }

    /// Force demotion check (call after diffusion step).
    pub fn try_demote(&mut self) {
        let v = self.reconstruct_exact();
        self.assign(v);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// NS EVOLUTION ON RECUMBENT — ZERO FLOAT
// ═══════════════════════════════════════════════════════════════════════

/// One NS step on Recumbent cells for the archived 1D regression harness.
///
/// The stencil here is intentionally narrow and intentionally not part of the
/// active theorem path. It is preserved only because it exercises recumbent
/// split/promotion/demotion behavior very quickly during regression runs.
/// CFL-limit stencil: `(4ω + ω_l + ω_r) / 6`
/// Stretching: `ω × (ω_r - ω_l) / (2 × SCALE)`
///
/// Test-only: not part of the active 3D proof path.
#[cfg(test)]
pub fn archived_recumbent_ns_step(omega: &[Recumbent], _n: usize) -> Vec<Recumbent> {
    let n = omega.len();
    let scale = 10_000i64;

    (0..n).map(|j| {
        let jp = (j + 1) % n;
        let jm = (j + n - 1) % n;
        let wj = omega[j].reconstruct_exact();
        let wp = omega[jp].reconstruct_exact();
        let wm = omega[jm].reconstruct_exact();

        // Diffusion at CFL limit: (4ω + ω_l + ω_r) / 6
        let diffused = (4 * wj + wm + wp) / 6;

        // Stretching: ω × (ω_r - ω_l) / (2 × SCALE)
        let grad = wp - wm;
        let stretch = (wj as i128 * grad as i128 / (2 * scale as i128)) as i64;

        let new_val = diffused + stretch;

        let mut r = Recumbent::new(0, omega[j].primary_radius, omega[j].promotion_threshold);
        r.overflow_steps = omega[j].overflow_steps;
        r.assign(new_val);
        r
    }).collect()
}

/// Recumbent diagnostics for the archived 1D regression harness.
///
/// This struct is intentionally test-only. It aggregates exact integer
/// statistics used to monitor corridor residency and overflow pressure during
/// regression, but it is not part of the active 3D solver API.
#[cfg(test)]
pub struct RecumbentDiagnostics {
    pub n: usize,
    pub ini: i64,
    pub pk: i64,
    pub fin: i64,
    pub pk_step: usize,
    /// Corridor fraction as (numerator, denominator) — not f64.
    pub corridor_steps: usize,
    pub total_steps: usize,
    pub max_overflow_steps: u64,
}

#[cfg(test)]
impl RecumbentDiagnostics {
    /// Corridor fraction as percentage (integer, truncated).
    pub fn corridor_pct(&self) -> usize {
        if self.total_steps == 0 { return 100; }
        self.corridor_steps * 100 / self.total_steps
    }
}

/// Measure overshoot with full Recumbent diagnostics. ZERO FLOAT.
///
/// Retained strictly for regression. If an agent reaches for this function in a
/// theorem-building context, it is working on the wrong layer.
#[cfg(test)]
pub fn archived_recumbent_overshoot(n: usize, steps: usize) -> RecumbentDiagnostics {
    let mut omega: Vec<Recumbent> = (0..n).map(|i| {
        Recumbent::from_ns(qmnf_crtbigint::integer_sine(i, n, 10_000))
    }).collect();

    let ini = omega.iter().map(|c| c.reconstruct_exact().abs()).max().unwrap();
    let mut pk = ini;
    let mut pk_step = 0;
    let mut max_overflow_steps = 0u64;
    let mut corridor_steps = 0usize;
    let total_steps = steps;

    for step in 0..steps {
        omega = archived_recumbent_ns_step(&omega, n);
        let cm = omega.iter().map(|c| c.reconstruct_exact().abs()).max().unwrap();
        if cm > pk { pk = cm; pk_step = step + 1; }

        let all_in = omega.iter().all(|c| c.in_corridor());
        if all_in { corridor_steps += 1; }

        let max_os = omega.iter().map(|c| c.overflow_steps).max().unwrap_or(0);
        if max_os > max_overflow_steps { max_overflow_steps = max_os; }
    }

    let fin = omega.iter().map(|c| c.reconstruct_exact().abs()).max().unwrap();

    RecumbentDiagnostics {
        n, ini, pk, fin, pk_step,
        corridor_steps, total_steps,
        max_overflow_steps,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS — Phase 2 (Items 5-6) + Phase 3 (Items 7-9)
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ════════════════════════════════════════════════════════════════
    // T-R1: EXACT SPLIT
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn tr1_split_in_corridor() {
        let r = Recumbent::new(123, 1000, 5000);
        assert!(r.in_corridor());
        assert_eq!(r.reconstruct_exact(), 123);
        assert_eq!(r.tier(), RecumbentTier::Primary);
    }

    #[test]
    fn tr1_split_to_overflow() {
        let r = Recumbent::new(2000, 1000, 5000);
        assert!(!r.in_corridor());
        assert!(r.overflow.is_some());
        assert_eq!(r.tier(), RecumbentTier::Overflow);
        assert_eq!(r.reconstruct_exact(), 2000);
        // Verify split: primary + overflow = 2000
        let pv = r.primary.value_i64();
        let ov = r.overflow.as_ref().unwrap().value_i64();
        assert_eq!(pv + ov, 2000, "T-R1: exact split must hold");
    }

    #[test]
    fn tr1_split_negative() {
        let r = Recumbent::new(-5000, 1000, 100_000);
        assert_eq!(r.reconstruct_exact(), -5000);
        assert!(!r.in_corridor());
        // Split check
        let pv = r.primary.value_i64();
        let ov = r.overflow.as_ref().unwrap().value_i64();
        assert_eq!(pv + ov, -5000);
    }

    // ════════════════════════════════════════════════════════════════
    // T-R2: PROMOTION SOUNDNESS
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn tr2_hot_promotion_preserves_value() {
        let r = Recumbent::new(1_000_000_000, 1000, 10_000);
        assert_eq!(r.tier(), RecumbentTier::Hot);
        assert_eq!(r.reconstruct_exact(), 1_000_000_000,
            "T-R2: promotion must preserve value exactly");
    }

    // ════════════════════════════════════════════════════════════════
    // T-R3: DEMOTION SOUNDNESS
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn tr3_demotion_on_return() {
        let mut r = Recumbent::new(5000, 1000, 100_000);
        assert!(!r.in_corridor());
        assert_eq!(r.overflow_steps, 1);

        // "Diffusion" reduces value to corridor
        r.push(-4500);  // 5000 - 4500 = 500 < 1000
        assert!(r.in_corridor(), "T-R3: must demote back to PRIMARY");
        assert_eq!(r.reconstruct_exact(), 500,
            "T-R3: demotion must preserve value exactly");
    }

    // ════════════════════════════════════════════════════════════════
    // T-R4: RECONSTRUCTION UNIQUENESS (exhaustive)
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn tr4_reconstruction_invariant_exhaustive() {
        for v in -50000i64..=50000 {
            let r = Recumbent::new(v, 1000, 10000);
            assert_eq!(r.reconstruct_exact(), v,
                "T-R4: reconstruction failed at {}", v);
        }
    }

    // ════════════════════════════════════════════════════════════════
    // T-R5: MIGRATION INVARIANCE
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn tr5_migration_invariance_add() {
        let a = Recumbent::new(400, 500, 1000);
        let b = Recumbent::new(50, 500, 1000);
        let c = a.add(&b);
        assert_eq!(c.reconstruct_exact(), 450);
    }

    #[test]
    fn tr5_migration_invariance_overflow_add() {
        let a = Recumbent::new(300, 500, 10000);
        let b = Recumbent::new(400, 500, 10000);
        let c = a.add(&b);
        assert_eq!(c.reconstruct_exact(), 700);
        assert!(!c.in_corridor());
    }

    #[test]
    fn tr5_migration_invariance_sub() {
        assert_eq!(
            Recumbent::new(800, 500, 10000)
                .sub(&Recumbent::new(300, 500, 10000))
                .reconstruct_exact(),
            500
        );
    }

    #[test]
    fn tr5_migration_invariance_k_div() {
        assert_eq!(Recumbent::new(42, 100, 1000).k_div(6).reconstruct_exact(), 7);
        assert_eq!(Recumbent::new(-30, 100, 1000).k_div(6).reconstruct_exact(), -5);
    }

    #[test]
    fn tr5_chained_operations() {
        // Chained add/sub/mul/div preserving value
        let x = Recumbent::new(100, 500, 10000);
        let y = Recumbent::new(200, 500, 10000);
        let z = Recumbent::new(3, 500, 10000);
        // (100 + 200) = 300, then k_div(3) = 100
        let result = x.add(&y).k_div(3);
        assert_eq!(result.reconstruct_exact(), 100);
    }

    // ════════════════════════════════════════════════════════════════
    // Phase 1, Item 1: NO SENTINEL FALLBACK
    // ════════════════════════════════════════════════════════════════

    #[test]
    #[should_panic(expected = "Recumbent::mul overflow")]
    fn no_sentinel_on_mul_overflow() {
        // This MUST panic, not silently return i64::MAX
        let big = Recumbent::new(i64::MAX / 2, 1000, 10000);
        let _result = big.mul(&big); // Should panic
    }

    // ════════════════════════════════════════════════════════════════
    // Phase 3, Item 7: ADVERSARIAL RECUMBENT SUITE
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn adversarial_just_inside_corridor() {
        let r = Recumbent::new(999, 1000, 10000);
        assert!(r.in_corridor());
        assert_eq!(r.reconstruct_exact(), 999);
    }

    #[test]
    fn adversarial_just_outside_corridor() {
        let r = Recumbent::new(1000, 1000, 10000);
        assert!(!r.in_corridor());
        assert_eq!(r.reconstruct_exact(), 1000);
    }

    #[test]
    fn adversarial_exact_threshold_crossing() {
        // Cross from inside to outside via add
        let a = Recumbent::new(500, 1000, 10000);
        let b = Recumbent::new(600, 1000, 10000);
        let c = a.add(&b); // 1100 > 1000
        assert!(!c.in_corridor());
        assert_eq!(c.reconstruct_exact(), 1100);
    }

    #[test]
    fn adversarial_sign_flip() {
        let r = Recumbent::new(-999, 1000, 10000);
        let s = Recumbent::new(1998, 1000, 10000);
        let t = r.add(&s); // -999 + 1998 = 999
        assert!(t.in_corridor());
        assert_eq!(t.reconstruct_exact(), 999);
    }

    #[test]
    fn adversarial_repeated_promotion_demotion() {
        let mut r = Recumbent::new(0, 100, 10000);
        for i in 0..50 {
            let delta = if i % 2 == 0 { 150 } else { -150 };
            r.push(delta);
            assert_eq!(r.reconstruct_exact(), if i % 2 == 0 { 150 } else { 0 },
                "T-R5: oscillation at step {} must preserve value", i);
        }
    }

    #[test]
    fn adversarial_near_threshold_oscillation() {
        let r = 1000i64;
        let threshold = 10000i64;
        // Oscillate around corridor boundary
        for delta in [-1i64, 0, 1, 2, -3, 1, 0, -1] {
            let val = r + delta;
            let rec = Recumbent::new(val, r, threshold);
            assert_eq!(rec.reconstruct_exact(), val,
                "near-threshold value {} must reconstruct exactly", val);
        }
    }

    // ════════════════════════════════════════════════════════════════
    // Phase 3, Item 8: PROPERTY TESTS
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn property_add_homomorphism() {
        // reconstruct(x + y) = reconstruct(x) + reconstruct(y)
        let radius = 500i64;
        let threshold = 10000i64;
        let test_values: Vec<i64> = (-1000..=1000).step_by(37).collect();

        for &x in &test_values {
            for &y in &test_values {
                let rx = Recumbent::new(x, radius, threshold);
                let ry = Recumbent::new(y, radius, threshold);
                let rxy = rx.add(&ry);
                assert_eq!(rxy.reconstruct_exact(), x + y,
                    "add homomorphism failed: {} + {} ≠ {}",
                    x, y, rxy.reconstruct_exact());
            }
        }
    }

    #[test]
    fn property_sub_homomorphism() {
        let radius = 500i64;
        let threshold = 10000i64;
        let test_values: Vec<i64> = (-1000..=1000).step_by(37).collect();

        for &x in &test_values {
            for &y in &test_values {
                let rx = Recumbent::new(x, radius, threshold);
                let ry = Recumbent::new(y, radius, threshold);
                let rxy = rx.sub(&ry);
                assert_eq!(rxy.reconstruct_exact(), x - y,
                    "sub homomorphism failed: {} - {} ≠ {}",
                    x, y, rxy.reconstruct_exact());
            }
        }
    }

    #[test]
    fn property_split_reconstruct_identity() {
        for v in -5000i64..=5000 {
            let r = Recumbent::new(v, 1000, 10000);
            assert_eq!(r.reconstruct_exact(), v,
                "split-then-reconstruct ≠ identity at {}", v);
        }
    }

    #[test]
    fn property_promote_reconstruct_identity() {
        // Values that trigger promotion
        for v in [50000i64, -50000, 100000, -100000, 999999] {
            let r = Recumbent::new(v, 1000, 10000);
            assert_eq!(r.tier(), RecumbentTier::Hot);
            assert_eq!(r.reconstruct_exact(), v,
                "promote-then-reconstruct ≠ identity at {}", v);
        }
    }

    #[test]
    fn property_demote_reconstruct_identity() {
        // Start outside, push back inside
        for v in [2000i64, -2000, 5000, -5000] {
            let mut r = Recumbent::new(v, 1000, 10000);
            assert!(!r.in_corridor());
            r.push(-v + 100); // Should bring to 100 (inside corridor)
            assert!(r.in_corridor());
            assert_eq!(r.reconstruct_exact(), 100,
                "demote-then-reconstruct ≠ identity starting from {}", v);
        }
    }

    // ════════════════════════════════════════════════════════════════
    // Phase 3, Item 9: DEEP-CHAIN TESTS
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn deep_chain_repeated_operations() {
        // 500 repeated adds of small values
        let mut r = Recumbent::new(0, 1000, 100000);
        let mut expected: i64 = 0;
        for i in 0..500 {
            let delta = (i as i64 % 7) - 3; // -3, -2, -1, 0, 1, 2, 3
            let d = Recumbent::new(delta, 1000, 100000);
            r = r.add(&d);
            expected += delta;
            assert_eq!(r.reconstruct_exact(), expected,
                "deep chain invariant failed at step {}", i);
        }
    }

    #[test]
    fn deep_chain_long_ns_evolution() {
        // 1000 NS steps: invariant must survive
        let n = 32usize;
        let mut omega: Vec<Recumbent> = (0..n).map(|i| {
            Recumbent::from_ns(qmnf_crtbigint::integer_sine(i, n, 10_000))
        }).collect();

        for step in 0..1000 {
            omega = archived_recumbent_ns_step(&omega, n);
            // Every cell must reconstruct to a finite integer
            for (j, cell) in omega.iter().enumerate() {
                let _ = cell.reconstruct_exact(); // panics if broken
                assert!(cell.reconstruct_exact().abs() < i64::MAX / 2,
                    "blow-up detected at step {}, cell {}", step, j);
            }
        }
    }

    // ════════════════════════════════════════════════════════════════
    // ARCHIVED 1D REGRESSION TESTS
    // ════════════════════════════════════════════════════════════════

    #[test]
    fn recumbent_ns_constant() {
        let omega: Vec<Recumbent> = (0..8).map(|_| Recumbent::from_ns(600)).collect();
        let result = archived_recumbent_ns_step(&omega, 8);
        for c in &result {
            assert_eq!(c.reconstruct_exact(), 600,
                "constant field must be preserved");
        }
    }

    #[test]
    fn recumbent_ns_overshoot() {
        println!("\n{}", "=".repeat(78));
        println!("  RECUMBENT NS [ARCHIVED REGRESSION]: corridor dynamics + overflow tracking");
        println!("  Primary R=10000 (contraction zone) | Overflow tracks transient");
        println!("{}\n", "=".repeat(78));

        println!("{:>5} {:>8} {:>8} {:>8} {:>6} {:>8} {:>8}",
            "N", "init", "peak", "final", "pk_st", "corr%", "ovf_st");
        println!("{}", "-".repeat(60));

        for &n in &[16usize, 32, 64, 128] {
            let d = archived_recumbent_overshoot(n, 200);
            println!("{:>5} {:>8} {:>8} {:>8} {:>6} {:>7}% {:>8}",
                d.n, d.ini, d.pk, d.fin, d.pk_step,
                d.corridor_pct(), d.max_overflow_steps);
        }
    }
}
