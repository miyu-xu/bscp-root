# AOSP 17 default: supported reference deployments

AOSP 17 is published and selected by the manifest repository's `main/default.xml`. [Release](https://github.com/miyu-xu/bscp-root/releases/tag/bscp-aosp17-20261102). r14 is a preserved rollback baseline, not the current default.

These packages are reference artifacts for the already admitted deployments, **not fresh-machine installers**. Their original fixed-path, account, hardware and file-identity requirements still apply. Public documentation omits personal accounts, machine names, home directories and LAN addresses. Place the original binding in a private deployment configuration; placeholders do not authorize a new layout or bypass an admission check.

## Platform prerequisites

- Windows: original admitted account and protected runtime root, Windows build 26200, RTX2060 driver 32.0.16.1088 and the matching v1830 Host/Guest closure. Independent CLI reconnect is disabled.
- Linux: original admitted runtime layout, KVM access, recorded kernel 7.0.0-38-generic, RTX2060 and existing NVIDIA ICD/system libraries.
- Mac: original admitted volume layout, Apple M1, macOS 26.5.2/25F84, frozen signed crosvm, Vulkan loader, MoltenVK and OpenSSL dependencies. No dylib paths or security settings are changed by these instructions.

## Source checkout, build and upgrades

For a new source workspace, use `repo init -u https://github.com/miyu-xu/bscp-manifest-repo.git -b main`, then `repo sync`. The default includes `aosp17.xml`, which fixes 1087 official project SHAs and the published BSCP root commit. Do not force-sync, reset or overwrite existing local changes.

The custom Host closures and their platform-specific build records live under `releases/aosp17/source`. Official AOSP sync alone does not build the BSCP Host ports. A build or `--help` success does not establish VM acceptance. Existing source snapshots and runtime archives are subject to the privacy limitations below; do not republish private build logs.

## Read-only installed-deployment verification

The deployment owner must provide a private directory containing the original `BINDING.json`, `profile.json` and `closure.json`. Do not construct new hashes merely to make a check pass. The verifier preserves exact installed-profile/closure hash checks and the production controller's admission checks. It does not create directories, change ACLs, install dependencies or start a VM.

After setting `BSCP_DEPLOYMENT_CONFIG` to that private directory:

```sh
python3 -B releases/aosp17/verify_installed.py --platform linux --deployment-config "$BSCP_DEPLOYMENT_CONFIG"
```

On Mac, use `--platform macos`. On Windows PowerShell:

```powershell
python -B releases/aosp17/verify_installed.py --platform windows --deployment-config $env:BSCP_DEPLOYMENT_CONFIG
```

An unset variable is an error, not a request to discover or create a deployment. These commands are configuration-dependent examples, not copy-and-run installation commands. Use `python -B scripts/microdroid.py --release-info` to inspect the AOSP17 default and `--help` for the exact platform controller syntax. Actual start/stop commands require the already admitted private state root and existing lifecycle procedure; no generic start command is asserted here.

## Downloads, checksums and privacy status

The Release lists exact package names. Verify the original 24 archives with `SHA256SUMS`, and the additional license archive with `license-asset-v1837.json`. The metadata in `releases/default.json` records the corresponding released files and hashes. Do not extract over a live deployment or blindly materialize Guest absolute symlinks on the host.

The initial published attachments and Git history can retain personal build paths; binary debug/source paths require rebuild and affected runtime revalidation, not byte substitution. Corrected public descriptions are not proof that old assets or history have been erased. Immutable historical receipts remain preserved privately; any redacted document is a presentation copy, not a new original test result.

## Acceptance and rollback

Windows v1830 has four normal/healthy-restart and two fault cases. Linux/Mac retain their separate version-specific accepted evidence. Fault tasks remain UNKNOWN when execution is uncertain even if resources are REAPED. Mac ext4 history remains 70 pass / 1 fail. A/H3, real UI and input remain outside this acceptance.

For r14 rollback, use the preserved manifest branch `bscp-android-15.0.0_r14` and exact manifest `89c0365a4dff5c0e52a5386877592a2f6fd45548`, with root `43e4786a48c142b4c8b6f4bfb0db38cd32b20e42`. Do not overwrite active/failed instance disks or replay UNKNOWN requests. `--release r14` selects the retained controller; it does not restore disk state or change a machine's runtime profile.
