#!/usr/bin/env bash
set -euo pipefail

PRODUCT_DIR="/opt/workspace/aosp/out/target/product/vsoc_x86_64"
KERNEL_DIST="/opt/workspace/android-kernel-6.6/out/virtual_device_x86_64/dist"
OUTPUT=""
LZ4_BIN="${LZ4_BIN:-/opt/workspace/aosp/out/host/linux-x86/bin/lz4}"

usage() {
    cat <<EOF
Usage: $0 [options]

Options:
  --product-dir DIR     AOSP product output with vendor_ramdisk.img (default: $PRODUCT_DIR)
  --kernel-dist DIR     Android kernel dist with bzImage/initramfs.img (default: $KERNEL_DIST)
  --output FILE         Output vendor ramdisk lz4 cpio (default: PRODUCT_DIR/vendor_ramdisk.<kernel-release>.img)
  --lz4 FILE            lz4 binary (default: $LZ4_BIN)
  --help                Show this help

This script only repacks an existing AOSP vendor_ramdisk with modules from an existing kernel
dist. It does not clean or rebuild AOSP.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --product-dir) PRODUCT_DIR="$2"; shift 2 ;;
        --kernel-dist) KERNEL_DIST="$2"; shift 2 ;;
        --output) OUTPUT="$2"; shift 2 ;;
        --lz4) LZ4_BIN="$2"; shift 2 ;;
        --help|-h) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

require_file() {
    local path="$1"
    local label="$2"
    if [[ ! -f "$path" ]]; then
        echo "Error: missing $label: $path" >&2
        exit 1
    fi
}

require_dir() {
    local path="$1"
    local label="$2"
    if [[ ! -d "$path" ]]; then
        echo "Error: missing $label: $path" >&2
        exit 1
    fi
}

require_file "$PRODUCT_DIR/vendor_ramdisk.img" "AOSP vendor ramdisk"
require_file "$KERNEL_DIST/initramfs.img" "kernel dist initramfs"
require_file "$KERNEL_DIST/bzImage" "kernel dist bzImage"
require_file "$LZ4_BIN" "lz4"
require_dir "$KERNEL_DIST" "kernel dist"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

product_root="$tmp/product-vendor-ramdisk"
dist_root="$tmp/kernel-dist-initramfs"
dep_root="$tmp/depmod"
mkdir -p "$product_root" "$dist_root"

(
    cd "$product_root"
    "$LZ4_BIN" -dc "$PRODUCT_DIR/vendor_ramdisk.img" | cpio -idmu 2>/dev/null
)
(
    cd "$dist_root"
    "$LZ4_BIN" -dc "$KERNEL_DIST/initramfs.img" | cpio -idmu 2>/dev/null
)

kernel_release="$(find "$dist_root/lib/modules" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | sort | head -n 1)"
if [[ -z "$kernel_release" ]]; then
    echo "Error: could not determine kernel release from $KERNEL_DIST/initramfs.img" >&2
    exit 1
fi

if [[ -z "$OUTPUT" ]]; then
    OUTPUT="$PRODUCT_DIR/vendor_ramdisk.${kernel_release}.img"
fi

src_modules="$dist_root/lib/modules/$kernel_release"
dst_modules="$product_root/lib/modules"
require_dir "$src_modules" "kernel dist module tree"
require_dir "$dst_modules" "AOSP vendor ramdisk module dir"
require_file "$dst_modules/modules.load" "AOSP modules.load"

find_module_by_basename() {
    local basename="$1"
    find "$src_modules" -type f -name "$basename" -print -quit
}

rm -f "$dst_modules"/*.ko "$dst_modules"/modules.alias "$dst_modules"/modules.dep \
    "$dst_modules"/modules.softdep "$dst_modules"/modules.symbols "$dst_modules"/modules.builtin \
    "$dst_modules"/modules.builtin.modinfo "$dst_modules"/modules.devname

mapfile -t modules_to_copy < <(sed -e 's/#.*//' -e '/^[[:space:]]*$/d' "$product_root/lib/modules/modules.load")
for module_name in "${modules_to_copy[@]}"; do
    src="$(find_module_by_basename "$module_name")"
    if [[ -z "$src" ]]; then
        echo "Error: kernel dist does not contain module required by modules.load: $module_name" >&2
        exit 1
    fi
    cp -L "$src" "$dst_modules/$module_name"
done

mkdir -p "$dep_root/lib/modules/$kernel_release"
cp -L "$dst_modules"/*.ko "$dep_root/lib/modules/$kernel_release/"
cp -L "$KERNEL_DIST/modules.builtin" "$dep_root/lib/modules/$kernel_release/modules.builtin" 2>/dev/null || true
cp -L "$KERNEL_DIST/modules.builtin.modinfo" "$dep_root/lib/modules/$kernel_release/modules.builtin.modinfo" 2>/dev/null || true
depmod -b "$dep_root" "$kernel_release"

python3 - "$dep_root/lib/modules/$kernel_release" "$dst_modules" <<'PY'
import pathlib
import sys

src = pathlib.Path(sys.argv[1])
dst = pathlib.Path(sys.argv[2])

def flat_dep_line(line: str) -> str:
    left, sep, right = line.partition(":")
    if not sep:
        return line
    left_name = pathlib.PurePosixPath(left).name
    deps = [
        f"/lib/modules/{pathlib.PurePosixPath(dep).name}"
        for dep in right.strip().split()
        if dep
    ]
    if deps:
        return f"/lib/modules/{left_name}: {' '.join(deps)}"
    return f"/lib/modules/{left_name}:"

dep = src / "modules.dep"
if dep.exists():
    lines = [flat_dep_line(line) for line in dep.read_text(encoding="utf-8").splitlines()]
    (dst / "modules.dep").write_text("\n".join(lines) + "\n", encoding="utf-8")

for name in ("modules.alias", "modules.softdep", "modules.symbols", "modules.devname"):
    src_file = src / name
    if src_file.exists():
        (dst / name).write_bytes(src_file.read_bytes())
PY

(
    cd "$product_root"
    find . | LC_ALL=C sort | cpio -o -H newc --owner 0:0 2>/dev/null
) | "$LZ4_BIN" -l -f - "$OUTPUT" >/dev/null

echo "Kernel release: $kernel_release"
echo "Kernel: $KERNEL_DIST/bzImage"
echo "Vendor ramdisk: $OUTPUT"
