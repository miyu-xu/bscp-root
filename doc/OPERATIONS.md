# Operations

[简体中文](OPERATIONS.zh-CN.md) | English

## Supported entry points

| Purpose | Linux | macOS | Windows |
| --- | --- | --- | --- |
| Build host tools | `build_all.sh` | `build_all.sh` | `build_all.bat` |
| Microdroid lifecycle | `scripts/vm_linux.sh` | `scripts/vm_macos.sh` | `scripts/vm_windows.ps1` |
| Guest shell/console | `vm_shell_linux.sh` | `vm_shell_macos.sh` | `vm_shell_windows.ps1` |
| Regression | `run_linux_avf_regression.sh` | `run_macos_avf_regression.sh` | `run_windows_avf_regression.ps1` |
| Full Android (optional) | `run_android_linux.sh` | `run_android_macos.sh` | `run_android_windows_gfxstream_angle.ps1` |

Run wrapper help before passing lower-level crosvm arguments. Prefer `validate-prereqs`, then
`info`, then `run-microdroid`; this distinguishes host setup failures from guest boot failures.

## Validation sequence

1. Run the platform prerequisite check.
2. Perform a clean host build when changing native ABI or Rust target settings.
3. Run the Microdroid smoke path without optional devices.
4. Run the platform regression wrapper and inspect its output directory.
5. Enable optional networking, graphics, or full Android only after the baseline passes.

Python helpers can be checked with `python -m compileall scripts tests`; shell scripts with
`bash -n`; PowerShell scripts should be parsed in a clean PowerShell process. `tests/` contains the
root orchestration unit tests.

## Logs and failure handling

Wrappers place logs under `out/dist/logs/` or an explicit `--log-dir`/`-LogDir`. Preserve the launch
command, host capability report, crosvm log, guest console, and manifest revision together. Do not
publish logs until they have been checked for local paths, tokens, guest data, and keys.

Common failure classes are missing virtualization support, architecture-mismatched APEX assets,
mixed Windows ABI dependencies, stale output directories, unavailable TAP devices, and graphics
loader mismatch. Re-run prerequisite validation after changing the host rather than bypassing it.
