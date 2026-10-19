#!/usr/bin/env bash
# The server for a home server, in one folder like Plex: the static backend, the web UI and ffmpeg.
#   scripts/package-server.sh [x86_64|aarch64]   -> dist/kinema-server-<version>-linux-<arch>.tar.gz
# Builds through the Dockerfile (needs Docker), so the image and this tarball share one recipe.
# Another architecture than the machine's own goes through QEMU and is slow; CI builds each natively.
# scripts/install-server.sh installs the result.
set -euo pipefail
cd "$(dirname "$0")/.."

ARCH="${1:-$(uname -m)}"
case "$ARCH" in
  x86_64 | amd64) ARCH=x86_64 PLATFORM=linux/amd64 FFMPEG_ARCH=linux64 ;;
  aarch64 | arm64) ARCH=aarch64 PLATFORM=linux/arm64 FFMPEG_ARCH=linuxarm64 ;;
  *) echo "unsupported architecture: $ARCH" >&2; exit 1 ;;
esac
VERSION="$(node -p "require('./package.json').version")"
lock() { sed -n "s/^$1=//p" scripts/ffmpeg.lock; }
FFMPEG_VERSION="$(lock version)"
NAME="kinema-server-$VERSION-linux-$ARCH"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

docker buildx build --platform "$PLATFORM" --target bundle --output "type=local,dest=$WORK/$NAME" .

archive="ffmpeg-n$FFMPEG_VERSION-latest-$FFMPEG_ARCH-gpl-$FFMPEG_VERSION.tar.xz"
curl -fsSL --retry 3 -o "$WORK/$archive" "$(lock source)/$archive"
curl -fsSL --retry 3 -o "$WORK/checksums.sha256" "$(lock source)/checksums.sha256"
(cd "$WORK" && grep " $archive\$" checksums.sha256 | sha256sum -c -)
tar -xJf "$WORK/$archive" -C "$WORK"
ff="$WORK/${archive%.tar.xz}"
cp "$ff/bin/ffmpeg" "$ff/bin/ffprobe" "$WORK/$NAME/"
mkdir -p "$WORK/$NAME/licenses"
cp LICENSE "$WORK/$NAME/licenses/kinema.txt"
cp "$ff/LICENSE.txt" "$WORK/$NAME/licenses/ffmpeg.txt"
echo "ffmpeg $FFMPEG_VERSION, $archive from $(lock source)" > "$WORK/$NAME/licenses/ffmpeg-source.txt"
chmod 755 "$WORK/$NAME/kinema"

mkdir -p dist
tar -czf "dist/$NAME.tar.gz" -C "$WORK" "$NAME"
echo "dist/$NAME.tar.gz"
