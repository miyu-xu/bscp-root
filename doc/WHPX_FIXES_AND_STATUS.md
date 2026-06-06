# WHPX Fixes and Current Status

Date: 2026-06-05

## 1. Fix Summary

### 1.1 Critical: CONFIG_X86_X2APIC=y

WHPX exposes x2APIC mode. Without kernel support, APIC disabled entirely:
- 1 CPU only (SMP fails), IOAPIC unavailable, MSI/MSI-X broken
- virtio devices get no interrupts → disk I/O hangs → guest init fails
- GPU capset timeout (virtio_gpu_get_capsets: timed out)

**Fix**: Rebuild custom kernel with `CONFIG_X86_X2APIC=y`

### 1.2 crosvm Commits

| # | Fix | File | Description |
|---|------|------|-------------|
| 1 | WHPX IO | `hypervisor/src/whpx/vcpu.rs` | `handle_io_with_mmio` for IO port emulation |
| 2 | WHPX interrupt | `devices/src/irqchip/whpx.rs` | `WHvRequestInterrupt` + remove `kick_all_vcpus` |
| 3 | MSR #GP → RAZ/WI | `hypervisor/src/whpx/vcpu.rs` | Unknown MSR reads return 0, writes ignored |
| 4 | ACPI PNP0501 | `x86_64/src/lib.rs` | COM1-COM4 serial ports in ACPI DSDT |
| 5 | Remote IRR cleanup | `devices/src/irqchip/whpx.rs` | Clear Remote IRR after MSI fallback delivery |
| 6 | MSI all dest_ids | `devices/src/irqchip/whpx.rs` | MSI delivery to all destinations, not just ID 0 |
| 7 | WHPX SMP | `hypervisor/src/whpx/vcpu.rs` | INIT/SIPI state machine, vCPU mp_state |
| 8 | MMIO GP fault | `hypervisor/src/whpx/vcpu.rs` | `inject_gp_fault()` on MMIO failure (upstream) |
| 9 | Segment limits | `hypervisor/src/whpx/types.rs` | Use limit_bytes directly (upstream) |
| 10 | TscDeadlineTimer | `hypervisor/src/whpx/vm.rs` | Capability info (upstream parity) |

### 1.3 GPU Display (6 files)

| File | Change | Purpose |
|------|--------|---------|
| `devices/src/virtio/gpu/virtio_gpu.rs` | Auto-create surface for scanout 0 | Window appears without guest SET_SCANOUT |
| `gpu_display/src/gpu_display_win/virtual_display_manager.rs` | Return proper client rect bounds | gfxstream gets correct rendering dimensions (was 0x0) |
| `gpu_display/src/gpu_display_win/window.rs` | Remove premature ShowWindow | Fixes 16 invisible 1x1 windows in taskbar |
| `gpu_display/src/gpu_display_win/mod.rs` | SetWindowPos + window resize | Window shown at 1024x768 on surface create |
| `gpu_display/src/gpu_display_win/window_procedure_thread.rs` | Remove WS_CLIPCHILDREN | GDI rendering visible through gfxstream child window |

### 1.4 TPM (2 files)

| File | Change | Purpose |
|------|--------|---------|
| `devices/src/tpm_tis.rs` | Full TPM TIS MMIO device (1086 lines) | 23+ TPM2 commands, NVRAM persistence to disk |
| `x86_64/src/lib.rs` | TPM MMIO registration + NVRAM path | 0xFED40000, ACPI TPM2 table, env var path |

### 1.5 ChromeOS Boot Fixes

| Issue | Root Cause | Fix |
|-------|-----------|-----|
| Reboot loop after chromeos_startup | TPM NVRAM recreated each boot | `rw` root + `cros_debug` kernel params |
| session_manager exits status 2 | Minijail crashes without CONFIG_USER_NS | Simplified ui.conf (no minijail) |
| DRM connector status unknown | No userspace compositor | modetest sets mode before session_manager |
| Lockbox-cache blocks UI | TPM NVRAM missing | ui.conf removes `started lockbox-cache` requirement |
| Initramfs config not applied | `/copy` dir missing | `mkdir -p /copy` before file creation |

---

## 2. Guest Verification Log (kernel #14)

```
Linux 6.1.119-crosvm #14 SMP PREEMPT_DYNAMIC
x2apic: enabled by BIOS, switching to x2apic ops         ✅
smpboot: Allowing 2 CPUs                                 ✅ (was 1)
IOAPIC[0]: apic_id 0, version 32, GSI 0-23              ✅
[drm] pci: virtio-gpu-pci detected                       ✅
[drm] Initialized virtio_gpu 0.1.0                       ✅ (no timeout)
vhost-user gpu device ready                              ✅
create_surface: id=1 scanout=Some(0) type=Scanout       ✅
EXT4-fs (vda3): mounted rootfs                           ✅
devtmpfs: mounted                                        ✅
Run /sbin/init as init process                           ✅
udevd: Starting version 249                              ✅
chromeos_startup: encrypted stateful mounted             ✅
frecon: Setting card0 as the best drm                    ✅
modetest: setting mode 1024x768 on connectors 34         ✅
session_manager launched                                 ✅
```

---

## 3. Known Issues

### 3.1 Black GPU Window

**Status**: Open. Window appears but shows class background color.
**Root cause**: gfxstream creates a WS_CHILD window that covers the parent. Only guest GPU commands render to this child window. ChromeOS UI never sends GPU commands without login.
**Debug proof**: GDI drawing on parent window is visible (green bands verified). Parent window not blocked.
**Impact**: Cosmetic. Does not affect boot, functionality, or regression.

### 3.2 ChromeOS Init Stuck at frecon

**Status**: Open. Boot stops after ureadahead, before system-services.
**Root cause**: cros_configfs replacement not taking effect; boot-splash start condition not met.
**Impact**: Prevents session_manager/Chrome from starting. Userspace init otherwise functional.

### 3.3 OVMF UEFI Variable Errors

**Status**: Open. `ProtectUefiImage` errors during firmware boot.
**Impact**: May prevent kernel loading in OVMF path. Direct kernel boot unaffected.

### 3.4 Microdroid Not Supported

**Status**: Missing feature. `run-microdroid` subcommand not in current crosvm binary.
**Impact**: Microdroid regression tests cannot run.

---

## 4. Regression Status

| Test | Result |
|------|--------|
| ChromeOS Direct Kernel | PARTIAL PASS — boots, init runs, known frecon hang |
| ChromeOS FW (OVMF) | PARTIAL PASS — firmware detects devices, UEFI errors |
| Microdroid | SKIPPED — feature not implemented |

---

## 5. Upstream WHPX Patches Adopted

From `C:\workspace\single-module\crosvm` (comparison baseline):

| Commit | Description | Status |
|--------|-----------|--------|
| `6dc36202d` | MMIO GP fault injection | Already in our code |
| `89d641d2a` | Fix segment limits | Already in our code |
| `6c7ff51e0` | TscDeadlineTimer capability | Cherry-picked |
| `9ea4c0c4d` | Ignore unsupported MSR writes | Not yet — lower priority |
| `c3a2322f1` | Replace unsound UINT128 conversion | Not yet — lower priority |

---

## 6. File Layout

```
bscp/
├── build_all.bat / build_all.sh              # Main build scripts
├── doc/
│   ├── CHROMIUMOS_WHPX.md                    # ChromeOS on WHPX guide
│   └── WHPX_FIXES_AND_STATUS.md              # This document
├── scripts/
│   ├── run_chromeos.bat                      # Direct kernel boot
│   ├── run_chromeos_fw.bat                   # OVMF firmware boot
│   ├── create_initramfs.sh                   # Builds initramfs CPIO
│   └── rebuild_initramfs.sh                  # Rebuild from backup
└── external/crosvm/                          # crosvm fork with fixes
```
