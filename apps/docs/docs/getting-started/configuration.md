---
id: configuration
title: Configuration & Variables
sidebar_position: 3
---

# Configuration & Environment Variables

Kinema can be configured through a configuration file (`kinema.toml`), system environment variables, or command-line flags. Environment variables take precedence over config files.

---

## Environment Variables Reference

| Variable | Default | Description |
|---|---|---|
| `KINEMA_DATA_DIR` | `data` / `$XDG_DATA_HOME/kinema` | Directory where SQLite database (`kinema.db`), caches, and TLS keys are stored. |
| `KINEMA_HOST` | `0.0.0.0` | IP interface to bind Axum HTTP/HTTPS listener. |
| `KINEMA_PORT` | `3000` | Port to bind server. Tries up to 20 consecutive ports if occupied. |
| `KINEMA_PUBLIC_URL` | Auto-detected | Fully qualified external URL sent to clients for remote access. |
| `KINEMA_FFMPEG` | Resolved from PATH | Explicit override for the `ffmpeg` executable. |
| `KINEMA_FFPROBE` | Resolved from PATH | Explicit override for the `ffprobe` executable. |
| `KINEMA_MPV` | Resolved from PATH | Path to `mpv` binary for Desktop client playback. |
| `KINEMA_WEB_DIST` | Resolved from build | Path to static React web assets folder (`apps/web/dist`). |
| `KINEMA_TMDB_API_KEY` | None | Initial TMDB v3 API Key for metadata indexing. |
| `KINEMA_MAX_TRANSCODES`| `2` | Maximum concurrent video re-encodes allowed before returning 503. |
| `KINEMA_FLAVOR` | `prod` | Runtime flavor: `prod` (uses `kinema.token`) or `dev` (uses `kinema.dev.token`). |

---

## Configuration File (`kinema.toml`)

You can place a `kinema.toml` file in your working directory or inside `$KINEMA_DATA_DIR`:

```toml
[server]
host = "0.0.0.0"
port = 3000
public_url = "https://media.lan:3000"
max_transcodes = 2

[storage]
data_dir = "/var/lib/kinema"
cache_images = "/var/cache/kinema/images"
cache_thumbs = "/var/cache/kinema/thumbs"
cache_subtitles = "/var/cache/kinema/subtitles"

[transcoding]
# Video quality and speed
crf = 22
preset = "fast"
max_height = 1080
prefer_hardware_accel = true

[metadata]
tmdb_key = "your_tmdb_api_key_here"
default_language = "en-US"
scan_interval_minutes = 60

[network]
discovery_enabled = true
discovery_port = 48555
```

---

## Transcoding Profiles & Settings

Administrators can customize encoding profiles on the fly via the Settings dashboard or the REST API:

- **CRF (Constant Rate Factor)**: Controls visual quality for H.264 video transcode.
  - Range: `18` (high quality, large bandwidth) to `28` (low bandwidth, small devices). Default: `22`.
- **CPU Preset**:
  - `ultrafast`, `superfast`, `veryfast`, `faster`, `fast` (default), `medium`.
  - On low-power systems like Raspberry Pi or older Intel Celerons, `veryfast` or `faster` is recommended.
- **Max Stream Height**:
  - Restricts maximum transcoded resolution (`720p`, `1080p`, `4K`). Files with higher resolutions are scaled down with lanczos filtering to fit the cap.
- **Subtitle Languages**:
  - Sets preferred ISO-639-1 subtitle tracks to select automatically when playback starts.
