# Cross-platform `gfxstream + ANGLE(Vulkan)` integration

This document records the implemented cross-platform plan and the exact code changes made to wire
`gfxstream + ANGLE(Vulkan backend)` across Linux, Windows, and macOS, with special focus on a
uniform guest-visible `HOST_VISIBLE | HOST_COHERENT` memory contract.

It also captures the final architecture decisions, the reasons certain alternatives were rejected,
the external dependency layout, the Windows bring-up work needed to close the host build chain, and
the current validation status.

## 1. Goals

The target was not a single-platform workaround. The required end state was:

1. Support a single logical `angle=true` GPU mode at the `crosvm` configuration layer.
2. Keep **guest Vulkan** on the native `gfxstream` Vulkan decode path.
3. Keep **guest EGL/GLES** on the existing `gfxstream` GL/EGL decoder path, but make the host GL
   provider be **ANGLE using its Vulkan backend**.
4. Make the host build chain consume external source trees:
   - `..\angle`
   - `..\aemu`
   - `..\flatbuffers`
   - `..\MoltenVK` on macOS
5. Expose a consistent guest-visible `HOST_VISIBLE | HOST_COHERENT` memory contract across all
   three host platforms.

## 2. Final architecture

The implemented architecture is intentionally split by guest API surface:

- **Guest Vulkan API**
  - guest Vulkan command stream
  - `virtio-gpu`
  - `rutabaga_gfx`
  - `gfxstream` Vulkan decoder / host Vulkan path
  - host Vulkan driver

- **Guest EGL/GLES API**
  - guest EGL / GLES command stream
  - `virtio-gpu`
  - `rutabaga_gfx`
  - `gfxstream` GL / EGL host decoder path
  - host `libEGL` / `libGLESv2`
  - **ANGLE Vulkan backend**
  - host Vulkan driver

On macOS, the last two layers become:

- host `libEGL` / `libGLESv2`
- ANGLE Vulkan backend
- Vulkan loader
- **MoltenVK**
- Metal

This means the solution is explicitly **not** `GuestVulkanOnly`.

## 3. Key decisions

### 3.1 `GuestVulkanOnly` was not used

`GuestVulkanOnly` disables the host GL emulation initialization path. That would break the required
guest EGL/GLES flow and reduce the solution to a Vulkan-only path, which was outside the requested
scope.

### 3.2 `ExternalBlob` stays disabled for `angle=true`

`ExternalBlob` was deliberately not used for the `angle=true` path.

Reasons:

1. In `gfxstream`, the `ExternalBlob` path is selected before the
   `VulkanAllocateHostMemory` path, which would bypass the host-memory path used for the coherent
   guest-visible contract.
2. macOS + MoltenVK already does not support `ExternalBlob` as the common cross-platform path.
3. The requirement was coherent guest-visible semantics first, not external-handle export semantics.

So the enforced policy is:

- `ExternalBlob:disabled`
- `VulkanAllocateHostMemory:enabled`

### 3.3 Guest-visible coherent memory is treated as a contract layer

The project requirement was not "only expose coherent if the host driver reports it naturally".
The requirement was to give the guest a stable coherent contract across platforms.

Accordingly, the implementation makes the guest-visible memory type contract an emulated policy at
the `gfxstream` physical-device-memory layer:

- if `VulkanAllocateHostMemory` is active and the guest-visible type is `HOST_VISIBLE`,
- the guest-visible type is exposed as `HOST_VISIBLE | HOST_COHERENT`.

This is an intentional guest ABI policy and not a claim that every host Vulkan driver reports the
same native memory flags.

## 4. Configuration and feature flow

The `angle` mode now flows through the stack as:

`crosvm cmdline/config -> GpuParameters -> devices::virtio::gpu -> rutabaga builder -> gfxstream features`

For `angle=true`, the policy enforced in `crosvm` is:

- `backend=gfxstream`
- `egl=true`
- `gles=true`
- `glx=false`
- `vulkan=true`
- `external-blob=false`

And the renderer feature injection is:

- `AngleIndirect:enabled`
- `GuestVulkanOnly:disabled`
- `ExternalBlob:disabled`
- `VulkanAllocateHostMemory:enabled`

## 5. Detailed code changes

### 5.1 `external/crosvm`

#### `devices/src/virtio/gpu/parameters.rs`

- Added `angle: bool` to `GpuParameters`.
- This makes `angle` a real configuration field instead of only existing as documentation text.

#### `src/crosvm/gpu_config.rs`

- Added validation/fixup logic for `angle=true`.
- Enforced the required backend shape:
  - `gfxstream`
  - EGL/GLES enabled
  - Vulkan enabled
  - `external-blob=false`
- Prevented incompatible option combinations from silently drifting into an unsupported runtime.

#### `devices/src/virtio/gpu/mod.rs`

- Added the `angle=true` renderer feature synthesis.
- Injects:
  - `AngleIndirect`
  - `VulkanAllocateHostMemory`
- Explicitly disables:
  - `GuestVulkanOnly`
  - `ExternalBlob`

This file is the main policy choke point that converts user GPU config into the
`rutabaga_gfx/gfxstream` renderer feature set.

#### `src/crosvm/cmdline.rs`

- Updated command-line documentation to reflect the actual `angle` semantics and constraints.

#### `src/crosvm/sys/windows/cmdline.rs`

- Updated the Windows command implementation to stay compatible with the current `argh_shared`
  `CommandInfo` layout by adding the `short` field.

#### `vm_control/src/lib.rs`

- Fixed the Windows GPU build break in descriptor conversion.
- Simplified `RutabagaDescriptor` construction to use `from_raw_descriptor`.
- This removed a Windows-specific blocker for `cargo check/build --features gpu`.

### 5.2 `hardware/google/gfxstream`

#### `host/features/include/gfxstream/host/Features.h`

- Added the `AngleIndirect` feature declaration.

This gives `crosvm` a stable named feature to request the ANGLE-backed host GL/EGL path.

#### `host/vulkan/VkEmulatedPhysicalDeviceMemory.cpp`

- Implemented the guest-visible coherent memory policy.
- When `VulkanAllocateHostMemory` is enabled, guest-visible `HOST_VISIBLE` memory types are exposed
  to the guest as `HOST_VISIBLE | HOST_COHERENT`.

This is the central implementation for the requested cross-platform coherent contract.

#### `host/vulkan/VkEmulatedPhysicalDeviceMemoryTests.cpp`

- Added regression coverage for the coherent guest-memory exposure behavior.

#### `third-party/CMakeLists.txt`

- Removed hardcoded assumptions that dependencies only exist in AOSP-style locations.
- Added external path support for:
  - `ANGLE_PATH` / `ANGLE_ROOT`
  - `AEMU_COMMON_PATH`
  - `FLATBUFFERS_PATH`
- Added fallback handling for platforms where `libdrm` is not the primary path.

This is what allowed the build to consume:

- `..\angle`
- `..\aemu`
- `..\flatbuffers`

instead of requiring `external/angle`, `hardware/google/aemu`, and `external/flatbuffers`.

#### `CMakeLists.txt`

- Fixed Windows/MinGW compiler-flag handling so `/Zi` is only used under MSVC.

#### `host/compressedTextureFormats/AstcCpuDecompressor.h`

- Added `<cstdint>` to fix MinGW compilation.

#### `host/gl/glestranslator/EGL/EglOsApi_wgl.cpp`

- Added `<vector>` to fix MinGW compilation in the Windows host GL path.

#### `host/BorrowedImage.h`

- Added `<cstdint>` to fix MinGW compilation.

### 5.3 `hardware/google/aemu`

These fixes were required so the Windows `gfxstream_backend` build could consume the now-present
`aemu` checkout correctly under MinGW.

#### `CMakeLists.txt`

- Limited `/Zi` to MSVC builds.

#### `base/CMakeLists.txt`

- Stopped forcing `msvc.cpp` into non-MSVC builds.

#### `build-config/gfxstream/CMakeLists.txt`

- Limited `msvc.cpp` inclusion to real MSVC builds in the active `gfxstream` build config.

#### `base/System.cpp`

- Removed Unix-only header usage from the MinGW Windows path.

### 5.4 Root build scripts

#### `build_all.bat`

- Added support for:
  - `ANGLE_ROOT`
  - `AEMU_COMMON_PATH`
  - `FLATBUFFERS_PATH`
  - `GFXSTREAM_PATH`
  - `ANGLE_RUNTIME_DIR`
  - stable Rust toolchain invocation through `rustup`
- Added support for using a prebuilt `gfxstream_backend` directory.
- Stages:
  - `virtmgr.exe`
  - `vm.exe`
  - `crosvm.exe`
  - `libbinder-rpc.dll`
  - `libgfxstream_backend.dll`
  - ANGLE runtime DLLs
  - Windows runtime DLLs needed by the built toolchain path
- Generates `bin/crosvm-angle.bat`.
- Updated staging so `libbinder-rpc.dll` exists in both:
  - `dist\lib`
  - `dist\bin`

This makes the Windows dist tree self-contained enough to run the staged tools directly.

#### `build_all.sh`

- Added external dependency support for:
  - `ANGLE_ROOT`
  - `AEMU_COMMON_PATH`
  - `FLATBUFFERS_PATH`
  - `GFXSTREAM_PATH`
  - `MOLTENVK_ROOT`
  - `MOLTENVK_RUNTIME_DIR`
- Added ANGLE runtime staging.
- Added macOS `MoltenVK` runtime staging:
  - `libMoltenVK.dylib`
  - `MoltenVK_icd.json` when present
- Generates a Unix `crosvm-angle` wrapper that sets:
  - ANGLE runtime search path
  - `VK_ICD_FILENAMES` for MoltenVK when present

### 5.5 Root Binder host build fixes

The Windows host chain could not close until the root `binder-rpc` CMake build linked on MinGW.
The following fixes were required:

#### `frameworks/native/libs/binder/platform/unix_socket_compat.h`

- Added Windows-specific socket flag handling.
- Fixed host socket setup behavior that previously assumed POSIX `fcntl` semantics.
- Fixed `socketpair()` handling for Windows socket handle types.

#### `frameworks/native/libs/binder/binder_module.h`

- Removed temporary Windows-conflicting freeze notification definitions that collided with the
  win32 binder shim definitions.

#### `frameworks/native/libs/binder/Binder.cpp`

- Removed the Windows-inapplicable `BBinder::setMinSchedulerPolicy` implementation branch.

#### `frameworks/native/libs/binder/platform/OS_windows.cpp`

- Added a Windows host `__android_log_print` implementation.
- Added a Windows host `systemTime()` implementation used by the libutils thread path.

#### `frameworks/native/libs/binder/CMakeLists.txt`

- Removed duplicate Windows thread/mutex/condition source inclusion that conflicted with
  `system/core/libutils/Threads.cpp`.

These changes were what moved the root host binder build from compile/link failure to a produced
`libbinder-rpc.dll`.

### 5.6 Virtualization host-side Windows compatibility fixes outside `crosvm`

#### `packages/modules/Virtualization/libs/desktop_host/src/windows.rs`

- Fixed Win32 constant usage for `GENERIC_READ` / `GENERIC_WRITE`.
- Fixed `OwnedHandle::from_raw_handle` raw-handle typing.

#### `packages/modules/Virtualization/android/virtmgr/src/crosvm/crosvm_windows.rs`

- Fixed Win32 API imports and features for `windows-sys 0.48`.
- Moved `PIPE_ACCESS_INBOUND` / `PIPE_ACCESS_OUTBOUND` to the correct module.
- Added `Win32_System_IO` dependency feature.
- Fixed pointer type passed to `ReadFile`.
- Added missing `warn` macro import.

#### `packages/modules/Virtualization/android/virtmgr/Cargo.toml`

- Enabled the missing `windows-sys` feature set needed by the current Win32 API usage.

These changes were needed for the Windows `virtmgr` release build to complete.

## 6. External dependency layout

The host build now supports the following layout:

- `..\angle`
- `..\aemu`
- `..\flatbuffers`
- `..\MoltenVK` on macOS

Important environment variables:

| Variable | Purpose |
| --- | --- |
| `ENABLE_GFXSTREAM_ANGLE` | Enables the `gfxstream + ANGLE` build path in the host scripts |
| `ANGLE_ROOT` | Path to the ANGLE source tree |
| `ANGLE_RUNTIME_DIR` | Directory containing built `libEGL` / `libGLESv2` runtime binaries |
| `AEMU_COMMON_PATH` | Path to the `aemu` checkout consumed by `gfxstream` |
| `FLATBUFFERS_PATH` | Path to the `flatbuffers` checkout |
| `GFXSTREAM_PATH` | Path to a prebuilt `gfxstream_backend` directory |
| `MOLTENVK_ROOT` | Path to the `MoltenVK` source/package tree |
| `MOLTENVK_RUNTIME_DIR` | Path to a built `libMoltenVK.dylib` directory |

## 7. Coherent memory strategy in detail

The coherent-memory plan is intentionally layered.

### 7.1 Guest-visible capability layer

Implemented in:

- `hardware/google/gfxstream/host/vulkan/VkEmulatedPhysicalDeviceMemory.cpp`

Responsibility:

- decide what Vulkan memory flags the guest sees
- present a stable guest contract independent of host-driver variability

Policy:

- if a host-visible path is chosen for guest-accessible Vulkan memory, expose it to the guest as
  coherent as well

### 7.2 Host allocation strategy layer

Controlled by feature policy:

- `VulkanAllocateHostMemory:enabled`
- `ExternalBlob:disabled`

Responsibility:

- force the host-memory-backed Vulkan allocation path to be the primary path for the ANGLE-enabled
  mode
- avoid feature precedence that would bypass the guest-coherent contract

### 7.3 ANGLE usage layer

Relevant to:

- ANGLE Vulkan backend memory selection
- coherent-preferred mapping
- explicit flush/invalidate fallback when the underlying host requires it

Design intent:

- the guest contract remains coherent
- any host non-coherent maintenance is hidden behind the host implementation boundary instead of
  being leaked as a guest-visible capability downgrade

## 8. Windows build closure work

The Windows path needed substantially more than the original `angle=true` feature work.

### 8.1 `gfxstream_backend` closure

The Windows `gfxstream_backend` build required:

1. a valid `aemu` checkout
2. a valid `flatbuffers` checkout
3. MinGW portability fixes in both `gfxstream` and `aemu`

Final produced backend:

- `out\gfxstream_build_windows\libgfxstream_backend.dll`

### 8.2 ANGLE runtime closure

The Windows ANGLE runtime was built successfully using the clang toolchain path.

Produced runtime:

- `..\angle\out\Release-GfxAngle-Clang\libEGL.dll`
- `..\angle\out\Release-GfxAngle-Clang\libGLESv2.dll`

### 8.3 Binder RPC closure

After the compatibility fixes listed above, the root host binder build now produces:

- `out\build_windows\bin\libbinder-rpc.dll`
- `out\build_windows\lib\libbinder-rpc.dll.a`

### 8.4 Rust host tools closure

Windows release builds completed for:

- `virtmgr.exe`
- `vm.exe`
- `crosvm.exe`

### 8.5 Final staged Windows dist tree

The final Windows dist tree currently contains:

- `out\dist\windows\bin\virtmgr.exe`
- `out\dist\windows\bin\vm.exe`
- `out\dist\windows\bin\crosvm.exe`
- `out\dist\windows\bin\crosvm-angle.bat`
- `out\dist\windows\bin\libbinder-rpc.dll`
- `out\dist\windows\bin\libgfxstream_backend.dll`
- `out\dist\windows\bin\libslirp-0.dll`
- `out\dist\windows\bin\r8Brain.dll`
- `out\dist\windows\bin\ucrtbased.dll` when present
- `out\dist\windows\bin\vcruntime140d.dll` when present
- `out\dist\windows\gfx\angle\libEGL.dll`
- `out\dist\windows\gfx\angle\libGLESv2.dll`
- `out\dist\windows\lib\libbinder-rpc.dll`

## 9. Validation status

### 9.1 Completed

- `crosvm` `angle=true` configuration path implemented
- `gfxstream` `AngleIndirect` feature introduced
- guest-visible coherent memory policy implemented and test added
- external dependency path support added for ANGLE / `aemu` / `flatbuffers`
- Windows `gfxstream_backend` build completed
- Windows ANGLE runtime build completed
- Windows `binder-rpc` build completed
- Windows `virtmgr` / `vm` / `crosvm` release builds completed
- Windows dist staging completed
- staged `virtmgr.exe` and `vm.exe` help-path launch verified
- staged `crosvm-angle.bat --help` launch verified

### 9.2 Script-level ready but not fully host-validated in this environment

- Linux `build_all.sh` external dependency path
- macOS `build_all.sh` ANGLE + MoltenVK staging path

These paths were updated in source, but could not be fully compiled and runtime-tested in the
current Windows environment.

## 10. Remaining caveats

1. The Windows path is now build-closed and staged, but that does not by itself prove every guest
   runtime workload is functionally correct.
2. Linux and macOS code paths are implemented at the configuration and staging level, but still
   require native-host validation.
3. macOS depends on an actual usable `MoltenVK` runtime directory for the staged wrapper path to be
   complete at runtime.
4. `ExternalBlob` remains intentionally disabled for the ANGLE-enabled path; if future work needs
   external-handle export, it must be designed so it does not break the coherent guest-visible
   contract.

## 11. File-by-file change index

### Root repo

- `build_all.bat`
- `build_all.sh`
- `frameworks/native/libs/binder/CMakeLists.txt`
- `frameworks/native/libs/binder/Binder.cpp`
- `frameworks/native/libs/binder/binder_module.h`
- `frameworks/native/libs/binder/platform/OS_windows.cpp`
- `frameworks/native/libs/binder/platform/unix_socket_compat.h`
- `packages/modules/Virtualization/android/virtmgr/Cargo.toml`
- `packages/modules/Virtualization/android/virtmgr/src/crosvm/crosvm_windows.rs`
- `packages/modules/Virtualization/libs/desktop_host/src/windows.rs`

### `external/crosvm`

- `devices/src/virtio/gpu/mod.rs`
- `devices/src/virtio/gpu/parameters.rs`
- `src/crosvm/cmdline.rs`
- `src/crosvm/gpu_config.rs`
- `src/crosvm/sys/windows/cmdline.rs`
- `vm_control/src/lib.rs`

### `hardware/google/gfxstream`

- `CMakeLists.txt`
- `third-party/CMakeLists.txt`
- `host/BorrowedImage.h`
- `host/compressedTextureFormats/AstcCpuDecompressor.h`
- `host/features/include/gfxstream/host/Features.h`
- `host/gl/glestranslator/EGL/EglOsApi_wgl.cpp`
- `host/vulkan/VkEmulatedPhysicalDeviceMemory.cpp`
- `host/vulkan/VkEmulatedPhysicalDeviceMemoryTests.cpp`

### `hardware/google/aemu`

- `CMakeLists.txt`
- `base/CMakeLists.txt`
- `base/System.cpp`
- `build-config/gfxstream/CMakeLists.txt`

## 12. Recommended operational usage

For the Windows staged build, use:

- `out\dist\windows\bin\virtmgr.exe`
- `out\dist\windows\bin\vm.exe`
- `out\dist\windows\bin\crosvm-angle.bat`

The wrapper is preferred over invoking `crosvm.exe` directly when the ANGLE-backed path is needed,
because it sets the runtime search path to:

- `dist\lib`
- `dist\gfx\angle`

This is what makes the staged `gfxstream + ANGLE` chain self-contained.
