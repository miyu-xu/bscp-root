# Deployment

[简体中文](DEPLOYMENT.zh-CN.md) | English

## Checkout

Use the manifest repository and its pinned branch. A successful `repo sync` creates this root plus
the independent component repositories. Do not clone component directories over an existing
workspace or commit generated `.repo/` metadata.

## Host prerequisites

- Common: Git, Python 3, CMake, Ninja, Rust through rustup, and an AOSP-compatible `repo` launcher.
- Linux: KVM access, a supported C/C++ toolchain, and permission to create networking devices when
  network tests are enabled.
- macOS: Apple Silicon, Xcode command-line tools, Hypervisor.framework entitlement, and an arm64
  `com.android.virt` APEX tree.
- Windows: WHPX enabled, PowerShell 5.1 or newer, CMake, Ninja, MinGW-w64, the pinned GNU-hosted
  Rust toolchain, and `libclang.dll` discoverable through `LIBCLANG_PATH` or a supported install.

## Build

```bash
./build_all.sh            # incremental Linux/macOS build
./build_all.sh --clean    # clean output directories first
```

```powershell
.\build_all.bat
.\build_all.bat --clean
```

The build assembles `virtmgr`, `vm`, `crosvm`, binder RPC libraries, and optional graphics runtime
artifacts under `out/dist/<platform>/`. Source directories remain immutable; generated state stays
under `out/`.

## Microdroid guest assets

Linux and Windows normally consume the matching `com.android.virt` APEX tree from the synchronized
Android build. macOS must receive an arm64 tree explicitly:

```bash
MACOS_AVF_APEX_TREE_SOURCE=/absolute/path/to/apex_tree ./build_all.sh
```

Validate before launch with `vm_linux.sh`, `vm_macos.sh`, or `vm_windows.ps1` and the
`validate-prereqs` command.

## Production checklist

- Pin every manifest revision to a reviewed commit for a release tag.
- Disable debug policy, shell, unrestricted networking, and host-path passthrough unless required.
- Use per-instance writable storage and protect host keys outside the workspace.
- Verify artifact hashes and preserve third-party notices with the release.
- Run the platform regression wrapper and archive its logs before promotion.
- Review [Security](SECURITY.md); a development launch is not automatically production-safe.
