#!/usr/bin/env bash
# Verify build artifacts after a successful build_all.sh / build_all.bat
# Usage: verify_build.sh [--dist-root <path>]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd -P)"
DIST_ROOT="$REPO_ROOT/out/dist"
VERBOSE=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dist-root) DIST_ROOT="$2"; shift 2 ;;
        --verbose|-v) VERBOSE=1; shift ;;
        --help|-h) echo "Usage: $0 [--dist-root <path>] [--verbose]"; exit 0 ;;
        *) echo "Unknown option: $1"; exit 2 ;;
    esac
done

PASS=0
FAIL=0
WARN=0

check() {
    local status="$1" label="$2" msg="$3"
    case "$status" in
        pass) echo "  [PASS] $label"; PASS=$((PASS + 1)) ;;
        fail) echo "  [FAIL] $label: $msg" >&2; FAIL=$((FAIL + 1)) ;;
        warn) echo "  [WARN] $label: $msg" >&2; WARN=$((WARN + 1)) ;;
    esac
}

require_file() {
    local path="$1" label="$2"
    if [[ -f "$path" ]]; then
        if [[ -s "$path" ]]; then
            check pass "$label ($path)"
        else
            check fail "$label" "file is empty"
        fi
    else
        check fail "$label" "not found at $path"
    fi
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || check fail "$1" "not on PATH"
}

echo "=== Build Artifact Verification ==="
echo "Dist root: $DIST_ROOT"
echo ""

# Detect platform
OS_NAME="$(uname -s)"
case "$OS_NAME" in
    Darwin) DIST_DIR="macos" ;;
    Linux)  DIST_DIR="linux" ;;
    *)      echo "Unsupported OS: $OS_NAME"; exit 1 ;;
esac

BIN_DIR="$DIST_ROOT/$DIST_DIR/bin"
LIB_DIR="$DIST_ROOT/$DIST_DIR/lib"

# Check dist directory exists
if [[ ! -d "$DIST_ROOT" ]]; then
    check fail "dist root" "$DIST_ROOT does not exist"
    echo ""
    echo "Summary: $PASS pass, $FAIL fail, $WARN warn"
    exit 1
fi

# Check binaries
echo "[Binaries]"
require_file "$BIN_DIR/virtmgr" "virtmgr"
require_file "$BIN_DIR/vm" "vm"
require_file "$BIN_DIR/crosvm" "crosvm"

# Check library
echo ""
echo "[Libraries]"
LIB_PATTERN="$LIB_DIR/libbinder-rpc*"
if compgen -G "$LIB_PATTERN" >/dev/null; then
    for f in $LIB_PATTERN; do
        require_file "$f" "binder-rpc ($(basename "$f"))"
    done
else
    check fail "binder-rpc library" "no matching file in $LIB_DIR"
fi

# File type verification
echo ""
echo "[File Types]"
for bin in virtmgr vm crosvm; do
    path="$BIN_DIR/$bin"
    if [[ -f "$path" ]]; then
        file_type="$(file -b "$path" 2>/dev/null || true)"
        if grep -Eiq 'Mach-O|ELF' <<<"$file_type"; then
            [[ $VERBOSE -eq 1 ]] && check pass "$bin file type" "$file_type"
        else
            check warn "$bin" "unexpected file type: $file_type"
        fi

        # Check executable flag
        if [[ -x "$path" ]]; then
            [[ $VERBOSE -eq 1 ]] && check pass "$bin executable" ""
        else
            check warn "$bin" "not executable"
        fi
    fi
done

# macOS-specific: check codesigning
echo ""
echo "[Platform Verification]"
if [[ "$OS_NAME" == "Darwin" ]]; then
    if command -v codesign >/dev/null 2>&1; then
        for bin in virtmgr vm crosvm; do
            path="$BIN_DIR/$bin"
            if [[ -f "$path" ]] && codesign -v "$path" 2>/dev/null; then
                [[ $VERBOSE -eq 1 ]] && check pass "codesign $bin" ""
            else
                check warn "codesign $bin" "not signed or signature invalid"
            fi
        done
    fi
    # Check HVF entitlement on crosvm
    if [[ -f "$BIN_DIR/crosvm" ]]; then
        if codesign -d --entitlements :- "$BIN_DIR/crosvm" 2>/dev/null | grep -q '<key>com.apple.security.hypervisor</key>'; then
            check pass "crosvm HVF entitlement" ""
        else
            check fail "crosvm HVF entitlement" "missing com.apple.security.hypervisor"
        fi
    fi
fi

if [[ "$OS_NAME" == "Linux" ]]; then
    for bin in virtmgr vm crosvm; do
        path="$BIN_DIR/$bin"
        if [[ -f "$path" ]] && ldd "$path" >/dev/null 2>&1; then
            [[ $VERBOSE -eq 1 ]] && check pass "$bin shared libs" "ldd OK"
        elif [[ -f "$path" ]]; then
            check warn "$bin" "ldd failed (possibly statically linked)"
        fi
    done
fi

echo ""
echo "=== Summary ==="
echo "  PASS: $PASS"
echo "  FAIL: $FAIL"
echo "  WARN: $WARN"
echo ""

if [[ $FAIL -gt 0 ]]; then
    echo "Verification FAILED — $FAIL check(s) failed."
    exit 1
fi

echo "Verification PASSED."
exit 0
