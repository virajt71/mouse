#!/usr/bin/env bash
set -e

# Script to build a portable universal .tar.gz bundle for any Linux distro
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

PKG_NAME="mouser-rs"
VERSION="0.1.0"
ARCH="x86_64"
TARBALL_DIR_NAME="${PKG_NAME}-v${VERSION}-${ARCH}-linux-gnu"
BUILD_ROOT="$ROOT_DIR/target/tarball/$TARBALL_DIR_NAME"

echo "=== Building Release Binary ==="
cd "$ROOT_DIR"
cargo build --release

echo "=== Staging Universal Tarball ==="
rm -rf "$BUILD_ROOT"
mkdir -p "$BUILD_ROOT"

cp "$ROOT_DIR/target/release/mouser-rs" "$BUILD_ROOT/"
cp "$ROOT_DIR/packaging/common/69-mouser-logitech.rules" "$BUILD_ROOT/"
cp "$ROOT_DIR/packaging/common/io.github.mouser.desktop" "$BUILD_ROOT/"
cp "$ROOT_DIR/packaging/common/mouser.service" "$BUILD_ROOT/"
cp "$ROOT_DIR/packaging/common/install.sh" "$BUILD_ROOT/"
chmod +x "$BUILD_ROOT/mouser-rs" "$BUILD_ROOT/install.sh"

echo "=== Compressing Tarball Archive ==="
cd "$ROOT_DIR/target/tarball"
tar -czf "$ROOT_DIR/${TARBALL_DIR_NAME}.tar.gz" "$TARBALL_DIR_NAME"

echo "=== Tarball successfully built: ${ROOT_DIR}/${TARBALL_DIR_NAME}.tar.gz ==="
