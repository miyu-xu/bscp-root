#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 LOG_DIR" >&2
    exit 2
fi

LOG_DIR="$1"
LOGCAT="$LOG_DIR/logcat-hvc2.txt"
KERNEL_LOG="$LOG_DIR/hvc.txt"
STDERR_LOG="$LOG_DIR/stderr.txt"

if [[ ! -f "$LOGCAT" || ! -f "$KERNEL_LOG" || ! -f "$STDERR_LOG" ]]; then
    echo "Missing Android/gfxstream logs under $LOG_DIR" >&2
    exit 1
fi

require_fixed() {
    local pattern="$1"
    local file="$2"
    local label="$3"
    if ! rg -q -F "$pattern" "$file"; then
        echo "Missing marker: $label" >&2
        exit 1
    fi
}

require_regex() {
    local pattern="$1"
    local file="$2"
    local label="$3"
    if ! rg -q "$pattern" "$file"; then
        echo "Missing marker: $label" >&2
        exit 1
    fi
}

reject_regex() {
    local pattern="$1"
    local file="$2"
    local label="$3"
    if rg -q "$pattern" "$file"; then
        echo "Unexpected marker: $label" >&2
        rg -n "$pattern" "$file" | head -n 20 >&2
        exit 1
    fi
}

"$(dirname "$0")/check_android_linux_markers.sh" "$LOG_DIR"

require_fixed "SurfaceFlinger: Boot is finished" "$LOGCAT" "SurfaceFlinger boot finished"
require_regex "RenderEngine: renderer  : ANGLE .*Vulkan" "$LOGCAT" \
    "RenderEngine uses ANGLE over Vulkan"
require_fixed "RenderEngine: version   : OpenGL ES" "$LOGCAT" "RenderEngine OpenGL ES version"

require_fixed "Gfxstream feature GuestVulkanOnly enabled" "$STDERR_LOG" \
    "gfxstream GuestVulkanOnly enabled"
require_fixed "Gfxstream feature VulkanAllocateHostMemory disabled" "$STDERR_LOG" \
    "gfxstream VulkanAllocateHostMemory disabled"
require_fixed "stream_renderer_init Gfxstream initialized successfully!" "$STDERR_LOG" \
    "gfxstream initialized"
require_fixed "Graphics Adapter" "$STDERR_LOG" "host graphics adapter selected"
require_regex "Created VkDevice:.*application:surfaceflinger engine:ANGLE" "$STDERR_LOG" \
    "surfaceflinger ANGLE VkDevice"
require_regex "Created VkDevice:.*application:com\\.android\\.launcher3 engine:ANGLE" "$STDERR_LOG" \
    "launcher ANGLE VkDevice"
require_fixed "update_scanout_resource type=Scanout" "$STDERR_LOG" "scanout resource updates"
require_fixed "flush_resource: id=" "$STDERR_LOG" "scanout resource flushes"

reject_regex "eglMakeCurrent failed|null ctx|vkGetMemoryHostPointerPropertiesEXT|Invalid device|\\[Vulkan Loader\\] ERROR|Segmentation fault|SIGSEGV" \
    "$STDERR_LOG" "host gfxstream/ANGLE failure"
reject_regex "RenderEngine:.*(failed|error)|SurfaceFlinger:.*(crash|fatal)|HWComposer.*(crash|fatal)|eglMakeCurrent failed|null ctx" \
    "$LOGCAT" "guest graphics failure"

echo "GFX marker check passed: android-linux gfxstream+ANGLE"
