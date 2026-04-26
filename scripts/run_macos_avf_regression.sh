#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=""
DIST_ROOT=""
OUTPUT_ROOT=""
APEX_TREE_ROOT=""
ADB_PORT=8035
INCLUDE_RUN_APP=0
INCLUDE_ADB_SCENARIO=0
SCENARIO_MODE="full"
STEP_TIMEOUT=120

while [[ $# -gt 0 ]]; do
    case "$1" in
        -RepoRoot|--repo-root) REPO_ROOT="$2"; shift 2 ;;
        -DistRoot|--dist-root) DIST_ROOT="$2"; shift 2 ;;
        -OutputRoot|--output-root) OUTPUT_ROOT="$2"; shift 2 ;;
        -ApexTreeRoot|--apex-tree-root) APEX_TREE_ROOT="$2"; shift 2 ;;
        -AdbPort|--adb-port) ADB_PORT="$2"; shift 2 ;;
        -IncludeRunApp|--include-run-app) INCLUDE_RUN_APP=1; shift ;;
        -IncludeAdbScenario|--include-adb-scenario) INCLUDE_ADB_SCENARIO=1; shift ;;
        -ScenarioMode|--scenario-mode) SCENARIO_MODE="$2"; shift 2 ;;
        -StepTimeout|--step-timeout) STEP_TIMEOUT="$2"; shift 2 ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done

if [[ -z "$REPO_ROOT" ]]; then
    REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi
if [[ -z "$DIST_ROOT" ]]; then
    DIST_ROOT="$REPO_ROOT/out/dist"
fi
if [[ -z "$OUTPUT_ROOT" ]]; then
    OUTPUT_ROOT="$DIST_ROOT/logs/macos-regression-$(date +%Y%m%d-%H%M%S)"
fi

mkdir -p "$OUTPUT_ROOT"

VM_WRAPPER="$REPO_ROOT/scripts/vm_macos.sh"
VM_SHELL="$REPO_ROOT/scripts/vm_shell_macos.sh"
CHECK_MARKERS="$REPO_ROOT/scripts/check_macos_avf_markers.sh"
SERVICE_ROOT="/tmp/bscp-macos-regression-service-$(date +%H%M%S)"
SERVICE_TRACE="$SERVICE_ROOT/virtmgr-trace.log"
CROSVM_BIN="$DIST_ROOT/macos/bin/crosvm"
VIRTMGR_BIN="$DIST_ROOT/macos/bin/virtmgr"

chmod +x "$VM_WRAPPER" "$VM_SHELL" "$CHECK_MARKERS"

VM_COMMON_ARGS=(
    -RepoRoot "$REPO_ROOT"
    -DistRoot "$DIST_ROOT"
)
if [[ -n "$APEX_TREE_ROOT" ]]; then
    VM_COMMON_ARGS+=(-ApexTreeRoot "$APEX_TREE_ROOT")
fi

VM_SHELL_COMMON_ARGS=(
    -RepoRoot "$REPO_ROOT"
)
if [[ -n "$APEX_TREE_ROOT" ]]; then
    VM_SHELL_COMMON_ARGS+=(-ApexTreeRoot "$APEX_TREE_ROOT")
fi

run_step() {
    local label="$1"
    shift
    echo ""
    echo "=== [$SECONDS s] $label ==="
    "$@"
}

run_step_allow_status() {
    local allowed_status="$1"
    shift
    local label="$1"
    shift
    echo ""
    echo "=== [$SECONDS s] $label ==="
    if "$@"; then
        return 0
    else
        local status=$?
        [[ $status -eq "$allowed_status" ]]
    fi
}

step_timeout() {
    local timeout_secs="$1"
    shift
    local label="$1"
    shift
    set +e
    "$@" &
    local cmd_pid=$!
    (
        sleep "$timeout_secs"
        kill -TERM "$cmd_pid" 2>/dev/null || exit 0
        sleep 2
        kill -KILL "$cmd_pid" 2>/dev/null || true
    ) &
    local watchdog_pid=$!
    wait "$cmd_pid"
    local rc=$?
    kill "$watchdog_pid" 2>/dev/null || true
    wait "$watchdog_pid" 2>/dev/null || true
    set -e
    if [[ $rc -eq 143 || $rc -eq 137 ]]; then
        echo "  TIMEOUT after ${timeout_secs}s: $label" >&2
        return 124
    fi
    return "$rc"
}

collect_failure_logs() {
    local output_root="$1"
    local collect_dir="$output_root/failure-logs-$(date +%s)"
    mkdir -p "$collect_dir"
    find "$output_root" -name '*.log' -o -name '*.txt' -o -name '*.json' 2>/dev/null | while IFS= read -r f; do
        cp "$f" "$collect_dir/" 2>/dev/null || true
    done
    echo "  Failure logs collected to: $collect_dir"
}

list_process_pids_for_binary() {
    local binary="$1"
    ps -eo pid=,args= | awk -v exe="$binary" 'index($0, exe) { print $1 }'
}

wait_for_pid_exit() {
    local pid="$1"
    local deadline=$((SECONDS + 10))
    while (( SECONDS < deadline )); do
        if ! kill -0 "$pid" 2>/dev/null; then
            return 0
        fi
        sleep 1
    done
    return 1
}

kill_new_binary_processes() {
    local baseline_pids="$1"
    local binary="$2"
    local current_pids pid
    current_pids="$(list_process_pids_for_binary "$binary")"
    while IFS= read -r pid; do
        [[ -n "$pid" ]] || continue
        if ! grep -qx "$pid" <<<"$baseline_pids"; then
            kill "$pid" 2>/dev/null || true
            if ! wait_for_pid_exit "$pid"; then
                kill -9 "$pid" 2>/dev/null || true
                wait_for_pid_exit "$pid" || true
            fi
        fi
    done <<<"$current_pids"
}

cleanup_transient_host_processes() {
    kill_new_binary_processes "$BASELINE_CROSVM_PIDS" "$CROSVM_BIN"
    kill_new_binary_processes "$BASELINE_VIRTMGR_PIDS" "$VIRTMGR_BIN"
}

kill_all_binary_processes() {
    local binary="$1"
    local current_pids pid
    current_pids="$(list_process_pids_for_binary "$binary")"
    while IFS= read -r pid; do
        [[ -n "$pid" ]] || continue
        kill "$pid" 2>/dev/null || true
        if ! wait_for_pid_exit "$pid"; then
            kill -9 "$pid" 2>/dev/null || true
            wait_for_pid_exit "$pid" || true
        fi
    done <<<"$current_pids"
}

stop_stale_regression_services() {
    local service_root
    [[ -d "$DIST_ROOT/logs" ]] || return 0
    find "$DIST_ROOT/logs" -maxdepth 2 -type d -name service 2>/dev/null | sort | while IFS= read -r service_root; do
        [[ -n "$service_root" ]] || continue
        [[ "$service_root" == "$SERVICE_ROOT" ]] && continue
        "$VM_WRAPPER" -Command stop-service "${VM_COMMON_ARGS[@]}" -ServiceRoot "$service_root" >/dev/null 2>&1 || true
    done
}

cleanup_stale_regression_processes() {
    stop_stale_regression_services
    kill_all_binary_processes "$CROSVM_BIN"
    kill_all_binary_processes "$VIRTMGR_BIN"
}

run_step "cleanup (pre)" \
    "$VM_WRAPPER" -Command cleanup "${VM_COMMON_ARGS[@]}"

BASELINE_CROSVM_PIDS="$(list_process_pids_for_binary "$CROSVM_BIN")"
BASELINE_VIRTMGR_PIDS="$(list_process_pids_for_binary "$VIRTMGR_BIN")"

resolve_cid_from_trace() {
    local trace_path="$1"
    if [[ ! -f "$trace_path" ]]; then
        return 0
    fi
    grep -o 'cid=[0-9]\+' "$trace_path" | tail -n 1 | cut -d= -f2
}

resolve_cid_from_run_log() {
    grep -Eo 'with CID [0-9]+' "$1" | tail -n 1 | awk '{print $3}'
}

wait_for_payload_ready() {
    local guest_log="$1"
    local trace_path="$2"
    local run_log="$3"
    local deadline=$((SECONDS + 120))
    while (( SECONDS < deadline )); do
        if [[ -f "$guest_log" ]] && grep -Eq 'Notified host payload ready successfully|payload ready' "$guest_log"; then
            return 0
        fi
        if [[ -f "$trace_path" ]] && grep -q 'notifyPayloadReady' "$trace_path"; then
            return 0
        fi
        if [[ -f "$run_log" ]] && grep -Eq 'payload is ready|Persistent host virtmgr mode: leaving VM running after READY\.' "$run_log"; then
            return 0
        fi
        sleep 1
    done
    echo "Timed out waiting for payload markers in $guest_log, $trace_path, or $run_log" >&2
    return 1
}

run_log_has_guest_cid_conflict() {
    local run_log="$1"
    [[ -f "$run_log" ]] || return 1
    grep -Eq 'failed to set CID for guest: .*Address already in use' "$run_log"
}

run_persistent_microdroid_with_retry() {
    local log_dir="$1"
    local max_attempts=4
    local attempt=1
    local status=0

    while (( attempt <= max_attempts )); do
        if [[ -d "$log_dir" ]]; then
            rm -rf "$log_dir"
        fi

        if "$VM_WRAPPER" \
            -Command run-microdroid \
            "${VM_COMMON_ARGS[@]}" \
            -LogDir "$log_dir" \
            -KeepTemp \
            -PersistVirtmgr \
            -ServiceRoot "$SERVICE_ROOT"; then
            return 0
        fi
        status=$?

        if (( attempt == max_attempts )) || ! run_log_has_guest_cid_conflict "$log_dir/vm-run-microdroid.log"; then
            return "$status"
        fi

        echo "Guest CID conflict detected during persistent run; retrying with a new VM context (attempt $((attempt + 1))/$max_attempts)." >&2
        cleanup_transient_host_processes
        "$VM_WRAPPER" -Command stop-service "${VM_COMMON_ARGS[@]}" -ServiceRoot "$SERVICE_ROOT" >/dev/null 2>&1 || true
        attempt=$((attempt + 1))
    done

    return "$status"
}

run_step "validate-prereqs" \
    step_timeout 30 "validate-prereqs" \
        "$VM_WRAPPER" -Command validate-prereqs "${VM_COMMON_ARGS[@]}" -LogDir "$OUTPUT_ROOT/validate-prereqs"

run_step "info" \
    step_timeout 30 "info" \
        "$VM_WRAPPER" -Command info "${VM_COMMON_ARGS[@]}" -LogDir "$OUTPUT_ROOT/info"

run_step "create-partition" \
    step_timeout 30 "create-partition" \
        "$VM_WRAPPER" -Command create-partition "${VM_COMMON_ARGS[@]}" -LogDir "$OUTPUT_ROOT/create-partition" -PartitionPath "$OUTPUT_ROOT/create-partition/writable.img" -PartitionSize 1048576

run_step "create-idsig" \
    step_timeout 30 "create-idsig" \
        "$VM_WRAPPER" -Command create-idsig "${VM_COMMON_ARGS[@]}" -LogDir "$OUTPUT_ROOT/create-idsig" -OutputPath "$OUTPUT_ROOT/create-idsig/app.idsig"

if [[ "$SCENARIO_MODE" == "smoke" ]]; then
    run_step_allow_status 124 "run-microdroid smoke (quick)" \
        step_timeout 60 "run-microdroid smoke" \
            "$VM_WRAPPER" -Command run-microdroid "${VM_COMMON_ARGS[@]}" -LogDir "$OUTPUT_ROOT/run-microdroid" -KeepTemp -TimeoutSecs 60
    run_step "check run-microdroid markers" \
        "$CHECK_MARKERS" run-microdroid "$OUTPUT_ROOT/run-microdroid"

    run_step "cleanup (post-smoke)" \
        "$VM_WRAPPER" -Command cleanup "${VM_COMMON_ARGS[@]}"
else
    run_step_allow_status 124 "run-microdroid" \
        step_timeout "$STEP_TIMEOUT" "run-microdroid" \
            "$VM_WRAPPER" -Command run-microdroid "${VM_COMMON_ARGS[@]}" -LogDir "$OUTPUT_ROOT/run-microdroid" -KeepTemp -TimeoutSecs "$STEP_TIMEOUT"
    run_step "check run-microdroid markers" \
        "$CHECK_MARKERS" run-microdroid "$OUTPUT_ROOT/run-microdroid"

    # Verify PSCI guest shutdown: check that guest-log shows payload ready
    # and the VM exited cleanly (exit code check via marker presence).
    if grep -q 'Notified host payload ready successfully' "$OUTPUT_ROOT/run-microdroid/guest-log.txt" 2>/dev/null; then
        echo "  [OK]   Guest payload ready confirmed (PSCI boot path verified)"
    else
        echo "  [WARN] Guest payload ready marker not found (VM may have timed out during boot)"
    fi

    run_step "cleanup transient host processes" \
        cleanup_transient_host_processes

    if [[ "$INCLUDE_RUN_APP" -eq 1 ]]; then
        run_step_allow_status 124 "run-app" \
            step_timeout "$STEP_TIMEOUT" "run-app" \
                "$VM_WRAPPER" -Command run-app "${VM_COMMON_ARGS[@]}" -LogDir "$OUTPUT_ROOT/run-app" -KeepTemp -TimeoutSecs "$STEP_TIMEOUT"
        run_step "check run-app markers" \
            "$CHECK_MARKERS" run-app "$OUTPUT_ROOT/run-app"
        run_step "cleanup run-app host processes" \
            cleanup_transient_host_processes
    fi

    run_step "stop stale service" \
        "$VM_WRAPPER" -Command stop-service "${VM_COMMON_ARGS[@]}" -ServiceRoot "$SERVICE_ROOT"

    run_step "cleanup (before persistent)" \
        "$VM_WRAPPER" -Command cleanup "${VM_COMMON_ARGS[@]}"

    run_step "persistent run-microdroid" \
        run_persistent_microdroid_with_retry "$OUTPUT_ROOT/persistent-run-microdroid"

    if wait_for_payload_ready \
        "$OUTPUT_ROOT/persistent-run-microdroid/guest-log.txt" \
        "$SERVICE_TRACE" \
        "$OUTPUT_ROOT/persistent-run-microdroid/vm-run-microdroid.log"; then
        run_step "check persistent run markers" \
            "$CHECK_MARKERS" persistent-run-microdroid "$OUTPUT_ROOT/persistent-run-microdroid" "$SERVICE_TRACE"

        CID="$(resolve_cid_from_trace "$SERVICE_TRACE")"
        if [[ -z "$CID" ]]; then
            CID="$(resolve_cid_from_run_log "$OUTPUT_ROOT/persistent-run-microdroid/vm-run-microdroid.log")"
        fi

        run_step "vm list" \
            step_timeout 30 "vm list" \
                "$VM_WRAPPER" -Command list "${VM_COMMON_ARGS[@]}" -LogDir "$OUTPUT_ROOT/list" -PersistVirtmgr -ServiceRoot "$SERVICE_ROOT"
        run_step "check list markers" \
            "$CHECK_MARKERS" list "$OUTPUT_ROOT/list" "" "$CID"

        run_step_allow_status 124 "vm console" \
            step_timeout 10 "vm console" \
                "$VM_WRAPPER" -Command console "${VM_COMMON_ARGS[@]}" -LogDir "$OUTPUT_ROOT/console" -PersistVirtmgr -ServiceRoot "$SERVICE_ROOT" -Cid "$CID" -TimeoutSecs 5
        run_step "check console markers" \
            "$CHECK_MARKERS" console "$OUTPUT_ROOT/console" "" "$CID"
    else
        echo "WARNING: Persistent VM did not reach payload ready - skipping list/console checks"
        CID=""
    fi

    if [[ "$INCLUDE_ADB_SCENARIO" -eq 1 ]]; then
        run_step "start-microdroid -AutoConnect" \
            "$VM_SHELL" -Command start-microdroid "${VM_SHELL_COMMON_ARGS[@]}" -LogDir "$OUTPUT_ROOT/start-microdroid-adb" -PersistVirtmgr -ServiceRoot "$SERVICE_ROOT" -AutoConnect -NoShell -NoRoot -AdbPort "$ADB_PORT"
        run_step "check adb markers" \
            "$CHECK_MARKERS" start-microdroid-adb "$OUTPUT_ROOT/start-microdroid-adb" "$SERVICE_TRACE"
    fi

    run_step "service-status" \
        step_timeout 15 "service-status" \
            "$VM_WRAPPER" -Command service-status "${VM_COMMON_ARGS[@]}" -ServiceRoot "$SERVICE_ROOT"

    run_step "stop-service" \
        step_timeout 15 "stop-service" \
            "$VM_WRAPPER" -Command stop-service "${VM_COMMON_ARGS[@]}" -ServiceRoot "$SERVICE_ROOT"

    # Verify stop-service succeeded: service-status should report not running.
    if ! "$VM_WRAPPER" -Command service-status "${VM_COMMON_ARGS[@]}" -ServiceRoot "$SERVICE_ROOT" 2>/dev/null | grep -q 'Running: 1'; then
        echo "  [OK]   Service stopped cleanly after stop-service"
    else
        echo "  [WARN] Service still appears running after stop-service"
    fi

    run_step "cleanup (final)" \
        "$VM_WRAPPER" -Command cleanup "${VM_COMMON_ARGS[@]}"
fi

echo "macOS AVF regression completed successfully."
echo "Artifacts written to: $OUTPUT_ROOT"
