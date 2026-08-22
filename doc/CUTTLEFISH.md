# Cuttlefish compatibility path

[简体中文](CUTTLEFISH.zh-CN.md) | English

The Cuttlefish-derived full Android path is an optional compatibility feature. The supported
baseline, documentation order, and release gate remain Microdroid-first.

For the reviewed three-platform implementation matrix, guest disk layout, graphics architecture,
virtual devices, source changes, security limits, and release gates, see
[Full Android / Cuttlefish Cross-Platform Implementation](ANDROID.md).

Use this path for framework-level application compatibility, graphics scanout testing, or device
simulation that cannot be represented by the minimal guest. It requires AOSP product images,
additional host daemons, larger writable storage, and a broader security review.

## Workflow

1. Build or obtain matching AOSP Cuttlefish product artifacts outside this repository.
2. Package immutable source artifacts with `scripts/package_aosp_vm_artifacts.sh`.
3. Create per-instance writable images; never run directly against the release source image.
4. Start only the required host device daemons.
5. Launch with the platform `run_android_*` wrapper and archive the generated logs.

Linux uses `run_android_linux.sh`; macOS uses `run_android_macos.sh`; Windows uses
`run_android_windows_gfxstream_angle.ps1` for the accelerated path. Use each script's help output
as the command-line authority.

Development profiles that disable data encryption must be labeled as such and must not be promoted
as production images. Cuttlefish success does not replace the Microdroid regression gate.
