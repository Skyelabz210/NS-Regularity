//! # Arrow-Corrected Step Chimera
//!
//! ## What this fixes
//!
//! The base `StepChimera` (in `step_chimera.rs`) is symmetric — its
//! multiplicative algebra has no preferred direction, the step count is a
//! plain `u64`, and any (multiplier, modulus) coprime pair is accepted.
//! That construction is correct as algebra but **does not match the NS
//! substrate**, which has an arrow:
//!
//! 1. **Forward-only.** Per V10-T16 (ARROW_LIFT_V10), reverse gears `(q,p)`
//!    collapse to forward gears `(p,q)` as permutations — they add no
//!    coverage under the union architecture. Forward direction is canonical.
//! 2. **Append-only step log.** The recumbent architecture's
//!    `overflow_steps: u64` field is `+= 1`-mutated only; this gives the
//!    arrow of time at the substrate level. The chimera's step count
//!    likewise must only grow.
//! 3. **V10-T15 shadow gate.** The 2D-gated architecture filters admissible
//!    steps by `gcd(k_diff, 11) == 1`. Gate-dead steps are rejected at
//!    construction; they are not part of the chimera's domain.
//! 4. **2D, not 3D.** Reverse gears collapse (V10-T16). A step parameter
//!    is `(p, q)` with `p` from the discriminating set `{7, 11, 13}` and
//!    `q` from the probe set `{17, 19, 23, 29, 31, 41, 43, 47}`. The
//!    stable base runs through 19; larger capacity belongs in a composite
//!    or prime-power layer phase-locked through 11.
//!
//! The "account for the arrow" requirement says: the StepChimera as
//! algebra is correct but must be wrapped in a structure that enforces the
//! arrow's directionality and the gate's admissibility test.
//!
//! ## What this delivers
//!
//! `ArrowStepChimera`: a step chimera with forward-only advancement,
//! gate-checked construction, and an append-only step log. The underlying
//! algebra is the same `mod_pow`-folded multiplier, but the API:
//!
//! - Refuses construction if the step parameter is gate-dead
//! - Provides only `advance(steps_to_add)` — never a backward operation
//! - Records each advance in an append-only log (the operator signature)
//! - Carries the gate-active witness as part of the chimera's identity
//!
//! ## A1 invariants
//!
//! - All arithmetic is integer modular (matches base StepChimera)
//! - The gate test `gcd(k_diff, 11) == 1` is itself an integer comparison
//! - No floating point anywhere
//!
//! ## V10 references
//!
//! - V10-T14: For twin pairs, `k_diff` in any gear is a function of the
//!   gear only, not of the pair. Pre-filter, not per-pair.
//! - V10-T15: 2D-gated achieves 100% coverage at cap=100K with K-Elim
//!   interpretation preserved. Optimal architecture.
//! - V10-T16: Reverse gears collapse to forward gears in 2D. Optimal is
//!   2D gated forward-only.

use crate::step_chimera::{StepChimera, StepLane, mod_pow, multiplicative_order};

/// Discriminating prime set per V10 / safe-basis convention.
pub const DISC_PRIMES: [u64; 3] = [7, 11, 13];

/// Probe prime set per V10. Values above 19 are gear probes, not a raw
/// replacement for the safe-base composite/prime-power layer.
pub const EXT_PRIMES: [u64; 8] = [17, 19, 23, 29, 31, 41, 43, 47];

/// The shadow prime — the gate's pivot.
pub const SHADOW_PRIME: u64 = 11;

/// A V10-style 2D gear specification: a discriminating prime `p` from
/// `DISC_PRIMES` paired with a probe prime `q` from `EXT_PRIMES`.
/// Forward-only by construction (per V10-T16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gear {
    pub p: u64,  // discriminating prime
    pub q: u64,  // probe prime; higher capacity is layered via composites/powers
}

impl Gear {
    /// Construct a gear, asserting that p ∈ DISC_PRIMES and q ∈ EXT_PRIMES.
    /// Panics on misconfig — gear identity is structural, not dynamic.
    pub fn new(p: u64, q: u64) -> Self {
        assert!(DISC_PRIMES.contains(&p),
            "Gear: discriminating prime {} not in DISC_PRIMES = {:?}",
            p, DISC_PRIMES);
        assert!(EXT_PRIMES.contains(&q),
            "Gear: probe prime {} not in EXT_PRIMES = {:?}",
            q, EXT_PRIMES);
        Self { p, q }
    }

    /// Compute k_diff for an amplitude difference under this gear:
    ///   k_diff = (|A - B| mod q) * inv(p) mod q
    /// Per V10-T14 source (h2_closure.rs:691-695).
    pub fn k_diff(&self, amp_diff: u64) -> u64 {
        let diff_mod = amp_diff % self.q;
        let inv_p = mod_inv(self.p % self.q, self.q);
        ((diff_mod as u128 * inv_p as u128) % self.q as u128) as u64
    }

    /// V10-T15 gate test: a step is admissible iff `gcd(k_diff, 11) == 1`.
    /// Gate-active: shadow prime does NOT divide k_diff.
    /// Gate-dead: shadow prime divides k_diff (step excluded).
    pub fn gate_active(&self, amp_diff: u64) -> bool {
        let k = self.k_diff(amp_diff);
        gcd(k, SHADOW_PRIME) == 1
    }
}

/// One entry in the append-only step log. Records only the per-advance
/// dynamic state. The static gate witness (k_diff, gate_active) lives on
/// the chimera itself — it does not change once the gear is fixed (V10-T14).
#[derive(Clone, Debug)]
pub struct StepRecord {
    /// Cumulative step index — monotone (1, 2, 3, ...).
    /// Equals total_steps after this advance was applied.
    pub step_index: u64,
    /// Number of steps added in this advance (the increment, always > 0).
    pub increment: u64,
}

/// Arrow-corrected step chimera. Wraps a base `StepChimera` with:
/// 1. Forward-only step count (no `set_step_count`, only `advance`)
/// 2. Append-only step log
/// 3. V10-T15 gate-active witness (static per gear)
/// 4. 2D forward gear identity
#[derive(Clone, Debug)]
pub struct ArrowStepChimera {
    /// The gear this chimera advances under.
    pub gear: Gear,
    /// The amplitude difference defining the step (used for k_diff).
    /// Per V10-T14, k_diff is a function of (gear, amp_diff) only.
    pub amp_diff: u64,
    /// The k_diff witness for this (gear, amp_diff). Computed once at
    /// construction; static per V10-T14.
    pub k_diff: u64,
    /// The base chimera at the current step count.
    pub base: StepChimera,
    /// Append-only step log. Length grows monotonically.
    pub log: Vec<StepRecord>,
    /// Cumulative step count (sum of all increments). Mirrors
    /// `log.last().step_index` invariantly when log is non-empty.
    pub total_steps: u64,
}

impl ArrowStepChimera {
    /// Construct an arrow-corrected step chimera at zero steps.
    /// Returns `Err` on:
    /// - `GateDead`: V10-T15 rejection (gcd(k_diff, 11) != 1)
    /// - `WrongMultiplierCount`: multipliers vector length != 2
    /// - `MultiplierNotCoprime`: a multiplier shares a factor with its lane modulus
    pub fn new(gear: Gear, amp_diff: u64, multipliers: Vec<u64>) -> Result<Self, ArrowError> {
        let k = gear.k_diff(amp_diff);
        let active = gcd(k, SHADOW_PRIME) == 1;
        if !active {
            return Err(ArrowError::GateDead {
                gear,
                amp_diff,
                k_diff: k,
            });
        }
        if multipliers.len() != 2 {
            return Err(ArrowError::WrongMultiplierCount {
                expected: 2,
                got: multipliers.len(),
            });
        }
        // Check coprimality before constructing base, which would panic.
        for (i, &m_val) in multipliers.iter().enumerate() {
            let lane_mod = if i == 0 { gear.p } else { gear.q };
            if gcd(m_val % lane_mod, lane_mod) != 1 {
                return Err(ArrowError::MultiplierNotCoprime {
                    lane_index: i,
                    multiplier: m_val,
                    modulus: lane_mod,
                });
            }
        }
        let lanes = vec![
            StepLane { modulus: gear.p, multiplier: multipliers[0] },
            StepLane { modulus: gear.q, multiplier: multipliers[1] },
        ];
        let base = StepChimera::new(lanes, 0);
        Ok(Self {
            gear,
            amp_diff,
            k_diff: k,
            base,
            log: Vec::new(),
            total_steps: 0,
        })
    }

    /// Advance the chimera by `increment` forward steps. The increment
    /// must be positive — there is no "advance backward" operation. Zero
    /// increments are rejected as `Err(ZeroIncrement)` rather than panic
    /// for consistency with the other Result-returning APIs.
    /// Returns the new total step count on success.
    pub fn advance(&mut self, increment: u64) -> Result<u64, ArrowError> {
        if increment == 0 {
            return Err(ArrowError::ZeroIncrement);
        }
        let new_total = self.total_steps.checked_add(increment)
            .ok_or(ArrowError::StepOverflow)?;
        // Re-fold the base chimera to the new total.
        self.base = StepChimera::new(self.base.lanes.clone(), new_total);
        self.total_steps = new_total;
        // Append to the log. k_diff and gate_active are static; not stored.
        self.log.push(StepRecord {
            step_index: new_total,
            increment,
        });
        Ok(new_total)
    }

    /// Apply the chimera to the current 2-lane state.
    /// State must be `[r_p, r_q]` matching the gear lane order.
    pub fn apply(&self, state: &[u64]) -> Vec<u64> {
        self.base.apply(state)
    }

    /// Read the cumulative step count.
    pub fn steps(&self) -> u64 { self.total_steps }

    /// Return true iff this chimera at the current step count is the
    /// identity on both lanes (full closure).
    pub fn is_identity(&self) -> bool { self.base.is_global_identity() }

    /// Return the latest log entry, or None if no advances have occurred.
    pub fn last_record(&self) -> Option<&StepRecord> { self.log.last() }

    /// The gate is active by construction (gate-dead chimeras are rejected
    /// in `new`). This accessor is for diagnostic clarity at call sites.
    pub fn gate_active(&self) -> bool { true }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrowError {
    /// V10-T15 rejection: gate-dead, gcd(k_diff, 11) != 1.
    GateDead { gear: Gear, amp_diff: u64, k_diff: u64 },
    /// Wrong number of multipliers for the gear's lane count.
    WrongMultiplierCount { expected: usize, got: usize },
    /// A multiplier is not coprime to its lane modulus.
    MultiplierNotCoprime { lane_index: usize, multiplier: u64, modulus: u64 },
    /// `advance(0)` is rejected — arrow is strictly forward.
    ZeroIncrement,
    /// `total_steps` would overflow `u64` — caller advanced past 2^64 - 1.
    StepOverflow,
}

// ----------------------------------------------------------------------------
// Local helpers (kept in this module to avoid cross-crate dep churn).
// ----------------------------------------------------------------------------

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn mod_inv(a: u64, m: u64) -> u64 {
    // Extended Euclidean. Returns x in [0, m) with a*x ≡ 1 (mod m).
    let (mut t, mut new_t): (i128, i128) = (0, 1);
    let (mut r, mut new_r): (i128, i128) = (m as i128, a as i128);
    while new_r != 0 {
        let q = r / new_r;
        let tmp = t - q * new_t; t = new_t; new_t = tmp;
        let tmp = r - q * new_r; r = new_r; new_r = tmp;
    }
    assert!(r == 1, "mod_inv: gcd({}, {}) = {} != 1", a, m, r);
    let m_i = m as i128;
    (((t % m_i) + m_i) % m_i) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gear_k_diff_v10_t14() {
        // V10-T14 transplant from h2_closure.rs:
        //   gear (p=3, q=53)? — but 3 is not in DISC_PRIMES per the V10
        //   doc, that example was from the original B6 lift. Use actual
        //   V10 disc/ext combos.
        // Take gear (7, 17): twin pair (n, n+2):
        //   k_diff = 2 * inv(7) mod 17. inv(7) mod 17 = 5 (7*5=35=2*17+1). k=10.
        let gear = Gear::new(7, 17);
        assert_eq!(gear.k_diff(2), 10);
        // V10-T14 says k_diff is constant for twin pairs — verify by
        // taking another amp_diff = 2 starting at any base.
        // (k_diff depends only on amp_diff, not on the absolute amplitudes.)
        assert_eq!(gear.k_diff(2), 10);
    }

    #[test]
    fn test_gate_active_dead_split() {
        // Gate-active: gcd(k, 11) == 1. Gate-dead: 11 | k.
        //
        // Find a real gate-dead case. For gear (7, 41):
        //   inv(7) mod 41: 7*6 = 42 ≡ 1, so inv(7) = 6.
        // Want amp_diff a with k = a*6 mod 41 divisible by 11.
        //   a = 11 * inv(6) mod 41. inv(6) mod 41: 6*7=42≡1, so inv(6)=7.
        //   a = 11*7 = 77 mod 41 = 36.
        // Verify: k = 36*6 mod 41 = 216 mod 41 = 216 - 5*41 = 11.
        // gcd(11, 11) = 11 → gate-dead.
        let gear = Gear::new(7, 41);
        assert_eq!(gear.k_diff(36), 11);
        assert!(!gear.gate_active(36), "amp_diff=36 on gear(7,41) must be gate-dead");

        // For (7, 23) with amp_diff = 4: k = 4 * inv(7) mod 23.
        // inv(7) mod 23: 7*10 = 70 = 3*23+1 → inv(7)=10. k = 4*10 mod 23 = 40-23 = 17.
        // gcd(17, 11) = 1 → gate-active.
        let gear2 = Gear::new(7, 23);
        assert_eq!(gear2.k_diff(4), 17);
        assert!(gear2.gate_active(4), "amp_diff=4 on gear(7,23) must be gate-active");
    }

    #[test]
    fn test_construction_rejects_gate_dead() {
        // Use the verified gate-dead case from test_gate_active_dead_split.
        let gear = Gear::new(7, 41);
        let result = ArrowStepChimera::new(gear, 36, vec![3, 5]);
        match result {
            Err(ArrowError::GateDead { k_diff, .. }) => {
                assert_eq!(k_diff, 11);
            }
            _ => panic!("Expected GateDead error, got {:?}", result),
        }
    }

    #[test]
    fn test_construction_accepts_gate_active() {
        let gear = Gear::new(7, 23);
        let result = ArrowStepChimera::new(gear, 4, vec![3, 5]);
        assert!(result.is_ok(), "Gate-active step must be accepted");
        let chimera = result.unwrap();
        assert_eq!(chimera.steps(), 0);
        assert_eq!(chimera.log.len(), 0);
    }

    #[test]
    fn test_advance_is_forward_only() {
        let gear = Gear::new(7, 23);
        let mut c = ArrowStepChimera::new(gear, 4, vec![3, 5]).unwrap();
        c.advance(50).unwrap();
        assert_eq!(c.steps(), 50);
        c.advance(150).unwrap();
        assert_eq!(c.steps(), 200);
        // The log records both increments separately; they are NEVER merged
        // or reordered. Append-only.
        assert_eq!(c.log.len(), 2);
        assert_eq!(c.log[0].step_index, 50);
        assert_eq!(c.log[0].increment, 50);
        assert_eq!(c.log[1].step_index, 200);
        assert_eq!(c.log[1].increment, 150);
    }

    #[test]
    fn test_advance_zero_returns_error() {
        // advance(0) is rejected via Result, consistent with the rest of
        // the API. Earlier panic-based design was inconsistent with
        // GateDead returning Err.
        let gear = Gear::new(7, 23);
        let mut c = ArrowStepChimera::new(gear, 4, vec![3, 5]).unwrap();
        let result = c.advance(0);
        assert_eq!(result, Err(ArrowError::ZeroIncrement));
        // State is unchanged after rejection.
        assert_eq!(c.steps(), 0);
        assert_eq!(c.log.len(), 0);
    }

    #[test]
    fn test_advance_step_overflow_returns_error() {
        // Advancing past u64::MAX returns StepOverflow rather than panic.
        let gear = Gear::new(7, 23);
        let mut c = ArrowStepChimera::new(gear, 4, vec![3, 5]).unwrap();
        c.advance(u64::MAX - 5).unwrap();
        let result = c.advance(10);
        assert_eq!(result, Err(ArrowError::StepOverflow));
        // State unchanged.
        assert_eq!(c.steps(), u64::MAX - 5);
    }

    #[test]
    fn test_non_coprime_multiplier_returns_error() {
        // Gate-active but multiplier not coprime to lane modulus.
        // Use gear (7, 17) with amp_diff=2 (gate-active per other tests),
        // multiplier 7 on lane 7 — gcd(7, 7) = 7, not coprime.
        let gear = Gear::new(7, 17);
        let result = ArrowStepChimera::new(gear, 2, vec![7, 5]);
        match result {
            Err(ArrowError::MultiplierNotCoprime { lane_index, multiplier, modulus }) => {
                assert_eq!(lane_index, 0);
                assert_eq!(multiplier, 7);
                assert_eq!(modulus, 7);
            }
            _ => panic!("Expected MultiplierNotCoprime, got {:?}", result),
        }
    }

    #[test]
    fn test_arrow_chimera_apply_matches_base_after_advance() {
        // After advancing N steps, apply must match a fresh base StepChimera
        // built directly at N. This shows the arrow wrapper does not break
        // the underlying algebra — only adds the gate / forward-only / log.
        let gear = Gear::new(7, 23);
        let mut c = ArrowStepChimera::new(gear, 4, vec![3, 5]).unwrap();
        c.advance(200).unwrap();

        let direct = StepChimera::new(vec![
            StepLane { modulus: 7, multiplier: 3 },
            StepLane { modulus: 23, multiplier: 5 },
        ], 200);

        let state = vec![2u64, 11];
        assert_eq!(c.apply(&state), direct.apply(&state));
    }

    #[test]
    fn test_v10_t14_k_diff_invariant_under_amp_translation() {
        // V10-T14: k_diff for a twin pair (n, n+2) is constant over n.
        // Equivalently: k_diff is a function of amp_diff only, not of the
        // absolute amplitudes. Our Gear::k_diff takes amp_diff directly,
        // making this property structural.
        let gear = Gear::new(11, 41);
        let amp_diff = 2;
        assert_eq!(gear.k_diff(amp_diff), gear.k_diff(amp_diff));
        // This is trivially true for our API; the test exists to document
        // the V10-T14 invariant the API enforces by construction.
    }

    #[test]
    fn test_log_records_only_dynamic_state() {
        // The static gate witness (k_diff, gate_active) lives on the
        // chimera struct, not in the log. Each StepRecord stores only
        // step_index and increment — the dynamic state of the advance.
        // This is the redundancy fix: gate state never changes per step.
        let gear = Gear::new(7, 17);
        let c_built = ArrowStepChimera::new(gear, 2, vec![3, 5]).unwrap();
        assert_eq!(c_built.k_diff, 10);  // V10-T14 static witness
        assert!(c_built.gate_active());

        let mut c = c_built;
        c.advance(100).unwrap();
        let last = c.last_record().unwrap();
        assert_eq!(last.step_index, 100);
        assert_eq!(last.increment, 100);
        // Note: no `k_diff` or `gate_active` field on StepRecord — those
        // would be redundant since they're constant per chimera.
    }

    #[test]
    fn test_disc_ext_prime_constants_match_v10() {
        // Cross-check: the constants here match ARROW_LIFT_V10 §8.1 prime
        // sets (DISC_PRIMES from V10 discriminating set, EXT_PRIMES from
        // the V10 external set used in 2D-gated gear architecture).
        // Source: ARROW_LIFT_V10_PACKET.md §8.1 V10-T15.
        assert_eq!(DISC_PRIMES, [7u64, 11, 13]);
        assert_eq!(EXT_PRIMES, [17u64, 19, 23, 29, 31, 41, 43, 47]);
        assert_eq!(SHADOW_PRIME, 11);
    }

    #[test]
    fn test_two_hundred_step_arrow_advance() {
        // The original "step step step ... 200" requirement, now arrow-aware.
        // 200 forward steps applied via single advance. Log carries one
        // entry. Apply is the algebraic shortcut.
        let gear = Gear::new(11, 41);
        // amp_diff=2 (twin), gear (11, 41):
        //   k = 2 * inv(11) mod 41. 11*15=165=4*41+1, so inv(11)=15. k=30.
        //   gcd(30, 11) = 1 → gate-active.
        let mut c = ArrowStepChimera::new(gear, 2, vec![4, 5]).unwrap();
        c.advance(200).unwrap();

        // Lane (mod 11, ×4): order 5, 200 mod 5 = 0, returns identity.
        // Lane (mod 41, ×5): order = ord_41(5). 5^40 ≡ 1; we don't need the
        // exact order — just check via cross-validation.
        let state = vec![3u64, 7];
        let direct_immediate = c.apply(&state);

        let single = StepChimera::new(vec![
            StepLane { modulus: 11, multiplier: 4 },
            StepLane { modulus: 41, multiplier: 5 },
        ], 1);
        let mut iter_state = state.clone();
        for _ in 0..200 { iter_state = single.apply(&iter_state); }

        assert_eq!(direct_immediate, iter_state,
            "Arrow chimera at 200 steps must equal 200-fold iteration");

        // The log records one entry: step 200, increment 200.
        // Gate witness is static per-gear, not per-record (V10-T14).
        assert_eq!(c.log.len(), 1);
        assert_eq!(c.log[0].step_index, 200);
        assert_eq!(c.log[0].increment, 200);
        assert!(c.gate_active());
    }
}
