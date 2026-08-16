#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/start_cf_host_daemons.sh"

PRODUCT_DIR="/opt/workspace/aosp/out/target/product/vsoc_x86_64"
DIST_ROOT="$REPO_ROOT/out/dist"
WORK_DIR="$REPO_ROOT/out/android-linux"
LOG_DIR="$REPO_ROOT/out/dist/logs/android-linux"
MODE="headless"
MEM="4096"
CPUS="4"
CID="100"
TIMEOUT_SECS="0"
BOOT_DEVICE="${CROSVM_ANDROID_BOOT_DEVICE:-}"
REBUILD_DISK=1
RUN_VM=1
DRY_RUN=0
KEEP_GOING=0
GPU_GUEST_ANGLE=0
GPU_HOST_SWIFTSHADER=0
GPU_HOST_VISIBLE_COHERENT="${CROSVM_ANDROID_HOST_VISIBLE_COHERENT:-0}"
X11_GFXSTREAM_SUBWINDOW="${CROSVM_X11_GFXSTREAM_SUBWINDOW:-auto}"
X_DISPLAY="${DISPLAY:-}"
DISABLE_AVB=0
DYNAMIC_FS_TYPE=""
DATA_FS_TYPE="${CROSVM_ANDROID_DATA_FS_TYPE:-auto}"
DATA_ENCRYPTION="${CROSVM_ANDROID_DATA_ENCRYPTION:-metadata}"
FSTAB_SUFFIX="${CROSVM_ANDROID_FSTAB_SUFFIX:-cf.f2fs.hctr2}"
ENABLE_NETWORK=1
MOBILE_TAP="${CROSVM_ANDROID_MOBILE_TAP:-}"
ETHERNET_TAP="${CROSVM_ANDROID_ETHERNET_TAP:-}"
MOBILE_HOST_IP="${CROSVM_ANDROID_MOBILE_HOST_IP:-192.168.96.1}"
ETHERNET_HOST_IP="${CROSVM_ANDROID_ETHERNET_HOST_IP:-192.168.97.1}"
NETWORK_NETMASK="${CROSVM_ANDROID_NETMASK:-255.255.255.0}"
ENABLE_BLUETOOTH=1
ENABLE_NFC=1
ENABLE_MODEM=1
MODEM_INSTANCE_NUM=1
MODEM_SIM_TYPE=1
MODEM_BASE_PORT=9600
BT_HCI_PORT="${CROSVM_ANDROID_BT_HCI_PORT:-7300}"
CASIMIR_NCI_PORT="${CROSVM_ANDROID_CASIMIR_NCI_PORT:-7800}"
CASIMIR_RF_PORT="${CROSVM_ANDROID_CASIMIR_RF_PORT:-7900}"
AOSP_HOST_BIN="${AOSP_HOST_BIN:-/opt/workspace/aosp/out/host/linux-x86/bin}"
HOST_DAEMON_PIDS=()
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
  --kernel FILE         Kernel image override (default: PRODUCT_DIR/kernel)
  --ramdisk FILE        Ramdisk image override (default: PRODUCT_DIR/ramdisk.img)
  --vendor-ramdisk FILE Vendor ramdisk image override (default: PRODUCT_DIR/vendor_ramdisk.img)
  --vendor-bootconfig FILE
                        Vendor bootconfig override (default: PRODUCT_DIR/vendor-bootconfig.img)
  --mode MODE           headless or gpu (default: $MODE)
  --mem MiB             Guest memory (default: $MEM)
  --cpus N              Guest vCPU count (default: $CPUS)
  --cid N               Guest vsock CID (default: $CID)
  --timeout-secs N      Kill crosvm after N seconds; 0 disables timeout
  --boot-device DEVICE  Android boot device; auto-detected from the guest kernel
                        (ARM64 crosvm: 10000.pci; x86_64: pci0000:00/0000:00:03.0)
  --bootconfig K=V      Append a bootconfig key/value
  --gpu-guest-angle     Use guest ANGLE EGL with gfxstream-vulkan contexts
  --gpu-host-swiftshader
                        Force ANGLE's staged SwiftShader Vulkan ICD in gpu mode
  --gpu-host-visible-coherent
                        Enable gfxstream VulkanAllocateHostMemory for guest host-visible coherent testing
  --x11-gfxstream-subwindow
                        Use gfxstream's native X11 subwindow instead of XShm readback
  --x-display DISPLAY   X11 display for the host GPU window (default: DISPLAY env)
  --disable-avb         Strip avb/avb_keys flags from the generated direct-boot DT fstab
  --dynamic-fs-type FS  Keep only this fs type for dynamic partition fstab duplicates
  --data-fs-type FS     Keep only this fs type for /data fstab duplicates; auto uses userdata.img
  --data-encryption MODE
                        /data encryption profile: metadata or none (default: $DATA_ENCRYPTION).
                        none is for explicitly labeled development artifacts only and mounts
                        /data in first stage to match crosvm's DT-fstab semantics.
  --fstab-suffix NAME   Boot fstab suffix (default: $FSTAB_SUFFIX). Use a dedicated suffix
                        when the generated profile must not merge with the vendor fstab.
  --no-network          Do not add Cuttlefish-compatible virtio-net devices
  --mobile-tap NAME     Existing first/mobile TAP to pass to crosvm
  --ethernet-tap NAME   Existing second/eth1 TAP to pass to crosvm
  --mobile-host-ip IP   Host IP for the first/mobile crosvm TAP (default: $MOBILE_HOST_IP)
  --ethernet-host-ip IP Host IP for the second/eth1 crosvm TAP (default: $ETHERNET_HOST_IP)
  --network-netmask IP  Netmask for generated TAP devices (default: $NETWORK_NETMASK)
  --no-bluetooth        Do not wire /dev/hvc5 or start RootCanal/bt_connector
  --no-nfc              Do not wire /dev/hvc12 or start Casimir/nfc_connector
  --no-modem            Do not start modem_simulator or pass modem bootconfig
  --bt-hci-port PORT    RootCanal HCI port (default: $BT_HCI_PORT)
  --casimir-nci-port PORT
                        Casimir NCI port (default: $CASIMIR_NCI_PORT)
  --casimir-rf-port PORT
                        Casimir RF port (default: $CASIMIR_RF_PORT)
  --aosp-host-bin DIR   AOSP host bin dir for CF daemons (default: $AOSP_HOST_BIN)
  --reuse-disk          Rebuild initrd/fstab but keep an existing aggregate disk
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
        --kernel) KERNEL_OVERRIDE="$2"; shift 2 ;;
        --ramdisk) RAMDISK_OVERRIDE="$2"; shift 2 ;;
        --vendor-ramdisk) VENDOR_RAMDISK_OVERRIDE="$2"; shift 2 ;;
        --vendor-bootconfig) VENDOR_BOOTCONFIG_OVERRIDE="$2"; shift 2 ;;
        --mode) MODE="$2"; shift 2 ;;
        --mem) MEM="$2"; shift 2 ;;
        --cpus) CPUS="$2"; shift 2 ;;
        --cid) CID="$2"; shift 2 ;;
        --timeout-secs) TIMEOUT_SECS="$2"; shift 2 ;;
        --boot-device) BOOT_DEVICE="$2"; shift 2 ;;
        --bootconfig) EXTRA_BOOTCONFIG+=("$2"); shift 2 ;;
        --gpu-guest-angle) GPU_GUEST_ANGLE=1; shift ;;
        --gpu-host-swiftshader) GPU_HOST_SWIFTSHADER=1; shift ;;
        --gpu-host-visible-coherent) GPU_HOST_VISIBLE_COHERENT=1; shift ;;
        --x11-gfxstream-subwindow) X11_GFXSTREAM_SUBWINDOW=1; shift ;;
        --x-display) X_DISPLAY="$2"; shift 2 ;;
        --disable-avb) DISABLE_AVB=1; shift ;;
        --dynamic-fs-type) DYNAMIC_FS_TYPE="$2"; shift 2 ;;
        --data-fs-type) DATA_FS_TYPE="$2"; shift 2 ;;
        --data-encryption) DATA_ENCRYPTION="$2"; shift 2 ;;
        --fstab-suffix) FSTAB_SUFFIX="$2"; shift 2 ;;
        --no-network) ENABLE_NETWORK=0; shift ;;
        --mobile-tap) MOBILE_TAP="$2"; shift 2 ;;
        --ethernet-tap) ETHERNET_TAP="$2"; shift 2 ;;
        --mobile-host-ip) MOBILE_HOST_IP="$2"; shift 2 ;;
        --ethernet-host-ip) ETHERNET_HOST_IP="$2"; shift 2 ;;
        --network-netmask) NETWORK_NETMASK="$2"; shift 2 ;;
        --no-bluetooth) ENABLE_BLUETOOTH=0; shift ;;
        --no-nfc) ENABLE_NFC=0; shift ;;
        --no-modem) ENABLE_MODEM=0; shift ;;
        --bt-hci-port) BT_HCI_PORT="$2"; shift 2 ;;
        --casimir-nci-port) CASIMIR_NCI_PORT="$2"; shift 2 ;;
        --casimir-rf-port) CASIMIR_RF_PORT="$2"; shift 2 ;;
        --aosp-host-bin) AOSP_HOST_BIN="$2"; shift 2 ;;
        --reuse-disk) REBUILD_DISK=0; shift ;;
        --no-run) RUN_VM=0; shift ;;
        --dry-run) DRY_RUN=1; RUN_VM=0; shift ;;
        --help|-h) usage; exit 0 ;;
        --) shift; EXTRA_CROSVM_ARGS+=("$@"); break ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

resolve_cf_host_features() {
    cf_probe_host_daemons "$ENABLE_BLUETOOTH" "$ENABLE_NFC" "$DIST_ROOT/linux/bin" "$ENABLE_MODEM"
    ENABLE_BLUETOOTH="${CF_HOST_BLUETOOTH_ENABLED:-0}"
    ENABLE_NFC="${CF_HOST_NFC_ENABLED:-0}"
    ENABLE_MODEM="${CF_HOST_MODEM_ENABLED:-0}"
}

cf_modem_vsock_port() {
    local guest_cid="$1"
    echo $((MODEM_BASE_PORT + guest_cid - 3))
}

ensure_network_taps() {
    if [[ "$ENABLE_NETWORK" -ne 1 || -n "$MOBILE_TAP" || -n "$ETHERNET_TAP" ]]; then
        return 0
    fi
    local tap_exports
    if ! tap_exports="$("$SCRIPT_DIR/create_cf_taps.sh")"; then
        echo "Error: could not create Cuttlefish TAP devices; run scripts/create_cf_taps.sh manually or pass --mobile-tap/--ethernet-tap." >&2
        exit 1
    fi
    # shellcheck disable=SC1090
    eval "$tap_exports"
}

if [[ "$MODE" != "headless" && "$MODE" != "gpu" ]]; then
    echo "Error: --mode must be headless or gpu" >&2
    exit 2
fi
if [[ "$DATA_ENCRYPTION" != "metadata" && "$DATA_ENCRYPTION" != "none" ]]; then
    echo "Error: --data-encryption must be metadata or none" >&2
    exit 2
fi
if [[ ! "$FSTAB_SUFFIX" =~ ^[A-Za-z0-9._-]+$ ]]; then
    echo "Error: --fstab-suffix must contain only letters, digits, dot, underscore, or hyphen" >&2
    exit 2
fi
FIFO_DIR="$WORK_DIR/hvc"

CROSVM_BIN="${CROSVM_BIN:-$DIST_ROOT/linux/bin/crosvm}"
KERNEL="${KERNEL_OVERRIDE:-$PRODUCT_DIR/kernel}"
RAMDISK="${RAMDISK_OVERRIDE:-$PRODUCT_DIR/ramdisk.img}"
VENDOR_RAMDISK="${VENDOR_RAMDISK_OVERRIDE:-$PRODUCT_DIR/vendor_ramdisk.img}"
VENDOR_BOOTCONFIG="${VENDOR_BOOTCONFIG_OVERRIDE:-$PRODUCT_DIR/vendor-bootconfig.img}"
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

if [[ -z "$DYNAMIC_FS_TYPE" && -f "$PRODUCT_DIR/misc_info.txt" ]]; then
    DYNAMIC_FS_TYPE="$(sed -n 's/^system_fs_type=//p' "$PRODUCT_DIR/misc_info.txt" | head -n 1)"
fi
if [[ -z "$DYNAMIC_FS_TYPE" ]]; then
    DYNAMIC_FS_TYPE="$(
        awk '
            $1 ~ /^(odm|odm_dlkm|product|system|system_ext|system_dlkm|vendor|vendor_dlkm)$/ {
                if (!($3 in types)) {
                    types[$3] = 1
                    count += 1
                }
            }
            END {
                if (count == 1) {
                    for (type in types) {
                        print type
                    }
                }
            }
        ' "$ANDROID_FSTAB"
    )"
fi
if [[ -z "$DYNAMIC_FS_TYPE" ]]; then
    echo "Error: cannot select one dynamic partition filesystem type from $ANDROID_FSTAB." >&2
    echo "       Pass --dynamic-fs-type or provide system_fs_type in misc_info.txt." >&2
    exit 1
fi

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
        "androidboot.fstab_suffix=$FSTAB_SUFFIX"
        "androidboot.boot_devices=$boot_device"
        "androidboot.hypervisor.version=crosvm"
        "androidboot.hypervisor.vm.supported=1"
        "androidboot.hypervisor.protected_vm.supported=0"
        "androidboot.openthread_node_id=1"
        "androidboot.vsock_lights_port=6900"
        "androidboot.vsock_lights_cid=$CID"
        "androidboot.vendor.apex.com.android.hardware.graphics.composer=com.android.hardware.graphics.composer.ranchu"
        "androidboot.vendor.apex.com.google.emulated.camera.provider.hal=com.google.emulated.camera.provider.hal"
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

    if [[ "$ENABLE_MODEM" -eq 1 ]]; then
        BOOTCONFIG_ARGS+=(
            "androidboot.modem_simulator_ports=$(cf_modem_vsock_port "$CID")"
        )
    fi

    if [[ "${#EXTRA_BOOTCONFIG[@]}" -gt 0 ]]; then
        BOOTCONFIG_ARGS+=("${EXTRA_BOOTCONFIG[@]}")
    fi
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
    for port in keymaster gatekeeper gnss location confui uwb oemlock keymint sensors mcu_control mcu_uart; do
        : >"$FIFO_DIR/${port}.in"
    done
}

stop_host_daemons() {
    cf_stop_host_daemons
}

start_host_daemons() {
    cf_start_host_daemons "$WORK_DIR" "$LOG_DIR" "$DIST_ROOT/linux/bin" \
        "$ENABLE_BLUETOOTH" "$ENABLE_NFC" "$BT_HCI_PORT" "$CASIMIR_NCI_PORT" "$CASIMIR_RF_PORT" \
        "$ENABLE_MODEM" "$CID" "$MODEM_INSTANCE_NUM" "$MODEM_SIM_TYPE" \
        "$ETHERNET_HOST_IP" "192.168.97.2" "30" "8.8.8.8"
    HOST_DAEMON_PIDS=("${CF_HOST_DAEMON_PIDS[@]:-}")
    ENABLE_BLUETOOTH="${CF_HOST_BLUETOOTH_ENABLED:-0}"
    ENABLE_NFC="${CF_HOST_NFC_ENABLED:-0}"
    ENABLE_MODEM="${CF_HOST_MODEM_ENABLED:-0}"
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
    for fstab_name in "fstab.$FSTAB_SUFFIX"; do
        install -m 0644 "$ANDROID_DT_FSTAB" "$extra_dir/$fstab_name"
        install -m 0644 "$ANDROID_DT_FSTAB" "$extra_dir/first_stage_ramdisk/$fstab_name"
        install -m 0644 "$ANDROID_DT_FSTAB" "$extra_dir/first_stage_ramdisk/system/etc/$fstab_name"
        install -m 0644 "$ANDROID_DT_FSTAB" "$extra_dir/system/etc/$fstab_name"
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
    local data_fs_type="$DATA_FS_TYPE"
    if [[ "$data_fs_type" == "auto" ]]; then
        data_fs_type=""
        if [[ -f "$PRODUCT_DIR/misc_info.txt" ]]; then
            data_fs_type="$(sed -n 's/^userdata_fs_type=//p' "$PRODUCT_DIR/misc_info.txt" | head -n 1)"
        fi
        if [[ -z "$data_fs_type" ]] && file "$PRODUCT_DIR/userdata.img" | grep -qi 'F2FS'; then
            data_fs_type="f2fs"
        elif [[ -z "$data_fs_type" ]] && file "$PRODUCT_DIR/userdata.img" | grep -qi 'ext[234]'; then
            data_fs_type="ext4"
        elif [[ -z "$data_fs_type" ]]; then
            data_fs_type=""
        fi
    fi
    python3 - \
        "$ANDROID_FSTAB" \
        "$ANDROID_DT_FSTAB" \
        "$DISABLE_AVB" \
        "$DYNAMIC_FS_TYPE" \
        "$data_fs_type" \
        "$DATA_ENCRYPTION" <<'PY'
import pathlib
import sys

src = pathlib.Path(sys.argv[1])
dst = pathlib.Path(sys.argv[2])
disable_avb = sys.argv[3] == "1"
dynamic_fs_type = sys.argv[4]
data_fs_type = sys.argv[5]
data_encryption = sys.argv[6]
dst.parent.mkdir(parents=True, exist_ok=True)

lines = []
mount_filesystems = {}
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
    fs_mgr_flags = set(columns[4].split(","))
    if "recoveryonly" in fs_mgr_flags and "first_stage_mount" not in fs_mgr_flags:
        continue
    if dynamic_fs_type and columns[0] in {
        "odm", "odm_dlkm", "product", "system", "system_ext", "system_dlkm", "vendor", "vendor_dlkm",
    } and columns[2] != dynamic_fs_type:
        continue
    if data_fs_type and columns[1] == "/data" and columns[2] != data_fs_type:
        continue
    if columns[1] == "/data" and data_encryption == "none":
        mount_flags = [
            flag for flag in columns[3].split(",")
            if flag != "inlinecrypt"
        ]
        fs_mgr_flags = [
            flag for flag in columns[4].split(",")
            if not flag.startswith("keydirectory=")
            and not flag.startswith("fileencryption=")
            and flag != "latemount"
        ]
        if "first_stage_mount" not in fs_mgr_flags:
            fs_mgr_flags.append("first_stage_mount")
        columns[3] = ",".join(mount_flags)
        columns[4] = ",".join(fs_mgr_flags)
    if disable_avb:
        columns[4] = ",".join(
            flag for flag in columns[4].split(",")
            if flag != "avb" and not flag.startswith("avb=") and not flag.startswith("avb_keys=")
        )
    mount_filesystems.setdefault(columns[1], set()).add(columns[2])
    lines.append("\t".join(columns))

ambiguous = {
    mount_point: sorted(filesystems)
    for mount_point, filesystems in mount_filesystems.items()
    if len(filesystems) > 1
}
if ambiguous:
    raise SystemExit(
        "filtered fstab retains conflicting filesystem types: "
        + ", ".join(
            f"{mount_point}={filesystems}"
            for mount_point, filesystems in sorted(ambiguous.items())
        )
    )
for required_mount in ("/system", "/data", "/metadata"):
    if required_mount not in mount_filesystems:
        raise SystemExit(f"filtered fstab is missing required mount {required_mount}")

data_lines = [
    line.split()
    for line in lines
    if line.split()[1] == "/data"
]
if len(data_lines) != 1:
    raise SystemExit(f"filtered fstab must contain exactly one /data entry, found {len(data_lines)}")
data_flags = set(data_lines[0][3].split(",")) | set(data_lines[0][4].split(","))
has_encryption = (
    "inlinecrypt" in data_flags
    or any(flag.startswith("keydirectory=") for flag in data_flags)
    or any(flag.startswith("fileencryption=") for flag in data_flags)
)
if data_encryption == "metadata" and not has_encryption:
    raise SystemExit("metadata encryption profile selected but /data has no encryption flags")
if data_encryption == "none" and has_encryption:
    raise SystemExit("unencrypted development profile still contains /data encryption flags")
if data_encryption == "none":
    data_fs_mgr_flags = set(data_lines[0][4].split(","))
    if "first_stage_mount" not in data_fs_mgr_flags or "latemount" in data_fs_mgr_flags:
        raise SystemExit(
            "unencrypted development profile must mount /data in first stage "
            "so crosvm DT-fstab is not mounted twice"
        )

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
    if [[ -z "$X_DISPLAY" ]]; then
        echo "Error: --mode gpu requires DISPLAY or --x-display for the host render window" >&2
        exit 1
    fi
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
        if [[ "$X11_GFXSTREAM_SUBWINDOW" == "auto" ]]; then
            X11_GFXSTREAM_SUBWINDOW=1
        fi
        GPU_EXTERNAL_BLOB="false"
        GPU_RENDERER_FEATURES=""
        if [[ "$GPU_HOST_VISIBLE_COHERENT" != "0" ]]; then
            GPU_EXTERNAL_BLOB="true"
            GPU_RENDERER_FEATURES=",renderer-features=VulkanAllocateHostMemory:enabled"
        fi
        GPU_ARGS=(
            --gpu "backend=gfxstream,displays=[[mode=windowed[1280,720],dpi=[320,320],refresh-rate=60]],context-types=gfxstream-vulkan:gfxstream-composer,angle=true,gles=false,vulkan=true,wsi=vk,external-blob=${GPU_EXTERNAL_BLOB},udmabuf=true${GPU_RENDERER_FEATURES}"
        )
    else
        GPU_ARGS=(
            --gpu "backend=gfxstream,width=1280,height=720,angle=true,vulkan=true,wsi=vk"
        )
    fi
fi

if [[ "$ENABLE_NETWORK" -eq 1 ]] && ! env LD_LIBRARY_PATH="$(IFS=:; echo "${LD_PATHS[*]}")${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" "$CROSVM_BIN" run --help 2>&1 | grep -q -- "--net"; then
    echo "Error: $CROSVM_BIN was built without crosvm's net feature; rebuild with build_all.sh so direct Cuttlefish networking can create eth1." >&2
    echo "       Use --no-network to keep the old no-network direct boot path." >&2
    exit 1
fi

NET_ARGS=()
if [[ "$ENABLE_NETWORK" -eq 1 ]]; then
    ensure_network_taps
    if [[ -n "$MOBILE_TAP" || -n "$ETHERNET_TAP" ]]; then
        if [[ -z "$MOBILE_TAP" || -z "$ETHERNET_TAP" ]]; then
            echo "Error: --mobile-tap and --ethernet-tap must be provided together." >&2
            exit 2
        fi
        NET_ARGS=(
            --net "tap-name=${MOBILE_TAP},mac=00:1a:11:e0:cf:00,pci-address=00:01.1"
            --net "tap-name=${ETHERNET_TAP},mac=00:1a:11:e1:cf:00,pci-address=00:01.2"
        )
    else
        # Match Cuttlefish's network PCI ordering: mobile first, ethernet/eth1 second.
        NET_ARGS=(
            --net "host-ip=${MOBILE_HOST_IP},netmask=${NETWORK_NETMASK},mac=00:1a:11:e0:cf:00,pci-address=00:01.1"
            --net "host-ip=${ETHERNET_HOST_IP},netmask=${NETWORK_NETMASK},mac=00:1a:11:e1:cf:00,pci-address=00:01.2"
        )
    fi
fi

if [[ -z "$BOOT_DEVICE" ]]; then
    if file -b "$KERNEL" | grep -qi 'ARM64'; then
        # On ARM64 crosvm all PCI block devices sit below this platform bridge.
        BOOT_DEVICE="10000.pci"
    else
        BOOT_DEVICE="pci0000:00/0000:00:03.0"
    fi
fi
resolve_cf_host_features
build_bootconfig_args "$BOOT_DEVICE"

KERNEL_PARAMS="console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init"
HVC_ARGS=(
    --serial "hardware=legacy-virtio-console,num=1,type=file,path=$LOG_DIR/hvc.txt,console=true"
    --serial "hardware=legacy-virtio-console,num=2,type=sink"
    --serial "hardware=legacy-virtio-console,num=3,type=file,path=$LOG_DIR/logcat-hvc2.txt"
    --serial "hardware=legacy-virtio-console,num=4,type=file,path=$LOG_DIR/keymaster-hvc3.txt,input=$FIFO_DIR/keymaster.in"
    --serial "hardware=legacy-virtio-console,num=5,type=file,path=$LOG_DIR/gatekeeper-hvc4.txt,input=$FIFO_DIR/gatekeeper.in"
)
if [[ "$ENABLE_BLUETOOTH" -eq 1 ]]; then
    HVC_ARGS+=(
        --serial "hardware=legacy-virtio-console,num=6,type=file,path=$FIFO_DIR/bt.out,input=$FIFO_DIR/bt.in"
    )
else
    HVC_ARGS+=(--serial "hardware=legacy-virtio-console,num=6,type=sink")
fi
HVC_ARGS+=(
    --serial "hardware=legacy-virtio-console,num=7,type=sink"
    --serial "hardware=legacy-virtio-console,num=8,type=sink"
    --serial "hardware=legacy-virtio-console,num=9,type=file,path=$LOG_DIR/confui-hvc8.txt,input=$FIFO_DIR/confui.in"
    --serial "hardware=legacy-virtio-console,num=10,type=file,path=$LOG_DIR/uwb-hvc9.txt,input=$FIFO_DIR/uwb.in"
    --serial "hardware=legacy-virtio-console,num=11,type=file,path=$LOG_DIR/oemlock-hvc10.txt,input=$FIFO_DIR/oemlock.in"
    --serial "hardware=legacy-virtio-console,num=12,type=file,path=$LOG_DIR/keymint-hvc11.txt,input=$FIFO_DIR/keymint.in"
)
if [[ "$ENABLE_NFC" -eq 1 ]]; then
    HVC_ARGS+=(
        --serial "hardware=legacy-virtio-console,num=13,type=file,path=$FIFO_DIR/nfc.out,input=$FIFO_DIR/nfc.in"
    )
else
    HVC_ARGS+=(--serial "hardware=legacy-virtio-console,num=13,type=sink")
fi
HVC_ARGS+=(
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
)
if [[ "$MODE" == "gpu" ]]; then
    CROSVM_CMD+=(--x-display "$X_DISPLAY")
fi
CROSVM_CMD+=("${GPU_ARGS[@]}")
if [[ "${#NET_ARGS[@]}" -gt 0 ]]; then
    CROSVM_CMD+=("${NET_ARGS[@]}")
fi
CROSVM_CMD+=(
    --block "path=$DISK_IMAGE,ro=false,lock=false,sparse=false,pci-address=00:03.0"
    --serial "type=file,path=$LOG_DIR/serial.txt,hardware=serial,num=1,earlycon=true"
    --serial "type=sink,hardware=serial,num=2"
    "${HVC_ARGS[@]}"
    --android-fstab "$ANDROID_DT_FSTAB"
    --initrd "$INITRD_IMAGE"
    --params "$KERNEL_PARAMS"
)
if [[ "${#EXTRA_CROSVM_ARGS[@]}" -gt 0 ]]; then
    CROSVM_CMD+=("${EXTRA_CROSVM_ARGS[@]}")
fi
CROSVM_CMD+=("$KERNEL")

if [[ "$DRY_RUN" -eq 1 ]]; then
    printf 'LD_LIBRARY_PATH=%s\n' "$(IFS=:; echo "${LD_PATHS[*]}")"
    if [[ -n "${VK_ICD_FILENAMES:-}" ]]; then
        printf 'VK_ICD_FILENAMES=%s\n' "$VK_ICD_FILENAMES"
    fi
    if [[ -n "${GFXSTREAM_ANGLE_ROOT:-}" ]]; then
        printf 'GFXSTREAM_ANGLE_ROOT=%s\n' "$GFXSTREAM_ANGLE_ROOT"
    fi
    if [[ "$X11_GFXSTREAM_SUBWINDOW" != "auto" ]]; then
        printf 'CROSVM_X11_GFXSTREAM_SUBWINDOW=%s\n' "$X11_GFXSTREAM_SUBWINDOW"
    fi
    if [[ "$MODE" == "gpu" ]]; then
        printf 'X_DISPLAY=%s\n' "$X_DISPLAY"
    fi
    printf 'BOOTCONFIG:\n'
    printf '  %s\n' "${BOOTCONFIG_ARGS[@]}"
    if [[ "${#NET_ARGS[@]}" -gt 0 ]]; then
        printf 'NETWORK:\n'
        printf '  %s\n' "${NET_ARGS[@]}"
    fi
    printf 'BLUETOOTH: %s\n' "$ENABLE_BLUETOOTH"
    printf 'NFC: %s\n' "$ENABLE_NFC"
    printf 'MODEM: %s (vsock port %s)\n' "$ENABLE_MODEM" "$(cf_modem_vsock_port "$CID")"
    printf 'CROSVM:\n'
    printf '  %q' "${CROSVM_CMD[@]}"
    printf '\n'
    exit 0
fi

prepare_writable_images
prepare_hvc_inputs
write_android_dt_fstab
write_initrd
if [[ "$REBUILD_DISK" -eq 1 ]]; then
    prepare_disk
else
    require_file "$DISK_IMAGE" "existing aggregate disk"
fi

if [[ "$RUN_VM" -eq 0 ]]; then
    echo "Prepared:"
    echo "  disk: $DISK_IMAGE"
    echo "  initrd: $INITRD_IMAGE"
    exit 0
fi

start_host_daemons
trap stop_host_daemons EXIT

export LD_LIBRARY_PATH="$(IFS=:; echo "${LD_PATHS[*]}")${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
if [[ "$X11_GFXSTREAM_SUBWINDOW" != "auto" ]]; then
    export CROSVM_X11_GFXSTREAM_SUBWINDOW="$X11_GFXSTREAM_SUBWINDOW"
fi
mkdir -p "$LOG_DIR"
: >"$LOG_DIR/stdout.txt"
: >"$LOG_DIR/stderr.txt"
: >"$LOG_DIR/serial.txt"
: >"$LOG_DIR/hvc.txt"
: >"$LOG_DIR/logcat-hvc2.txt"
echo "Launching Android; logs: $LOG_DIR"
if [[ "$TIMEOUT_SECS" != "0" ]]; then
    TIMEOUT_CMD=(timeout)
    if timeout --help 2>/dev/null | grep -q -- '--foreground'; then
        TIMEOUT_CMD+=(--foreground)
    fi
    "${TIMEOUT_CMD[@]}" "$TIMEOUT_SECS" "${CROSVM_CMD[@]}" </dev/null >"$LOG_DIR/stdout.txt" 2>"$LOG_DIR/stderr.txt" || KEEP_GOING=$?
    if [[ "$KEEP_GOING" -ne 0 && "$KEEP_GOING" -ne 124 ]]; then
        exit "$KEEP_GOING"
    fi
else
    "${CROSVM_CMD[@]}" </dev/null >"$LOG_DIR/stdout.txt" 2>"$LOG_DIR/stderr.txt"
fi
