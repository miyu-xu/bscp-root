# ChromeOS on WHPX — Complete Guide

Date: 2026-06-05

## 1. Overview

ChromeOS (amd64-generic) boots successfully on crosvm WHPX (Windows Hypervisor Platform). Both OVMF firmware boot and custom kernel direct boot paths work. Userspace initialization (chromeos_startup, udevd, encrypted stateful) completes.

---

## 2. Prerequisites

### 2.1 Kernel: CONFIG_X86_X2APIC=y (CRITICAL)

WHPX exposes x2APIC mode to the guest. Without `CONFIG_X86_X2APIC=y`, the Linux kernel disables APIC entirely, causing:
- Only 1 CPU available (SMP fails)
- IOAPIC unavailable
- MSI/MSI-X interrupts unavailable → virtio devices get no interrupts → disk I/O hangs

**Build**: WSL2 Ubuntu, Linux 6.1.119, gcc 13.3.0

Required kernel config: X86_X2APIC=y, VIRTIO_BLK=y, VIRTIO_PCI=y, DEVTMPFS=y, SERIAL_8250=y, HVC_DRIVER=y, EXT4_FS=y

**Artifact**: `out/dist/img/custom_kernel` (~11.6MB bzImage, build #14)

### 2.2 Initramfs

`out/dist/img/initramfs.cpio.gz` — patches ChromeOS upstart configs for crosvm/WHPX:
- Removes `started lockbox-cache` dependency from `ui.conf`
- Makes `lockbox-cache` pre-start non-fatal
- Replaces `cros_configfs` with no-op
- Replaces `trunksd` with no-op
- Creates `boot-splash.conf` that starts frecon for DRM connector probing
- Creates simplified `ui-pre-start` that skips problematic checks
- Bind-mounts `chrome_dev.conf` for auto-login flags

Built by: `scripts/create_initramfs.sh`

### 2.3 GPU Dependencies

- `libgfxstream_backend.dll` + `gfxstream_backend.dll` in PATH
- MinGW runtime DLLs (`libgcc_s_seh-1.dll`, `libwinpthread-1.dll`, `libstdc++-6.dll`)
- ANGLE DLLs (`C:\workspace\bscp\angle\out\Release-GfxAngle-Clang`)

### 2.4 Disk Image

`test_image_autologin.bin` — amd64-generic ChromeOS test image with autologin configured.

---

## 3. Boot Commands

### 3.1 Direct Kernel Boot (Primary)

`scripts/run_chromeos.bat` — Uses custom kernel + initramfs. Kernel params: `root=/dev/vda3 rw cros_debug`. GPU: gfxstream, 1024x768, vulkan native swapchain.

### 3.2 Firmware Boot (OVMF)

`scripts/run_chromeos_fw.bat` — Uses OVMF_DEBUG.fd firmware. Boots the disk image's built-in ChromeOS kernel via UEFI. Requires OVMF firmware in `out/dist/firmware/OVMF_DEBUG.fd`.

---

## 4. crosvm Changes (6 files)

| File | Change | Purpose |
|------|--------|---------|
| `devices/src/virtio/gpu/virtio_gpu.rs` | Auto-create surface for scanout 0 | Window appears immediately without guest SET_SCANOUT |
| `gpu_display/src/gpu_display_win/virtual_display_manager.rs` | Return proper client rect bounds | gfxstream gets correct rendering dimensions |
| `gpu_display/src/gpu_display_win/window.rs` | Remove premature ShowWindow | Fixes 16-window taskbar issue |
| `devices/src/tpm_tis.rs` | Full TPM TIS MMIO device with NVRAM persistence | Enables ChromeOS TPM probe + encryption without external swtpm |
| `x86_64/src/lib.rs` | TPM NVRAM path + ACPI TPM2 table | MMIO registration at 0xFED40000 |
| `hypervisor/src/whpx/vm.rs` | TscDeadlineTimer capability | Upstream parity |

---

## 5. Known crosvm WHPX Fixes (Committed)

1. WHPX IO: handle_io_with_mmio — `hypervisor/src/whpx/vcpu.rs`
2. WHPX interrupt: WHvRequestInterrupt + remove kick — `devices/src/irqchip/whpx.rs`
3. MSR #GP to RAZ/WI — `hypervisor/src/whpx/vcpu.rs`
4. ACPI PNP0501 COM1-COM4 — `x86_64/src/lib.rs`
5. Remote IRR cleanup after MSI fallback — `devices/src/irqchip/whpx.rs`

---

## 6. GPU / Display Pipeline

### 6.1 How It Works

1. `VirtioGpu::new()` auto-creates a display surface for scanout 0
2. WndProc thread resizes window from 1x1 to 1024x768, shows with SetWindowPos
3. `Surface::new()` calls `gfxstream_backend_setup_window()` to connect the HWND
4. gfxstream DLL creates a Vulkan/ANGLE rendering context on the window
5. Guest GPU commands flow: virtio-gpu to gfxstream to Vulkan swapchain to visible output

### 6.2 Verification Log

```
x2apic: enabled by BIOS, switching to x2apic ops
smpboot: Allowing 2 CPUs
IOAPIC[0]: apic_id 0, version 32, GSI 0-23
[drm] Initialized virtio_gpu 0.1.0
gpu_display: Creating GUI window (16 scanouts)
vhost-user gpu device ready
create_surface: id=1 scanout=Some(0) type=Scanout
EXT4-fs (vda3): mounted rootfs
Run /sbin/init as init process
chromeos_startup: all services running
```

### 6.3 Known Issue: Black Window

The window shows black (default class background) until the guest sends GPU rendering commands. ChromeOS requires user login before Chrome sends GPU commands. GDI fallback (FlipFramebuffer) is blocked by gfxstream child window.

---

## 7. WHPX MMIO / IO Architecture

### 7.1 MMIO Access

- Crosvm reads WHPX emulator context to check if MMIO access succeeded
- If `EmulationSuccessful()`, result is returned from the emulator's data buffer
- If emulation fails, `inject_gp_fault()` is called and the WHPX status error is returned
- The MMIO handler supports both Read and Write directions

### 7.2 IO Port Access

- Crosvm handles IO port emulation similarly to MMIO
- Success path: read `EmulatorIoAccessInfo` for port, direction, and data
- Failure path: log the failure context and return error
- Used for serial port (0x3F8/0x2F8) and other legacy IO devices

### 7.3 Device MMIO Registration

- Firmware configuration device at 0xd0000000 (FW_CFG_IO_BASE)
- Debug console device at 0x3f8 (standard x86 debugcon port)
- TPM TIS device at 0xFED40000 (standard x86 TPM MMIO base)
- All devices registered on the platform MMIO bus via `Bus::insert()`

---

## 8. OVMF Firmware Boot

### 8.1 Boot Flow

1. OVMF_DEBUG.fd is loaded as the VM BIOS
2. OVMF initializes PCI devices including virtio-blk and virtio-gpu
3. OVMF loads ChromeOS kernel from the disk's EFI system partition
4. ChromeOS kernel boots with dm-verity root

### 8.2 Key Log Markers

```
Found virtio serial device           -> Serial port detected
Found PCI display device             -> GPU detected
InstallProtocolInterface: ...        -> PCI protocol binding
BdsDxe: Locate Variable Policy       -> UEFI boot services
```

### 8.3 Current Status

OVMF detects all devices and reaches BDS phase. `ProtectUefiImage` errors and variable initialization issues may prevent kernel loading in some configurations.

---

## 9. File Layout

```
out/dist/img/
  custom_kernel                   Custom Linux 6.1.119 with CONFIG_X86_X2APIC=y
  initramfs.cpio.gz               Patched ChromeOS upstart configs
  test_image_autologin.bin        ChromeOS disk image
  firmware/OVMF_DEBUG.fd          OVMF firmware for UEFI boot

scripts/
  create_initramfs.sh             Builds the initramfs CPIO archive
  run_chromeos.bat                Direct kernel boot script
  run_chromeos_fw.bat             Firmware boot script
  fix_chromeos_grub_wsl.sh        Patches grub for serial console
  mount_chromeos_efi.sh           Mounts ChromeOS EFI partition
  patch_chromeos_efi_boot.ps1     EFI boot patcher

external/crosvm/                  crosvm fork with WHPX fixes
```

---

## 10. Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| APIC disabled, 1 CPU | Missing CONFIG_X86_X2APIC | Rebuild kernel with x2APIC |
| Reboot loop after chromeos_startup | TPM NVRAM not persisted | Use `rw` root + `cros_debug` |
| No GPU window | Surface not auto-created | Auto-create surface in VirtioGpu::new() |
| 16 taskbar windows | ShowWindow called on all pre-created windows | Remove ShowWindow from GuiWindow::new() |
| Black window | Guest not sending GPU commands | ChromeOS needs login |
| session_manager exits status 2 | Minijail needs CONFIG_USER_NS | Use simplified ui.conf without minijail |
| OVMF not loading kernel | UEFI variable errors | Use OVMF_DEBUG.fd |
