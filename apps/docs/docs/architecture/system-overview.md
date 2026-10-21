---
id: system-overview
title: System Architecture
sidebar_position: 1
---

# System Architecture & Crates Overview

Kinema is organized as a Cargo workspace with distinct, single-responsibility Rust crates designed to maximize testability, keep dependencies minimal, and ensure strict platform independence.

```mermaid
flowchart TB
    subgraph Clients["Clients Layer"]
        Web["Web SPA\n(React 18 + TS + Tailwind)"]
        Desk["Desktop Client\n(Electron + LAN Discovery)"]
        And["Android App\n(Compose + ExoPlayer)"]
        Mpv["mpv Player\n(Desktop Loopback)"]
    end

    subgraph ServerBinary["backend (Single Binary Axum + Tokio Runtime)"]
        Router["Axum HTTP/HTTPS Router"]
        AuthMid["Auth & Session Middleware"]
        Jobs["Background Job Supervisor"]
        Discovery["UDP Discovery Beacon (48555)"]
        
        Router --> AuthMid
        AuthMid --> Jobs
    end

    subgraph InternalCrates["Internal Crates (crates/)"]
        Core["kinema-core\n• Platform cfg abstractions\n• Config & env parser\n• Process priority (nice/creation_flags)"]
        DB["kinema-db\n• sqlx SQLite pool (WAL mode)\n• Migrations 01-07\n• FTS5 search"]
        Scanner["kinema-scanner\n• WalkDir recursive indexer\n• Notify kernel FS watcher\n• Episode & movie filename parser"]
        Media["kinema-media\n• ffprobe parser\n• Playback decision engine\n• fMP4 streaming builder\n• Audio fingerprinting & markers"]
        Meta["kinema-meta\n• TMDB v3 API client\n• TVmaze fallback parser\n• Image downloader & cache"]
    end

    subgraph StorageLayer["Persistence & Storage ($data_dir)"]
        SqliteDB[("kinema.db (WAL)")]
        DiskCache["cache/ (images, thumbs, vtt, audio prints)"]
        TLSCerts["tls/ (cert.pem + key.pem)"]
    end

    Web -->|HTTPS / Range 206| Router
    Desk -->|HTTPS / Pinned TLS| Router
    And -->|HTTPS / Pinned TLS| Router
    Mpv -->|HTTP Ranged Read| Desk

    Router --> Core & DB & Media & Meta
    Jobs --> Scanner & Media & Meta & DB
    
    DB --> SqliteDB
    Media --> DiskCache
    Router --> TLSCerts
    Discovery -.->|UDP Broadcast| Desk & And
```

---

## Workspace Crates Breakdown

### 1. `backend` (`backend/`)
The primary Rust binary crate.
- **Axum Web Server**: Exposes REST endpoints, validates authentication, handles HTTP range requests, and manages progressive fMP4 chunked streaming.
- **TLS Management**: Automatically generates self-signed TLS certificates on first boot using `rcgen`, computes a SHA-256 fingerprint for client pinning, and binds via `rustls`.
- **Job Supervisor**: Asynchronous scheduler executing folder rescans, missing thumbnail generation, and audio fingerprint theme matching without blocking request workers.
- **Discovery Service**: Listens and responds to `KINEMA_DISCOVER` broadcasts over UDP port `48555`.

### 2. `kinema-core` (`crates/kinema-core/`)
Cross-platform primitives and configuration:
- **Platform Independence**: OS-specific logic lives exclusively behind `cfg(unix)` and `cfg(windows)`.
  - Unix: Configures child process priorities via `libc::nice(10)` (playback) and `libc::nice(19)` (background jobs).
  - Windows: Uses `BELOW_NORMAL_PRIORITY_CLASS` and `IDLE_PRIORITY_CLASS`.
- **Virtual Filesystem Checks**: Detects network drives or virtual filesystems (e.g., FUSE, SMB, NFS) to disable inotify and tune read buffers accordingly.

### 3. `kinema-db` (`crates/kinema-db/`)
Persistent storage and relational queries:
- **Connection Management**: Configures SQLite with `PRAGMA journal_mode = WAL;`, `PRAGMA synchronous = NORMAL;`, and `PRAGMA foreign_keys = ON;`.
- **Append-Only Migrations**: 7 standard SQL migrations managing users, libraries, files, streams, track choices, and markers.
- **FTS5 Full-Text Search**: Instant search over movie, show, and episode titles and summaries.

### 4. `kinema-media` (`crates/kinema-media/`)
Audio, video, and subtitle pipeline:
- **Resolver**: Locates `ffmpeg` and `ffprobe` binaries across system paths, environment overrides, and bundled directories.
- **Decision Engine**: Calculates playback mode (`Direct Play`, `Remux`, `Transcode`) based on container, video codec, audio codec, and client capabilities.
- **Progress Throttling**: Monitors real-time ffmpeg progress via stderr; pauses stdout reads when transcoding exceeds 60 seconds ahead of playback.
- **Audio Fingerprint Theme Detection**: Computes Haitsma & Kalker 33-band spectrogram hashes to identify shared theme music across season episodes.

### 5. `kinema-meta` (`crates/kinema-meta/`)
Metadata scraping and caching:
- **TMDB API Client**: Scrapes posters, backdrops, overviews, ratings, and cast members.
- **TVmaze Fallback**: Zero-credential fallback for TV show hierarchies and artwork when no TMDB key is provided.
- **Artwork Cache**: Hashes images and writes them to disk with immutable HTTP caching headers.

### 6. `kinema-scanner` (`crates/kinema-scanner/`)
Filesystem discovery and monitoring:
- **WalkDir Indexer**: High-speed traversal of media folders, resolving multi-part movies and sidecar subtitles (`.srt`, `.vtt`, `.ass`).
- **Filename Parser**: Heuristic regex parser identifying release years, standard `S01E02` patterns, anime numbering, and special feature tags.
- **Change Watcher**: `notify`-based kernel filesystem watcher that registers new or modified media without periodic disk scanning.

---

## Detailed Request Lifecycle

The diagram below traces an incoming playback session request from a client to the underlying media process:

```mermaid
sequenceDiagram
    autonumber
    participant C as Client App
    participant A as Axum Router / Auth
    participant H as Media Handler (/plan)
    participant DB as SQLite DB
    participant Engine as kinema-media
    participant FF as ffmpeg Child Process

    C->>A: GET /api/files/:id/plan?hevc=true&av1=false&audio=0
    A->>A: Validate Session (Header or Cookie)
    A->>H: Invoke plan handler
    H->>DB: Query file codecs, streams, user track preferences
    DB-->>H: Stream metadata (MKV, H.264, DTS 5.1)
    H->>Engine: Evaluate decision matrix
    Note over Engine: Video is H.264 (direct copy)<br/>Audio is DTS (transcode to AAC)
    Engine-->>H: Mode: Remux + Audio Transcode
    H-->>C: 200 OK { mode: "remux", start: 0, audio: 0 }

    C->>A: GET /api/stream/:id?audio=0&start=0
    A->>H: Invoke stream handler
    H->>Engine: Spawn ffmpeg command (-c:v copy -c:a aac -f mp4 pipe:1)
    Engine->>FF: tokio::process::Command::spawn (nice 10, kill_on_drop)
    FF-->>H: Progressive fMP4 stdout stream
    H->>H: Progress::pace() throttles read if > 60s ahead
    H-->>C: 200 OK (video/mp4 chunked fMP4 stream)
    
    Note over C,FF: When user closes player, connection drops.<br/>Counted wrapper triggers kill_on_drop(true), immediately killing ffmpeg.
```
