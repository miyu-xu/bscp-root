#!/usr/bin/env bash
# Fetch aarch64 Microdroid guest kernel + ramdisk into the host APEX tree.
# Usage:
#   fetch_arm64_guest_artifacts.sh [--apex-tree <path>]
#
# Default apex tree: $REPO_ROOT/out/dist/apex_dir
#
# The script accepts an optional --kernel-url from which to download a pre-built
# arm64 Microdroid kernel Image.gz (raw, not compressed).  If omitted it checks
# whether the existing kernel is already arm64, and if not, prints a diagnostic
# suggesting how to obtain one.
#
# Environment:
#   BSCP_GUEST_KERNEL_URL   — override download URL for the arm64 kernel image
#   BSCP_GUEST_RAMDISK_URL  — override download URL for the initramfs (optional)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd -P)"
APEX_TREE="${BSCP_APEX_TREE:-$REPO_ROOT/out/dist/apex_dir}"
KERNEL_DST="$APEX_TREE/apex/com.android.virt/etc/fs/microdroid_kernel"
RAMDISK_DST="$APEX_TREE/apex/com.android.virt/etc/fs/initramfs.img"

# Default URL: AOSP aosp-main built at ci.android.com (generic arm64 target).
# This is a stable-ish prebuilt; adjust if you have a custom build.
DEFAULT_KERNEL_URL="${BSCP_GUEST_KERNEL_URL:-}"
DEFAULT_RAMDISK_URL="${BSCP_GUEST_RAMDISK_URL:-}"

usage() {
    cat <<'EOF'
Usage: fetch_arm64_guest_artifacts.sh [options]

Options:
  --apex-tree <path>        Target apex tree root (default: out/dist/apex_dir)
  --kernel-url <url>        Download kernel Image from this URL
  --ramdisk-url <url>       Download initramfs from this URL (optional)
  --skip-download           Only check and validate, do not download
  -h, --help                Show this help
EOF
}

SKIP_DOWNLOAD=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --apex-tree)
            APEX_TREE="$2"; shift 2 ;;
        --kernel-url)
            DEFAULT_KERNEL_URL="$2"; shift 2 ;;
        --ramdisk-url)
            DEFAULT_RAMDISK_URL="$2"; shift 2 ;;
        --skip-download)
            SKIP_DOWNLOAD=1; shift ;;
        -h|--help)
            usage; exit 0 ;;
        *)
            echo "Unknown option: $1" >&2; usage; exit 2 ;;
    esac
done

require_command() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "Required command missing: $1" >&2; exit 1
    }
}

require_command file
require_command curl

# --- Check existing kernel ---
EXISTING_KERNEL=""
EXISTING_ARCH=""
if [[ -f "$KERNEL_DST" ]]; then
    EXISTING_KERNEL="yes"
    EXISTING_INFO="$(file -b "$KERNEL_DST" 2>/dev/null || true)"
    if grep -Eiq 'ARM aarch64|ARM64|arm64' <<<"$EXISTING_INFO"; then
        EXISTING_ARCH="arm64"
    elif grep -Eiq 'x86[-_ ]?64|x86 boot|amd64' <<<"$EXISTING_INFO"; then
        EXISTING_ARCH="x86_64"
    else
        EXISTING_ARCH="unknown ($EXISTING_INFO)"
    fi
fi

echo "=== Microdroid Guest Artifact Check ==="
echo "APEX tree: $APEX_TREE"
echo "Kernel dst: $KERNEL_DST"

if [[ "$EXISTING_ARCH" == "arm64" ]]; then
    echo ""
    echo "  Kernel: arm64 (correct for macOS HVF)"
    echo "  Path:   $KERNEL_DST"
    echo "  Info:   $EXISTING_INFO"
    if [[ $SKIP_DOWNLOAD -eq 1 ]]; then
        exit 0
    fi
    echo ""
    echo "Kernel already arm64. Pass --skip-download to only validate."
    echo "To force re-download, remove $KERNEL_DST first."
elif [[ -n "$EXISTING_KERNEL" ]]; then
    echo ""
    echo "  Kernel arch: $EXISTING_ARCH"
    echo "  Path:        $KERNEL_DST"
    echo "  Info:        $EXISTING_INFO"
    echo ""
    echo "WARNING: macOS HVF requires an arm64 guest kernel."
    echo "The current kernel is NOT arm64."
fi

# --- Download if URLs are configured ---
DOWNLOAD_KERNEL=0
DOWNLOAD_RAMDISK=0

if [[ -n "$DEFAULT_KERNEL_URL" ]]; then
    DOWNLOAD_KERNEL=1
fi
if [[ -n "$DEFAULT_RAMDISK_URL" ]]; then
    DOWNLOAD_RAMDISK=1
fi

if [[ $SKIP_DOWNLOAD -eq 1 ]]; then
    if [[ "$EXISTING_ARCH" != "arm64" ]]; then
        echo ""
        echo "ERROR: No arm64 kernel available and --skip-download is set."
        echo ""
        echo "To obtain an arm64 Microdroid kernel:"
        echo "  Option A: Build from AOSP source"
        echo "    source build/envsetup.sh && lunch aosp_arm64 && m microdroid_kernel"
        echo "    The output is at \$OUT/.../microdroid_kernel"
        echo ""
        echo "  Option B: Use prepare_host_apex_tree.sh with an arm64 apex tree"
        echo "    prepare_host_apex_tree.sh --source-root <arm64-apex> --target-root \$APEX_TREE --expect-kernel-arch arm64"
        echo ""
        echo "  Option C: Specify a download URL via --kernel-url or BSCP_GUEST_KERNEL_URL"
        exit 1
    fi
    exit 0
fi

if [[ $DOWNLOAD_KERNEL -eq 0 && $DOWNLOAD_RAMDISK -eq 0 ]]; then
    if [[ "$EXISTING_ARCH" != "arm64" ]]; then
        echo ""
        echo "No download URLs specified and kernel is not arm64."
        echo ""
        echo "To resolve:"
        echo "  1. Set BSCP_GUEST_KERNEL_URL to a prebuilt arm64 Microdroid kernel URL"
        echo "  2. Or run with --kernel-url <url>"
        echo "  3. Or use prepare_host_apex_tree.sh with an arm64 AOSP build output"
        echo ""
        echo "Example kernel URLs (from AOSP CI artifacts):"
        echo "  https://ci.android.com/builds/latest/branches/aosp-main/targets/generic_arm64/view/index.html"
        echo ""
        echo "Prebuilt arm64 Microdroid kernel (if available at your org):"
        echo "  See AOSP: packages/modules/Virtualization/guest/microdroid_kernel/"
        exit 1
    fi
    echo ""
    echo "Kernel is already arm64 — nothing to do."
    exit 0
fi

# --- Download ---
mkdir -p "$(dirname "$KERNEL_DST")"

if [[ $DOWNLOAD_KERNEL -eq 1 ]]; then
    echo ""
    echo "Downloading arm64 kernel from:"
    echo "  $DEFAULT_KERNEL_URL"
    curl -fsSL -o "$KERNEL_DST.tmp" "$DEFAULT_KERNEL_URL"
    DOWNLOAD_INFO="$(file -b "$KERNEL_DST.tmp" 2>/dev/null || true)"
    if grep -Eiq 'ARM aarch64|ARM64|arm64' <<<"$DOWNLOAD_INFO"; then
        mv "$KERNEL_DST.tmp" "$KERNEL_DST"
        echo "  OK: kernel saved to $KERNEL_DST"
        echo "  File info: $DOWNLOAD_INFO"
    else
        rm -f "$KERNEL_DST.tmp"
        echo "  FAIL: downloaded file does not appear to be an arm64 kernel."
        echo "  File info: $DOWNLOAD_INFO"
        echo "  URL may point to wrong artifact."
        exit 1
    fi
fi

if [[ $DOWNLOAD_RAMDISK -eq 1 ]]; then
    echo ""
    echo "Downloading initramfs from:"
    echo "  $DEFAULT_RAMDISK_URL"
    mkdir -p "$(dirname "$RAMDISK_DST")"
    curl -fsSL -o "$RAMDISK_DST" "$DEFAULT_RAMDISK_URL"
    echo "  OK: initramfs saved to $RAMDISK_DST"
fi

echo ""
echo "=== Done ==="
echo "Kernel: $KERNEL_DST"
if [[ -f "$RAMDISK_DST" ]]; then
    echo "Ramdisk: $RAMDISK_DST"
fi
