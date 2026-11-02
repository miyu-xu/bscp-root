# Explicit supported deployment conditions

## Recommended publication description

Publish the AOSP17 source/manifest baseline and clearly labelled reference
runtime archives for the three admitted deployments. Do not advertise the
archives as arbitrary-machine installers. No new universal installer is
required by the original workspace-build instructions, but an arbitrary-root
installation claim would require more implementation and validation.

The archives do not contain existing instance disks, claims, credentials or
security settings. They must not be extracted over a live deployment. Linux/Mac
Guest symlinks are represented as metadata, not blindly materialized on the
host. These archives alone are not a tested fresh-machine restoration command.

## Windows existing deployment

- Existing account MARK\miyu1 and already approved private root:
  `E:\bscp\out\aosp17-windows-formal-v1461\runtime-private-v1550`.
- New Host bundle: `assets\host-source-closed-v1828`; state:
  `run-gpu-v1828`. Existing matching Guest assets must remain present.
- Recorded OS: Windows build 26200; GPU provider RTX2060 driver 32.0.16.1088.
  Exact provider paths/hashes are checked by the frozen closure.
- No new ACL operation is required for this existing deployment. An absent
  private root is not permission to recreate it with guessed permissions.
- Windows independent CLI reconnect remains disabled.

From a checkout of the publication commit, this read-only command checks the
existing installation, without starting a VM:

```powershell
python -B releases/aosp17/windows-v1830/scripts/microdroid.py --platform windows --state-root E:/bscp/out/aosp17-windows-formal-v1461/runtime-private-v1550/run-gpu-v1828 check --gpu on
```

Repeat with `--gpu off` for the other configuration. The actual six-case VM
results are in `WINDOWS-FINAL-RESULT-v1830.json`; `check` is not a new VM test.

## Linux existing deployment

- Existing mark account, KVM access and state
  `/home/mark/bscp/out/a17l/v6/s`.
- Host files under `/home/mark/bscp/out/a17l/v6/dist/linux/bin` and exact matching
  Guest/profile closure must remain present.
- Recorded kernel: 7.0.0-38-generic; RTX2060 with the existing
  `/usr/share/vulkan/icd.d/nvidia_icd.json` and system dynamic libraries.
  No bundled replacement GPU driver or new device permission is supplied.
- Current production checks for GPU on/off returned exit0 again in this round.
  The previously accepted Linux VM version was unchanged.

Run on that Linux machine from the publication checkout:

```sh
python3 -B releases/aosp17/verify_installed.py --platform linux
```

## Mac existing deployment

- Existing huangxiaoling account, Apple M1, recorded macOS26.5.2/25F84 and
  `/Volumes/Mark SSD` layout.
- State `/Volumes/Mark SSD/a17r/n32-v1786/s`; Host
  `/Volumes/Mark SSD/a17r/v32/virtmgr`; crosvm
  `/Volumes/Mark SSD/aosp17-host-port-v1/runtime-signed-v1001/crosvm`.
- Existing Vulkan loader path
  `/Volumes/Mark SSD/aosp17-vulkan-loader-v1062/runtime-v1063/lib/libvulkan.1.4.350.dylib`,
  MoltenVK ICD under `graphics-native-20261023/runtime`, existing OpenSSL and
  all exact closure files are prerequisites. The recovered MoltenVK source is
  the retained development commit, not a newly substituted release binary.
- Current production checks for GPU on/off returned exit0 again. They do not
  turn historical GPU-off evidence into a new-version run or erase the ext4
  suite's recorded 70pass/1fail boundary.

Run on that Mac from the publication checkout:

```sh
python3 -B releases/aosp17/verify_installed.py --platform macos
```

## Remaining publication action

The source/notice and new Windows runtime gates are closed for this explicitly
bounded scope. The exact source commit 814a66ede631002a791aa8b51590d8623ea4c3f3 passed Linux, Windows and Mac provenance CI (run41484422286). This is source validation, not a new VM run.
Release asset upload needs a supported live authenticated browser operation;
the old login screenshot cannot authorize stale-coordinate interaction.
Default promotion must follow actual asset/reference verification. Neither
GitHub default-branch settings nor machine running profiles are changed by
these preparation commits.
