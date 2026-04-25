#!/usr/bin/env bash
set -euo pipefail

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
            assert_contains "$TRACE_PATH" 'notifyPayloadStarted'
            assert_contains "$TRACE_PATH" 'notifyPayloadReady'
        else
            assert_contains "$RUN_LOG" 'Created .* with CID [0-9]+'
            assert_contains "$RUN_LOG" 'vm: after vm.start'
            if [[ -s "$GUEST_LOG" ]]; then
                assert_contains "$GUEST_LOG" 'notifying payload started|payload started|Notified host payload ready successfully|payload ready'
            fi
        fi
        ;;
    persistent-run-microdroid)
        assert_contains "$RUN_LOG" 'Created .* with CID [0-9]+'
        assert_contains "$RUN_LOG" 'vm: after vm.start'
        if [[ -f "$TRACE_PATH" ]]; then
            assert_contains "$TRACE_PATH" 'notifyPayloadReady'
        elif [[ -s "$GUEST_LOG" ]]; then
            assert_contains "$GUEST_LOG" 'Notified host payload ready successfully|payload ready'
        else
            assert_contains "$RUN_LOG" 'payload is ready|Persistent host virtmgr mode: leaving VM running after READY\.'
        fi
        ;;
    start-microdroid-adb)
        if [[ -f "$TRACE_PATH" ]]; then
            assert_contains "$TRACE_PATH" 'notifyPayloadReady'
        else
            assert_contains "$GUEST_LOG" 'Notified host payload ready successfully|payload ready'
        fi
        assert_contains "$LOG_DIR/adb-connect.log" 'connected to|already connected to'
        assert_contains "$LOG_DIR/adb-connect.log" '^device$'
        ;;
    list)
        assert_contains "$LOG_DIR/vm-list.log" 'Running VMs:'
        [[ -z "$EXPECTED_CID" ]] || assert_contains "$LOG_DIR/vm-list.log" "$EXPECTED_CID"
        ;;
    console)
        assert_contains "$LOG_DIR/vm-console.log" 'Connecting to host VM console for CID'
        [[ -z "$EXPECTED_CID" ]] || assert_contains "$LOG_DIR/vm-console.log" "$EXPECTED_CID"
        ;;
    *)
        echo "unsupported scenario: $SCENARIO" >&2
        exit 2
        ;;
esac

echo "Marker check passed: $SCENARIO"
