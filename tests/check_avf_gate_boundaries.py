#!/usr/bin/env python3
"""No-hardware marker and extracted-wait boundary fixtures (stdlib + Bash/awk)."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import tempfile

START = 'Created VM with CID 42\nvm: after vm.start\n'
READY = 'I microdroid_manager: Notified host payload ready successfully\n'
CALLBACK = 'payload is ready\n'
DETACH = 'Persistent host virtmgr mode: leaving VM running after READY.\n'
ADB = 'connected to 127.0.0.1:8035\ndevice\n'


def vm_list(cid, extra=''):
    return f'Running VMs: [\n    VirtualMachineDebugInfo {{\n        cid: {cid},\n{extra}    }},\n]\n'


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--repo-root', type=Path, required=True)
    p.add_argument('--json-output', type=Path)
    args = p.parse_args()
    results = []

    def record(platform, scenario, name, files, expected, cid='', wait=False):
        with tempfile.TemporaryDirectory(prefix='bscp-boundary-') as tmp:
            root = Path(tmp)
            for path, data in files.items():
                (root / path).write_text(data)
            if wait:
                source = (args.repo_root / 'scripts' / f'run_{platform}_avf_regression.sh').read_text()
                function = re.search(r'^wait_for_payload_ready\(\) \{\n.*?^\}', source, re.M | re.S)
                assert function
                # Advance Bash's clock on the first sleep; no actual 120-second wait.
                script = 'set -euo pipefail\nsleep() { SECONDS=$((SECONDS + 121)); }\n'
                script += function.group() + '\nwait_for_payload_ready "$1/guest-log.txt" "$1/virtmgr-trace.log" "$1/vm-run-microdroid.log"\n'
                command = ['bash', '-c', script, 'test', tmp]
            else:
                command = ['bash', str(args.repo_root.resolve() / 'scripts' / f'check_{platform}_avf_markers.sh'), scenario, tmp, '', cid]
            r = subprocess.run(command, capture_output=True, text=True, timeout=5)
            results.append(dict(platform=platform, scenario=scenario, case=name, expected_exit=expected,
                                actual_exit=r.returncode, passed=r.returncode == expected,
                                stdout=r.stdout, stderr=r.stderr))

    for platform in ['linux', 'macos']:
        for scenario in ['persistent-run-microdroid', 'start-microdroid-adb']:
            base = {'vm-run-microdroid.log': START, 'adb-connect.log': ADB}
            for name, guest, expected in [
                ('guest_success', READY, 0),
                ('guest_success_crlf', READY.rstrip('\n') + '\t\r\n', 0),
                ('negative_ready', 'Failed to notify payload ready\n', 1),
                ('not_ready', 'payload ready: false\n', 1),
                ('guest_empty', '', 1),
                ('guest_started', 'Notified host payload started successfully\n', 1),
                ('success_suffix_false', READY.rstrip('\n') + ': false\n', 1),
            ]:
                record(platform, scenario, name, {**base, 'guest-log.txt': guest}, expected)
            record(platform, scenario, 'missing_guest', base, 1)
            record(platform, scenario, 'trace_ready', {**base, 'virtmgr-trace.log': 'notifyPayloadReady\n'}, 0)
            record(platform, scenario, 'empty_trace_not_fallback', {**base, 'virtmgr-trace.log': '', 'guest-log.txt': READY}, 1)
        for name, output, expected in [
            ('callback_exact', CALLBACK, 0), ('callback_crlf', 'payload is ready\t\r\n', 0),
            ('detach_exact', DETACH, 0), ('callback_prefixed_error', 'Error: ' + CALLBACK, 1),
            ('callback_suffix_false', 'payload is ready: false\n', 1),
            ('detach_prefixed_error', 'Error: ' + DETACH, 1),
        ]:
            record(platform, 'persistent-run-microdroid', name, {'vm-run-microdroid.log': START + output}, expected)
        list_cases = [
            ('pretty_42', vm_list(42), '42', 0), ('pretty_142', vm_list(142), '142', 0),
            ('cid_3', vm_list(3), '3', 0), ('cid_max', vm_list(2147483647), '2147483647', 0),
            ('cid_zero', vm_list(0), '0', 0),
            ('larger_cid', vm_list(142), '42', 1), ('suffix_cid', vm_list(420), '42', 1),
            ('wrong_cid_pid_matches', vm_list(142, '        requesterPid: 42,\n'), '42', 1),
            ('wrong_cid_path_matches', vm_list(142, '        temporaryDirectory: "/tmp/vm42",\n'), '42', 1),
            ('whitespace_crlf', 'Running VMs: [\r\n\t cid:\t42, \r\n]\r\n', '42', 0),
            ('multiple_vms', vm_list(142).replace(']\n', '') + '    VirtualMachineDebugInfo {\n        cid: 42,\n    },\n]\n', '42', 0),
            ('bare_number', 'Running VMs:\n42\n', '42', 1),
            ('error_text', 'Running VMs: []\nError: requested CID 42 not found\n', '42', 1),
            ('empty_list', 'Running VMs: []\n', '42', 1),
            ('no_expected_cid', 'Running VMs: []\n', '', 0),
            ('regex_expected_cid', vm_list(42), '4.', 1),
            ('negative_expected_cid', vm_list(42), '-42', 1),
            ('decimal_expected_cid', vm_list(42), '42.0', 1),
            ('cid_field_prefix', 'Running VMs: [\n requester_cid: 42,\n]\n', '42', 1),
            ('cid_field_suffix', 'Running VMs: [\n cid: 42oops,\n]\n', '42', 1),
        ]
        for name, output, cid, expected in list_cases:
            record(platform, 'list', name, {'vm-list.log': output}, expected, cid)
        record(platform, 'list', 'absent_log', {}, 1, '42')
        for name, output, cid, expected in [
            ('exact', 'Connecting to host VM console for CID 42\n', '42', 0),
            ('exact_142', 'Connecting to host VM console for CID 142\n', '142', 0),
            ('crlf', 'Connecting to host VM console for CID 42\t\r\n', '42', 0),
            ('wrong_prefix', 'Connecting to host VM console for CID 142\n', '42', 1),
            ('wrong_suffix', 'Connecting to host VM console for CID 420\n', '42', 1),
            ('other_line_matches', 'Connecting to host VM console for CID 142\npath: /tmp/42\n', '42', 1),
            ('error_prefix', 'Error: Connecting to host VM console for CID 42\n', '42', 1),
            ('error_suffix', 'Connecting to host VM console for CID 42 failed\n', '42', 1),
            ('regex_cid', 'Connecting to host VM console for CID 42\n', '4.', 1),
            ('no_expected_cid', 'Connecting to host VM console for CID 142\n', '', 0),
        ]:
            record(platform, 'console', name, {'vm-console.log': output}, expected, cid)
        record(platform, 'console', 'absent_log', {}, 1, '42')
        for name, files, expected in [
            ('guest_success', {'guest-log.txt': READY}, 0),
            ('negative_guest', {'guest-log.txt': 'Failed to notify payload ready\n'}, 1),
            ('callback', {'vm-run-microdroid.log': CALLBACK}, 0),
            ('detach', {'vm-run-microdroid.log': DETACH}, 0),
            ('callback_error', {'vm-run-microdroid.log': 'Error: ' + CALLBACK}, 1),
            ('callback_suffix', {'vm-run-microdroid.log': 'payload is ready: false\n'}, 1),
            ('trace_ready', {'virtmgr-trace.log': 'notifyPayloadReady\n'}, 0),
            ('missing_all', {}, 1),
            ('guest_success_trailing_space', {'guest-log.txt': READY.rstrip('\n') + '\t\r\n'}, 0),
        ]:
            record(platform, 'wait_for_payload_ready', name, files, expected, wait=True)
    summary = dict(total=len(results), passed=sum(r['passed'] for r in results), results=results)
    if args.json_output:
        args.json_output.write_text(json.dumps(summary, indent=2) + '\n')
    for r in results:
        if not r['passed']:
            print('FAIL', r['platform'], r['scenario'], r['case'], r['expected_exit'], r['actual_exit'])
    print(f"{summary['passed']}/{summary['total']} passed")
    return int(summary['passed'] != summary['total'])


if __name__ == '__main__':
    raise SystemExit(main())
