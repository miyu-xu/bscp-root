# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Non-Optional Conversation Protocol

## Core Execution Constraints (MUST comply)

1. At every turn: EXACTLY ONE action below
   a. Provide requested output → Invoke Ask_User
   b. If uncertain → Invoke Ask_User (NO speculative output)
2. Voluntary conversation termination: PROHIBITED
3. Terminal response generation: PROHIBITED
4. Ask_User invocation: MANDATORY per turn

## Overview

This workspace contains two primary components:

1. **Cross-platform Binder RPC library** – A CMake-based project that extracts Android Binder components from AOSP to build a cross-platform (Windows/Linux/macOS) RPC library.
2. **Android Virtualization Framework (AVF)** – Userspace components of Android's virtualization framework, built as an APEX (`com.android.virt`) using the Android build system.

The workspace also includes external dependencies:
- `external/crosvm` – ChromeOS Virtual Machine Monitor, used for running protected VMs.
- `external/minijail` – Sandboxing library.
- `external/rust/crates` – Rust crates for binder-rpc.

## Cross-platform Binder RPC Library

### Build Commands

- **Windows/Linux/macOS (recommended)**: use unified scripts at repo root: `build_all.bat` / `build_all.sh`
- **Manual CMake (optional)**: source `.` with out-of-tree build dir under `out/` (e.g. `out/build_windows` or `out/build`)

### Host unified build (binder-rpc + AVF Rust + crosvm)

- **Windows**: Run `build_all.bat` from repo root. Intermediate outputs go to `out/build_windows` and `out/target`. Final artifacts go to `out/dist/windows` (`bin`: examples + `virtmgr.exe` + `vm.exe` + `crosvm.exe`; `lib`: `libbinder-rpc.dll`). `RUST_TARGET` defaults to `x86_64-pc-windows-gnu`.
- **Unix**: `chmod +x build_all.sh && ./build_all.sh`. Intermediate outputs go to `out/build` and `out/target`. Final artifacts go to `out/dist/linux`. `RUST_TARGET` defaults to `x86_64-unknown-linux-gnu`.
- **Order**: CMake `binder-rpc` → copy `libbinder-rpc` into `frameworks/native/libs/binder/rust/sys/libs/` for linking → cargo build `virtmgr` + `vm` via explicit `--manifest-path` → cargo build `crosvm` in `external/crosvm` → copy into `out/dist/`.
- **Note**: On some Windows + MinGW combinations, `virtmgr` / `vm` may fail in the `binder` crate (e.g. const-eval around `ParcelFileDescriptor`); fix the toolchain or crate issue before the unified build can complete.

### Architecture

- **Source locations**:
  - Core Binder: `frameworks/native/libs/binder/`
  - Platform-specific code (host CMake only): `frameworks/native/libs/binder/platform/` (Windows), `frameworks/native/libs/binder/OS_non_android_linux.cpp` (Unix)
  - Support libraries: `system/core/libutils/`, `system/core/libcutils/`
  - RPC-specific files: `RpcServer.cpp`, `RpcSession.cpp`, `RpcState.cpp`, `RpcTransportRaw.cpp`
- **Public headers**: `frameworks/native/libs/binder/include/`, `frameworks/native/libs/binder/include_rpc_unstable/`, `frameworks/native/libs/binder/platform/` (Windows stubs)
- **Platform detection**: CMake defines `PLATFORM_WINDOWS`, `PLATFORM_LINUX`, `PLATFORM_MACOS`. Windows code uses named pipes for transport; Unix uses sockets.

### Development Notes

- When adding new Binder source files, update `frameworks/native/libs/binder/CMakeLists.txt` (`RPC_CORE_SOURCES`, `BINDER_CORE_SOURCES`, `PLATFORM_SOURCES`).
- Windows platform code must implement missing POSIX/Android APIs (see `frameworks/native/libs/binder/platform/windows_stubs.cpp`).
- The library aims to be compatible with Android's Binder interface; changes should be tested against Android's `binderRpcTest`.

## Android Virtualization Framework (AVF)

### Build Commands (requires full AOSP environment)

- **Set up build environment**: `source build/envsetup.sh; lunch aosp_<target>`
- **Build AVF APEX**:
  ```sh
  banchan com.android.virt aosp_arm64   # or aosp_x86_64 for Cuttlefish
  UNBUNDLED_BUILD_SDKS_FROM_SOURCE=true m apps_only dist
  ```
- **Install to device**: `adb install out/dist/com.android.virt.apex; adb reboot`
- **Build individual modules**: `m <module_name>` (e.g., `m microdroid`)

### Test Commands

- **Run Microdroid VM**: `packages/modules/Virtualization/vm/vm_shell.sh start-microdroid --auto-connect -- --protected`
- **Run AVF tests**: `atest MicrodroidHostTestCases`, `atest MicrodroidTestApp`
- **Debug logs**: Check `host_log_*.zip` produced by `atest`.

### Architecture

- **APEX module**: `com.android.virt` – contains virtualization services, tools, and crosvm.
- **Key directories**:
  - `packages/modules/Virtualization/` – Userspace components.
  - `packages/modules/Virtualization/build/microdroid/` – Microdroid build system.
  - `packages/modules/Virtualization/guest/` – Guest components (pVM firmware, kernel, encrypted storage).
  - `packages/modules/Virtualization/libs/` – Libraries (framework-virtualization, libvm_payload, libvmbase).
- **Virtualization stack**: Uses crosvm (in `external/crosvm`) as the VMM, with Minijail for sandboxing.
- **Guest types**: Protected VM (pVM) for strong isolation, non-protected VM for Cuttlefish.

### Development Notes

- AVF is built as an APEX; changes require rebuilding the APEX and installing to device.
- The `vm_shell.sh` script is the primary tool for launching VMs; it sets up crosvm with appropriate parameters.
- For custom VM payloads, see `libs/libvm_payload` and `guest/`.

### virtmgr (Windows host)

- **Location**: `packages/modules/Virtualization/android/virtmgr/` (Rust; cross-compile with e.g. `cargo check --target x86_64-pc-windows-gnu`).
- **`VIRTMGR_CROSVM_PATH`**: Optional. Sets the full path to **`crosvm.exe`** when `virtmgr` on Windows launches or controls the VMM. If unset, the code looks for **`crosvm.exe` on `PATH`** (same variable is used by the Windows `vm_control` shim for `suspend` / `resume` / `balloon` CLI delegation).
- **Details**: `packages/modules/Virtualization/android/virtmgr/HOST_WINDOWS_PORT.md` (architecture, env vars, limitations). Parameter parity: `android/virtmgr/WINDOWS_PARITY_MATRIX.md`.

## External Dependencies

- **crosvm**: Prebuilt binaries are included in the APEX; source is in `external/crosvm`. Building crosvm separately requires Rust toolchain and ChromeOS build environment.
- **minijail**: Source in `external/minijail`. Used for sandboxing crosvm processes.
- **Rust binder-rpc**: `external/rust/crates/binder-rust` – Rust bindings for Binder RPC.

## Workspace Structure

```
.
├── CMakeLists.txt              # Top-level CMake (delegates to frameworks/native/libs/binder)
├── build_all.bat               # Windows: unified host build
├── build_all.sh                # Unix: unified host build
├── out/                        # Intermediate + final host outputs
│   ├── build_windows/          # Windows CMake output
│   ├── build/                  # Unix CMake output
│   ├── target/                 # Cargo target dir for host builds
│   └── dist/                   # Collected host binaries/libraries
├── frameworks/native/libs/binder/
│   ├── CMakeLists.txt          # binder-rpc targets, examples, install rules
│   ├── platform/               # Windows host implementations (named pipes, stubs)
├── system/core/libutils/       # Utilities
├── packages/modules/Virtualization/ # AVF userspace components
├── external/crosvm/            # ChromeOS VMM
└── external/minijail/          # Sandboxing library
```

## Common Tasks

- **Adding a new Binder RPC test**: Add source under `frameworks/native/libs/binder/tests/` and update `frameworks/native/libs/binder/CMakeLists.txt`.
- **Modifying Windows transport**: See `frameworks/native/libs/binder/platform/namedpipe_rpc_transport.cpp` and `frameworks/native/libs/binder/platform/namedpipe_vsock.cpp`.
- **Debugging AVF**: Use `adb logcat` and check crosvm logs in `/data/misc/avf/`.
- **Running a custom VM payload**: Implement `VmPayload` interface and use `vm_shell.sh start --payload-binary`.

## Code Style

- **C/C++**: Follow Android's coding style (clang-format). Root `.clang-format` files exist in `frameworks/native/` and `packages/modules/Virtualization/`.
- **Rust**: Use standard Rust formatting (`cargo fmt`).
- **Java**: Follow Android Java style (AOSP guidelines).

## Notes

- The cross-platform Binder library is independent of the Android build system; it compiles selected AOSP source files directly.
- AVF device/APEX components rely on the Android build system (Soong). Host tools (`virtmgr`, `vm`, etc.) can be built with Cargo via `build_all.*` without Soong, subject to crate support for your target.
- Windows development requires MinGW-w64 (for GCC) or Visual Studio (CMake generator "Visual Studio 16 2019").
- The `reorganized_backup/` directory contains a backup of the original directory structure before reorganization.