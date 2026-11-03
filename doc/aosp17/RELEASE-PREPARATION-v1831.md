> Historical preparation record, not current release status. AOSP 17 is now published and the default. See [current deployment guide](SUPPORTED-DEPLOYMENT-CONDITIONS-v1830.md). Personal environment fields below are redacted presentation values, not executable configuration.

# AOSP17 publication preparation — scope choice pending

No assets have been uploaded and no default has been promoted. The user is
choosing between the existing supported deployment scope and further portable
installation work. This note records readiness within the former scope only.

## Installation conditions and commands

See [Supported deployment conditions](SUPPORTED-DEPLOYMENT-CONDITIONS-v1830.md).
It names the exact Windows private root, Linux KVM/NVIDIA prerequisites and
Mac volume/loader/OpenSSL dependencies, and provides read-only production check
commands. The archives are reference deployment artifacts, not fresh-machine
installers. Do not extract them over live instances or automatically materialize
Guest absolute symlinks on the host.

## Candidate runtime assets

| Platform | File | Bytes | SHA256 |
|---|---|---:|---|
| Windows x86_64 | bscp-aosp17-windows-x86_64-source-closed-v1830.zip | 310438830 | f9a4e4563b6ea90d17a1a982447b2d988521bda2583bc00c50a70f441041068c |
| Linux x86_64 | bscp-aosp17-linux-x86_64-frozen.tar.gz | 224355893 | 6fc1d93a800fb1e459734ded1061ec8c2d0c877166c16432f6b2021963343c46 |
| Mac arm64 | bscp-aosp17-macos-arm64-frozen.tar.gz | 150858340 | a9ddc29219f1eb05e609dfd05ead7f4a2d99ed3e8b0046a11d3a7d955ee8e0a0 |

The old Windows frozen ZIP is retained historical material and is not this
candidate. The replacement uses libslirp4.6.1/GLib2.70.5 and its companion
libintl DLL. Both DLLs and the unchanged Host/renderer/Guest were tested through
four normal and two fault cases. Fault tasks remain UNKNOWN, resources REAPED,
with disks and claims preserved. See WINDOWS-FINAL-RESULT-v1830.json.

Linux/Mac binaries were not replaced in this round. Existing production
GPU-on/off checks were rerun successfully, without new VMs. Their historical
version-specific runtime evidence remains the authority; no results were
relabelled to a later untested version.

## Source and notice correspondence

`FINAL-ASSETS-local-v1831.json`, `FINAL-ASSETS-linux-v1831.json` and
`FINAL-ASSETS-macos-v1831.json` record 23 current archive hash checks. The four
Host/crosvm source archives were additionally checked against all5,193 indexed
source files. Native/renderer and MoltenVK archives retain their original
source-binding qualifications; current snapshots are not retroactive build
attestations.

The matching libslirp source archive SHA256 is
`01da5677eafb2017b68607f2ccb1ce19071b9c72474e72c956b72de70f7059c9`.
Distribute its `INDEX.json`, README, build recipes, 12 notice texts and the exact
libslirp COPYRIGHT in `additional-notices/` together. The COPYRIGHT was already
inside the sealed source archive; `NOTICE-SUPPLEMENT.json` exposes it separately
without rewriting that archive or its original index.

MoltenVK is the retained development commit
`3afb1b3be8f508cee8ed67aad4bd2f3088826b79`, with seven fixed external revisions.
Use the eight archives and14 notices in MOLTENVK-EXACT-SOURCE-INDEX.json.
The old `MoltenVK-1.4.2` comparison/notice directory is historical and must not
be described as this runtime's release provenance. The libslirp-rs MIT notice
applies to that wrapper, not to the entire new libslirp/GLib DLL.

The check also verified390 existing notice-index entries and27 exact additional
notice files. This is a file identity check, not permission to apply one license
to unrelated components. Original upstream texts and the repository license
policy must accompany distribution; source/toolchain prerequisites stay explicit.

## Validation and rollback

Source commit814a66ede631002a791aa8b51590d8623ea4c3f3 completed all three CI jobs
successfully: https://github.com/miyu-xu/bscp-root/actions/runs/41484422286.
The first new snapshot check failure was corrected by refreshing the current
document/controller index; no historical failure evidence was edited.

r14 code43e4786a48c142b4c8b6f4bfb0db38cd32b20e42 and
manifest89c0365a4dff5c0e52a5386877592a2f6fd45548 remain intact. Existing default
entries still select the old baseline while publication is pending. Do not
restore failed instance disks automatically or replay unknown business requests.
Windows independent CLI reconnect remains disabled; Mac ext4 history remains
70pass/1fail. A/H3 and real UI/input remain outside this release validation.
