# AOSP 17 project update summary (publication preparation)

Status: not yet published. Old values below are measured pre-upgrade engineering snapshots, not a claim that all hosts used one r14 commit. New values are official android17-release pins; BSCP Host modifications remain separate from these upstream commits.

| Project | Observed old Linux SHA | Fixed official A17 SHA |
|---|---|---|
| packages/modules/Virtualization | c35fd6521016b95f66a754dcf0eaad5d1ff959cf | 22f1c9ee92b146e10c8f5e71338618f77ab1631e |
| external/crosvm | ae1c3d0cea78aa4a35510579e40377162b3b50ef | c7dd8d6950912fa5e6fde7ec2d1616e0ddc090f8 |
| external/minijail | 4b9c6d91573e8b3a96609339b46361b5476dd0f9 | 26ff82bac9c77ef25d0a913ee09ed11ccdfdab78 |
| external/rust/rutabaga_gfx | 9c86617123ce261fbbed7b9ffee2e3ef08a4b6bb | 1f181b0b3e4b821459f21fd9025cd4c408c4f9ec |
| hardware/google/gfxstream | f1c050c99dc31512946fd53d708b5cc93d34c62e | b46f0d4b4545def09bdf30651d4bf848b2ba0c34 |
| external/angle | not recorded | 1f4500d3906c24de59eec5b9cda17a2ad2a8ba2d |
| external/MoltenVK | not recorded | not an AOSP manifest project |
| kernel/prebuilts/6.6/arm64 | not recorded | 350899556b7cae26d5981c7055759428aef7598d |
| kernel/prebuilts/6.6/x86_64 | not recorded | b87e9b115da5dd1ed94b423657279c766057d5b3 |
| build/make | not recorded | 5ce6f787337d0223710bf7d4a16dbe6d2a35f777 |
| build/soong | not recorded | 6722dd8833db7482df1a2543ca3fcf67ddf0f7b1 |
| frameworks/native | not recorded | ae266dcb706d083868578cfedce381ef44488a07 |
| system/core | 561a3479c0be7a18212bc636c94944922097cd7d | 545d2487e38192a2ce25040897ced877cf6b4f53 |

## What changed and what was actually demonstrated

- Virtualization / Microdroid: moved to the matching A17 RPC/AIDL/SDK37 interfaces and real 64-byte instance-ID contract. BSCP ports cover desktop APEX discovery, explicit per-caller authorization, foreground service ownership, and platform lifecycle evidence. Both architecture Guests were rebuilt and checked; final platform acceptance covers startup/stop, healthy persistence and failure cleanup. This does not enable protected-VM attestation or arbitrary workloads.
- crosvm: fixed A17 upstream base, with maintained HVF/KVM/WHPX platform ports. Windows work includes typed HANDLE ownership, RPC/Guest pipe and graphics setup; Mac work includes event/reset, HVF register semantics and original-object observation. Linux retains original pidfd ownership. Actual VMM startup, GPU on/off and cleanup were exercised; no performance uplift is measured.
- minijail: official source pin advanced to A17. A new source revision does not demonstrate a new sandbox/security capability on every desktop. No global host security settings were changed by this upgrade.
- rutabaga_gfx: use the A17-matched source/API contract with the selected crosvm and renderer. Historical Mac baseline was b4961c43bef081f485bb2b12374d3f538976f805, unlike the recorded Linux baseline. The tested GPU transport works; unrelated cross-domain/Wayland/display functionality is not claimed.
- gfxstream: A17 backend and matching Guest Vulkan transport. BSCP preserves B timeout/error versus successful retirement semantics, platform loader/display integration and KVM coherent behavior. Real formal-loader enumeration, VADD, offscreen pixels and coherent tests passed in finite profiles; Host device creation alone was never accepted as Guest GPU success.
- ANGLE: official A17 external/angle is pinned in the full source set. The historical standalone Linux ANGLE checkout was abcbf90919e8da3b310971620649cb76138fdf6a, outside the BSCP standard external path. The finite final workload is Vulkan; no new ANGLE GLES capability or real UI/input claim follows merely from source alignment.
- MoltenVK: not a project in the official AOSP lock. The final Mac profile still uses the existing /Volumes/Mark SSD/graphics-native-20261023/runtime/MoltenVK_icd.json and libMoltenVK.dylib from the pre-upgrade runtime, not a newly upgraded MoltenVK release. ICD api_version 1.4.0 is an advertised Vulkan API, not proof of a MoltenVK release tag. Exact library hash/closure must accompany distribution; the original clean source was subsequently recovered at 3afb1b3be8f508cee8ed67aad4bd2f3088826b79 (v1.4.1-55-g3afb1b3b), with seven matching ExternalRevisions dependencies. The original built dylib exactly matches the retained runtime. This is not the official v1.4.2 release binary; see MOLTENVK-EXACT-SOURCE-INDEX.json.
- Kernel: use paired official A17-selected 6.6 arm64/x86_64 prebuilt kernel/module inputs. The guest_os name microdroid_gki-android15-6.6 denotes the kernel family and is not evidence that the Guest system image stayed on Android15. BSCP added build/package wiring for the matching DRM modules; no claim of a wholesale new custom kernel implementation.
- Build/toolchain: official pinned A17 manifest and all1087 project snapshots; matching AIDL generator, normal/noADB Guest builds, AVB/SBOM/module-signature checks. Soong VINTF module metadata registration was repaired rather than skipping SBOM. Rust/compiler/protoc versions are recorded per platform, not flattened into a single toolchain claim.

## Narrow approved Guest configuration changes

Graphics-only labels ro.hardware.vulkan -> build_prop and ro.boot.hardware.gltransport -> bootloader_prop; bootconfig androidboot.hardware.gltransport=virtio-gpu-asg; single-library sphal -> default link for vulkan.ranchu.so. Compiled SELinux policy bytes remain unchanged in the recorded comparisons. No new allow rules, write grants or disabled enforcing.

## Evidence and limitations

UPSTREAM-LOCK.json; linux-inventory.json; mac-inventory.json; delivery-v1792/versions.json; linux-guest-patchset/manifest.json; WINDOWS-FINAL-RESULT-v1777.json; LINUX-V6-EVIDENCE-INDEX-v1420.json; MAC-FINAL-NORMAL-RESULT-v1786.json; MAC-FINAL-VERSION-INVENTORY-v1783.json.

Windows independent CLI reattachment remains disabled, Mac historical ext4 suite retains70pass/1fail, taskUNKNOWN and resourceREAPED remain distinct. No UI/input, A/H3, arbitrary workload, comprehensive isolation or performance guarantee. Upstream change themes beyond the recorded API/build transitions require commit-diff evidence; this summary does not invent changelog features.


### Publication preparation: Windows user-mode networking library

The pre-upgrade Google libslirp binary's embedded version is 4.6.1.15-20e2, but its complete build-source mapping could not be established. It was retained rather than relabelled. The v1830 distribution candidate uses a fresh official libslirp v4.6.1 / GLib 2.70.5 source closure plus proxy-libintl. No performance or network-forwarding improvement is claimed. The affected Windows four normal and two fault cases were rerun; tasks UNKNOWN in fault cases remain UNKNOWN. This distribution preparation change is separate from the upstream AOSP17 project upgrade.
