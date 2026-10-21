---
id: packaging-matrix
title: Packaging & Binary Releases
sidebar_position: 2
---

# Packaging & Binary Releases

Kinema produces static, self-contained binary artifacts for all major desktop and server operating systems through automated GitHub Actions workflows.

---

## Release Artifacts Matrix

```
Linux (x86_64 and aarch64):
├── Kinema-<ver>-<arch>.AppImage      Bundles static musl server, web UI, ffmpeg & ffprobe
└── kinema-<ver>-<arch>-linux.tar.gz  Standalone tarball for servers without FUSE

Windows (x86_64):
├── Kinema-<ver>-setup.exe            Inno Setup installer with optional autostart service
└── Kinema-<ver>-windows-x64.zip      Portable zip archive

Docker (linux/amd64 and linux/arm64):
└── ghcr.io/aiyu-ayaan/kinema:<ver>   Official multi-arch container image
```

---

## Building Native Binaries from Source

### Static Linux Binary (Musl + Mimalloc)

```bash
# Install musl target
rustup target add x86_64-unknown-linux-musl

# Build web frontend distribution first
pnpm build:web

# Build backend binary with release optimizations
cargo build --release --target x86_64-unknown-linux-musl --manifest-path backend/Cargo.toml
```

The resulting binary `target/x86_64-unknown-linux-musl/release/kinema` has zero dynamic C library dependencies and runs on any Linux distribution (Ubuntu, Debian, Alpine, Fedora, Arch, NixOS).

---

## Automated Packaging Scripts

The repository includes packaging scripts in `scripts/`:

- `scripts/package-appimage.sh`: Builds Linux AppImage bundles with embedded ffmpeg binaries.
- `scripts/package-windows.ps1`: Builds Inno Setup installer and portable zip archives for Windows.
- `scripts/package-docker.sh`: Builds and publishes multi-architecture Docker manifests to GitHub Container Registry.
- `scripts/update-ffmpeg.sh`: Bumps and verifies pinned checksums in `scripts/ffmpeg.lock`.
