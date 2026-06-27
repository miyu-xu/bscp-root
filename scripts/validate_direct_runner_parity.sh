#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LINUX_LOG_DIR="${1:-$REPO_ROOT/out/dist/logs/android-linux-parity2}"

echo "Linux direct-runner parity validation"
echo "Log dir: $LINUX_LOG_DIR"

"$REPO_ROOT/scripts/run_android_linux.sh" --dry-run >/dev/null
echo "[PASS] run_android_linux.sh dry-run"

if [[ -d "$LINUX_LOG_DIR" ]]; then
    "$REPO_ROOT/scripts/check_android_linux_markers.sh" "$LINUX_LOG_DIR"
    echo "[PASS] boot completion markers"
    "$REPO_ROOT/scripts/check_android_parity_markers.sh" "$LINUX_LOG_DIR"
    echo "[PASS] radio-adjacent parity markers"
else
    echo "[SKIP] no captured log dir at $LINUX_LOG_DIR"
    echo "       Run: scripts/run_android_linux.sh --mode headless --timeout-secs 240 --log-dir $LINUX_LOG_DIR"
fi

echo
echo "Windows follow-up (requires Windows host rebuild):"
echo "  build_all.bat with net,slirp crosvm features"
echo "  scripts\\run_android_windows_gfxstream_angle.ps1 -FullHvc -TimeoutSecs 420"
echo "  inspect LogDir\\crosvm-command.txt for two --net entries and hvc5/hvc12 wiring"
