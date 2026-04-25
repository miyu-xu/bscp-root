# Windows AVF `vm.exe` bring-up

This workspace can boot **unprotected Microdroid** on Windows through the full host chain:

`vm.exe -> libvmclient -> virtmgr.exe -> Binder RPC -> crosvm.exe -> guest`

The current Windows target is **x86_64 + WHPX**. On this path:

- **Unprotected VMs:** supported
- **Protected VMs:** **not** supported

## Current status

The host-side Windows bring-up fixes are in place:

- Windows named-pipe Binder RPC / vsock transport works
- `virtmgr` can create idsig, partitions, and VM instances
- `crosvm` is built with the required `whpx,composite-disk,android-sparse` features
- Windows host capability reporting no longer claims protected VM support on x86_64
- `run-microdroid` and `run-app` have both reached `notifyPayloadReady`

The practical success bar on this host is now:

- `scripts\vm_windows.ps1 -Command run-microdroid`
- `scripts\vm_windows.ps1 -Command run-app`
- `scripts\vm_shell_windows.ps1 -Command start-microdroid`

The remaining platform limit is **protected VM support**, not basic AVF boot:

- Windows `x86_64 + WHPX` currently supports **unprotected** Microdroid
- Protected VMs are still unsupported on this path
- The Windows ADB path now injects `com.android.adbd` into the Microdroid payload, the guest reaches `adbd listening on vsock:5555`, and `vm_shell_windows.ps1 -Command start-microdroid -AutoConnect` now brings `adb connect localhost:<port>` to `device` through an in-process `virtmgr` bridge

## Prerequisites

1. Build the Windows binaries with `build_all.bat`.
2. Ensure `out\dist\com.android.virt` exists.
3. Enable the Windows virtualization features in an elevated PowerShell session:

```powershell
Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V-All,HypervisorPlatform,VirtualMachinePlatform -All
Restart-Computer
```

4. After reboot, verify:

```powershell
scripts\vm_windows.ps1 -Command validate-prereqs
```

Success requires:

- `HypervisorPlatformState = Enabled` or `MicrosoftHyperVAllState = Enabled`
- `HypervisorPresent = True`

## Script entrypoints

Windows wrappers:

| File | Purpose |
| --- | --- |
| `scripts\vm_windows.ps1` | Main PowerShell wrapper for `vm.exe` |
| `scripts\vm_windows.bat` | Batch wrapper that forwards to `vm_windows.ps1` |
| `scripts\vm_shell_windows.ps1` | Windows helper roughly equivalent to `packages\modules\Virtualization\android\vm\vm_shell.sh` |
| `scripts\vm_shell_windows.bat` | Batch wrapper that forwards to `vm_shell_windows.ps1` |
| `scripts\run_microdroid_windows.ps1` | Legacy PowerShell entrypoint that now forwards to `vm_windows.ps1` |
| `scripts\run_microdroid_windows.bat` | Batch wrapper that forwards to `run_microdroid_windows.ps1` |

The older `scripts\run_microdroid_windows.ps1` entrypoint remains and now delegates to `vm_windows.ps1`.

If you launch from **Git Bash** or **cmd.exe**, prefer the `.bat` wrappers. Direct `.ps1` invocation is intended for a PowerShell host.

Windows host APEX artifacts now use the restored AOSP-like tree under `out\dist\apex_dir` directly. `vm_windows.ps1` points `VIRTMGR_APEX_ROOT` at `out\dist\apex_dir\apex`, keeps `com.android.virt` under `out\dist\apex_dir\apex\com.android.virt`, generates `out\dist\apex_dir\apex\apex-info-list.xml`, and extracts only `.capex` payloads into `out\dist\apex_dir\apex\decompressed\*.apex` as a runtime cache.

## Supported commands

`scripts\vm_windows.ps1` supports these values for `-Command`:

| Command | Status on Windows | Notes |
| --- | --- | --- |
| `validate-prereqs` | working | Checks Hyper-V / WHPX readiness |
| `run-microdroid` | working when WHPX is enabled | Validated to `notifyPayloadStarted` and `notifyPayloadReady` |
| `run-app` | working when WHPX is enabled | Validated to `notifyPayloadStarted` and `notifyPayloadReady` with `EmptyPayloadApp.apk` |
| `run` | partially working | The raw-config path boots the VM, but Microdroid raw configs exit with `Failed to load payload metadata`; use `run-microdroid` or `run-app` for payload-ready Microdroid |
| `info` | working | Verified output includes `Assignable devices: []` and `Available OS list: ["microdroid"]` |
| `list` | limited | Exits cleanly, but each CLI invocation currently spawns its own `virtmgr`, so it does not see VMs owned by another invocation |
| `check-feature-enabled` | working | Pure feature query |
| `create-partition` | working | Verified with a 1 MiB image |
| `create-idsig` | working | Verified against `EmptyPayloadApp.apk` |
| `console` | unsupported | `vm.exe` itself rejects this on Windows |

## Common usage

### 1. Validate host readiness

```powershell
scripts\vm_windows.ps1 -Command validate-prereqs
```

### 2. Run unprotected Microdroid

```powershell
scripts\vm_windows.ps1 -Command run-microdroid -KeepTemp -CaptureGuestConsole -CaptureCrosvmStdio
```

Batch form:

```bat
scripts\vm_windows.bat -Command run-microdroid -KeepTemp -CaptureGuestConsole -CaptureCrosvmStdio
```

### 3. Run an APK payload

Default APK:

- `out\dist\com.android.virt\app\EmptyPayloadApp@AP4A.250205.002\EmptyPayloadApp.apk`

Example:

```powershell
scripts\vm_windows.ps1 -Command run-app -KeepTemp -CaptureGuestConsole -CaptureCrosvmStdio
```

Override the payload APK:

```powershell
scripts\vm_windows.ps1 -Command run-app -Apk C:\path\to\payload.apk -PayloadBinaryName MyPayload.so
```

### 4. Run a raw config

Default config:

- `scripts\microdroid_windows_raw.json`

Example:

```powershell
scripts\vm_windows.ps1 -Command run -KeepTemp -CaptureGuestConsole -CaptureCrosvmStdio
```

### 5. Utility commands

```powershell
scripts\vm_windows.ps1 -Command info
scripts\vm_windows.ps1 -Command check-feature-enabled -Feature dice_changes
scripts\vm_windows.ps1 -Command create-partition -PartitionPath $env:TEMP\writable.img -PartitionSize 1048576
scripts\vm_windows.ps1 -Command create-idsig -Apk .\out\dist\com.android.virt\app\EmptyPayloadApp@AP4A.250205.002\EmptyPayloadApp.apk -OutputPath $env:TEMP\app.idsig
```

### 6. `vm_shell.sh`-style startup and ADB connection

Start Microdroid and keep all artifacts under one directory:

```powershell
scripts\vm_shell_windows.ps1 -Command start-microdroid -LogDir .\out\dist\logs\windows-vm-shell-demo
```

Start Microdroid, wait for `notifyPayloadReady`, then ask `virtmgr` to host the local ADB bridge:

```powershell
scripts\vm_shell_windows.ps1 -Command start-microdroid -AutoConnect -NoShell -LogDir .\out\dist\logs\windows-vm-shell-demo
```

This path now uses the built-in Windows code bridge (`vm.exe -> libvmclient -> IVirtualMachine.startHostVsockTcpBridge() -> virtmgr`) rather than an external PowerShell bridge process.

Reconnect ADB to an already running guest:

```powershell
scripts\vm_shell_windows.ps1 -Command connect -LogDir .\out\dist\logs\windows-vm-shell-demo -NoShell
```

## Logs and artifacts

By default, each invocation writes **all runtime artifacts under one run directory**:

- Run root: `out\dist\logs\windows-<command>-<timestamp>\`

Typical layout:

```text
out\dist\logs\windows-run-microdroid-<timestamp>\
  vm-run-microdroid.log
  virtmgr-trace.log
  vmclient-trace.log
  guest-log.txt
  vm-console.txt
  vm-console-in.txt
  work\
  temp\
```

Useful files in the run root:

- `vm-<command>.log`
- `virtmgr-trace.log`
- `vmclient-trace.log`
- `guest-log.txt`
- `vm-console.txt`
- `adb-connect.log` when using `vm_shell_windows.ps1`
- `..\apex_dir\apex\apex-info-list.xml` for the generated Windows host APEX map

`work\` contains command-owned intermediate files such as:

- `instance.img`
- `app.idsig`
- writable partition images

`temp\` becomes the process `TEMP` / `TMP` root for the run. If `-KeepTemp` is set, `virtmgr` temporary files stay under the same run directory, typically:

- `temp\virtmgr\<cid>\`

That keeps `crosvm-stderr.txt` and related files with the rest of the run artifacts instead of scattering them under the global `%TEMP%`.

## Why `run-microdroid` often looks like it has no logs

On Windows, the files that look most obvious are often **not** the ones with the most useful signal:

- `guest-log.txt` is frequently empty
- `vm-console.txt` is frequently empty
- that does **not** mean the guest failed to boot

The authoritative files are usually:

1. `virtmgr-trace.log` — best high-level lifecycle log; this is where to look for `notifyPayloadStarted`, `notifyPayloadReady`, and fatal errors.
2. `vm-run-microdroid.log` — wrapper + `vm.exe` output, argument resolution, and host-side failures.
3. `vmclient-trace.log` — client/service connection details when the control plane itself is failing.
4. `temp\virtmgr\<cid>\crosvm-stderr.txt` — low-level VMM failures such as `Whpx not enabled`.
5. `adb-connect.log` — exact `adb disconnect/connect/get-state/root` outputs for `vm_shell_windows.ps1`.

So the common “no logs” symptom is usually “looking at the wrong files”, not “nothing ran”.

## Recommended debug order

1. Open `virtmgr-trace.log`.
2. If there is no `notifyPayloadReady`, open `vm-run-<command>.log`.
3. If the failure is below the `virtmgr` layer, open `temp\virtmgr\<cid>\crosvm-stderr.txt` with `-KeepTemp`.
4. If you are debugging ADB attach, open `adb-connect.log`.
5. Open `temp\virtmgr\<cid>\guest-virtio-console3.txt` and check whether the guest reached `adbd listening on vsock:5555`.

Useful markers:

- Success markers:
  - `notifyPayloadStarted`
  - `notifyPayloadReady`
- Common failure markers:
  - `Whpx not enabled`
  - `protected VMs not supported on x86_64`
  - `Failed to load payload metadata`
  - `adb connect to localhost:<port> did not succeed`
  - `adb transport for localhost:<port> did not become online`
  - `service adbd not found`
  - `adbd listening on vsock:5555`

## Current ADB status on this runtime

The Windows `vm_shell`-style flow is now implemented:

1. boot Microdroid
2. wait for `notifyPayloadReady`
3. start a localhost TCP -> guest vsock bridge inside `virtmgr`
4. run `adb connect localhost:<port>`

That flow has now been validated far enough to show:

- `out\dist\apex_dir\system\apex\com.android.adbd.capex` is present
- the Windows scripts consume the restored `out\dist\apex_dir` tree directly and only materialize `.capex` payloads under `out\dist\apex_dir\apex\decompressed`
- `out\dist\apex_dir\apex\apex-info-list.xml` is generated so `virtmgr` includes `com.android.adbd` and other system/system_ext APEX metadata in the Microdroid payload
- the guest reaches `adbd listening on vsock:5555`
- guest `adbd` reaches `host-10: read thread spawning` / `host-10: write thread spawning`
- Windows `crosvm` now logs `host-initiated connection established` and `read 258 raw host bytes` for guest vsock:5555
- `adb connect localhost:<port>` reaches `device`

This means the Windows host ADB path is now functional for unprotected Microdroid.

## Persistent `virtmgr`, `vm list`, and `vm console`

Windows now has an opt-in persistent host service mode:

1. start a VM with `-PersistVirtmgr`
2. `vm.exe` returns after the guest reaches `READY`
3. the persistent `virtmgr` instance keeps the VM registered
4. later `vm list` / `vm console <cid>` commands reuse the same service state

Wrapper example:

```powershell
scripts\vm_windows.ps1 -Command run-microdroid -PersistVirtmgr
scripts\vm_windows.ps1 -Command list -PersistVirtmgr
scripts\vm_windows.ps1 -Command console -PersistVirtmgr -Cid 2048 --read-only --timeout-secs 3
scripts\vm_windows.ps1 -Command service-status -PersistVirtmgr
scripts\vm_windows.ps1 -Command stop-service -PersistVirtmgr
```

Important notes:

- `vm console` on Windows is a **file-backed attach model**, not a raw TTY/microcom equivalent.
- `vm list` / `vm console` only have persistent semantics when `VIRTMGR_SERVICE_DIR` is set (the wrappers do this for `-PersistVirtmgr`).
- `service-status` reads `virtmgr-service.state` and `virtmgr-trace.log` from the selected service root.

## Regression scripts

The default host regression path is now:

```powershell
scripts\run_windows_avf_regression.ps1
```

That default run validates:

1. `validate-prereqs`
2. `info`
3. `create-partition`
4. `create-idsig`
5. baseline `run-microdroid`
6. persistent `run-microdroid`
7. `vm list`
8. `vm console`
9. marker checks for the above

Optional extra coverage:

```powershell
scripts\run_windows_avf_regression.ps1 -IncludeRunApp
scripts\run_windows_avf_regression.ps1 -IncludeAdbScenario
```

## Known limitations

1. **Protected VMs are unsupported** on the current Windows `x86_64 + WHPX` path.
2. The Windows persistent service model is still a **host-side approximation** of Android's long-lived system service; it is opt-in and file/state based.
3. `vm console` is functional on Windows, but it is a **file-backed console attach** rather than a raw interactive TTY.
4. The default raw-config sample for Microdroid (`scripts\microdroid_windows_raw.json`) reaches the raw VM path, but exits with `Failed to load payload metadata`; for successful Microdroid payload startup, use `run-microdroid` or `run-app`.
5. Current builds still emit Binder/RPC debug output, so `info` / `list` are functional but noisy.
6. If `validate-prereqs` reports `HypervisorPresent = False`, all VM start commands will fail with `Whpx not enabled` until after reboot.
7. `vm_shell_windows.ps1 -AutoConnect` depends on the restored `out\dist\apex_dir` tree plus the generated `out\dist\apex_dir\apex\apex-info-list.xml` and `out\dist\apex_dir\apex\decompressed\*.apex` cache; if you manually replace `out\dist`, rerun `scripts\vm_windows.ps1` once so those runtime artifacts are regenerated.

## Previously validated proof points

The successful unprotected boot path has already reached:

- `virtmgr: idsig after output_len=4213`
- `microdroid_manager::verify: payload verification successful`
- `virtmgr: notifyPayloadStarted`
- `virtmgr: notifyPayloadReady`
- `vm_payload: Notified host payload ready successfully`

Additional validated Windows results:

- `vm info` exits and prints `Assignable devices: []` and `Available OS list: ["microdroid"]`
- `vm create-partition` creates a writable image
- `vm create-idsig` creates a valid non-empty idsig
- `vm list` works against a persistent Windows `virtmgr` service and shows the live VM CID / temp dir / host console metadata
- `vm console` works in persistent mode and resolves the registered Windows file-backed console paths
- `vm run` with `scripts\microdroid_windows_raw.json` reaches the raw-config VM path, then exits with `Failed to load payload metadata`

That is the expected success bar once the host hypervisor is enabled again.
