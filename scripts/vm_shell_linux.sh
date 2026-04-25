#!/usr/bin/env bash
set -euo pipefail

COMMAND="help"
REPO_ROOT=""
CID=0
ADB_PORT=8035
ADB_PATH="adb"
LOG_DIR=""
WORK_DIR=""
SERVICE_ROOT=""
DEBUG_POLICY_JSON=""
AUTO_CONNECT=0
PERSIST_VIRTMGR=0
NO_SHELL=0
NO_ROOT=0
VM_ARGS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        -Command|--command) COMMAND="$2"; shift 2 ;;
        -RepoRoot|--repo-root) REPO_ROOT="$2"; shift 2 ;;
        -Cid|--cid) CID="$2"; shift 2 ;;
        -AdbPort|--adb-port) ADB_PORT="$2"; shift 2 ;;
        -AdbPath|--adb-path) ADB_PATH="$2"; shift 2 ;;
        -LogDir|--log-dir) LOG_DIR="$2"; shift 2 ;;
        -WorkDir|--work-dir) WORK_DIR="$2"; shift 2 ;;
        -ServiceRoot|--service-root) SERVICE_ROOT="$2"; shift 2 ;;
        -DebugPolicyJson|--debug-policy-json) DEBUG_POLICY_JSON="$2"; shift 2 ;;
        -AutoConnect|--auto-connect) AUTO_CONNECT=1; shift ;;
        -PersistVirtmgr|--persist-virtmgr) PERSIST_VIRTMGR=1; shift ;;
        -NoShell|--no-shell) NO_SHELL=1; shift ;;
        -NoRoot|--no-root) NO_ROOT=1; shift ;;
        --) shift; VM_ARGS+=("$@"); break ;;
        -h|--help) COMMAND="help"; shift ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done

if [[ -z "$REPO_ROOT" ]]; then
    REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi

VM_WRAPPER="$REPO_ROOT/scripts/vm_linux.sh"
[[ -x "$VM_WRAPPER" ]] || chmod +x "$VM_WRAPPER"

if [[ -z "$SERVICE_ROOT" ]]; then
    SERVICE_ROOT="$REPO_ROOT/out/dist/logs/linux-virtmgr-service"
fi
if [[ -z "$LOG_DIR" ]]; then
    LOG_DIR="$REPO_ROOT/out/dist/logs/linux-vm-shell-$(date +%Y%m%d-%H%M%S)"
fi
if [[ -z "$WORK_DIR" ]]; then
    WORK_DIR="$LOG_DIR/work"
fi
mkdir -p "$LOG_DIR" "$WORK_DIR"

service_trace_path() {
    printf '%s\n' "$SERVICE_ROOT/virtmgr-trace.log"
}

resolve_cid_from_trace() {
    local trace_path="$1"
    grep -o 'cid=[0-9]\+' "$trace_path" | tail -n 1 | cut -d= -f2
}

wait_for_trace_pattern() {
    local trace_path="$1"
    local pattern="$2"
    local deadline=$((SECONDS + 90))
    while (( SECONDS < deadline )); do
        [[ -f "$trace_path" ]] && grep -q "$pattern" "$trace_path" && return 0
        sleep 1
    done
    return 1
}

adb_connect() {
    local serial="localhost:$ADB_PORT"
    "$ADB_PATH" disconnect "$serial" >/dev/null 2>&1 || true
    local connected=0
    for _ in $(seq 1 20); do
        if "$ADB_PATH" connect "$serial" | tee -a "$LOG_DIR/adb-connect.log" | grep -Eq 'connected to|already connected to'; then
            connected=1
            break
        fi
        sleep 1
    done
    [[ "$connected" -eq 1 ]] || { echo "adb connect to $serial did not succeed" >&2; return 1; }

    for _ in $(seq 1 20); do
        if "$ADB_PATH" -s "$serial" get-state | tee -a "$LOG_DIR/adb-connect.log" | grep -qx 'device'; then
            break
        fi
        sleep 1
    done

    if [[ "$NO_ROOT" -ne 1 ]]; then
        "$ADB_PATH" -s "$serial" root | tee -a "$LOG_DIR/adb-connect.log" || true
    fi
    if [[ "$NO_SHELL" -ne 1 ]]; then
        "$ADB_PATH" -s "$serial" shell
    fi
}

case "$COMMAND" in
    help)
        cat <<'EOF'
Usage:
  vm_shell_linux.sh -Command start-microdroid [-AutoConnect] [-- extra vm args]
  vm_shell_linux.sh -Command connect [-AdbPort 8035]
EOF
        ;;
    start-microdroid)
        args=(
            -Command run-microdroid
            -RepoRoot "$REPO_ROOT"
            -LogDir "$LOG_DIR"
            -WorkDir "$WORK_DIR"
            -KeepTemp
        )
        if [[ "$PERSIST_VIRTMGR" -eq 1 ]]; then
            args+=(-PersistVirtmgr -ServiceRoot "$SERVICE_ROOT")
        fi
        if [[ -n "$DEBUG_POLICY_JSON" ]]; then
            args+=(-DebugPolicyJson "$DEBUG_POLICY_JSON")
        fi
        if [[ "$AUTO_CONNECT" -eq 1 ]]; then
            args+=(-- --adb-tcp-port "$ADB_PORT")
        fi
        if [[ ${#VM_ARGS[@]} -gt 0 ]]; then
            args+=(-- "${VM_ARGS[@]}")
        fi
        "$VM_WRAPPER" "${args[@]}"
        if [[ "$AUTO_CONNECT" -eq 1 ]]; then
            trace_path="$LOG_DIR/virtmgr-trace.log"
            if [[ "$PERSIST_VIRTMGR" -eq 1 ]]; then
                trace_path="$(service_trace_path)"
            fi
            wait_for_trace_pattern "$trace_path" "notifyPayloadReady"
            CID="${CID:-0}"
            if [[ "$CID" -eq 0 ]]; then
                CID="$(resolve_cid_from_trace "$trace_path")"
            fi
            echo "CID                 : $CID"
            echo "ADB tcp bridge      : localhost:$ADB_PORT"
            adb_connect
        fi
        ;;
    connect)
        adb_connect
        ;;
    *)
        echo "Unsupported command: $COMMAND" >&2
        exit 2
        ;;
esac
