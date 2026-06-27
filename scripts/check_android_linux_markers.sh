#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 LOG_DIR" >&2
    exit 2
fi

LOG_DIR="$1"
LOGCAT="$LOG_DIR/logcat-hvc2.txt"
KERNEL_LOG="$LOG_DIR/hvc.txt"

if [[ ! -f "$LOGCAT" || ! -f "$KERNEL_LOG" ]]; then
    echo "Missing Android logs under $LOG_DIR" >&2
    exit 1
fi

require_marker() {
    local pattern="$1"
    local file="$2"
    local label="$3"
    if ! rg -q -F "$pattern" "$file"; then
        echo "Missing marker: $label" >&2
        exit 1
    fi
}

reject_marker() {
    local pattern="$1"
    local file="$2"
    local label="$3"
    if rg -q -F "$pattern" "$file"; then
        echo "Unexpected marker: $label" >&2
        exit 1
    fi
}

require_marker "Finished executing PersistentDataBlockService.onStart" "$LOGCAT" \
    "PersistentDataBlockService initialized"
require_marker "OnBootPhase_1000" "$LOGCAT" "system_server boot phase 1000"
require_marker "processing action (sys.boot_completed=1)" "$KERNEL_LOG" "sys.boot_completed=1"

reject_marker "Service PersistentDataBlockService init timeout" "$LOGCAT" \
    "PersistentDataBlockService timeout"
reject_marker "FATAL EXCEPTION IN SYSTEM PROCESS" "$LOGCAT" "system_server fatal exception"
reject_marker "Exit zygote because system server" "$LOGCAT" "system_server terminated"

echo "Marker check passed: android-linux"
