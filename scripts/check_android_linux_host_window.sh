#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

LOG_DIR="$REPO_ROOT/out/dist/logs/android-linux"
WAIT_SECS="60"
WIDTH="1280"
HEIGHT="720"
X_DISPLAY="${DISPLAY:-}"
MIN_NONBLACK_RATIO="0.001"
MIN_UNIQUE_COLORS="8"
MIN_MEAN_LUMA="8"
MIN_BRIGHT_RATIO="0.005"

usage() {
    cat <<EOF
Usage: $0 [options]

Options:
  --log-dir DIR       Output log dir (default: $LOG_DIR)
  --wait-secs N       Seconds to wait for a matching X11 window (default: $WAIT_SECS)
  --width N           Expected crosvm window width (default: $WIDTH)
  --height N          Expected crosvm window height (default: $HEIGHT)
  --x-display VALUE   X11 display (default: DISPLAY env)
  --min-nonblack R    Minimum sampled non-black pixel ratio (default: $MIN_NONBLACK_RATIO)
  --min-unique N      Minimum sampled unique RGB colors (default: $MIN_UNIQUE_COLORS)
  --min-mean-luma R   Minimum sampled mean luma on 0..255 scale (default: $MIN_MEAN_LUMA)
  --min-bright R      Minimum sampled pixel ratio with luma >= 40 (default: $MIN_BRIGHT_RATIO)
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
        --min-nonblack) MIN_NONBLACK_RATIO="$2"; shift 2 ;;
        --min-unique) MIN_UNIQUE_COLORS="$2"; shift 2 ;;
        --min-mean-luma) MIN_MEAN_LUMA="$2"; shift 2 ;;
        --min-bright) MIN_BRIGHT_RATIO="$2"; shift 2 ;;
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
METRICS_FILE="$LOG_DIR/host-window/crosvm-window.metrics.txt"

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
    python3 - "$XWD_FILE" "$METRICS_FILE" "$MIN_NONBLACK_RATIO" "$MIN_UNIQUE_COLORS" "$MIN_MEAN_LUMA" "$MIN_BRIGHT_RATIO" <<'PY'
import struct
import sys
from pathlib import Path

xwd_path = Path(sys.argv[1])
metrics_path = Path(sys.argv[2])
min_nonblack_ratio = float(sys.argv[3])
min_unique_colors = int(sys.argv[4])
min_mean_luma = float(sys.argv[5])
min_bright_ratio = float(sys.argv[6])

data = xwd_path.read_bytes()
if len(data) < 100:
    raise SystemExit(f"XWD capture is too small: {xwd_path}")

fields = struct.unpack(">25I", data[:100])
(
    header_size,
    file_version,
    pixmap_format,
    pixmap_depth,
    width,
    height,
    _xoffset,
    byte_order,
    _bitmap_unit,
    _bitmap_bit_order,
    _bitmap_pad,
    bits_per_pixel,
    bytes_per_line,
    _visual_class,
    red_mask,
    green_mask,
    blue_mask,
    _bits_per_rgb,
    _colormap_entries,
    ncolors,
    _window_width,
    _window_height,
    _window_x,
    _window_y,
    _window_bdrwidth,
) = fields

if file_version != 7 or pixmap_format != 2:
    raise SystemExit(f"Unsupported XWD format: version={file_version} pixmap_format={pixmap_format}")
if bits_per_pixel not in (24, 32):
    raise SystemExit(f"Unsupported XWD bits_per_pixel={bits_per_pixel}")

pixel_bytes = bits_per_pixel // 8
pixel_stride = bytes_per_line // width if width and bytes_per_line % width == 0 else pixel_bytes
if pixel_stride < pixel_bytes:
    pixel_stride = pixel_bytes
image_offset = header_size + ncolors * 12
image_bytes = bytes_per_line * height
if image_offset + image_bytes > len(data):
    raise SystemExit(
        f"Truncated XWD image: need {image_offset + image_bytes} bytes, got {len(data)}"
    )

image = memoryview(data)[image_offset : image_offset + image_bytes]
pixel_endian = ">" if byte_order == 1 else "<"
rgb_mask = red_mask | green_mask | blue_mask
step_x = max(1, width // 160)
step_y = max(1, height // 90)
samples = 0
nonblack = 0
bright = 0
luma_sum = 0.0
unique = set()

def scale_channel(pixel, mask):
    if not mask:
        return 0
    shift = (mask & -mask).bit_length() - 1
    raw = (pixel & mask) >> shift
    bits = mask.bit_count()
    return (raw * 255) // ((1 << bits) - 1)

for y in range(0, height, step_y):
    row_offset = y * bytes_per_line
    for x in range(0, width, step_x):
        offset = row_offset + x * pixel_stride
        if bits_per_pixel == 32:
            pixel = struct.unpack_from(pixel_endian + "I", image, offset)[0]
            red = scale_channel(pixel, red_mask)
            green = scale_channel(pixel, green_mask)
            blue = scale_channel(pixel, blue_mask)
        else:
            raw = bytes(image[offset : offset + 3])
            if byte_order == 1:
                red, green, blue = raw[0], raw[1], raw[2]
            else:
                blue, green, red = raw[0], raw[1], raw[2]
        rgb = (red << 16) | (green << 8) | blue
        luma = 0.2126 * red + 0.7152 * green + 0.0722 * blue
        samples += 1
        if rgb:
            nonblack += 1
        if luma >= 40:
            bright += 1
        luma_sum += luma
        if len(unique) <= min_unique_colors:
            unique.add(rgb)

nonblack_ratio = nonblack / samples if samples else 0.0
bright_ratio = bright / samples if samples else 0.0
mean_luma = luma_sum / samples if samples else 0.0
metrics = {
    "file": str(xwd_path),
    "width": width,
    "height": height,
    "pixmap_depth": pixmap_depth,
    "bits_per_pixel": bits_per_pixel,
    "pixel_stride": pixel_stride,
    "bytes_per_line": bytes_per_line,
    "samples": samples,
    "nonblack": nonblack,
    "nonblack_ratio": f"{nonblack_ratio:.6f}",
    "bright": bright,
    "bright_ratio": f"{bright_ratio:.6f}",
    "mean_luma": f"{mean_luma:.3f}",
    "unique_rgb_sample": len(unique),
    "min_nonblack_ratio": min_nonblack_ratio,
    "min_unique_colors": min_unique_colors,
    "min_mean_luma": min_mean_luma,
    "min_bright_ratio": min_bright_ratio,
}
metrics_path.write_text(
    "".join(f"{key}={value}\n" for key, value in metrics.items()),
    encoding="utf-8",
)

if (
    nonblack_ratio < min_nonblack_ratio
    or len(unique) < min_unique_colors
    or mean_luma < min_mean_luma
    or bright_ratio < min_bright_ratio
):
    raise SystemExit(
        "Host window capture is black, too dim, or static: "
        f"nonblack_ratio={nonblack_ratio:.6f}, unique_rgb_sample={len(unique)}, "
        f"mean_luma={mean_luma:.3f}, bright_ratio={bright_ratio:.6f}"
    )
PY
fi

cat "$WINDOW_FILE"
if [[ -f "$XWD_FILE" ]]; then
    ls -l "$XWD_FILE"
fi
if [[ -f "$METRICS_FILE" ]]; then
    cat "$METRICS_FILE"
fi
echo "Host window check passed: $WINDOW_ID"
