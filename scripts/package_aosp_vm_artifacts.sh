#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

AOSP_ROOT="/opt/workspace/aosp"
DIST_ROOT="$REPO_ROOT/out/dist"
OUTPUT_ROOT="/mnt/workspace/Windows/bscp-vm-artifacts"
PACKAGE_NAME="bscp-vm-artifacts-$(date +%Y%m%d-%H%M%S)"
PRODUCTS=("vsoc_x86_64" "vsoc_arm64_only")
CREATE_ARCHIVE=0
ZSTD_LEVEL="${ZSTD_LEVEL:-3}"
DATA_ENCRYPTION="metadata"
FSTAB_SUFFIX=""

usage() {
    cat <<EOF
Usage: $0 [options]

Options:
  --aosp-root DIR       AOSP tree with existing out/ products (default: $AOSP_ROOT)
  --dist-root DIR       bscp dist root (default: $DIST_ROOT)
  --output-root DIR     Output root (default: $OUTPUT_ROOT)
  --package-name NAME   Package directory name (default: timestamped)
  --products LIST       Comma-separated Android products (default: vsoc_x86_64,vsoc_arm64_only)
  --data-encryption MODE
                        Direct-boot /data profile: metadata or none (default: $DATA_ENCRYPTION).
                        none is an explicit development-only compatibility profile.
  --fstab-suffix NAME   Direct-boot fstab suffix. Defaults to cf.f2fs.hctr2 for metadata
                        encryption and dev.direct for the unencrypted development profile.
  --archive             Also create a compressed archive
  --no-archive          Only create the directory package (default)
  --help                Show this help

This script does not clean or build AOSP. It packages existing AOSP products and regenerates a
compact Android-sparse direct-boot aggregate disk from those product images. The temporary raw
aggregate used during conversion is removed after validation.
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
        --data-encryption) DATA_ENCRYPTION="$2"; shift 2 ;;
        --fstab-suffix) FSTAB_SUFFIX="$2"; shift 2 ;;
        --archive) CREATE_ARCHIVE=1; shift ;;
        --no-archive) CREATE_ARCHIVE=0; shift ;;
        --help|-h) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

if [[ "$DATA_ENCRYPTION" != "metadata" && "$DATA_ENCRYPTION" != "none" ]]; then
    echo "Error: --data-encryption must be metadata or none" >&2
    exit 2
fi
if [[ -z "$FSTAB_SUFFIX" ]]; then
    if [[ "$DATA_ENCRYPTION" == "none" ]]; then
        FSTAB_SUFFIX="dev.direct"
    else
        FSTAB_SUFFIX="cf.f2fs.hctr2"
    fi
fi
if [[ ! "$FSTAB_SUFFIX" =~ ^[A-Za-z0-9._-]+$ ]]; then
    echo "Error: --fstab-suffix must contain only letters, digits, dot, underscore, or hyphen" >&2
    exit 2
fi

AOSP_PRODUCT_ROOT="$AOSP_ROOT/out/target/product"
AOSP_HOST_ROOT="$AOSP_ROOT/out/host"
PACKAGE_DIR="$OUTPUT_ROOT/$PACKAGE_NAME"
MANIFEST="$PACKAGE_DIR/manifest.txt"
ANDROID_LINUX_OUT_ROOT="$REPO_ROOT/out"

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
        cp -L --sparse=always "$src" "$dst"
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
    local product="$1"
    local product_dir="$AOSP_PRODUCT_ROOT/$product"
    local source_dir="$ANDROID_LINUX_OUT_ROOT/android-linux-$product"
    local dst="$PACKAGE_DIR/products/android/$product/direct-linux"

    if [[ "$product" == "vsoc_x86_64" && ( -f "$ANDROID_LINUX_OUT_ROOT/android-linux/aggregate_android.img" || -f "$ANDROID_LINUX_OUT_ROOT/android-linux/aggregate_android.sparse.img" ) ]]; then
        source_dir="$ANDROID_LINUX_OUT_ROOT/android-linux"
    fi

    "$REPO_ROOT/scripts/run_android_linux.sh" \
        --product-dir "$product_dir" \
        --work-dir "$source_dir" \
        --log-dir "$DIST_ROOT/logs/android-linux-$product-package" \
        --dynamic-fs-type "$(sed -n 's/^system_fs_type=//p' "$product_dir/misc_info.txt" | head -n 1)" \
        --data-encryption "$DATA_ENCRYPTION" \
        --fstab-suffix "$FSTAB_SUFFIX" \
        --no-run

    local raw_image="$source_dir/aggregate_android.img"
    local sparse_image="$source_dir/aggregate_android.sparse.img"
    local img2simg="$AOSP_HOST_ROOT/linux-x86/bin/img2simg"
    require_file "$raw_image" "raw aggregate Android image for $product"
    require_file "$img2simg" "AOSP img2simg"
    require_file "$product_dir/misc_info.txt" "AOSP misc_info.txt for $product"
    local expected_super_size
    local expected_userdata_size
    expected_super_size="$(sed -n 's/^super_partition_size=//p' "$product_dir/misc_info.txt" | head -n 1)"
    expected_userdata_size="$(sed -n 's/^userdata_size=//p' "$product_dir/misc_info.txt" | head -n 1)"
    if [[ ! "$expected_super_size" =~ ^[0-9]+$ || ! "$expected_userdata_size" =~ ^[0-9]+$ ]]; then
        echo "Error: misc_info.txt has no numeric super_partition_size/userdata_size for $product" >&2
        exit 1
    fi
    rm -f "$sparse_image"
    "$img2simg" "$raw_image" "$sparse_image"
    validate_direct_android_images \
        "$raw_image" \
        "$sparse_image" \
        "$source_dir/android_fstab.dt" \
        "$expected_super_size" \
        "$expected_userdata_size"
    rm -f "$raw_image"

    mkdir -p "$dst"
    copy_file "$sparse_image" "$dst/aggregate_android.sparse.img"
    copy_file "$source_dir/initrd_android.img" "$dst/initrd_android.img"
    copy_file "$source_dir/android_fstab.dt" "$dst/android_fstab.dt"
    copy_file "$source_dir/android.dtb" "$dst/android.dtb"
    copy_file "$source_dir/android_fstab_extra.cpio.lz4" "$dst/android_fstab_extra.cpio.lz4"
    copy_file "$source_dir/misc.img" "$dst/misc.img"
    copy_file "$source_dir/factory_reset_protected.img" "$dst/factory_reset_protected.img"
    copy_file "$source_dir/metadata.img" "$dst/metadata.img"
    copy_dir "$source_dir/hvc" "$dst/hvc"
    copy_file "$product_dir/kernel" "$dst/kernel"

    cat >"$dst/README.txt" <<EOF
# Android direct Linux boot image set

This directory contains the synthesized Android image set for $product from:

$source_dir

Data profile: $DATA_ENCRYPTION
Fstab suffix: $FSTAB_SUFFIX

Consumers should expand aggregate_android.sparse.img into a private writable filesystem-sparse
raw overlay for each instance. The packaged source remains compact and unchanged.
The 'none' profile is a development compatibility profile and is not a production-security
substitute for a persistent, hardware-backed KeyMint implementation. It intentionally marks
/data as first_stage_mount because crosvm exposes this generated fstab through the Android DT;
the second-stage mount_all then skips the already-mounted partition instead of failing with EBUSY.

crosvm's Android sparse backend is read-only, so do not pass this source directly as a writable
block device. For standalone crosvm use, first expand it with the packaged simg2img tool:

host-tools/linux-x86_64/bin/simg2img \
  products/android/$product/direct-linux/aggregate_android.sparse.img \
  /path/to/writable/aggregate_android.img

Then launch with the writable raw GPT image:

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

validate_direct_android_images() {
    local raw_image="$1"
    local sparse_image="$2"
    local android_fstab="$3"
    local expected_super_size="$4"
    local expected_userdata_size="$5"
    python3 - \
        "$raw_image" \
        "$sparse_image" \
        "$android_fstab" \
        "$expected_super_size" \
        "$expected_userdata_size" <<'PY'
import pathlib
import struct
import sys

ANDROID_SPARSE_MAGIC = 0xED26FF3A
LP_METADATA_GEOMETRY_MAGIC = 0x616C4467
SECTOR_SIZE = 512
GPT_ENTRY_SIZE = 128

raw = pathlib.Path(sys.argv[1])
sparse = pathlib.Path(sys.argv[2])
android_fstab = pathlib.Path(sys.argv[3])
expected_partition_sizes = {
    "super": int(sys.argv[4]),
    "userdata": int(sys.argv[5]),
}

with raw.open("rb") as stream:
    stream.seek(SECTOR_SIZE)
    if stream.read(8) != b"EFI PART":
        raise SystemExit(f"invalid raw aggregate GPT header: {raw}")
    stream.seek(2 * SECTOR_SIZE)
    entries = stream.read(128 * GPT_ENTRY_SIZE)
    partitions = {}
    for index in range(128):
        entry = entries[index * GPT_ENTRY_SIZE : (index + 1) * GPT_ENTRY_SIZE]
        first_lba = struct.unpack_from("<Q", entry, 32)[0]
        last_lba = struct.unpack_from("<Q", entry, 40)[0]
        name = entry[56:128].decode("utf-16-le").rstrip("\0")
        if name:
            if last_lba < first_lba:
                raise SystemExit(f"invalid GPT extent for {name}: {raw}")
            partitions[name] = (first_lba, last_lba)
    missing = {"super", "userdata"} - partitions.keys()
    if missing:
        raise SystemExit(f"raw aggregate is missing GPT partitions {sorted(missing)}: {raw}")
    for name, expected_size in expected_partition_sizes.items():
        first_lba, last_lba = partitions[name]
        actual_size = (last_lba - first_lba + 1) * SECTOR_SIZE
        if actual_size != expected_size:
            raise SystemExit(
                f"raw aggregate {name} partition is {actual_size} bytes, "
                f"expected {expected_size} from misc_info.txt: {raw}"
            )
    super_offset = partitions["super"][0] * SECTOR_SIZE
    stream.seek(super_offset)
    if struct.unpack("<I", stream.read(4))[0] == ANDROID_SPARSE_MAGIC:
        raise SystemExit(f"raw aggregate embeds an unexpanded Android sparse super image: {raw}")
    geometry = []
    for offset in (4096, 8192):
        stream.seek(super_offset + offset)
        geometry.append(struct.unpack("<I", stream.read(4))[0])
    if LP_METADATA_GEOMETRY_MAGIC not in geometry:
        raise SystemExit(f"raw aggregate super partition has no valid LP geometry: {raw}")

with sparse.open("rb") as stream:
    header = stream.read(28)
    if len(header) != 28 or struct.unpack_from("<I", header)[0] != ANDROID_SPARSE_MAGIC:
        raise SystemExit(f"invalid Android sparse aggregate header: {sparse}")
    _, major, _, file_header_size, chunk_header_size, block_size, total_blocks, total_chunks, _ = struct.unpack(
        "<IHHHHIIII", header
    )
    if major != 1 or file_header_size < 28 or chunk_header_size < 12 or block_size == 0:
        raise SystemExit(f"unsupported Android sparse aggregate header: {sparse}")
    stream.seek(file_header_size)
    logical_blocks = 0
    for _ in range(total_chunks):
        chunk_header = stream.read(chunk_header_size)
        if len(chunk_header) != chunk_header_size:
            raise SystemExit(f"truncated Android sparse aggregate chunk: {sparse}")
        chunk_type, _, chunk_blocks, total_size = struct.unpack_from("<HHII", chunk_header)
        data_size = total_size - chunk_header_size
        expected_data_size = {
            0xCAC1: chunk_blocks * block_size,
            0xCAC2: 4,
            0xCAC3: 0,
            0xCAC4: 4,
        }.get(chunk_type)
        if expected_data_size is None or data_size != expected_data_size:
            raise SystemExit(f"invalid Android sparse aggregate chunk 0x{chunk_type:04x}: {sparse}")
        stream.seek(data_size, 1)
        logical_blocks += chunk_blocks
    if logical_blocks != total_blocks or stream.tell() != sparse.stat().st_size:
        raise SystemExit(f"Android sparse aggregate chunk layout is inconsistent: {sparse}")

expanded_size = block_size * total_blocks
if expanded_size != raw.stat().st_size:
    raise SystemExit(
        f"sparse aggregate expands to {expanded_size} bytes, expected raw size {raw.stat().st_size}: {sparse}"
    )

mount_filesystems = {}
for raw_line in android_fstab.read_text(encoding="utf-8").splitlines():
    line = raw_line.strip()
    if not line or line.startswith("#"):
        continue
    columns = line.split()
    if len(columns) != 5:
        raise SystemExit(f"invalid filtered Android fstab line: {raw_line}")
    mount_filesystems.setdefault(columns[1], set()).add(columns[2])
ambiguous = {
    mount_point: sorted(filesystems)
    for mount_point, filesystems in mount_filesystems.items()
    if len(filesystems) > 1
}
if ambiguous:
    raise SystemExit(
        "filtered Android fstab has conflicting filesystem types: "
        + ", ".join(
            f"{mount_point}={filesystems}"
            for mount_point, filesystems in sorted(ambiguous.items())
        )
    )
for required_mount in ("/system", "/data", "/metadata"):
    if required_mount not in mount_filesystems:
        raise SystemExit(f"filtered Android fstab is missing {required_mount}: {android_fstab}")

print(f"validated direct Android images: raw={raw.stat().st_size} sparse={sparse.stat().st_size}")
PY
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

copy_product_apex_dir() {
    local product="$1"
    local product_dir="$AOSP_PRODUCT_ROOT/$product"
    local dst="$PACKAGE_DIR/products/microdroid/$product/apex_dir"

    copy_dir "$product_dir/system/apex" "$dst/system/apex"
    copy_dir "$product_dir/system_ext/apex" "$dst/system_ext/apex"
    copy_dir "$product_dir/vendor/apex" "$dst/vendor/apex"
    copy_dir "$product_dir/apex" "$dst/apex"
}

copy_selected_host_tools() {
    copy_file "$AOSP_HOST_ROOT/linux-x86/bin/adb" "$PACKAGE_DIR/host-tools/linux-x86_64/bin/adb"
    copy_file "$AOSP_HOST_ROOT/linux-x86/bin/avbtool" "$PACKAGE_DIR/host-tools/linux-x86_64/bin/avbtool"
    copy_file "$AOSP_HOST_ROOT/linux-x86/bin/lpmake" "$PACKAGE_DIR/host-tools/linux-x86_64/bin/lpmake"
    copy_file "$AOSP_HOST_ROOT/linux-x86/bin/simg2img" "$PACKAGE_DIR/host-tools/linux-x86_64/bin/simg2img"
    copy_file "$AOSP_HOST_ROOT/linux_musl-arm64/bin/adb" "$PACKAGE_DIR/host-tools/linux-arm64/bin/adb"
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
Android direct-boot data profile: $DATA_ENCRYPTION
Android direct-boot fstab suffix: $FSTAB_SUFFIX

Contents:
- products/android/<product>/images: AOSP top-level boot/kernel/vbmeta/super/userdata/partition images.
- products/android/<product>/direct-linux: one validated Android-sparse aggregate disk, initrd, fstab DT, helper partitions, and kernel for direct boot. Expand it into a private writable filesystem-sparse raw overlay before launch.
- products/android/<product>/meta: product metadata useful for bring-up and diffing.
- products/microdroid/<product>/com.android.virt: Microdroid payloads from each product's com.android.virt apex.
- products/microdroid/<product>/apex_dir: product-specific mounted APEX runtime tree required by Microdroid.
- products/microdroid/soong/<arch>: architecture-specific Microdroid kernel/initrd/super/vbmeta/json/fstab.
- host/linux-x86_64: current bscp Linux host runtime only.
- host-tools: selected AOSP tools that exist in this workspace, not full host output trees.

This package was produced from existing AOSP artifacts without a clean or full rebuild. Direct-boot
aggregate disks were regenerated from the packaged product images and structurally validated.
If the direct-boot data profile is 'none', the package is development-only and must not be
represented as providing Android at-rest data encryption.
This package was produced by copying existing artifacts only. No AOSP clean or full rebuild was run.
The default ARM product is vsoc_arm64_only because Apple Silicon/HVF cannot execute
the AArch32 userspace binaries included by the mixed-ABI vsoc_arm64 product.
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
    copy_android_direct_linux "$product"
    copy_product_apex_dir "$product"
done
copy_microdroid_soong_arch "x86_64" "android_x86_64_silvermont"
copy_microdroid_soong_arch "arm64" "android_arm64_armv8-a_cortex-a53"
copy_host_runtime
copy_selected_host_tools
write_summary
create_archive

du -sh "$PACKAGE_DIR"
if [[ -f "$PACKAGE_DIR/archive.path" ]]; then
    du -h "$(cat "$PACKAGE_DIR/archive.path")"
fi
echo "Package directory: $PACKAGE_DIR"
