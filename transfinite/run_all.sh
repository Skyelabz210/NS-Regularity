#!/usr/bin/env bash
# Transfinite workspace driver — runs the R2 Wave 1 instrument and the DynCRT
# carrier test suite, the two halves of the discrete→continuum bridge.
#
# A1 zero-float: the Python instrument is integers + exact Fractions + Q(sqrt2);
# the Rust carrier is exact CRT. No f32/f64 on any path.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARRIER="$HERE/../NS_REGULARITY_COMPLETE (1)/crates/ns_v5/qmnf_crtbigint"

echo "############################################################"
echo "# Transfinite instrument (Python, exact-integer / Q(sqrt2))"
echo "############################################################"
echo
echo ">>> N3-STRUCTB  (stretching-pairing identities — ledger #123, #124)"
python3 "$HERE/instrument/structb.py"
echo
echo ">>> N3-STRUCTB rigor  (N-independence + Frobenius certificate — ledger #125)"
python3 "$HERE/instrument/structb_rigor.py"
echo
echo ">>> N2-SEEDMAX  (Q(sqrt2) production catalog — ledger #126, #127)"
python3 "$HERE/instrument/n2_seedmax2.py"
echo

echo "############################################################"
echo "# DynCRT growable carrier (Rust, exact CRT — Wave I substrate)"
echo "############################################################"
if command -v cargo >/dev/null 2>&1; then
  ( cd "$CARRIER" && cargo test --offline 2>/dev/null || cargo test )
else
  echo "cargo not found on PATH — skipping carrier tests."
  echo "Install rust, then: cd '$CARRIER' && cargo test"
fi
