# PROPOSAL — P5B-9A Execution

**Issued:** 2026-04-29  
**Gate:** `per_prime_spectral_residues_crt_consistent` PASS ✓  
**Status:** Ready to implement  

---

## WHAT WAS SETTLED THIS SESSION

The reviewer was correct: the proposed shortcut (seeding p5,p6 from Garner4) does not widen the spectral wall. The forbidden shortcut is now documented in a guard test.

The P5B-9A gate test (`per_prime_spectral_residues_crt_consistent`) confirms:
- Computing spectral residues for p5=1409 and p6=1601 via independent NTT in Z/p_iZ gives CRT-consistent results
- garner6(r0..r5) == garner4(r0..r3) for all cells within the 4p corridor
- 0 mismatches across all N³=32768 cells for TG amp=100

**The previous P5B-3 failure was a bug (incorrect small lane population), not a mathematical impossibility.** The math has always supported 6-prime spectral — we just had a bug in a different part of the code that was blamed on the NTT.

---

## IMPLEMENTATION PLAN

### Step 1: Generalize ntt3d_lane (30 min)

Rename `ntt3d_4p` / `intt3d_4p` to `ntt3d_lane` / `intt3d_lane` (runtime-indexed, uses `NTT_PRIMES_6`):

```rust
const SPECTRAL_LANES: usize = 6;

fn ntt3d_lane(f: &[[[i64; N]; N]; N], pi: usize) -> Box<[[[u64; N]; N]; N]>
fn intt3d_lane(fhat: &[[[u64; N]; N]; N], pi: usize) -> Box<[[[u64; N]; N]; N]>
```

Both functions already exist as `ntt3d_4p` / `intt3d_4p` in v4 and `ntt3d_ext` / `intt3d_ext` in ntt6. Consolidate into one set, indexed by `pi` in 0..6.

### Step 2: Implement biot_savart_6p_spectral (60 min)

```rust
pub fn biot_savart_6p_spectral(w: &CRTVortExt8) -> (ExtVel8, BsRawStats) {
    const LANES: usize = 6;

    // Per-prime NTT (6 primes × 3 components)
    let mut ux_rs: [Box; 6], uy_rs: [Box; 6], uz_rs: [Box; 6];

    for pi in 0..LANES {
        let p = NTT_PRIMES_6[pi];
        let i_p = I_MOD_6[pi];
        // Extract residue from CRTVortExt8 lane pi
        let wx_f = extract_lane(w.wx, pi, p);
        let wy_f = extract_lane(w.wy, pi, p);
        let wz_f = extract_lane(w.wz, pi, p);
        // Forward NTT
        let wx_hat = ntt3d_lane(&wx_f, pi);
        // ... (same as existing 4p code, extended to 6 primes)
        // Spectral cross-product
        // Inverse NTT → ux_rs[pi], uy_rs[pi], uz_rs[pi]
    }

    // Per-cell: Garner6(r0..r5) → raw_BS
    // Corridor: M_NTT_6/2 ≈ 1.13×10^18 (2.25M× larger)
    for i in 0..N { for j in 0..N { for k in 0..N {
        let r6x: [u64; 6] = std::array::from_fn(|pi| ux_rs[pi][i][j][k]);
        let raw_x = garner6_centered_i128(r6x);
        // K-Elim lift (NOT seeding from Garner4)
        // ExtRes8::from_i128(raw_x) → all 8 lanes consistent
        // trunc_div_signed_8_witnessed / N^6
    }}}
}
```

### Step 3: Add side-by-side validation test (30 min)

```rust
fn bs_6p_matches_4p_in_4p_corridor() {
    // While in 4p corridor: garner6 == garner4 for all cells
    // Confirms no regression from the generalization
}
```

### Step 4: Replace 4p spectral with 6p in ns_step_crt_v4 (15 min)

Update `biot_savart_residue_native` to call `biot_savart_6p_spectral`.  
NTT4 THIN-margin warnings should disappear.

### Step 5: Verify assault still clean (30 min)

Run `setsid ./abc_wall_assault` with MAX_STEPS=30.  
Expected: NTT4 corridor warnings gone (wall is now NTT6).  
Division contracts unchanged. Canonical dies at ~step 21 as before.

---

## EXPECTED OUTCOME

After P5B-9A:
- NTT4/raw_BS corridor: not the limiting factor (replaced by NTT6)
- Small-prime NTT6/K-Elim corridor: M_NTT_6/2 ≈ 1.13×10^18 protects moderate spectral values but not the Push 10 v4 TG amp=1000 step-15 regime (~1.3×10^20 state scale). For that regime, replace this proposal with a CRAM-safe widened construction: preserve 11 as shadow link and 2 as recumbent overflow, keep 13 as stable boundary and 19 as edge-of-chaos, then build composites/prime powers such as `3*5*7` or `7*p^k` with explicit roles. Higher/chaos-regime primes are allowed only behind a composite layer and must be identified by K-Elim on the winding number.
- THIN margin warnings: gone
- ABC assault: can run to step 30+ without spectral aliasing
- The 4p spectral basis is retired as a hard precondition

**Total estimated time:** ~2.5 hours
**Tests passing after:** 62+ (4 new validation tests)
