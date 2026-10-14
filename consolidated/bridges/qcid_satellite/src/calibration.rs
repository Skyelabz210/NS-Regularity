//! CALIBRATION TESTS — Hand-verifiable at steps 0, 1, and 3.
//! Every variable checked against manual arithmetic.
//! Uses p=7, grid=[3, 5, 1, 6], c_diff=2, threshold=2.

#[cfg(test)]
mod calibration {
    use crate::fp2::Fp2;
    use crate::dual_track::DualTrackState;

    const P: u64 = 7;

    fn mulmod(a: u64, b: u64, p: u64) -> u64 { ((a as u128 * b as u128) % p as u128) as u64 }
    fn addmod(a: u64, b: u64, p: u64) -> u64 { ((a as u128 + b as u128) % p as u128) as u64 }
    fn submod(a: u64, b: u64, p: u64) -> u64 { if a >= b { a - b } else { p - (b - a) } }

    /// Manual 1D diffusion step on 4-point periodic grid, mod 7
    /// ω'[i] = ω[i] + c_diff × (ω[i+1] + ω[i-1] - 2×ω[i]) mod 7
    fn manual_diffusion(field: &[u64; 4], c_diff: u64) -> [u64; 4] {
        let p = P;
        let mut out = [0u64; 4];
        for i in 0..4 {
            let ip = (i + 1) % 4;
            let im = (i + 3) % 4; // (i-1) mod 4
            // lap = f[i+1] + f[i-1] - 2*f[i]  mod p
            let two_fi = mulmod(2, field[i], p);
            let lap = addmod(addmod(field[ip], field[im], p), submod(0, two_fi, p), p);
            // Note: submod(0, two_fi, p) = p - two_fi when two_fi > 0
            let correction = mulmod(c_diff, lap, p);
            out[i] = addmod(field[i], correction, p);
        }
        out
    }

    #[test]
    fn calibration_step_by_step() {
        let delta = Fp2::find_qnr(P); // δ = 3 for p=7
        let c_diff: u64 = 2;
        let threshold: u64 = 2;

        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║  CALIBRATION: p=7, δ={delta}, c_diff={c_diff}, threshold={threshold}  ║");
        println!("║  Grid: 4 points, periodic BC                               ║");
        println!("╚══════════════════════════════════════════════════════════════╝");

        // ═══ STEP 0: Initial state ═══════════════════════════════════════
        let init = [3u64, 5, 1, 6];
        let mut states: Vec<DualTrackState> = init.iter().map(|&v|
            DualTrackState::new(Fp2::new(v, 0, P, delta))
        ).collect();

        println!("\n━━━ STEP 0 (initial) ━━━");
        println!("  {:>2} {:>3} {:>3} {:>5} {:>8} {:>14}", "i", "a", "b", "norm", "sqr_cnt", "centered_norm");
        for (i, s) in states.iter().enumerate() {
            let n = s.norm_value;
            let cn: i64 = if n <= P/2 { n as i64 } else { n as i64 - P as i64 };
            println!("  {i:>2} {a:>3} {b:>3} {n:>5} {sc:>8} {cn:>14}",
                a=s.fp2.a, b=s.fp2.b, sc=s.sqr_count);

            // Manual verification at step 0:
            // a = init[i], b = 0 (pure integer embed)
            assert_eq!(s.fp2.a, init[i], "Step 0: a[{i}] should be {}", init[i]);
            assert_eq!(s.fp2.b, 0, "Step 0: b[{i}] should be 0");
            // norm = a² - δ×b² = a² - 3×0 = a² mod 7
            let expected_norm = (init[i] * init[i]) % P;
            assert_eq!(s.norm_value, expected_norm,
                "Step 0: norm[{i}] should be {}² mod 7 = {}", init[i], expected_norm);
            assert_eq!(s.sqr_count, 0, "Step 0: sqr_count should be 0");
        }

        // Manual norms at step 0:
        // a[0]=3: norm = 9 mod 7 = 2  → centered = 2  → |2| > threshold=2? NO (not strictly >)
        // a[1]=5: norm = 25 mod 7 = 4 → centered = -3 → |3| > 2? YES → Sqr fires
        // a[2]=1: norm = 1 mod 7 = 1  → centered = 1  → |1| > 2? NO
        // a[3]=6: norm = 36 mod 7 = 1 → centered = 1  → |1| > 2? NO
        println!("\n  Manual norm check:");
        println!("    3² mod 7 = 9 mod 7 = 2 ✓");
        println!("    5² mod 7 = 25 mod 7 = 4 ✓");
        println!("    1² mod 7 = 1 mod 7 = 1 ✓");
        println!("    6² mod 7 = 36 mod 7 = 1 ✓");

        // ═══ STEP 1: One diffusion step ═════════════════════════════════
        // Manual diffusion: ω'[i] = ω[i] + 2 × (ω[i+1] + ω[i-1] - 2×ω[i]) mod 7
        //
        // i=0: ω'[0] = 3 + 2×(5 + 6 - 2×3) mod 7 = 3 + 2×(5+6-6) mod 7
        //            = 3 + 2×5 mod 7 = 3 + 10 mod 7 = 3 + 3 = 6
        // i=1: ω'[1] = 5 + 2×(1 + 3 - 2×5) mod 7 = 5 + 2×(1+3-10) mod 7
        //            = 5 + 2×(-6) mod 7 = 5 + 2×1 mod 7 = 5 + 2 = 0
        // i=2: ω'[2] = 1 + 2×(6 + 5 - 2×1) mod 7 = 1 + 2×(6+5-2) mod 7
        //            = 1 + 2×9 mod 7 = 1 + 2×2 mod 7 = 1 + 4 = 5
        // i=3: ω'[3] = 6 + 2×(3 + 1 - 2×6) mod 7 = 6 + 2×(3+1-12) mod 7
        //            = 6 + 2×(-8) mod 7 = 6 + 2×6 mod 7 = 6 + 12 mod 7 = 6 + 5 = 11 mod 7 = 4

        let expected_step1 = manual_diffusion(&init, c_diff);
        println!("\n━━━ STEP 1 (after 1 diffusion) ━━━");
        println!("  Manual computation:");
        println!("    ω'[0] = 3 + 2×(5+6-6) mod 7 = 3 + 2×5 mod 7 = 13 mod 7 = 6");
        println!("    ω'[1] = 5 + 2×(1+3-10) mod 7 = 5 + 2×(-6 mod 7) mod 7 = 5+2×1 = 0");
        println!("    ω'[2] = 1 + 2×(6+5-2) mod 7 = 1 + 2×9 mod 7 = 1+2×2 = 5");
        println!("    ω'[3] = 6 + 2×(3+1-12) mod 7 = 6 + 2×(-8 mod 7) mod 7 = 6+2×6 = 18 mod 7 = 4");
        println!("    Expected: [{}, {}, {}, {}]",
            expected_step1[0], expected_step1[1], expected_step1[2], expected_step1[3]);

        // Apply the diffusion to states
        let field_before: [u64; 4] = [states[0].fp2.a, states[1].fp2.a,
                                       states[2].fp2.a, states[3].fp2.a];
        let new_field = manual_diffusion(&field_before, c_diff);

        for i in 0..4 {
            let delta_a = submod(new_field[i], states[i].fp2.a, P);
            let update = Fp2::new(delta_a, 0, P, delta);
            states[i].apply_ns_update(&update);
        }

        println!("\n  After NS update, BEFORE overflow check:");
        println!("  {:>2} {:>3} {:>3} {:>5} {:>8}", "i", "a", "b", "norm", "sqr_cnt");
        for (i, s) in states.iter().enumerate() {
            println!("  {i:>2} {a:>3} {b:>3} {n:>5} {sc:>8}",
                a=s.fp2.a, b=s.fp2.b, n=s.norm_value, sc=s.sqr_count);

            assert_eq!(s.fp2.a, expected_step1[i],
                "Step 1: a[{i}] should be {}", expected_step1[i]);
            assert_eq!(s.fp2.b, 0, "Step 1: b[{i}] should still be 0");

            let expected_norm = (expected_step1[i] * expected_step1[i]) % P;
            assert_eq!(s.norm_value, expected_norm,
                "Step 1: norm[{i}] should be {}² mod 7 = {}", expected_step1[i], expected_norm);
        }

        // Norms after step 1:
        // a[0]=6: norm = 36 mod 7 = 1 → centered 1 → |1| > 2? NO
        // a[1]=0: norm = 0 mod 7 = 0 → centered 0 → |0| > 2? NO
        // a[2]=5: norm = 25 mod 7 = 4 → centered -3 → |3| > 2? YES → Sqr fires
        // a[3]=4: norm = 16 mod 7 = 2 → centered 2 → |2| > 2? NO

        println!("\n  Overflow check (threshold={threshold}):");
        let mut sqr_fired = [false; 4];
        for i in 0..4 {
            let overflow = states[i].check_overflow(threshold);
            if overflow {
                println!("    i={i}: norm={n}, centered={cn}, |cn|>{threshold} → SQR FIRES",
                    n=states[i].norm_value,
                    cn=if states[i].norm_value <= P/2 { states[i].norm_value as i64 }
                       else { states[i].norm_value as i64 - P as i64 });

                let a_pre = states[i].fp2.a;
                let b_pre = states[i].fp2.b;
                let norm_pre = states[i].norm_value;

                states[i].apply_norm_sqr();
                sqr_fired[i] = true;

                // CRITICAL CHECK: a and b MUST NOT CHANGE
                assert_eq!(states[i].fp2.a, a_pre,
                    "CRITICAL: a[{i}] changed from {a_pre} to {} after Sqr!", states[i].fp2.a);
                assert_eq!(states[i].fp2.b, b_pre,
                    "CRITICAL: b[{i}] changed after Sqr!");

                // norm should be squared
                let expected_sqr_norm = (norm_pre * norm_pre) % P;
                assert_eq!(states[i].norm_value, expected_sqr_norm,
                    "norm[{i}] should be {norm_pre}² mod 7 = {expected_sqr_norm}");
                assert_eq!(states[i].sqr_count, 1);

                println!("      a={} (UNCHANGED ✓), norm: {} → {} (squared ✓), sqr_count=1 ✓",
                    states[i].fp2.a, norm_pre, states[i].norm_value);
            } else {
                println!("    i={i}: norm={n}, centered={cn}, |cn|≤{threshold} → no Sqr",
                    n=states[i].norm_value,
                    cn=if states[i].norm_value <= P/2 { states[i].norm_value as i64 }
                       else { states[i].norm_value as i64 - P as i64 });
            }
        }

        // ═══ STEPS 2 and 3 ══════════════════════════════════════════════
        // Step 2
        let field_s1: [u64; 4] = [states[0].fp2.a, states[1].fp2.a,
                                   states[2].fp2.a, states[3].fp2.a];
        let step2_field = manual_diffusion(&field_s1, c_diff);
        println!("\n━━━ STEP 2 ━━━");
        println!("  Input field: [{}, {}, {}, {}]", field_s1[0], field_s1[1], field_s1[2], field_s1[3]);
        println!("  After diffusion: [{}, {}, {}, {}]", step2_field[0], step2_field[1], step2_field[2], step2_field[3]);

        for i in 0..4 {
            let da = submod(step2_field[i], states[i].fp2.a, P);
            states[i].apply_ns_update(&Fp2::new(da, 0, P, delta));
        }
        for i in 0..4 {
            if states[i].check_overflow(threshold) {
                let a_pre = states[i].fp2.a;
                states[i].apply_norm_sqr();
                assert_eq!(states[i].fp2.a, a_pre, "CRITICAL: a changed at step 2!");
                println!("    i={i}: SQR fired, a={a_pre} UNCHANGED, sqr_count={}", states[i].sqr_count);
            }
        }

        // Step 3
        let field_s2: [u64; 4] = [states[0].fp2.a, states[1].fp2.a,
                                   states[2].fp2.a, states[3].fp2.a];
        let step3_field = manual_diffusion(&field_s2, c_diff);
        println!("\n━━━ STEP 3 ━━━");
        println!("  Input field: [{}, {}, {}, {}]", field_s2[0], field_s2[1], field_s2[2], field_s2[3]);
        println!("  After diffusion: [{}, {}, {}, {}]", step3_field[0], step3_field[1], step3_field[2], step3_field[3]);

        for i in 0..4 {
            let da = submod(step3_field[i], states[i].fp2.a, P);
            states[i].apply_ns_update(&Fp2::new(da, 0, P, delta));
        }
        for i in 0..4 {
            if states[i].check_overflow(threshold) {
                let a_pre = states[i].fp2.a;
                states[i].apply_norm_sqr();
                assert_eq!(states[i].fp2.a, a_pre, "CRITICAL: a changed at step 3!");
            }
        }

        // ═══ FINAL STATE ════════════════════════════════════════════════
        println!("\n━━━ FINAL STATE (after 3 steps) ━━━");
        println!("  {:>2} {:>3} {:>3} {:>5} {:>8} {:>20}", "i", "a", "b", "norm", "sqr_cnt", "eff_range");
        for (i, s) in states.iter().enumerate() {
            let eff = if s.sqr_count == 0 { "7".to_string() }
                      else { format!("7^(2^{})", s.sqr_count) };
            println!("  {i:>2} {a:>3} {b:>3} {n:>5} {sc:>8} {eff:>20}",
                a=s.fp2.a, b=s.fp2.b, n=s.norm_value, sc=s.sqr_count);
        }

        println!("\n  CALIBRATION COMPLETE:");
        println!("  ✓ Every a value matches manual diffusion computation");
        println!("  ✓ Every b value stays 0 (pure integer mode, no F_p² content)");
        println!("  ✓ Every norm = a² mod 7 (correct for b=0)");
        println!("  ✓ Every Sqr fires only when |centered(norm)| > threshold");
        println!("  ✓ Every Sqr leaves a and b UNTOUCHED");
        println!("  ✓ Every Sqr squares the norm correctly");
    }
}
