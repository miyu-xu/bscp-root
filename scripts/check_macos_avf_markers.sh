#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/avf_trace_evidence.sh"

SCENARIO="${1:-}"
LOG_DIR="${2:-}"
TRACE_PATH="${3:-}"
EXPECTED_CID="${4:-}"

[[ -n "$SCENARIO" && -n "$LOG_DIR" ]] || {
    echo "usage: check_macos_avf_markers.sh <scenario> <log_dir> [trace_path] [expected_cid]" >&2
    exit 2
}

assert_contains() {
    local path="$1"
    local pattern="$2"
    [[ -f "$path" ]] || { echo "missing file: $path" >&2; exit 1; }
    grep -Eq "$pattern" "$path" || { echo "missing pattern '$pattern' in $path" >&2; exit 1; }
}


assert_expected_cid() {
    local path="$1"
    local kind="$2"
    [[ -z "$EXPECTED_CID" ]] && return 0
    # CID is a decimal field, never a regular expression or arbitrary substring.
    [[ "$EXPECTED_CID" =~ ^[0-9]+$ ]] || {
        echo "invalid expected CID: $EXPECTED_CID" >&2; exit 1;
    }
    if [[ "$kind" == "list" ]]; then
        # vm list uses pretty Rust Debug output: a named cid field on its own line.
        # Match the field rather than requesterPid, paths, or another CID's digits.
        awk -v expected="$EXPECTED_CID" '
            /^[[:space:]]*cid:[[:space:]]*[0-9]+,[[:space:]]*$/ {
                value = $0
                sub(/^[[:space:]]*cid:[[:space:]]*/, "", value)
                sub(/,[[:space:]]*$/, "", value)
                if (value == expected) found = 1
            }
            END { exit !found }
        ' "$path" || { echo "missing CID $EXPECTED_CID in $path" >&2; exit 1; }
    else
        assert_contains "$path" "^Connecting to host VM console for CID ${EXPECTED_CID}[[:space:]]*$"
    fi
}

if [[ -z "$TRACE_PATH" ]]; then
    TRACE_PATH="$LOG_DIR/virtmgr-trace.log"
fi

GUEST_LOG="$LOG_DIR/guest-log.txt"
RUN_LOG="$LOG_DIR/vm-run-microdroid.log"

case "$SCENARIO" in
    run-microdroid|run-app)
        if [[ "$SCENARIO" == "run-app" ]]; then
            RUN_LOG="$LOG_DIR/vm-run-app.log"
        fi
        if [[ -f "$TRACE_PATH" ]]; then
            avf_trace_has_event "$TRACE_PATH" "$RUN_LOG" started "$EXPECTED_CID" || { echo "missing current-run payload-started trace evidence" >&2; exit 1; }
            avf_trace_has_event "$TRACE_PATH" "$RUN_LOG" ready "$EXPECTED_CID" || { echo "missing current-run payload-ready trace evidence" >&2; exit 1; }
        else
            assert_contains "$RUN_LOG" 'Created .* with CID [0-9]+'
            assert_contains "$RUN_LOG" 'vm: after vm.start'
            # VM creation and payload start are not evidence of payload readiness.
            # The wrapper captures the vm client's Ready callback in RUN_LOG.
            if [[ -s "$GUEST_LOG" ]] && grep -Eq 'Notified host payload ready successfully[[:space:]]*$' "$GUEST_LOG"; then
                :
            elif grep -Eq '^payload is ready[[:space:]]*$' "$RUN_LOG"; then
                :
            else
                echo "missing payload-ready evidence in $GUEST_LOG and $RUN_LOG" >&2
                exit 1
            fi
        fi
        ;;
    persistent-run-microdroid)
        assert_contains "$RUN_LOG" 'Created .* with CID [0-9]+'
        assert_contains "$RUN_LOG" 'vm: after vm.start'
        if [[ -f "$TRACE_PATH" ]]; then
            avf_trace_has_event "$TRACE_PATH" "$RUN_LOG" ready "$EXPECTED_CID" || { echo "missing current-run payload-ready trace evidence" >&2; exit 1; }
        elif [[ -s "$GUEST_LOG" ]]; then
            assert_contains "$GUEST_LOG" 'Notified host payload ready successfully[[:space:]]*$'
        else
            assert_contains "$RUN_LOG" '^(payload is ready|Persistent host virtmgr mode: leaving VM running after READY\.)[[:space:]]*$'
        fi
        ;;
    start-microdroid-adb)
        if [[ -f "$TRACE_PATH" ]]; then
            avf_trace_has_event "$TRACE_PATH" "$RUN_LOG" ready "$EXPECTED_CID" || { echo "missing current-run payload-ready trace evidence" >&2; exit 1; }
        else
            assert_contains "$GUEST_LOG" 'Notified host payload ready successfully[[:space:]]*$'
        fi
        assert_contains "$LOG_DIR/adb-connect.log" 'connected to|already connected to'
        assert_contains "$LOG_DIR/adb-connect.log" '^device$'
        ;;
    list)
        assert_contains "$LOG_DIR/vm-list.log" 'Running VMs:'
        assert_expected_cid "$LOG_DIR/vm-list.log" list
        ;;
    console)
        assert_contains "$LOG_DIR/vm-console.log" 'Connecting to host VM console for CID'
        assert_expected_cid "$LOG_DIR/vm-console.log" console
        ;;
    *)
        echo "unsupported scenario: $SCENARIO" >&2
        exit 2
        ;;
esac

echo "Marker check passed: $SCENARIO"
