#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-latest}"
echo "Building Kinema multi-arch Docker image with tag: $TAG..."
docker buildx build --platform linux/amd64,linux/arm64 -t "ghcr.io/kinema/kinema:${TAG}" .
