#!/usr/bin/env python3
"""Real producer-shaped logs; no VM/service/hypervisor or real sleeps."""
from pathlib import Path
import re
import shlex
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
HELPER = ROOT / 'scripts/avf_trace_evidence.sh'
SESSION = 'virtmgr: run start\n'


def created(cid=2048):
    return f'vm: after vm.start\nCreated VM from payload with CID {cid}, state is STARTED.\n'


def context(cid=2048):
    return f'virtmgr: create_vm_internal vm_context cid={cid} temp_dir=/tmp/virtmgr/{cid}\n'


def events(cid=2048):
    return (f'virtmgr: notifyPayloadStarted cid={cid}\n'
            f'virtmgr: notify_payload_started cid={cid}\n'
            f'virtmgr: notifyPayloadReady cid={cid}\n'
            f'virtmgr: notifyPayloadReady found vm cid={cid}\n'
            f'virtmgr: notify_payload_ready cid={cid}\n')


def extract(path, name):
    m = re.search(r'^' + name + r'\(\) \{\n.*?^\}', path.read_text(), re.M | re.S)
    assert m, name
    return m.group()


class TraceCorrelation(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='bscp-trace-test-')
        self.addCleanup(self.tmp.cleanup)
        self.p = Path(self.tmp.name)
        self.trace = self.p / 'virtmgr-trace.log'
        self.log = self.p / 'vm-run-microdroid.log'
        (self.p / 'adb-connect.log').write_text('connected to 127.0.0.1:8035\ndevice\n')

    def shell(self, body, *args):
        script = 'set -euo pipefail\nsource ' + shlex.quote(str(HELPER)) + '\n' + body
        return subprocess.run(['bash', '-c', script, 'test', *map(str, args)],
                              capture_output=True, text=True, timeout=5)

    def test_trace_boundaries_all_consumers(self):
        good = SESSION + context() + events()
        cases = [
            ('current_ready', good, created(), 0),
            ('unchanged_old_trace_no_current_ready', good, created(), 1),
            ('same_session_other_vm', good + context(4096), created(4096), 1),
            ('wrong_prefix_cid', SESSION + context(20480) + events(20480), created(), 1),
            ('cid_reused_after_restart', good + SESSION + context(), created(), 1),
            ('cid_reused_then_ready', good + SESSION + context() + events(), created(), 0),
            ('same_cid_new_context', good + context(), created(), 1),
            ('other_vm_after_ready', good + context(4096) + events(4096), created(), 0),
            ('other_vm_late_death', good + 'virtmgr: callback_on_died cid=4096 reason=Shutdown\n', created(), 0),
            ('missing_session', context() + events(), created(), 1),
            ('missing_context', SESSION + events(), created(), 1),
            ('missing_cid_in_context', SESSION + 'virtmgr: create_vm_internal vm_context temp_dir=/tmp\n' + events(), created(), 1),
            ('missing_ready_cid', SESSION + context() + 'virtmgr: notify_payload_ready\n', created(), 1),
            ('unknown_rpc', SESSION + context() + 'virtmgr: notifyPayloadReady unknown cid=2048\n', created(), 1),
            ('failed_state', SESSION + context() + 'virtmgr: notifyPayloadReady found vm cid=2048\nvirtmgr: notifyPayloadReady invalid state cid=2048 err=Invalid\n', created(), 1),
            ('entry_rpc_only', SESSION + context() + 'virtmgr: notifyPayloadReady cid=2048\n', created(), 1),
            ('partial_final_trace_line', good.rstrip('\n'), created(), 1),
            ('partial_run_line', good, created().rstrip('\n'), 1),
            ('empty_rotated_trace', '', created(), 1),
            ('rotated_tail_only', events(), created(), 1),
            ('old_ready_before_context', SESSION + events() + context(), created(), 1),
            ('old_ready_before_new_session', good + SESSION, created(), 1),
            ('prefixed_success', SESSION + context() + 'Error: virtmgr: notify_payload_ready cid=2048\n', created(), 1),
            ('suffixed_success', SESSION + context() + 'virtmgr: notify_payload_ready cid=2048 failed\n', created(), 1),
            ('missing_run_cid', good, 'vm: after vm.start\n', 1),
            ('duplicate_run_cid', good, created() + created(), 1),
            ('conflicting_run_cid', good, created() + created(4096), 1),
            ('legacy_bare_trace', 'notifyPayloadStarted\nnotifyPayloadReady\n', created(), 1),
        ]
        for label, trace, run, expected in cases:
            if label != "unchanged_old_trace_no_current_ready" and run.endswith("\n"):
                run += "payload is ready\n"
            self.trace.write_text(trace)
            self.log.write_text(run)
            (self.p / 'vm-run-app.log').write_text(run)
            for platform in ('linux', 'macos'):
                for scenario in ('run-microdroid', 'run-app', 'persistent-run-microdroid', 'start-microdroid-adb'):
                    with self.subTest(case=label, platform=platform, scenario=scenario):
                        proc = subprocess.run(['bash', str(ROOT / 'scripts' / f'check_{platform}_avf_markers.sh'), scenario, str(self.p)], capture_output=True, text=True, timeout=5)
                        self.assertEqual(proc.returncode, expected, proc.stderr)
                for filename, function in [(f'run_{platform}_avf_regression.sh', 'wait_for_payload_ready'),
                                           (f'vm_shell_{platform}.sh', 'wait_for_payload_trace_ready')]:
                    with self.subTest(case=label, platform=platform, consumer=filename):
                        body = 'sleep() { SECONDS=$((SECONDS + 121)); }\n'
                        body += extract(ROOT / 'scripts' / filename, function) + '\n'
                        if function == 'wait_for_payload_ready':
                            body += function + ' "$1/guest-log.txt" "$1/virtmgr-trace.log" "$1/vm-run-microdroid.log"'
                        else:
                            body += function + ' "$1/virtmgr-trace.log" "$1/vm-run-microdroid.log"'
                        proc = self.shell(body, self.p)
                        self.assertEqual(proc.returncode, expected, proc.stderr)

    def test_current_run_cid_ignores_unrelated_trace(self):
        self.log.write_text(created(2048))
        self.trace.write_text(SESSION + context() + events() + 'virtmgr: callback_on_died cid=4096 reason=Shutdown\n')
        for platform in ('linux', 'macos'):
            body = extract(ROOT / 'scripts' / f'run_{platform}_avf_regression.sh', 'resolve_cid_from_run_log')
            body += '\nresolve_cid_from_run_log "$1"'
            proc = self.shell(body, self.log)
            self.assertEqual((proc.returncode, proc.stdout), (0, '2048\n'))
        for suffix in ('', created(), created(4096)):
            self.log.write_text(created() + suffix)
            proc = self.shell('avf_run_cid "$1"', self.log)
            self.assertEqual(proc.returncode, 0 if not suffix else 1)

    def test_explicit_expected_cid_and_debuggable_format(self):
        self.log.write_text(created().replace('Created VM', 'Created debuggable VM') + 'payload is ready\n')
        self.trace.write_text(SESSION + context() + events())
        for expected, rc in [('2048', 0), ('20480', 1), ('20.*', 1), ('-1', 1)]:
            proc = self.shell('avf_trace_has_event "$1" "$2" ready "$3"', self.trace, self.log, expected)
            self.assertEqual(proc.returncode, rc, proc.stderr)

    def test_ready_witness_order_and_detach(self):
        self.trace.write_text(SESSION + context() + events())
        for state in ('STARTED', 'READY', 'FINISHED', 'DEAD'):
            for ready in ('payload is ready\n',
                          'Persistent host virtmgr mode: leaving VM running after READY.\n'):
                for before in (True, False):
                    run = created().replace('state is STARTED', 'state is ' + state)
                    self.log.write_text(ready + run if before else run + ready)
                    proc = self.shell('avf_trace_has_event "$1" "$2" ready', self.trace, self.log)
                    self.assertEqual(proc.returncode, 0, proc.stderr)

    def test_truncate_rotate_then_restore(self):
        self.log.write_text(created() + 'payload is ready\n')
        good = SESSION + context() + events()
        for trace, expected in [(good, 0), ('', 1), (events(), 1), (SESSION + context(), 1), (good, 0)]:
            self.trace.write_text(trace)
            proc = self.shell('avf_trace_has_event "$1" "$2" ready', self.trace, self.log)
            self.assertEqual(proc.returncode, expected, proc.stderr)


if __name__ == '__main__':
    unittest.main()
