# Security model

[简体中文](SECURITY.zh-CN.md) | English

## Protected assets and trust

The primary assets are host credentials and files, payload confidentiality and integrity, guest
instance state, inter-VM separation, and release provenance. The host OS, hypervisor API, selected
crosvm build, guest kernel, boot chain, and payload verifier are trusted. Payloads and their input
may be untrusted. Optional device endpoints expand the attack surface and must be enabled
deliberately.

## Controls

- Hardware-backed CPU and memory virtualization separates guest execution from the host process.
- Microdroid minimizes guest services and verifies payload metadata before execution.
- Launch plans enumerate devices and host bridges; absent capabilities are denied by construction.
- Per-instance work directories avoid writable disk sharing between workloads.
- Component revisions are pinned by the manifest and release artifacts are expected to be hashed.

## Limitations and hardening

This repository alone does not certify a deployment, provide a hardware root of trust on every
host, or make debug and unencrypted development profiles suitable for production. Fail closed when
requested protection is unavailable. Remove debug policy and guest shell access, restrict network
egress, run bridge processes with least privilege, use immutable base images plus private overlays,
and keep signing material outside build machines. A full Android guest has a larger attack surface
than the Microdroid baseline.

Report vulnerabilities privately under the repository-level [security policy](../SECURITY.md).
