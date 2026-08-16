# Architecture

[简体中文](ARCHITECTURE.zh-CN.md) | English

## Design goals

BSCP provides one host-facing control plane for isolated Android compute across KVM, HVF, and
WHPX. Portability is achieved at explicit interfaces; platform behavior is not hidden behind
claims of identical capability.

## Runtime layers

1. `vm` and `virtmgr` validate configuration, prepare payload metadata and instance storage, and
   expose lifecycle operations.
2. The Virtualization libraries translate Android VM requests into a host-neutral launch plan.
3. `crosvm` maps that plan to KVM, HVF, or WHPX and exposes only requested virtio devices.
4. Microdroid boots a minimal Android userspace and executes the payload in the guest boundary.
5. Optional gfxstream/ANGLE integration accelerates graphics without changing the control-plane
   trust boundary.

## Repository ownership

The manifest pins independent repositories for crosvm, Virtualization, binder/native support,
gfxstream, aemu support, and Android core pieces. This root repository owns composition only.
Keeping one Git owner per file makes review, rollback, licensing, and release provenance explicit.

## Isolation boundary

The VM process, guest kernel, payload, virtual devices, host bridge processes, and output storage
are distinct principals. A payload must not receive host filesystem, network, graphics, audio, or
debug access unless the launch plan enables it. Host wrappers use private work and log directories
per instance; production systems should additionally apply OS-level process sandboxing.

## Portability contract

- Linux/KVM is the reference implementation.
- macOS/HVF uses arm64 guests and Apple virtualization entitlements.
- Windows/WHPX uses the GNU Windows Rust target and MinGW-compatible native libraries.
- Unsupported capabilities fail during prerequisite validation or launch-plan construction; they
  must not silently downgrade a protected workload.

The full Android compatibility flow reuses crosvm and the graphics/device stack but is separate
from the Microdroid security and release baseline.
