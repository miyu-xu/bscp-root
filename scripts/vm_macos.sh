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
  validate-prereqs | diagnose | cleanup | run-microdroid | run-app | run | info | list | console
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
        hv_support="$(sysctl -n kern.hv_support 2>/dev/null || echo "0")"
        hypervisor_entitlement=0
        guest_kernel_arm64=0
        com_android_adbd_found=0
        guest_kernel_info="$(guest_kernel_file_info)"
        if crosvm_has_hypervisor_entitlement; then
            hypervisor_entitlement=1
        fi
        if guest_kernel_is_arm64; then
            guest_kernel_arm64=1
        fi
        if [[ -f "$APEX_ROOT/com.android.virt/app/com.android.adbd/com.android.adbd.apk" || \
              -f "$APEX_ROOT/com.android.virt/app/com.android.adbd/apex-0000.adbd" ]]; then
            com_android_adbd_found=1
        fi
        ALL_PASS=1
        {
            echo "Host OS    : $(uname -s)"
            echo "Host arch  : $host_arch"
            echo "HVF support: ${hv_support}"
            echo "HVF entitlement: $hypervisor_entitlement"
            echo "Guest kernel arm64: $guest_kernel_arm64"
            echo "Guest kernel info : ${guest_kernel_info:-unknown}"
            echo "com.android.adbd  : $com_android_adbd_found"
            echo "crosvm     : $CROSVM_EXE"
            echo "virtmgr    : $VIRTMGR_EXE"
            echo "apex tree  : $APEX_TREE_ROOT"
        } > "$RUN_LOG_FILE"

        # Check 1: Host architecture
        if [[ "$host_arch" == "arm64" ]]; then
            echo "  [OK]   Host architecture: arm64" | tee -a "$RUN_LOG_FILE"
        else
            echo "  [FAIL] Host architecture: $host_arch (requires arm64 for HVF)" | tee -a "$RUN_LOG_FILE"
            echo "         Intel Macs are not supported (see doc/HVF_X86_64_FEASIBILITY.md)" | tee -a "$RUN_LOG_FILE"
            ALL_PASS=0
        fi

        # Check 2: HVF support
        if [[ "$hv_support" == "1" ]]; then
            echo "  [OK]   HVF (kern.hv_support): enabled" | tee -a "$RUN_LOG_FILE"
        else
            echo "  [FAIL] HVF (kern.hv_support): not enabled" | tee -a "$RUN_LOG_FILE"
            echo "         Fix: Ensure you are on Apple Silicon and run:" | tee -a "$RUN_LOG_FILE"
            echo "           sudo nvram boot-args=\"-arm64e_preview_abi\" && reboot" | tee -a "$RUN_LOG_FILE"
            ALL_PASS=0
        fi

        # Check 3: crosvm Hypervisor entitlement
        if [[ "$hypervisor_entitlement" -eq 1 ]]; then
            echo "  [OK]   crosvm HVF entitlement: present" | tee -a "$RUN_LOG_FILE"
        else
            echo "  [FAIL] crosvm HVF entitlement: MISSING" | tee -a "$RUN_LOG_FILE"
            echo "         Fix: Rebuild with build_all.sh or manually sign:" | tee -a "$RUN_LOG_FILE"
            echo "           codesign --force --sign - --entitlements $REPO_ROOT/scripts/macos_crosvm.entitlements --timestamp=none $CROSVM_EXE" | tee -a "$RUN_LOG_FILE"
            ALL_PASS=0
        fi

        # Check 4: Guest kernel architecture
        if [[ "$guest_kernel_arm64" -eq 1 ]]; then
            echo "  [OK]   Guest kernel: arm64" | tee -a "$RUN_LOG_FILE"
        else
            echo "  [WARN] Guest kernel: NOT arm64" | tee -a "$RUN_LOG_FILE"
            echo "         Info: ${guest_kernel_info:-unknown}" | tee -a "$RUN_LOG_FILE"
            echo "         Fix: Run scripts/fetch_arm64_guest_artifacts.sh --apex-tree $APEX_TREE_ROOT" | tee -a "$RUN_LOG_FILE"
            echo "         Or provide arm64 APEX tree via MACOS_AVF_APEX_TREE_SOURCE" | tee -a "$RUN_LOG_FILE"
            ALL_PASS=0
        fi

        # Check 5: com.android.adbd availability
        if [[ "$com_android_adbd_found" -eq 1 ]]; then
            echo "  [OK]   com.android.adbd: found in APEX tree" | tee -a "$RUN_LOG_FILE"
        else
            echo "  [WARN] com.android.adbd: NOT found" | tee -a "$RUN_LOG_FILE"
            echo "         ADB bridge will fail. Provide a complete com.android.virt APEX tree." | tee -a "$RUN_LOG_FILE"
        fi

        echo "" | tee -a "$RUN_LOG_FILE"
        if [[ "$ALL_PASS" -eq 1 ]]; then
            echo "Result: PASS — all prerequisites satisfied" | tee -a "$RUN_LOG_FILE"
        else
            echo "Result: FAIL — one or more prerequisites not met (see above for fixes)" | tee -a "$RUN_LOG_FILE"
        fi

        # Write structured JSON diagnostic
        cat > "$LOG_DIR/diagnostic.json" <<- JSONEOF
		{
		  "host_os": "$(uname -s)",
		  "host_arch": "$host_arch",
		  "hv_support": "$hv_support",
		  "hypervisor_entitlement": $hypervisor_entitlement,
		  "guest_kernel_arm64": $guest_kernel_arm64,
		  "com.android.adbd": $com_android_adbd_found,
		  "crosvm": "$CROSVM_EXE",
		  "virtmgr": "$VIRTMGR_EXE",
		  "apex_tree": "$APEX_TREE_ROOT",
		  "result": "$([[ $ALL_PASS -eq 1 ]] && echo PASS || echo FAIL)"
		}
		JSONEOF
        RUN_VM=0
        [[ "$ALL_PASS" -eq 1 ]]
        ;;
    diagnose)
        {
            echo "=== macOS VM Diagnostic ==="
            echo ""

            echo "[1/6] Checking build artifacts..."
            missing_artifacts=0
            for f in "$VM_EXE" "$VIRTMGR_EXE" "$CROSVM_EXE"; do
                if [[ -f "$f" ]]; then
                    echo "  OK: $(basename "$f")"
                else
                    echo "  MISSING: $f"
                    missing_artifacts=1
                fi
            done
            if [[ $missing_artifacts -eq 1 ]]; then
                echo "  Fix: Run build_all.sh from repo root."
            fi

            echo ""
            echo "[2/6] Checking APEX tree..."
            if [[ -d "$APEX_ROOT/com.android.virt" ]]; then
                echo "  OK: com.android.virt found at $APEX_ROOT/com.android.virt"
            else
                echo "  MISSING: com.android.virt APEX tree"
                echo "  Fix: Set MACOS_AVF_APEX_TREE_SOURCE and run build_all.sh"
            fi

            echo ""
            echo "[3/6] Checking HVF availability..."
            hv="$(sysctl -n kern.hv_support 2>/dev/null || echo "0")"
            if [[ "$hv" == "1" ]]; then
                echo "  OK: HVF is enabled (kern.hv_support = 1)"
            else
                echo "  FAIL: HVF is not enabled (kern.hv_support = $hv)"
                echo "  Fix: Run on Apple Silicon or check Virtualization.framework availability."
            fi

            echo ""
            echo "[4/6] Checking crosvm Hypervisor entitlement..."
            if crosvm_has_hypervisor_entitlement; then
                echo "  OK: crosvm has HVF entitlement"
            else
                echo "  FAIL: crosvm is missing HVF entitlement"
                echo "  Fix: codesign --force --sign - --entitlements $REPO_ROOT/scripts/macos_crosvm.entitlements --timestamp=none $CROSVM_EXE"
            fi

            echo ""
            echo "[5/6] Checking guest kernel..."
            kernel="$(guest_kernel_path)"
            if [[ -f "$kernel" ]]; then
                kinfo="$(guest_kernel_file_info)"
                if guest_kernel_is_arm64; then
                    echo "  OK: Guest kernel is arm64"
                    echo "  Info: $kinfo"
                else
                    echo "  WARN: Guest kernel is not arm64"
                    echo "  Info: $kinfo"
                    echo "  Fix: scripts/fetch_arm64_guest_artifacts.sh"
                fi
            else
                echo "  MISSING: No guest kernel at $kernel"
                echo "  Fix: Provide a complete APEX tree with arm64 microdroid_kernel."
            fi

            echo ""
            echo "[6/6] Checking virtmgr service state..."
            if [[ -f "$(service_state_path)" ]]; then
                echo "  Service state file exists: $(service_state_path)"
                show_service_status | sed 's/^/  /'
            else
                echo "  No persistent virtmgr service is registered."
            fi

            echo ""
            echo "=== Diagnostic complete ==="
            echo "Logs saved to: $LOG_DIR"
        } | tee "$RUN_LOG_FILE"
        RUN_VM=0
        ;;
    cleanup)
        echo "=== Cleaning up macOS virtmgr/crosvm resources ===" | tee "$RUN_LOG_FILE"
        cleaned=0

        state_file="$(service_state_path)"
        if [[ -f "$state_file" ]]; then
            pid="$(read_service_value pid || true)"
            socket="$(read_service_value socket_path || true)"
            if [[ -n "${pid:-}" ]] && kill -0 "$pid" 2>/dev/null; then
                echo "  Stopping virtmgr service (PID $pid)..." | tee -a "$RUN_LOG_FILE"
                kill "$pid" 2>/dev/null || true
                wait "$pid" 2>/dev/null || true
                echo "  Stopped." | tee -a "$RUN_LOG_FILE"
                cleaned=1
            fi
            rm -f "$state_file"
            [[ -n "${socket:-}" ]] && rm -f "$socket" 2>/dev/null || true
        fi

        for proc_name in virtmgr crosvm; do
            while IFS= read -r line; do
                proc_pid="$(echo "$line" | awk '{print $2}')"  # ps aux: PID is column 2
                exe="$(echo "$line" | awk '{for(i=11;i<=NF;i++) printf "%s ", $i; print ""}')"
                if [[ "$exe" == *"$BIN_DIR/$proc_name"* ]]; then
                    echo "  Killing $proc_name (PID $proc_pid)" | tee -a "$RUN_LOG_FILE"
                    kill "$proc_pid" 2>/dev/null || true
                    cleaned=1
                fi
            done < <(ps aux | grep -E "[v]irtmgr|[c]rosvm" | head -20 || true)
        done

        echo "  Cleaning temp root: $TEMP_ROOT" | tee -a "$RUN_LOG_FILE"
        rm -rf "$TEMP_ROOT"/* 2>/dev/null || true
        rm -f /tmp/binder_rpc_vsock_*.sock 2>/dev/null || true
        rm -f /tmp/binder_rpc_test_* 2>/dev/null || true

        if [[ $cleaned -eq 1 ]]; then
            echo "  Cleanup complete." | tee -a "$RUN_LOG_FILE"
        else
            echo "  Nothing to clean up." | tee -a "$RUN_LOG_FILE"
        fi
        RUN_VM=0
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

# Generate run-summary.txt
{
    echo "=== Run Summary ==="
    echo "Command     : $COMMAND"
    echo "Date        : $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    echo "RepoRoot    : $REPO_ROOT"
    echo "DistRoot    : $DIST_ROOT"
    echo "ApexTree    : $APEX_TREE_ROOT"
    echo "WorkDir     : $WORK_DIR"
    echo "LogDir      : $LOG_DIR"
    echo "PersistVirtmgr : $PERSIST_VIRTMGR"
    echo "vm          : $VM_EXE"
    echo "virtmgr     : $VIRTMGR_EXE"
    echo "crosvm      : $CROSVM_EXE"
    if [[ ${#COMMAND_ARGS[@]} -gt 0 ]]; then
        echo "Args        : ${COMMAND_ARGS[*]}"
    fi
    if [[ -n "${GUEST_LOG:-}" ]]; then
        echo "GuestLog    : $GUEST_LOG"
    fi
    echo "TraceFile   : $TRACE_FILE"
} > "$LOG_DIR/run-summary.txt"

if [[ "$RUN_VM" -eq 1 ]]; then
    run_logged
    rc=$?
    echo "ExitCode    : $rc" >> "$LOG_DIR/run-summary.txt"
    exit $rc
fi
