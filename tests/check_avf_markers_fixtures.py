#!/usr/bin/env python3
"""Exercise AVF log checkers with synthetic files, without starting a VM.

Run with Python's standard library and bash/grep/awk only:
    python3 tests/check_avf_markers_fixtures.py
    python3 tests/check_avf_markers_fixtures.py --repo-root /path/to/checkout

Exit 0 means all checker outcomes match the readiness contract. This is not
evidence that any guest, hypervisor, graphics backend, or native host passed.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import tempfile
from pathlib import Path


START = "Created VM from payload with CID 42, state is STARTED.\nvm: after vm.start\n"
READY = "Notified host payload ready successfully\n"
TRACE = "notifyPayloadStarted\nnotifyPayloadReady\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--json-output", type=Path)
    args = parser.parse_args()
    results = []

    def run_case(checker: Path, scenario: str, name: str, files: dict[str, str],
                 expected: int, extra: tuple[str, ...] = ()) -> None:
        with tempfile.TemporaryDirectory(prefix="bscp-avf-marker-") as tmp:
            root = Path(tmp)
            for filename, content in files.items():
                (root / filename).write_text(content, encoding="utf-8")
            command = ["bash", str(checker), scenario, str(root)]
            command.extend(str(root / value[1:]) if value.startswith("@") else value
                           for value in extra)
            completed = subprocess.run(command, capture_output=True, text=True, timeout=5)
            results.append({"checker": checker.name, "scenario": scenario, "case": name,
                            "expected_exit": expected, "actual_exit": completed.returncode,
                            "matched": completed.returncode == expected,
                            "stdout": completed.stdout, "stderr": completed.stderr})

    for platform in ("linux", "macos"):
        checker = args.repo_root.resolve() / "scripts" / f"check_{platform}_avf_markers.sh"
        if not checker.is_file():
            parser.error(f"missing checker: {checker}")
        for scenario in ("run-microdroid", "run-app"):
            log = f"vm-{scenario}.log"
            cases = [
                ("missing_all_logs", {}, 1, ()),
                ("creation_only", {log: START}, 1, ()),
                ("empty_guest_log", {log: START, "guest-log.txt": ""}, 1, ()),
                ("payload_started_only", {log: START, "guest-log.txt": "notifying payload started\n"}, 1, ()),
                ("unrelated_guest_log", {log: START, "guest-log.txt": "kernel booting\n"}, 1, ()),
                ("guest_ready", {log: START, "guest-log.txt": "I microdroid_manager: " + READY}, 0, ()),
                ("callback_ready_missing_guest", {log: START + "payload is ready\n"}, 0, ()),
                ("callback_ready_empty_guest", {log: START + "payload is ready\n", "guest-log.txt": ""}, 0, ()),
                ("callback_ready_started_guest", {log: START + "payload is ready\n", "guest-log.txt": "notifying payload started\n"}, 0, ()),
                ("callback_ready_other_guest_output", {log: START + "payload is ready\n", "guest-log.txt": "kernel booting\n"}, 0, ()),
                ("negative_ready_text", {log: START, "guest-log.txt": "Failed to notify payload ready\n"}, 1, ()),
                ("ready_callback_requires_whole_line", {log: START + "payload is ready: false\n"}, 1, ()),
                ("missing_creation_marker", {log: "vm: after vm.start\n", "guest-log.txt": READY}, 1, ()),
                ("missing_start_marker", {log: "Created VM with CID 42\n", "guest-log.txt": READY}, 1, ()),
                ("empty_trace_is_authoritative", {log: START, "virtmgr-trace.log": "", "guest-log.txt": READY}, 1, ()),
                ("started_trace_is_not_ready", {log: START, "virtmgr-trace.log": "notifyPayloadStarted\n", "guest-log.txt": READY}, 1, ()),
                ("ready_trace_requires_started", {"virtmgr-trace.log": "notifyPayloadReady\n"}, 1, ()),
                ("complete_trace", {"virtmgr-trace.log": TRACE}, 0, ()),
                ("explicit_trace_path", {"custom-trace.log": TRACE}, 0, ("@custom-trace.log",)),
                ("other_scenario_log_not_accepted", {"vm-run.log": START, "guest-log.txt": READY}, 1, ()),
            ]
            for name, files, expected, extra in cases:
                run_case(checker, scenario, name, files, expected, extra)

        # Positive controls use producer-shaped output; scenario selection remains unchanged.
        controls = [
            ("persistent-run-microdroid", "persistent_trace_ready", {"vm-run-microdroid.log": START, "virtmgr-trace.log": "notifyPayloadReady\n"}, 0, ()),
            ("persistent-run-microdroid", "persistent_guest_ready", {"vm-run-microdroid.log": START, "guest-log.txt": READY}, 0, ()),
            ("persistent-run-microdroid", "persistent_callback_ready", {"vm-run-microdroid.log": START + "payload is ready\n"}, 0, ()),
            ("persistent-run-microdroid", "persistent_detach_ready", {"vm-run-microdroid.log": START + "Persistent host virtmgr mode: leaving VM running after READY.\n"}, 0, ()),
            ("persistent-run-microdroid", "persistent_missing_ready", {"vm-run-microdroid.log": START}, 1, ()),
            ("start-microdroid-adb", "adb_ready", {"virtmgr-trace.log": "notifyPayloadReady\n", "adb-connect.log": "connected to 127.0.0.1:8035\ndevice\n"}, 0, ()),
            ("start-microdroid-adb", "adb_requires_device", {"virtmgr-trace.log": "notifyPayloadReady\n", "adb-connect.log": "connected to 127.0.0.1:8035\n"}, 1, ()),
            ("list", "list_expected_cid", {"vm-list.log": "Running VMs: [\n    VirtualMachineDebugInfo {\n        cid: 42,\n    },\n]\n"}, 0, ("", "42")),
            ("list", "list_wrong_cid", {"vm-list.log": "Running VMs: [\n    VirtualMachineDebugInfo {\n        cid: 43,\n    },\n]\n"}, 1, ("", "42")),
            ("console", "console_expected_cid", {"vm-console.log": "Connecting to host VM console for CID 42\n"}, 0, ("", "42")),
            ("console", "console_wrong_cid", {"vm-console.log": "Connecting to host VM console for CID 43\n"}, 1, ("", "42")),
            ("unsupported", "unsupported_scenario", {}, 2, ()),
        ]
        for scenario, name, files, expected, extra in controls:
            run_case(checker, scenario, name, files, expected, extra)

    failures = [result for result in results if not result["matched"]]
    for result in failures:
        print(f"FAIL {result['checker']} {result['scenario']} {result['case']}: "
              f"expected {result['expected_exit']}, got {result['actual_exit']}")
    summary = {"scope": "synthetic log fixtures only; no VM launched",
               "total": len(results), "passed": len(results) - len(failures),
               "failed": len(failures), "results": results}
    if args.json_output:
        args.json_output.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(f"{len(results) - len(failures)}/{len(results)} fixture outcomes matched")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
