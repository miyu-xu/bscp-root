"""Portable integrity/application regression; no VM, network or installation."""
import hashlib
import pathlib
import shutil
import subprocess
import sys
import tempfile

B = pathlib.Path(__file__).resolve().parent
for line in (B / 'SHA256SUMS').read_text().splitlines():
    digest, rel = line.split('  ', 1)
    path = pathlib.PurePosixPath(rel)
    assert not path.is_absolute() and '..' not in path.parts
    assert hashlib.sha256((B / path).read_bytes()).hexdigest() == digest, rel
with tempfile.TemporaryDirectory(prefix='bscp-overlay-') as tmp:
    tmp = pathlib.Path(tmp)
    def run(host, output):
        return subprocess.run([sys.executable, str(B/'apply_candidate.py'),
            '--host', str(host), '--guest', str(B/'snapshots/guest-before'),
            '--output', str(output)], capture_output=True, text=True)
    good = run(B/'snapshots/host-before', tmp/'good')
    assert good.returncode == 0, good.stderr
    assert run(B/'snapshots/host-before', tmp/'good').returncode != 0
    shutil.copytree(B/'snapshots/host-before', tmp/'bad')
    target = tmp/'bad/host/ring_stream.cpp'
    target.write_bytes(target.read_bytes() + b'\n// baseline mismatch\n')
    bad = run(tmp/'bad', tmp/'must-not-exist')
    assert bad.returncode != 0 and not (tmp/'must-not-exist').exists()
print('PASS integrity, exact application, no-overwrite, fail-before-write baseline guard')
