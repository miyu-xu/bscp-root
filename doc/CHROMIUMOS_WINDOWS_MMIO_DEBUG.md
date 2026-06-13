# ChromiumOS Windows Boot — mmio@0 Debug Notes

Date: 2026-05-29

## Symptom

After `crosvm` loads the extracted ChromeOS bzImage, ~17–18s later the vCPU enters a loop:

1. `mmio read/write failed: 0` (GPA 0x0)
2. `failed to handle mmio: … (os error 2)` — WHPX emulator status / `ENOENT`
3. `unexpected vcpu.run return value: UnrecoverableException`

Guest serial (`ttyS0` / `hvc0`) stays **0 bytes**. `crosvm-stderr.txt` can grow to hundreds of MB.

## What is NOT the cause

| Hypothesis | Result |
|------------|--------|
| virtmgr / fstab injection | Fixed in `aidl.rs` (`android_fstab` only for App VM). ChromiumOS raw config no longer gets `--android-fstab`. |
| Hyper-V disabled | `Microsoft-Hyper-V-All=Enabled`, `validate-prereqs` passes. Same failure with/without `-SkipHypervisorCheck`. |
| Named pipe console input | Reproduces with direct `crosvm run` (no virtmgr, no input pipe). |
| Wrong `root=/dev/vda3` only | Full vboot dm-verity cmdline still fails. |
| Disk required for crash | **Kernel-only** (no `--block`) still crashes at ~17s with 4GB RAM. |
| RAM too small (2GB) | 2GB → probes `guest memory read failed at 0x80000000`. 4GB+ → mmio loop (different phase, still no console). |

## Control experiment: Microdroid kernel

Direct `crosvm run` with `microdroid_kernel` + `microdroid_initrd` (no fstab):

- **3785 bytes** virtio-console output (normal dmesg)
- No mmio storm

→ **crosvm + WHPX on this host is healthy.** Failure is **ChromeOS-kernel-specific**.

## ChromeOS kernel image

- KERN-A partition starts with `CHROMEOS` vboot header (not raw bzImage).
- `scripts/extract_cros_kernel.ps1` finds inner bzImage at partition offset `0xAA0000`; extracted file has valid `55 AA` + `HdrS`.
- Passing **full KERN-A** (64 MiB) as kernel → `bad kernel header signature` (crosvm expects bare bzImage).
- Stripped bzImage loads (`Loaded bzImage kernel`) but guest never prints to serial.

## crosvm command-line diff (virtmgr)

| | ChromiumOS raw | Microdroid app |
|--|----------------|----------------|
| `--android-fstab` | No (after fix) | Yes |
| `--initrd` | No | Yes |
| `--params` | vboot dm-verity cmdline | `androidboot.*` |
| `--block` | Single 10GB GPT image | 3× composite images |
| `--mem` | 4096 | 256 |

## Interpretation

The ChromeOS **amd64-generic** kernel is built for the ChromeOS boot chain (vboot / depthcharge / firmware handoff). Direct bzImage boot via crosvm — as used for Microdroid and generic Linux — gets far enough to load the image but the guest enters a bad CPU state ~17s later (after TSC calibration), without ever writing to the serial console.

This matches Android ferrochrome using **disk-only** `vm_config.json` (no explicit `kernel` field): the Android `VmLauncherApp` stack likely supplies kernel extraction + platform setup that the Windows `vm.exe run -Config` JSON path does not implement.

## Reproduction (minimal)

```powershell
$env:PATH = "out\dist\windows\bin;$env:PATH"
# Use job + call operator so --params with spaces is one argument (Start-Process splits on spaces).
$job = Start-Job {
  & out\dist\windows\bin\crosvm.exe --log-level warn run --disable-sandbox `
    --cid 4096 --mem 4096 --cpus 1 --no-usb `
    --serial type=file,path=ser.txt,hardware=serial,num=1,earlycon=true `
    --params "console=ttyS0,115200n8 loglevel=7" `
    out\dist\img\amd64-generic_kernel.bin
}
Wait-Job $job -Timeout 30
```

Helper script: `scripts/run_crosvm_chromeos_direct.ps1` (needs job/`&` invocation for params).

## Recommended next steps

1. **OVMF + disk (priority)** — `scripts/chromiumos_windows_firmware.json` + `scripts/run_chromeos_firmware.ps1`. virtmgr trace confirms `--bios OVMF.fd` + `--block` with **no kernel/params/fstab**. Blocked on WHPX firmware IO emulation (`os error 8`); see `doc/CHROMIUMOS_FIRMWARE.md`.
2. ~~**SeaBIOS / depthcharge path**~~ — same WHPX blocker as OVMF.
3. **Port Android kernel extraction** — fallback only if firmware path is fixed on Linux but Windows WHPX remains broken.
4. **Microdroid fstab on Windows**: separate issue — `fstab.microdroid` triggers `invalid fstab format` in crosvm FDT parser; needs Windows-compatible fstab or crosvm fix.

## Artifacts

- Logs: `out/dist/logs/cros-attempt-4/`, `cros-direct-microdroid/`, `cros-mem-4g/`
- Kernel: `out/dist/img/amd64-generic_kernel.bin`
- Cmdline: `out/dist/img/amd64-generic.cmdline.resolved.txt`
- Config: `scripts/chromiumos_windows_raw.json`
