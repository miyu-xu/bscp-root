# llama.cpp: measured Host / Microdroid results

[中文](LLAMA-PERFORMANCE.zh-CN.md) · [Candidate source](../experiments/aosp17-sync-asg/README.md)

## Verified capabilities, not a universal performance claim

BSCP has executed actual Microdroid Vulkan compute/inference on Linux KVM,
Windows WHPX and macOS HVF. Separate release acceptance also verified finite
offscreen pixel readback, coherent-buffer workloads, GPU on/off, same-disk restart
and bounded failure cleanup. These are finite workloads, not arbitrary Vulkan,
UI/input, automatic multi-host inference, or cache coherence between machines.
CPU/GPU synchronization is still required even for coherent memory.

The numbers below are historical controlled **same-machine** pairs, not the new
ASG shutdown build, and not a ranking of operating systems. Hardware differs.
Public reference binaries remain limited-deployment packages, not generic installers.

## Vulkan throughput: historical paired baseline

Mean ± pooled sample SD, tokens/s; three process rounds per side, three samples
per shape per process (n=9). SD is not a confidence interval. pp512 processes a
512-token prompt; tg128 generates 128 tokens. Neither measurement is TTFT.

| Host / hardware | Host pp512 | Guest pp512 | Host tg128 | Guest tg128 |
|---|---:|---:|---:|---:|
| Linux/KVM, i7-12700KF, RTX 2060 6GB, 32GiB | 1616.96 ± 4.52 | 1125.59 ± 21.44 | 257.12 ± 3.86 | 178.64 ± 11.61 |
| Windows/WHPX, i5-12400F, RTX 2060, 32GiB | 1623.47 ± 9.60 | 1136.83 ± 7.61 | 270.59 ± 1.96 | 228.97 ± 9.49 |
| macOS/HVF, Apple M1, 8GiB | 784.05 ± 4.85 | 296.24 ± 2.72 | 60.28 ± 1.16 | 9.13 ± 0.03 |

Shared model: Qwen2.5-0.5B-Instruct GGUF Q4_K_M, upstream model revision
`9217f5db79a29953eb74d5343926648285ec7e67`, 491400032 bytes,
SHA256 `74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db`.
Metadata reports 630167424 parameters; llama-bench labels it “qwen2 1B”.
llama.cpp `fac3a894fc04901c3e5abaff457e024c6b52e04f` plus recorded core16/diagnostic
patch SHA256 `9418ef8dada0ee31eec7f607d0a35a5cd851a4d45c6affa4982acc9db982a3ef`.
Threads=1, batch=512, ubatch=128, requested GPU layers=99, flash attention off,
FP16 arithmetic disabled on both sides, upstream warmup enabled. Guest: SDK37,
normal/noADB, one vCPU; x86 memory=4096MiB, Mac=2048MiB. Linux driver=580.178.04.
Not every historical OS/driver build string was retained in this summary; it is
not a bit-for-bit environment reproduction specification.

Linux host CPU set was 2/4/6/8; Windows original Job affinity likewise. Mac was not
pinned. Frequency/power policy was not changed. Compilers and runtimes differ
(native GCC/MSVC/Apple Clang versus Android NDK/bionic), so these differences are
not solely hypervisor overhead. Mac uses Vulkan through MoltenVK, **not llama's
native Metal backend**, and an isolated debug crosvm with a vsock fragmentation
fix (`d78d357e0c4779c603dac1fa94e8c783bab374d013535d4ede0242deb9053c67`).
Mac model preparation took about 208–212 seconds. Startup, transfer and hash checks
are excluded from throughput. This old Mac baseline predates later tuning; do not
present it as the latest optimized candidate or a released-runtime benchmark.

## Mac optimization follow-up (isolated candidates)

The severe original Mac loss above was substantially corrected in later experiments.
It is not the current candidate's performance. These stages are separate experiments,
not a single contemporaneous Host/Guest comparison or a released-binary benchmark.

| Stage / workload | Host tokens/s | Guest tokens/s | Sampling and meaning |
|---|---:|---:|---|
| v1916 blocking fence wait, pp512 | 778.72 | 793.73 | One invocation per mode, 3 internal samples; about 2% is not evidence of stable Guest superiority |
| v1916 blocking fence wait, tg128 | 59.97 | 46.13 | Same experiment; polling Guest was 9.1272 |
| v1921 nodes-per-submit=1000, tg128 | 60.95 ± 0.93 | 53.16 ± 0.69 | 5 samples per side; Guest/Host 87.2% |
| v1927 combined wait, tg128 depth 0 | Not remeasured | 59.33 ± 1.63 | OFF/ON/ON/OFF; 6 samples/mode; OFF 52.46 ± 1.89, gain 13.1% |
| v1927 combined wait, tg128 depth 1024 | Not remeasured | 58.82 ± 1.27 | 6 samples/mode; OFF 54.74 ± 1.86, gain 7.46% |
| v1961 strict submit results, depth 0 / 1024 | Not remeasured | 53.75 ± 1.72 / 54.90 ± 1.22 | 6 samples/mode; result-checking OFF 57.60 / 58.66; costs 6.69% / 6.41% |
| v2026 profile guard, depth 0 / 1024 | Not remeasured | 53.58 / 54.00 | Mean of 2 launches/mode, 3 internal samples/shape; control 53.19 / 54.54; changes +0.73% / −0.99%, not a robust speedup claim |

SD where shown is sample SD, not a confidence interval. Same M1, Vulkan/MoltenVK
(not native Metal), Q4_K_M model/hash and llama commit/core16 basis listed above;
1 thread, batch 512, ubatch 128, requested GPU layers 99, FP16 arithmetic off,
Guest 1 vCPU/2048 MiB. Later stages use nodes-per-submit=1000 and isolated backend
patches; they must not be attributed to an unchanged upstream build. The v1927
backend source SHA256 is `a71b102e757b4ba176e9a3d2f9281b0b407a500326cf8d53165d627ec8a96d36`.
Frequency was not locked. Startup/model staging is excluded. There is no new
paired Host denominator for 59.33 or the later strict-profile results.

The main change replaced repeated final-fence status polling with blocking completion;
submission grouping and combined waiting further improved decode. Strict submission
error handling subsequently incurred measurable cost: do not publish only the fastest
59.33 figure as the latest complete candidate. v1924/v1927 compared 3 prompts with
1024 input tokens, 3 repeats, 151936 logits each and generation; compared GPU outputs
were byte-equal. v1927 numeric comparison passed but one correctness instance remained
UNKNOWN after a lifecycle identity change, so its overall case did not pass. Its four
performance instances were PASS/REAPED. v2026 correctness was PASS/REAPED with EOF.
These comparisons do not prove CPU/GPU equality or arbitrary-model correctness.

The later ASG shutdown regression below has no new Mac throughput measurement.
Optional llama patches are source experiments, not automatically applied by the
candidate overlay or enabled in the formal release. The frozen release selector and
binary assets are unchanged. Original record names, hashes, samples and stage-specific
limits are in [the aggregate JSON](benchmarks/llama-20261109.json), `mac_optimization`.

## CPU observations (different sampling designs)

| Hardware / sampling | Host pp512 | Guest pp512 | Host tg128 | Guest tg128 |
|---|---:|---:|---:|---:|
| Linux i7-12700KF, early n=5 | 102.59 ± 0.05 | 93.60 ± 0.03 | 33.15 ± 0.07 | 31.91 ± 0.13 |
| Windows i5-12400F, n=9 | 90.14 ± 0.50 | 87.18 ± 0.94 | 32.26 ± 1.20 | 31.79 ± 0.44 |
| Mac M1, posthoc matched-feature n=9 | 73.06 ± 0.04 | 66.91 ± 0.11 | 55.49 ± 0.06 | 51.70 ± 0.12 |

Same model, one thread, ngl=0. Linux is the early unpinned five-repeat batch,
not the later controlled GPU sequence. Mac native FP16 CPU features were disabled
to match Guest features; its new native samples were compared with stored Guest
samples, not contemporaneously interleaved. Compiler/runtime and scheduling
differences remain; these are observations, not a common cross-platform ranking.

## Latest ASG regression is a separate experiment

Windows: three real controlled-error exits, normal external export and nine
inference records passed; Linux: real export plus 18 bridge-off/on inference
records; Mac: nine inference records and accurate unsupported capability reporting.
Logits matched each platform's recorded regression baseline byte-for-byte.
This does **not** assert CPU versus GPU numerical identity.

Linux's latest sequential bridge-off/on sample (three internal samples each) was
pp512 1059.26±4.15 / 1058.83±3.81 and tg128 218.01±0.64 / 209.64±13.09.
The on-group decode samples were 194.52, 217.18, 217.21: retain the slow sample.
This is neither an ASG before/after comparison nor evidence of zero regression.
No new Mac throughput claim accompanies its ASG correctness regression.
See the source package's [sanitized evidence](../experiments/aosp17-sync-asg/REGRESSION.json).

## Android phone: native only

Pixel 7, API35, arm64, Mali-G710; same GGUF hash. Five samples, same pp512/tg128,
threads/batch/ubatch, CPU ngl=0 or Vulkan ngl=99; no frequency lock:

| Native backend | pp512 tokens/s | tg128 tokens/s |
|---|---:|---:|
| CPU | 42.2227 ± 0.5176 | 28.0427 ± 0.5649 |
| Vulkan default FP16 | 64.7743 ± 0.0888 | 34.5737 ± 0.1312 |
| Vulkan F16 disabled | 82.8302 ± 0.0828 | 34.3895 ± 0.1123 |

CPU embedded commit is unknown, GPU reports `1b61d01`; they have **not** been
verified as the same clean source build. No direct virtualization-overhead inference
is valid. Actual phone-native external semaphore export/signal/wait passed three
times. Phone Microdroid GPU and phone Host/Guest pairing were **not run**: a matching
supported graphics deployment chain is still missing; no system component was replaced.

Strict CPU/GPU logits tolerance (atol=rtol=1e-4) failed. Matching generated tokens
or top1 does not erase that failure. An F32 expansion of the same quantized weights
(not original unquantized weights) showed activation-quantization effects. Disabling
integer dot reduced maximum residuals to 0.0075–0.0137 and matched sampled top10
ordering, but strict residuals remain and TTFT rose from roughly 0.55–0.60s to
3.13–3.14s. No default precision change or relaxed threshold is claimed.

## Evidence and retained boundaries

Public aggregate data: [benchmark data](benchmarks/llama-20261109.json).
Original controlled record: `BSCP-llama-controlled-evaluation-20261103.json`, version26;
Linux lines1–130, Windows2973–3202, Mac10695–10910. Public data is a curated excerpt,
not an assertion that private logs are downloadable. Latest report v2176 SHA256:
`23ddfe70034a43301e350c84143e66eba9b7139839c3af6732ed8f8b56ba178d`.
Experimental external-sync gates remain closed by default; Mac SYNC_FD is unsupported.
Historical UNKNOWN and Mac ext4 70-pass/1-fail remain. Independent Windows CLI
reconnect is disabled. No formal binaries, tags, frozen release manifests or old
assets are replaced by this source/documentation update.
