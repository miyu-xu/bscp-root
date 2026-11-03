> Personal environment fields are redacted. Placeholders do not admit a new deployment or preserve executable historical commands. Original evidence is retained privately.

# Fixed source availability

All 1087 manifest-pinned project snapshots are available on Linux. `SOURCE-INDEX.json` binds every path/name/SHA to its location and hashed evidence. This is distributed source availability, not a new integrated repo checkout or a new build.

- 901 existing matching snapshots: exact HEAD and current commit/tree/blob presence checked with lazy fetch disabled. Maintained patches and documented symlink representations remain; this is not a fresh clean checkout or full historical fsck.
- 186 clean isolated checkouts: batches v1800 (15), v1808 (142), v1812 (8), v1814 (10), v1819 (11), under `<PRIVATE_LINUX_HOME>/bscp/out/aosp17-source-transition-v1/full-source-batch-<version>/trees`. Each has exact revision, full fsck and clean checkout evidence.
- Original workspace remains 901 matching HEADs, 54 different HEADs, and 132 absent project paths. User worktrees, r14 refs, modified files and rollback assets were not reset or removed.
- Eight original network failures remain recorded. Recovery v1811 verified six repositories, then exited1 while replacing its own Windows receipt. Recovery v1823 reused verified objects, then stopped on one Git DNS-helper thread creation failure. Recovery v1825 reused all completed objects and finished the remaining snapshot. All prior receipts remain preserved; final state is RECOVERY_COMPLETE.
- Missing promised historical ancestors in the older v1804 audit are distinct from current snapshot files. No claim is made that all historical Git objects were downloaded.

The locked XML preserves official copyfile/linkfile metadata. No default switch, publication, destructive repo sync, new VM test or replacement of original build-time evidence occurred during source completion. Use the original platform source/patch/tool/lock-to-artifact-to-runtime records for accepted binaries, and the runbook for a fresh isolated reproduction.
