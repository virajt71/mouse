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
USERUNIT_DIR="/usr/lib/systemd/user"

echo "=== Building Mouser-RS Release Binary ==="
cd "$ROOT_DIR"
cargo build --release

echo "=== Installing Mouser-RS to $PREFIX ==="
sudo mkdir -p "$BIN_DIR" "$UDEV_DIR" "$DESKTOP_DIR" "$ICON_DIR"

sudo install -m 755 "$ROOT_DIR/target/release/mouser-rs" "$BIN_DIR/mouser-rs"
sudo install -m 644 "$ROOT_DIR/packaging/common/69-mouser-logitech.rules" "$UDEV_DIR/69-mouser-logitech.rules"
sudo install -m 644 "$ROOT_DIR/packaging/common/io.github.mouser.desktop" "$DESKTOP_DIR/io.github.mouser.desktop"
if [ -f "$ROOT_DIR/assets/icons/mouser.svg" ]; then
    sudo install -m 644 "$ROOT_DIR/assets/icons/mouser.svg" "$PREFIX/share/icons/hicolor/scalable/apps/mouser.svg" 2>/dev/null || true
fi

echo "=== Reloading udev rules ==="
sudo udevadm control --reload-rules 2>/dev/null || true
sudo udevadm trigger 2>/dev/null || true

echo "=== Enabling user service (starts at login) ==="
sudo mkdir -p "$USERUNIT_DIR"
sudo install -m 644 "$ROOT_DIR/packaging/common/mouser.service" "$USERUNIT_DIR/mouser.service"
if command -v systemctl >/dev/null 2>&1; then
    systemctl --global daemon-reload 2>/dev/null || true
    systemctl --global enable mouser.service 2>/dev/null || true
    # Start now for the installing user (and any active sessions)
    systemctl --user start mouser.service 2>/dev/null || true
    if command -v loginctl >/dev/null 2>&1; then
        for s in $(loginctl list-sessions --no-legend 2>/dev/null | awk '{print $1}'); do
            u=$(loginctl show-session "$s" -p Name --value 2>/dev/null)
            [ -n "$u" ] && sudo runuser -u "$u" -- systemctl --user start mouser.service 2>/dev/null || true
        done
    fi
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    sudo gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" 2>/dev/null || true
fi

echo "=== Success! Mouser-RS has been installed to $BIN_DIR/mouser-rs ==="
echo "Run 'mouser-rs status' to verify connection."
