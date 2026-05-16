# Cross-platform `gfxstream + ANGLE(Vulkan)` integration

This document records the implemented cross-platform plan and the exact code changes made to wire
`gfxstream + ANGLE(Vulkan backend)` across Linux, Windows, and macOS.

It also captures two coherent-memory phases:

1. the original `angle=true` integration phase, which enabled the host-memory path and established
   coherent memory as a first-class requirement
2. the current **runtime-gated coherent-memory phase**, which tightens guest `HOST_COHERENT`
   exposure so it is only shown when the host backing is both import-compatible and genuinely
   coherent

So this document now covers:

- the original cross-platform ANGLE + gfxstream integration work
- the current coherent-memory implementation strategy
- the current implementation / validation status
- the remaining follow-up tasks

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

### 3.3 Guest-visible coherent memory is now gated by runtime probe results

The original integration phase broadly treated coherent memory as a guest contract goal.

The current implementation is stricter:

- guest `HOST_COHERENT` is **not** exposed simply because `VulkanAllocateHostMemory` is enabled
- it is exposed only when runtime probing proves that the selected host memory type is:
  1. import-compatible through `VK_EXT_external_memory_host`
  2. `HOST_VISIBLE`
  3. genuinely `HOST_COHERENT`

This means guest coherent memory is now tied to the real backing used by the host allocation path
instead of being inferred only from feature policy.

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

- Replaced the previous unconditional `HOST_VISIBLE -> HOST_COHERENT` upgrade.
- Guest-visible `HOST_COHERENT` is now exposed **per memory type** and only when the mapped host
  memory type appears in the runtime probe result.
- Removed the old split coherent-gating decision from this file so the final decision is centralized
  here.

This file is now the guest-capability gating layer for coherent memory.

#### `host/vulkan/VkEmulatedPhysicalDeviceMemory.h`

- Added `CoherentHostMemoryProbeResult`.
- Extended `EmulatedPhysicalDeviceMemoryProperties` so the runtime probe result is an explicit
  constructor input and stored member.
- Added accessor support so the allocation path can reuse the same coherent-compatible host type
  mask.

#### `host/vulkan/VkCommonOperations.h`

- Declared `probeCoherentHostMemory(...)`.

#### `host/vulkan/VkCommonOperations.cpp`

- Implemented `probeCoherentHostMemory(...)`.
- Added an init-time dummy host-pointer probe using `vkGetMemoryHostPointerPropertiesEXT()`.
- Filters import-compatible memory type bits down to host types that are both `HOST_VISIBLE` and
  `HOST_COHERENT`.
- Uses platform-appropriate aligned host allocations:
  - `posix_memalign` on Linux / POSIX hosts
  - `_aligned_malloc` on Windows

This file is now the runtime-probe layer for Linux / Windows coherent gating.

#### `host/vulkan/VkDecoderGlobalState.cpp`

- Wired the coherent host-memory probe into physical-device memory helper construction.
- Tightened the `VulkanAllocateHostMemory` allocation path:
  - detects whether the guest requested coherent memory
  - intersects import-compatible host types with the coherent probe mask
  - rejects allocations that would otherwise fall back to non-coherent host memory

This file is now the allocation-path enforcement layer.

#### `host/vulkan/VkEmulatedPhysicalDeviceMemoryTests.cpp`

- Updated all helper construction sites to pass explicit probe results.
- Replaced the original single coherent test with finer-grained coverage:
  - probe succeeds -> guest sees coherent
  - probe returns no coherent-compatible types -> guest does not see coherent
  - partial coherent type mask -> only matching guest-visible types keep coherent
  - empty probe -> coherent fully removed from guest-visible memory properties

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

The coherent-memory plan is now explicitly split into **Phase A / Phase B / Phase C**.

### 7.1 Phase A: capability gating (implemented)

This is the part that is now landed in code.

#### Scope

- do not add guest ABI
- do not introduce `ExternalBlob`
- do not introduce shadow/staging
- do not introduce non-coherent software flush/invalidate emulation
- do not extract a new platform abstraction layer yet

#### Linux / Windows coherent semantics

For Linux and Windows, the coherent semantics path is the same class of implementation:

- `VK_EXT_external_memory_host`
- direct imported host allocation
- per-memory-type runtime probe
- allocation-time coherent filtering

The semantic contract is:

1. gfxstream creates an aligned host allocation
2. the allocation is imported via
   `VK_EXTERNAL_MEMORY_HANDLE_TYPE_HOST_ALLOCATION_BIT_EXT`
3. the selected host memory type must be both:
   - import-compatible
   - genuinely `HOST_COHERENT`
4. only then is guest `HOST_COHERENT` exposed

That means Linux / Windows coherent memory is now tied to a direct imported coherent backing rather
than to a broad feature-flag promise.

#### Gating flow

The implementation now works like this:

1. **init-time probe**
   - `probeCoherentHostMemory(...)`
   - probe a dummy aligned host pointer with `vkGetMemoryHostPointerPropertiesEXT()`
   - keep only host memory types that are:
     - import-compatible
     - `HOST_VISIBLE`
     - `HOST_COHERENT`
2. **guest capability exposure**
   - `VkEmulatedPhysicalDeviceMemory.cpp`
   - only guest memory types whose mapped host type is in the probe mask keep `HOST_COHERENT`
3. **allocation-time enforcement**
   - `VkDecoderGlobalState.cpp`
   - if the guest requested coherent memory, the final host type selection is restricted to the same
     coherent-compatible mask

This closes the old gap where the guest could see coherent while the actual host allocation still
landed on a non-coherent host memory type.

#### `ExternalBlob`

`ExternalBlob` remains intentionally excluded from Phase A.

Reason:

- it is a different backing/handle path
- it does not participate in the `VK_EXT_external_memory_host` probe chain
- introducing it here would split the coherent decision path again

So for the current coherent-memory implementation:

- `VulkanAllocateHostMemory` is the relevant backing path
- `ExternalBlob` is not part of the coherent solution

### 7.2 Phase B: macOS validation experiment (not implemented, environment-blocked)

macOS is intentionally not forced into the Linux / Windows implementation path.

The current rule is:

- keep the same coherent goal
- keep the same "avoid copy / avoid explicit sync" preference
- but do **not** add a new guest ABI
- and do **not** assume that MoltenVK can use the same host-pointer-import path as Linux / Windows

So the current macOS work is reduced to a validation task:

- under **pure crosvm** constraints
- with **no guest kernel changes**
- check whether the existing:
  - `virtio-gpu`
  - `rutabaga.map()`
  - `SharedMemoryMapper`
  - host-visible shared memory path
  can lead to a real direct coherent backing

The planned experiment is to export the backing as `MTLBuffer` and check `MTLStorageMode`.

Acceptance rule:

- `MTLStorageModeShared` -> macOS may have a viable coherent path
- `MTLStorageModeManaged` or explicit flush requirements -> macOS is considered unsupported for this
  phase

The earlier `1.diff / 2.diff / 3.diff` GMD/HVF path remains a design reference only, not an
implementation path, because it would require guest-kernel-visible interfaces and violates the
current "pure crosvm, no kernel intrusion" constraint.

### 7.3 Phase C: future abstraction (not started)

Only after Linux / Windows / macOS all have validated coherent paths should the code introduce a
shared abstraction such as:

- coherent-backing probe interface
- platform-specific backing selector

Until then, abstraction is intentionally postponed so unverified platform differences do not get
frozen into the code structure.

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

## 9. Current status

### 9.1 Completed in code

- `crosvm` `angle=true` configuration path implemented
- `gfxstream` `AngleIndirect` feature introduced
- `ExternalBlob` remains disabled for the `angle=true` path
- external dependency path support added for ANGLE / `aemu` / `flatbuffers`
- Windows `gfxstream_backend` build completed
- Windows ANGLE runtime build completed
- Windows `binder-rpc` build completed
- Windows `virtmgr` / `vm` / `crosvm` release builds completed
- Windows dist staging completed

### 9.2 Completed in the new coherent-memory Phase A

- `CoherentHostMemoryProbeResult` added
- `probeCoherentHostMemory(...)` added
- guest `HOST_COHERENT` exposure converted from feature-flag policy to per-memory-type runtime
  gating
- `VkDecoderGlobalState.cpp` allocation path tightened so coherent requests cannot silently fall back
  to non-coherent host memory types
- old parallel coherent-gating branches removed from `VkEmulatedPhysicalDeviceMemory.cpp`
- coherent-memory unit-test coverage updated in source

### 9.3 Verified in the current Windows environment

- the modified gfxstream Vulkan/backend code compiles in the existing Windows MinGW build directory
- `gfxstream_backend` rebuild completes successfully with the new gating changes

### 9.4 Still blocked or not yet validated

- Linux native end-to-end validation still requires a Linux host environment
- macOS Phase B validation still requires Apple Silicon + MoltenVK
- a standalone `ENABLE_VKCEREAL_TESTS=ON` Windows test-build path is still constrained by host-side
  dependency availability in this environment, even though the test source updates are in place

## 10. Next tasks

1. **Linux native validation**
   - run the updated coherent path on a Linux host
   - confirm that guest `HOST_COHERENT` appears only when the probed imported host memory type is
     genuinely coherent
2. **Windows test-environment closure**
   - provide the missing host-side dependencies required for the standalone gfxstream unit-test
     configuration
   - build and run the updated coherent-memory tests
3. **macOS Phase B experiment**
   - stay within pure-crosvm constraints
   - validate whether the existing virtio-gpu / rutabaga / `SharedMemoryMapper` path can land on
     `MTLStorageModeShared`
   - if not, record macOS as unsupported for the current coherent phase
4. **Future abstraction only after platform proof**
   - do not extract a shared coherent-backing abstraction before Linux / Windows / macOS each have a
     validated path

## 11. Remaining caveats

1. The Windows path is build-closed, but that does not by itself prove every guest runtime workload
   is functionally correct.
2. Linux and macOS still require native-host validation for the coherent-memory story.
3. `ExternalBlob` remains intentionally outside the current coherent-memory implementation path.
4. The current implementation is intentionally Phase-A-scoped; it does not solve macOS coherent
   backing yet.

## 12. File-by-file change index

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
- `host/vulkan/VkCommonOperations.cpp`
- `host/vulkan/VkCommonOperations.h`
- `host/vulkan/VkDecoderGlobalState.cpp`
- `host/vulkan/VkEmulatedPhysicalDeviceMemory.cpp`
- `host/vulkan/VkEmulatedPhysicalDeviceMemory.h`
- `host/vulkan/VkEmulatedPhysicalDeviceMemoryTests.cpp`

### `hardware/google/aemu`

- `CMakeLists.txt`
- `base/CMakeLists.txt`
- `base/System.cpp`
- `build-config/gfxstream/CMakeLists.txt`

## 13. Recommended operational usage

For the Windows staged build, use:

- `out\dist\windows\bin\virtmgr.exe`
- `out\dist\windows\bin\vm.exe`
- `out\dist\windows\bin\crosvm-angle.bat`

The wrapper is preferred over invoking `crosvm.exe` directly when the ANGLE-backed path is needed,
because it sets the runtime search path to:

- `dist\lib`
- `dist\gfx\angle`

This is what makes the staged `gfxstream + ANGLE` chain self-contained.
