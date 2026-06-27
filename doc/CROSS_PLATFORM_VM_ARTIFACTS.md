# Cross-platform VM artifacts and debug logs

Last updated: 2026-06-27

This document records how to package the existing AOSP and BSCP VM artifacts for
Linux, Windows, and macOS bring-up, and how to export comparable Microdroid and
Android logs. The scripts copy existing outputs only; they do not clean or
rebuild AOSP.

## Packaging

Create a package from the existing `~/aosp` products and the current BSCP dist:

```bash
./scripts/package_aosp_vm_artifacts.sh \
  --aosp-root /opt/workspace/aosp \
  --output-root /mnt/workspace/Windows/bscp-vm-artifacts
```

The default products are:

- `vsoc_x86_64`
- `vsoc_arm64`

The package contains:

- `products/android/<product>/images`: boot, init_boot, vendor_boot, vbmeta, super,
  userdata, metadata, kernel, and other top-level product images.
- `products/android/<product>/direct-linux`: the synthesized
  `aggregate_android.img`, `initrd_android.img`, `android_fstab.dt`, helper
  partitions, HVC inputs, and kernel for direct-crosvm boot input comparison.
- `products/android/<product>/meta`: build fingerprints, image lists, installed-files
  metadata, and related product metadata when present.
- `products/microdroid/<product>/com.android.virt`: the product
  `com.android.virt` tree.
- `products/microdroid/<product>/apex_dir`: the product-specific mounted APEX
  runtime tree required by Microdroid. Keep this split by product/architecture;
  do not treat it as a Linux host directory.
- `products/microdroid/soong/x86_64` and `products/microdroid/soong/arm64`:
  Microdroid kernel, signed kernel, initrd, super, vbmeta, fstab, and JSON from
  Soong intermediates.
- `host/linux-x86_64`: the current Linux host runtime under `out/dist/linux`.
- `host-tools`: selected AOSP host tools that exist in this workspace, not full
  host output trees.

By default the script writes only the directory package. Use `--archive` when a
compressed archive is explicitly needed.

## Debug Log Export

After running Microdroid and Android validation, export the comparable logs:

```bash
./scripts/export_vm_debug_logs.sh \
  --output-root /mnt/workspace/Windows/bscp-vm-debug-logs
```

The export contains:

- `android-linux`: full direct-kernel Android logs, including `hvc.txt`,
  `logcat-hvc2.txt`, `serial.txt`, `stderr.txt`, `stdout.txt`, host window
  dumps, and ADB screenshot validation output.
- `microdroid-linux`: full Linux Microdroid smoke logs from `vm_linux.sh`,
  including `vm-run-microdroid.log` and `guest-log.txt`.

These logs are intended for Windows and macOS comparison. Keep the directory
layout unchanged when diffing across platforms.

## Linux Android Graphics Validation

Android graphics validation is not complete unless both host and guest evidence
exist:

```bash
DISPLAY=:1 ./scripts/run_android_linux.sh \
  --mode gpu \
  --gpu-guest-angle \
  --mem 8192 \
  --timeout-secs 420 \
  --x-display :1

DISPLAY=:1 ./scripts/check_android_linux_host_window.sh \
  --log-dir out/dist/logs/android-linux \
  --x-display :1

./scripts/check_android_linux_gfx_markers.sh out/dist/logs/android-linux
./scripts/check_android_linux_gfx_screenshot.sh --log-dir out/dist/logs/android-linux
```

The 2026-06-27 Linux run validated:

- X11 host window tree:
  - parent crosvm window `0x4200001`, `1280x720`
  - gfxstream native child window `0x4400001`, `1280x720`
- Host window dumps:
  - `out/dist/logs/android-linux/host-window/gfxstream-child-default.xwd`
  - `out/dist/logs/android-linux/host-window/gfxstream-parent-default.xwd`
- Host child-window 12-frame sampling: stable nonblack frames, about 73% nonzero pixels, about
  0.3% white pixels, no all-white/all-black frame observed.
- Guest screenshot: `out/dist/logs/android-linux/adb/gfxstream-angle.png`.
- Screenshot metrics: 1280x720 RGBA, non-empty bbox, 6631 unique colors,
  `mean_rgba=[27.15, 29.0, 39.67, 255.0]`.
- gfxstream host init and guest ANGLE Vulkan markers.
- Runtime feature state:
  - `GuestVulkanOnly enabled`
  - `ExternalBlob disabled`
  - `VulkanAllocateHostMemory disabled`

`--gpu-host-visible-coherent` is available as a diagnostic option. It enables
`external-blob=true` and `renderer-features=VulkanAllocateHostMemory:enabled`, but it is not the
default Android display path because the current guest logs show missing
`VIRTGPU_PARAM_CREATE_GUEST_HANDLE`, `VIRTGPU_PARAM_RESOURCE_SYNC`, and
`VIRTGPU_PARAM_GUEST_VRAM`. Forcing that path currently leads to guest mmap failures and ANGLE
allocation failures.

## Build Notes

Linux gfxstream+ANGLE crosvm must be built with the top-level `gpu`,
`gfxstream`, and `x` features. `build_all.sh` appends these when
`ENABLE_GFXSTREAM_ANGLE=1`. The direct Android launch script runs crosvm under
`timeout --foreground` and redirects stdin from `/dev/null` so interactive
shell job control cannot stop the VM before Android boots.

On a normal Linux development host install the X11 and Wayland build helpers
used by crosvm display backends, for example `libx11-dev`, `libxext-dev`,
`libwayland-dev`, and `wayland-protocols`. In this workspace, X11 validation was
done without a system package install by linking against the existing runtime
`libXext.so.6` through a repo-local symlink under `out/host-deps/x11/lib`.

Do not run `m clean`, `make clean`, `soong clean`, or a full AOSP rebuild for
this package flow. The package scripts consume the existing `~/aosp/out`
artifacts.
