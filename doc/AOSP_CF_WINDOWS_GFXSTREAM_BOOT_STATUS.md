# AOSP Cuttlefish on Windows WHPX + crosvm + rutabaga/gfxstream boot status

Date: 2026-06-13
Last updated: 2026-06-24

## Goal

Boot `aosp_cf_x86_64_phone-trunk_staging-userdebug` on Windows with the local
`crosvm.exe`, WHPX, and the mandatory rutabaga/gfxstream graphics path.

The active route is:

- Windows host with WHPX.
- Local crosvm build with `whpx,composite-disk,android-sparse,gfxstream`.
- Direct-kernel Cuttlefish x86_64 phone boot from the composite disk.
- `force_normal_boot=1`, AVB/verity/encryption disabled for bring-up.
- Nonsecure Gatekeeper/KeyMint for the non-protected Windows host VM.
- `rutabaga + gfxstream` with ANGLE/Vulkan; host Vulkan is currently provided by
  SwiftShader loader/ICD DLLs beside `crosvm.exe`.
- Single vCPU, block queues forced to 1, and `clearcpuid=297` until the UI boot
  is stable.

This route is still correct. It is not a headless-only route and it is not
switching away from rutabaga/gfxstream.


## 2026-06-24 Linux host update

Linux host direct-kernel Android boot now reaches framework boot completion using the local
Cuttlefish-style flow:

```bash
./scripts/run_android_linux.sh --timeout-secs 180
./scripts/check_android_linux_markers.sh out/dist/logs/android-linux
```

Validated markers:

- `PersistentDataBlockService.onStart` completes.
- system_server reaches `OnBootPhase_1000`.
- init processes `sys.boot_completed=1`.
- `scripts/check_android_linux_markers.sh` passes.

The previous phase-500 blocker was caused by a missing `/dev/block/by-name/frp` device in the
ad-hoc aggregate disk. The Linux disk builder now creates a 1 MiB `frp` partition backed by
`out/android-linux/factory_reset_protected.img`, matching Cuttlefish persistent disk semantics.
This fixes the `PersistentDataBlockService init timeout` and the resulting system_server/zygote
restart loop on Linux.

The Linux non-headless graphics route is now validated:

```bash
./scripts/build_angle_linux.sh
./scripts/run_android_linux.sh --mode gpu --gpu-guest-angle --mem 8192 --timeout-secs 240
./scripts/check_android_linux_markers.sh out/dist/logs/android-linux
./scripts/check_android_linux_gfx_markers.sh out/dist/logs/android-linux
./scripts/check_android_linux_gfx_screenshot.sh --log-dir out/dist/logs/android-linux --cid 100
```

Validated Linux GPU markers:

- init processes `sys.boot_completed=1`.
- `SurfaceFlinger: Boot is finished`.
- RenderEngine reports guest ANGLE on Vulkan:
  `ANGLE (NVIDIA, Vulkan 1.3.0 (... NVIDIA GeForce RTX 2060 ...))`.
- gfxstream host reports `stream_renderer_init Gfxstream initialized successfully!`.
- gfxstream features include `GuestVulkanOnly: enabled` and
  `VulkanAllocateHostMemory: disabled`.
- surfaceflinger, bootanimation, Settings, SystemUI, Launcher3, and system_server create
  `engine:ANGLE` Vulkan devices.
- virtio-gpu receives scanout updates and resource flushes from the guest composition path.
- ADB over AF_VSOCK reaches guest `adbd`, and `screencap` pulls a nonblank 1280x720 RGBA PNG
  from SurfaceFlinger.

The passing Linux mode is guest ANGLE with gfxstream Vulkan-only contexts:

```text
backend=gfxstream,
displays=[[mode=windowed[1280,720],dpi=[320,320],refresh-rate=60]],
context-types=gfxstream-vulkan:gfxstream-composer,
angle=true,gles=false,vulkan=true,wsi=vk
```

This is the route Windows should borrow next. The older `gfxstream-gles` guest GL pipe route is
not the passing direct-crosvm Linux path; it can reach host gfxstream init but leaves the guest
blocked on unavailable GL transport.

No AOSP clean or full rebuild was used for this Linux validation. The run used the existing
`~/aosp/out/target/product/vsoc_x86_64` artifacts.

Latest Linux screenshot validation wrote:

```text
out/dist/logs/android-linux/adb/gfxstream-angle.png
size=1280x720
mode=RGBA
bbox=(0, 0, 1280, 720)
unique_colors=7020
mean_rgba=[22.38, 24.0, 56.12, 255.0]
```

## Current Windows result

Latest useful diagnostic run before cleanup:

```text
out/dist/logs/run82_logcat_dd_swiftshader_clearcpuid_1q
```

The GPU path now initializes successfully:

```text
SharedLibrary::open succeeded for [vulkan-1.dll]
Selecting Vulkan device: SwiftShader Device (Subzero)
GPU-FRONTEND-INIT: rutabaga_builder.build after
GPU-FRONTEND-INIT: initialize_frontend exit
```

Guest-side display bring-up also progresses:

```text
codx_dms: skip_blocking_sf_composition_query
codx_system_server: before_WaitForDisplay
codx_system_server: after_WaitForDisplay
```

SystemServer repeatedly reaches `startOtherServices`:

```text
codx_system_server: after_StartPackageManagerService
codx_system_server: end_startBootstrapServices
codx_system_server: end_startCoreServices
codx_system_server: enter_startOtherServices
```

The run82 `dd bs=900` logcat-to-kmsg diagnostic captured the actual Java fatal
path:

```text
E/AndroidRuntime: *** FATAL EXCEPTION IN SYSTEM PROCESS: main
java.lang.RuntimeException: Failed to boot service
    com.android.server.pdb.PersistentDataBlockService:
    onBootPhase threw an exception during phase 500
Caused by: java.lang.IllegalStateException:
    Service PersistentDataBlockService init timeout
at com.android.server.pdb.PersistentDataBlockService.waitForInitDoneSignal
at com.android.server.pdb.PersistentDataBlockService.onBootPhase
```

After the fatal exception, zygote is killed and restarted. No
`sys.boot_completed=1` or `BOOT_COMPLETED` evidence was found in the captured
run82 log before cleanup.

## Route assessment

The route is correct.

What is now proven:

- Direct-kernel WHPX boots Android userspace from the Cuttlefish composite disk.
- The ext4/super image replacement flow is working.
- The early bring-up choices are valid: force normal boot, disabled verity,
  disabled userdata encryption, nonsecure Gatekeeper/KeyMint.
- rutabaga/gfxstream is active and no longer blocked at capset/frontend init.
- DisplayManagerService is no longer waiting forever on the early SurfaceFlinger
  composition query.
- SystemServer reaches bootstrap, core services, PackageManager, and
  `startOtherServices`.
- The zygote/system_server restart loop now has a concrete first fatal:
  `PersistentDataBlockService` times out during boot phase 500.

What should stay temporary:

- APEX/apexd bind-mount workaround.
- AVB/verity/encryption disablement.
- DMS skip of the blocking composition query.
- SystemServer/DMS/SurfaceFlinger kmsg markers.
- crosvm GPU and WHPX diagnostic log edits.
- Single-vCPU and block-queue limits.

Do not revisit SMP, x2APIC, AVB, encryption, or cleanup until the single-vCPU
gfxstream path reaches a stable UI boot.

## Current Windows blocker

The current Windows blocker from the older WHPX run is not rutabaga/gfxstream.

The older Windows blocker is:

```text
PersistentDataBlockService init timeout during SystemServer.startOtherServices()
boot phase 500.
```

run82 shows useful progress after `enter_startOtherServices`, including service
manager queries for power, vibrator, consumer IR, sensors, OEM lock, WiFi,
soundtrigger, thermal, USB, biometrics, and authsecret. The first fatal is not
in GPU or DisplayManagerService; it is:

```text
com.android.server.pdb.PersistentDataBlockService.waitForInitDoneSignal()
```

Linux has already fixed this class of blocker by adding an `frp` partition to the generated
Cuttlefish aggregate disk. Port that disk layout to Windows before continuing WHPX UI runs.

Two diagnostic details matter:

- Plain `logcat >> /dev/kmsg` loses the Java stack when a long line hits kmsg's
  write-size limit.
- Piping logcat through `/system/bin/dd bs=900 of=/dev/kmsg` preserves enough
  stack data to identify the failing service.

There is also a recurring early WTF:

```text
android.util.Log$TerribleFailure:
Outgoing transactions from this process must be FLAG_ONEWAY
```

This appears during SystemServer context/display metric setup and should be
tracked, but it is not the fatal that kills the boot loop in run82.

## Next steps

1. Port the Linux aggregate disk layout to Windows, including the 1 MiB `frp`
   partition backed by `factory_reset_protected.img`.
2. Keep the route unchanged after the disk fix: WHPX, crosvm, single vCPU, block
   queue 1, `clearcpuid=297`, and rutabaga/gfxstream.
3. Preserve the `dd bs=900` logcat diagnostic if more Java stack traces are
   needed:
   ```text
   /system/bin/logcat ... 2>&1 | /system/bin/dd bs=900 of=/dev/kmsg
   ```
4. Incrementally rebuild `systemimage superimage` for
   `aosp_cf_x86_64_phone-trunk_staging-userdebug`, copy through
   `/media/sf_Desktop`, and inject the sparse super image into a fresh composite
   disk on `D:\bscp_run`.
5. Boot with the Linux-proven gfxstream guest ANGLE shape adapted to Windows:
   - `--cpus 1`
   - `CROSVM_WHPX_BLOCK_QUEUES=1`
   - `clearcpuid=297`
   - `backend=gfxstream`
   - `displays=[[mode=windowed[1280,720],dpi=[320,320],refresh-rate=60]]`
   - `context-types=gfxstream-vulkan:gfxstream-composer`
   - `angle=true`
   - `gles=false`
   - `vulkan=true`
   - SwiftShader Vulkan ICD/DLLs beside `crosvm.exe`
6. The success criterion for the next Windows run is matching the Linux markers:
   `sys.boot_completed=1`, `SurfaceFlinger: Boot is finished`, and RenderEngine
   reporting ANGLE over Vulkan. After that, use the Windows localhost TCP ->
   guest vsock bridge to run `adb shell screencap -p`, pull the PNG, and apply
   the same nonblank 1280x720 image check used by
   `scripts/check_android_linux_gfx_screenshot.sh`.

## Local runtime artifacts

Known-good local inputs:

```text
C:\workspace\bscp\bscp\out\dist\img\cf_os_composite_run16_halboot_pre_run43.img
C:\workspace\bscp\bscp\out\dist\img\initrd_20260612_halboot.img
C:\workspace\bscp\bscp\out\dist\img\kernel_cf
C:\workspace\bscp\bscp\out\dist\windows\bin\crosvm.exe
C:\workspace\bscp\bscp\out\dist\windows\bin\vulkan-1.dll
C:\workspace\bscp\bscp\out\dist\windows\bin\vk_swiftshader.dll
C:\workspace\bscp\bscp\out\dist\windows\bin\vk_swiftshader_icd.json
```

Temporary run80/run81/run82 logs, desktop `super_codx_logcat*.img` files, and
`D:\bscp_run\cf_os_composite_run8*.img` images are cleanup candidates once this
document has the summarized evidence.

The known-good base composite disk to preserve is:

```text
C:\workspace\bscp\bscp\out\dist\img\cf_os_composite_run16_halboot_pre_run43.img
```

## Bottom line

The Linux host route is now proven through Android boot completion with
gfxstream + guest ANGLE rendering over Vulkan. For Windows, carry over the Linux
disk layout fix and the guest ANGLE `gles=false` gfxstream mode before spending
more time on AOSP-side service work.
