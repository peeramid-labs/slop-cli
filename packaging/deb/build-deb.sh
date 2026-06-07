#!/usr/bin/env bash
# Build a slop_<version>_<arch>.deb from a release binary.
# Usage: ./build-deb.sh <version> <arch> <binary_path>
#   e.g. ./build-deb.sh 0.1.0 amd64 ./target/release/slop

set -euo pipefail

VERSION="${1:?Usage: $0 <version> <arch> <binary_path>}"
ARCH="${2:?Usage: $0 <version> <arch> <binary_path>}"
BINARY="${3:?Usage: $0 <version> <arch> <binary_path>}"

PKG_NAME="slop"
PKG_DIR="${PKG_NAME}_${VERSION}_${ARCH}"

rm -rf "$PKG_DIR"
mkdir -p "${PKG_DIR}/DEBIAN"
mkdir -p "${PKG_DIR}/usr/bin"
mkdir -p "${PKG_DIR}/usr/share/doc/${PKG_NAME}"

install -m 755 "$BINARY" "${PKG_DIR}/usr/bin/slop"

cat > "${PKG_DIR}/DEBIAN/control" <<EOF
Package: ${PKG_NAME}
Version: ${VERSION}
Section: devel
Priority: optional
Architecture: ${ARCH}
Maintainer: Peeramid Labs <engineering@peeramid.xyz>
Homepage: https://github.com/peeramid-labs/slop-cli
Description: Blazing-fast AI-slop firewall CLI
 slop scans a unified git diff for AI-pattern scaffolding, naming
 slop, defensive crud, TODO placeholders, emoji-in-code,
 restating-code comments, and unused generics. Detection runs
 server-side; the binary is a thin sender + applier that strips
 flagged lines and amends HEAD automatically.
EOF

cat > "${PKG_DIR}/usr/share/doc/${PKG_NAME}/copyright" <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: slop
Source: https://github.com/peeramid-labs/slop-cli

Files: *
Copyright: 2026 Peeramid Labs
License: MIT
EOF

dpkg-deb --build "$PKG_DIR" >/dev/null
echo "${PKG_DIR}.deb"
