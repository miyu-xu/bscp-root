# AVF Performance Benchmark Suite

> Date: 2026-04-25
> Scope: VM startup time measurement across Linux (KVM), macOS (HVF), Windows (WHPX)

## Overview

This framework measures end-to-end VM boot time on AVF desktop hosts. The primary metric is
**wall-clock time from VM launch to guest payload ready** (`notifyPayloadReady` marker).

## Quick Start

```sh
# Run default benchmark (5x microdroid, 1x warmup)
./scripts/bench_vm_startup.sh

# 10 iterations, app variant, custom dist
./scripts/bench_vm_startup.sh \
    -Iterations 10 \
    -Variant app \
    -DistRoot /path/to/dist

# Skip warmup, custom output directory
./scripts/bench_vm_startup.sh \
    -NoWarmup \
    -OutputDir /tmp/bench-results
```

## Scripts

| Script | Description |
|--------|-------------|
| `scripts/bench_vm_startup.sh` | Main benchmark driver (Linux/macOS via shell; Windows via WSL/Cygwin) |

## Methodology

### Measurement

1. **Start time**: wall clock (`date +%s%N`) immediately before `vm_wrapper -Command run-microdroid`
2. **End time**: when `vm_wrapper` exits with success (exit 0) after the VM has reached `notifyPayloadReady`
3. **Guest-log validation**: cross-checked against guest kernel log timestamps for `Notified host payload ready successfully`

### Iterations

- Default: 5 timed iterations + 1 warmup
- Warmup iteration ensures disk caches are populated and JIT compilation (if any) is done
- Each iteration includes full VM teardown and cleanup between runs

### Statistics

- **Min/Max**: best/worst case boot time
- **Average**: arithmetic mean
- **Median**: P50 (resistant to outliers)

## Platform-Specific Notes

### Linux (KVM)

```sh
# Direct benchmark
./scripts/bench_vm_startup.sh

# Results stored in out/dist/linux/logs/bench-<timestamp>/
```

- KVM warm caches significantly improve second+ run times
- CPU governor affects results: `cpufreq-set -g performance` recommended
- `/dev/kvm` must be accessible

### macOS (HVF)

```sh
# Direct benchmark
./scripts/bench_vm_startup.sh

# Results stored in out/dist/macos/logs/bench-<timestamp>/
```

- First boot after system start may be slower (Hypervisor.framework warmup)
- Apple Silicon (M-series) has different performance characteristics than Intel
- Use `build_all.sh` to ensure up-to-date binaries

### Windows (WHPX)

The shell script works in WSL or Cygwin environments. For native PowerShell:

```powershell
# Use the existing run_windows_avf_regression.ps1 with stopwatch
$timer = [System.Diagnostics.Stopwatch]::StartNew()
& .\scripts\vm_windows.ps1 -Command run-microdroid ...
$timer.Stop()
Write-Host "Boot time: $($timer.Elapsed.TotalSeconds)s"
```

## Output Format

### Console Output

```
| Iteration | Boot Time |
|-----------|-----------|
| 1         | 12.34s    |
| 2         | 11.89s    |
| 3         | 11.56s    |
| 4         | 12.01s    |
| 5         | 11.78s    |
```

### JSON Results

```json
{
    "platform": "Darwin",
    "variant": "run-microdroid",
    "iterations": 5,
    "warmup": 1,
    "timestamp": 1745623456,
    "statistics": {
        "count": 5,
        "min": 11.56,
        "max": 12.34,
        "avg": 11.92,
        "median": 11.89
    }
}
```

## Extension: Adding New Benchmarks

### virtio Disk Throughput

Planned benchmark: measure sequential/random read/write throughput inside the VM.

```sh
# Inside guest (via vsock ADB bridge):
adb shell dd if=/dev/zero of=/data/test.img bs=1M count=100 conv=fsync
```

### virtio Network Throughput

Planned benchmark: measure TCP/UDP throughput between host and guest.

```sh
# Host: iperf3 -s
# Guest (via ADB): iperf3 -c 192.168.<host_ip>
```

### Memory Latency

Planned benchmark: measure guest memory access latency using `lmbench` or custom payload.

### CPU Virtualization Overhead

Planned benchmark: compare native vs. virtualized CPU performance (Sysbench, Stress-NG).
