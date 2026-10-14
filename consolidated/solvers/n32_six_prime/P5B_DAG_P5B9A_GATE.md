# QMNF NS Solver — Master DAG
# 2026-04-29 | P5B-8H COMPLETE | P5B-9A GATE PASS | Tests: 58/58

---

## CORRECTED ARCHITECTURE (from executive review)

### Two-Mechanism Terminology

```
4p spectral witness basis: {769, 929, 1153, 1217}
  Role: Garner4 reconstruction of raw_BS inside biot_savart_residue_native
  HARD PRECONDITION: if raw_BS exceeds M_NTT_4/2 ≈ 5.01×10^11, Garner4 ALIASES
  Current status: THIN (margins as low as 396,393 out of 501,224,098,800)

6p division/state basis: {769,929,1153,1217,1409,1601} + {2^31, 5^9}
  Role: K-Elim division quotients + ExtRes8 state representation
  Corridor: M_NTT_6/2 ≈ 1.13×10^18 (2.25M× larger)
  Current status: 11.69% used for d=S^2 (first real constraint)

FORBIDDEN SHORTCUT (now documented and tested):
  raw_p5 = Garner4_result mod 1409   ← does NOT widen the wall
  raw_p6 = Garner4_result mod 1601   ← encodes aliased integer into p5/p6
```

### The Critical Correction

The reviewer identified: seeding p5,p6 from Garner4 re-encodes the aliased integer faithfully into the new lanes. The Garner4 hard precondition is NOT replaced — it is inherited.

---

## DECISIVE P5B-9A GATE RESULT

**`per_prime_spectral_residues_crt_consistent`: PASS ✓ (0 mismatches)**

```
Test: TG amp=100
For each cell (i,j,k), compute spectral ux residues INDEPENDENTLY:
  - pi=0..3: NTT in Z/769Z, Z/929Z, Z/1153Z, Z/1217Z
  - pi=4:    NTT in Z/1409Z (p5, root ω=10)   ← NOT seeded from Garner4
  - pi=5:    NTT in Z/1601Z (p6, root ω=109)  ← NOT seeded from Garner4

Then: verify garner6(r0..r5) == garner4(r0..r3) for all cells within 4p corridor.

Result: CRT consistency = 0 mismatches, max_diff = 0
```

**What this proves:** The previous P5B-3 failure was a BUG (incorrect small lane population in ExtRes8), not a mathematical impossibility. Per-prime NTT spectral computation in Z/1409Z and Z/1601Z correctly produces `V mod p₅` and `V mod p₆` where V is the true integer raw velocity. Garner6 reconstructs V correctly.

**What this enables:** P5B-9A is implementable for the small-prime NTT6 basis. The 6-prime spectral computation widens the Biot-Savart wall from M_NTT_4/2 ≈ 5×10^11 to M_NTT_6/2 ≈ 1.13×10^18.

**Maintenance note (Push 10 v4):** TG amp=1000 reaches state magnitude ~1.3×10^20 by step 15, and the u128 verifier shows the next failure is the Biot-Savart spectral corridor. The small `{1409,1601}` extension is not enough for that regime. The next widening must stay CRAM-safe: preserve 11 as the shadow link, 2 as recumbent overflow, 13 as stable boundary, and 19 as edge-of-chaos; use composites and prime powers such as `3*5*7` or `7*p^k` when their roles are explicit. Higher/chaos-regime primes may appear only behind a composite layer, with K-Elim on the winding number used to identify the unique object even when residue views look identical.

---

## ADDITIONAL GUARDS ADDED

| Test | Result | Purpose |
|------|--------|---------|
| `garner6_near_corridor_edge` | PASS ✓ | Garner6 handles values at/beyond 4p wall correctly |
| `p5_p6_seeding_from_garner4_forbidden` | PASS ✓ | Documents and verifies the forbidden shortcut |
| `per_prime_spectral_residues_crt_consistent` | **PASS ✓** | **P5B-9A mathematical validity gate** |

---

## OPEN NODE: P5B-9A — True 6-Prime Spectral Biot-Savart

### Goal

Replace `biot_savart_residue_native` (4p spectral, hard precondition)
with `biot_savart_6p_spectral` (6p spectral, 2.25M× wider corridor).

### Required Implementation

```
1. Extend spectral lane count: 4 → 6
2. NTT root tables for p5=1409, p6=1601 (already in ns3d_n32_ntt6.rs)
3. Forward NTT for 6 primes × 3 components (wx, wy, wz)
4. Spectral cross-product for 6 primes
5. Inverse NTT for 6 primes
6. Per-cell: Garner6(r0..r5) → raw_BS (now in M_NTT_6 corridor)
7. ExtRes8::from_i128(raw_BS) → 8-lane representation
8. trunc_div_signed_8_witnessed / N^6
```

### EXPLICITLY FORBIDDEN

```rust
// FORBIDDEN: seeding p5/p6 from Garner4 result
let raw_p5 = garner4_result.rem_euclid(1409);  // NO
let raw_p6 = garner4_result.rem_euclid(1601);  // NO
```

### Acceptance Gates

1. 58/58 existing tests still pass
2. `per_prime_spectral_residues_crt_consistent`: PASS (already passing)
3. New 6p BS test: garner6(r0..r5) matches garner4(r0..r3) while in 4p corridor
4. 6p BS continues correctly after 4p corridor becomes THIN/exceeded
5. Zero float, zero i64 seam, all 5 division sites witnessed
6. NTT4 thin-margin warnings no longer fire (4p wall replaced by 6p wall)

### Generalized NTT Functions

Replace `ntt3d_4p` / `intt3d_4p` with:
```rust
fn ntt3d_lane(f: &[[[i64; N]; N]; N], pi: usize) -> Box<[[[u64; N]; N]; N]>
fn intt3d_lane(fhat: &[[[u64; N]; N]; N], pi: usize) -> Box<[[[u64; N]; N]; N]>
const SPECTRAL_LANES: usize = 6;
for pi in 0..SPECTRAL_LANES { ... }
```

### Side-by-Side Validation

During implementation: run both 4p and 6p BS, assert:
```
while 4p not THIN: assert!(garner4(r0..r3) == garner6(r0..r5))
when 4p THIN/exceeded: log as diagnostic only
```

---

## TEST COUNTS (authoritative: 58/58)

| Module | Tests |
|--------|-------|
| cram_div | 19 |
| cram_div_ext | 8 |
| ns3d_n32_ntt6 (existing) | 7 |
| ns3d_n32_ntt6 (P5B-9A prereqs) | 3 |
| assault_witness | 13 |
| ns3d_n32_crt_v3 | 2 |
| ns3d_n32_crt_v4 | 6 |
| **TOTAL** | **58** |

Zero failures. Zero float. Zero regressions.

---

## NEXT NODE PRIORITY ORDER

1. **P5B-9A** — True 6-prime spectral BS (prerequisite gate passed)
2. **P5B-8I** — Extended 30-step assault (safe after P5B-9A; currently THIN at 4p)
3. **D-024A** — Classify discrete trajectory (data available after P5B-8I)
