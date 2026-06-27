#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

PRODUCT_DIR="/opt/workspace/aosp/out/target/product/vsoc_x86_64"
DIST_ROOT="$REPO_ROOT/out/dist"
WORK_DIR="$REPO_ROOT/out/android-linux"
LOG_DIR="$REPO_ROOT/out/dist/logs/android-linux"
MODE="headless"
MEM="4096"
CPUS="4"
CID="100"
TIMEOUT_SECS="0"
RUN_VM=1
DRY_RUN=0
KEEP_GOING=0
GPU_GUEST_ANGLE=0
GPU_HOST_SWIFTSHADER=0
EXTRA_BOOTCONFIG=()
EXTRA_CROSVM_ARGS=()

usage() {
    cat <<EOF
Usage: $0 [options] [-- extra-crosvm-args...]

Options:
  --product-dir DIR     AOSP product output (default: $PRODUCT_DIR)
  --dist-root DIR       Built bscp dist root (default: $DIST_ROOT)
  --work-dir DIR        Runtime image/cache dir (default: $WORK_DIR)
  --log-dir DIR         Log dir (default: $LOG_DIR)
  --mode MODE           headless or gpu (default: $MODE)
  --mem MiB             Guest memory (default: $MEM)
  --cpus N              Guest vCPU count (default: $CPUS)
  --cid N               Guest vsock CID (default: $CID)
  --timeout-secs N      Kill crosvm after N seconds; 0 disables timeout
  --bootconfig K=V      Append a bootconfig key/value
  --gpu-guest-angle     Use guest ANGLE EGL with gfxstream-vulkan contexts
  --gpu-host-swiftshader
                        Force ANGLE's staged SwiftShader Vulkan ICD in gpu mode
  --no-run              Prepare disk/initrd but do not launch crosvm
  --dry-run             Validate inputs and print command without preparing
  --help                Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --product-dir) PRODUCT_DIR="$2"; shift 2 ;;
        --dist-root) DIST_ROOT="$2"; shift 2 ;;
        --work-dir) WORK_DIR="$2"; shift 2 ;;
        --log-dir) LOG_DIR="$2"; shift 2 ;;
        --mode) MODE="$2"; shift 2 ;;
        --mem) MEM="$2"; shift 2 ;;
        --cpus) CPUS="$2"; shift 2 ;;
        --cid) CID="$2"; shift 2 ;;
        --timeout-secs) TIMEOUT_SECS="$2"; shift 2 ;;
        --bootconfig) EXTRA_BOOTCONFIG+=("$2"); shift 2 ;;
        --gpu-guest-angle) GPU_GUEST_ANGLE=1; shift ;;
        --gpu-host-swiftshader) GPU_HOST_SWIFTSHADER=1; shift ;;
        --no-run) RUN_VM=0; shift ;;
        --dry-run) DRY_RUN=1; RUN_VM=0; shift ;;
        --help|-h) usage; exit 0 ;;
        --) shift; EXTRA_CROSVM_ARGS+=("$@"); break ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

if [[ "$MODE" != "headless" && "$MODE" != "gpu" ]]; then
    echo "Error: --mode must be headless or gpu" >&2
    exit 2
fi
FIFO_DIR="$WORK_DIR/hvc"

CROSVM_BIN="$DIST_ROOT/linux/bin/crosvm"
KERNEL="$PRODUCT_DIR/kernel"
RAMDISK="$PRODUCT_DIR/ramdisk.img"
VENDOR_RAMDISK="$PRODUCT_DIR/vendor_ramdisk.img"
VENDOR_BOOTCONFIG="$PRODUCT_DIR/vendor-bootconfig.img"
ANDROID_FSTAB="$PRODUCT_DIR/vendor/etc/fstab.cf.f2fs.hctr2"
DISK_IMAGE="$WORK_DIR/aggregate_android.img"
INITRD_IMAGE="$WORK_DIR/initrd_android.img"
ANDROID_DT_FSTAB="$WORK_DIR/android_fstab.dt"
MISC_IMAGE="$WORK_DIR/misc.img"
METADATA_IMAGE="$WORK_DIR/metadata.img"
FRP_IMAGE="$WORK_DIR/factory_reset_protected.img"
MKFS_EXT4="${MKFS_EXT4:-/usr/sbin/mkfs.ext4}"
LZ4_BIN="${LZ4_BIN:-/opt/workspace/aosp/out/host/linux-x86/bin/lz4}"

require_file() {
    local path="$1"
    local label="$2"
    if [[ ! -f "$path" ]]; then
        echo "Error: missing $label: $path" >&2
        exit 1
    fi
}

require_file "$CROSVM_BIN" "crosvm"
require_file "$KERNEL" "kernel"
require_file "$RAMDISK" "ramdisk"
require_file "$VENDOR_RAMDISK" "vendor ramdisk"
require_file "$VENDOR_BOOTCONFIG" "vendor bootconfig"
require_file "$ANDROID_FSTAB" "Android fstab"
require_file "$LZ4_BIN" "lz4"
for image in boot.img vendor_boot.img vbmeta.img vbmeta_system.img super.img userdata.img; do
    require_file "$PRODUCT_DIR/$image" "$image"
done

build_bootconfig_args() {
    local boot_device="$1"
    BOOTCONFIG_ARGS=(
        "androidboot.hardware=cutf_cvm"
        "androidboot.selinux=permissive"
        "androidboot.serialno=CUTTLEFISHCVD01"
        "androidboot.ddr_size=${MEM}MB"
        "androidboot.lcd_density=320"
        "androidboot.setupwizard_mode=DISABLED"
        "androidboot.enable_bootanimation=1"
        "androidboot.fstab_suffix=cf.f2fs.hctr2"
        "androidboot.boot_devices=$boot_device"
        "androidboot.hypervisor.version=crosvm"
        "androidboot.hypervisor.vm.supported=1"
        "androidboot.hypervisor.protected_vm.supported=0"
        "androidboot.openthread_node_id=1"
        "androidboot.vsock_lights_port=6900"
        "androidboot.vsock_lights_cid=$CID"
        "androidboot.vendor.apex.com.android.hardware.keymint=com.android.hardware.keymint.rust_nonsecure"
        "androidboot.vendor.apex.com.android.hardware.gatekeeper=com.android.hardware.gatekeeper.nonsecure"
    )

    if [[ "$MODE" == "headless" ]]; then
        BOOTCONFIG_ARGS+=(
            "androidboot.cpuvulkan.version=4202496"
            "androidboot.hardware.gralloc=minigbm"
            "androidboot.hardware.hwcomposer=ranchu"
            "androidboot.hardware.hwcomposer.display_finder_mode=drm"
            "androidboot.hardware.hwcomposer.display_framebuffer_format=rgba"
            "androidboot.hardware.egl=angle"
            "androidboot.hardware.vulkan=pastel"
            "androidboot.opengles.version=196609"
        )
    else
        local egl_impl="emulation"
        if [[ "$GPU_GUEST_ANGLE" -eq 1 ]]; then
            egl_impl="angle"
        fi
        BOOTCONFIG_ARGS+=(
            "androidboot.cpuvulkan.version=0"
            "androidboot.hardware.gralloc=minigbm"
            "androidboot.hardware.hwcomposer=ranchu"
            "androidboot.hardware.hwcomposer.display_finder_mode=drm"
            "androidboot.hardware.hwcomposer.display_framebuffer_format=rgba"
            "androidboot.hardware.egl=$egl_impl"
            "androidboot.hardware.vulkan=ranchu"
            "androidboot.hardware.gltransport=virtio-gpu-asg"
            "androidboot.opengles.version=196609"
        )
    fi

    BOOTCONFIG_ARGS+=("${EXTRA_BOOTCONFIG[@]}")
}

prepare_writable_images() {
    mkdir -p "$WORK_DIR" "$LOG_DIR" "$FIFO_DIR"

    if [[ ! -f "$MISC_IMAGE" ]]; then
        truncate -s 64M "$MISC_IMAGE"
    fi
    if [[ ! -f "$METADATA_IMAGE" ]]; then
        if [[ ! -x "$MKFS_EXT4" ]]; then
            echo "Error: mkfs.ext4 not found at $MKFS_EXT4" >&2
            exit 1
        fi
        truncate -s 64M "$METADATA_IMAGE"
        "$MKFS_EXT4" -F -q "$METADATA_IMAGE"
    fi
    if [[ ! -f "$FRP_IMAGE" ]]; then
        truncate -s 1M "$FRP_IMAGE"
    fi
}

prepare_hvc_inputs() {
    mkdir -p "$FIFO_DIR"
    local port
    for port in keymaster gatekeeper bt gnss location confui uwb oemlock keymint nfc sensors mcu_control mcu_uart; do
        : >"$FIFO_DIR/${port}.in"
    done
}

write_initrd() {
    local extra_dir
    local extra_cpio_lz4
    extra_dir="$(mktemp -d "$WORK_DIR/android-initrd-extra.XXXXXX")"
    extra_cpio_lz4="$WORK_DIR/android_fstab_extra.cpio.lz4"
    trap 'rm -rf "$extra_dir"' RETURN

    install -d \
        "$extra_dir/first_stage_ramdisk" \
        "$extra_dir/first_stage_ramdisk/system/etc" \
        "$extra_dir/system/etc"
    for fstab_name in fstab.cf.f2fs.hctr2 fstab.cutf_cvm; do
        install -m 0644 "$ANDROID_FSTAB" "$extra_dir/$fstab_name"
        install -m 0644 "$ANDROID_FSTAB" "$extra_dir/first_stage_ramdisk/$fstab_name"
        install -m 0644 "$ANDROID_FSTAB" "$extra_dir/first_stage_ramdisk/system/etc/$fstab_name"
        install -m 0644 "$ANDROID_FSTAB" "$extra_dir/system/etc/$fstab_name"
    done

    (
        cd "$extra_dir"
        find . | LC_ALL=C sort | cpio -o -H newc --owner 0:0 2>/dev/null
    ) | "$LZ4_BIN" -l -f - "$extra_cpio_lz4" >/dev/null

    python3 - "$INITRD_IMAGE" "$RAMDISK" "$VENDOR_RAMDISK" "$extra_cpio_lz4" "$VENDOR_BOOTCONFIG" "${BOOTCONFIG_ARGS[@]}" <<'PY'
import pathlib
import struct
import sys

out_path = pathlib.Path(sys.argv[1])
ramdisk_path = pathlib.Path(sys.argv[2])
vendor_ramdisk_path = pathlib.Path(sys.argv[3])
extra_cpio_lz4_path = pathlib.Path(sys.argv[4])
vendor_bootconfig_path = pathlib.Path(sys.argv[5])
extra_args = sys.argv[6:]

bootconfig = (
    "androidboot.slot_suffix=_a\n"
    "androidboot.force_normal_boot=1\n"
    "androidboot.verifiedbootstate=orange\n"
)
bootconfig += vendor_bootconfig_path.read_bytes().rstrip(b"\0").decode("utf-8")
if not bootconfig.endswith("\n"):
    bootconfig += "\n"
for arg in extra_args:
    bootconfig += arg + "\n"

ordered_keys = []
merged = {}
for raw_line in bootconfig.splitlines():
    line = raw_line.strip()
    if not line:
        continue
    key = line.split("=", 1)[0]
    if key not in merged:
        ordered_keys.append(key)
    merged[key] = line
bootconfig_bytes = ("\n".join(merged[key] for key in ordered_keys) + "\n").encode("utf-8")
checksum = sum(bootconfig_bytes) & 0xFFFFFFFF

out_path.parent.mkdir(parents=True, exist_ok=True)
with out_path.open("wb") as out_file:
    out_file.write(ramdisk_path.read_bytes())
    out_file.write(vendor_ramdisk_path.read_bytes())
    out_file.write(extra_cpio_lz4_path.read_bytes())
    out_file.write(bootconfig_bytes)
    out_file.write(struct.pack("<I", len(bootconfig_bytes)))
    out_file.write(struct.pack("<I", checksum))
    out_file.write(b"#BOOTCONFIG\n")
PY
}

prepare_disk() {
    "$SCRIPT_DIR/create_cf_android_disk.py" \
        --product-dir "$PRODUCT_DIR" \
        --misc-image "$MISC_IMAGE" \
        --metadata-image "$METADATA_IMAGE" \
        --frp-image "$FRP_IMAGE" \
        --output "$DISK_IMAGE"
}

write_android_dt_fstab() {
    python3 - "$ANDROID_FSTAB" "$ANDROID_DT_FSTAB" <<'PY'
import pathlib
import sys

src = pathlib.Path(sys.argv[1])
dst = pathlib.Path(sys.argv[2])
dst.parent.mkdir(parents=True, exist_ok=True)

lines = []
for raw_line in src.read_text(encoding="utf-8").splitlines():
    line = raw_line.strip()
    if not line or line.startswith("#"):
        continue
    columns = line.split()
    if len(columns) != 5:
        raise SystemExit(f"unsupported fstab line: {raw_line}")
    if not columns[1].startswith("/"):
        continue
    if "/" in columns[1].lstrip("/"):
        continue
    lines.append("\t".join(columns))

dst.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
}

ANGLE_RUNTIME_DIR="${ANGLE_RUNTIME_DIR:-}"
if [[ "$MODE" == "gpu" && -z "$ANGLE_RUNTIME_DIR" ]]; then
    for candidate in "$DIST_ROOT/linux/gfx/angle" /opt/workspace/angle/out/*; do
        if [[ -f "$candidate/libEGL.so" && -f "$candidate/libGLESv2.so" ]]; then
            ANGLE_RUNTIME_DIR="$candidate"
            break
        fi
    done
fi

LD_PATHS=("$DIST_ROOT/linux/lib")
GPU_ARGS=(
    --gpu "displays=[[mode=windowed[1280,720],dpi=[320,320],refresh-rate=60]],backend=2D,pci-address=00:02.0"
)
if [[ "$MODE" == "gpu" ]]; then
    if [[ -z "$ANGLE_RUNTIME_DIR" ]]; then
        echo "Error: --mode gpu requires ANGLE_RUNTIME_DIR with libEGL.so and libGLESv2.so" >&2
        echo "       Current ~/angle checkout has no built runtime under /opt/workspace/angle/out." >&2
        echo "       Run scripts/build_angle_linux.sh first, or set ANGLE_RUNTIME_DIR explicitly." >&2
        exit 1
    fi
    LD_PATHS+=("$ANGLE_RUNTIME_DIR")
    export GFXSTREAM_ANGLE_ROOT="$ANGLE_RUNTIME_DIR"
    if [[ "$GPU_HOST_SWIFTSHADER" -eq 1 && -z "${VK_ICD_FILENAMES:-}" && -f "$ANGLE_RUNTIME_DIR/vk_swiftshader_icd.json" ]]; then
        export VK_ICD_FILENAMES="$ANGLE_RUNTIME_DIR/vk_swiftshader_icd.json"
    fi
    if [[ "$GPU_GUEST_ANGLE" -eq 1 ]]; then
        GPU_ARGS=(
            --gpu "backend=gfxstream,width=1280,height=720,context-types=gfxstream-vulkan:gfxstream-composer,angle=true,gles=false,vulkan=true,wsi=vk"
        )
    else
        GPU_ARGS=(
            --gpu "backend=gfxstream,width=1280,height=720,angle=true,vulkan=true,wsi=vk"
        )
    fi
fi

BOOT_DEVICE="pci0000:00/0000:00:03.0"
build_bootconfig_args "$BOOT_DEVICE"

KERNEL_PARAMS="console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init"
HVC_ARGS=(
    --serial "hardware=legacy-virtio-console,num=1,type=file,path=$LOG_DIR/hvc.txt,console=true"
    --serial "hardware=legacy-virtio-console,num=2,type=sink"
    --serial "hardware=legacy-virtio-console,num=3,type=file,path=$LOG_DIR/logcat-hvc2.txt"
    --serial "hardware=legacy-virtio-console,num=4,type=file,path=$LOG_DIR/keymaster-hvc3.txt,input=$FIFO_DIR/keymaster.in"
    --serial "hardware=legacy-virtio-console,num=5,type=file,path=$LOG_DIR/gatekeeper-hvc4.txt,input=$FIFO_DIR/gatekeeper.in"
    --serial "hardware=legacy-virtio-console,num=6,type=sink"
    --serial "hardware=legacy-virtio-console,num=7,type=sink"
    --serial "hardware=legacy-virtio-console,num=8,type=sink"
    --serial "hardware=legacy-virtio-console,num=9,type=file,path=$LOG_DIR/confui-hvc8.txt,input=$FIFO_DIR/confui.in"
    --serial "hardware=legacy-virtio-console,num=10,type=file,path=$LOG_DIR/uwb-hvc9.txt,input=$FIFO_DIR/uwb.in"
    --serial "hardware=legacy-virtio-console,num=11,type=file,path=$LOG_DIR/oemlock-hvc10.txt,input=$FIFO_DIR/oemlock.in"
    --serial "hardware=legacy-virtio-console,num=12,type=file,path=$LOG_DIR/keymint-hvc11.txt,input=$FIFO_DIR/keymint.in"
    --serial "hardware=legacy-virtio-console,num=13,type=sink"
    --serial "hardware=legacy-virtio-console,num=14,type=file,path=$LOG_DIR/sensors-hvc13.txt,input=$FIFO_DIR/sensors.in"
    --serial "hardware=legacy-virtio-console,num=15,type=file,path=$LOG_DIR/mcu-control-hvc14.txt,input=$FIFO_DIR/mcu_control.in"
    --serial "hardware=legacy-virtio-console,num=16,type=file,path=$LOG_DIR/mcu-uart-hvc15.txt,input=$FIFO_DIR/mcu_uart.in"
)

CROSVM_CMD=(
    "$CROSVM_BIN"
    --log-level info
    run
    --disable-sandbox
    --cid "$CID"
    --mem "$MEM"
    --cpus "$CPUS"
    --no-balloon
    --no-usb
    "${GPU_ARGS[@]}"
    --block "path=$DISK_IMAGE,ro=false,lock=false,sparse=false,pci-address=00:03.0"
    --serial "type=file,path=$LOG_DIR/serial.txt,hardware=serial,num=1,earlycon=true"
    --serial "type=sink,hardware=serial,num=2"
    "${HVC_ARGS[@]}"
    --android-fstab "$ANDROID_DT_FSTAB"
    --initrd "$INITRD_IMAGE"
    --params "$KERNEL_PARAMS"
    "${EXTRA_CROSVM_ARGS[@]}"
    "$KERNEL"
)

if [[ "$DRY_RUN" -eq 1 ]]; then
    printf 'LD_LIBRARY_PATH=%s\n' "$(IFS=:; echo "${LD_PATHS[*]}")"
    if [[ -n "${VK_ICD_FILENAMES:-}" ]]; then
        printf 'VK_ICD_FILENAMES=%s\n' "$VK_ICD_FILENAMES"
    fi
    if [[ -n "${GFXSTREAM_ANGLE_ROOT:-}" ]]; then
        printf 'GFXSTREAM_ANGLE_ROOT=%s\n' "$GFXSTREAM_ANGLE_ROOT"
    fi
    printf 'BOOTCONFIG:\n'
    printf '  %s\n' "${BOOTCONFIG_ARGS[@]}"
    printf 'CROSVM:\n'
    printf '  %q' "${CROSVM_CMD[@]}"
    printf '\n'
    exit 0
fi

prepare_writable_images
prepare_hvc_inputs
write_android_dt_fstab
write_initrd
prepare_disk

if [[ "$RUN_VM" -eq 0 ]]; then
    echo "Prepared:"
    echo "  disk: $DISK_IMAGE"
    echo "  initrd: $INITRD_IMAGE"
    exit 0
fi

export LD_LIBRARY_PATH="$(IFS=:; echo "${LD_PATHS[*]}")${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
mkdir -p "$LOG_DIR"
: >"$LOG_DIR/stdout.txt"
: >"$LOG_DIR/stderr.txt"
: >"$LOG_DIR/serial.txt"
: >"$LOG_DIR/hvc.txt"
: >"$LOG_DIR/logcat-hvc2.txt"
echo "Launching Android; logs: $LOG_DIR"
if [[ "$TIMEOUT_SECS" != "0" ]]; then
    timeout "$TIMEOUT_SECS" "${CROSVM_CMD[@]}" >"$LOG_DIR/stdout.txt" 2>"$LOG_DIR/stderr.txt" || KEEP_GOING=$?
    if [[ "$KEEP_GOING" -ne 0 && "$KEEP_GOING" -ne 124 ]]; then
        exit "$KEEP_GOING"
    fi
else
    "${CROSVM_CMD[@]}" >"$LOG_DIR/stdout.txt" 2>"$LOG_DIR/stderr.txt"
fi
