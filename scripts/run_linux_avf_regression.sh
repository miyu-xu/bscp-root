#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=""
DIST_ROOT=""
OUTPUT_ROOT=""
ADB_PORT=8035
INCLUDE_RUN_APP=0
INCLUDE_ADB_SCENARIO=0
SCENARIO_MODE="smoke"
STEP_TIMEOUT=300

while [[ $# -gt 0 ]]; do
    case "$1" in
        -RepoRoot|--repo-root) REPO_ROOT="$2"; shift 2 ;;
        -DistRoot|--dist-root) DIST_ROOT="$2"; shift 2 ;;
        -OutputRoot|--output-root) OUTPUT_ROOT="$2"; shift 2 ;;
        -AdbPort|--adb-port) ADB_PORT="$2"; shift 2 ;;
        -IncludeRunApp|--include-run-app) INCLUDE_RUN_APP=1; shift ;;
        -IncludeAdbScenario|--include-adb-scenario) INCLUDE_ADB_SCENARIO=1; shift ;;
        --scenario-mode) SCENARIO_MODE="$2"; shift 2 ;;
        --step-timeout) STEP_TIMEOUT="$2"; shift 2 ;;
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
    OUTPUT_ROOT="$DIST_ROOT/logs/linux-regression-$(date +%Y%m%d-%H%M%S)"
fi

mkdir -p "$OUTPUT_ROOT"

VM_WRAPPER="$REPO_ROOT/scripts/vm_linux.sh"
VM_SHELL="$REPO_ROOT/scripts/vm_shell_linux.sh"
CHECK_MARKERS="$REPO_ROOT/scripts/check_linux_avf_markers.sh"
SERVICE_ROOT="$OUTPUT_ROOT/service"
SERVICE_TRACE="$SERVICE_ROOT/virtmgr-trace.log"
CROSVM_BIN="$DIST_ROOT/linux/bin/crosvm"
VIRTMGR_BIN="$DIST_ROOT/linux/bin/virtmgr"

chmod +x "$VM_WRAPPER" "$VM_SHELL" "$CHECK_MARKERS"

run_step() {
    local label="$1"
    shift
    echo "=== $label ==="
    "$@"
}

step_timeout() {
    local duration="$1"
    shift
    local label="$1"
    shift
    if ! command -v timeout &>/dev/null; then
        run_step "$label" "$@"
        return $?
    fi
    if ! timeout "$duration" "$@"; then
        echo "STEP TIMEOUT: '$label' exceeded ${duration}s" >&2
        collect_failure_logs
        return 1
    fi
}

collect_failure_logs() {
    local archive="$OUTPUT_ROOT/failure-logs.tar.gz"
    find "$OUTPUT_ROOT" -type f \( -name "*.log" -o -name "*.txt" -o -name "vm-console*" \) 2>/dev/null \
        | tar czf "$archive" -T - 2>/dev/null || true
    echo "Failure logs collected: $archive"
}

run_step_allow_status() {
    local allowed_status="$1"
    shift
    local label="$1"
    shift
    echo "=== $label ==="
    if "$@"; then
        return 0
    else
        local status=$?
        [[ $status -eq "$allowed_status" ]]
    fi
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
    find "$DIST_ROOT/logs" -maxdepth 2 -type d -name service 2>/dev/null | sort | while IFS= read -r service_root; do
        [[ -n "$service_root" ]] || continue
        [[ "$service_root" == "$SERVICE_ROOT" ]] && continue
        "$VM_WRAPPER" -Command stop-service -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -ServiceRoot "$service_root" >/dev/null 2>&1 || true
    done
}

cleanup_stale_regression_processes() {
    stop_stale_regression_services
    kill_all_binary_processes "$CROSVM_BIN"
    kill_all_binary_processes "$VIRTMGR_BIN"
}

run_step "cleanup stale regression processes" \
    cleanup_stale_regression_processes

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

wait_for_trace_pattern() {
    local trace_path="$1"
    local pattern="$2"
    local deadline=$((SECONDS + 30))
    while (( SECONDS < deadline )); do
        [[ -f "$trace_path" ]] && grep -q "$pattern" "$trace_path" && return 0
        sleep 1
    done
    echo "Timed out waiting for pattern '$pattern' in $trace_path" >&2
    return 1
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
            -RepoRoot "$REPO_ROOT" \
            -DistRoot "$DIST_ROOT" \
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
        "$VM_WRAPPER" -Command stop-service -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -ServiceRoot "$SERVICE_ROOT" >/dev/null 2>&1 || true
        attempt=$((attempt + 1))
    done

    return "$status"
}

run_step "validate-prereqs" \
    "$VM_WRAPPER" -Command validate-prereqs -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -LogDir "$OUTPUT_ROOT/validate-prereqs"

run_step "info" \
    "$VM_WRAPPER" -Command info -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -LogDir "$OUTPUT_ROOT/info"

run_step "create-partition" \
    "$VM_WRAPPER" -Command create-partition -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -LogDir "$OUTPUT_ROOT/create-partition" -PartitionPath "$OUTPUT_ROOT/create-partition/writable.img" -PartitionSize 1048576

run_step "create-idsig" \
    "$VM_WRAPPER" -Command create-idsig -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -LogDir "$OUTPUT_ROOT/create-idsig" -OutputPath "$OUTPUT_ROOT/create-idsig/app.idsig"

run_step_allow_status 124 "run-microdroid" \
    "$VM_WRAPPER" -Command run-microdroid -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -LogDir "$OUTPUT_ROOT/run-microdroid" -KeepTemp -TimeoutSecs 120
run_step "check run-microdroid markers" \
    "$CHECK_MARKERS" run-microdroid "$OUTPUT_ROOT/run-microdroid"

run_step "cleanup transient host processes" \
    cleanup_transient_host_processes

if [[ "$INCLUDE_RUN_APP" -eq 1 ]]; then
    run_step_allow_status 124 "run-app" \
        "$VM_WRAPPER" -Command run-app -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -LogDir "$OUTPUT_ROOT/run-app" -KeepTemp -TimeoutSecs 120
    run_step "check run-app markers" \
        "$CHECK_MARKERS" run-app "$OUTPUT_ROOT/run-app"
    run_step "cleanup run-app host processes" \
        cleanup_transient_host_processes
fi

run_step "stop stale service" \
    "$VM_WRAPPER" -Command stop-service -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -ServiceRoot "$SERVICE_ROOT"

run_step "persistent run-microdroid" \
    run_persistent_microdroid_with_retry "$OUTPUT_ROOT/persistent-run-microdroid"
wait_for_payload_ready \
    "$OUTPUT_ROOT/persistent-run-microdroid/guest-log.txt" \
    "$SERVICE_TRACE" \
    "$OUTPUT_ROOT/persistent-run-microdroid/vm-run-microdroid.log"
run_step "check persistent run markers" \
    "$CHECK_MARKERS" persistent-run-microdroid "$OUTPUT_ROOT/persistent-run-microdroid" "$SERVICE_TRACE"

CID="$(resolve_cid_from_trace "$SERVICE_TRACE")"
if [[ -z "$CID" ]]; then
    CID="$(resolve_cid_from_run_log "$OUTPUT_ROOT/persistent-run-microdroid/vm-run-microdroid.log")"
fi

run_step "vm list" \
    "$VM_WRAPPER" -Command list -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -LogDir "$OUTPUT_ROOT/list" -PersistVirtmgr -ServiceRoot "$SERVICE_ROOT"
run_step "check list markers" \
    "$CHECK_MARKERS" list "$OUTPUT_ROOT/list" "" "$CID"

run_step "vm console" \
    timeout 3s "$VM_WRAPPER" -Command console -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -LogDir "$OUTPUT_ROOT/console" -PersistVirtmgr -ServiceRoot "$SERVICE_ROOT" -Cid "$CID" || [[ $? -eq 124 ]]
run_step "check console markers" \
    "$CHECK_MARKERS" console "$OUTPUT_ROOT/console" "" "$CID"

if [[ "$INCLUDE_ADB_SCENARIO" -eq 1 ]]; then
    run_step "start-microdroid -AutoConnect" \
        "$VM_SHELL" -Command start-microdroid -RepoRoot "$REPO_ROOT" -LogDir "$OUTPUT_ROOT/start-microdroid-adb" -PersistVirtmgr -ServiceRoot "$SERVICE_ROOT" -AutoConnect -NoShell -NoRoot -AdbPort "$ADB_PORT"
    run_step "check adb markers" \
        "$CHECK_MARKERS" start-microdroid-adb "$OUTPUT_ROOT/start-microdroid-adb" "$SERVICE_TRACE"
fi

run_step "service-status" \
    "$VM_WRAPPER" -Command service-status -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -ServiceRoot "$SERVICE_ROOT"

run_step "stop-service" \
    "$VM_WRAPPER" -Command stop-service -RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT" -ServiceRoot "$SERVICE_ROOT"

echo "Linux AVF regression completed successfully."
echo "Artifacts written to: $OUTPUT_ROOT"
