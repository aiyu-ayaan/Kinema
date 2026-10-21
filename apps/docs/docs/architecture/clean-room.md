---
id: clean-room
title: Clean-Room Engineering
sidebar_position: 4
---

# Clean-Room Engineering & Compliance

Kinema is developed in strict accordance with **Clean-Room Software Engineering** standards to ensure complete copyright compliance, maintainable architectural independence, and freedom from proprietary lock-in.

---

## Clean-Room Principles

1. **No Proprietary Code or Binaries**:
   - Kinema contains no reverse-compiled, decompiled, or disassembled code from Plex Media Server, Jellyfin, Emby, or any proprietary streaming products.
   - Proprietary assets, logos, and closed client packages are strictly forbidden in the codebase.
2. **Open Standards Alignment**:
   - Media parsing adheres exclusively to publicly documented ISO and RFC standards:
     - ISO/IEC 14496-12 (ISO base media file format / MP4 / fMP4)
     - Matroska Specification (MKV/MKA)
     - RFC 8216 (HTTP Live Streaming / HLS)
     - W3C WebVTT (Web Video Text Tracks)
3. **Open Metadata APIs**:
   - Metadata is retrieved from public community-governed endpoints:
     - [The Movie Database (TMDB)](https://www.themoviedb.org) v3/v4 REST APIs
     - [TVmaze](https://www.tvmaze.com) CC BY-SA 4.0 endpoints for keyless fallback
4. **Isolated Scratch Workspace**:
   - The `research/` directory is isolated from all build pipelines and package outputs. It is permanently ignored in `.gitignore` and never committed or compiled into release binaries.

---

## Clean-Room vs. Conventional Media Servers

```mermaid
graph TD
    subgraph Proprietary["Closed-Source Solutions (e.g. Plex)"]
        P1["Proprietary Transcode Binaries"]
        P2["Cloud Account Mandate\n(Authentication depends on vendor servers)"]
        P3["Telemetry & Ad Insertion Engines"]
    end

    subgraph KinemaClean["Kinema Clean-Room Architecture"]
        K1["Standard Open-Source ffmpeg / ffprobe\n(Separate OS child processes)"]
        K2["100% Self-Hosted Local Auth\n(Argon2id + Local SQLite)"]
        K3["Zero Cloud Telemetry\n(Zero tracking; your data stays on your hardware)"]
    end
```

---

## Licensing Architecture

- **Kinema Backend & Clients**: Released under the permissive **MIT License**.
- **FFmpeg & Media Binaries**:
  - Kinema does **not** link statically or dynamically against `libavcodec`, `libavformat`, or `libavutil`.
  - Instead, Kinema interacts with `ffmpeg` and `ffprobe` exclusively as external child processes via OS pipes (`stdin`, `stdout`, `stderr`).
  - This architecture cleanly adheres to GNU General Public License (GPL) requirements while preserving the permissive MIT license of the core Kinema application.
