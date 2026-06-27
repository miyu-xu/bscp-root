# AOSP Cuttlefish on Windows WHPX + crosvm + rutabaga/gfxstream boot status

Date: 2026-06-13
Last updated: 2026-06-26

## Goal

Boot `aosp_cf_x86_64_phone-trunk_staging-userdebug` on Windows with the local
`crosvm.exe`, WHPX, and the mandatory rutabaga/gfxstream graphics path.

The active route is:

- Windows host with WHPX.
- Local crosvm build with `whpx,composite-disk,android-sparse,gfxstream`.
- Direct-kernel Cuttlefish x86_64 phone boot from the Linux-proven
  `direct-linux` artifact package.
- `run-mp`, 4 vCPU, 8192 MiB RAM, first block device pinned at PCI `00:03.0`,
  and fixed HVC legacy virtio-console ports for console/logcat/HAL channels.
- `rutabaga + gfxstream` with guest ANGLE over Vulkan:
  `context-types=gfxstream-vulkan:gfxstream-composer`, `angle=true`,
  `gles=false`, `vulkan=true`, `wsi=vk`.
- Native host Vulkan is working on the tested NVIDIA Windows host; SwiftShader
  remains only an optional fallback.

This route is still correct. It is not a headless-only route and it is not
switching away from rutabaga/gfxstream.

## 2026-06-26 Windows parity update

Linux has now booted both Microdroid and Android with visible
`gfxstream + ANGLE` rendering. The Windows runner was aligned to that route:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass `
  -File scripts\run_android_windows_gfxstream_angle.ps1 `
  -FullHvc -TimeoutSecs 420
```

Inputs:

```text
D:\bscp-vm-artifacts\bscp-vm-artifacts-20260626-gfxstream-angle-visible\products\android\vsoc_x86_64\direct-linux
D:\bscp-vm-debug-logs\bscp-vm-debug-logs-20260626-gfxstream-angle-visible\android-linux
```

Validated Windows graphics markers:

- crosvm creates a GUI window for scanout 0.
- gfxstream enables `AngleIndirect`, `GuestVulkanOnly`, and
  `VulkanNativeSwapchain`.
- `VulkanAllocateHostMemory` is disabled on Windows, avoiding the invalid host
  pointer import path.
- `stream_renderer_init Gfxstream initialized successfully!`.
- virtio-gpu receives `update_scanout_resource` and creates a scanout surface.
- Guest SurfaceFlinger starts, bootanimation starts, and RenderEngine reports
  ANGLE over Vulkan on `NVIDIA GeForce MX450`.

Representative log evidence:

```text
gpu_display_win::window: Creating GUI window for scanout 0
Gfxstream feature GuestVulkanOnly enabled
Gfxstream feature VulkanAllocateHostMemory disabled
stream_renderer_init Gfxstream initialized successfully!
GPU-FRONTEND: update_scanout_resource type=Scanout scanout_id=0 resource_id=6
gpu_display_win create_surface: id=2 scanout=Some(0) type=Scanout
SurfaceFlinger: SurfaceFlinger is starting
ANGLE: Renderer (Vulkan 1.3.0 (NVIDIA NVIDIA GeForce MX450))
RenderEngine: renderer  : ANGLE (NVIDIA, Vulkan 1.3.0 (...))
SurfaceFlinger: Enter boot animation
```

The current Windows blocker is no longer GPU bring-up. Android does not yet
reach `sys.boot_completed=1` because `vendor.threadnetwork_hal` crashes before
boot completion:

```text
apexd: Successfully mounted package /vendor/apex/com.android.hardware.threadnetwork.apex
init: Parsing file /apex/com.android.hardware.threadnetwork/etc/threadnetwork-service.rc
init: starting service 'vendor.threadnetwork_hal'
android.hardware.threadnetwork-service: Read() at hdlc_interface.cpp:206: I/O error
init: process with updatable components 'vendor.threadnetwork_hal' exited 4 times before boot completed
apexd: Native process 'vendor.threadnetwork_hal' is crashing. Attempting a revert
init: processing action (sys.init.updatable_crashing=1) from (/system/etc/init/flags_health_check.rc:10)
```

Route assessment:

- The crosvm/WHPX/rutabaga/gfxstream/ANGLE route is correct and should stay.
- Cuttlefish supports ThreadNetwork; do not remove or globally disable the
  ThreadNetwork HAL/APEX for this product.
- The remaining issue is that the hand-written direct-crosvm runners do not yet
  fully reproduce Cuttlefish networking. The HAL launches `ot-rcp` with
  `-Leth1`; Linux logs show the same HAL crash, but Linux reaches boot
  completion before the repeated crash trips rollback, while Windows does not.
- A raw byte patch of `aggregate_android.img` against
  `service vendor.threadnetwork_hal` is not a valid fix. It hits an inactive
  outer copy/metadata area; the live rc is loaded from the mounted APEX path
  `/apex/com.android.hardware.threadnetwork/etc/threadnetwork-service.rc`.
- Appending a ramdisk `/system/etc/init/*.rc` overlay is also not a valid fix;
  that path was not parsed in the direct boot layout.
- Patching the first-stage initrd `/init` does not hit the rollback property
  setter; the relevant strings are in the second-stage init inside the aggregate
  image.

Correct next fixes are direct-runner/network fixes and should be incremental:

1. Keep the ThreadNetwork HAL/APEX in the Cuttlefish product.
2. Rebuild the Linux crosvm dist with the `net` feature and pass two virtio-net
   devices in Cuttlefish order: mobile first at PCI `00:01.1`, ethernet/`eth1`
   second at PCI `00:01.2`.
3. On Linux, either run with enough privilege for crosvm to create transient TAP
   devices, or pass pre-created TAPs to `scripts/run_android_linux.sh` with
   `--mobile-tap` and `--ethernet-tap`.
4. Extend the Windows crosvm networking path. The current Windows/slirp route
   creates only one virtio-net device, so it does not provide the `eth1`
   interface expected by the default ThreadNetwork HAL command line.
5. Re-run the Windows runner and require all Linux parity markers:
   `sys.boot_completed=1`, SurfaceFlinger boot finished, ANGLE/Vulkan
   RenderEngine, and a nonblank 1280x720 window/screenshot.

## 2026-06-26 Linux direct-runner parity findings

Evidence log set (Windows backup):

```text
/mnt/workspace/Windows/bscp-vm-debug-logs/android-linux-xshm-sync-20260626-210645/android-linux/
```

These are supported Cuttlefish product features for
`aosp_cf_x86_64_phone-trunk_staging-userdebug`. Keep them enabled; fix the
hand-written direct runners instead of removing HALs or APEX packages.

| Feature | Guest service / HAL | Cuttlefish host path | Direct-runner gap | Priority |
| --- | --- | --- | --- | --- |
| ThreadNetwork | `vendor.threadnetwork_hal` / `com.android.hardware.threadnetwork` | Two virtio-net devices; `ot-rcp -Leth1` on second NIC (`00:01.2`) | Linux/Windows runners expose fewer than two net devices | P0 |
| Bluetooth | `com.google.cf.bt` / `/dev/hvc5` | `root-canal` HCI `:7300` + `tcp_connector` to HVC FIFOs | `hvc5` is a sink; no RootCanal/connector | P1 |
| NFC | `com.google.cf.nfc` / `/dev/hvc12` | `casimir` NCI `:7800` + `tcp_connector` to HVC FIFOs | `hvc12` is a sink; no Casimir/connector | P1 |
| Telephony | `com.google.cf.rild` / `com.android.phone` | modem simulator + RIL channel | Recheck after radio-adjacent fixes; may be secondary | P2 |

Linux backup-log markers (same product, direct-crosvm without host daemons):

```text
android.hardware.threadnetwork-service: Read() at hdlc_interface.cpp:206: I/O error
bluetooth: Can't start stack, last instance: starting HciHal
libnfc_nci: NFA_DM_NFCC_TIMEOUT_EVT; abort
ActivityManager: ANR in com.android.phone
```

Linux still reached `sys.boot_completed=1` in this run because ThreadNetwork rollback
did not trip before boot completion. Windows boot is slower, so the same missing
devices block `sys.boot_completed=1` there.

ThreadNetwork service URL in the Linux log confirms the `eth1` dependency:

```text
Url: spinel+hdlc+forkpty:///apex/com.android.hardware.threadnetwork/bin/ot-rcp?forkpty-arg=-Leth1&forkpty-arg=1
```

Graphics route conclusions above are unchanged. The new work is host-device parity
for ThreadNetwork, Bluetooth, and NFC in the direct runners.

Linux direct-runner validation (2026-06-27, log dir
`out/dist/logs/android-linux-parity2`):

- Dual-net TAP path exposes guest `eth1` at `00:1a:11:e1:cf:00`.
- `sys.boot_completed=1` is reached with the parity runner.
- ThreadNetwork `hdlc_interface.cpp:206` no longer appears once `eth1` exists.
- Bluetooth reaches `Started HciHal`; NFC Casimir accepts an NCI connection.
- Host `modem_simulator` is started by `scripts/start_cf_host_daemons.sh` via
  `scripts/modem_simulator_launcher.py`, which writes a minimal Cuttlefish config,
  listens on vsock port `9600 + (guest_cid - 3)` (9697 for `--cid 100`), and passes
  `androidboot.modem_simulator_ports` in bootconfig.
- Validated in `out/dist/logs/android-linux-modem`: guest RIL opens port 9697 and
  parity checks pass with no `com.android.phone` ANR.


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

The 2026-06-26 Linux route uses the direct-crosvm windowed gfxstream path with
`wsi=vk`. A surfaceless/no-`wsi` experiment initialized host gfxstream but left
the guest stopped before useful serial output, so it is not the default route.
A host window is not accepted as passing until the captured XWD has real pixel
content; guest `screencap` alone only proves SurfaceFlinger rendered inside
Android.

```bash
DISPLAY=:1 ./scripts/run_android_linux.sh --mode gpu --gpu-guest-angle --mem 8192 --timeout-secs 420 --x-display :1
DISPLAY=:1 ./scripts/check_android_linux_host_window.sh --log-dir out/dist/logs/android-linux --x-display :1
./scripts/check_android_linux_gfx_screenshot.sh --log-dir out/dist/logs/android-linux
```

The host window evidence is:

```text
display=:1
window_id=0x4600001
window_line=0x4600001 "crosvm": () 1280x720+14+49 +50+119
```

The host dump is saved as:

```text
out/dist/logs/android-linux/host-window/crosvm-window.xwd
out/dist/logs/android-linux/host-window/crosvm-window.metrics.txt
```

Latest host-window validation after unlocking Android and opening Settings:

```text
nonblack_ratio=0.731875
bright_ratio=0.673819
mean_luma=157.292
unique_rgb_sample=65
```

Latest Linux screenshot validation wrote:

```text
out/dist/logs/android-linux/adb/gfxstream-angle.png
size=1280x720
mode=RGBA
bbox=(0, 0, 1280, 720)
unique_colors=785
mean_rgba=[243.38, 241.89, 248.12, 255.0]
```

## Historical Windows result

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

## Historical route assessment

The older run82 route was useful for narrowing the problem, but it is now
superseded by the Linux-parity Windows route above.

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

What was temporary diagnostic scaffolding:

- APEX/apexd bind-mount workaround.
- AVB/verity/encryption disablement.
- DMS skip of the blocking composition query.
- SystemServer/DMS/SurfaceFlinger kmsg markers.
- crosvm GPU and WHPX diagnostic log edits.
- Single-vCPU and block-queue limits.

## Historical Windows blocker

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

1. Keep the current Windows graphics route unchanged:
   WHPX, `run-mp`, 4 vCPU, block PCI `00:03.0`, legacy virtio-console HVC, and
   `rutabaga + gfxstream + guest ANGLE`.
2. Keep the Cuttlefish ThreadNetwork HAL/APEX enabled; fix the hand-written
   direct runners so they provide the Cuttlefish-compatible network interfaces
   expected by the HAL.
3. Rebuild Linux crosvm with the `net` feature and validate the new
   `scripts/run_android_linux.sh` dual-net path, using either privileged
   transient TAP creation or pre-created `--mobile-tap`/`--ethernet-tap`
   devices.
4. Extend the Windows crosvm/slirp path to expose the matching second network
   device, then copy refreshed products through `/media/sf_Desktop` into
   `D:\bscp-vm-artifacts`, run
   `scripts\run_android_windows_gfxstream_angle.ps1 -FullHvc -RefreshImages`,
   and require `sys.boot_completed=1`.
5. After boot completion, capture the host window or guest screenshot and apply
   the same nonblank 1280x720 validation used on Linux.

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

## 2026-06-26 Linux host scanout validation

The latest root-side Windows status commit (`e43d0b7`) does not change Linux
runtime behavior. Its gfxstream C++ changes are guarded by `_WIN32`. The latest
`external/crosvm` Windows device-layout commit (`721296366`) changes the Windows
runner path and feature plumbing only; Linux does not execute `src/sys/windows.rs`.

The Linux host still shows the same scanout symptom in
`out/dist/logs/android-linux/stderr.txt`:

```text
GpuFlush: import failed, trying framebuffer copy for surface=1
```

This means the Linux X11 route is not presenting via imported/native scanout. It
falls back to CPU readback into XShm buffers. Guest rendering itself is healthy:
SurfaceFlinger, Launcher, Settings, and SystemServer create Vulkan devices
through ANGLE, and `scripts/check_android_linux_gfx_screenshot.sh` passes with a
nonblank 1280x720 guest screenshot.

Observed host-window risk:

- The problem is not Windows-only; Linux can hit the same fallback-copy class of
  flicker, deformed frames, or discontinuous updates.
- XWD captures during interactive Settings/Home/rotation changes stayed
  structurally valid: 1280x720, 32 bpp, 5120-byte stride, full expected frame
  payload.
- Static host-window captures were byte-identical across repeated samples, so
  no random repaint noise was observed after the XShm sync fix.

Current mitigation in `external/crosvm`:

- Add the `XSync` binding to the generated Xlib wrapper and generator allowlist.
- Make `gpu_display_x.rs` submit fallback XShm frames synchronously: after
  `XShmPutImage`, block until the X server has processed the request, then mark
  the shared-memory buffer reusable.

This is intentionally conservative. It favors a stable Linux baseline over
maximum fallback-copy throughput. The longer-term fix is still to avoid this
path by making gfxstream scanout import succeed on the host display backend.

## Bottom line

The Linux host route is proven through Android boot completion with gfxstream +
guest ANGLE rendering over Vulkan. Windows now matches the graphics/rendering
path and reaches bootanimation, but the hand-written direct runners still need
Cuttlefish-compatible network devices for ThreadNetwork before Windows can reach
`sys.boot_completed=1`.
