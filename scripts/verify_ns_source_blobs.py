#!/usr/bin/env python3
"""Verify immutable source-lineage copies; stdlib only."""
from pathlib import Path
import hashlib
import sys

root = Path(__file__).resolve().parents[1]
manifest = root / "consolidated" / "PROVENANCE_BLOBS.tsv"
failures = []
checked = 0
for line in manifest.read_text(encoding="utf-8").splitlines():
    if not line or line.startswith("#"):
        continue
    rel, expected = line.split("\t", 1)
    path = root / rel
    if not path.is_file():
        failures.append(f"missing: {rel}")
        continue
    data = path.read_bytes()
    header = b"blob " + str(len(data)).encode() + bytes([0])
    actual = hashlib.sha1(header + data).hexdigest()
    checked += 1
    if actual != expected:
        failures.append(f"SHA mismatch: {rel} (actual: {actual})")
print(f"Verified {checked} imported source blobs; discrepancies: {len(failures)}")
if failures:
    print("\n".join(failures))
    sys.exit(1)
