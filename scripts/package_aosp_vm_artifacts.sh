#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

AOSP_ROOT="/opt/workspace/aosp"
DIST_ROOT="$REPO_ROOT/out/dist"
OUTPUT_ROOT="/mnt/workspace/Windows/bscp-vm-artifacts"
PACKAGE_NAME="bscp-vm-artifacts-$(date +%Y%m%d-%H%M%S)"
PRODUCTS=("vsoc_x86_64" "vsoc_arm64")
CREATE_ARCHIVE=0
ZSTD_LEVEL="${ZSTD_LEVEL:-3}"

usage() {
    cat <<EOF
Usage: $0 [options]

Options:
  --aosp-root DIR       AOSP tree with existing out/ products (default: $AOSP_ROOT)
  --dist-root DIR       bscp dist root (default: $DIST_ROOT)
  --output-root DIR     Output root (default: $OUTPUT_ROOT)
  --package-name NAME   Package directory name (default: timestamped)
  --products LIST       Comma-separated Android products (default: vsoc_x86_64,vsoc_arm64)
  --archive             Also create a compressed archive
  --no-archive          Only create the directory package (default)
  --help                Show this help

This script only copies existing artifacts. It does not clean or build AOSP.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --aosp-root) AOSP_ROOT="$2"; shift 2 ;;
        --dist-root) DIST_ROOT="$2"; shift 2 ;;
        --output-root) OUTPUT_ROOT="$2"; shift 2 ;;
        --package-name) PACKAGE_NAME="$2"; shift 2 ;;
        --products)
            IFS=',' read -r -a PRODUCTS <<<"$2"
            shift 2
            ;;
        --archive) CREATE_ARCHIVE=1; shift ;;
        --no-archive) CREATE_ARCHIVE=0; shift ;;
        --help|-h) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

AOSP_PRODUCT_ROOT="$AOSP_ROOT/out/target/product"
AOSP_HOST_ROOT="$AOSP_ROOT/out/host"
PACKAGE_DIR="$OUTPUT_ROOT/$PACKAGE_NAME"
MANIFEST="$PACKAGE_DIR/manifest.txt"
ANDROID_LINUX_OUT="$REPO_ROOT/out/android-linux"

require_dir() {
    local path="$1"
    local label="$2"
    if [[ ! -d "$path" ]]; then
        echo "Error: missing $label: $path" >&2
        exit 1
    fi
}

copy_file() {
    local src="$1"
    local dst="$2"
    if [[ -f "$src" ]]; then
        mkdir -p "$(dirname "$dst")"
        cp -L "$src" "$dst"
        printf 'file %s -> %s\n' "$src" "${dst#$PACKAGE_DIR/}" >>"$MANIFEST"
    fi
}

copy_dir() {
    local src="$1"
    local dst="$2"
    if [[ -d "$src" ]]; then
        mkdir -p "$(dirname "$dst")"
        rsync -aL --delete "$src/" "$dst/"
        printf 'dir  %s -> %s\n' "$src" "${dst#$PACKAGE_DIR/}" >>"$MANIFEST"
    fi
}

copy_glob_files() {
    local dst_dir="$1"
    shift
    local pattern match
    mkdir -p "$dst_dir"
    for pattern in "$@"; do
        shopt -s nullglob
        for match in $pattern; do
            if [[ -f "$match" ]]; then
                copy_file "$match" "$dst_dir/$(basename "$match")"
            fi
        done
        shopt -u nullglob
    done
}

copy_android_product() {
    local product="$1"
    local product_dir="$AOSP_PRODUCT_ROOT/$product"
    local dst="$PACKAGE_DIR/products/android/$product"
    require_dir "$product_dir" "AOSP product $product"

    mkdir -p "$dst/images" "$dst/meta" "$dst/vendor/etc"
    copy_glob_files "$dst/images" "$product_dir"/*.img "$product_dir"/kernel "$product_dir"/kernel_* "$product_dir"/bootloader
    copy_glob_files "$dst/meta" \
        "$product_dir"/android-info.txt \
        "$product_dir"/fastboot-info.txt \
        "$product_dir"/misc_info.txt \
        "$product_dir"/build_fingerprint.txt \
        "$product_dir"/build_thumbprint.txt \
        "$product_dir"/required_images \
        "$product_dir"/installed-files*.txt \
        "$product_dir"/installed-files*.json
    copy_file "$product_dir/vendor/etc/fstab.cf.f2fs.hctr2" "$dst/vendor/etc/fstab.cf.f2fs.hctr2"
    copy_file "$product_dir/module-info.json" "$dst/meta/module-info.json"

    if [[ -d "$product_dir/apex/com.android.virt" ]]; then
        copy_dir "$product_dir/apex/com.android.virt" "$PACKAGE_DIR/products/microdroid/$product/com.android.virt"
    fi
    if [[ -d "$product_dir/symbols/apex/com.android.virt/app" ]]; then
        copy_dir "$product_dir/symbols/apex/com.android.virt/app" "$PACKAGE_DIR/products/microdroid/$product/symbols/app"
    fi
}

copy_android_direct_linux() {
    local product="vsoc_x86_64"
    local product_dir="$AOSP_PRODUCT_ROOT/$product"
    local dst="$PACKAGE_DIR/products/android/$product/direct-linux"
    if [[ ! -d "$ANDROID_LINUX_OUT" ]]; then
        return 0
    fi

    mkdir -p "$dst"
    copy_file "$ANDROID_LINUX_OUT/aggregate_android.img" "$dst/aggregate_android.img"
    copy_file "$ANDROID_LINUX_OUT/initrd_android.img" "$dst/initrd_android.img"
    copy_file "$ANDROID_LINUX_OUT/android_fstab.dt" "$dst/android_fstab.dt"
    copy_file "$ANDROID_LINUX_OUT/android.dtb" "$dst/android.dtb"
    copy_file "$ANDROID_LINUX_OUT/android_fstab_extra.cpio.lz4" "$dst/android_fstab_extra.cpio.lz4"
    copy_file "$ANDROID_LINUX_OUT/misc.img" "$dst/misc.img"
    copy_file "$ANDROID_LINUX_OUT/factory_reset_protected.img" "$dst/factory_reset_protected.img"
    copy_file "$ANDROID_LINUX_OUT/metadata.img" "$dst/metadata.img"
    copy_dir "$ANDROID_LINUX_OUT/hvc" "$dst/hvc"
    copy_file "$product_dir/kernel" "$dst/kernel"

    cat >"$dst/README.txt" <<EOF
# Android direct Linux boot image set

This directory contains the already synthesized x86_64 Android image set from:

$ANDROID_LINUX_OUT

Use with the packaged Linux host runtime:

DISPLAY=:1 host/linux-x86_64/dist/bin/crosvm --log-level info run --disable-sandbox --cid 100 --mem 8192 --cpus 4 --no-balloon --no-usb --x-display :1 \\
  --gpu 'backend=gfxstream,displays=[[mode=windowed[1280,720],dpi=[320,320],refresh-rate=60]],context-types=gfxstream-vulkan:gfxstream-composer,angle=true,gles=false,vulkan=true,wsi=vk' \\
  --block path=products/android/$product/direct-linux/aggregate_android.img,ro=false,lock=false,sparse=false,pci-address=00:03.0 \\
  --android-fstab products/android/$product/direct-linux/android_fstab.dt \\
  --initrd products/android/$product/direct-linux/initrd_android.img \\
  --params 'console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init' \\
  products/android/$product/direct-linux/kernel

The helper script scripts/run_android_linux.sh can regenerate this set from the source AOSP
product images when running inside the BSCP workspace. This package includes the generated set so
other platforms can compare layout and boot inputs directly.
EOF
    printf 'file %s -> %s\n' "$dst/README.txt" "${dst#$PACKAGE_DIR/}/README.txt" >>"$MANIFEST"
}

copy_microdroid_soong_arch() {
    local label="$1"
    local soong_arch="$2"
    local base="$AOSP_ROOT/out/soong/.intermediates/packages/modules/Virtualization/build/microdroid"
    local dst="$PACKAGE_DIR/products/microdroid/soong/$label"

    copy_file "$base/microdroid_kernel/$soong_arch/microdroid_kernel" "$dst/microdroid_kernel"
    copy_file "$base/microdroid_kernel_signed/$soong_arch/microdroid_kernel" "$dst/microdroid_kernel_signed"
    copy_file "$base/microdroid_vbmeta/$soong_arch/microdroid_vbmeta.img" "$dst/microdroid_vbmeta.img"
    copy_file "$base/microdroid_super/$soong_arch/microdroid_super.img" "$dst/microdroid_super.img"
    copy_file "$base/initrd/microdroid_initrd_normal/$soong_arch/microdroid_initrd_normal.img" "$dst/microdroid_initrd_normal.img"
    copy_file "$base/initrd/microdroid_initrd_debuggable/$soong_arch/microdroid_initrd_debuggable.img" "$dst/microdroid_initrd_debuggable.img"
    copy_file "$base/microdroid.json/$soong_arch/microdroid.json" "$dst/microdroid.json"
    copy_file "$base/microdroid_fstab/$soong_arch/fstab.microdroid" "$dst/fstab.microdroid"
}

copy_host_runtime() {
    copy_dir "$DIST_ROOT/linux" "$PACKAGE_DIR/host/linux-x86_64/dist"
}

copy_product_runtime() {
    copy_dir "$DIST_ROOT/apex_dir" "$PACKAGE_DIR/products/microdroid/apex_dir"
}

copy_selected_host_tools() {
    copy_file "$AOSP_HOST_ROOT/linux-x86/bin/adb" "$PACKAGE_DIR/host-tools/linux-x86_64/bin/adb"
    copy_file "$AOSP_HOST_ROOT/linux-x86/bin/avbtool" "$PACKAGE_DIR/host-tools/linux-x86_64/bin/avbtool"
    copy_file "$AOSP_HOST_ROOT/linux-x86/bin/lpmake" "$PACKAGE_DIR/host-tools/linux-x86_64/bin/lpmake"
    copy_file "$AOSP_HOST_ROOT/linux-x86/bin/simg2img" "$PACKAGE_DIR/host-tools/linux-x86_64/bin/simg2img"
    copy_file "$AOSP_HOST_ROOT/linux_musl-arm64/bin/adb" "$PACKAGE_DIR/host-tools/linux-arm64/bin/adb"
}

copy_repo_helpers() {
    copy_dir "$REPO_ROOT/scripts" "$PACKAGE_DIR/scripts"
    copy_file "$REPO_ROOT/doc/LINUX_AVF_VM.md" "$PACKAGE_DIR/docs/LINUX_AVF_VM.md"
    copy_file "$REPO_ROOT/doc/AOSP_CF_WINDOWS_GFXSTREAM_BOOT_STATUS.md" "$PACKAGE_DIR/docs/AOSP_CF_WINDOWS_GFXSTREAM_BOOT_STATUS.md"
    copy_file "$REPO_ROOT/doc/CROSS_PLATFORM_VM_ARTIFACTS.md" "$PACKAGE_DIR/docs/CROSS_PLATFORM_VM_ARTIFACTS.md"
    copy_file "$REPO_ROOT/doc/PLATFORM_DIFFERENCES.md" "$PACKAGE_DIR/docs/PLATFORM_DIFFERENCES.md"
}

write_summary() {
    local head
    head="$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || true)"
    cat >"$PACKAGE_DIR/README.txt" <<EOF
# BSCP VM artifact package

Created: $(date -Iseconds)
Repo: $REPO_ROOT
Repo HEAD: $head
AOSP root: $AOSP_ROOT

Contents:
- products/android/<product>/images: AOSP top-level boot/kernel/vbmeta/super/userdata/partition images.
- products/android/vsoc_x86_64/direct-linux: synthesized aggregate Android disk, initrd, fstab DT, helper partitions, and kernel used by the validated Linux direct-crosvm boot.
- products/android/<product>/meta: product metadata useful for bring-up and diffing.
- products/microdroid/<product>/com.android.virt: Microdroid payloads from each product's com.android.virt apex.
- products/microdroid/soong/<arch>: architecture-specific Microdroid kernel/initrd/super/vbmeta/json/fstab.
- products/microdroid/apex_dir: mounted APEX runtime tree required by Microdroid on every host platform.
- host/linux-x86_64: current bscp Linux host runtime only.
- host-tools: selected AOSP tools that exist in this workspace, not full host output trees.
- scripts and docs: the current launch, check, and packaging helpers.

This package was produced by copying existing artifacts only. No AOSP clean or full rebuild was run.
EOF
}

create_archive() {
    local archive
    if [[ "$CREATE_ARCHIVE" -ne 1 ]]; then
        return 0
    fi
    if command -v zstd >/dev/null 2>&1; then
        archive="$OUTPUT_ROOT/$PACKAGE_NAME.tar.zst"
        tar -C "$OUTPUT_ROOT" -cf - "$PACKAGE_NAME" | zstd -T0 "-$ZSTD_LEVEL" -o "$archive"
    else
        archive="$OUTPUT_ROOT/$PACKAGE_NAME.tar.gz"
        tar -C "$OUTPUT_ROOT" -czf "$archive" "$PACKAGE_NAME"
    fi
    printf '%s\n' "$archive" >"$PACKAGE_DIR/archive.path"
    echo "Archive: $archive"
}

require_dir "$AOSP_PRODUCT_ROOT" "AOSP product root"
require_dir "$DIST_ROOT" "dist root"
rm -rf "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR"
: >"$MANIFEST"

for product in "${PRODUCTS[@]}"; do
    copy_android_product "$product"
done
copy_android_direct_linux
copy_microdroid_soong_arch "x86_64" "android_x86_64_silvermont"
copy_microdroid_soong_arch "arm64" "android_arm64_armv8-a_cortex-a53"
copy_product_runtime
copy_host_runtime
copy_selected_host_tools
copy_repo_helpers
write_summary
create_archive

du -sh "$PACKAGE_DIR"
if [[ -f "$PACKAGE_DIR/archive.path" ]]; then
    du -h "$(cat "$PACKAGE_DIR/archive.path")"
fi
echo "Package directory: $PACKAGE_DIR"
