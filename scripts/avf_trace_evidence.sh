#!/usr/bin/env bash
# Shared Linux/macOS log contract. Sourced by checkers and wrappers; no side effects.
# A CID is unique only within a virtmgr session. Trace is append-only across restarts.

avf_complete_log() {
    [[ -s "$1" ]] && [[ -z "$(tail -c 1 "$1")" ]]
}

avf_run_cid() {
    local run_log="$1"
    avf_complete_log "$run_log" || return 1
    awk '
        /^Created (debuggable VM|VM) from .* with CID [0-9]+, state is [A-Z_]+\.[[:space:]]*$/ {
            value = $0
            sub(/^.* with CID /, "", value)
            sub(/, state is .*$/, "", value)
            cid = value
            count++
        }
        END {
            if (count != 1) exit 1
            print cid
        }
    ' "$run_log"
}

avf_trace_has_event() {
    local trace_path="$1" run_log="$2" event="${3:-ready}" expected_cid="${4:-}" cid
    [[ "$event" == ready || "$event" == started ]] || return 1
    cid="$(avf_run_cid "$run_log")" || return 1
    if [[ -n "$expected_cid" ]]; then
        [[ "$expected_cid" =~ ^[0-9]+$ && "$expected_cid" == "$cid" ]] || return 1
    fi
    # Trace writes are best-effort and can silently fail. A retained old session
    # plus a reused CID cannot establish freshness without this run-local witness.
    grep -Eq '^(payload is ready|Persistent host virtmgr mode: leaving VM running after READY\.)[[:space:]]*$' "$run_log" || return 1
    avf_complete_log "$trace_path" || return 1
    awk -v cid="$cid" -v event="$event" '
        # Discard earlier service sessions, including a prior use of the same CID.
        $0 == "virtmgr: run start" { session = 1; context = 0; found = 0; next }
        session && /^virtmgr: create_vm_internal vm_context cid=[0-9]+ temp_dir=.+$/ {
            value = $0
            sub(/^virtmgr: create_vm_internal vm_context cid=/, "", value)
            sub(/ temp_dir=.*$/, "", value)
            if (value == cid) { context = 1; found = 0 }
            next
        }
        session && context && $0 == "virtmgr: notify_payload_" event " cid=" cid { found = 1 }
        END { exit !(session && context && found) }
    ' "$trace_path"
}
