# Transfinite workspace — verified run log

**Run:** 2026-06-15 · `bash transfinite/run_all.sh` · clean checkout
**Toolchain:** Python 3.11.15 (stdlib only), cargo 1.94.1 / rustc 1.94.1
**A1 zero-float:** integers + exact `Fraction` + `Q(sqrt2)` (Python); exact CRT (Rust). No f32/f64 on any path.

---

## N3-STRUCTB — `structb.py` (ledger #123, #124)

```
                  seed |            <S,w> | anti(=0?) | I2 residual (=0?) | T1(=0?)
     TG-like psi amp=3 |                0 |         0 |                 0 |       0
checkerboard psi amp=5 |                0 |         0 |                 0 |       0
     delta psi amp=100 |          -810000 |         0 |                 0 |       0
         random psi #1 |           -45597 |         0 |                 0 |       0
         random psi #2 |           -88400 |         0 |                 0 |       0
 random psi #3 (large) |        142345631 |         0 |                 0 |       0

IDENTITY 1 (strain form, antisym annihilation): EXACT — all seeds
IDENTITY 2 (flux form, Leibniz-defect cocycle):  EXACT — all seeds
```

Structured seeds (TG, checkerboard) produce `<S,w>=0`; broadband seeds
produce nonzero — consistent with "broadband is the adversary."

## N3-STRUCTB rigor — `structb_rigor.py` (ledger #125)

```
N= 4 seed0: ID1=EXACT  ID2=EXACT  FrobCert=HOLDS  T1=0  tightest q²/(F²|w|⁴)=370028808/665562269
N= 4 seed1: ID1=EXACT  ID2=EXACT  FrobCert=HOLDS  T1=0
N= 4 seed2: ID1=EXACT  ID2=EXACT  FrobCert=HOLDS  T1=0
N= 8 seed0: ID1=EXACT  ID2=EXACT  FrobCert=HOLDS  T1=0  tightest q²/(F²|w|⁴)=26296384/40587399
N= 8 seed1: ID1=EXACT  ID2=EXACT  FrobCert=HOLDS  T1=0
N= 8 seed2: ID1=EXACT  ID2=EXACT  FrobCert=HOLDS  T1=0
N=16 seed0: ID1=EXACT  ID2=EXACT  FrobCert=HOLDS  T1=0  tightest q²/(F²|w|⁴)=52265088/79759309
N=16 seed1: ID1=EXACT  ID2=EXACT  FrobCert=HOLDS  T1=0
```

N-independence confirmed; Frobenius certificate `q² ≤ F²·|w|⁴` holds at
every cell, every seed, every N. Tightest observed ratio ≈ 0.66 of bound.

## N2-SEEDMAX — `n2_seedmax2.py` (ledger #126, #127)

```
rank                   seed  sign(P) sqrt2?   R_sq ≈
   1        delta (3-point)        +      Y   1.348210e-07
   2              random #2        +      Y   8.106600e-08
   3              random #1        -      Y   3.230300e-08
   4              random #3        -      Y   1.568000e-08
   5–14  TG / ABC / stripes / single modes   0  (exact integer zero)

sqrt2 content in BS output across catalog: PRESENT (Kill #128 reconfirmed)
```

Every single-mode and separable structured seed gives **exact integer-zero**
production — one mode cannot stretch itself. Broadband `delta` is worst case.

## DynCRT growable carrier — `qmnf_crtbigint` (Wave I substrate)

```
running 28 tests
test dyncrt::tests::activation_level_tracks_magnitude ... ok
test dyncrt::tests::basis_extends_unbounded ... ok
test dyncrt::tests::growing_int_no_ceiling ... ok
test dyncrt::tests::reproject_lossless ... ok
test dyncrt::tests::dyncrt_signed_matches_native ... ok
... (23 more) ...
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The transfinite-specific tests confirm: activation level `ℓ(t)` tracks
magnitude exactly, the basis extends without bound, integers grow with no
representation ceiling, and reprojection is lossless.

---

**Status:** transfinite instrument + carrier fully operational and
reproducible in-tree. R2 production port into `ns_v5`, CSTRETCH-PHYS, and
FN-T continuum passage remain open (see `README.md`).
