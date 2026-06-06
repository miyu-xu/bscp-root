# Prebuilt Firmware

OVMF (Open Virtual Machine Firmware) for x86_64 virtual machines.

## Files

| File | Size | Description |
|------|------|-------------|
| `OVMF.fd` | 4MB | Release build, stripped debug output. Use for production. |
| `OVMF_DEBUG.fd` | 4MB | Debug build with verbose PEI/DXE logs. Use for debugging. |
| `OVMF_CODE.fd` | 3.5MB | Split format: code only (requires OVMF_VARS.fd for NVRAM) |
| `OVMF_VARS.fd` | 540KB | Split format: variable store template |

## Usage

```bash
# Release build
crosvm run --bios prebuilts/firmware/OVMF.fd ...

# Debug build (verbose boot logs)
crosvm run --bios prebuilts/firmware/OVMF_DEBUG.fd ...

# Split format (QEMU-compatible)
crosvm run --bios prebuilts/firmware/OVMF_CODE.fd ...
```

## Compatibility

- Works with WHPX, KVM, HAXM hypervisors
- Supports x86_64 guests
- Supports SMP (tested with --cpus 2 on WHPX)
- UEFI 2.7 compliant

## Build Info

- EDK2 stable202408
- Target: OVMF X64
- Toolchain: GCC5
- Features: VirtioBlk, VirtioNet, VirtioSerial, NVMe, TPM2
