# Repository layout and sync

This tree is a **trimmed AOSP layout** for host-side builds: **libbinder-rpc** (CMake) plus **virtmgr**, **vm**, and **crosvm** (Cargo). The canonical pin is tag **`android-15.0.0_r14`**.

## What you need to build

| Component | Root path | Build entry |
|-----------|-----------|-------------|
| RPC Binder (C++) | `frameworks/native/libs/binder` | Top-level `CMakeLists.txt` → `add_subdirectory(frameworks/native/libs/binder)` |
| Binder Rust / RPC | `frameworks/native/libs/binder/rust` | `path` dependency from AVF crates |
| AVF apps + libs | `packages/modules/Virtualization` | `virtmgr`, `vm`, `libs/*`, `virtualizationservice/aidl` (generated/stub crates) |
| crosvm | `external/crosvm` | `cargo build -p crosvm --features <host-hypervisor>,composite-disk,android-sparse` |

**Minimum git repos** for the current `build_all.bat` / `build_all.sh` flow: the four **Layer 1** projects in `manifest.xml` (`Virtualization`, `crosvm`, `minijail`, `frameworks/native`). Layer 2 and `hardware/interfaces` are included for AOSP parity and future steps (e.g. building AIDL from source); they are not required for the default CMake + Cargo path in this workspace if you already have the checked-in layout.

## Sync with `repo`

From an empty directory:

```bash
repo init -u https://example.com/your/manifest.git -m manifest.xml
repo sync -c -j4
```

To use this file **locally** without hosting it:

```bash
mkdir aosp && cd aosp
repo init -u file:///path/to/parent --no-clone-bundle -m manifest.xml
# or copy manifest.xml into .repo/manifests/ and point repo init there
repo sync -c -j4
```

Adjust `-u` to wherever you host the manifest repository; the `<project>` `name=` values are standard `platform/...` AOSP paths.

## Manual `git clone` (same layout)

Example (shallow):

```bash
TAG=android-15.0.0_r14
BASE=https://android.googlesource.com
clone() { git clone --depth 1 -b "$TAG" "$BASE/$1" "$2"; }

clone platform/packages/modules/Virtualization packages/modules/Virtualization
clone platform/external/crosvm external/crosvm
clone platform/external/minijail external/minijail
clone platform/frameworks/native frameworks/native
```

Add Layer 2 / `hardware/interfaces` only if you need them (see `manifest.xml`).

## Host build after sync

- **Windows**: install MinGW-w64 and CMake as in `build_all.bat`, then run `build_all.bat` from the repo root. Default `RUST_TARGET` is `x86_64-pc-windows-gnu`, and the script builds `crosvm` with the `whpx` feature enabled.
- **Linux**: install `cmake` and `ninja`, then run `chmod +x build_all.sh && ./build_all.sh`. The script picks `RUST_TARGET` from `uname -m` and preserves the Linux crosvm feature set on Linux.
- **macOS**: install `cmake` and a real `ninja` binary, keep full Xcode selected (`sudo xcode-select -s /Applications/Xcode.app/Contents/Developer`), then run `chmod +x build_all.sh && ./build_all.sh`. The script defaults to `RUST_TARGET=aarch64-apple-darwin`, resolves the active Xcode clang/SDK through `xcrun`, builds `crosvm` with `+nightly --no-default-features --features hvf,default-no-sandbox,config-file,qcow,balloon,android-sparse,composite-disk,tokio`, and ad-hoc signs the collected `out/dist/macos` binaries and libraries (with the Hypervisor entitlement applied to `crosvm`). If you already have an extracted **arm64** `com.android.virt` host APEX tree, set `MACOS_AVF_APEX_TREE_SOURCE=/path/to/apex_tree` while invoking `build_all.sh`; the build will stage it into `out/dist/apex_dir` through `scripts/prepare_host_apex_tree.sh` and validate that `microdroid_kernel` is arm64.

## Cross-check targets on Windows (no linker for foreign OS)

For type-checking only:

```bash
rustup target add aarch64-apple-darwin
rustup target add aarch64-unknown-linux-gnu
cargo check -p <crate> --target aarch64-apple-darwin
cargo check -p <crate> --target aarch64-unknown-linux-gnu
```

Crates that run **bindgen** against NDK headers (e.g. `binder-ndk-sys`) need a **target SDK and clang with those sysroots**. On Windows, `cargo check --target aarch64-apple-darwin` typically fails at bindgen with missing `sys/cdefs.h` until you point clang at an Apple SDK (or run the check on a Mac). Use cross-target checks for **mostly-Rust** crates; validate **binder + virtmgr** linking on macOS/Linux hosts.

## Path dependency map (Cargo)

High-level `path =` roots used by **virtmgr** / **vm** / **libvmclient**:

- `frameworks/native` — `binder`, `rpcbinder`, `binder-ndk-sys`
- `packages/modules/Virtualization` — all `../../libs/*`, `../virtualizationservice/aidl/*`, and app crates

No extra AOSP repo is required beyond the trees above for those paths; everything else is either in-tree or from **crates.io**.
