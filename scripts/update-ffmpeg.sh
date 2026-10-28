#!/usr/bin/env bash
# Bumps the ffmpeg release line in scripts/ffmpeg.lock, after checking BtbN publishes builds for it.
#   scripts/update-ffmpeg.sh 8.2
set -euo pipefail
cd "$(dirname "$0")/.."

version="${1:?usage: scripts/update-ffmpeg.sh <version, e.g. 8.2>}"
source="$(sed -n 's/^source=//p' scripts/ffmpeg.lock)"
sums="$(curl -fsSL --retry 3 "$source/checksums.sha256")"
for platform in linux64 linuxarm64 win64; do
  grep -q " ffmpeg-n$version-latest-$platform-gpl-$version\." <<<"$sums" \
    || { echo "no $platform GPL build for $version at $source" >&2; exit 1; }
done
sed -i "s/^version=.*/version=$version/" scripts/ffmpeg.lock
echo "scripts/ffmpeg.lock -> ffmpeg $version"
