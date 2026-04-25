
# Copilot Instructions

Please keep the conversation coherent. When requirements are unclear, confirm with me first instead of making assumptions. After each response, naturally ask if further adjustments or additions are needed to make communication smoother.

## Build, test, and formatting commands

### Host-side unified build

- **Windows:** run `build_all.bat` from the repo root. This builds `binder-rpc` with CMake/MinGW, copies the host library into `frameworks/native/libs/binder/rust/sys/libs`, then builds `virtmgr`, `vm`, and `crosvm`, and collects artifacts under `out\dist\windows`.
- **Linux/macOS:** run `./build_all.sh` from the repo root. It performs the same sequence and writes artifacts under `out/dist/linux` or `out/dist/macos`.
- **Manual binder-rpc build on Unix-like hosts:** `cmake -B out/build -G Ninja -DCMAKE_BUILD_TYPE=Release . && cmake --build out/build --parallel`

### Rust host-only checks

- `cargo check --manifest-path packages/modules/Virtualization/libs/libvmclient/Cargo.toml`
- `cargo check --manifest-path packages/modules/Virtualization/android/vm/Cargo.toml`
- `cargo check --manifest-path packages/modules/Virtualization/android/virtmgr/Cargo.toml`
- On Windows host work, `cargo check --manifest-path packages/modules/Virtualization/android/virtmgr/Cargo.toml --target x86_64-pc-windows-gnu` is the documented cross-check path.
- `cargo build --release -p crosvm --target <triple>` from `external/crosvm`

### Tests

- **AVF test modules:** `atest MicrodroidHostTestCases` and `atest MicrodroidTestApp`
- **Single AVF test example:** `atest MicrodroidHostTestCases:com.android.microdroid.test.MicrodroidHostTests#testMicrodroidBoots`

### Formatting

- Rust formatting is per crate. Examples:
  - `cargo fmt --manifest-path packages/modules/Virtualization/android/virtmgr/Cargo.toml`
  - `cargo fmt --manifest-path packages/modules/Virtualization/android/vm/Cargo.toml`
  - `cargo fmt --manifest-path packages/modules/Virtualization/libs/libvmclient/Cargo.toml`
  - from `external/crosvm`: `cargo fmt --all`
- C/C++ formatting uses `clang-format -i <file>` with style taken from `frameworks/native/.clang-format` or `packages/modules/Virtualization/.clang-format`, depending on the subtree.

## High-level architecture

This repository is a trimmed AOSP workspace pinned around `android-15.0.0_r14`. The main host-side flow is a CMake build that produces the shared `binder-rpc` library from `frameworks/native/libs/binder` plus support code from `system/core/libutils` and `system/core/libcutils`. The host build then copies that shared library into `frameworks/native/libs/binder/rust/sys/libs` so the Rust `binder` and `rpcbinder` crates can link while building `packages/modules/Virtualization/android/virtmgr`, `packages/modules/Virtualization/android/vm`, and `external/crosvm`.

On the host control plane, `vm` is the CLI entrypoint, `libvmclient` connects to or spawns `virtmgr`, `virtmgr` exposes `IVirtualizationService` over Binder RPC, and `virtmgr` then launches or controls `crosvm`. Microdroid guest communication is a separate path: the guest connects to `IVirtualMachineService` over vsock-style transport rather than calling `IVirtualizationService` directly.

## Key repository-specific conventions

- Treat `build_all.bat` / `build_all.sh` as the source of truth for host builds. They are not simple wrappers; they encode the required build order and the binder library copy step that Rust linking depends on.
- Keep Rust path dependencies relative and in-tree. `virtmgr`, `vm`, and `libvmclient` intentionally depend on checked-in crates under `frameworks/native/libs/binder/rust` and `packages/modules/Virtualization/libs`.
- If you add or move Binder host sources, update `frameworks/native/libs/binder/CMakeLists.txt` source lists (`RPC_CORE_SOURCES`, `BINDER_CORE_SOURCES`, `PLATFORM_SOURCES`) so the host library still builds.
- Host-side `cargo check` omits some cfgs that upstream Android builds inject. The `unexpected_cfgs` declarations in Cargo manifests are intentional and should stay aligned with the real `#[cfg(...)]` usage.
- Windows host support is deliberately a **pure RPC host** port. Expect named-pipe transports in `frameworks/native/libs/binder/platform`, CRT-fd/HANDLE bridging in Rust host code, and environment-variable overrides such as `VIRTMGR_CROSVM_PATH`, `VIRTMGR_PATH`, and `VIRTMGR_RPC_CLIENT_FD`.
- `reorganized_backup/` is a reference snapshot of the pre-reorganization tree, not the active source tree to edit.
