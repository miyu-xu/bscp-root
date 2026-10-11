import pathlib
import subprocess
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


def function(path, name):
    text = path.read_text()
    start = text.index(name + '() {')
    return text[start:text.index('\n}', start) + 2]


class TimeoutProvenance(unittest.TestCase):
    def run_case(self, body, duration='5', setup='', expected=None):
        for filename, name in [('vm_macos.sh', 'run_command_with_timeout'),
                               ('run_macos_avf_regression.sh', 'step_timeout')]:
            with self.subTest(layer=name), tempfile.TemporaryDirectory() as tmp:
                root = pathlib.Path(tmp)
                payload = root / 'payload'
                payload.write_text('#!/usr/bin/env bash\n' + body + '\n')
                payload.chmod(0o755)
                args = f'{duration} "{payload}"' if name == 'run_command_with_timeout' else f'{duration} fixture "{payload}"'
                script = f'''set -euo pipefail
export TMPDIR="{tmp}"
RUN_LOG_FILE="{tmp}/run.log"
{function(ROOT / 'scripts' / filename, name)}
{setup}
rc=0
{name} {args} || rc=$?
printf '%s\\n' "$rc"
'''
                with (root/'stdout').open('w') as out, (root/'stderr').open('w') as err:
                    result = subprocess.run(['bash', '-c', script], stdout=out, stderr=err, timeout=10)
                self.assertEqual(result.returncode, 0, (root/'stderr').read_text())
                rc = int((root/'stdout').read_text().strip().splitlines()[-1])
                if expected is not None:
                    self.assertIn(rc, expected)
                self.assertEqual(list(root.glob('bscp-timeout.*')), [])

    def test_early_exit_and_signals(self):
        for body, code in [('exit 0', 0), ('exit 1', 1), ('exit 124', 124), ('exit 137', 137), ('exit 143', 143),
                           ('kill -TERM $$', 143), ('kill -KILL $$', 137)]:
            self.run_case(body, expected=[code])

    def test_real_deadline(self):
        self.run_case('exec sleep 3', duration='0.05', expected=[124])

    def test_timer_failure_preserves_status(self):
        self.run_case('sleep 0.05; exit 143', setup='sleep() { return 1; }', expected=[143])

    def test_success_before_deadline(self):
        self.run_case('sleep 0.02; exit 0', duration='0.1', expected=[0])

    def test_deadline_near_success(self):
        # A racing successful exit or a proven deadline is legitimate; signals
        # without persisted evidence must remain failures, never fabricated 124.
        for _ in range(10):
            self.run_case('sleep 0.01; exit 0', duration='0.01', expected=[0, 124, 143])

    def test_allocation_failure_does_not_launch(self):
        self.run_case('exit 99', setup='mktemp() { return 1; }', expected=[1])

    def test_external_signal_after_timer_failure(self):
        self.run_case('sleep 0.05; kill -KILL $$', setup='sleep() { return 1; }', expected=[137])

    def test_recording_race_fails_closed(self):
        # Delay the watchdog between successful TERM and writing its receipt.
        # The parent may reap it first: retaining 143 is deliberately conservative.
        self.run_case('exec sleep 3', duration='0.01', setup='''kill() {
    builtin kill "$@"
    local rc=$?
    if [[ "${1:-}" == -TERM ]]; then command sleep 0.2; fi
    return "$rc"
}''', expected=[143])


if __name__ == '__main__':
    unittest.main()
