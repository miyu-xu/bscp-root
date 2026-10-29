# Microdroid r14 delivery status — 2026-10-29

The integrated engineering/release branch is `bscp-android-15.0.0_r14` in both repositories. GitHub default branches remain `main`. Lifecycle is opt-in; existing run-microdroid defaults are unchanged.

Runtime integration: [89a584c2d9a5203b5e1f01116d1be5a806dd8056](https://github.com/miyu-xu/bscp-root/commit/89a584c2d9a5203b5e1f01116d1be5a806dd8056). The manifest initially pinned this at [3b9e395754b09627d5138631ca64b10598d555f0](https://github.com/miyu-xu/bscp-manifest-repo/commit/3b9e395754b09627d5138631ca64b10598d555f0); the current r14 default.xml is authoritative, including later documentation-only root commits. No unrelated project dependency was upgraded.

## Version-bound acceptance

| Capability | Linux x86_64/KVM | Windows x86_64/WHPX | Mac arm64/HVF |
|---|---|---|---|
| Fixed normal/no-adbd payload, UID/GID6000 | Limited native PASS | Limited native PASS | Limited native PASS |
| Real GPU-off and GPU-on start + healthy resume | Both tested | Both tested; run/run-mp distinction | Both tested |
| Vulkan offscreen + coherent | RTX2060,256 pixels/0 error;12 rounds/12288 words | RTX2060,same checks | M1,same checks |
| GPU compute | 5 VADD requests/14 uint32 values, overflow included | Same task/reference check | Same task/reference check |
| GPU-off compute request | GPU_DISABLED; no CPU fallback | Same | Same |
| Persistence | SET118 → normal stop → same-disk GET118 → update119 | Same | Same |
| Timeout and disconnect | Original pidfd, single TERM | Original Job/HANDLE, single termination | Original audit token/task-name port/kqueue |

The repository direct-entry batch had12 normal boots and6 separate abnormal cases. It used e700090 production files, then fixture-only acc143c; r14 integration retained those production scripts byte-for-byte. New r14 local regression had45 tests (one Windows-only skip on Linux); [three-OS CI](https://github.com/miyu-xu/bscp-root/actions/runs/40795076616) passed for89a584c. Publication/documentation did not launch additional VMs. Six checkout files differ only in CRLF/LF; normalized source matches, raw hashes are not all equal.

The earlier unified-candidate batch (544 evidence files), direct repository batch (272 indexed evidence files), and earlier platform experiments are distinct evidence sets, not additive repetitions of one test. The latest direct batch does not assert a third boot reading119. Earlier three-stage persistence reports remain separately bound. Native delayed observe-only tests belong to their earlier versions; current production-loop regressions include injected delayed observations. Hosted CI is no-VM testing, not GPU acceptance.

Abnormal tasks stay UNKNOWN even when resources are REAPED. Failed disks and claims remain needsRecovery and cannot healthy-resume. Each case used one ADD1 with no retry/STOP; first outer EOF/exit0 and resource evidence preceded the second case. Normal Guest exit0, service cleanup, original-handle exit, inner/outer EOF and disk release are separate evidence. No universal five-second recovery SLA is claimed. Historical failed/UNKNOWN records remain unchanged, including lost reader evidence and retained claims; cleanup is not task success.

## Entry and deployment

See [foreground lifecycle usage](MICRODROID_LIFECYCLE.md). Use `python3 -B scripts/microdroid.py --platform linux|macos|windows --state-root ABSOLUTE_ROOT` followed by check/create/start/status/resume. Create requires `--gpu on|off`; mode is persistent. The root must already have reviewed profiles/platform.json and a pinned closure manifest; instance_root equals state-root/instances. Fixed runtime/Guest artifacts are external deployment inputs, not an automatically built or downloadable universal release bundle.

Keep the owning controller's stdin/stdout open. Send `status`, sequential `request ID COMMAND`, and normal `stop` only after complete request outcomes. Standalone CLI status reads recorded state, not live ownership; standalone stop refuses takeover. UNKNOWN never auto-retries or auto-recovers. Fixed payload commands include INFO, GET, SET, ADD, PROPERTIES, PIXEL, COHERENT and bounded VADD; this is not arbitrary payload/shell scheduling.

## Limits and next work

Current profiles use protected=false and disable-sandbox; normal/no-adbd is not proof of protected VMs or production isolation. GPU-off still links libvulkan; it does not imply zero library loading. No Microdroid GLES/ANGLE/AHB/strict external pending-fence, real display/input/audio, general networking/shared-folder, concurrent instances, snapshots/migration, arbitrary fault recovery, power-loss/corrupt-disk recovery, stress or performance qualification is claimed. Full Android graphics evidence cannot substitute for Microdroid. A is paused; H3 and UI/input are deferred.

Completed: dual mode, bounded compute, finite normal/fault resource reclamation, opt-in repository integration, r14 publication and CI. Future work requires a separately chosen scope; it is not blocked on branch selection. Default activation is not part of publication.

Local evidence is retained in bscp-delivery/microdroid-repository-integration-v1: RESULT.md, FINAL-RUNTIME-REVIEW.md, EVIDENCE-INDEX.json, ACTUAL-SOURCE-IDENTITY.json, CHECKOUT-EOL-IDENTITY.json, R14-REVIEW.md and R14-PUBLICATION.md. These local files are not falsely presented as GitHub-hosted attachments. Git rollback is an ordinary code/manifest revert; never delete or restore failed disks as rollback.
