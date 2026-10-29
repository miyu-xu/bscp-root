# Foreground Microdroid lifecycle integration

This repository entry integrates the tested common lifecycle implementation while
retaining the platform's original Job/HANDLE, pidfd, or audit-token/kqueue backend.
It is opt-in. Existing `vm_<platform>` defaults and low-level commands are unchanged.
No arbitrary payload/shell service or automatic recovery is introduced.

Use `scripts/microdroid.py --platform linux|macos|windows --state-root ABSOLUTE_ROOT ...`.
POSIX wrappers also accept `--lifecycle` as the first argument. Windows accepts
`-Command lifecycle -WorkDir ABSOLUTE_ROOT` with remaining arguments passed as an
array. Python 3.9+ is required. Keep the foreground controller stdin/stdout open.

The explicitly prepared state root contains `profiles/<platform>.json` and
`closure/<name>.json`. These are deployment-specific pinned asset inventories,
not checked-in credentials or machine discovery. `profile.instance_root` must be
exactly `state-root/instances`; old instances are never adopted. The closure path
must stay under `closure/` without symlinks. Profile/closure SHA256 and each external
asset hash are checked before running and the profile is rechecked after ownership
claim acquisition. No missing config is silently copied from an old directory.

Keep the reviewed candidate profile schema: normal initrd/debug none, fixed payload
port5558, immutable asset hashes, launcher/dist/APK/virtmgr/crosvm paths, environment,
platform closure and its hash, explicitly reviewed runtime gates. Review these paths
against the actual installed artifacts. This lifecycle-only integration does not
rebuild or certify an arbitrary repository checkout of those binary dependencies.
Its acceptance profiles reuse the fixed external runtime assets and wrappers from
the baseline; this is not a claim that unbuilt Host changes have been tested.

Example on Linux, with a new short name and free port (CID2048 is the current Linux
contract; instances are serial):

```sh
bash scripts/vm_linux.sh --lifecycle --state-root /absolute/new-state check --gpu on
bash scripts/vm_linux.sh --lifecycle --state-root /absolute/new-state create --name a --cid 2048 --port 46010 --timeout 240 --gpu on
bash scripts/vm_linux.sh --lifecycle --state-root /absolute/new-state start --name a --gpu on
```

When RUNNING, send `status`, or `request 1 INFO`, then wait for the complete request
response before the next incrementing request ID. `request 2 VADD 2 1 4294967295 2 2`
must produce uint32 results3,1 on the GPU. GPUoff rejects VADD without CPU fallback.
Send `stop` on this same live controller only when no request outcome is unknown.
Wait for STOP/BYE, Guest natural exit, separately recorded service cleanup,
controller exit and genuine output EOF. The output includes a multiline JSON
summary; parse complete JSON, reject duplicate keys and keep reading to EOF.

Only a healthy STOPPED instance can `resume --name a --gpu on`. GPU mode is bound
at create and cannot be changed by resume. Standalone `status` reads the record but
does not claim live observation. Standalone `stop` refuses process takeover; it does
not find an old PID and signal it. UNKNOWN/needsRecovery cannot resume. Never remove
a claim or rebuild/overwrite a failed disk to disguise failure.

Normal task state and resource state are separate. A lost response remains UNKNOWN
even if scoped cleanup proves resources REAPED. Abnormal termination may lose data;
failed disks/claims remain. No KILL escalation or business retry is added. Initial
five-second and later observe-only policies are not hard whole-system deadlines.
Observation/API failures can retain resources. Windows external controller death
and Job-close behavior remain a limitation.

Regression: `python3 -B -m unittest discover -s tests/microdroid -p 'test_*.py' -v`.
The existing regression workflow runs this no-VM suite on its three OS runners.
Real pipeline tests use harmless bounded subprocesses and inject native probes;
they do not replace actual Job/pidfd/audit-token or GPU acceptance. No new runner
privileges, credentials, network permissions or branch-protection changes are used.

Rollback is a revert of the isolated integration commit or selecting the prior
explicit entry for compatible healthy instances. No machine default is changed by
this patch. Failed integration instances must not be opened by older entries.
Push, merge and default activation are separate deployment decisions. A/H3/UI,
arbitrary faults, disk recovery, long stress and performance claims are out of scope.
