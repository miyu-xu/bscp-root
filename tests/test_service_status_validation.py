#!/usr/bin/env python3
"""Test extracted macOS status validation only; no VM or real process cleanup."""
import argparse
import json
from pathlib import Path
import re
import shlex
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--repo-root', type=Path, required=True)
    parser.add_argument('--json-output', type=Path)
    args = parser.parse_args()
    source = (args.repo_root / 'scripts/run_macos_avf_regression.sh').read_text()
    function = re.search(r'^verify_service_status_stopped\(\) \{\n.*?^\}', source, re.M | re.S)
    assert function, 'missing status validation helper'
    tail = re.search(r'^    service_status_result=0\n.*?^    fi', source, re.M | re.S)
    assert tail, 'missing final status/cleanup sequence'
    cases = [
        ('stopped', 'Registered: false\nRunning: 0\n', 0, 0),
        ('running', 'Running: 1\n', 0, 1),
        ('failed_empty', '', 42, 1),
        ('failed_stopped_text', 'Running: 0\n', 42, 1),
        ('empty_success', '', 0, 1),
        ('missing_field', 'Registered: false\n', 0, 1),
        ('malformed_field', 'Running: 0oops\n', 0, 1),
        ('duplicate_field', 'Running: 0\nRunning: 0\n', 0, 1),
        ('conflicting_fields', 'Running: 0\nRunning: 1\n', 0, 1),
        ('prefixed_field', 'Error: Running: 0\n', 0, 1),
    ]
    results = []
    for name, output, status_rc, expected in cases:
        script = '''set -euo pipefail
VM_WRAPPER=mock_vm
VM_COMMON_ARGS=()
SERVICE_ROOT=/unused
run_step() { shift; "$@"; }
mock_vm() {
    if [[ "$2" == cleanup ]]; then
        echo FINAL_CLEANUP_CALLED
        return 0
    fi
    printf %s OUTPUT
    return STATUS_RC
}
'''.replace('OUTPUT', shlex.quote(output)).replace('STATUS_RC', str(status_rc))
        script += function.group() + '\n' + tail.group() + '\necho REGRESSION_SUCCEEDED\n'
        result = subprocess.run(['bash', '-c', script], text=True, capture_output=True, timeout=5)
        passed = (result.returncode == expected
                  and result.stdout.count('FINAL_CLEANUP_CALLED') == 1
                  and ('REGRESSION_SUCCEEDED' in result.stdout) == (expected == 0)
                  and ('[OK]' in result.stdout) == (expected == 0))
        results.append(dict(case=name, passed=passed, expected_exit=expected,
                            actual_exit=result.returncode, stdout=result.stdout, stderr=result.stderr))
        print(('PASS' if passed else 'FAIL'), name)
    if args.json_output:
        args.json_output.write_text(json.dumps(results, indent=2) + '\n')
    print(f"{sum(item['passed'] for item in results)}/{len(results)} passed")
    return 0 if all(item['passed'] for item in results) else 1


if __name__ == '__main__':
    raise SystemExit(main())
