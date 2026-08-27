#!/usr/bin/env bash
set -e

# Mouser-RS Universal System / User Installer
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

PREFIX="${PREFIX:-/usr/local}"
BIN_DIR="$PREFIX/bin"
UDEV_DIR="/etc/udev/rules.d"
DESKTOP_DIR="$PREFIX/share/applications"
ICON_DIR="$PREFIX/share/icons/hicolor/scalable/apps"

echo "=== Building Mouser-RS Release Binary ==="
cd "$ROOT_DIR"
cargo build --release

echo "=== Installing Mouser-RS to $PREFIX ==="
sudo mkdir -p "$BIN_DIR" "$UDEV_DIR" "$DESKTOP_DIR" "$ICON_DIR"

sudo install -m 755 "$ROOT_DIR/target/release/mouser-rs" "$BIN_DIR/mouser-rs"
sudo install -m 644 "$ROOT_DIR/packaging/common/69-mouser-logitech.rules" "$UDEV_DIR/69-mouser-logitech.rules"
sudo install -m 644 "$ROOT_DIR/packaging/common/io.github.mouser.desktop" "$DESKTOP_DIR/io.github.mouser.desktop"
if [ -f "$ROOT_DIR/assets/images/mouse.png" ]; then
    sudo install -m 644 "$ROOT_DIR/assets/images/mouse.png" "$PREFIX/share/icons/hicolor/256x256/apps/mouser.png" 2>/dev/null || true
fi

echo "=== Reloading udev rules ==="
sudo udevadm control --reload-rules 2>/dev/null || true
sudo udevadm trigger 2>/dev/null || true

echo "=== Success! Mouser-RS has been installed to $BIN_DIR/mouser-rs ==="
echo "Run 'mouser-rs status' to verify connection."
