# Building OVMF with serial debug (DEBUG_ON_SERIAL_PORT)

Use when Debian **release** `OVMF.fd` produces no serial/debugcon output under crosvm.

## Quick start (Windows + WSL)

```powershell
.\scripts\build_ovmf_debug.ps1
# or
.\scripts\fetch_ovmf_debug.ps1 -Build
```

WSL 需 `build-essential nasm python-is-python3`（脚本会检测；也可用 `wsl -u root apt install …`）。

edk2 源码缓存于 WSL `~/.cache/bscp-edk2`（避免 `/mnt/c` 上编译过慢）。

## CI (GitHub Actions)

Workflow [`.github/workflows/build-ovmf-debug.yml`](../.github/workflows/build-ovmf-debug.yml) uploads artifact `OVMF_DEBUG.fd`.

```powershell
.\scripts\fetch_ovmf_debug.ps1 -WorkflowRunId <run_id> -Repo owner/repo
```

## Manual Linux build

```bash
chmod +x scripts/build_ovmf_debug.sh
scripts/build_ovmf_debug.sh
```

Output (path may vary by toolchain):

`Build/OvmfX64/DEBUG_GCC5/FV/OVMF.fd`

## Install on Windows host

Copy to:

`out/dist/firmware/OVMF_DEBUG.fd`

Run:

```powershell
.\scripts\run_crosvm_chromeos_firmware.ps1 -Firmware ovmf-debug -TimeoutSecs 180
```

Expect logs in `guest-serial-num1.txt` (COM1 / 0x3F8).

## Notes

- **DEBUG** build is larger and slower than release.
- Without `DEBUG_ON_SERIAL_PORT`, DEBUG builds still prefer **debugcon 0x402** — keep debugcon serial in the run script.
- CI alternative: build in GitHub Actions and download artifact; no WSL required on the Windows dev machine.
