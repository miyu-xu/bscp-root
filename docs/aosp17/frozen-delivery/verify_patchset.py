"""Verify captured patch bytes and parse syntax; never apply to a source tree."""
from pathlib import Path
import hashlib
import json
import subprocess

ROOT = Path(__file__).resolve().parent

def main():
    base = ROOT / 'linux-guest-patchset'
    manifest = json.loads((base / 'manifest.json').read_text(encoding='utf-8'))
    rows = []
    for entry in manifest['rows']:
        patch = base / entry['patch_file']
        data = patch.read_bytes()
        assert hashlib.sha256(data).hexdigest() == entry['diff_sha256'], patch
        if data:
            result = subprocess.run(['git', 'apply', '--numstat', str(patch)],
                                    capture_output=True, check=True)
            parsed = result.stdout.decode('utf-8')
        else:
            parsed = 'empty tracked diff; new files verified separately'
        for new in entry['untracked']:
            path = base / 'untracked' / entry['path'] / new['path']
            assert path.resolve().is_relative_to(base.resolve()), path
            content = path.read_bytes()
            assert len(content) == new['bytes'], path
            assert hashlib.sha256(content).hexdigest() == new['sha256'], path
        rows.append({'project': entry['path'], 'base_sha': entry['base_sha'],
                     'patch_sha256': entry['diff_sha256'], 'numstat': parsed,
                     'new_files_verified': len(entry['untracked'])})
    print(json.dumps({'state': 'BYTES_AND_PATCH_SYNTAX_VERIFIED', 'rows': rows,
                      'applied': False, 'build_repeated': False}, indent=2))

if __name__ == '__main__':
    main()
