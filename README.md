# BSCP

[简体中文](README.zh-CN.md) | English

BSCP is a cross-platform, security-isolated Android compute platform. It brings the Android
Virtualization Framework host workflow to Linux/KVM, macOS/HVF, and Windows/WHPX while retaining
the same guest-control model: `virtmgr` and `vm` manage payloads, `crosvm` supplies the virtual
machine monitor, and Microdroid provides the primary minimal Android guest.

The project is designed for isolated workloads, reproducible host builds, and portable compute
rather than for phone emulation. Full Android through a Cuttlefish-derived image is an optional
compatibility path and is intentionally documented after the Microdroid path.

> The manifest repository is the canonical checkout and release entry point. Start there for
> pinned component revisions and multi-repository synchronization.

## Platform model

| Host | Hypervisor | Primary guest | Status boundary |
| --- | --- | --- | --- |
| Linux x86_64/arm64 | KVM | Microdroid | Reference development and validation host |
| macOS arm64 | Hypervisor.framework | arm64 Microdroid | Requires an arm64 `com.android.virt` APEX tree |
| Windows x86_64 | Windows Hypervisor Platform | x86_64 Microdroid | MinGW host toolchain; protected-VM support is host dependent |

Security isolation is layered: hardware virtualization, a dedicated guest kernel, explicit
virtio devices, least-privilege host services, authenticated payload metadata, and per-instance
storage. See [Security model](doc/SECURITY.md) before handling untrusted payloads.

## Repository layout

This repository owns orchestration, host scripts, release documentation, tests, and firmware
assets. Source projects under `external/`, `frameworks/`, `hardware/`, `packages/`, and `system/`
are separate Git repositories managed by `repo`; they are deliberately not duplicated in this
repository's Git index.

Key components are described in [Architecture](doc/ARCHITECTURE.md). Script responsibilities and
supported entry points are indexed in [Operations](doc/OPERATIONS.md).

## Quick start: Microdroid

Initialize the workspace from the manifest repository, then build from this repository root.

Linux or macOS:

```bash
./build_all.sh
./scripts/vm_linux.sh --command validate-prereqs   # Linux
./scripts/vm_linux.sh --command run-microdroid

# macOS uses the corresponding wrapper and an arm64 APEX tree.
MACOS_AVF_APEX_TREE_SOURCE=/path/to/arm64/apex_tree ./build_all.sh
./scripts/vm_macos.sh --command validate-prereqs
./scripts/vm_macos.sh --command run-microdroid
```

Windows PowerShell:

```powershell
.\build_all.bat
.\scripts\vm_windows.ps1 -Command validate-prereqs
.\scripts\run_microdroid_windows.ps1
```

Run the platform regression wrapper after a successful build:

```bash
./scripts/run_linux_avf_regression.sh
./scripts/run_macos_avf_regression.sh
```

```powershell
.\scripts\run_windows_avf_regression.ps1
```

See [Deployment](doc/DEPLOYMENT.md) for toolchain prerequisites, artifacts, APEX staging, and
production hardening.

## No-hardware AVF checker tests

The Linux/macOS log checkers require Ready evidence rather than creation/start messages,
match the requested CID field or complete console line, and preserve persistent-retry
failure exit codes. Timeout, scenario selection, and cleanup policies are unchanged.

Trace-based checks require this run’s client Ready callback or persistent detach line,
use the current run log’s Created CID, and require successful
`notify_payload_ready` evidence after that CID’s creation in the latest virtmgr session.
RPC-entry `notifyPayloadReady` text, other VMs, and earlier sessions do not count.
Incomplete or rotated traces missing the required session/context records fail closed;
keep the complete trace and current run log together. Trace writes can silently fail,
so retained trace alone cannot prove freshness. These conservative checks can reject a
Ready VM when trace evidence is incomplete. This is log correlation under the
wrapper’s per-service trace ownership convention, not independent VM liveness proof.
No-trace guest/client Ready fallbacks remain supported.

Run these focused checks with Python 3 and Bash, grep, and awk; no VM is started:

```bash
python3 tests/check_avf_markers_fixtures.py
python3 tests/check_avf_gate_boundaries.py --repo-root .
python3 tests/test_retry_exit_status.py --repo-root .
python3 -m unittest discover -s tests -p test_avf_trace_correlation.py -v
python3 tests/test_service_status_validation.py --repo-root .
python3 -m unittest discover -s tests -v
```

Synthetic log and mocked retry tests do not verify native macOS/Windows execution,
guest boot/exit, hypervisors, graphics, or release readiness. The macOS UDS unit test
is skipped on other hosts. Platform prerequisite and full regression checks are still required.

The macOS full regression requires a successful post-stop status command with one
explicit `Running: 0` field. Failed or ambiguous status output fails validation
after final cleanup. This verifies the wrapper's status report, not independent
process-exit or guest-shutdown evidence.

## Optional full Android path

The Cuttlefish-derived path is for compatibility testing, graphics bring-up, and workloads that
require a full Android framework. It is not the default runtime. Start with
[Cuttlefish compatibility](doc/CUTTLEFISH.md), then use the platform-specific `run_android_*`
wrappers. Keep generated images and writable overlays outside Git.

## Release and contribution policy

- Build outputs live under `out/` and must never be committed.
- Component changes belong to their component repository, not this orchestration repository.
- Release branches must pass shell syntax checks, Python compilation/tests, documentation-link
  checks, forbidden-identity checks, and commit-metadata checks.
- GitHub Actions are intentionally not part of this repository; validation is performed by the
  documented local or downstream release pipeline.

Read [Contributing](CONTRIBUTING.md), [Security](SECURITY.md), and the bilingual
[documentation index](doc/README.md) before submitting changes.
## License and commercial use

BSCP-original orchestration, scripts, tests, configuration, and documentation tracked by this
repository are available under the
[PolyForm Noncommercial License 1.0.0](LICENSE). Commercial use requires prior written
authorization under a separate [commercial license](COMMERCIAL_LICENSING.md).

This commercial restriction means the BSCP-original portion is source-available, not
OSI-approved open-source software. Independent AOSP, crosvm, gfxstream, aemu, firmware, and other
third-party components retain their own licenses and notices; the BSCP license does not replace or
restrict them. Read the complete [license policy](LICENSE_POLICY.md) before use or redistribution.
