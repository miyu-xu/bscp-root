#!/usr/bin/env bash
# Single host build script: binder-rpc (CMake) -> copy libs for Rust -> cargo virtmgr+vm+crosvm -> dist/<platform>
# Usage: from repo root:  chmod +x build_all.sh && ./build_all.sh
# Override triple: RUST_TARGET=aarch64-unknown-linux-gnu ./build_all.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$REPO_ROOT"
OUT_ROOT="$REPO_ROOT/out"
CMAKE_BUILD_DIR="$OUT_ROOT/build"

# Host OS defaults (Darwin / Linux). Windows developers use build_all.bat.
OS_NAME="$(uname -s)"
case "$OS_NAME" in
    Darwin)
        RUST_TARGET="${RUST_TARGET:-aarch64-apple-darwin}"
        LIB_EXT="dylib"
        DIST_DIR_NAME="macos"
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

echo
echo "[1/4] binder-rpc (CMake)"
if ! command -v cmake >/dev/null; then
  echo "Error: cmake not found"
  exit 1
fi
CMAKE_GEN="${CMAKE_GENERATOR:-Ninja}"
mkdir -p "$CMAKE_BUILD_DIR"
if ! cmake -B "$CMAKE_BUILD_DIR" -G "$CMAKE_GEN" -DCMAKE_BUILD_TYPE=Release .; then
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
CROSVM_FEATURES="${CROSVM_FEATURES:-default-no-sandbox,config-file,qcow,balloon,android-sparse,composite-disk}"
echo "[cargo] virtmgr --release --target $RUST_TARGET"
cargo build --manifest-path "$REPO_ROOT/packages/modules/Virtualization/android/virtmgr/Cargo.toml" --release --target "$RUST_TARGET"
echo "[cargo] vm --release --target $RUST_TARGET"
cargo build --manifest-path "$REPO_ROOT/packages/modules/Virtualization/android/vm/Cargo.toml" --release --target "$RUST_TARGET"
echo "[cargo] crosvm +stable --release -p crosvm --target $RUST_TARGET --no-default-features --features $CROSVM_FEATURES"
(
  cd "$REPO_ROOT/external/crosvm"
  cargo +stable build --release -p crosvm --target "$RUST_TARGET" \
    --no-default-features --features "$CROSVM_FEATURES"
)

echo
echo "[4/4] Collect artifacts into dist/$DIST_DIR_NAME"
mkdir -p "$DIST_BIN" "$DIST_LIB"
TGT_OUT="$CARGO_TARGET_DIR/$RUST_TARGET/release"

cp -f "$CMAKE_BUILD_DIR/bin/"* "$DIST_BIN/" 2>/dev/null || true
cp -a "$BLIB"/libbinder-rpc.so* "$DIST_LIB/"
if [[ -f "$TGT_OUT/virtmgr" ]]; then
  cp -f "$TGT_OUT/virtmgr" "$DIST_BIN/"
fi
if [[ -f "$TGT_OUT/vm" ]]; then
  cp -f "$TGT_OUT/vm" "$DIST_BIN/"
fi
if [[ -f "$TGT_OUT/crosvm" ]]; then
  cp -f "$TGT_OUT/crosvm" "$DIST_BIN/"
fi

{
  echo "build_all: OK"
  echo "RUST_TARGET=$RUST_TARGET"
  echo "BINDER_LIB=$BINDER_LIB"
  echo
  echo "bin: virtmgr vm crosvm (as built)"
  echo "lib: $BINDER_LIB"
} > "$DIST/README.txt"

echo
echo "Build completed successfully."
echo "Artifacts: $DIST"
