# AOSP 17 — limited-deployment reference release

This release provides the AOSP 17 source baseline and reference runtime packages for the existing validated Windows, Linux and Mac deployments. These packages are **not fresh-machine installers**. They do not create accounts, private runtime directories, ACLs or driver permissions, and must not be extracted over live instances. Guest absolute symlinks are metadata only and must not be blindly materialized on the host.

## Required deployment conditions

- Windows: existing MARK\miyu1 account and approved `E:\bscp\out\aosp17-windows-formal-v1461\runtime-private-v1550` layout; Host `assets\host-source-closed-v1828`, state `run-gpu-v1828`, matching Guest assets. Validated Windows build 26200, RTX2060 driver 32.0.16.1088. Independent CLI reconnect is disabled.
- Linux: existing mark account with KVM access, `/home/mark/bscp/out/a17l/v6/s`, matching Host/Guest closure under `/home/mark/bscp/out/a17l/v6/dist/linux/bin`; recorded kernel 7.0.0-38-generic and RTX2060 with the existing NVIDIA ICD/system libraries.
- Mac: existing huangxiaoling account, Apple M1, macOS 26.5.2/25F84, `/Volumes/Mark SSD` layout, state `a17r/n32-v1786/s`, Host `a17r/v32/virtmgr`, frozen signed crosvm, Vulkan loader, MoltenVK and OpenSSL dependencies. Exact absolute paths and read-only verification commands are in the deployment document below.

[Full deployment conditions and commands](https://github.com/miyu-xu/bscp-root/blob/c5e797a0c28948fc9f9b4a5df0155261ea0be078/doc/aosp17/SUPPORTED-DEPLOYMENT-CONDITIONS-v1830.md)

## Validation and provenance

Windows v1830 passed four normal/healthy-restart cases and two fault-recovery cases with the rebuilt source-closed libslirp/GLib dependency. Linux/Mac retained their separately recorded accepted binaries; later read-only checks are not new VM runs. Fault tasks remain UNKNOWN even when resources are REAPED. Historical failures, disks and claims remain preserved. Mac ext4 history remains 70 pass / 1 fail. A/H3 and real UI/input are outside this release validation.

Source preparation commit: `c5e797a0c28948fc9f9b4a5df0155261ea0be078` ([three-platform CI](https://github.com/miyu-xu/bscp-root/actions/runs/41486111329)). The manifest preparation pins the tested source commit `814a66ede631002a791aa8b51590d8623ea4c3f3`; the later preparation commit adds deployment documentation. CI is source/provenance validation, not VM execution.

Use SHA256SUMS for the original 24 archives and license-asset-v1837.json for the additional root-license archive. All 25 archives and accompanying metadata were downloaded and SHA256-verified. BSCP-original material is source-available under PolyForm Noncommercial 1.0.0 unless otherwise noted; it is not represented as OSI-approved open source. See the attached root license policy and commercial licensing notice. Source archives and the source/notices/deployment ZIP accompany the runtime packages. Retained MoltenVK is development commit `3afb1b3be8f508cee8ed67aad4bd2f3088826b79`, not a substituted v1.4.2 binary. New libslirp is v4.6.1 with GLib 2.70.5. Preserve each dependency's original license; the wrapper's MIT notice does not relicense all DLL dependencies. The repository license policy also applies.

## Rollback

r14 code: `43e4786a48c142b4c8b6f4bfb0db38cd32b20e42`; r14 manifest: `89c0365a4dff5c0e52a5386877592a2f6fd45548`, retained on `bscp-android-15.0.0_r14`. Do not overwrite active or failed instance state or replay UNKNOWN business requests during rollback.

Release assets were downloaded and SHA256-verified before default-entry promotion. The pinned manifest and releases/default.json identify the published source and exact asset set.
