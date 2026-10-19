#!/usr/bin/env bash
set -euo pipefail

NEW_VERSION="${1:-7.1}"
echo "Updating pinned ffmpeg version to $NEW_VERSION in scripts/ffmpeg.lock..."
sed -i "s/^version=.*/version=${NEW_VERSION}/" scripts/ffmpeg.lock
echo "Updated scripts/ffmpeg.lock"
