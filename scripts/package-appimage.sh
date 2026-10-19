#!/usr/bin/env bash
set -euo pipefail

# Package Kinema Linux AppImage and tarball
echo "Packaging Kinema Linux AppImage..."
ARCH="${1:-x86_64}"
VERSION="${2:-0.1.0}"
echo "Arch: $ARCH, Version: $VERSION"

# Ensure web dist is built
pnpm run build:web

# Build backend with musl
cargo build --release --target "${ARCH}-unknown-linux-musl" --manifest-path backend/Cargo.toml || {
  echo "Fallback to host release build..."
  cargo build --release --manifest-path backend/Cargo.toml
}

echo "AppImage packaging workflow prepared."
