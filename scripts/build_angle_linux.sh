#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

ANGLE_ROOT="${ANGLE_ROOT:-/opt/workspace/angle}"
ANGLE_OUT_DIR="${ANGLE_OUT_DIR:-$ANGLE_ROOT/out/Release-GfxAngle-Linux}"
DIST_ROOT="${DIST_ROOT:-$REPO_ROOT/out/dist}"
JOBS="${JOBS:-$(nproc)}"
RUN_GN=1
RUN_NINJA=1
RUN_STAGE=1

usage() {
    cat <<EOF
Usage: $0 [options]

Options:
  --angle-root DIR   ANGLE checkout (default: $ANGLE_ROOT)
  --out-dir DIR      ANGLE GN output dir (default: $ANGLE_OUT_DIR)
  --dist-root DIR    bscp dist root (default: $DIST_ROOT)
  --jobs N           Ninja parallelism (default: $JOBS)
  --gen-only         Generate build files, do not build or stage
  --no-stage         Build ANGLE but do not copy runtime libraries
  --stage-only       Copy existing runtime libraries, do not build
  --help             Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --angle-root) ANGLE_ROOT="$2"; shift 2 ;;
        --out-dir) ANGLE_OUT_DIR="$2"; shift 2 ;;
        --dist-root) DIST_ROOT="$2"; shift 2 ;;
        --jobs) JOBS="$2"; shift 2 ;;
        --gen-only) RUN_GN=1; RUN_NINJA=0; RUN_STAGE=0; shift ;;
        --no-stage) RUN_STAGE=0; shift ;;
        --stage-only) RUN_GN=0; RUN_NINJA=0; RUN_STAGE=1; shift ;;
        --help|-h) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

if [[ ! -d "$ANGLE_ROOT" ]]; then
    echo "Error: ANGLE_ROOT not found: $ANGLE_ROOT" >&2
    exit 1
fi

GN_BIN="$ANGLE_ROOT/buildtools/linux64/gn"
if [[ ! -x "$GN_BIN" ]]; then
    GN_BIN="$(command -v gn || true)"
fi
if [[ -z "$GN_BIN" || ! -x "$GN_BIN" ]]; then
    echo "Error: GN not found; expected $ANGLE_ROOT/buildtools/linux64/gn" >&2
    exit 1
fi

DEPOT_TOOLS_ROOT=""
for candidate in /opt/workspace/depot_tools "$ANGLE_ROOT/third_party/depot_tools"; do
    if [[ -x "$candidate/autoninja" && -f "$candidate/python3_bin_reldir.txt" ]]; then
        DEPOT_TOOLS_ROOT="$candidate"
        break
    fi
done
if [[ -z "$DEPOT_TOOLS_ROOT" ]]; then
    for candidate in "$ANGLE_ROOT/third_party/depot_tools" /opt/workspace/depot_tools; do
        if [[ -x "$candidate/autoninja" ]]; then
            DEPOT_TOOLS_ROOT="$candidate"
            break
        fi
    done
fi
AUTONINJA_BIN="${DEPOT_TOOLS_ROOT:+$DEPOT_TOOLS_ROOT/autoninja}"
if [[ -z "$AUTONINJA_BIN" || ! -x "$AUTONINJA_BIN" ]]; then
    AUTONINJA_BIN="$(command -v autoninja || true)"
fi
if [[ -z "$AUTONINJA_BIN" || ! -x "$AUTONINJA_BIN" ]]; then
    echo "Error: autoninja not found" >&2
    exit 1
fi

export PATH="$ANGLE_ROOT/buildtools/linux64${DEPOT_TOOLS_ROOT:+:$DEPOT_TOOLS_ROOT}:$PATH"

GN_ARGS='is_debug=false
is_component_build=false
target_cpu="x64"
angle_enable_vulkan=true
angle_enable_gl=true
angle_enable_null=false
angle_enable_wgpu=false
angle_enable_cl=false
clang_use_chrome_plugins=false
symbol_level=1'

if [[ "$RUN_GN" -eq 1 ]]; then
    echo "[gn] $ANGLE_OUT_DIR"
    (cd "$ANGLE_ROOT" && "$GN_BIN" gen "$ANGLE_OUT_DIR" --args="$GN_ARGS")
fi
if [[ "$RUN_NINJA" -eq 1 ]]; then
    echo "[ninja] libEGL libGLESv2"
    "$AUTONINJA_BIN" -C "$ANGLE_OUT_DIR" -j "$JOBS" libEGL libGLESv2
fi

if [[ "$RUN_STAGE" -eq 1 && ( ! -f "$ANGLE_OUT_DIR/libEGL.so" || ! -f "$ANGLE_OUT_DIR/libGLESv2.so" ) ]]; then
    echo "Error: ANGLE runtime not found in $ANGLE_OUT_DIR" >&2
    exit 1
fi

if [[ "$RUN_STAGE" -eq 1 ]]; then
    STAGE_DIR="$DIST_ROOT/linux/gfx/angle"
    mkdir -p "$STAGE_DIR"
    cp -af "$ANGLE_OUT_DIR"/libEGL.so* "$STAGE_DIR/"
    cp -af "$ANGLE_OUT_DIR"/libGLESv2.so* "$STAGE_DIR/"
    if compgen -G "$ANGLE_OUT_DIR/libGLESv1_CM.so*" >/dev/null; then
        cp -af "$ANGLE_OUT_DIR"/libGLESv1_CM.so* "$STAGE_DIR/"
    fi
    for optional in libvulkan.so* libvk_swiftshader.so* libVkICD_mock_icd.so* vk_swiftshader_icd.json; do
        if compgen -G "$ANGLE_OUT_DIR/$optional" >/dev/null; then
            cp -af "$ANGLE_OUT_DIR"/$optional "$STAGE_DIR/"
        fi
    done
    echo "Staged ANGLE runtime: $STAGE_DIR"
fi
