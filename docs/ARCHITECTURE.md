> Technical reference. Current AOSP17 deployment/capability boundaries are defined by [deployment](DEPLOYMENT.md) and [release](RELEASE.md). Android/Cuttlefish is outside the current Microdroid release acceptance.

# Architecture

[简体中文](ARCHITECTURE.zh-CN.md) | English

## Design goals

BSCP provides one host-facing control plane for isolated Android compute across KVM, HVF, and
WHPX. Portability is achieved at explicit interfaces; platform behavior is not hidden behind
claims of identical capability.

## Payload contract: implemented mechanisms and future composition

The Guest entry is `AVmPayload_main`; returning finishes the VM with an exit code.
Readiness notification, Binder RPC serving and access to packaged resources and
configured instance storage let a payload implement a one-shot task or service.
These mechanisms do not restrict the business logic to AI. Current finite evidence
covers CPU/GPU workloads, offscreen readback, coherent memory and persistence; it
does not prove arbitrary native-library or Android-app compatibility.

NDK/CMake, bionic and APK assets can be reused with the matching ABI/API level and
library namespace. AIDL needs matching generated bindings, runtime and transport;
it does not expose all Android system services. Framework/UI, device HAL services
and hardware codecs are not automatic payload capabilities. The BSCP graphics
Guest is an explicit integration beyond the minimal Microdroid environment.

A unified task schema for artifact identities, inputs/outputs, resource requests,
capability negotiation and state migration is a design direction, not a completed
portable SDK. x86_64 and arm64 require separate native artifacts. Android phone
native tests do not establish phone Microdroid GPU support. Cross-device scheduling,
unified quotas and adversarial multi-tenant isolation remain goals. Desktop HVF/WHPX
are not Android pKVM; pVM/attestation guarantees require a separately supported and
verified hardware, firmware and runtime configuration.

## Runtime layers

1. `vm` and `virtmgr` validate configuration, prepare payload metadata and instance storage, and
   expose lifecycle operations.
2. The Virtualization libraries translate Android VM requests into a host-neutral launch plan.
3. `crosvm` maps that plan to KVM, HVF, or WHPX and exposes only requested virtio devices.
4. Microdroid boots a minimal Android userspace and executes the payload in the guest boundary.
5. Optional gfxstream/ANGLE integration accelerates graphics. Its renderer, shared buffers and
   host driver add data-plane trust dependencies that must be assessed separately.

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
