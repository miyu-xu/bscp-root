#!/usr/bin/env bash
# Copyright 2026 The Android Open Source Project
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# AVF VM Boot Time Benchmark
# Measures end-to-end VM boot time across platforms.

set -euo pipefail

REPO_ROOT=""
DIST_ROOT=""
OUTPUT_DIR=""
ITERATIONS=5
VARIANT="run-microdroid"
WARMUP=1

while [[ $# -gt 0 ]]; do
    case "$1" in
        -RepoRoot|--repo-root) REPO_ROOT="$2"; shift 2 ;;
        -DistRoot|--dist-root) DIST_ROOT="$2"; shift 2 ;;
        -OutputDir|--output-dir) OUTPUT_DIR="$2"; shift 2 ;;
        -Iterations|--iterations) ITERATIONS="$2"; shift 2 ;;
        -Variant|--variant)
            case "$2" in
                microdroid|run-microdroid) VARIANT="run-microdroid" ;;
                app|run-app) VARIANT="run-app" ;;
                *) echo "Unknown variant: $2 (use: microdroid|app)" >&2; exit 2 ;;
            esac
            shift 2 ;;
        -NoWarmup|--no-warmup) WARMUP=0; shift ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo "  -RepoRoot,--repo-root PATH   Repository root (default: auto-detect)"
            echo "  -DistRoot,--dist-root PATH   Dist directory"
            echo "  -OutputDir,--output-dir PATH Output directory"
            echo "  -Iterations,--iterations N   Number of benchmark iterations (default: 5)"
            echo "  -Variant,--variant TYPE      Benchmark variant: microdroid|app (default: microdroid)"
            echo "  -NoWarmup,--no-warmup        Skip warmup iteration"
            exit 0 ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done

OS_NAME="$(uname -s)"
case "$OS_NAME" in
    Darwin)  VM_WRAPPER="vm_macos.sh" ;;
    Linux)   VM_WRAPPER="vm_linux.sh" ;;
    *)       echo "Unsupported OS: $OS_NAME" >&2; exit 2 ;;
esac

if [[ -z "$REPO_ROOT" ]]; then
    REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi
if [[ -z "$DIST_ROOT" ]]; then
    DIST_ROOT="$REPO_ROOT/out/dist"
    case "$OS_NAME" in
        Darwin) DIST_ROOT="$DIST_ROOT/macos" ;;
        Linux)  DIST_ROOT="$DIST_ROOT/linux" ;;
    esac
fi
if [[ -z "$OUTPUT_DIR" ]]; then
    OUTPUT_DIR="$DIST_ROOT/logs/bench-$(date +%Y%m%d-%H%M%S)"
fi

mkdir -p "$OUTPUT_DIR"

VM_WRAPPER="$REPO_ROOT/scripts/$VM_WRAPPER"
COMMON_ARGS=(-RepoRoot "$REPO_ROOT" -DistRoot "$DIST_ROOT")

echo "=== AVF Benchmark Suite ==="
echo "Platform:    $OS_NAME"
echo "Variant:     $VARIANT"
echo "Iterations:  $ITERATIONS"
echo "Warmup:      $(( WARMUP == 1 ? 1 : 0 )) iteration(s)"
echo "Output:      $OUTPUT_DIR"
echo "VM wrapper:  $VM_WRAPPER"
echo ""

collect_results() {
    local log_dir="$1"
    local start_marker end_marker elapsed
    local guest_log="$log_dir/guest-log.txt"
    local vm_log="$log_dir/vm-run-microdroid.log"

    # Primary: measure from VM wrapper start to payload ready in guest log
    start_marker=$(grep -E '=== \[.*s\] run-microdroid' "$OUTPUT_DIR/bench-run.log" 2>/dev/null | tail -1 | sed 's/=== \[\([0-9.]*\)s\].*/\1/')
    if [[ -f "$guest_log" ]]; then
        end_marker=$(grep -oP '^\[ *[0-9.]+\]' "$guest_log" | tail -1 | tr -d '[]' | awk '{print $1}')
        if [[ -n "$end_marker" ]]; then
            elapsed=$(echo "$end_marker" | awk '{printf "%.2f", $1}')
            echo "$elapsed"
            return 0
        fi
    fi

    # Fallback: measure wall clock from VM launch to payload ready
    if [[ -f "$guest_log" ]] && grep -q 'Notified host payload ready successfully' "$guest_log"; then
        local ready_line
        ready_line=$(grep -n 'Notified host payload ready successfully' "$guest_log" | head -1 | cut -d: -f1)
        if [[ -n "$ready_line" ]]; then
            local first_line
            first_line=$(head -1 "$guest_log" | grep -oP '^\[ *[0-9.]+\]' | tr -d '[]')
            local last_line
            last_line=$(sed -n "${ready_line}p" "$guest_log" | grep -oP '^\[ *[0-9.]+\]' | tr -d '[]')
            if [[ -n "$first_line" && -n "$last_line" ]]; then
                elapsed=$(echo "$last_line - $first_line" | bc 2>/dev/null || echo "0")
                echo "$elapsed"
                return 0
            fi
        fi
    fi

    # Last resort: measure by marker presence
    if [[ -f "$guest_log" ]] && grep -q 'Notified host payload ready successfully' "$guest_log"; then
        echo "measured"
        return 0
    fi

    echo "FAILED"
    return 1
}

bench_run() {
    local iteration="$1"
    local log_dir="$OUTPUT_DIR/run-$iteration"
    local result="FAILED"

    echo "--- Iteration $iteration ---"

    # Pre-cleanup
    "$VM_WRAPPER" -Command cleanup "${COMMON_ARGS[@]}" >/dev/null 2>&1 || true
    rm -rf "$log_dir"
    mkdir -p "$log_dir"

    local start_time end_time elapsed
    start_time=$(date +%s%N 2>/dev/null || echo "0")

    # Run VM
    if "$VM_WRAPPER" \
        -Command "$VARIANT" \
        "${COMMON_ARGS[@]}" \
        -LogDir "$log_dir" \
        -KeepTemp \
        -TimeoutSecs 120 \
        >> "$OUTPUT_DIR/bench-run.log" 2>&1; then

        end_time=$(date +%s%N 2>/dev/null || echo "0")
        if [[ "$start_time" != "0" && "$end_time" != "0" ]]; then
            elapsed=$(echo "scale=2; ($end_time - $start_time) / 1000000000" | bc 2>/dev/null || echo "0")
        else
            elapsed=$(collect_results "$log_dir")
        fi

        if [[ "$elapsed" != "FAILED" ]]; then
            result="$elapsed"
            echo "  Boot time: ${result}s"
        fi
    fi

    # Post-cleanup
    "$VM_WRAPPER" -Command cleanup "${COMMON_ARGS[@]}" >/dev/null 2>&1 || true

    echo "$result"
}

# Warmup iteration
if [[ "$WARMUP" -eq 1 ]]; then
    echo "=== Warmup iteration ==="
    warmup_result=$(bench_run "warmup")
    echo "  Result: $warmup_result"
    echo ""
fi

# Benchmark iterations
echo "=== Benchmark ($ITERATIONS iterations) ==="
> "$OUTPUT_DIR/bench-run.log"
RESULTS=()
for i in $(seq 1 "$ITERATIONS"); do
    result=$(bench_run "$i")
    RESULTS+=("$result")
    echo ""
done

# Summary
echo "=== Results ==="
echo ""
echo "All times in seconds (lower is better)"
echo ""
echo "| Iteration | Boot Time |"
echo "|-----------|-----------|"
VALID_TIMES=()
for i in $(seq 0 $((ITERATIONS - 1))); do
    idx=$((i + 1))
    val="${RESULTS[$i]}"
    if [[ "$val" =~ ^[0-9]+\.?[0-9]*$ ]]; then
        VALID_TIMES+=("$val")
        printf "| %-9d | %-9s |\n" "$idx" "${val}s"
    else
        printf "| %-9d | %-9s |\n" "$idx" "$val"
    fi
done
echo ""

if [[ ${#VALID_TIMES[@]} -ge 1 ]]; then
    total=0
    min="${VALID_TIMES[0]}"
    max="${VALID_TIMES[0]}"
    for t in "${VALID_TIMES[@]}"; do
        total=$(echo "$total + $t" | bc)
        min=$(echo "if ($t < $min) $t else $min" | bc -l)
        max=$(echo "if ($t > $max) $t else $max" | bc -l)
    done
    avg=$(echo "scale=2; $total / ${#VALID_TIMES[@]}" | bc)

    # Median
    sorted=($(printf '%s\n' "${VALID_TIMES[@]}" | sort -n))
    mid=$(( ${#sorted[@]} / 2 ))
    if [[ $(( ${#sorted[@]} % 2 )) -eq 0 ]]; then
        med=$(echo "scale=2; (${sorted[$mid-1]} + ${sorted[$mid]}) / 2" | bc)
    else
        med="${sorted[$mid]}"
    fi

    echo "Statistics:"
    echo "  Count:   ${#VALID_TIMES[@]}"
    echo "  Min:     ${min}s"
    echo "  Max:     ${max}s"
    echo "  Average: ${avg}s"
    echo "  Median:  ${med}s"

    # Save JSON results
    cat > "$OUTPUT_DIR/results.json" <<JSONEOF
{
    "platform": "$OS_NAME",
    "variant": "$VARIANT",
    "iterations": $ITERATIONS,
    "warmup": $WARMUP,
    "timestamp": $(date +%s),
    "results": [
$(for t in "${VALID_TIMES[@]}"; do echo "        $t,"; done)
    ],
    "statistics": {
        "count": ${#VALID_TIMES[@]},
        "min": $min,
        "max": $max,
        "avg": $avg,
        "median": $med
    }
}
JSONEOF
    echo ""
    echo "JSON results saved to: $OUTPUT_DIR/results.json"
fi

echo ""
echo "Benchmark complete. Artifacts: $OUTPUT_DIR"
