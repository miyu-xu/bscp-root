#!/usr/bin/env bash
set -euo pipefail

shopt -s nullglob

COMMAND="run-microdroid"
REPO_ROOT=""
DIST_ROOT=""
APEX_TREE_ROOT=""
WORK_DIR=""
LOG_DIR=""
SERVICE_ROOT=""
TEMP_ROOT=""
APK=""
IDSIG=""
INSTANCE=""
PAYLOAD_BINARY_NAME="MicrodroidEmptyPayloadJniLib.so"
CONFIG=""
FEATURE="dice_changes"
PARTITION_PATH=""
PARTITION_SIZE=0
PARTITION_TYPE="raw"
OUTPUT_PATH=""
CID=0
NAME=""
CONSOLE=""
CONSOLE_IN=""
GUEST_LOG=""
TRACE_FILE=""
VMCLIENT_TRACE_FILE=""
DEBUG_POLICY_JSON=""
KEEP_TEMP=0
PERSIST_VIRTMGR=0
DRY_RUN=0
TIMEOUT_SECS=0
NO_ROOT=0
NO_SHELL=0

VM_ARGS=()
RUN_VM=1

usage() {
    cat <<'EOF'
Usage: vm_macos.sh [options] [-- extra vm args]

Commands:
  validate-prereqs | run-microdroid | run-app | run | info | list | console
  check-feature-enabled | create-partition | create-idsig | service-status | stop-service
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -Command|--command) COMMAND="$2"; shift 2 ;;
        -RepoRoot|--repo-root) REPO_ROOT="$2"; shift 2 ;;
        -DistRoot|--dist-root) DIST_ROOT="$2"; shift 2 ;;
        -ApexTreeRoot|--apex-tree-root) APEX_TREE_ROOT="$2"; shift 2 ;;
        -WorkDir|--work-dir) WORK_DIR="$2"; shift 2 ;;
        -LogDir|--log-dir) LOG_DIR="$2"; shift 2 ;;
        -ServiceRoot|--service-root) SERVICE_ROOT="$2"; shift 2 ;;
        -TempRoot|--temp-root) TEMP_ROOT="$2"; shift 2 ;;
        -Apk|--apk) APK="$2"; shift 2 ;;
        -Idsig|--idsig) IDSIG="$2"; shift 2 ;;
        -Instance|--instance) INSTANCE="$2"; shift 2 ;;
        -PayloadBinaryName|--payload-binary-name) PAYLOAD_BINARY_NAME="$2"; shift 2 ;;
        -Config|--config) CONFIG="$2"; shift 2 ;;
        -Feature|--feature) FEATURE="$2"; shift 2 ;;
        -PartitionPath|--partition-path) PARTITION_PATH="$2"; shift 2 ;;
        -PartitionSize|--partition-size) PARTITION_SIZE="$2"; shift 2 ;;
        -PartitionType|--partition-type) PARTITION_TYPE="$2"; shift 2 ;;
        -OutputPath|--output-path) OUTPUT_PATH="$2"; shift 2 ;;
        -Cid|--cid) CID="$2"; shift 2 ;;
        -Name|--name) NAME="$2"; shift 2 ;;
        -Console|--console) CONSOLE="$2"; shift 2 ;;
        -ConsoleIn|--console-in) CONSOLE_IN="$2"; shift 2 ;;
        -GuestLog|--guest-log) GUEST_LOG="$2"; shift 2 ;;
        -TraceFile|--trace-file) TRACE_FILE="$2"; shift 2 ;;
        -VmclientTraceFile|--vmclient-trace-file) VMCLIENT_TRACE_FILE="$2"; shift 2 ;;
        -DebugPolicyJson|--debug-policy-json) DEBUG_POLICY_JSON="$2"; shift 2 ;;
        -KeepTemp|--keep-temp) KEEP_TEMP=1; shift ;;
        -PersistVirtmgr|--persist-virtmgr) PERSIST_VIRTMGR=1; shift ;;
        -DryRun|--dry-run) DRY_RUN=1; shift ;;
        -TimeoutSecs|--timeout-secs) TIMEOUT_SECS="$2"; shift 2 ;;
        -NoRoot|--no-root) NO_ROOT=1; shift ;;
        -NoShell|--no-shell) NO_SHELL=1; shift ;;
        --) shift; VM_ARGS+=("$@"); break ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown argument: $1" >&2; usage; exit 2 ;;
    esac
done

require_path() {
    local path="$1"
    local label="$2"
    [[ -e "$path" ]] || { echo "$label not found: $path" >&2; exit 1; }
}

crosvm_has_hypervisor_entitlement() {
    codesign -d --entitlements :- "$CROSVM_EXE" 2>/dev/null | grep -q '<key>com.apple.security.hypervisor</key>'
}

guest_kernel_path() {
    printf '%s\n' "$APEX_ROOT/com.android.virt/etc/fs/microdroid_kernel"
}

guest_kernel_file_info() {
    file -b "$(guest_kernel_path)" 2>/dev/null || true
}

guest_kernel_is_arm64() {
    guest_kernel_file_info | grep -Eiq 'ARM aarch64|ARM64|arm64'
}

require_arm64_guest_kernel() {
    local kernel info
    kernel="$(guest_kernel_path)"
    require_path "$kernel" "Microdroid guest kernel"
    info="$(guest_kernel_file_info)"
    if ! guest_kernel_is_arm64; then
        echo "macOS HVF requires an arm64 Microdroid guest kernel, but found: ${info:-unknown}" >&2
        echo "Provide an arm64 com.android.virt apex tree under $APEX_TREE_ROOT before running macOS guest commands." >&2
        exit 1
    fi
}

mkdir_parent() {
    mkdir -p "$(dirname "$1")"
}

find_default_apk() {
    find "$APEX_ROOT/com.android.virt/app" -type f -name 'EmptyPayloadApp*.apk' -print | head -n 1
}

service_state_path() {
    printf '%s\n' "$SERVICE_ROOT/virtmgr-service.state"
}

service_trace_path() {
    printf '%s\n' "$SERVICE_ROOT/virtmgr-trace.log"
}

read_service_value() {
    local key="$1"
    local state_file
    state_file="$(service_state_path)"
    [[ -f "$state_file" ]] || return 1
    awk -F= -v wanted="$key" '$1==wanted { print $2 }' "$state_file" | tail -n 1
}

stop_service() {
    local state_file pid socket_path
    state_file="$(service_state_path)"
    pid="$(read_service_value pid || true)"
    socket_path="$(read_service_value socket_path || true)"
    if [[ -n "${pid:-}" ]] && kill -0 "$pid" 2>/dev/null; then
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
        echo "Stopped virtmgr service PID $pid."
    else
        echo "No running persistent virtmgr service is registered."
    fi
    rm -f "$state_file"
    [[ -n "${socket_path:-}" ]] && rm -f "$socket_path"
    return 0
}

show_service_status() {
    local pid socket_path running=0
    pid="$(read_service_value pid || true)"
    socket_path="$(read_service_value socket_path || true)"
    if [[ -n "${pid:-}" ]] && kill -0 "$pid" 2>/dev/null; then
        running=1
    fi
    cat <<EOF
ServiceRoot: $SERVICE_ROOT
StateFile: $(service_state_path)
TraceFile: $(service_trace_path)
Registered: $([[ -f "$(service_state_path)" ]] && echo true || echo false)
Pid: ${pid:-}
SocketPath: ${socket_path:-}
Running: $running
EOF
}

append_common_vm_args() {
    if [[ -n "$NAME" ]]; then
        COMMAND_ARGS+=(--name "$NAME")
    fi
}

run_command_with_timeout() {
    local timeout_secs="$1"
    shift

    set +e
    "$@" >"$RUN_LOG_FILE" 2>&1 &
    local cmd_pid=$!
    local watchdog_pid=0
    (
        sleep "$timeout_secs"
        kill -TERM "$cmd_pid" 2>/dev/null || exit 0
        sleep 2
        kill -KILL "$cmd_pid" 2>/dev/null || true
    ) &
    watchdog_pid=$!

    wait "$cmd_pid"
    local rc=$?
    kill "$watchdog_pid" 2>/dev/null || true
    wait "$watchdog_pid" 2>/dev/null || true
    set -e

    if [[ $rc -eq 143 || $rc -eq 137 ]]; then
        return 124
    fi
    return "$rc"
}

run_logged() {
    local rc
    local vm_args=("$VM_EXE")
    if [[ ${#COMMAND_ARGS[@]} -gt 0 ]]; then
        vm_args+=("${COMMAND_ARGS[@]}")
    fi
    if [[ "$DRY_RUN" -eq 1 ]]; then
        printf 'Dry run: %q ' "${vm_args[@]}"
        printf '\n'
        return 0
    fi

    if [[ "$TIMEOUT_SECS" -gt 0 ]]; then
        rc=0
        run_command_with_timeout "$TIMEOUT_SECS" "${vm_args[@]}" || rc=$?
    else
        set +e
        "${vm_args[@]}" >"$RUN_LOG_FILE" 2>&1
        rc=$?
        set -e
    fi

    cat "$RUN_LOG_FILE"
    return "$rc"
}

if [[ -z "$REPO_ROOT" ]]; then
    REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi
if [[ -z "$DIST_ROOT" ]]; then
    DIST_ROOT="$REPO_ROOT/out/dist"
fi
if [[ -z "$APEX_TREE_ROOT" ]]; then
    APEX_TREE_ROOT="${MACOS_AVF_APEX_TREE_ROOT:-$DIST_ROOT/apex_dir}"
fi
if [[ -z "$SERVICE_ROOT" ]]; then
    SERVICE_ROOT="/tmp/bscp-macos-virtmgr-service"
fi

BIN_DIR="$DIST_ROOT/macos/bin"
LIB_DIR="$DIST_ROOT/macos/lib"
VM_EXE="$BIN_DIR/vm"
VIRTMGR_EXE="$BIN_DIR/virtmgr"
CROSVM_EXE="$BIN_DIR/crosvm"
APEX_ROOT="$APEX_TREE_ROOT/apex"
SYSTEM_ROOT="$APEX_TREE_ROOT/system"
SYSTEM_EXT_ROOT="$APEX_TREE_ROOT/system_ext"

require_path "$REPO_ROOT" "Repository root"
require_path "$DIST_ROOT" "dist root"
require_path "$BIN_DIR" "macOS dist bin directory"
require_path "$LIB_DIR" "macOS dist lib directory"
require_path "$VM_EXE" "vm"
require_path "$VIRTMGR_EXE" "virtmgr"
require_path "$CROSVM_EXE" "crosvm"
require_path "$APEX_TREE_ROOT" "macOS apex tree root"
require_path "$APEX_ROOT" "mounted apex root"
require_path "$APEX_ROOT/com.android.virt" "mounted com.android.virt apex"

export DYLD_LIBRARY_PATH="$LIB_DIR${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"

timestamp="$(date +%Y%m%d-%H%M%S)"
if [[ -z "$LOG_DIR" ]]; then
    LOG_DIR="$DIST_ROOT/logs/macos-${COMMAND}-${timestamp}"
fi
if [[ -z "$WORK_DIR" ]]; then
    WORK_DIR="$LOG_DIR/work"
fi
if [[ -z "$TEMP_ROOT" ]]; then
    TEMP_ROOT="$LOG_DIR/temp"
fi
mkdir -p "$LOG_DIR" "$WORK_DIR" "$TEMP_ROOT"

if [[ -z "$CONSOLE" ]]; then
    CONSOLE="$LOG_DIR/vm-console.txt"
fi
if [[ -z "$CONSOLE_IN" ]]; then
    CONSOLE_IN="$LOG_DIR/vm-console-in.txt"
fi
if [[ -z "$GUEST_LOG" ]]; then
    GUEST_LOG="$LOG_DIR/guest-log.txt"
fi
if [[ -z "$TRACE_FILE" ]]; then
    if [[ "$PERSIST_VIRTMGR" -eq 1 ]]; then
        TRACE_FILE="$(service_trace_path)"
    else
        TRACE_FILE="$LOG_DIR/virtmgr-trace.log"
    fi
fi
if [[ -z "$VMCLIENT_TRACE_FILE" ]]; then
    VMCLIENT_TRACE_FILE="$LOG_DIR/vmclient-trace.log"
fi
RUN_LOG_FILE="$LOG_DIR/vm-${COMMAND}.log"

export PATH="$BIN_DIR:$PATH"
export DYLD_LIBRARY_PATH="$LIB_DIR:${DYLD_LIBRARY_PATH:-}"
export VIRTMGR_PATH="$VIRTMGR_EXE"
export VIRTMGR_CROSVM_PATH="$CROSVM_EXE"
export VIRTMGR_APEX_ROOT="$APEX_ROOT"
export VIRTMGR_SYSTEM_ROOT="$SYSTEM_ROOT"
export VIRTMGR_SYSTEM_EXT_ROOT="$SYSTEM_EXT_ROOT"
export ANDROID_PROP_RO_BUILD_VERSION_SDK=35
export VIRTMGR_TRACE_FILE="$TRACE_FILE"
export VMCLIENT_TRACE_FILE="$VMCLIENT_TRACE_FILE"
export TMPDIR="$TEMP_ROOT"

if [[ "$PERSIST_VIRTMGR" -eq 1 ]]; then
    mkdir -p "$SERVICE_ROOT"
    export VIRTMGR_SERVICE_DIR="$SERVICE_ROOT"
else
    unset VIRTMGR_SERVICE_DIR || true
fi
if [[ "$KEEP_TEMP" -eq 1 ]]; then
    export VIRTMGR_KEEP_TEMP=1
else
    unset VIRTMGR_KEEP_TEMP || true
fi
if [[ -n "$DEBUG_POLICY_JSON" ]]; then
    require_path "$DEBUG_POLICY_JSON" "Debug policy JSON"
    export VIRTMGR_DEBUG_POLICY_JSON="$DEBUG_POLICY_JSON"
fi

COMMAND_ARGS=()

case "$COMMAND" in
    validate-prereqs)
        host_arch="$(uname -m)"
        hv_support="$(sysctl -n kern.hv_support 2>/dev/null || true)"
        hypervisor_entitlement=0
        guest_kernel_arm64=0
        guest_kernel_info="$(guest_kernel_file_info)"
        if crosvm_has_hypervisor_entitlement; then
            hypervisor_entitlement=1
        fi
        if guest_kernel_is_arm64; then
            guest_kernel_arm64=1
        fi
        {
            echo "Host OS    : $(uname -s)"
            echo "Host arch  : $host_arch"
            echo "HVF support: ${hv_support:-unknown}"
            echo "HVF entitlement: $hypervisor_entitlement"
            echo "Guest kernel arm64: $guest_kernel_arm64"
            echo "Guest kernel info : ${guest_kernel_info:-unknown}"
            echo "crosvm     : $CROSVM_EXE"
        } | tee "$RUN_LOG_FILE"
        RUN_VM=0
        [[ "$host_arch" == "arm64" && "$hv_support" == "1" && "$hypervisor_entitlement" == "1" && "$guest_kernel_arm64" == "1" ]]
        ;;
    run-microdroid)
        require_arm64_guest_kernel
        append_common_vm_args
        COMMAND_ARGS+=(run-microdroid --work-dir "$WORK_DIR" --log "$GUEST_LOG")
        ;;
    run-app)
        require_arm64_guest_kernel
        append_common_vm_args
        if [[ -z "$APK" ]]; then
            APK="$(find_default_apk)"
        fi
        [[ -n "$IDSIG" ]] || IDSIG="$WORK_DIR/app.idsig"
        [[ -n "$INSTANCE" ]] || INSTANCE="$WORK_DIR/instance.img"
        require_path "$APK" "VM payload APK"
        mkdir_parent "$IDSIG"
        mkdir_parent "$INSTANCE"
        COMMAND_ARGS+=(run-app "$APK" "$IDSIG" "$INSTANCE" --payload-binary-name "$PAYLOAD_BINARY_NAME" --log "$GUEST_LOG")
        ;;
    run)
        require_arm64_guest_kernel
        append_common_vm_args
        [[ -n "$CONFIG" ]] || CONFIG="$REPO_ROOT/scripts/microdroid_macos_raw.json"
        require_path "$CONFIG" "VM config JSON"
        COMMAND_ARGS+=(run "$CONFIG" --log "$GUEST_LOG")
        ;;
    info)
        COMMAND_ARGS+=(info)
        ;;
    list)
        COMMAND_ARGS+=(list)
        ;;
    console)
        COMMAND_ARGS+=(console)
        [[ "$CID" -gt 0 ]] && COMMAND_ARGS+=("$CID")
        ;;
    check-feature-enabled)
        COMMAND_ARGS+=(check-feature-enabled "$FEATURE")
        ;;
    create-partition)
        [[ -n "$PARTITION_PATH" ]] || PARTITION_PATH="$WORK_DIR/writable.img"
        [[ "$PARTITION_SIZE" -gt 0 ]] || PARTITION_SIZE=1048576
        mkdir_parent "$PARTITION_PATH"
        COMMAND_ARGS+=(create-partition "$PARTITION_PATH" "$PARTITION_SIZE")
        [[ "$PARTITION_TYPE" != "raw" ]] && COMMAND_ARGS+=(--type "$PARTITION_TYPE")
        ;;
    create-idsig)
        if [[ -z "$APK" ]]; then
            APK="$(find_default_apk)"
        fi
        [[ -n "$OUTPUT_PATH" ]] || OUTPUT_PATH="$WORK_DIR/app.idsig"
        require_path "$APK" "VM payload APK"
        mkdir_parent "$OUTPUT_PATH"
        COMMAND_ARGS+=(create-idsig "$APK" "$OUTPUT_PATH")
        ;;
    service-status)
        show_service_status | tee "$RUN_LOG_FILE"
        exit 0
        ;;
    stop-service)
        stop_service | tee "$RUN_LOG_FILE"
        exit 0
        ;;
    *)
        echo "Unsupported command: $COMMAND" >&2
        exit 2
        ;;
esac

if [[ ${#VM_ARGS[@]} -gt 0 ]]; then
    COMMAND_ARGS+=("${VM_ARGS[@]}")
fi

{
    resolved_vm_args=("$VM_EXE")
    if [[ ${#COMMAND_ARGS[@]} -gt 0 ]]; then
        resolved_vm_args+=("${COMMAND_ARGS[@]}")
    fi
    echo "Command           : $COMMAND"
    echo "RepoRoot          : $REPO_ROOT"
    echo "DistRoot          : $DIST_ROOT"
    echo "ApexTreeRoot      : $APEX_TREE_ROOT"
    echo "WorkDir           : $WORK_DIR"
    echo "LogDir            : $LOG_DIR"
    echo "ServiceRoot       : $SERVICE_ROOT"
    echo "PersistVirtmgr    : $PERSIST_VIRTMGR"
    echo "TempRoot          : $TEMP_ROOT"
    echo "vm                : $VM_EXE"
    echo "virtmgr           : $VIRTMGR_EXE"
    echo "crosvm            : $CROSVM_EXE"
    echo "TraceFile         : $TRACE_FILE"
    echo "VmclientTraceFile : $VMCLIENT_TRACE_FILE"
    echo "RunLog            : $RUN_LOG_FILE"
    [[ "$COMMAND" =~ ^(run-microdroid|run-app|run)$ ]] && {
        echo "GuestLog          : $GUEST_LOG"
        echo "ConsoleTTY        : vm console (pty-backed when persistent virtmgr is enabled)"
    }
    echo -n "Resolved vm args  : "
    printf '%q ' "${resolved_vm_args[@]}"
    echo
} | tee "$RUN_LOG_FILE"

if [[ "$RUN_VM" -eq 1 ]]; then
    run_logged
fi
