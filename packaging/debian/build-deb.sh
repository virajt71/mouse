#!/usr/bin/env bash
set -e

# Build script for Debian/Ubuntu .deb package
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

PKG_NAME="mouser-rs"
VERSION="0.1.0"
ARCH="amd64"
DEB_DIR_NAME="${PKG_NAME}_${VERSION}_${ARCH}"
BUILD_ROOT="$ROOT_DIR/target/deb/$DEB_DIR_NAME"

echo "=== Building Release Binary ==="
cd "$ROOT_DIR"
cargo build --release

echo "=== Staging Debian Package Structure ==="
rm -rf "$BUILD_ROOT"
mkdir -p "$BUILD_ROOT/DEBIAN"
mkdir -p "$BUILD_ROOT/usr/bin"
mkdir -p "$BUILD_ROOT/lib/udev/rules.d"
mkdir -p "$BUILD_ROOT/usr/share/applications"
mkdir -p "$BUILD_ROOT/usr/lib/systemd/user"

# Copy binary
cp "$ROOT_DIR/target/release/mouser-rs" "$BUILD_ROOT/usr/bin/mouser-rs"
chmod 755 "$BUILD_ROOT/usr/bin/mouser-rs"

# Copy common metadata files
cp "$ROOT_DIR/packaging/common/69-mouser-logitech.rules" "$BUILD_ROOT/lib/udev/rules.d/69-mouser-logitech.rules"
chmod 644 "$BUILD_ROOT/lib/udev/rules.d/69-mouser-logitech.rules"

cp "$ROOT_DIR/packaging/common/io.github.mouser.desktop" "$BUILD_ROOT/usr/share/applications/io.github.mouser.desktop"
chmod 644 "$BUILD_ROOT/usr/share/applications/io.github.mouser.desktop"

cp "$ROOT_DIR/packaging/common/mouser.service" "$BUILD_ROOT/usr/lib/systemd/user/mouser.service"
chmod 644 "$BUILD_ROOT/usr/lib/systemd/user/mouser.service"

# Generate DEBIAN/control
cat <<EOF > "$BUILD_ROOT/DEBIAN/control"
Package: $PKG_NAME
Version: $VERSION
Architecture: $ARCH
Maintainer: Mouser Team <dev@mouser.local>
Section: utils
Priority: optional
Depends: libc6 (>= 2.31), libhidapi-hidraw0, libudev1, libgtk-3-0
Description: Native Linux daemon and GUI for Logitech HID++ mice and keyboards.
 Includes button remapping, gesture control, DPI tuning, SmartShift, and Logitech Flow.

EOF

# Generate DEBIAN/postinst
cat <<EOF > "$BUILD_ROOT/DEBIAN/postinst"
#!/bin/sh
set -e
if command -v udevadm >/dev/null 2>&1; then
    udevadm control --reload-rules || true
    udevadm trigger || true
fi
exit 0
EOF
chmod 755 "$BUILD_ROOT/DEBIAN/postinst"

echo "=== Building .deb Package ==="
dpkg-deb -z8 -Zxz --build "$BUILD_ROOT" "$ROOT_DIR/${DEB_DIR_NAME}.deb"

echo "=== Package successfully built: ${ROOT_DIR}/${DEB_DIR_NAME}.deb ==="
