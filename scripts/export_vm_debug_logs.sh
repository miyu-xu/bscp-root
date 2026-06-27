#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

DIST_ROOT="$REPO_ROOT/out/dist"
OUTPUT_ROOT="/mnt/workspace/Windows/bscp-vm-debug-logs"
EXPORT_NAME="bscp-vm-debug-logs-$(date +%Y%m%d-%H%M%S)"
CREATE_ARCHIVE=0
ANDROID_LOG_DIR="$DIST_ROOT/logs/android-linux"
MICRODROID_LOG_DIR="$DIST_ROOT/logs/run-microdroid-smoke"

usage() {
    cat <<EOF
Usage: $0 [options]

Options:
  --dist-root DIR        bscp dist root (default: $DIST_ROOT)
  --output-root DIR      Output root (default: $OUTPUT_ROOT)
  --export-name NAME     Export directory name (default: timestamped)
  --android-log-dir DIR  Android log dir (default: $ANDROID_LOG_DIR)
  --microdroid-log-dir DIR
                         Microdroid log dir (default: $MICRODROID_LOG_DIR)
  --archive              Also create a compressed archive
  --no-archive           Only create the directory export (default)
  --help                 Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dist-root) DIST_ROOT="$2"; shift 2 ;;
        --output-root) OUTPUT_ROOT="$2"; shift 2 ;;
        --export-name) EXPORT_NAME="$2"; shift 2 ;;
        --android-log-dir) ANDROID_LOG_DIR="$2"; shift 2 ;;
        --microdroid-log-dir) MICRODROID_LOG_DIR="$2"; shift 2 ;;
        --archive) CREATE_ARCHIVE=1; shift ;;
        --no-archive) CREATE_ARCHIVE=0; shift ;;
        --help|-h) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

EXPORT_DIR="$OUTPUT_ROOT/$EXPORT_NAME"
mkdir -p "$EXPORT_DIR"

copy_dir_if_exists() {
    local src="$1"
    local dst="$2"
    if [[ -d "$src" ]]; then
        mkdir -p "$(dirname "$dst")"
        rsync -aL --delete "$src/" "$dst/"
    else
        mkdir -p "$dst"
        printf 'missing source: %s\n' "$src" >"$dst/MISSING.txt"
    fi
}

copy_dir_if_exists "$ANDROID_LOG_DIR" "$EXPORT_DIR/android-linux"
copy_dir_if_exists "$MICRODROID_LOG_DIR" "$EXPORT_DIR/microdroid-linux"

cat >"$EXPORT_DIR/README.txt" <<EOF
# BSCP VM debug logs

Created: $(date -Iseconds)
Repo: $REPO_ROOT
Repo HEAD: $(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || true)

android-linux:
- Full hvc/logcat/serial/stderr/stdout logs from direct-kernel Android Linux run.
- host-window/ contains X11 xwininfo data and crosvm-window.xwd when captured.
- adb/ contains gfxstream-angle.png and metrics from ADB-over-vsock screencap validation.

microdroid-linux:
- Full Linux Microdroid smoke logs from vm_linux.sh.
EOF

if [[ "$CREATE_ARCHIVE" -eq 1 ]]; then
    if command -v zstd >/dev/null 2>&1; then
        tar -C "$OUTPUT_ROOT" -cf - "$EXPORT_NAME" | zstd -T0 -19 -o "$OUTPUT_ROOT/$EXPORT_NAME.tar.zst"
        echo "$OUTPUT_ROOT/$EXPORT_NAME.tar.zst" >"$EXPORT_DIR/archive.path"
    else
        tar -C "$OUTPUT_ROOT" -czf "$OUTPUT_ROOT/$EXPORT_NAME.tar.gz" "$EXPORT_NAME"
        echo "$OUTPUT_ROOT/$EXPORT_NAME.tar.gz" >"$EXPORT_DIR/archive.path"
    fi
fi

du -sh "$EXPORT_DIR"
if [[ -f "$EXPORT_DIR/archive.path" ]]; then
    du -h "$(cat "$EXPORT_DIR/archive.path")"
fi
echo "Log export directory: $EXPORT_DIR"
