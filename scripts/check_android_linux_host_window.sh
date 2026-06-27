#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

LOG_DIR="$REPO_ROOT/out/dist/logs/android-linux"
WAIT_SECS="60"
WIDTH="1280"
HEIGHT="720"
X_DISPLAY="${DISPLAY:-}"

usage() {
    cat <<EOF
Usage: $0 [options]

Options:
  --log-dir DIR       Output log dir (default: $LOG_DIR)
  --wait-secs N       Seconds to wait for a matching X11 window (default: $WAIT_SECS)
  --width N           Expected crosvm window width (default: $WIDTH)
  --height N          Expected crosvm window height (default: $HEIGHT)
  --x-display VALUE   X11 display (default: DISPLAY env)
  --help              Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --log-dir) LOG_DIR="$2"; shift 2 ;;
        --wait-secs) WAIT_SECS="$2"; shift 2 ;;
        --width) WIDTH="$2"; shift 2 ;;
        --height) HEIGHT="$2"; shift 2 ;;
        --x-display) X_DISPLAY="$2"; shift 2 ;;
        --help|-h) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

if [[ -z "$X_DISPLAY" ]]; then
    echo "Error: DISPLAY or --x-display is required" >&2
    exit 1
fi
if ! command -v xwininfo >/dev/null 2>&1; then
    echo "Error: xwininfo is required" >&2
    exit 1
fi

mkdir -p "$LOG_DIR/host-window"
TREE_FILE="$LOG_DIR/host-window/xwininfo-root-tree.txt"
WINDOW_FILE="$LOG_DIR/host-window/crosvm-window.txt"
XWD_FILE="$LOG_DIR/host-window/crosvm-window.xwd"

find_window() {
    DISPLAY="$X_DISPLAY" xwininfo -root -tree >"$TREE_FILE"
    python3 - "$TREE_FILE" "$WIDTH" "$HEIGHT" <<'PY'
import re
import sys
from pathlib import Path

tree = Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace").splitlines()
width = int(sys.argv[2])
height = int(sys.argv[3])

line_re = re.compile(r'^\s*(0x[0-9a-fA-F]+)\s+(.*?):.*?\s+(\d+)x(\d+)[+-]')
fallback = None

for line in tree:
    match = line_re.search(line)
    if not match:
        continue
    window_id, title, w, h = match.groups()
    if int(w) != width or int(h) != height:
        continue
    lower = title.lower()
    if "crosvm" in lower or "gpu" in lower or "display" in lower:
        print(window_id)
        print(line)
        raise SystemExit(0)
    fallback = (window_id, line)

if fallback:
    print(fallback[0])
    print(fallback[1])
    raise SystemExit(0)
raise SystemExit(1)
PY
}

WINDOW_INFO=""
for _ in $(seq 1 "$WAIT_SECS"); do
    if WINDOW_INFO="$(find_window 2>/dev/null)"; then
        break
    fi
    sleep 1
done

if [[ -z "$WINDOW_INFO" ]]; then
    echo "Missing host render window ${WIDTH}x${HEIGHT} on DISPLAY=$X_DISPLAY" >&2
    echo "Saved X11 tree: $TREE_FILE" >&2
    exit 1
fi

WINDOW_ID="$(printf '%s\n' "$WINDOW_INFO" | sed -n '1p')"
WINDOW_LINE="$(printf '%s\n' "$WINDOW_INFO" | sed -n '2p')"
{
    printf 'display=%s\n' "$X_DISPLAY"
    printf 'window_id=%s\n' "$WINDOW_ID"
    printf 'window_line=%s\n' "$WINDOW_LINE"
} >"$WINDOW_FILE"

if command -v xwd >/dev/null 2>&1; then
    DISPLAY="$X_DISPLAY" xwd -silent -id "$WINDOW_ID" -out "$XWD_FILE"
    if [[ ! -s "$XWD_FILE" ]]; then
        echo "Host window capture is empty: $XWD_FILE" >&2
        exit 1
    fi
fi

cat "$WINDOW_FILE"
if [[ -f "$XWD_FILE" ]]; then
    ls -l "$XWD_FILE"
fi
echo "Host window check passed: $WINDOW_ID"
