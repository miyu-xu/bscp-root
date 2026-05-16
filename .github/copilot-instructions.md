
# Copilot Instructions

Please keep the conversation coherent. When requirements are unclear, confirm with me first instead of making assumptions. After each response, naturally ask if further adjustments or additions are needed to make communication smoother.

## Build, test, and formatting commands

### Host build entrypoints

- **Windows unified host build:** `build_all.bat` or `build_all.bat --clean`
- **Linux/macOS unified host build:** `./build_all.sh` or `./build_all.sh --clean`
- **Manual binder-rpc host build on Unix-like hosts:** `cmake -B out/build -G Ninja -DCMAKE_BUILD_TYPE=Release . && cmake --build out/build --parallel`
- **macOS arm64 APEX staging during host build:** `MACOS_AVF_APEX_TREE_SOURCE=/path/to/arm64/apex_tree ./build_all.sh`

### Rust host-only checks

- `cargo check --manifest-path packages/modules/Virtualization/libs/libvmclient/Cargo.toml`
- `cargo check --manifest-path packages/modules/Virtualization/android/vm/Cargo.toml`
- `cargo check --manifest-path packages/modules/Virtualization/android/virtmgr/Cargo.toml`
- **Windows cross-check:** `cargo check --manifest-path packages/modules/Virtualization/android/virtmgr/Cargo.toml --target x86_64-pc-windows-gnu`
- **crosvm host build:** from `external/crosvm`, `cargo build --release -p crosvm --target <triple>`

### Host-side binder tests

- **Build test targets:** `cd out/build && cmake -DBUILD_TESTING=ON ../.. && cmake --build . --target rpc_smoke_test --target binder-rpc`
- **Run a single host test:** `cd out/build && ctest -R rpc_smoke_test --output-on-failure`
- **Run symbol export check (Linux):** `bash frameworks/native/libs/binder/tests/host/check_symbols.sh out/build/lib/libbinder-rpc.so`
- **Run symbol export check (macOS):** `bash frameworks/native/libs/binder/tests/host/check_symbols.sh out/build/lib/libbinder-rpc.1.0.0.dylib`

### Host regression scripts

- **Linux smoke/full regression:** `bash scripts/run_linux_avf_regression.sh --scenario-mode smoke --step-timeout 120`
- **macOS smoke/full regression:** `bash scripts/run_macos_avf_regression.sh --scenario-mode smoke --step-timeout 120`
- **Windows regression:** `.\scripts\run_windows_avf_regression.ps1`

### AOSP / AVF build and test commands

- **Set up Android build env:** `source build/envsetup.sh; lunch aosp_<target>`
- **Build the AVF APEX:** `banchan com.android.virt aosp_arm64 && UNBUNDLED_BUILD_SDKS_FROM_SOURCE=true m apps_only dist`
- **Build a single module:** `m microdroid`
- **Run AVF test modules:** `atest MicrodroidHostTestCases` and `atest MicrodroidTestApp`
- **Run a single AVF test:** `atest MicrodroidHostTestCases:com.android.microdroid.test.MicrodroidHostTests#testMicrodroidBoots`

### Formatting

- `cargo fmt --manifest-path packages/modules/Virtualization/android/virtmgr/Cargo.toml`
- `cargo fmt --manifest-path packages/modules/Virtualization/android/vm/Cargo.toml`
- `cargo fmt --manifest-path packages/modules/Virtualization/libs/libvmclient/Cargo.toml`
- from `external/crosvm`: `cargo fmt --all`
- `clang-format -i <file>` using `frameworks/native/.clang-format` or `packages/modules/Virtualization/.clang-format`

## High-level architecture

This repo is a trimmed AOSP workspace with two coupled build paths:

- **Host runtime path:** `build_all.bat` / `build_all.sh` first builds the cross-platform `binder-rpc` shared library with CMake from `frameworks/native/libs/binder` plus `system/core/libutils` and `system/core/libcutils`, then copies that library into `frameworks/native/libs/binder/rust/sys/libs`, then builds the Rust host tools (`virtmgr`, `vm`, `libvmclient`) and `external/crosvm`, and finally stages runnable artifacts under `out/dist/<platform>`.
- **Android/Soong path:** `packages/modules/Virtualization` still contains the AVF APEX build (`com.android.virt`) and test surfaces that are built with Android tooling (`banchan`, `m`, `atest`) rather than the host scripts.

The host control plane is layered:

- `scripts/vm_<platform>` and `scripts/vm_shell_<platform>` are the normal entrypoints for local host runs.
- `vm` is the CLI entrypoint.
- `libvmclient` either connects to or spawns `virtmgr`.
- `virtmgr` exposes `IVirtualizationService` over Binder RPC and launches or controls `crosvm`.
- Guest-side communication is separate: Microdroid talks to `IVirtualMachineService` over vsock-style transport, not directly to `IVirtualizationService`.

Platform transport differences matter to debugging: Windows host Binder RPC uses named-pipe implementations under `frameworks/native/libs/binder/platform`, while Linux/macOS use the Unix socket-based host path. macOS host runs also depend on an arm64 `com.android.virt` tree under `out/dist/apex_dir` (or an explicit override) and signed Mach-O artifacts from `build_all.sh`.

## Key repository-specific conventions

- Treat `build_all.bat` / `build_all.sh` as the source of truth for host builds. They are not thin wrappers: they encode build order, the binder library copy step, crosvm feature/toolchain selection, artifact staging, and macOS signing/APEX preparation.
- Keep Rust path dependencies relative and in-tree. `virtmgr`, `vm`, and `libvmclient` intentionally point at checked-in Binder and Virtualization crates under `frameworks/native/libs/binder/rust` and `packages/modules/Virtualization/libs`.
- If you add or move Binder host sources, update `frameworks/native/libs/binder/CMakeLists.txt` source lists (`RPC_CORE_SOURCES`, `BINDER_CORE_SOURCES`, `PLATFORM_SOURCES`). Host-only test targets also live there behind `BUILD_TESTING`.
- Prefer the platform wrapper scripts over invoking `vm`, `virtmgr`, or `crosvm` directly. The wrappers set the runtime contract via `VIRTMGR_PATH`, `VIRTMGR_CROSVM_PATH`, `VIRTMGR_APEX_ROOT`, `VIRTMGR_SYSTEM_ROOT`, `VIRTMGR_SYSTEM_EXT_ROOT`, tracing paths, and per-platform library-loader variables.
- Host-side `cargo check` does not get the same cfgs that Soong injects. The `unexpected_cfgs` declarations in the Cargo manifests are intentional and should stay aligned with the real `#[cfg(...)]` usage.
- Windows host support is intentionally a **pure RPC host** port: expect named-pipe transports, HANDLE/CRT-fd bridging, and desktop-host mock providers for permission/SELinux semantics (`VIRTMGR_MOCK_PERMISSION_JSON`, `VIRTMGR_MOCK_SELINUX_JSON`) instead of Android system services.
- macOS host work is Apple Silicon/HVF-first. If the default `out/dist/apex_dir` tree is not correct, use `MACOS_AVF_APEX_TREE_SOURCE` during build or `-ApexTreeRoot` with the macOS wrapper scripts, and make sure the staged `microdroid_kernel` is arm64.
- `reorganized_backup/` is a reference snapshot of the pre-reorganization tree, not the active source tree to edit.
