---
id: server-admin-api
title: Server Administration API
sidebar_position: 5
---

# Server Administration API

Monitor live streaming telemetry, manage global server settings, clear caches, and inspect system health.

---

### Public Server Information

Returns public metadata and capabilities:

```http
GET /api/server/info
```

#### Response (`200 OK`)
```json
{
  "name": "Kinema Media Server",
  "version": "0.0.1-alpha.5",
  "public_url": "https://192.168.1.120:3000",
  "tls_fingerprint": "a4f89d3c71e2...58",
  "transcoding_count": 0,
  "hevc_drm": true
}
```

---

### Server Health & Telemetry Dashboard (Admin)

Returns live metrics on active streams, system resource utilization, and cache sizes:

```http
GET /api/server/status
Authorization: Bearer <ADMIN_TOKEN>
```

#### Response (`200 OK`)
```json
{
  "active_streams": 2,
  "active_transcodes": 1,
  "max_transcodes": 2,
  "cache_sizes": {
    "images_bytes": 104857600,
    "thumbnails_bytes": 52428800,
    "subtitles_bytes": 4194304
  },
  "database": {
    "total_items": 1240,
    "total_files": 1240,
    "wal_size_bytes": 128000
  }
}
```

---

### Update Transcoding Settings (Admin)

Dynamically updates video encoding presets and resolution constraints:

```http
PUT /api/settings/server
Authorization: Bearer <ADMIN_TOKEN>
Content-Type: application/json

{
  "crf": 23,
  "preset": "veryfast",
  "max_height": 1080,
  "max_transcodes": 3
}
```

#### Response (`200 OK`)
```json
{
  "ok": true
}
```

---

### Purge Cache (Admin)

Deletes cached files for a specific subsystem:

```http
POST /api/server/cache/:kind/clear
Authorization: Bearer <ADMIN_TOKEN>
```

#### Path Parameters
- `kind`: `thumbnails`, `subtitles`, `images`, or `all`.

#### Response (`200 OK`)
```json
{
  "cleared": true,
  "freed_bytes": 52428800
}
```
