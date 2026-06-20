# AOSP Cuttlefish on Windows WHPX + crosvm + rutabaga/gfxstream boot status

Date: 2026-06-13
Last updated: 2026-06-18

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

## Current result

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

## Current blocker

The current blocker is not rutabaga/gfxstream.

The current blocker is:

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

1. Keep the route unchanged: WHPX, crosvm, single vCPU, block queue 1,
   `clearcpuid=297`, and rutabaga/gfxstream.
2. Fix or bypass `PersistentDataBlockService` for this direct-kernel
   Cuttlefish bring-up. The likely local fix is to make the service tolerate a
   missing/slow persistent block backend on this path, or disable the service
   while `force_normal_boot` Windows bring-up is active.
3. Preserve the `dd bs=900` logcat diagnostic if more Java stack traces are
   needed:
   ```text
   /system/bin/logcat ... 2>&1 | /system/bin/dd bs=900 of=/dev/kmsg
   ```
4. Incrementally rebuild `systemimage superimage` for
   `aosp_cf_x86_64_phone-trunk_staging-userdebug`, copy through
   `/media/sf_Desktop`, and inject the sparse super image into a fresh composite
   disk on `D:\bscp_run`.
5. Boot with the same crosvm command:
   - `--cpus 1`
   - `CROSVM_WHPX_BLOCK_QUEUES=1`
   - `clearcpuid=297`
   - `backend=gfxstream`
   - `context-types=gfxstream-vulkan:gfxstream-gles:gfxstream-composer`
   - SwiftShader Vulkan ICD/DLLs beside `crosvm.exe`
6. The success criterion for the next run is passing boot phase 500 and reaching
   `end_startOtherServices` or `sys.boot_completed=1`.

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

The route is sound and now better proven than the older run58 notes: the VM uses
WHPX, crosvm, rutabaga, and gfxstream, and it reaches SystemServer
`startOtherServices`. The immediate work is not more GPU surgery. The next
targeted AOSP-side fix is `PersistentDataBlockService` timing out during boot
phase 500.
