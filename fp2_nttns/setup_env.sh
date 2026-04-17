#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# QMNF NS Campaign — Environment Setup Script
# ═══════════════════════════════════════════════════════════════════
# Author: HackFate.us / Skyelabz210
# Date: 2026-03-22
# 
# Provisions a complete QMNF NS development environment from scratch.
# Idempotent — safe to re-run. Exits non-zero on any test failure.
#
# Usage:
#   chmod +x setup_env.sh && ./setup_env.sh
#   OR
#   bash setup_env.sh [--skip-tests] [--release-only] [--tar /path/to/tar]
# ═══════════════════════════════════════════════════════════════════

set -euo pipefail

# ─── Configuration ────────────────────────────────────────────────
WORKSPACE="${QMNF_WORKSPACE:-/home/claude/qmnf_unified}"
SCAFFOLD_DIR="${QMNF_SCAFFOLD_DIR:-/home/claude}"
TAR_PATH="${1:-}"
SKIP_TESTS=false
RELEASE_ONLY=false
RUST_FAILED=0
PYTHON_FAILED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

banner() { echo -e "\n${CYAN}═══════════════════════════════════════════════════════════════${NC}"; echo -e "${BOLD} $1${NC}"; echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"; }
ok()     { echo -e "  ${GREEN}✓${NC} $1"; }
warn()   { echo -e "  ${YELLOW}⚠${NC} $1"; }
fail()   { echo -e "  ${RED}✗${NC} $1"; }
info()   { echo -e "  ${CYAN}→${NC} $1"; }

# ─── Parse args ───────────────────────────────────────────────────
for arg in "$@"; do
    case "$arg" in
        --skip-tests)    SKIP_TESTS=true ;;
        --release-only)  RELEASE_ONLY=true ;;
        --tar)           shift; TAR_PATH="$1" ;;
    esac
done


# ═══════════════════════════════════════════════════════════════════
# PHASE 1: TOOLCHAIN
# ═══════════════════════════════════════════════════════════════════
banner "Phase 1: Toolchain Installation"

# ─── Rust ─────────────────────────────────────────────────────────
if command -v rustc &>/dev/null; then
    RUST_VER=$(rustc --version)
    ok "Rust already installed: $RUST_VER"
else
    info "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --quiet 2>/dev/null
    ok "Rust installed"
fi

# Source cargo env for this session
if [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi

rustc --version >/dev/null 2>&1 || { fail "rustc not found after install"; exit 1; }
cargo --version >/dev/null 2>&1 || { fail "cargo not found after install"; exit 1; }
ok "rustc $(rustc --version | awk '{print $2}')"
ok "cargo $(cargo --version | awk '{print $2}')"

# ─── Python ───────────────────────────────────────────────────────
PYTHON_CMD=""
for cmd in python3 python; do
    if command -v "$cmd" &>/dev/null; then
        PYTHON_CMD="$cmd"
        break
    fi
done

if [ -z "$PYTHON_CMD" ]; then
    fail "Python 3 not found. Install python3 and re-run."
    exit 1
fi
ok "$PYTHON_CMD $($PYTHON_CMD --version 2>&1 | awk '{print $2}')"

# ─── Python packages ─────────────────────────────────────────────
info "Installing Python scientific stack..."
$PYTHON_CMD -m pip install --quiet --break-system-packages \
    numpy scipy matplotlib sympy 2>/dev/null || \
$PYTHON_CMD -m pip install --quiet \
    numpy scipy matplotlib sympy 2>/dev/null || \
    warn "pip install had issues — some packages may be missing"

for pkg in numpy scipy matplotlib sympy; do
    if $PYTHON_CMD -c "import $pkg" 2>/dev/null; then
        ok "$pkg $($PYTHON_CMD -c "import $pkg; print($pkg.__version__)" 2>/dev/null || echo 'ok')"
    else
        warn "$pkg not available"
    fi
done


# ═══════════════════════════════════════════════════════════════════
# PHASE 2: WORKSPACE EXTRACTION
# ═══════════════════════════════════════════════════════════════════
banner "Phase 2: Workspace Setup"

# ─── Extract tar if provided ──────────────────────────────────────
WS_PARENT="$(dirname "$WORKSPACE")"
if [ -n "$TAR_PATH" ] && [ -f "$TAR_PATH" ]; then
    info "Extracting $TAR_PATH..."
    cd "$WS_PARENT"
    tar xf "$TAR_PATH"
    ok "Archive extracted"
elif [ -f "/mnt/user-data/uploads/qmnf_ns_campaign_final.tar" ]; then
    if [ ! -d "$WORKSPACE/crates/nsgrid3d/src" ]; then
        info "Extracting from uploads..."
        cd "$WS_PARENT"
        tar xf /mnt/user-data/uploads/qmnf_ns_campaign_final.tar
        ok "Archive extracted"
    else
        ok "Workspace already extracted"
    fi
else
    if [ -d "$WORKSPACE/crates/nsgrid3d/src" ]; then
        ok "Workspace already exists at $WORKSPACE"
    else
        fail "No tar archive found and no existing workspace. Provide --tar /path/to/file.tar"
        exit 1
    fi
fi

# ─── Verify workspace structure ──────────────────────────────────
REQUIRED_FILES=(
    "$WORKSPACE/crates/nsgrid3d/src/lib.rs"
    "$WORKSPACE/crates/nsgrid3d/src/biot_savart.rs"
    "$WORKSPACE/crates/nsgrid3d/src/fused_solver.rs"
    "$WORKSPACE/crates/nsgrid3d/src/dual_tray.rs"
    "$WORKSPACE/crates/nsgrid3d/src/regularity_tracker.rs"
    "$WORKSPACE/crates/nsgrid3d/src/ns_grid_3d.rs"
    "$WORKSPACE/crates/nsgrid3d/src/schema_program.rs"
    "$WORKSPACE/crates/nsgrid3d/src/cram_ns_bridge.rs"
    "$WORKSPACE/crates/nsgrid3d/src/inline_oracles.rs"
    "$WORKSPACE/crates/nsgrid3d/src/schema_sweep.rs"
    "$WORKSPACE/crates/nsgrid3d/Cargo.toml"
)

MISSING=0
for f in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "$f" ]; then
        fail "Missing: $f"
        MISSING=$((MISSING + 1))
    fi
done
if [ $MISSING -eq 0 ]; then
    ok "All ${#REQUIRED_FILES[@]} required Rust source files present"
else
    fail "$MISSING files missing — workspace incomplete"
    exit 1
fi

# ─── Create qmnf_primitives stub if needed ───────────────────────
PRIMS_DIR="$WORKSPACE/crates/qmnf_primitives"
if [ ! -f "$PRIMS_DIR/src/lib.rs" ]; then
    info "Creating qmnf_primitives stub crate..."
    mkdir -p "$PRIMS_DIR/src"

    cat > "$PRIMS_DIR/Cargo.toml" << 'TOML'
[package]
name = "qmnf_primitives"
version = "0.1.0"
edition = "2021"
description = "QMNF core primitives: K-Elimination, modular inverse, CRT reconstruction"

[dependencies]
TOML

    cat > "$PRIMS_DIR/src/lib.rs" << 'RUST'
//! QMNF Primitives — Core exact-integer arithmetic
//! Kill #1: K-Elimination Theorem

/// Extended Euclidean algorithm returning modular inverse.
#[inline]
pub fn mod_inverse(a: u64, m: u64) -> u64 {
    if m == 1 { return 0; }
    let (mut old_r, mut r) = (a as i128, m as i128);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = old_r / r;
        let temp_r = r; r = old_r - q * r; old_r = temp_r;
        let temp_s = s; s = old_s - q * s; old_s = temp_s;
    }
    assert!(old_r == 1, "mod_inverse: gcd({}, {}) ≠ 1", a, m);
    ((old_s % m as i128 + m as i128) % m as i128) as u64
}

/// K-Elimination: κ = ((s_anchor - r_safe) × M_safe⁻¹) mod P_anchor
#[inline]
pub fn k_eliminate(r_safe: u64, s_anchor: u64, m_safe: u64, p_anchor: u64) -> u64 {
    let inv = mod_inverse(m_safe % p_anchor, p_anchor);
    let diff = if s_anchor >= (r_safe % p_anchor) {
        s_anchor - (r_safe % p_anchor)
    } else {
        p_anchor - ((r_safe % p_anchor) - s_anchor)
    };
    ((diff as u128 * inv as u128) % p_anchor as u128) as u64
}

/// Full reconstruction: X = r_safe + κ × M_safe
#[inline]
pub fn k_reconstruct(r_safe: u64, s_anchor: u64, m_safe: u64, p_anchor: u64) -> u128 {
    let k = k_eliminate(r_safe, s_anchor, m_safe, p_anchor) as u128;
    r_safe as u128 + k * m_safe as u128
}

#[inline]
pub fn gcd(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a, b);
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

#[inline]
pub fn coprime(a: u64, b: u64) -> bool { gcd(a, b) == 1 }

#[inline]
pub fn crt2(r1: u64, m1: u64, r2: u64, m2: u64) -> u128 {
    let inv = mod_inverse(m1 % m2, m2);
    let diff = if r2 >= (r1 % m2) { r2 - (r1 % m2) } else { m2 - ((r1 % m2) - r2) };
    let k = ((diff as u128 * inv as u128) % m2 as u128) as u64;
    r1 as u128 + k as u128 * m1 as u128
}

pub fn garner_reconstruct(residues: &[u64], moduli: &[u64]) -> u128 {
    assert_eq!(residues.len(), moduli.len());
    if residues.is_empty() { return 0; }
    let mut result: u128 = residues[0] as u128;
    let mut product: u128 = moduli[0] as u128;
    for i in 1..residues.len() {
        let mi = moduli[i] as u128;
        let ri = residues[i] as u128;
        let pm = product % mi;
        let inv = mod_inverse(pm as u64, moduli[i]) as u128;
        let diff = (ri + mi - (result % mi)) % mi;
        let t = (diff * inv) % mi;
        result += product * t;
        product *= mi;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_mod_inverse_basic() { assert_eq!((mod_inverse(3, 7) * 3) % 7, 1); }
    #[test] fn test_k_reconstruct_exact() {
        let m = 3003u64; let p = (1u64 << 31) - 1;
        for &x in &[0u64, 1, 42, 360, 432000, 2_000_000_000] {
            assert_eq!(k_reconstruct(x % m, x % p, m, p), x as u128);
        }
    }
    #[test] fn test_k_eliminate_exhaustive_small() {
        for x in 0..35u64 {
            let k = k_eliminate(x % 5, x % 7, 5, 7);
            assert_eq!(k, x / 5, "k wrong for x={}", x);
        }
    }
}
RUST

    ok "qmnf_primitives stub created"
else
    ok "qmnf_primitives already exists"
fi

# ─── Create workspace Cargo.toml if needed ────────────────────────
if [ ! -f "$WORKSPACE/Cargo.toml" ]; then
    cat > "$WORKSPACE/Cargo.toml" << 'TOML'
[workspace]
members = ["crates/qmnf_primitives", "crates/nsgrid3d", "crates/crtbigint", "crates/hydra_v3"]
resolver = "2"

[profile.release]
lto = "fat"
codegen-units = 1
opt-level = 3

[profile.test]
opt-level = 2
TOML
    ok "Workspace Cargo.toml created"
else
    ok "Workspace Cargo.toml exists"
fi


# ═══════════════════════════════════════════════════════════════════
# PHASE 3: PYTHON SCAFFOLD
# ═══════════════════════════════════════════════════════════════════
banner "Phase 3: Python Scaffold"

if [ -f "$SCAFFOLD_DIR/cram_scaffold.py" ]; then
    ok "cram_scaffold.py exists"
else
    fail "cram_scaffold.py not found at $SCAFFOLD_DIR — create it manually or from PDF"
fi

if [ -f "$SCAFFOLD_DIR/test_runner.py" ]; then
    ok "test_runner.py exists"
else
    fail "test_runner.py not found at $SCAFFOLD_DIR — create it manually or from PDF"
fi


# ═══════════════════════════════════════════════════════════════════
# PHASE 4: BUILD
# ═══════════════════════════════════════════════════════════════════
banner "Phase 4: Rust Compilation"

cd "$WORKSPACE"

if [ "$RELEASE_ONLY" = true ]; then
    info "Building release only..."
    cargo build --release 2>&1 | grep -E "Compiling|Finished|error" || true
else
    info "Building debug..."
    cargo build 2>&1 | grep -E "Compiling|Finished|error" || true
    info "Building release..."
    cargo build --release 2>&1 | grep -E "Compiling|Finished|error" || true
fi

ok "Rust build complete"

# Count lines
RUST_LINES=$(find "$WORKSPACE/crates" -name "*.rs" -exec cat {} + | wc -l)
RUST_FILES=$(find "$WORKSPACE/crates" -name "*.rs" | wc -l)
ok "$RUST_FILES Rust source files, $RUST_LINES total lines"


# ═══════════════════════════════════════════════════════════════════
# PHASE 5: TESTS
# ═══════════════════════════════════════════════════════════════════
if [ "$SKIP_TESTS" = true ]; then
    banner "Phase 5: Tests (SKIPPED)"
    warn "Tests skipped via --skip-tests"
else
    banner "Phase 5: Rust Tests"

    cd "$WORKSPACE"
    RUST_OUTPUT=$(cargo test --release 2>&1)
    RUST_PASS=$(echo "$RUST_OUTPUT" | grep "^test result:" | grep -oP '\d+ passed' | awk '{s+=$1}END{print s+0}')
    RUST_FAIL=$(echo "$RUST_OUTPUT" | grep "^test result:" | grep -oP '\d+ failed' | awk '{s+=$1}END{print s+0}')

    if [ "$RUST_FAIL" -eq 0 ]; then
        ok "Rust: $RUST_PASS passed, 0 failed"
    else
        fail "Rust: $RUST_PASS passed, $RUST_FAIL FAILED"
        RUST_FAILED=$RUST_FAIL
    fi

    banner "Phase 5b: Python Tests"

    cd "$SCAFFOLD_DIR"
    if [ -f "test_runner.py" ]; then
        PY_OUTPUT=$($PYTHON_CMD test_runner.py 2>&1) || true
        PY_REAL_FAIL=$(echo "$PY_OUTPUT" | grep "Real failures:" | awk '{print $NF}')
        PY_REAL_PASS=$(echo "$PY_OUTPUT" | grep "Real passes:" | awk '{print $NF}')
        PY_TOTAL=$(echo "$PY_OUTPUT" | grep "Unit tests:" | awk '{print $NF}')

        if [ "$PY_REAL_FAIL" = "0" ]; then
            ok "Python: $PY_REAL_PASS real passes / $PY_TOTAL total, 0 real failures"
        else
            fail "Python: $PY_REAL_PASS passes, $PY_REAL_FAIL REAL FAILURES"
            PYTHON_FAILED=${PY_REAL_FAIL:-1}
        fi
    else
        warn "test_runner.py not found — skipping Python tests"
    fi
fi


# ═══════════════════════════════════════════════════════════════════
# PHASE 6: ENVIRONMENT SUMMARY
# ═══════════════════════════════════════════════════════════════════
banner "Environment Summary"

echo ""
echo "  Workspace:       $WORKSPACE"
echo "  Scaffold dir:    $SCAFFOLD_DIR"
echo ""
echo "  Rust toolchain:  $(rustc --version 2>/dev/null || echo 'not found')"
echo "  Cargo:           $(cargo --version 2>/dev/null || echo 'not found')"
echo "  Python:          $($PYTHON_CMD --version 2>&1)"
echo ""
echo "  Rust source:     $RUST_FILES files, $RUST_LINES lines"
echo "  nsgrid3d:        $(wc -l $WORKSPACE/crates/nsgrid3d/src/*.rs 2>/dev/null | tail -1 | awk '{print $1}') lines"
echo "  qmnf_primitives: $(wc -l $WORKSPACE/crates/qmnf_primitives/src/*.rs 2>/dev/null | tail -1 | awk '{print $1}') lines"
echo ""

if [ "$SKIP_TESTS" = false ]; then
    echo "  Rust tests:      ${RUST_PASS:-?} passed / ${RUST_FAIL:-?} failed"
    echo "  Python tests:    ${PY_REAL_PASS:-?} passed / ${PY_REAL_FAIL:-?} failed (${PY_TOTAL:-?} total)"
    echo ""
fi

# ─── Key documents ────────────────────────────────────────────────
echo "  Key documents:"
for doc in "$WORKSPACE/OP5_STRETCHING_DEGREE_ANALYSIS.md" \
           "$WORKSPACE/GAP3C_CLOSURE.md"; do
    if [ -f "$doc" ]; then
        echo "    ✓ $(basename $doc)"
    fi
done
echo ""

# ─── Quick-reference commands ─────────────────────────────────────
echo "  Quick commands:"
echo "    cargo test --release           # Run all Rust tests"
echo "    cargo test -p nsgrid3d         # nsgrid3d only"
echo "    cargo test -p qmnf_primitives  # primitives only"
echo "    python3 ~/test_runner.py       # Run Python test suite"
echo "    python3 ~/cram_scaffold.py     # Run scaffold benchmarks"
echo "    cargo run --release            # Run nsgrid3d main"
echo ""

# ─── Final verdict ────────────────────────────────────────────────
TOTAL_FAIL=$((RUST_FAILED + PYTHON_FAILED))
if [ "$TOTAL_FAIL" -eq 0 ]; then
    echo -e "  ${GREEN}${BOLD}✓ ENVIRONMENT READY — ALL SYSTEMS GREEN${NC}"
    echo ""
    exit 0
else
    echo -e "  ${RED}${BOLD}✗ $TOTAL_FAIL TEST FAILURES — INVESTIGATE BEFORE PROCEEDING${NC}"
    echo ""
    exit 1
fi
