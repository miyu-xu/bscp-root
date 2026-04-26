# Platform Differences

> Date: 2026-04-25
> This document catalogs all known behavioral differences between Linux (KVM), macOS (HVF), and Windows (WHPX) host runtimes.

## 0. DesktopHost Trait Abstraction (Phase 2)

All platform-specific operations are abstracted through the `DesktopHost` trait (defined in
`libs/desktop_host/src/traits.rs`). Each platform provides a concrete implementation:

| Aspect | Linux | macOS | Windows |
|--------|-------|-------|---------|
| **Type** | `LinuxDesktopHost` | `MacOSDesktopHost` | `WindowsDesktopHost` |
| **Permission** | `MockPermissionProvider` | `MockPermissionProvider` | `MockPermissionProvider` |
| **SELinux** | `MockSelinuxProvider` | `MockSelinuxProvider` | `MockSelinuxProvider` |
| **Vsock transport** | Real AF_VSOCK (`vsock` crate) | UDS (`/tmp/binder_rpc_vsock_*`) | Named pipe (`\\.\pipe\binder_rpc_vsock_*`) |

### Mock Providers (shared across all platforms)

Both `MockPermissionProvider` and `MockSelinuxProvider` support three modes:

| Mode | Env Var | Behavior |
|------|---------|----------|
| **bypass** (default) | (no env or absent allowlist) | Warn once per permission on first call; all checks pass |
| **allowlist** | `VIRTMGR_MOCK_PERMISSION_JSON` / `VIRTMGR_MOCK_SELINUX_JSON` | Check against allowlist; reject unknown entries |
| **strict** | `VIRTMGR_STRICT_PARITY=1` | Reject all; no silent bypass |

Backward-compatible CSV env vars (`VIRTMGR_MOCK_PERMISSION_ALLOWLIST`, etc.) are also supported.

### Bridge Module (shared TCP→vsock forwarding)

The `bridge.rs` module in virtmgr provides two shared utilities:

- `bridge_connection(tcp, vsock_file)`: Bidirectional io::copy in two threads (replaces duplicated platform code).
- `start_bridge(port, cid, gport, on_accept)`: Nonblocking TCP listener loop with `BridgeHandle::stop()`.

Both `crosvm_unix.rs` and `crosvm_windows.rs` now delegate to these shared functions instead of maintaining duplicate copy-thread logic.

## 1. Hypervisor

| Aspect | Linux (KVM) | macOS (HVF) | Windows (WHPX) |
|--------|-------------|-------------|----------------|
| Type | Kernel-based (in-kernel VMM) | Userspace (Hypervisor.framework) | Userspace (Microsoft Hypervisor Platform API) |
| AArch64 support | Yes | Yes (Apple Silicon only) | No (x86_64 only) |
| x86_64 support | Yes | No | Yes |
| Protected VM | pKVM available (not wired) | Not supported | Not supported |
| IRQ Chip modes | Split / Kernel / Userspace | Kernel only (GICv3) | Kernel only |
| Dirty page tracking | Yes | No | Partial |
| ioeventfd | Yes | Partial (no irqfd) | Partial |
| Guest debug | Yes | No | No |
| Snapshot/restore | Yes | No | No |
| PMU virtualization | Yes | No | No |

## 2. Binder RPC Transport

| Aspect | Linux | macOS | Windows |
|--------|-------|-------|---------|
| Transport type | Unix domain socket | Unix domain socket | Named pipe |
| Bootstrap | Unix domain socket | Unix domain socket | Named pipe |
| Connection model | fd passing | fd passing | Pipe handle |

## 3. VSOCK

| Aspect | Linux | macOS | Windows |
|--------|-------|-------|---------|
| Transport | Real AF_VSOCK | UDS files (`/tmp/binder_rpc_vsock_{cid}_{port}.sock`) | Named pipes (`\\.\pipe\binder_rpc_vsock_{cid}_{port}`) |
| CID allocation | Real CID | Simulated (trace log) | Simulated |
| ACL model | SELinux | no-op | no-op |
| **Unified interface** | `VsockConnector` trait → `vsock::VsockStream::connect_with_cid_port()` | `VsockConnector` trait → `UnixStream::connect()` | `VsockConnector` trait → `CreateFileW()` |

## 4. Network

| Aspect | Linux | macOS | Windows |
|--------|-------|-------|---------|
| Backend | TAP (kernel) | vmnet.framework (Phase 1) | TAP (Windows) |
| NAT mode | iptables | vmnet shared | WinNAT |
| Status | Full | L0→L1 (Phase 1) | Partial |

## 5. ACPI Events

| Aspect | Linux | macOS | Windows |
|--------|-------|-------|---------|
| Source | Netlink acpi_event | None (PSCI-based) | Windows API |
| Guest shutdown | Host→Guest via ACPI | Guest→Host via PSCI | Host→Guest via ACPI |
| Status | Full | L0 (guest PSCI works) | Full |

## 6. Sandbox (Minijail)

| Aspect | Linux | macOS | Windows |
|--------|-------|-------|---------|
| Implementation | Real minijail | All stubs | All stubs |
| Process isolation | Yes | No | No |
| Status | Full | L0 | L0 |

## 7. Console

| Aspect | Linux | macOS | Windows |
|--------|-------|-------|---------|
| Type | PTY/TTY interactive | PTY/TTY interactive | File-backed read-only |
| Interaction | Full interactive | Full interactive | Read-only |

## 8. VM Control

| Aspect | Linux | macOS | Windows |
|--------|-------|-------|---------|
| Suspend/Resume | crosvm control socket | crosvm control socket | crosvm CLI subprocess |
| Balloon | crosvm control socket | crosvm control socket | crosvm CLI subprocess |
| Control mechanism | Unix socket | Unix socket | Named pipe + CLI |

## 9. ADB Bridge

| Aspect | Linux | macOS | Windows |
|--------|-------|-------|---------|
| Guest connection | Real AF_VSOCK | UDS simulation | Named pipe simulation |
| TCP bridge | vsock→tcp | UDS→tcp | Pipe→tcp |
| **Bridge core** (shared) | `bridge_connection()` + `start_bridge()` in `bridge.rs` | Same | Same |

## 10. Path Mapping

| Aspect | Linux | macOS | Windows |
|--------|-------|-------|---------|
| APEX tree | `/apex/...` | Same (`/apex/...` in tree) | Same |
| System root | `/system` | Same | Same |
| Path translation | None needed | None needed | Forward-slash mapping |

## 11. Build

| Aspect | Linux | macOS | Windows |
|--------|-------|-------|---------|
| Script | `build_all.sh` | `build_all.sh` | `build_all.bat` |
| Cargo target | x86_64/aarch64-unknown-linux-gnu | aarch64-apple-darwin | x86_64-pc-windows-gnu |
| CMake generator | Unix Makefiles / Ninja | Ninja | Visual Studio / Ninja |
| crosvm features | default-no-sandbox,... | hvf,default-no-sandbox,... | whpx,... |
| Rust toolchain | stable | nightly | stable/nightly |
| Codesigning | None | ad-hoc + HVF entitlement | None (Authenticode optional) |
