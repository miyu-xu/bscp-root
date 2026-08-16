#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

VOLUME_ROOT="/Volumes/RTL9210B-CG"
ARTIFACT_DIR=""
DIST_ROOT="$REPO_ROOT/out/dist/macos"
WORK_DIR="$REPO_ROOT/out/runtime/android-macos"
LOG_ROOT="$REPO_ROOT/out/logs/android-macos"
MEM=8192
CPUS=4
LOG_LEVEL=error
CID=100
REFRESH_IMAGES=0
COPY_DISK=1
NO_RUN=0
DRY_RUN=0
PREPARED_ONLY=0
ENABLE_NET=1
ENABLE_INPUT=1
ENABLE_BLUETOOTH=1
ENABLE_NFC=1
ENABLE_MODEM=1
ENABLE_SENSORS=1

usage() {
    cat <<EOF
Usage: $0 [options]

Options:
  --artifact-dir DIR  Packaged vsoc_arm64_only direct-linux directory (raw or sparse disk)
  --dist-root DIR     macOS host dist root (default: $DIST_ROOT)
  --work-dir DIR      Writable runtime directory (default: $WORK_DIR)
  --log-root DIR      Local runtime log root (default: $LOG_ROOT)
  --mem MiB           Guest memory (default: $MEM)
  --cpus N            Guest vCPU count (default: $CPUS)
  --log-level LEVEL   crosvm log level (default: $LOG_LEVEL)
  --cid N             Guest vsock CID (default: $CID)
  --no-net            Disable vmnet.framework shared/NAT networking
                      (networking otherwise asks for macOS administrator access)
  --no-input          Disable Cocoa keyboard and touchscreen input devices
  --no-bluetooth      Do not start RootCanal or bridge hvc5
  --no-nfc            Do not start Casimir or bridge hvc12
  --no-modem          Do not start the host AT modem
  --no-sensors        Do not serve the hvc13 Sensors HAL protocol
  --copy-disk         Copy the aggregate into the work directory before launch (default)
  --direct-disk       Run directly from the artifact aggregate (writes its storage)
  --refresh-images    Refresh copied aggregate and boot inputs from the package
  --no-run            Prepare boot inputs but do not launch crosvm
  --dry-run           Validate and print paths/command without copying or launching
  --help              Show this help

With no --artifact-dir, a complete image set already in --work-dir is reused.
Use --refresh-images to search the external artifact volume and replace it.
External discovery only accepts products/android/vsoc_arm64_only/direct-linux;
the mixed-ABI vsoc_arm64 product contains AArch32 executables that Apple Silicon
cannot run under HVF.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --artifact-dir) ARTIFACT_DIR="$2"; shift 2 ;;
        --dist-root) DIST_ROOT="$2"; shift 2 ;;
        --work-dir) WORK_DIR="$2"; shift 2 ;;
        --log-root) LOG_ROOT="$2"; shift 2 ;;
        --mem) MEM="$2"; shift 2 ;;
        --cpus) CPUS="$2"; shift 2 ;;
        --log-level) LOG_LEVEL="$2"; shift 2 ;;
        --cid) CID="$2"; shift 2 ;;
        --no-net) ENABLE_NET=0; shift ;;
        --no-input) ENABLE_INPUT=0; shift ;;
        --no-bluetooth) ENABLE_BLUETOOTH=0; shift ;;
        --no-nfc) ENABLE_NFC=0; shift ;;
        --no-modem) ENABLE_MODEM=0; shift ;;
        --no-sensors) ENABLE_SENSORS=0; shift ;;
        --copy-disk) COPY_DISK=1; shift ;;
        --direct-disk) COPY_DISK=0; shift ;;
        --refresh-images) REFRESH_IMAGES=1; COPY_DISK=1; shift ;;
        --no-run) NO_RUN=1; shift ;;
        --dry-run) DRY_RUN=1; NO_RUN=1; shift ;;
        --help|-h) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "Error: this runner requires macOS." >&2
    exit 1
fi

find_default_artifact_dir() {
    local candidate=""
    while IFS= read -r candidate; do
        if [[ -f "$candidate/aggregate_android.img" ||
            -f "$candidate/aggregate_android.sparse.img" ]]; then
            printf '%s\n' "$candidate"
            return 0
        fi
    done < <(
        find "$VOLUME_ROOT/bscp-vm-artifacts" -type d \
            -path '*/products/android/vsoc_arm64_only/direct-linux' \
            -print 2>/dev/null |
            while IFS= read -r path; do
                printf '%s\t%s\n' "$(stat -f '%m' "$path" 2>/dev/null || echo 0)" "$path"
            done |
            sort -rn |
            cut -f2-
    )
    return 1
}

prepared_image_set_exists() {
    local name
    for name in aggregate_android.img android_fstab.dt initrd_android.img kernel; do
        if [[ ! -f "$WORK_DIR/$name" ]]; then
            return 1
        fi
    done
    return 0
}

if [[ -z "$ARTIFACT_DIR" ]]; then
    if [[ "$REFRESH_IMAGES" -eq 0 && "$COPY_DISK" -eq 1 ]] &&
        prepared_image_set_exists; then
        ARTIFACT_DIR="$WORK_DIR"
        PREPARED_ONLY=1
    elif ! ARTIFACT_DIR="$(find_default_artifact_dir)"; then
        echo "Error: no packaged vsoc_arm64_only direct-linux image set found under" >&2
        echo "       $VOLUME_ROOT/bscp-vm-artifacts" >&2
        exit 1
    fi
fi
ARTIFACT_DIR="$(cd "$ARTIFACT_DIR" && pwd)"
PRODUCT_NAME="$(basename "$(dirname "$ARTIFACT_DIR")")"
if [[ "$PREPARED_ONLY" -eq 0 &&
    "$PRODUCT_NAME" != "vsoc_arm64_only" ]]; then
    echo "Error: macOS HVF requires vsoc_arm64_only; got $PRODUCT_NAME" >&2
    exit 1
fi

ARTIFACT_DISK="$ARTIFACT_DIR/aggregate_android.img"
ARTIFACT_DISK_IS_SPARSE=0
if [[ -f "$ARTIFACT_DIR/aggregate_android.sparse.img" ]]; then
    ARTIFACT_DISK="$ARTIFACT_DIR/aggregate_android.sparse.img"
    ARTIFACT_DISK_IS_SPARSE=1
fi
if [[ "$COPY_DISK" -eq 0 && "$ARTIFACT_DISK_IS_SPARSE" -eq 1 ]]; then
    echo "Error: --direct-disk requires aggregate_android.img; sparse packages must use --copy-disk" >&2
    exit 1
fi

CROSVM="$DIST_ROOT/bin/crosvm-angle"
for path in \
    "$CROSVM" \
    "$ARTIFACT_DISK" \
    "$ARTIFACT_DIR/android_fstab.dt" \
    "$ARTIFACT_DIR/initrd_android.img" \
    "$ARTIFACT_DIR/kernel"; do
    if [[ ! -f "$path" ]]; then
        echo "Error: missing required file: $path" >&2
        exit 1
    fi
done

if [[ "$COPY_DISK" -eq 1 ]]; then
    DISK="$WORK_DIR/aggregate_android.img"
else
    DISK="$ARTIFACT_DISK"
fi
FSTAB="$WORK_DIR/android_fstab.dt"
INITRD="$WORK_DIR/initrd_android.img"
KERNEL="$WORK_DIR/kernel"
RUN_ID="macos-arm64-$(date +%Y%m%d-%H%M%S)"
LOG_DIR="$LOG_ROOT/$RUN_ID"
HVC_LOG="$LOG_DIR/hvc.txt"
LOGCAT_LOG="$LOG_DIR/logcat-hvc2.txt"
FIFO_DIR="$WORK_DIR/hvc"
SOCKET="/tmp/bscp-android-macos-$CID.sock"
HOST_DEVICE_PID=""

copy_if_needed() {
    local src="$1"
    local dst="$2"
    local src_size dst_size
    src_size="$(stat -f '%z' "$src")"
    dst_size="$(stat -f '%z' "$dst" 2>/dev/null || echo -1)"
    if [[ "$REFRESH_IMAGES" -eq 0 && "$src_size" == "$dst_size" ]]; then
        echo "reuse $dst"
        return
    fi

    local tmp="$dst.new"
    rm -f "$tmp"
    echo "copy $src -> $dst"
    if ! cp -c -p "$src" "$tmp" 2>/dev/null; then
        cp -p "$src" "$tmp"
    fi
    mv -f "$tmp" "$dst"
}

expand_sparse_disk() {
    local src="$1"
    local dst="$2"
    local tmp="$dst.new"
    rm -f "$tmp"
    echo "expand Android sparse disk $src -> $dst"
    if ! python3 "$SCRIPT_DIR/simg2img.py" "$src" "$tmp"; then
        rm -f "$tmp"
        echo "Error: failed to expand Android sparse aggregate: $src" >&2
        exit 1
    fi
    mv -f "$tmp" "$dst"
}

NET_ARGS=()
if [[ "$ENABLE_NET" -eq 1 ]]; then
    NET_ARGS=(
        --net "tap-name=vmnet,mac=00:1a:11:e0:cf:00,pci-address=00:01.1"
        --net "tap-name=vmnet,mac=00:1a:11:e1:cf:00,pci-address=00:01.2"
    )
fi

INPUT_ARGS=()
if [[ "$ENABLE_INPUT" -eq 1 ]]; then
    INPUT_ARGS=(--display-window-keyboard --display-window-mouse)
fi

# Match Cuttlefish's legacy virtio-console layout so guest HALs always get the
# expected device nodes.  Host daemons can write to the relevant input FIFOs:
# hvc5 = Bluetooth/RootCanal, hvc9 = UWB, hvc12 = NFC/Casimir, hvc13 = sensors.
HVC_ARGS=(
    --serial "hardware=legacy-virtio-console,num=1,type=file,path=$HVC_LOG,console=true,stdin=true,pci-address=00:02.0"
    --serial "hardware=legacy-virtio-console,num=2,type=sink,pci-address=00:04.0"
    --serial "hardware=legacy-virtio-console,num=3,type=file,path=$LOGCAT_LOG,pci-address=00:05.0"
    --serial "hardware=legacy-virtio-console,num=4,type=file,path=$LOG_DIR/keymaster-hvc3.bin,input=$FIFO_DIR/keymaster.in,pci-address=00:06.0"
    --serial "hardware=legacy-virtio-console,num=5,type=file,path=$LOG_DIR/gatekeeper-hvc4.bin,input=$FIFO_DIR/gatekeeper.in,pci-address=00:07.0"
    --serial "hardware=legacy-virtio-console,num=6,type=file,path=$LOG_DIR/bluetooth-hvc5.bin,input=$FIFO_DIR/bluetooth.in,pci-address=00:08.0"
    --serial "hardware=legacy-virtio-console,num=7,type=sink,pci-address=00:09.0"
    --serial "hardware=legacy-virtio-console,num=8,type=sink,pci-address=00:0a.0"
    --serial "hardware=legacy-virtio-console,num=9,type=file,path=$LOG_DIR/confui-hvc8.bin,input=$FIFO_DIR/confui.in,pci-address=00:0b.0"
    --serial "hardware=legacy-virtio-console,num=10,type=file,path=$LOG_DIR/uwb-hvc9.bin,input=$FIFO_DIR/uwb.in,pci-address=00:0c.0"
    --serial "hardware=legacy-virtio-console,num=11,type=file,path=$LOG_DIR/oemlock-hvc10.bin,input=$FIFO_DIR/oemlock.in,pci-address=00:0d.0"
    --serial "hardware=legacy-virtio-console,num=12,type=file,path=$LOG_DIR/keymint-hvc11.bin,input=$FIFO_DIR/keymint.in,pci-address=00:0e.0"
    --serial "hardware=legacy-virtio-console,num=13,type=file,path=$LOG_DIR/nfc-hvc12.bin,input=$FIFO_DIR/nfc.in,pci-address=00:0f.0"
    --serial "hardware=legacy-virtio-console,num=14,type=file,path=$LOG_DIR/sensors-hvc13.bin,input=$FIFO_DIR/sensors.in,pci-address=00:10.0"
    --serial "hardware=legacy-virtio-console,num=15,type=file,path=$LOG_DIR/mcu-control-hvc14.bin,input=$FIFO_DIR/mcu-control.in,pci-address=00:11.0"
    --serial "hardware=legacy-virtio-console,num=16,type=file,path=$LOG_DIR/mcu-uart-hvc15.bin,input=$FIFO_DIR/mcu-uart.in,pci-address=00:12.0"
)

CMD=(
    env
    CROSVM_COCOA_DISPLAY=1
    ANGLE_DEFAULT_PLATFORM=metal
    "$CROSVM"
    --log-level "$LOG_LEVEL"
    run
    --disable-sandbox
    --cid "$CID"
    --mem "$MEM"
    --cpus "$CPUS"
    --no-balloon
    --no-usb
    --socket "$SOCKET"
    --gpu "backend=gfxstream,displays=[[mode=windowed[1280,720],dpi=[320,320],refresh-rate=60]],context-types=gfxstream-vulkan:gfxstream-composer,angle=true,gles=false,vulkan=true,wsi=vk,external-blob=false,udmabuf=false"
)
if [[ "$ENABLE_NET" -eq 1 ]]; then
    CMD+=("${NET_ARGS[@]}")
fi
CMD+=(
    --block "path=$DISK,ro=false,lock=true,sparse=false,pci-address=00:03.0"
)
CMD+=("${HVC_ARGS[@]}")
if [[ "$ENABLE_INPUT" -eq 1 ]]; then
    CMD+=("${INPUT_ARGS[@]}")
fi
CMD+=(
    --android-fstab "$FSTAB"
    --initrd "$INITRD"
    --params "console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init"
    "$KERNEL"
)

NEEDS_VMNET_ROOT=0
RUN_CMD=("${CMD[@]}")
if [[ "$ENABLE_NET" -eq 1 && "$EUID" -ne 0 ]]; then
    NEEDS_VMNET_ROOT=1
    RUN_CMD=(sudo "${CMD[@]}")
fi

if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "Artifact: $ARTIFACT_DIR"
    echo "Work:     $WORK_DIR"
    echo "Logs:     $LOG_DIR"
    [[ "$COPY_DISK" -eq 1 ]] && echo "Disk mode: work copy" || echo "Disk mode: direct aggregate"
    printf 'Command:'
    printf ' %q' "${RUN_CMD[@]}"
    printf '\n'
    exit 0
fi

mkdir -p "$WORK_DIR" "$LOG_DIR"
if [[ "$PREPARED_ONLY" -eq 1 ]]; then
    echo "reuse prepared image set in $WORK_DIR"
else
    if [[ "$COPY_DISK" -eq 1 ]]; then
        if [[ "$ARTIFACT_DISK_IS_SPARSE" -eq 1 ]]; then
            expand_sparse_disk "$ARTIFACT_DISK" "$DISK"
        else
            copy_if_needed "$ARTIFACT_DISK" "$DISK"
        fi
    fi
    copy_if_needed "$ARTIFACT_DIR/android_fstab.dt" "$FSTAB"
    copy_if_needed "$ARTIFACT_DIR/initrd_android.img" "$INITRD"
    copy_if_needed "$ARTIFACT_DIR/kernel" "$KERNEL"
fi

BOOTCONFIG_UPDATES=(
    --set "androidboot.boot_devices=10000.pci"
)
if [[ "$ENABLE_MODEM" -eq 1 ]]; then
    MODEM_PORT=$((9600 + CID - 3))
    BOOTCONFIG_UPDATES+=(--set "androidboot.modem_simulator_ports=$MODEM_PORT")
fi
python3 "$SCRIPT_DIR/patch_initrd_bootconfig.py" \
    --initrd "$INITRD" \
    "${BOOTCONFIG_UPDATES[@]}"

if [[ "$NO_RUN" -eq 1 ]]; then
    echo "Prepared macOS Android image set:"
    echo "  disk: $DISK"
    echo "  fstab: $FSTAB"
    echo "  initrd: $INITRD"
    echo "  kernel: $KERNEL"
    exit 0
fi

mkdir -p "$FIFO_DIR"
HVC_FIFO_FDS=()
NEXT_HVC_FD=20
prepare_hvc_fifo() {
    local path="$1"
    local fd="$NEXT_HVC_FD"
    if [[ -e "$path" && ! -p "$path" ]]; then
        echo "Error: HVC input path exists but is not a FIFO: $path" >&2
        exit 1
    fi
    [[ -p "$path" ]] || mkfifo "$path"
    eval "exec ${fd}<>\"$path\""
    HVC_FIFO_FDS+=("$fd")
    NEXT_HVC_FD=$((NEXT_HVC_FD + 1))
}
for hvc_input in \
    keymaster gatekeeper bluetooth confui uwb oemlock keymint nfc sensors \
    mcu-control mcu-uart; do
    prepare_hvc_fifo "$FIFO_DIR/$hvc_input.in"
done
unset hvc_input

: >"$LOG_DIR/bluetooth-hvc5.bin"
: >"$LOG_DIR/nfc-hvc12.bin"
: >"$LOG_DIR/sensors-hvc13.bin"

HOST_DEVICE_ARGS=(
    python3 "$SCRIPT_DIR/cf_host_devices.py"
    --work-dir "$WORK_DIR"
    --log-dir "$LOG_DIR/host-devices"
    --host-bin-dir "$DIST_ROOT/bin"
    --host-bin-dir "$REPO_ROOT/out/dist/host-tools/darwin-arm64/bin"
    --guest-cid "$CID"
    --bt-out "$LOG_DIR/bluetooth-hvc5.bin"
    --bt-in "$FIFO_DIR/bluetooth.in"
    --nfc-out "$LOG_DIR/nfc-hvc12.bin"
    --nfc-in "$FIFO_DIR/nfc.in"
    --sensors-out "$LOG_DIR/sensors-hvc13.bin"
    --sensors-in "$FIFO_DIR/sensors.in"
)
[[ "$ENABLE_BLUETOOTH" -eq 1 ]] || HOST_DEVICE_ARGS+=(--no-bluetooth)
[[ "$ENABLE_NFC" -eq 1 ]] || HOST_DEVICE_ARGS+=(--no-nfc)
[[ "$ENABLE_MODEM" -eq 1 ]] || HOST_DEVICE_ARGS+=(--no-modem)
[[ "$ENABLE_SENSORS" -eq 1 ]] || HOST_DEVICE_ARGS+=(--no-sensors)

"${HOST_DEVICE_ARGS[@]}" >"$LOG_DIR/host-devices-supervisor.txt" 2>&1 &
HOST_DEVICE_PID=$!
sleep 0.5
if ! kill -0 "$HOST_DEVICE_PID" 2>/dev/null; then
    echo "Error: host-device supervisor failed; see $LOG_DIR/host-devices-supervisor.txt" >&2
    exit 1
fi
trap '[[ -n "${HOST_DEVICE_PID:-}" ]] && kill "$HOST_DEVICE_PID" 2>/dev/null || true' EXIT

if [[ "$NEEDS_VMNET_ROOT" -eq 1 ]]; then
    echo "vmnet.framework shared networking requires macOS administrator access."
    sudo -v
fi

MIN_OPEN_FILES=4096
CURRENT_OPEN_FILES="$(ulimit -Sn)"
if [[ "$CURRENT_OPEN_FILES" =~ ^[0-9]+$ ]] && (( CURRENT_OPEN_FILES < MIN_OPEN_FILES )); then
    if ! ulimit -Sn "$MIN_OPEN_FILES"; then
        echo "Warning: could not raise open-file limit to $MIN_OPEN_FILES; current limit is $CURRENT_OPEN_FILES" >&2
    fi
fi

cleanup_socket() {
    if [[ -n "$HOST_DEVICE_PID" ]] && kill -0 "$HOST_DEVICE_PID" 2>/dev/null; then
        kill "$HOST_DEVICE_PID" 2>/dev/null || true
        wait "$HOST_DEVICE_PID" 2>/dev/null || true
    fi
    rm -f "$SOCKET" 2>/dev/null || sudo -n rm -f "$SOCKET" 2>/dev/null || true
    rm -f "/tmp/binder_rpc_vsock_${CID}_$((9600 + CID - 3)).sock" 2>/dev/null || true
}
rm -f "$SOCKET" 2>/dev/null || sudo -n rm -f "$SOCKET" 2>/dev/null || true
trap cleanup_socket EXIT

if [[ "$NEEDS_VMNET_ROOT" -eq 1 ]]; then
    (
        for _ in {1..100}; do
            if [[ -S "$SOCKET" ]]; then
                sudo -n chown "$(id -u):$(id -g)" "$SOCKET" &&
                    sudo -n chmod 600 "$SOCKET"
                exit
            fi
            sleep 0.1
        done
    ) &
fi

: >"$LOG_DIR/stdout.txt"
: >"$LOG_DIR/stderr.txt"
: >"$HVC_LOG"
: >"$LOGCAT_LOG"
echo "Launching Android; logs: $LOG_DIR"
printf 'Command:'
printf ' %q' "${RUN_CMD[@]}"
printf '\n'
"${RUN_CMD[@]}" > >(tee "$LOG_DIR/stdout.txt") 2> >(tee "$LOG_DIR/stderr.txt" >&2)
