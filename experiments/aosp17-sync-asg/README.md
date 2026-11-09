# Isolated synchronization and ASG candidate

Source-only, **not enabled by the release selector**. [中文](README.zh-CN.md)

This package combines the previously validated conservative fence-ownership,
strict-submit-result, deferred-queue rejection, descriptor-retirement and Guest
wait-error/capability guards with the ASG shutdown repair. It is not the complete
experimental external-semaphore bridge. Linux kernel out-fence changes and the
later Windows native payload rotation/decoder generation work are **not** silently
included or enabled here. Unsupported external synchronization stays unavailable.

ASG teardown previously sent an Exit into a bounded queue, which could block when
the consumer had already terminated with the queue full. It now stops that queue,
handles receive failure as Exit, and resets the read-loop sleeping flag on each
iteration. Consumer destruction and GPU object lifetime waits remain intact.

## Reproduce the source overlay

Requires existing Python 3 and Git; installs nothing. Run from this directory:

```sh
python apply_candidate.py --host snapshots/host-before --guest snapshots/guest-before --output /path/to/new-overlay
```

The destination parent must exist and the destination must not exist. Inputs are
hash-checked before writing, outputs are hash-checked after applying, and source
trees are never modified. `MANIFEST.json` is a per-file baseline contract: this is
a **partial overlay**, not a complete checkout or a patch for arbitrary upstream
HEAD. Use a separate full, pinned source tree and matching toolchain to build.
Rebuild affected backend, common, address-space and renderer entry objects; keep
the matching strict-profile Vulkan archive. Guest requires
`BSCP_REQUIRE_NATIVE_SUBMIT_RESULT=1`. Do not overwrite a running deployment.

`REGRESSION.json` contains sanitized runtime aggregates. The combined-source
application test does not itself constitute a new VM run of this exact entire
overlay: component combinations were tested on their recorded platform branches.
ASG source semantics are the same; comments/formatting may differ from Windows.
Windows tested three controlled error exits plus normal export and inference.
Linux tested normal export, bridge on/off inference and a small throughput sample.
Mac tested inference and accurate **unsupported** capability reporting. No claim
of full fault-matrix parity, arbitrary concurrency or full external semaphore
support follows. Historical UNKNOWN tasks remain UNKNOWN.

Tests under `tests/` exercise the extracted production lifecycle methods and
MessageChannel with existing project headers/toolchains; mocks are identified in
their source. They are not a substitute for real Guest regression. Optional llama
patches are experiments and are not automatically applied by the core overlay.

Third-party source headers retain their original licenses. BSCP-original additions
follow the repository license policy; this package does not relicense upstream code.

See [performance and limits](../../docs/LLAMA-PERFORMANCE.md).
