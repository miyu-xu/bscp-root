# AOSP17 Microdroid candidate: reproduce, accept, and roll back

This handoff preserves the tested foreground lifecycle. It does not publish a new branch, switch machine defaults, migrate old VM disks, or enable independent Windows persistent CLI reattachment. Versions and original source/build/lock/tool/runtime evidence are in `versions.json`. Old reports remain historical; use `STATUS.json` for delivery progress and `versions.json` for version-scoped runtime coverage. The v1786 report remains historical; its earlier source/todo status is superseded by this handoff.

## Machines and fixed runtime routes

| Platform | Native entry | State/profile |
|---|---|---|
| Linux mark@192.168.1.21 | `/home/mark/bscp/out/a17l/v6/scripts/microdroid.py` | `/home/mark/bscp/out/a17l/v6/s` |
| Mac huangxiaoling@192.168.1.14 | `/Volumes/Mark SSD/a17r/v32/scripts/microdroid.py` | `/Volumes/Mark SSD/a17r/n32-v1786/s` |
| Windows MARK | `../windows-headless-gpu-v1774/scripts/microdroid.py` | `E:/bscp/out/aosp17-windows-formal-v1461/runtime-private-v1550/run-gpu-v1774` |

The frozen profile is the authority for the complete paths, normal initrd, noADB, SDK37, GPU mode, loader/renderer/ICD, APEX, APK, Host/crosvm and all dependency hashes. Do not substitute the current AOSP output tree for a frozen graphics APEX, a nongraphics output, or an older r14 library. Validate `profile.instance_root == state-root/instances`, source/closure hashes and recursive native dependencies before creating a new instance.

The numbered acceptance drivers are deliberately one-shot. Existing evidence/output paths and used instance names must fail rather than be overwritten. To repeat acceptance, prepare one new exclusive state root and driver copy, project only that root into any exact Windows allowlist, reseal all changed config/entry bytes, run the entry/receipt gate tests, then execute serially. Never rerun a historical failure or edit its result to PASS. A new run is new evidence and not byte-reproducibility proof.

## Actual build recipes and required inputs

Use original pinned source/native/tool manifests first. Build into a new exclusive output directory and retain stdout/stderr, command/environment identity, source-before/after hashes, locked dependency hashes and final binary hash. Keep `CARGO_NET_OFFLINE=true` when the captured cache is complete. A missing cache entry is a dependency issue, not permission to silently change the lock. Never rerun the historical migration scripts: many intentionally modify source once.

**Windows crosvm**

- Source: `E:/bscp/out/aosp17-windows-crosvm-v1462/guest-rpc-bridge-v1745/source-v1773`.
- Rust1.94.0 GNU; approved isolated protoc3.21.12, SHA256 `6edb2a02f51f65b3e2e8086a643aac5254b11d71d7d9f55387b475af696c76e5`; lock `859cb62ddcd8207a0af12a2a48b216ff8f2e532130410bb0ef9da5cf471f7d36`.
- Exact successful command/environment/source and dependency manifests: `E:/bscp/out/aosp17-windows-crosvm-v1462/guest-rpc-bridge-v1745/crosvm-full-build-v1773/`.
- Reusable existing build runner: `../windows-gpu-setup-v1770/build_headless.py --protoc-sha256 6edb2a02f51f65b3e2e8086a643aac5254b11d71d7d9f55387b475af696c76e5 --evidence-name crosvm-full-build-vNNNN` (replace NNNN with a new numeric version; existing output refuses). It uses the frozen source and official GPU backend, `--release --offline --locked -j1 --target x86_64-pc-windows-gnu --no-default-features --features whpx,composite-disk,android-sparse,net,slirp,balloon,gpu,gfxstream,vulkan_display,vulkano`.

**Windows virtmgr**

Source manifest: `../windows-host-thread-name-v1753/virtmgr/Cargo.toml`. Build command: pinned Rust1.94.0 GNU `cargo build --locked --offline --manifest-path <that manifest> --target x86_64-pc-windows-gnu -j1 --features desktop-gpu --message-format=json-render-diagnostics`. The exact successful native/Binder platform variables and dependency seal are documented by `../windows-host-desktop-gpu-v1765/build_host.py` and `HOST-DEPENDENCIES-SEALED.json`; use its environment assembly with a new target/output, not its used one-shot output. The final receipt is `E:/bscp/out/aosp17-windows-formal-v1461/desktop-gpu-v1765-output/COMPLETE.json`. Native6 dependencies and vm v1752 must remain paired; no r14 DLL substitution.

**Linux virtmgr**

Source: `/home/mark/bscp/out/aosp17-linux-host-v1228/formal-v1293/sources/aosp17-endpoint-retirement-v1253/virtmgr`. Pinned Rust tool path `/home/mark/aosp/prebuilts/rust-toolchain/linux-x86/1.93.1/bin`; Clang `clang-r584948`. Final actual build command in `../BUILD-GPU-v1393.stdout`: `cargo build --locked --offline --manifest-path <source>/Cargo.toml --target x86_64-unknown-linux-gnu --features desktop-gpu -j2`. Binder native source/libs, libcap include/library paths, bindgen resource path and LD_LIBRARY_PATH must match `../build_gpu_v1393.py`'s recorded environment. Use a fresh CARGO_TARGET_DIR; do not execute its source-edit phase again. `../LINUX-FINAL-SOURCE-v1413.stdout` and the v1420 evidence index bind the actual accepted runtime. Renderer KVM coherent adaptation and crosvm have separate receipts; their identities cannot be inferred from virtmgr compilation.

**Mac virtmgr**

Source: `/Volumes/Mark SSD/aosp17-endpoint-retirement-v1253/virtmgr`. Exact successful recipe and filtered tests are in `../build_endpoint_v1253.py` and `../ENDPOINT-BUILD-v1253.stdout`; the build is `cargo +stable build --offline --locked --manifest-path <source>/Cargo.toml --bin virtmgr -j2 --features desktop-gpu`. Preserve the actual tool binary identity and locked cache from its original evidence rather than assuming today's `stable` is the same compiler. Environment binds official A17 Binder native source and native-host/build plus existing OpenSSL. Use a fresh target/output; do not rerun its exclusive source-copy/patch phase. The signed crosvm and all renderer/Vulkan/MoltenVK inputs remain those in the v32 closure. The new Host normal impact receipts are `../MAC-FINAL-NORMAL-RESULT-v1786.json`; GPUoff v30 is still historical.

**Guest**

The fixed official manifest and every intended project SHA are in `../UPSTREAM-LOCK.json`. ARM and x86 normal/graphics APEX build receipts, AVB/SBOM, normal bootconfig/noADB, module ABI/signatures and namespace/property-policy checks remain separately bound in their original artifact indices. All1087 fixed project snapshots are now available through SOURCE-INDEX.json:901 preserved matching snapshots and186 clean isolated Linux checkouts. The original workspace still has54 different HEADs and132 absent paths; it was not destructively synchronized. The169 recorded Guest graph inputs are a separate build-time scope. Do not rebuild from a mixed tree and call it the frozen image. Preserve the exact three approved property/bootconfig additions and the single approved sphal→default `vulkan.ranchu.so` link, graphics-only, without new allow rules or compiled policy changes.

## Finite acceptance contract

1. Verify exact artifacts and profile; inspect relevant live resource/port/CID ownership. Create a new unused instance only; no adoption of stale PIDs, failure claim or unknown disk.
2. Use the foreground controller with its original Job/HANDLE, pidfd or audit-token/task-name-port/kqueue ownership. Wait for true RUNNING/READY. Send one command at a time and require strict complete response/identity.
3. GPUon: formal Vulkan hardware enumeration, the existing five VADD vectors, PIXEL256 and coherent matrix. GPUoff explicitly returns GPU_DISABLED, without CPU fallback. Counter118 must persist across one healthy same-disk resume. No UI/input test is included.
4. Normal stop is sent on the same controller with no unknown request. Require BYE, Guest0, separate service cleanup, original resource exits, true inner and outer EOF, process exit codes, log closure and storage-release proof. Only then begin the next independent case.
5. Fault cases use the exact existing normal/noADB/GPUoff contract: confirm118, ADD1 once, timeout10sec or close only the request socket. Original scoped termination once; observe-only afterward. UNKNOWN task remains UNKNOWN even with resource REAPED; retain needsRecovery, disk and claim. No business retry, STOP fallback, KILL escalation or recovery deletion. Five-second observation is not a whole-system hard SLA.

The Windows four normal/two fault cases, Linux v6 six cases, Mac historical v30 normal/v32 faults and final-v32 normal impact tests have explicitly different version scopes. See versions.json; do not combine them into a fabricated common binary identity.

## Executable read-only r14 rollback drill

From this directory on the Windows coordinator:

```powershell
python -B .\verify_r14.py --platform windows
python -B .\verify_r14.py --platform linux
python -B .\verify_r14.py --platform macos
```

`--plan` prints the fixed entry/profile hashes without executing checks. The helper only runs the preserved entry's `check`: no instance create/start/resume, default edits, disk writes or security changes. Linux107 files/49 path contracts/13 directories and source43e4786/manifest89c0365 were rechecked; Windows and Mac original pinned checks returned0. This proves retained route/config integrity, not a newly repeated r14 VM acceptance.

Actual rollback is selecting this preserved r14 route explicitly, NOT resetting a repository or opening an A17 disk. First finish/retain any A17 live/unknown ownership under its own controller. Use only a verified healthy compatible r14 instance for `resume`, or create a new exclusive r14 instance/name/free port with the preserved r14 entry/profile. Keep the same foreground stop/EOF/resource contract. Do not point r14 at A17 instance/storage images or remove an A17 claim. Since no default was changed, no global rollback edit is needed. r14 published code43e4786a48c142b4c8b6f4bfb0db38cd32b20e42 and manifest89c0365a4dff5c0e52a5386877592a2f6fd45548 remain retained; user trees and all historical failures remain untouched.

## Future upstream candidate cycle

Discover the official release manifest and superproject; resolve every project to a fixed SHA. Compare to the previous locked release and classify maintained platform patches. Stage sources without destructive sync, pin tools/locks, reproduce build outputs, run the finite three-platform gates, retain every failure and rollback closure, then review promotion. A branch label or manifest HEAD alone is insufficient. Never auto-merge or install a privileged monitoring service. Publication/default change is not performed by this handoff.


## Captured Guest platform patchset

`linux-guest-patchset/manifest.json` records exact base commits, tracked diffs and nine new files across eleven projects. All source HEAD/status values were stable during capture; no source tree was changed. `python -B ./verify_patchset.py` verifies every recorded byte hash and parses tracked patches without applying them. `PATCHSET-CHECKS.json` preserves the actual result. This capture supplements original build receipts; a later snapshot cannot replace build-time source evidence.

To reproduce in a new isolated tree, first check each base SHA, then review/apply its tracked patch and copy only its listed new files into absent destinations. Existing user trees and unknown destination files must not be overwritten. Re-run the official build and the same finite acceptance gates before claiming new artifacts equivalent. The v1798 isolated-index check additionally verified all eleven tracked patches apply to their exact base commits and all nine new paths are absent in those bases. No production index/worktree was changed, no patch was applied, and a fresh reconstruction has not been built.

The 901 matching project HEAD audit found eleven maintained patchsets and eight clean projects whose Git symlinks are represented by regular files. Read-only classification v1795 verified all 30 regular-file contents equal their indexed symlink blobs; this preserves target text but is not a standard Linux symlink checkout. The remaining186 fixed commits now have full fsck, exact revision and clean isolated Linux checkout evidence. SOURCE-INDEX.json maps every project to its actual location and proof. The original Linux worktree was not switched.


## Exact official Guest build invocation

The successful ARM v1168 and x86 v1198 build scripts use the following official A17 command from the verified source root. Run architectures serially, use new isolated output/dist directories, and capture the complete environment/source identity before starting. This example is documentation, not a command executed during this delivery check.

```bash
set -e
export OUT_DIR=/absolute/new/isolated/guest-output
export DIST_DIR=/absolute/new/isolated/guest-dist
export BSCP_MICRODROID_GRAPHICS=true
unset SOONG_INCREMENTAL_ANALYSIS OVERRIDE_ENABLE_UFFD_GC
source build/envsetup.sh
banchan com.android.virt aosp_arm64 # use aosp_x86_64 for the other architecture
UNBUNDLED_BUILD_SDKS_FROM_SOURCE=true timeout 7200 build/soong/soong_ui.bash --make-mode -j4 com.android.virt
```

A successful build is followed by the architecture-specific AVB, module/signature/dependency, bootconfig normal/noADB, complete file-context and namespace/policy checks and actual runtime gates. The already frozen accepted APEX identities remain in versions.json. The command is not a guarantee of byte-identical output from a different tool/environment.

## Source rollback references

Read-only v1796/v1797 rechecked all 1376 recorded r14 preservation refs: 906 via their current worktrees, 470 via their retained `.repo/projects/<path>.git` stores where the old worktree paths are absent. All refs equal the original recorded SHAs. No checkout was restored or changed. This ref check supplements the three runtime rollback checks; it does not claim every historical object or a fresh source rebuild was exercised.


## Foreground invocation contract

The verified entry parser accepts the same action syntax on all three platforms. Substitute the platform-specific entry from the table and a new validated, sealed state root. The example names/ports below are placeholders, not an instruction to reuse any historical instance:

```text
python -B <entry> --platform <windows|linux|macos> --state-root <absolute-state-root> check --gpu on
python -B <entry> --platform <platform> --state-root <absolute-state-root> create --name <new-exclusive-name> --cid <free-cid> --port <free-port> --gpu on --timeout 120
python -B <entry> --platform <platform> --state-root <absolute-state-root> start --name <same-name> --gpu on
```

Keep the start process stdin/stdout attached to its controller. The checked-in acceptance drivers implement workload request framing and foreground stop; do not replace that with an independent CLI process taking ownership by PID. After complete healthy shutdown/EOF and storage proof, `resume --name <same-name> --gpu on` starts a fresh owned execution against the same healthy disk. `status` reads retained state; it does not adopt a live process. `stop` from an unrelated invocation is not the normal stop contract.

**Linux crosvm and renderer reproduction inputs:** `../build_linux_gpu_v1289.py` records Rust1.93.1, Cargo.lock a27c5e185b62186e58477254c435bf3870f54f6c647721d4033d26c1d09bb4e2, libcap/Wayland/Clang bindgen inputs and `cargo build --locked --offline -p crosvm --target x86_64-unknown-linux-gnu --no-default-features --features android-sparse,balloon,composite-disk,net,qcow,gpu,gfxstream,noncoherent-dma -j2`. Final renderer is the later KVM coherent source/build in `../build_linux_renderer_kvm_v1398.py`, not the initial renderer build: official AOSP dependency resolution, Clang r584948/libc++, standalone Release CMake/Ninja target `gfxstream_backend -j2`. Its recorded `-Wno-error=unused-function` is explicit; no other warning policy may be silently changed. Use new output directories and the final accepted artifact identities in versions.json. These historical scripts are recipe evidence, not safe idempotent installers.

The protected local Virtualization commit `d508901005e3fd57b0368c96daf91920cdb9a70e` was also rechecked as a commit reachable through `refs/heads/codex/r14-local-preserved-20261029`. Its current A17 worktree HEAD was unchanged. Historical Mac ext4-specific test failure/platform skips are not superseded by APFS runtime acceptance; no ext4 mounting or new filesystem test was performed here.

The self-contained `android17-locked.xml` preserves official copyfile/linkfile/group metadata and pins all 1087 project revisions. Its XML/name/path/revision mapping was rechecked against bundled `upstream-lock.json` (`PINNED-MANIFEST-CHECKS.json`). It is a reproducible source specification, not proof that all objects or worktrees are present. Do not activate it destructively over protected user trees.
