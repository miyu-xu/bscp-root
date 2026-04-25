#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd -P)"
SOURCE_ROOT=""
TARGET_ROOT=""
EXPECT_KERNEL_ARCH=""
FORCE=0

usage() {
    cat <<'EOF'
Usage: prepare_host_apex_tree.sh --source-root <path> --target-root <path> [options]

Options:
  --source-root <path>         Source apex tree root containing apex/com.android.virt
  --target-root <path>         Target apex tree root to populate/refresh
  --expect-kernel-arch <arch>  Validate microdroid_kernel architecture (e.g. arm64)
  --force                      Replace target apex/system/system_ext/vendor directories
  -h, --help                   Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --source-root)
            SOURCE_ROOT="$2"
            shift 2
            ;;
        --target-root)
            TARGET_ROOT="$2"
            shift 2
            ;;
        --expect-kernel-arch)
            EXPECT_KERNEL_ARCH="$2"
            shift 2
            ;;
        --force)
            FORCE=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage
            exit 2
            ;;
    esac
done

require_path() {
    local path="$1"
    local label="$2"
    [[ -e "$path" ]] || { echo "$label not found: $path" >&2; exit 1; }
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || { echo "Required command not found: $1" >&2; exit 1; }
}

canonical_path() {
    cd "$1" >/dev/null 2>&1 && pwd -P
}

sync_tree_dir() {
    local name="$1"
    local src="$SOURCE_ROOT/$name"
    local dst="$TARGET_ROOT/$name"

    if [[ ! -e "$src" ]]; then
        return 0
    fi

    if [[ "$SOURCE_ROOT_REAL" == "$TARGET_ROOT_REAL" ]]; then
        return 0
    fi

    rm -rf "$dst"
    mkdir -p "$(dirname "$dst")"
    cp -a "$src" "$dst"
}

extract_capex_original_apex() {
    local capex_path="$1"
    local target_apex="$2"
    mkdir -p "$(dirname "$target_apex")"
    unzip -p "$capex_path" original_apex > "$target_apex"
}

write_apex_info_list() {
    local mounted_apex_root="$TARGET_ROOT/apex"
    local decompressed_root="$mounted_apex_root/decompressed"
    local apex_info_list="$mounted_apex_root/apex-info-list.xml"
    local now
    now="$(date +%s)"

    mkdir -p "$mounted_apex_root" "$decompressed_root"

    {
        printf '%s\n' '<?xml version="1.0" encoding="utf-8"?>'
        printf '%s\n' '<apex-info-list>'

        local partition partition_dir file name module_name preinstalled_path module_path version_code decompressed_path
        for partition in system system_ext; do
            partition_dir="$TARGET_ROOT/$partition/apex"
            [[ -d "$partition_dir" ]] || continue
            while IFS= read -r file; do
                name="$(basename "$file")"
                module_name="${name%.*}"
                preinstalled_path="/$partition/apex/$name"
                if [[ "$name" == *.capex ]]; then
                    decompressed_path="$decompressed_root/$module_name.apex"
                    extract_capex_original_apex "$file" "$decompressed_path"
                    module_path="/apex/decompressed/$module_name.apex"
                    version_code="352090000"
                else
                    module_path="$preinstalled_path"
                    version_code="1"
                fi

                printf '  <apex-info moduleName="%s" modulePath="%s" preinstalledModulePath="%s" versionCode="%s" versionName="" isFactory="true" isActive="true" lastUpdateMillis="%s" provideSharedApexLibs="false" />\n' \
                    "$module_name" "$module_path" "$preinstalled_path" "$version_code" "$now"
            done < <(find "$partition_dir" -maxdepth 1 -type f \( -name '*.apex' -o -name '*.capex' \) | sort)
        done

        printf '%s\n' '</apex-info-list>'
    } > "$apex_info_list"
}

install_microdroid_fstab() {
    local dst_dir="$TARGET_ROOT/apex/com.android.virt/etc"
    local src="$SOURCE_ROOT/apex/com.android.virt/etc/fstab.microdroid"
    local dst="$dst_dir/fstab.microdroid"
    local repo_fallback="$REPO_ROOT/packages/modules/Virtualization/build/microdroid/fstab.microdroid"
    local tmp=""
    if [[ ! -f "$src" ]]; then
        src="$repo_fallback"
    fi
    if [[ -f "$src" ]]; then
        mkdir -p "$dst_dir"
        if [[ -f "$repo_fallback" ]] && [[ -e "$dst" ]] && [[ "$(cd "$(dirname "$src")" >/dev/null 2>&1 && pwd -P)/$(basename "$src")" == "$(cd "$(dirname "$dst")" >/dev/null 2>&1 && pwd -P)/$(basename "$dst")" ]]; then
            src="$repo_fallback"
        fi
        if [[ -e "$dst" ]] && [[ "$(cd "$(dirname "$src")" >/dev/null 2>&1 && pwd -P)/$(basename "$src")" == "$(cd "$(dirname "$dst")" >/dev/null 2>&1 && pwd -P)/$(basename "$dst")" ]]; then
            tmp="$(mktemp "$dst_dir/fstab.microdroid.XXXXXX")"
        else
            tmp="$dst"
        fi

        awk '
            /^[[:space:]]*($|#)/ { next }

            NF != 5 {
                printf "Unsupported microdroid fstab line (expected 5 fields): %s\n", $0 > "/dev/stderr"
                exit 1
            }

            { print $1, $2, $3, $4, $5 }
        ' "$src" > "$tmp"

        if [[ "$tmp" != "$dst" ]]; then
            mv "$tmp" "$dst"
        fi
    fi
}

validate_guest_kernel_arch() {
    local expected="$1"
    local kernel_path="$TARGET_ROOT/apex/com.android.virt/etc/fs/microdroid_kernel"
    local kernel_info

    require_path "$kernel_path" "Microdroid guest kernel"
    kernel_info="$(file -b "$kernel_path")"

    case "$expected" in
        arm64|aarch64)
            if ! grep -Eiq 'ARM aarch64|ARM64|arm64' <<<"$kernel_info"; then
                echo "Expected arm64 microdroid_kernel, found: $kernel_info" >&2
                exit 1
            fi
            ;;
        x86_64|x86|amd64)
            if ! grep -Eiq 'x86 boot executable|x86[-_ ]64|x86_64|amd64' <<<"$kernel_info"; then
                echo "Expected x86 microdroid_kernel, found: $kernel_info" >&2
                exit 1
            fi
            ;;
        *)
            if ! grep -Eiq "$expected" <<<"$kernel_info"; then
                echo "Expected microdroid_kernel to match '$expected', found: $kernel_info" >&2
                exit 1
            fi
            ;;
    esac
}

[[ -n "$SOURCE_ROOT" ]] || { usage >&2; exit 2; }
[[ -n "$TARGET_ROOT" ]] || { usage >&2; exit 2; }

require_command unzip
require_command file
require_path "$SOURCE_ROOT" "Source apex tree root"

if [[ -d "$SOURCE_ROOT/apex" ]]; then
    SOURCE_ROOT_REAL="$(canonical_path "$SOURCE_ROOT")"
elif [[ -d "$SOURCE_ROOT/com.android.virt" ]]; then
    SOURCE_ROOT_REAL="$(canonical_path "$SOURCE_ROOT/..")"
else
    echo "Source apex tree must contain apex/com.android.virt or point at the apex directory itself: $SOURCE_ROOT" >&2
    exit 1
fi

mkdir -p "$TARGET_ROOT"
TARGET_ROOT_REAL="$(canonical_path "$TARGET_ROOT")"

require_path "$SOURCE_ROOT_REAL/apex/com.android.virt" "Mounted com.android.virt apex"

SOURCE_ROOT="$SOURCE_ROOT_REAL"
TARGET_ROOT="$TARGET_ROOT_REAL"

sync_tree_dir apex
sync_tree_dir system
sync_tree_dir system_ext
sync_tree_dir vendor

require_path "$TARGET_ROOT/apex/com.android.virt" "Target com.android.virt apex"
write_apex_info_list
install_microdroid_fstab

if [[ -n "$EXPECT_KERNEL_ARCH" ]]; then
    validate_guest_kernel_arch "$EXPECT_KERNEL_ARCH"
fi

echo "Prepared host apex tree:"
echo "  source: $SOURCE_ROOT"
echo "  target: $TARGET_ROOT"
echo "  com.android.virt: $TARGET_ROOT/apex/com.android.virt"
echo "  apex-info-list: $TARGET_ROOT/apex/apex-info-list.xml"
