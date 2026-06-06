# WHPX firmware boot IO fix (2026-05-29)

## Symptom

OVMF/SeaBIOS boot on Windows/WHPX flooded logs with:

```
failed to handle io: 内存资源不足，无法处理此命令。 (os error 8)
```

`guest-serial-num1.txt` stayed 0 bytes; `crosvm-stderr.txt` could exceed 100MB in seconds.

## Classification (instrumented build)

Minimal repro: `scripts/run_crosvm_ovmf_min_repro.ps1` with `CROSWVM_WHPX_IO_DEBUG=1`.

| Field | Value |
|-------|--------|
| Port | `0x511` (PIO IN, size=1) |
| RIP | `0x8362e0` (OVMF BIOS range) |
| `as_u32` | 8 |
| `mmio_cb_fail` | **1** |
| `io_cb_fail` | 0 |
| `insn_len` | 16 (string I/O style instruction) |

**Root cause:** `WHvEmulatorTryIoEmulation` invokes `memory_cb` for guest-memory operands in the same instruction (e.g. `ins`). `InstructionEmulatorContext` only set `handle_io`, so `memory_cb` returned `E_UNEXPECTED` → `MemoryCallbackFailed` → emulation loop.

This is **not** a ChromeOS image, virtmgr, or missing device on port `0x511` alone.

## Fix

1. **`WhpxVcpu::handle_io_with_mmio`** — pass both PIO and MMIO handlers into the WHPX instruction emulator ([`external/crosvm/hypervisor/src/whpx/vcpu.rs`](../external/crosvm/hypervisor/src/whpx/vcpu.rs)).

2. **Windows `vcpu_loop`** — on `VcpuExit::Io`, downcast to `WhpxVcpu` and call `handle_io_with_mmio` with the same MMIO path used for `VcpuExit::Mmio` ([`external/crosvm/src/sys/windows/run_vcpu.rs`](../external/crosvm/src/sys/windows/run_vcpu.rs)).

3. **Diagnostics** — `CROSWVM_WHPX_IO_DEBUG=1` enables rate-limited `whpx io emulation failed` lines with port/RIP/status bitfield (max 128).

## Verification

| Test | Before fix | After fix |
|------|------------|-----------|
| OVMF-only, 20s | ~21MB stderr, 138k+ IO errors | 113 bytes stderr, **0** IO errors |
| OVMF + 10GB disk, 120s | IO storm | 113 bytes stderr, **0** IO errors |
| Serial output | 0 bytes | Still 0 bytes (separate issue) |

Rebuild/deploy:

```powershell
cd external\crosvm
rustup run stable cargo build --release -p crosvm --target x86_64-pc-windows-gnu --features "whpx,composite-disk,android-sparse,gpu"
Copy-Item ..\..\out\target\x86_64-pc-windows-gnu\release\crosvm.exe ..\..\out\dist\windows\bin\
```

## Remaining work

- **No firmware serial yet** — OVMF may not use `ttyS0` at 0x3F8 in this configuration, or boot is waiting on display/TPM/other devices.
- **Next:** OVMF debug build (`DEBUG_ON_SERIAL_PORT`), verify boot order / `bootindex`, then gfxstream when console appears.
- **Linux/KVM** — same `OVMF.fd` + disk command on Linux host still recommended to confirm image boots independent of WHPX.

## Helper scripts

| Script | Purpose |
|--------|---------|
| `scripts/run_crosvm_ovmf_min_repro.ps1` | OVMF-only WHPX repro |
| `scripts/analyze_whpx_io_log.ps1` | Parse diagnostic lines |
| `scripts/run_crosvm_chromeos_firmware.ps1` | Direct crosvm OVMF+disk |
| `scripts/run_chromeos_firmware.ps1` | virtmgr path |
