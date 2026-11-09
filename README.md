# BSCP — AOSP 17

AOSP 17 is the default source baseline for the Microdroid compute workflow on Linux/KVM, macOS/HVF and Windows/WHPX. The published binaries are **limited-deployment reference packages**, not fresh-machine installers. Keep existing private admission configuration and matching Host/Guest artifacts; an arbitrary new account or directory is not supported by renaming paths.

- [Start here](docs/README.md): bilingual documentation and current/historical boundaries.
- [Download, verify and roll back](docs/RELEASE.md).
- [Supported deployment](docs/DEPLOYMENT.md) and [operations](docs/OPERATIONS.md).
- [Architecture](docs/ARCHITECTURE.md), [source/build](docs/BUILD.md), [troubleshooting](docs/TROUBLESHOOTING.md).

## Current capability boundary

The separately recorded three-platform acceptance covers GPU on/off, VADD, offscreen/coherent, healthy same-disk restart and bounded timeout/disconnect recovery. UNKNOWN business results remain UNKNOWN even after resources are REAPED. Windows independent CLI reconnect remains disabled. Mac historical ext4 results remain 70 pass / 1 fail. A/H3, real display/input and arbitrary-machine deployment are not part of this release acceptance. A build or source CI pass is not a VM test.

`main` selects AOSP 17; `bscp-android-15.0.0_r14` remains the rollback baseline. Do not reset a populated workspace or overwrite instance disks to change versions.

Actual AOSP17 Guest Vulkan offscreen rendering has been verified on Windows/Linux RTX 2060 and Mac M1: the finite PIXEL test read back 256 pixels with zero mismatches. See [versioned evidence and limits](docs/MICRODROID.md#8-verified-microdroid-vulkan-offscreen-rendering).

## Source and verification

Use a new workspace for this command; it initializes metadata and synchronization downloads source:

```sh
repo init -u https://github.com/miyu-xu/bscp-manifest-repo.git -b main
repo sync
```

This is not a runtime installation command. See the pinned [manifest](https://github.com/miyu-xu/bscp-manifest-repo/blob/main/aosp17.xml), [release selector](releases/default.json), and [build guide](docs/BUILD.md). Keep the resolved manifest commit for reproduction.

## License and contribution

BSCP-original material is source-available under [PolyForm Noncommercial 1.0.0](LICENSE), not represented as OSI-approved open source. Commercial use requires [separate authorization](COMMERCIAL_LICENSING.md). Third-party components retain their own licenses; see [license policy](LICENSE_POLICY.md). Read [contributing](CONTRIBUTING.md) and [security](SECURITY.md).

[简体中文](README.zh-CN.md)

## Measured inference and candidate fixes

[Multi-platform llama.cpp results](docs/LLAMA-PERFORMANCE.md) separate historical same-machine Host/Guest pairs, latest ASG regressions and native Android. Actual Guest Vulkan inference has run on all three desktop hosts; strict numerical failures, timing variability and untested phone Microdroid GPU remain explicit.

[Reproducible isolated source](experiments/aosp17-sync-asg/README.md) packages conservative synchronization/lifecycle and ASG fixes. It does not enable experimental capabilities or replace frozen release binaries.
