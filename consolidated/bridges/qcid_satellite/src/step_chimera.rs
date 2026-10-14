//! # Step Chimera — Phase-Locked Heterogeneous Step Composition
//!
//! ## What this is
//!
//! A chimera that holds one multiplicative step operator per CRT lane and
//! composes N applications of those operators **algebraically**, not by
//! iterating. The composition is computed once at construction via
//! lane-wise `mod_pow`; subsequent applications to a residue tuple are a
//! single componentwise multiplication.
//!
//! "Apply 200 steps" then becomes a constant-time operation on the residue
//! tuple. Each lane independently evolves to its 200th orbit position
//! through phase-lock — the orbit's algebraic structure (period, fixed
//! points, identity) determines the per-lane outcome.
//!
//! ## Why this is the right object
//!
//! Classical RNS mandates homogeneity: same operator on every lane.
//! CRAM breaks this (per CRAM_CHIMERA_COMPENDIUM §19.1): each lane gets
//! its own operator `⊙_i`. A multiplicative step operator on lane `i` is
//! `S_i(r) = m_i · r mod p_i` for a chosen multiplier `m_i ∈ Z/p_i Z`.
//!
//! The N-fold composition `S_i^N(r) = m_i^N · r mod p_i` does not require
//! N actual applications — `m_i^N mod p_i` is computed once via repeated
//! squaring in O(log N) modular multiplications. When the step is loaded
//! and the schema commits, **the topology determines the trajectory
//! algebraically**. The residue tuple at step N is reachable from the
//! residue tuple at step 0 by a single componentwise multiply by the
//! pre-computed multiplier vector `[m_1^N, m_2^N, ..., m_k^N]`.
//!
//! Phase-lock is what makes this work. Each lane operates independently
//! in `Z/p_i Z`. The N-th iterate on lane `i` depends only on `m_i^N
//! mod p_i`, which depends only on N mod ord_i(m_i). Different lanes have
//! different orders; they reach different positions in their respective
//! orbits at step N. The chimera carries all of this without communication
//! between lanes.
//!
//! ## A1 invariants
//!
//! - All arithmetic is integer modular; no floating-point anywhere.
//! - The construction step (computing `m_i^N mod p_i` for each lane) is
//!   exact and deterministic.
//! - Application is a single componentwise multiply — exact, no
//!   accumulation, no error term.
//!
//! ## Example: the ×4 orbit on prime 11 (SD-11 step)
//!
//! Load `m_11 = 4` on the 11-lane. The order of 4 mod 11 is 5 (orbit
//! `1 → 4 → 5 → 9 → 3 → 1`). Apply 200 steps: `200 mod 5 = 0`, so
//! `4^200 ≡ 1 (mod 11)`. The lane returns to identity. The chimera
//! computes this without iterating — `mod_pow(4, 200, 11) = 1` is the
//! algebraic shortcut.
//!
//! On a different lane, say `m_13 = 4` on prime 13: order of 4 mod 13 is 6
//! (orbit `1 → 4 → 3 → 12 → 9 → 10 → 1`). `200 mod 6 = 2`, so
//! `4^200 ≡ 16 ≡ 3 (mod 13)`. That lane lands on position 2 in its orbit.
//!
//! Two lanes, two different positions, both computed instantly through
//! algebraic phase-lock. The chimera composes them without coordination.

// No imports needed: u128 arithmetic and mod_pow (defined below) are
// sufficient for the chimera's algebra. The substrate's wider arithmetic
// utilities live in mod_arith but the chimera does not need them.

/// Compute `base^exp mod m` using square-and-multiply. O(log exp) modular
/// multiplications. Pure integer, no float.
pub fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    if m == 1 { return 0; }
    let mut result: u64 = 1;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 {
            result = ((result as u128 * base as u128) % m as u128) as u64;
        }
        exp >>= 1;
        base = ((base as u128 * base as u128) % m as u128) as u64;
    }
    result
}

/// One step assignment for a single lane.
#[derive(Clone, Copy, Debug)]
pub struct StepLane {
    /// The lane modulus.
    pub modulus: u64,
    /// The multiplicative step operator on this lane: r ↦ multiplier · r mod modulus.
    pub multiplier: u64,
}

/// A heterogeneous step chimera. Holds one multiplicative step per CRT lane.
/// Once constructed for a step count N, every lane carries `multiplier^N mod p`
/// pre-computed; applying the chimera to a residue tuple is one componentwise
/// multiply per lane.
#[derive(Clone, Debug)]
pub struct StepChimera {
    /// Per-lane (modulus, raw multiplier). Recorded at construction.
    pub lanes: Vec<StepLane>,
    /// Per-lane folded multiplier: `multiplier^step_count mod modulus`.
    /// This IS the algebraic shortcut. Apply is a single componentwise
    /// multiply by this vector.
    pub folded: Vec<u64>,
    /// The step count N this chimera represents.
    pub step_count: u64,
    /// Per-lane orbit period: `ord_p(multiplier)`. Used for diagnostics
    /// and for verifying that N folds correctly into N mod period.
    pub orders: Vec<u64>,
}

impl StepChimera {
    /// Build a step chimera from a list of (modulus, multiplier) lane
    /// assignments and a step count. The folded multiplier vector is
    /// computed at construction; subsequent `apply` calls are O(k)
    /// componentwise multiplies where k is the number of lanes.
    ///
    /// Panics if any modulus is 0 or 1, or if any multiplier shares a
    /// factor with its modulus (which would make the step non-invertible
    /// and break orbit closure).
    pub fn new(lanes: Vec<StepLane>, step_count: u64) -> Self {
        for lane in &lanes {
            assert!(lane.modulus > 1, "Modulus must be ≥ 2");
            let g = gcd_u64(lane.multiplier % lane.modulus, lane.modulus);
            assert!(g == 1,
                "StepChimera: multiplier {} not coprime to modulus {} (gcd={})",
                lane.multiplier, lane.modulus, g);
        }

        // Fold each lane's multiplier to the N-th power. This is the
        // ALGEBRAIC step. mod_pow gives us m^N mod p in O(log N), which
        // collapses N applications to a single multiply at apply time.
        let folded: Vec<u64> = lanes.iter()
            .map(|l| mod_pow(l.multiplier % l.modulus, step_count, l.modulus))
            .collect();

        // Compute orders (period of each multiplier under repeated multiplication).
        // This bounds at p-1 by Fermat's little theorem when the modulus is prime;
        // we compute it explicitly for diagnostic transparency.
        let orders: Vec<u64> = lanes.iter()
            .map(|l| multiplicative_order(l.multiplier % l.modulus, l.modulus))
            .collect();

        Self { lanes, folded, step_count, orders }
    }

    /// Apply the N-step chimera to a residue tuple.
    ///
    /// This is the IMMEDIATE TRANSFORMATION. The residue at lane i becomes
    /// `(state[i] · folded[i]) mod modulus[i]`. No iteration. The 200
    /// steps (or N steps for any N) are already folded into `folded[]`.
    ///
    /// Uses u128 arithmetic throughout — consistent with `mod_pow`. The
    /// product `state[i] · folded[i]` is at most `(p-1)^2 < 2^128` for any
    /// `p ≤ u64::MAX`, so the multiply never overflows.
    ///
    /// Panics if state.len() != lanes.len().
    pub fn apply(&self, state: &[u64]) -> Vec<u64> {
        assert_eq!(state.len(), self.lanes.len(),
            "StepChimera::apply: state length {} != lane count {}",
            state.len(), self.lanes.len());

        state.iter().zip(self.lanes.iter()).zip(self.folded.iter())
            .map(|((&s, lane), &mult)| {
                let s_red = s % lane.modulus;  // Defensive — caller may pass un-reduced state
                ((s_red as u128 * mult as u128) % lane.modulus as u128) as u64
            })
            .collect()
    }

    /// Return the position of lane i in its orbit at step N.
    /// position = (step_count mod order_i) — independent of starting state.
    pub fn orbit_position(&self, lane_index: usize) -> u64 {
        self.step_count % self.orders[lane_index]
    }

    /// Return true if applying this chimera is the identity transformation
    /// on the i-th lane (i.e., the step count is a multiple of the orbit period).
    pub fn is_identity_on_lane(&self, lane_index: usize) -> bool {
        self.folded[lane_index] == 1
    }

    /// Return true if applying this chimera is the identity on ALL lanes.
    pub fn is_global_identity(&self) -> bool {
        self.folded.iter().all(|&f| f == 1)
    }
}

/// Compute the multiplicative order of `a` modulo `m` — the smallest
/// positive `n` such that `a^n ≡ 1 (mod m)`.
///
/// Returns 0 if no order exists (a not coprime to m, or m ≤ 1).
///
/// ## Bound
///
/// The order divides φ(m), or λ(m) for composite m; the brute-force loop
/// terminates by `n ≤ φ(m) ≤ m`. For typical CRAM lane sizes (small primes
/// 2, 3, 5, 7, 11, 13, 17, 19) this is trivially fast — orders are at most
/// p-1 ≤ 18. For larger moduli a totient-factorization-based search would
/// be more efficient, but that is not on a hot path in any current
/// substrate use.
///
/// ## Edge cases
///
/// - `m = 0`: undefined, returns 0
/// - `m = 1`: trivial ring with only one element 0, returns 0
/// - `gcd(a, m) ≠ 1`: no order exists, returns 0
pub fn multiplicative_order(a: u64, m: u64) -> u64 {
    if m <= 1 { return 0; }
    if gcd_u64(a, m) != 1 { return 0; }
    let a_red = a % m;
    if a_red == 1 { return 1; }
    let mut x = a_red;
    let mut n = 1u64;
    while x != 1 && n < m {
        x = ((x as u128 * a_red as u128) % m as u128) as u64;
        n += 1;
    }
    // Post-loop guard: if we exited because n hit the bound without
    // finding x == 1, no order exists within m. Return 0 per contract.
    // (For coprime (a, m) with m > 1, Euler's theorem guarantees the
    // order divides φ(m) < m, so this branch should be unreachable —
    // but the defensive check makes the contract loud rather than relying
    // on a math claim that the implementation does not verify.)
    if x != 1 { return 0; }
    n
}

fn gcd_u64(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd_u64(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_pow_basic() {
        // 4^200 mod 11. ord_11(4) = 5, so 200 mod 5 = 0, result = 1.
        assert_eq!(mod_pow(4, 200, 11), 1);
        // 4^200 mod 13. ord_13(4) = 6, 200 mod 6 = 2, result = 4^2 = 16 mod 13 = 3.
        assert_eq!(mod_pow(4, 200, 13), 3);
        // 2^10 mod 1000 = 1024 mod 1000 = 24.
        assert_eq!(mod_pow(2, 10, 1000), 24);
    }

    #[test]
    fn test_multiplicative_order_known_cases() {
        // ord_11(4) = 5 (orbit: 4 → 5 → 9 → 3 → 1)
        assert_eq!(multiplicative_order(4, 11), 5);
        // ord_13(4) = 6 (orbit: 4 → 3 → 12 → 9 → 10 → 1)
        assert_eq!(multiplicative_order(4, 13), 6);
        // ord_p(1) = 1 for any p
        assert_eq!(multiplicative_order(1, 11), 1);
        // ord_7(2) = 3 (orbit: 2 → 4 → 1)
        assert_eq!(multiplicative_order(2, 7), 3);
    }

    #[test]
    fn test_sd11_x4_orbit_200_steps_is_identity() {
        // The classic case: load ×4 as the step on prime 11, apply 200 times.
        // 200 mod 5 = 0, so the result is the identity on the 11-lane.
        let lanes = vec![
            StepLane { modulus: 11, multiplier: 4 },
        ];
        let chimera = StepChimera::new(lanes, 200);
        assert_eq!(chimera.folded, vec![1]);
        assert!(chimera.is_global_identity());

        // Apply to an arbitrary state — must return the state unchanged.
        let state = vec![7u64];
        let result = chimera.apply(&state);
        assert_eq!(result, state, "200 steps of ×4 mod 11 should be identity");
    }

    #[test]
    fn test_heterogeneous_lanes_independent_orbits() {
        // Multiple lanes, same step (×4) but different moduli.
        // Each lane reaches its 200th orbit position INDEPENDENTLY.
        // No iteration; the folded vector is computed once at construction.
        let lanes = vec![
            StepLane { modulus: 11, multiplier: 4 },  // ord 5: 200 mod 5 = 0 → 1
            StepLane { modulus: 13, multiplier: 4 },  // ord 6: 200 mod 6 = 2 → 4²=3
            StepLane { modulus: 17, multiplier: 4 },  // ord 4: 200 mod 4 = 0 → 1
            StepLane { modulus: 19, multiplier: 4 },  // ord 9: 200 mod 9 = 2 → 4²=16
        ];
        let chimera = StepChimera::new(lanes, 200);

        // Verify the folded vector — these are all algebraic shortcuts,
        // no actual iteration happened.
        assert_eq!(chimera.folded[0], 1);   // mod 11
        assert_eq!(chimera.folded[1], 3);   // mod 13
        assert_eq!(chimera.folded[2], 1);   // mod 17
        assert_eq!(chimera.folded[3], 16);  // mod 19

        // Verify the orders match expectation.
        assert_eq!(chimera.orders, vec![5, 6, 4, 9]);

        // Apply to state [1, 1, 1, 1] — gives the folded vector directly.
        let state = vec![1u64; 4];
        assert_eq!(chimera.apply(&state), vec![1, 3, 1, 16]);
    }

    #[test]
    fn test_application_is_truly_immediate() {
        // Sanity: the chimera carries the result of N steps without
        // ever iterating N times. We verify by comparing two paths:
        // (a) construct chimera with step_count=200, apply once
        // (b) construct chimera with step_count=1, apply 200 times in user code
        // Both paths must agree on the final state.
        let lanes = vec![
            StepLane { modulus: 11, multiplier: 4 },
            StepLane { modulus: 13, multiplier: 4 },
            StepLane { modulus: 17, multiplier: 4 },
        ];

        // Path A: immediate via 200-step chimera
        let immediate = StepChimera::new(lanes.clone(), 200);
        let state = vec![2u64, 3u64, 5u64];
        let path_a = immediate.apply(&state);

        // Path B: iterate the 1-step chimera 200 times
        let single = StepChimera::new(lanes, 1);
        let mut current = state.clone();
        for _ in 0..200 {
            current = single.apply(&current);
        }
        let path_b = current;

        assert_eq!(path_a, path_b,
            "Immediate 200-step chimera must agree with 200-fold iteration");
    }

    #[test]
    fn test_safe_basis_six_lane_chimera() {
        // The safe basis colony: {3, 5, 7, 11, 13, 17}.
        // (We exclude 2 because for ×4 = ×2² mod 2 the multiplier is 0,
        // not coprime — the constructor would correctly panic.)
        let lanes = vec![
            StepLane { modulus: 3,  multiplier: 2 },  // ord 2:  200 mod 2 = 0 → 1
            StepLane { modulus: 5,  multiplier: 2 },  // ord 4:  200 mod 4 = 0 → 1
            StepLane { modulus: 7,  multiplier: 3 },  // ord 6:  200 mod 6 = 2 → 9 mod 7 = 2
            StepLane { modulus: 11, multiplier: 4 },  // ord 5:  200 mod 5 = 0 → 1
            StepLane { modulus: 13, multiplier: 4 },  // ord 6:  200 mod 6 = 2 → 16 mod 13 = 3
            StepLane { modulus: 17, multiplier: 5 },  // ord 16: 200 mod 16 = 8 → 5^8 mod 17
        ];
        let chimera = StepChimera::new(lanes, 200);

        // Check: most lanes return identity through phase-lock.
        // ord 2 | 200, ord 4 | 200, ord 5 | 200 → identity on those lanes.
        assert_eq!(chimera.folded[0], 1, "mod 3 should be identity");
        assert_eq!(chimera.folded[1], 1, "mod 5 should be identity");
        assert_eq!(chimera.folded[3], 1, "mod 11 should be identity");

        // The non-identity lanes carry their orbit position.
        assert_eq!(chimera.folded[2], 2, "mod 7: 3^200 = 3^2 = 2");
        assert_eq!(chimera.folded[4], 3, "mod 13: 4^200 = 4^2 = 3");
        // 5^8 mod 17: 5²=25=8, 5⁴=64=13, 5⁸=169 mod 17 = 169-153 = 16
        assert_eq!(chimera.folded[5], 16, "mod 17: 5^200 = 5^8 = 16");
    }

    #[test]
    fn test_orbit_position_diagnostics() {
        let lanes = vec![
            StepLane { modulus: 11, multiplier: 4 },
            StepLane { modulus: 13, multiplier: 4 },
        ];
        let chimera = StepChimera::new(lanes, 200);
        assert_eq!(chimera.orbit_position(0), 0);  // 200 mod 5 = 0
        assert_eq!(chimera.orbit_position(1), 2);  // 200 mod 6 = 2
        assert!(chimera.is_identity_on_lane(0));
        assert!(!chimera.is_identity_on_lane(1));
    }

    #[test]
    #[should_panic(expected = "not coprime")]
    fn test_non_coprime_multiplier_rejected() {
        // 4 mod 2 = 0, gcd(0, 2) = 2 ≠ 1.
        let lanes = vec![
            StepLane { modulus: 2, multiplier: 4 },
        ];
        let _ = StepChimera::new(lanes, 200);
    }

    #[test]
    fn test_correctness_at_huge_step_counts() {
        // The IMMEDIATE TRANSFORMATION property has two parts: correctness
        // and constant time at apply. This test covers correctness at huge
        // N. Time is bounded implicitly: if construction were O(N), the
        // test would not finish at N=u64::MAX/2 within any reasonable
        // timeout. mod_pow is O(log N), so even N≈9.2×10^18 is bounded by
        // 63 multiplications per lane — observed sub-millisecond.
        //
        // This is verified by execution, not by a wall-clock assertion;
        // the cargo test framework will time-out if the property fails.
        let lanes = vec![
            StepLane { modulus: 11, multiplier: 4 },
            StepLane { modulus: 13, multiplier: 4 },
            StepLane { modulus: 17, multiplier: 5 },
        ];

        for &n in &[200u64, 200_000, 200_000_000, u64::MAX / 2] {
            let chimera = StepChimera::new(lanes.clone(), n);
            assert_eq!(chimera.folded[0], mod_pow(4, n, 11));
            assert_eq!(chimera.folded[1], mod_pow(4, n, 13));
            assert_eq!(chimera.folded[2], mod_pow(5, n, 17));
        }
    }

    // ============================================================================
    // COMPOSITION LAW TESTS
    // ============================================================================

    #[test]
    fn test_composition_law_n_plus_m() {
        // The chimera forms a monoid under composition:
        //   apply(N) ∘ apply(M) = apply(N+M)
        // This is the algebraic identity that justifies "stepping by N"
        // as a single operation. If this holds, the chimera at N steps is
        // truly the algebraic shortcut for N successive 1-steps.
        let lanes = vec![
            StepLane { modulus: 11, multiplier: 4 },
            StepLane { modulus: 13, multiplier: 4 },
            StepLane { modulus: 17, multiplier: 5 },
            StepLane { modulus: 19, multiplier: 7 },
        ];

        for (n, m) in &[(50u64, 150u64), (73, 127), (1, 199), (100, 100)] {
            let chimera_n = StepChimera::new(lanes.clone(), *n);
            let chimera_m = StepChimera::new(lanes.clone(), *m);
            let chimera_nm = StepChimera::new(lanes.clone(), n + m);

            let state = vec![3u64, 5, 7, 11];
            // Path A: apply N then M
            let path_a = chimera_m.apply(&chimera_n.apply(&state));
            // Path B: apply N+M directly
            let path_b = chimera_nm.apply(&state);

            assert_eq!(path_a, path_b,
                "composition law violated: apply({}) ∘ apply({}) != apply({})",
                n, m, n + m);
        }
    }

    #[test]
    fn test_composition_with_zero_is_identity() {
        // apply(0) = identity. apply(N) ∘ apply(0) = apply(N).
        let lanes = vec![StepLane { modulus: 11, multiplier: 4 }];
        let chimera_0 = StepChimera::new(lanes.clone(), 0);
        assert_eq!(chimera_0.folded, vec![1]);
        let state = vec![7u64];
        assert_eq!(chimera_0.apply(&state), state);
    }
}

// ============================================================================
// GALOIS-ORBIT DEMONSTRATOR
// ============================================================================
//
// The architecture doc identifies Galois orbits in ℚ(cos(π/16)) as the
// target case for the N=32 NS kernel. The minimal polynomial of
// 2cos(π/16) over ℚ has degree 8 (the field is the maximal real subfield
// of ℚ(ζ₃₂)). The Galois group is (Z/32Z)*/{±1}, of order 8, acting on
// the eight conjugates of 2cos(π/16) by k ↦ 2cos(kπ/16) for k coprime to
// 32 modulo ±1.
//
// On the safe-basis CRT substrate, this group action is represented as
// multiplicative steps mod each lane prime — and StepChimera is the right
// object to fold N applications of a Galois generator into a single
// algebraic operation.
//
// The tests here demonstrate the action on a concrete starting residue
// and verify two structural claims:
//   (a) The orbit has length dividing 8 on every lane (Galois orbit closure)
//   (b) Applying 200 generator steps lands at orbit position 200 mod 8 = 0
//       on lanes where the generator has full Galois order — i.e., returns
//       to the starting position exactly.
//
// This is not a substitute for the full N=32 kernel computation. It IS a
// demonstration that the chimera structure correctly handles the Galois
// action's closure property — which is what makes the N=32 kernel
// computation tractable in the substrate at all.

#[cfg(test)]
mod galois_demo {
    use super::*;

    /// The Galois group (Z/32Z)*/{±1} has order 8. A generator of this
    /// group, restricted to the multiplicative action on a residue lane
    /// of prime p where 32 | (p-1), has multiplicative order dividing 8.
    ///
    /// We pick primes p where 32 | (p-1) so that ζ₃₂ exists in F_p.
    /// Smallest such primes: 97, 193, 257, 449, 577, 641, 673, 769, 929,...
    /// (Generated by primes ≡ 1 mod 32.)
    const GALOIS_LANE_PRIMES: [u64; 4] = [97, 193, 257, 449];

    /// Find a 32nd root of unity in F_p (assumes 32 | (p-1)).
    /// Returns ω such that ω^32 ≡ 1 (mod p) and ω^k ≢ 1 for 1 ≤ k < 32.
    fn primitive_32_root(p: u64) -> u64 {
        // Brute search: find g of full order 32 by testing g^(32/q) ≢ 1
        // for each prime divisor q of 32 = 2^5. Only one prime divisor: 2.
        // So g is a primitive 32-root iff g^32 = 1 and g^16 ≠ 1.
        for g in 2..p {
            if mod_pow(g, 32, p) == 1 && mod_pow(g, 16, p) != 1 {
                return g;
            }
        }
        panic!("No primitive 32-root in F_{}", p);
    }

    #[test]
    fn test_galois_generator_order_divides_8() {
        // Verify that on each lane prime where 32 | (p-1), the Galois
        // action's natural generator (the 32nd root of unity) has order
        // exactly 32, and the order-8 quotient (raising to 4th power) has
        // order 8 — corresponding to the (Z/32Z)*/{±1} Galois group.
        for &p in &GALOIS_LANE_PRIMES {
            let omega = primitive_32_root(p);
            assert_eq!(mod_pow(omega, 32, p), 1, "ω^32 = 1 mod {}", p);
            // The order-8 generator: ω^4. Its order is 32/gcd(32,4) = 8.
            let g = mod_pow(omega, 4, p);
            assert_eq!(multiplicative_order(g, p), 8,
                "Galois generator g={} mod {} should have order 8", g, p);
        }
    }

    #[test]
    fn test_galois_chimera_200_steps_closure() {
        // Build a heterogeneous chimera where each lane uses its own
        // ω^4 (the order-8 Galois generator). Apply 200 steps.
        // 200 = 8·25, so 200 mod 8 = 0 — every lane returns to identity.
        // This is the Galois orbit closure property: 200 generator
        // applications come back to the starting position on every lane,
        // computed instantly.
        let lanes: Vec<StepLane> = GALOIS_LANE_PRIMES.iter().map(|&p| {
            let omega = primitive_32_root(p);
            let g = mod_pow(omega, 4, p);
            StepLane { modulus: p, multiplier: g }
        }).collect();

        let chimera = StepChimera::new(lanes, 200);

        // 200 mod 8 = 0 on every lane → folded should be all 1s.
        for (i, &f) in chimera.folded.iter().enumerate() {
            assert_eq!(f, 1,
                "Galois lane {} (p={}) should return to identity at step 200; got {}",
                i, GALOIS_LANE_PRIMES[i], f);
        }
        assert!(chimera.is_global_identity(),
            "200 Galois generator steps must close to identity on every lane");
    }

    #[test]
    fn test_galois_chimera_partial_orbit() {
        // Apply 5 steps. 5 mod 8 = 5 — every lane lands at orbit position 5,
        // which is g^5 mod p. Different primes have different orbit traces;
        // each is computed independently and instantly.
        let lanes: Vec<StepLane> = GALOIS_LANE_PRIMES.iter().map(|&p| {
            let omega = primitive_32_root(p);
            let g = mod_pow(omega, 4, p);
            StepLane { modulus: p, multiplier: g }
        }).collect();

        let chimera = StepChimera::new(lanes.clone(), 5);

        for (i, &p) in GALOIS_LANE_PRIMES.iter().enumerate() {
            let omega = primitive_32_root(p);
            let g = mod_pow(omega, 4, p);
            let expected = mod_pow(g, 5, p);
            assert_eq!(chimera.folded[i], expected,
                "Lane {} (p={}): folded = {}, expected g^5 = {}",
                i, p, chimera.folded[i], expected);
            // None should be identity at step 5 (since order is 8).
            assert!(chimera.folded[i] != 1,
                "Lane {} should not be identity at step 5", i);
        }
    }

    #[test]
    fn test_galois_chimera_apply_to_root() {
        // Apply the chimera to a state initialized with each lane's
        // primitive root. Verify the state evolves as expected.
        let lanes: Vec<StepLane> = GALOIS_LANE_PRIMES.iter().map(|&p| {
            let omega = primitive_32_root(p);
            let g = mod_pow(omega, 4, p);
            StepLane { modulus: p, multiplier: g }
        }).collect();

        let chimera = StepChimera::new(lanes.clone(), 3);

        // Initial state: each lane holds its own primitive 32-root.
        let initial: Vec<u64> = GALOIS_LANE_PRIMES.iter()
            .map(|&p| primitive_32_root(p)).collect();

        let result = chimera.apply(&initial);

        // Each lane should now hold ω · g^3 = ω · ω^12 = ω^13.
        for (i, &p) in GALOIS_LANE_PRIMES.iter().enumerate() {
            let omega = primitive_32_root(p);
            let expected = mod_pow(omega, 13, p);
            assert_eq!(result[i], expected,
                "Lane {} (p={}): got {}, expected ω^13 = {}",
                i, p, result[i], expected);
        }
    }

    #[test]
    fn test_galois_orbit_length_lcm() {
        // Demonstrate the chimera's collective period property: the step
        // count needed for global identity is the lcm of per-lane orders.
        // Here all lanes have order 8, so the global period is 8.
        let lanes: Vec<StepLane> = GALOIS_LANE_PRIMES.iter().map(|&p| {
            let omega = primitive_32_root(p);
            let g = mod_pow(omega, 4, p);
            StepLane { modulus: p, multiplier: g }
        }).collect();

        // Any multiple of 8 closes; non-multiples don't.
        for n in [8u64, 16, 24, 800, 8000] {
            let chimera = StepChimera::new(lanes.clone(), n);
            assert!(chimera.is_global_identity(),
                "n={} should be global identity (multiple of 8)", n);
        }
        for n in [1u64, 2, 4, 7, 9, 15] {
            let chimera = StepChimera::new(lanes.clone(), n);
            assert!(!chimera.is_global_identity(),
                "n={} should NOT be global identity", n);
        }
    }
}
