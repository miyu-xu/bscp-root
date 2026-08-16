#!/usr/bin/env bash
# Single host build script: binder-rpc (CMake) -> copy libs for Rust -> cargo virtmgr+vm+crosvm -> dist/<platform>
# Usage: from repo root:  chmod +x build_all.sh && ./build_all.sh
#        ./build_all.sh --clean   (force clean rebuild from scratch)
# Override triple: RUST_TARGET=aarch64-unknown-linux-gnu ./build_all.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$REPO_ROOT"
OUT_ROOT="$REPO_ROOT/out"
CMAKE_BUILD_DIR="$OUT_ROOT/build"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$OUT_ROOT/target}"

# --- CLI options ---
CLEAN_BUILD=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --clean|-c) CLEAN_BUILD=1; shift ;;
        --help|-h)
            echo "Usage: $0 [--clean]"
            echo "  --clean  Force clean rebuild (removes CMake cache and cargo target dir)"
            exit 0 ;;
        *) echo "Unknown option: $1 (use --clean for clean rebuild)" >&2; exit 2 ;;
    esac
done

if [[ "$CLEAN_BUILD" -eq 1 ]]; then
    echo "=== Clean rebuild requested ==="
    echo "Removing CMake build dir: $CMAKE_BUILD_DIR"
    rm -rf "$CMAKE_BUILD_DIR"
    echo "Removing cargo target dir: $CARGO_TARGET_DIR"
    rm -rf "$CARGO_TARGET_DIR"
    echo "Removing dist dir"
    rm -rf "$OUT_ROOT/dist"
fi

resolve_working_ninja() {
    local candidate=""
    local version_check=""

    if candidate="$(command -v ninja 2>/dev/null)"; then
        if version_check="$("$candidate" --version 2>/dev/null)" && [[ -n "$version_check" ]]; then
            printf '%s\n' "$candidate"
            return 0
        fi
    fi

    for candidate in /opt/homebrew/bin/ninja /usr/local/bin/ninja; do
        if [[ -x "$candidate" ]]; then
            if version_check="$("$candidate" --version 2>/dev/null)" && [[ -n "$version_check" ]]; then
                printf '%s\n' "$candidate"
                return 0
            fi
        fi
    done

    if command -v brew >/dev/null 2>&1; then
        local brew_prefix=""
        if brew_prefix="$(brew --prefix ninja 2>/dev/null)"; then
            candidate="$brew_prefix/bin/ninja"
            if [[ -x "$candidate" ]]; then
                if version_check="$("$candidate" --version 2>/dev/null)" && [[ -n "$version_check" ]]; then
                    printf '%s\n' "$candidate"
                    return 0
                fi
            fi
        fi
    fi

    return 1
}

# Host OS defaults (Darwin / Linux). Windows developers use build_all.bat.
OS_NAME="$(uname -s)"
case "$OS_NAME" in
    Darwin)
        RUST_TARGET="${RUST_TARGET:-aarch64-apple-darwin}"
        LIB_EXT="dylib"
        DIST_DIR_NAME="macos"
        DEFAULT_CROSVM_FEATURES="hvf,default-no-sandbox,config-file,qcow,balloon,android-sparse,composite-disk,tokio,net,audio"
        CROSVM_CARGO_TOOLCHAIN="${CROSVM_CARGO_TOOLCHAIN:-nightly}"
        DEFAULT_ENABLE_GFXSTREAM_ANGLE=1
        HOST_APEX_TREE_SOURCE="${MACOS_AVF_APEX_TREE_SOURCE:-${HOST_APEX_TREE_SOURCE:-}}"
        ;;
    Linux)
        ARCH="$(uname -m)"
        if [[ "$ARCH" == "aarch64" ]]; then
            RUST_TARGET="${RUST_TARGET:-aarch64-unknown-linux-gnu}"
        else
            RUST_TARGET="${RUST_TARGET:-x86_64-unknown-linux-gnu}"
        fi
        LIB_EXT="so"
        DIST_DIR_NAME="linux"
        DEFAULT_CROSVM_FEATURES="default-no-sandbox,config-file,qcow,balloon,android-sparse,composite-disk,net"
        CROSVM_CARGO_TOOLCHAIN="${CROSVM_CARGO_TOOLCHAIN:-stable}"
        DEFAULT_ENABLE_GFXSTREAM_ANGLE=0
        ;;
    *)
        echo "Error: unsupported OS from uname: $OS_NAME (use Linux/macOS or build_all.bat on Windows)"
        exit 1
        ;;
esac

DIST="$OUT_ROOT/dist/$DIST_DIR_NAME"
DIST_BIN="$DIST/bin"
DIST_LIB="$DIST/lib"
DIST_GFX_ANGLE="$DIST/gfx/angle"
BINDER_LIB="libbinder-rpc.$LIB_EXT"
GFXSTREAM_BUILD_DIR="$OUT_ROOT/gfxstream_build_$DIST_DIR_NAME"
GFXSTREAM_LIB="libgfxstream_backend.$LIB_EXT"
ANGLE_ROOT="${ANGLE_ROOT:-$REPO_ROOT/../angle}"
AEMU_COMMON_PATH="${AEMU_COMMON_PATH:-$REPO_ROOT/hardware/google/aemu}"
FLATBUFFERS_PATH="${FLATBUFFERS_PATH:-$REPO_ROOT/external/flatbuffers}"
ANGLE_RUNTIME_DIR="${ANGLE_RUNTIME_DIR:-}"
ENABLE_GFXSTREAM_ANGLE="${ENABLE_GFXSTREAM_ANGLE:-$DEFAULT_ENABLE_GFXSTREAM_ANGLE}"
GFXSTREAM_PATH="${GFXSTREAM_PATH:-}"
MOLTENVK_ROOT="${MOLTENVK_ROOT:-$REPO_ROOT/../MoltenVK}"
MOLTENVK_RUNTIME_DIR="${MOLTENVK_RUNTIME_DIR:-}"
VULKAN_LOADER_PATH="${VULKAN_LOADER_PATH:-}"
MACOS_CROSVM_ENTITLEMENTS="$REPO_ROOT/scripts/macos_crosvm.entitlements"
PREPARE_APEX_TREE_SCRIPT="$REPO_ROOT/scripts/prepare_host_apex_tree.sh"
DIST_APEX_TREE="$OUT_ROOT/dist/apex_dir"
HOST_APEX_TREE_SOURCE="${HOST_APEX_TREE_SOURCE:-}"
TOTAL_STEPS=4
RUST_STEP=3
DIST_STEP=4
if [[ "$ENABLE_GFXSTREAM_ANGLE" == "1" ]]; then
    TOTAL_STEPS=5
    RUST_STEP=4
    DIST_STEP=5
fi

append_csv_feature() {
    local list="$1"
    local feature="$2"
    case ",$list," in
        *",$feature,"*) printf '%s\n' "$list" ;;
        *)
            if [[ -z "$list" ]]; then
                printf '%s\n' "$feature"
            else
                printf '%s\n' "$list,$feature"
            fi
            ;;
    esac
}

find_angle_runtime_dir() {
    local angle_lib="libEGL.$LIB_EXT"

    if [[ -n "$ANGLE_RUNTIME_DIR" && -f "$ANGLE_RUNTIME_DIR/$angle_lib" ]]; then
        printf '%s\n' "$ANGLE_RUNTIME_DIR"
        return 0
    fi

    if [[ ! -d "$ANGLE_ROOT/out" ]]; then
        return 1
    fi

    local match=""
    match="$(find "$ANGLE_ROOT/out" -type f -name "$angle_lib" 2>/dev/null | head -n 1 || true)"
    if [[ -n "$match" ]]; then
        dirname "$match"
        return 0
    fi

    return 1
}

find_moltenvk_runtime_dir() {
    if [[ -n "$MOLTENVK_RUNTIME_DIR" && -f "$MOLTENVK_RUNTIME_DIR/libMoltenVK.dylib" ]]; then
        printf '%s\n' "$MOLTENVK_RUNTIME_DIR"
        return 0
    fi

    local candidate="$MOLTENVK_ROOT/Package/Latest/MoltenVK/dynamic/dylib/macOS"
    if [[ -f "$candidate/libMoltenVK.dylib" ]]; then
        printf '%s\n' "$candidate"
        return 0
    fi

    return 1
}

find_macos_vulkan_loader() {
    local candidate=""
    if [[ -n "$VULKAN_LOADER_PATH" && -f "$VULKAN_LOADER_PATH" ]]; then
        printf '%s\n' "$VULKAN_LOADER_PATH"
        return 0
    fi
    for candidate in /opt/homebrew/lib/libvulkan.dylib /usr/local/lib/libvulkan.dylib; do
        if [[ -f "$candidate" ]]; then
            printf '%s\n' "$candidate"
            return 0
        fi
    done
    if command -v brew >/dev/null 2>&1; then
        local prefix=""
        prefix="$(brew --prefix vulkan-loader 2>/dev/null || true)"
        candidate="$prefix/lib/libvulkan.dylib"
        if [[ -n "$prefix" && -f "$candidate" ]]; then
            printf '%s\n' "$candidate"
            return 0
        fi
    fi
    return 1
}

stage_angle_runtime() {
    local angle_dir="$1"
    mkdir -p "$DIST_GFX_ANGLE"

    cp -f "$angle_dir/libEGL.$LIB_EXT" "$DIST_GFX_ANGLE/"
    cp -f "$angle_dir/libGLESv2.$LIB_EXT" "$DIST_GFX_ANGLE/"
    if [[ -f "$angle_dir/libGLESv1_CM.$LIB_EXT" ]]; then
        cp -f "$angle_dir/libGLESv1_CM.$LIB_EXT" "$DIST_GFX_ANGLE/"
    fi

    if [[ "$OS_NAME" == "Darwin" ]]; then
        local moltenvk_dir=""
        moltenvk_dir="$(find_moltenvk_runtime_dir || true)"
        if [[ -n "$moltenvk_dir" ]]; then
            cp -f "$moltenvk_dir/libMoltenVK.dylib" "$DIST_GFX_ANGLE/"
        fi
        if [[ -f "$MOLTENVK_ROOT/MoltenVK/icd/MoltenVK_icd.json" ]]; then
            cp -f "$MOLTENVK_ROOT/MoltenVK/icd/MoltenVK_icd.json" "$DIST_GFX_ANGLE/"
        fi
    fi
}

first_glob_match() {
    local pattern="$1"
    local match=""
    while IFS= read -r match; do
        printf '%s\n' "$match"
        return 0
    done < <(compgen -G "$pattern")
    return 1
}

write_crosvm_angle_wrapper() {
    local wrapper="$DIST_BIN/crosvm-angle"
    cat >"$wrapper" <<EOF
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
DIST_ROOT="\$(cd "\$SCRIPT_DIR/.." && pwd)"
ANGLE_ROOT="\$DIST_ROOT/gfx/angle"
export GFXSTREAM_ANGLE_ROOT="\$ANGLE_ROOT"
if [[ -d "\$DIST_ROOT/lib" ]]; then
  if [[ "\$(uname -s)" == "Darwin" ]]; then
    export DYLD_LIBRARY_PATH="\$DIST_ROOT/lib:\$ANGLE_ROOT\${DYLD_LIBRARY_PATH:+:\$DYLD_LIBRARY_PATH}"
  else
    export LD_LIBRARY_PATH="\$DIST_ROOT/lib:\$ANGLE_ROOT\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}"
  fi
fi
if [[ -f "\$ANGLE_ROOT/MoltenVK_icd.json" ]]; then
  export VK_ICD_FILENAMES="\$ANGLE_ROOT/MoltenVK_icd.json"
fi
exec "\$SCRIPT_DIR/crosvm" "\$@"
EOF
    chmod +x "$wrapper"
}

echo "=== Host build (Unix) ==="
echo "REPO_ROOT=$REPO_ROOT"
echo "OS_NAME=$OS_NAME"
echo "RUST_TARGET=$RUST_TARGET"
echo "LIB_EXT=$LIB_EXT"
echo "DIST=$DIST"

command -v cargo >/dev/null || { echo "Error: cargo not on PATH"; exit 1; }
command -v rustc >/dev/null || { echo "Error: rustc not on PATH"; exit 1; }
cargo -V
rustc -V
if ! cargo +"$CROSVM_CARGO_TOOLCHAIN" -V >/dev/null 2>&1; then
  echo "Error: cargo +$CROSVM_CARGO_TOOLCHAIN is not available"
  exit 1
fi

echo
echo "[1/$TOTAL_STEPS] binder-rpc (CMake)"
if ! command -v cmake >/dev/null; then
  echo "Error: cmake not found"
  exit 1
fi
CMAKE_GEN="${CMAKE_GENERATOR:-Ninja}"
CMAKE_ARGS=(
  -B "$CMAKE_BUILD_DIR"
  -G "$CMAKE_GEN"
  -DCMAKE_BUILD_TYPE=Release
)

if [[ "$CMAKE_GEN" == "Ninja" ]]; then
  if ! CMAKE_MAKE_PROGRAM_PATH="$(resolve_working_ninja)"; then
    echo "Error: a working ninja executable is required for CMake generator '$CMAKE_GEN'"
    echo "Hint: install ninja (for example with Homebrew on macOS) and ensure it is not shadowed by depot_tools."
    exit 1
  fi
  CMAKE_ARGS+=("-DCMAKE_MAKE_PROGRAM=$CMAKE_MAKE_PROGRAM_PATH")
fi

if [[ "$OS_NAME" == "Darwin" ]]; then
  command -v xcrun >/dev/null || { echo "Error: xcrun not found"; exit 1; }
  MACOS_CLANG="$(xcrun --find clang)"
  MACOS_CLANGXX="$(xcrun --find clang++)"
  MACOS_SDKROOT="$(xcrun --sdk macosx --show-sdk-path)"
  CMAKE_ARGS+=(
    "-DCMAKE_C_COMPILER=$MACOS_CLANG"
    "-DCMAKE_CXX_COMPILER=$MACOS_CLANGXX"
    "-DCMAKE_OSX_SYSROOT=$MACOS_SDKROOT"
  )
  # The build directory is reused across runs, so drop the cache before switching
  # to an explicit Xcode toolchain/SDK to avoid CMake reusing stale compiler paths.
  rm -f "$CMAKE_BUILD_DIR/CMakeCache.txt"
fi

mkdir -p "$CMAKE_BUILD_DIR"
if ! cmake "${CMAKE_ARGS[@]}" .; then
  echo "Error: cmake configure failed"
  exit 1
fi
cmake --build "$CMAKE_BUILD_DIR" --parallel
BLIB="$CMAKE_BUILD_DIR/lib"

echo
echo "[2/$TOTAL_STEPS] Copy libbinder-rpc into binder rust/sys/libs"
SYS_LIBS="$REPO_ROOT/frameworks/native/libs/binder/rust/sys/libs"
mkdir -p "$SYS_LIBS"
if [[ -f "$BLIB/$BINDER_LIB" ]]; then
  cp -f "$BLIB/$BINDER_LIB" "$SYS_LIBS/"
  echo "Copied $BINDER_LIB"
else
  echo "Error: missing $BLIB/$BINDER_LIB"
  exit 1
fi

# Export loader path for tools that resolve the shared library at runtime
if [[ "$OS_NAME" == "Darwin" ]]; then
  export DYLD_LIBRARY_PATH="$BLIB:${DYLD_LIBRARY_PATH:-}"
else
  export LD_LIBRARY_PATH="$BLIB:${LD_LIBRARY_PATH:-}"
fi

echo
if [[ "$ENABLE_GFXSTREAM_ANGLE" == "1" ]]; then
  echo "[3/$TOTAL_STEPS] gfxstream backend"
  if [[ -n "$GFXSTREAM_PATH" && -f "$GFXSTREAM_PATH/$GFXSTREAM_LIB" ]]; then
    echo "Using existing gfxstream backend: $GFXSTREAM_PATH"
  else
    if [[ ! -d "$ANGLE_ROOT" ]]; then
      echo "Error: ANGLE_ROOT not found: $ANGLE_ROOT"
      exit 1
    fi
    if [[ ! -d "$AEMU_COMMON_PATH" ]]; then
      echo "Error: AEMU_COMMON_PATH not found: $AEMU_COMMON_PATH"
      exit 1
    fi
    if [[ ! -d "$FLATBUFFERS_PATH" ]]; then
      echo "Error: FLATBUFFERS_PATH not found: $FLATBUFFERS_PATH"
      exit 1
    fi
    GFXSTREAM_CMAKE_ARGS=(
      -S "$REPO_ROOT/hardware/google/gfxstream"
      -B "$GFXSTREAM_BUILD_DIR"
      -G "$CMAKE_GEN"
      -DCMAKE_BUILD_TYPE=Release
      "-DANGLE_PATH=$ANGLE_ROOT"
      "-DAEMU_COMMON_PATH=$AEMU_COMMON_PATH"
      "-DFLATBUFFERS_PATH=$FLATBUFFERS_PATH"
      "-DMOLTENVK_ROOT=$MOLTENVK_ROOT"
      "-DMOLTENVK_RUNTIME_DIR=$MOLTENVK_RUNTIME_DIR"
    )

    if [[ "$CMAKE_GEN" == "Ninja" ]]; then
      GFXSTREAM_CMAKE_ARGS+=("-DCMAKE_MAKE_PROGRAM=$CMAKE_MAKE_PROGRAM_PATH")
    fi

    if [[ "$OS_NAME" == "Darwin" ]]; then
      GFXSTREAM_CMAKE_ARGS+=(
        "-DCMAKE_C_COMPILER=$MACOS_CLANG"
        "-DCMAKE_CXX_COMPILER=$MACOS_CLANGXX"
        "-DCMAKE_OSX_SYSROOT=$MACOS_SDKROOT"
      )
      rm -f "$GFXSTREAM_BUILD_DIR/CMakeCache.txt"
    fi

    mkdir -p "$GFXSTREAM_BUILD_DIR"
    if ! cmake "${GFXSTREAM_CMAKE_ARGS[@]}"; then
      echo "Error: gfxstream cmake configure failed. Set GFXSTREAM_PATH to a prebuilt backend or provide AEMU_COMMON_PATH and FLATBUFFERS_PATH."
      exit 1
    fi
    cmake --build "$GFXSTREAM_BUILD_DIR" --target gfxstream_backend --parallel
    GFXSTREAM_PATH="$GFXSTREAM_BUILD_DIR"
  fi
  export GFXSTREAM_PATH
fi

echo
echo "[$RUST_STEP/$TOTAL_STEPS] Rust (virtmgr + vm + crosvm)"
export CARGO_TARGET_DIR
CROSVM_FEATURES="${CROSVM_FEATURES:-$DEFAULT_CROSVM_FEATURES}"
if [[ "$ENABLE_GFXSTREAM_ANGLE" == "1" ]]; then
  CROSVM_FEATURES="$(append_csv_feature "$CROSVM_FEATURES" "gpu")"
  CROSVM_FEATURES="$(append_csv_feature "$CROSVM_FEATURES" "gfxstream")"
  if [[ "$OS_NAME" == "Linux" ]]; then
    CROSVM_FEATURES="$(append_csv_feature "$CROSVM_FEATURES" "x")"
    CROSVM_FEATURES="$(append_csv_feature "$CROSVM_FEATURES" "vulkan_display")"
    CROSVM_FEATURES="$(append_csv_feature "$CROSVM_FEATURES" "vulkano")"
  fi
fi
echo "[cargo] virtmgr --release --target $RUST_TARGET"
cargo build --manifest-path "$REPO_ROOT/packages/modules/Virtualization/android/virtmgr/Cargo.toml" --release --target "$RUST_TARGET"
echo "[cargo] vm --release --target $RUST_TARGET"
cargo build --manifest-path "$REPO_ROOT/packages/modules/Virtualization/android/vm/Cargo.toml" --release --target "$RUST_TARGET"
echo "[cargo] crosvm +$CROSVM_CARGO_TOOLCHAIN --release -p crosvm --target $RUST_TARGET --no-default-features --features $CROSVM_FEATURES"
(
  cd "$REPO_ROOT/external/crosvm"
  cargo +"$CROSVM_CARGO_TOOLCHAIN" build --release -p crosvm --target "$RUST_TARGET" \
    --no-default-features --features "$CROSVM_FEATURES"
)

echo
echo "[$DIST_STEP/$TOTAL_STEPS] Collect artifacts into dist/$DIST_DIR_NAME"
mkdir -p "$DIST_BIN" "$DIST_LIB"
TGT_OUT="$CARGO_TARGET_DIR/$RUST_TARGET/release"

cp -f "$CMAKE_BUILD_DIR/bin/"* "$DIST_BIN/" 2>/dev/null || true
if compgen -G "$BLIB/libbinder-rpc*.$LIB_EXT" >/dev/null; then
  cp -af "$BLIB"/libbinder-rpc*."$LIB_EXT" "$DIST_LIB/"
else
  echo "Error: missing $BINDER_LIB under $BLIB"
  exit 1
fi
if [[ "$ENABLE_GFXSTREAM_ANGLE" == "1" ]]; then
  if compgen -G "$GFXSTREAM_BUILD_DIR/*.$LIB_EXT" >/dev/null; then
    cp -af "$GFXSTREAM_BUILD_DIR"/*."$LIB_EXT" "$DIST_LIB/" 2>/dev/null || true
  fi
  if [[ ! -f "$DIST_LIB/$GFXSTREAM_LIB" ]]; then
    echo "Error: missing $GFXSTREAM_LIB under $GFXSTREAM_BUILD_DIR"
    exit 1
  fi
  if [[ "$OS_NAME" == "Darwin" ]]; then
    if ! macos_vulkan_loader="$(find_macos_vulkan_loader)"; then
      echo "Error: macOS gfxstream requires libvulkan.dylib; set VULKAN_LOADER_PATH." >&2
      exit 1
    fi
    cp -f "$macos_vulkan_loader" "$DIST_LIB/libvulkan.dylib"
    echo "Staged macOS Vulkan loader: $macos_vulkan_loader"
  fi
fi
if [[ -f "$TGT_OUT/virtmgr" ]]; then
  cp -f "$TGT_OUT/virtmgr" "$DIST_BIN/"
fi
if [[ -f "$TGT_OUT/vm" ]]; then
  cp -f "$TGT_OUT/vm" "$DIST_BIN/"
fi
if [[ -f "$TGT_OUT/crosvm" ]]; then
  cp -f "$TGT_OUT/crosvm" "$DIST_BIN/"
fi

if [[ "$ENABLE_GFXSTREAM_ANGLE" == "1" ]]; then
  if ! angle_runtime_dir="$(find_angle_runtime_dir)"; then
    echo "Error: ANGLE runtime libraries not found. Set ANGLE_RUNTIME_DIR or build ANGLE under $ANGLE_ROOT/out."
    exit 1
  fi
  stage_angle_runtime "$angle_runtime_dir"
  write_crosvm_angle_wrapper
fi

if [[ "$OS_NAME" == "Darwin" ]]; then
  if [[ -f "$DIST_BIN/crosvm" ]]; then
    CROSVM_RUNTIME_RPATH="@loader_path/../lib"
    if ! otool -l "$DIST_BIN/crosvm" |
      grep -Fq "path $CROSVM_RUNTIME_RPATH "; then
      if ! command -v install_name_tool >/dev/null 2>&1; then
        echo "Error: install_name_tool not found" >&2
        exit 1
      fi
      echo "[rpath] adding $CROSVM_RUNTIME_RPATH to crosvm"
      install_name_tool -add_rpath "$CROSVM_RUNTIME_RPATH" "$DIST_BIN/crosvm"
    fi
  fi
  if ! command -v codesign >/dev/null 2>&1; then
    echo "Error: codesign not found"
    exit 1
  fi
  if [[ ! -f "$MACOS_CROSVM_ENTITLEMENTS" ]]; then
    echo "Error: missing $MACOS_CROSVM_ENTITLEMENTS"
    exit 1
  fi
  echo "[codesign] ad-hoc signing dist libraries"
  find "$DIST_LIB" -maxdepth 1 -type f | while read -r artifact; do
    codesign --force --sign - --timestamp=none "$artifact"
  done

  echo "[codesign] ad-hoc signing dist binaries"
  find "$DIST_BIN" -maxdepth 1 -type f ! -name crosvm | while read -r artifact; do
    codesign --force --sign - --timestamp=none "$artifact"
  done

  if [[ -f "$DIST_BIN/crosvm" ]]; then
    echo "[codesign] ad-hoc signing crosvm with Hypervisor entitlement"
    codesign --force --sign - --entitlements "$MACOS_CROSVM_ENTITLEMENTS" \
      --options runtime --timestamp=none \
      "$DIST_BIN/crosvm"
  else
    echo "Error: missing $DIST_BIN/crosvm"
    exit 1
  fi
fi

if [[ -n "$HOST_APEX_TREE_SOURCE" ]]; then
  if [[ ! -x "$PREPARE_APEX_TREE_SCRIPT" ]]; then
    echo "Error: missing executable $PREPARE_APEX_TREE_SCRIPT"
    exit 1
  fi
  echo "[apex] staging host apex tree from $HOST_APEX_TREE_SOURCE"
  PREPARE_ARGS=(
    --source-root "$HOST_APEX_TREE_SOURCE"
    --target-root "$DIST_APEX_TREE"
    --force
  )
  if [[ "$OS_NAME" == "Darwin" ]]; then
    PREPARE_ARGS+=(--expect-kernel-arch arm64)
  fi
  "$PREPARE_APEX_TREE_SCRIPT" "${PREPARE_ARGS[@]}"
elif [[ -d "$DIST_APEX_TREE" && -x "$PREPARE_APEX_TREE_SCRIPT" ]]; then
  echo "[apex] refreshing host apex metadata under $DIST_APEX_TREE"
  "$PREPARE_APEX_TREE_SCRIPT" --source-root "$DIST_APEX_TREE" --target-root "$DIST_APEX_TREE"
  if [[ "$OS_NAME" == "Darwin" ]]; then
    GUEST_KERNEL_INFO="$(file -b "$DIST_APEX_TREE/apex/com.android.virt/etc/fs/microdroid_kernel" 2>/dev/null || true)"
    if [[ -n "$GUEST_KERNEL_INFO" ]] && ! grep -Eiq 'ARM aarch64|ARM64|arm64' <<<"$GUEST_KERNEL_INFO"; then
      echo "[apex] WARNING: macOS HVF requires an arm64 Microdroid guest kernel, but the staged tree contains:"
      echo "[apex]   $GUEST_KERNEL_INFO"
      echo "[apex]"
      echo "[apex]   To fix, run:"
      echo "[apex]     scripts/fetch_arm64_guest_artifacts.sh --apex-tree $DIST_APEX_TREE --help"
      echo "[apex]   Or set MACOS_AVF_APEX_TREE_SOURCE to an arm64 apex tree."
      echo "[apex]   Or run: scripts/fetch_arm64_guest_artifacts.sh --skip-download (shows manual steps)"
    fi
  fi
fi

if [[ "$OS_NAME" == "Darwin" && -d "$DIST_APEX_TREE" ]]; then
  FETCH_SCRIPT="$REPO_ROOT/scripts/fetch_arm64_guest_artifacts.sh"
  if [[ -x "$FETCH_SCRIPT" ]]; then
    "$FETCH_SCRIPT" --apex-tree "$DIST_APEX_TREE" --skip-download 2>/dev/null || true
  fi
fi

# --- Artifact verification ---
echo
echo "[verify] Checking build artifacts..."
VERIFY_FAIL=0
verify_file() {
  local path="$1" label="$2"
  if [[ -f "$path" ]]; then
    if [[ -s "$path" ]]; then
      echo "  [OK]   $label: $path ($(stat -f%z "$path" 2>/dev/null || stat -c%s "$path" 2>/dev/null || echo "?") bytes)"
    else
      echo "  [FAIL] $label: $path exists but is empty" >&2
      VERIFY_FAIL=1
    fi
  else
    echo "  [FAIL] $label: $path not found" >&2
    VERIFY_FAIL=1
  fi
}

verify_file "$DIST_BIN/virtmgr" "virtmgr binary"
verify_file "$DIST_BIN/vm" "vm binary"
verify_file "$DIST_BIN/crosvm" "crosvm binary"
binder_lib_match="$(first_glob_match "$DIST_LIB/libbinder-rpc*.$LIB_EXT" || true)"
verify_file "$binder_lib_match" "binder-rpc library"

for bin in virtmgr vm crosvm; do
  bin_path="$DIST_BIN/$bin"
  if [[ -f "$bin_path" ]]; then
    file_type="$(file -b "$bin_path" 2>/dev/null || true)"
    if ! grep -Eiq 'Mach-O|ELF' <<<"$file_type"; then
      echo "  [WARN] $bin: unexpected file type: $file_type" >&2
    fi
  fi
done

if [[ $VERIFY_FAIL -eq 1 ]]; then
  echo "  [FAIL] One or more artifacts are missing or empty." >&2
  if [[ "$CLEAN_BUILD" -eq 1 ]]; then
    echo "  Hint: A --clean rebuild was performed. Check build output for errors." >&2
  fi
  exit 1
else
  echo "  [PASS] All artifacts present"
fi

README_FILE="$DIST/README.txt"
echo "build_all: OK" > "$README_FILE"
echo "RUST_TARGET=$RUST_TARGET" >> "$README_FILE"
echo "BINDER_LIB=$BINDER_LIB" >> "$README_FILE"
echo >> "$README_FILE"
echo "bin: virtmgr vm crosvm (as built)" >> "$README_FILE"
echo "lib: $BINDER_LIB" >> "$README_FILE"
if [[ -d "$DIST_APEX_TREE" ]]; then
  echo "apex: $DIST_APEX_TREE" >> "$README_FILE"
fi

echo
echo "Build completed successfully."
echo "Artifacts: $DIST"
