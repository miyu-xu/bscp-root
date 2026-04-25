#!/usr/bin/env bash
# Single host build script: binder-rpc (CMake) -> copy libs for Rust -> cargo virtmgr+vm+crosvm -> dist/<platform>
# Usage: from repo root:  chmod +x build_all.sh && ./build_all.sh
# Override triple: RUST_TARGET=aarch64-unknown-linux-gnu ./build_all.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$REPO_ROOT"
OUT_ROOT="$REPO_ROOT/out"
CMAKE_BUILD_DIR="$OUT_ROOT/build"

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
        DEFAULT_CROSVM_FEATURES="hvf,default-no-sandbox,config-file,qcow,balloon,android-sparse,composite-disk,tokio"
        CROSVM_CARGO_TOOLCHAIN="${CROSVM_CARGO_TOOLCHAIN:-nightly}"
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
        DEFAULT_CROSVM_FEATURES="default-no-sandbox,config-file,qcow,balloon,android-sparse,composite-disk"
        CROSVM_CARGO_TOOLCHAIN="${CROSVM_CARGO_TOOLCHAIN:-stable}"
        ;;
    *)
        echo "Error: unsupported OS from uname: $OS_NAME (use Linux/macOS or build_all.bat on Windows)"
        exit 1
        ;;
esac

DIST="$OUT_ROOT/dist/$DIST_DIR_NAME"
DIST_BIN="$DIST/bin"
DIST_LIB="$DIST/lib"
BINDER_LIB="libbinder-rpc.$LIB_EXT"
MACOS_CROSVM_ENTITLEMENTS="$REPO_ROOT/scripts/macos_crosvm.entitlements"
PREPARE_APEX_TREE_SCRIPT="$REPO_ROOT/scripts/prepare_host_apex_tree.sh"
DIST_APEX_TREE="$OUT_ROOT/dist/apex_dir"
HOST_APEX_TREE_SOURCE="${HOST_APEX_TREE_SOURCE:-}"

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
echo "[1/4] binder-rpc (CMake)"
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
echo "[2/4] Copy libbinder-rpc into binder rust/sys/libs"
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
echo "[3/4] Rust (virtmgr + vm + crosvm)"
export CARGO_TARGET_DIR="$OUT_ROOT/target"
CROSVM_FEATURES="${CROSVM_FEATURES:-$DEFAULT_CROSVM_FEATURES}"
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
echo "[4/4] Collect artifacts into dist/$DIST_DIR_NAME"
mkdir -p "$DIST_BIN" "$DIST_LIB"
TGT_OUT="$CARGO_TARGET_DIR/$RUST_TARGET/release"

cp -f "$CMAKE_BUILD_DIR/bin/"* "$DIST_BIN/" 2>/dev/null || true
if compgen -G "$BLIB/libbinder-rpc*.$LIB_EXT" >/dev/null; then
  cp -af "$BLIB"/libbinder-rpc*."$LIB_EXT" "$DIST_LIB/"
else
  echo "Error: missing $BINDER_LIB under $BLIB"
  exit 1
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

if [[ "$OS_NAME" == "Darwin" ]]; then
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
    codesign --force --sign - --entitlements "$MACOS_CROSVM_ENTITLEMENTS" --timestamp=none \
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
      echo "[apex] warning: macOS HVF requires an arm64 Microdroid guest kernel, but the staged tree contains:"
      echo "[apex]   $GUEST_KERNEL_INFO"
      echo "[apex]   Set MACOS_AVF_APEX_TREE_SOURCE to an arm64 apex tree before running macOS guest commands."
    fi
  fi
fi

{
  echo "build_all: OK"
  echo "RUST_TARGET=$RUST_TARGET"
  echo "BINDER_LIB=$BINDER_LIB"
  echo
  echo "bin: virtmgr vm crosvm (as built)"
  echo "lib: $BINDER_LIB"
  if [[ -d "$DIST_APEX_TREE" ]]; then
    echo "apex: $DIST_APEX_TREE"
  fi
} > "$DIST/README.txt"

echo
echo "Build completed successfully."
echo "Artifacts: $DIST"
