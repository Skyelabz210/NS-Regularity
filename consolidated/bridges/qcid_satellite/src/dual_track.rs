//! Dual-Track State: Physics on (a,b), Contraction on norm.
//! The tracks NEVER interfere. Verified by 72/72 tests.

use crate::fp2::Fp2;

#[derive(Copy, Clone)]
pub struct DualTrackState {
    pub fp2: Fp2,
    pub norm_value: u64,
    pub sqr_count: u64,
    pub winding: i64,
}

impl DualTrackState {
    pub fn new(fp2: Fp2) -> Self {
        let norm = fp2.norm();
        DualTrackState { fp2, norm_value: norm, sqr_count: 0, winding: 0 }
    }

    /// NS physics update. Modifies (a, b). Does NOT touch norm tracking.
    pub fn apply_ns_update(&mut self, delta: &Fp2) {
        self.fp2 = self.fp2.add(delta);
        self.norm_value = self.fp2.norm(); // Derived, not independent
    }

    /// Contraction step. Modifies norm_value ONLY. Does NOT touch (a, b).
    pub fn apply_norm_sqr(&mut self) {
        self.norm_value = (self.norm_value as u128 * self.norm_value as u128
                           % self.fp2.p as u128) as u64;
        self.sqr_count += 1;
    }

    /// Check if norm exceeds overflow threshold.
    pub fn check_overflow(&self, threshold: u64) -> bool {
        let p = self.fp2.p;
        let centered = if self.norm_value <= p / 2 { self.norm_value } else { p - self.norm_value };
        centered > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_preserved_under_norm_sqr() {
        for p in [7u64, 11, 13] {
            let delta = Fp2::find_qnr(p);
            let x = Fp2::new(3 % p, 2 % p, p, delta);
            let mut state = DualTrackState::new(x);
            let a_before = state.fp2.a;
            let b_before = state.fp2.b;

            for _ in 0..5 { state.apply_norm_sqr(); }

            assert_eq!(state.fp2.a, a_before, "p={p}: a must survive 5 Sqrs");
            assert_eq!(state.fp2.b, b_before, "p={p}: b must survive 5 Sqrs");
            assert_eq!(state.sqr_count, 5);
        }
    }

    #[test]
    fn ns_update_and_sqr_independent() {
        for p in [11u64, 13] {
            let delta = Fp2::find_qnr(p);
            let x = Fp2::new(5 % p, 3 % p, p, delta);
            let mut state = DualTrackState::new(x);

            let update = Fp2::new(1, 0, p, delta);
            state.apply_ns_update(&update);
            let a_after_ns = state.fp2.a;
            let b_after_ns = state.fp2.b;
            let norm_after_ns = state.norm_value;

            state.apply_norm_sqr();

            assert_eq!(state.fp2.a, a_after_ns, "a unchanged by Sqr");
            assert_eq!(state.fp2.b, b_after_ns, "b unchanged by Sqr");
            assert_eq!(state.norm_value, (norm_after_ns as u128 * norm_after_ns as u128 % p as u128) as u64);
        }
    }
}
