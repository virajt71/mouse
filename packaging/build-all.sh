#!/usr/bin/env bash
set -e

# Master multi-distro package builder for Mouser-RS
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

DIST_DIR="$ROOT_DIR/dist"
mkdir -p "$DIST_DIR"

echo "=========================================================="
echo "          Mouser-RS Multi-Distro Package Builder          "
echo "=========================================================="

echo "=== Step 1: Building Release Binary ==="
cd "$ROOT_DIR"
cargo build --release

echo "=== Step 2: Building Debian (.deb) Package ==="
"$ROOT_DIR/packaging/debian/build-deb.sh"
cp "$ROOT_DIR"/mouser-rs_*.deb "$DIST_DIR/" 2>/dev/null || true

echo "=== Step 3: Building Universal Tarball (.tar.gz) Package ==="
"$ROOT_DIR/packaging/universal/build-tarball.sh"
cp "$ROOT_DIR"/mouser-rs-v*.tar.gz "$DIST_DIR/" 2>/dev/null || true

echo "=== Step 4: Checking for rpmbuild (Fedora/RPM Package) ==="
if command -v rpmbuild >/dev/null 2>&1; then
    echo "Building RPM package via rpmbuild..."
    RPM_BUILD_DIR="$ROOT_DIR/target/rpm"
    mkdir -p "$RPM_BUILD_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
    cp "$ROOT_DIR/packaging/fedora/mouser-rs.spec" "$RPM_BUILD_DIR/SPECS/"
    # Create source tarball for rpmbuild
    tar -czf "$RPM_BUILD_DIR/SOURCES/mouser-rs-0.1.0.tar.gz" --transform 's,^\.,mouser-rs-0.1.0,' --exclude='./target' --exclude='./dist' .
    rpmbuild --define "_topdir $RPM_BUILD_DIR" -ba "$RPM_BUILD_DIR/SPECS/mouser-rs.spec" || true    
    cp "$RPM_BUILD_DIR"/RPMS/*/*.rpm "$DIST_DIR/" 2>/dev/null || true
else
    echo "[Notice] rpmbuild not installed. Skipping direct RPM build. (Spec available at packaging/fedora/mouser-rs.spec)"
fi

echo "=== Step 5: Checking for Arch Linux makepkg ==="
if command -v makepkg >/dev/null 2>&1; then
    echo "PKGBUILD is ready at packaging/arch/PKGBUILD."
else
    echo "[Notice] makepkg not installed on host. (PKGBUILD recipe available at packaging/arch/PKGBUILD)"
fi

echo ""
echo "=========================================================="
echo " Build Summary: Artifacts compiled to $DIST_DIR:"
ls -lh "$DIST_DIR"
echo "=========================================================="
