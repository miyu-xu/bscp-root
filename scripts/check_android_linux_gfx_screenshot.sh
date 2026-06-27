#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

LOG_DIR="$REPO_ROOT/out/dist/logs/android-linux"
CID="100"
LOCAL_PORT="8555"
WAIT_SECS="120"
SCREENSHOT_RETRIES="10"
SCREENSHOT_DELAY_SECS="2"
ADB_BIN="/opt/workspace/aosp/out/host/linux-x86/bin/adb"
REMOTE_SCREENSHOT="/data/local/tmp/gfxstream-angle.png"

usage() {
    cat <<EOF
Usage: $0 [options]

Options:
  --log-dir DIR       Android log dir (default: $LOG_DIR)
  --cid N             Guest vsock CID (default: $CID)
  --local-port N      Local TCP port for the temporary ADB bridge (default: $LOCAL_PORT)
  --wait-secs N       Seconds to wait for guest adbd marker (default: $WAIT_SECS)
  --screenshot-retries N
                       Number of screencap attempts before failing (default: $SCREENSHOT_RETRIES)
  --screenshot-delay-secs N
                       Delay between screencap attempts (default: $SCREENSHOT_DELAY_SECS)
  --adb PATH          adb binary (default: $ADB_BIN)
  --help              Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --log-dir) LOG_DIR="$2"; shift 2 ;;
        --cid) CID="$2"; shift 2 ;;
        --local-port) LOCAL_PORT="$2"; shift 2 ;;
        --wait-secs) WAIT_SECS="$2"; shift 2 ;;
        --screenshot-retries) SCREENSHOT_RETRIES="$2"; shift 2 ;;
        --screenshot-delay-secs) SCREENSHOT_DELAY_SECS="$2"; shift 2 ;;
        --adb) ADB_BIN="$2"; shift 2 ;;
        --help|-h) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

if [[ ! -x "$ADB_BIN" ]]; then
    ADB_BIN="$(command -v adb || true)"
fi
if [[ -z "$ADB_BIN" || ! -x "$ADB_BIN" ]]; then
    echo "Error: adb not found" >&2
    exit 1
fi

LOGCAT="$LOG_DIR/logcat-hvc2.txt"
OUT_DIR="$LOG_DIR/adb"
SCREENSHOT="$OUT_DIR/gfxstream-angle.png"
METRICS="$OUT_DIR/gfxstream-angle.metrics.txt"
BRIDGE_LOG="$OUT_DIR/adb-vsock-bridge.log"
SERIAL="127.0.0.1:$LOCAL_PORT"
BRIDGE_PID=""

cleanup() {
    "$ADB_BIN" disconnect "$SERIAL" >/dev/null 2>&1 || true
    if [[ -n "$BRIDGE_PID" ]]; then
        kill "$BRIDGE_PID" >/dev/null 2>&1 || true
        wait "$BRIDGE_PID" >/dev/null 2>&1 || true
    fi
}
trap cleanup EXIT

if [[ ! -f "$LOGCAT" ]]; then
    echo "Error: missing logcat: $LOGCAT" >&2
    exit 1
fi

mkdir -p "$OUT_DIR"

for _ in $(seq 1 "$WAIT_SECS"); do
    if rg -q -F "adbd listening on vsock:5555" "$LOGCAT"; then
        break
    fi
    sleep 1
done
if ! rg -q -F "adbd listening on vsock:5555" "$LOGCAT"; then
    echo "Missing marker: adbd listening on vsock:5555" >&2
    exit 1
fi

for _ in $(seq 1 "$WAIT_SECS"); do
    if "$SCRIPT_DIR/check_android_linux_gfx_markers.sh" "$LOG_DIR" >/dev/null 2>&1; then
        break
    fi
    sleep 1
done
"$SCRIPT_DIR/check_android_linux_gfx_markers.sh" "$LOG_DIR" >/dev/null

python3 - "$CID" "$LOCAL_PORT" >"$BRIDGE_LOG" 2>&1 <<'PY' &
import socket
import sys
import threading

cid = int(sys.argv[1])
local_port = int(sys.argv[2])
guest_port = 5555

def close_quietly(sock):
    try:
        sock.shutdown(socket.SHUT_RDWR)
    except OSError:
        pass
    try:
        sock.close()
    except OSError:
        pass

def pump(src, dst):
    try:
        while True:
            data = src.recv(65536)
            if not data:
                break
            dst.sendall(data)
    except OSError:
        pass
    finally:
        close_quietly(src)
        close_quietly(dst)

listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
listener.bind(("127.0.0.1", local_port))
listener.listen(16)
print(f"bridge listening 127.0.0.1:{local_port} -> vsock:{cid}:{guest_port}", flush=True)

while True:
    client, _ = listener.accept()
    try:
        guest = socket.socket(socket.AF_VSOCK, socket.SOCK_STREAM)
        guest.connect((cid, guest_port))
    except OSError as e:
        print(f"connect vsock failed: {e}", flush=True)
        close_quietly(client)
        continue
    threading.Thread(target=pump, args=(client, guest), daemon=True).start()
    threading.Thread(target=pump, args=(guest, client), daemon=True).start()
PY
BRIDGE_PID="$!"

for _ in $(seq 1 20); do
    if (echo >"/dev/tcp/127.0.0.1/$LOCAL_PORT") >/dev/null 2>&1; then
        break
    fi
    sleep 0.25
done

"$ADB_BIN" disconnect "$SERIAL" >/dev/null 2>&1 || true
"$ADB_BIN" connect "$SERIAL" >/dev/null
"$ADB_BIN" -s "$SERIAL" wait-for-device

BOOT_COMPLETED="$("$ADB_BIN" -s "$SERIAL" shell getprop sys.boot_completed | tr -d '\r')"
if [[ "$BOOT_COMPLETED" != "1" ]]; then
    echo "Unexpected sys.boot_completed: $BOOT_COMPLETED" >&2
    exit 1
fi

verify_screenshot() {
    python3 - "$SCREENSHOT" "$METRICS" <<'PY'
from pathlib import Path
import sys
from PIL import Image, ImageStat

image_path = Path(sys.argv[1])
metrics_path = Path(sys.argv[2])

image = Image.open(image_path).convert("RGBA")
stat = ImageStat.Stat(image)
colors = image.getcolors(maxcolors=10_000_000)
unique = len(colors) if colors is not None else -1
bbox = image.getbbox()
extrema = stat.extrema
mean = [round(value, 2) for value in stat.mean]

metrics = [
    f"path={image_path}",
    f"size={image.size[0]}x{image.size[1]}",
    f"mode={image.mode}",
    f"bbox={bbox}",
    f"unique_colors={unique}",
    f"mean_rgba={mean}",
    f"extrema={extrema}",
]
metrics_path.write_text("\n".join(metrics) + "\n", encoding="utf-8")

if image.size != (1280, 720):
    raise SystemExit(f"unexpected screenshot size: {image.size}")
if bbox is None:
    raise SystemExit("blank screenshot: empty bbox")
if unique != -1 and unique < 2:
    raise SystemExit(f"blank screenshot: only {unique} color")
if all(channel == (0, 0) for channel in extrema[:3]):
    raise SystemExit("blank screenshot: RGB extrema are all zero")

print("\n".join(metrics))
PY
}

SCREENSHOT_OK=0
for attempt in $(seq 1 "$SCREENSHOT_RETRIES"); do
    "$ADB_BIN" -s "$SERIAL" shell screencap -p "$REMOTE_SCREENSHOT"
    "$ADB_BIN" -s "$SERIAL" pull "$REMOTE_SCREENSHOT" "$SCREENSHOT" >/dev/null
    if verify_screenshot; then
        SCREENSHOT_OK=1
        break
    fi
    if [[ "$attempt" -lt "$SCREENSHOT_RETRIES" ]]; then
        sleep "$SCREENSHOT_DELAY_SECS"
    fi
done

if [[ "$SCREENSHOT_OK" -ne 1 ]]; then
    echo "GFX screenshot check failed after $SCREENSHOT_RETRIES attempts" >&2
    exit 1
fi

echo "GFX screenshot check passed: $SCREENSHOT"
