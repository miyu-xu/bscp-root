#!/usr/bin/env python3
"""Hardware-free regression tests of the actual extracted retry function.

No wrapper, VM, service, process cleanup, or user-machine access is invoked.
Only the retry helper and CID-conflict predicate are extracted from source.
All external operations are substituted with shell functions in a temporary root.
"""
import argparse
import json
from pathlib import Path
import re
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--repo-root', type=Path, required=True)
    parser.add_argument('--json-output', type=Path)
    args = parser.parse_args()
    results = []
    cases = [('success', 0, 1), ('non_conflict_failure', 42, 1),
             ('conflict_exhausted', 42, 4), ('conflict_then_success', 0, 2)]
    for platform in ['linux', 'macos']:
        source = (args.repo_root / 'scripts' / f'run_{platform}_avf_regression.sh').read_text()
        functions = []
        for name in ['run_log_has_guest_cid_conflict', 'run_persistent_microdroid_with_retry']:
            match = re.search(r'^' + name + r'\(\) \{\n.*?^\}', source, re.M | re.S)
            assert match, name
            functions.append(match.group())
        for case, expected_rc, expected_attempts in cases:
            with tempfile.TemporaryDirectory(prefix='bscp-retry-') as tmp:
                script = '''set -euo pipefail
REPO_ROOT="$1"
DIST_ROOT="$1"
SERVICE_ROOT="$1/service"
VM_COMMON_ARGS=()
VM_WRAPPER=mock_vm
CASE="$2"
ATTEMPTS=0
cleanup_transient_host_processes() { :; }
mock_vm() {
    local command="" log_dir=""
    while (( $# )); do
        case "$1" in
            -Command) command="$2"; shift 2;;
            -LogDir) log_dir="$2"; shift 2;;
            *) shift;;
        esac
    done
    [[ "$command" == stop-service ]] && return 0
    ATTEMPTS=$((ATTEMPTS + 1))
    mkdir -p "$log_dir"
    if [[ "$CASE" == conflict* ]]; then
        echo 'failed to set CID for guest: Address already in use' > "$log_dir/vm-run-microdroid.log"
    fi
    [[ "$CASE" == success ]] && return 0
    [[ "$CASE" == conflict_then_success && "$ATTEMPTS" -eq 2 ]] && return 0
    return 42
}
''' + '\n'.join(functions) + '''
if run_persistent_microdroid_with_retry "$1/logs"; then
    RC=0
else
    RC=$?
fi
printf 'RESULT %s %s\\n' "$RC" "$ATTEMPTS"
'''
                completed = subprocess.run(['bash', '-c', script, 'test', tmp, case],
                                           capture_output=True, text=True, timeout=5)
                match = re.search(r'^RESULT (\d+) (\d+)$', completed.stdout, re.M)
                actual = tuple(map(int, match.groups())) if match else None
                results.append(dict(platform=platform, case=case,
                                    expected=[expected_rc, expected_attempts],
                                    actual=actual, passed=actual == (expected_rc, expected_attempts),
                                    harness_exit=completed.returncode, stderr=completed.stderr))
    if args.json_output:
        args.json_output.write_text(json.dumps(results, indent=2) + '\n')
    for item in results:
        print(('PASS' if item['passed'] else 'FAIL'), item['platform'], item['case'],
              'expected=', item['expected'], 'actual=', item['actual'])
    print(f"{sum(r['passed'] for r in results)}/{len(results)} passed")
    return 0 if all(r['passed'] for r in results) else 1


if __name__ == '__main__':
    raise SystemExit(main())
