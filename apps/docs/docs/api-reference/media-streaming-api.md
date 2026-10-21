---
id: media-streaming-api
title: Media & Streaming API
sidebar_position: 3
---

# Media & Streaming API

Plan playback modes, stream fragmented MP4 video or direct files, retrieve cached artwork, and fetch WebVTT subtitle tracks.

---

### Request Playback Plan

Evaluates the file's audio and video streams against client decoder capabilities and returns the optimal playback mode (`direct`, `remux`, `transcode`):

```http
GET /api/files/:id/plan?hevc=true&av1=false&audio=0&start=120.5
Authorization: Bearer <token>
```

#### Query Parameters
- `hevc`: `true` if client hardware can decode H.265/HEVC.
- `av1`: `true` if client hardware can decode AV1.
- `audio`: Desired audio stream index (defaults to 0 or user preference).
- `subtitle`: Desired subtitle stream index.
- `start`: Target seek timestamp in seconds.
- `raw`: Set to `true` for Desktop `mpv` direct playback bypass.

#### Response (`200 OK`)
```json
{
  "mode": "remux",
  "start": 118.4,
  "audio": 0,
  "subtitle": 2,
  "video_codec": "h264",
  "audio_codec": "aac",
  "width": 1920,
  "height": 1080
}
```

---

### Stream Media Stream

Streams the media stream using the planned parameters:

```http
GET /api/stream/:file_id?audio=0&start=118.4
Authorization: Bearer <token>
```

#### Playback Modes
1. **Direct Play**: Returns `206 Partial Content` (or `200 OK`) with `Accept-Ranges: bytes` and `Content-Type: video/mp4`. Supports fast seeking via byte offsets.
2. **Remux / Transcode**: Returns `200 OK` with chunked transfer encoding (`Content-Type: video/mp4`) streaming live fragmented MP4 fragments.

---

### Fetch WebVTT Subtitles

Retrieves an extracted or converted WebVTT subtitle stream:

```http
GET /api/subtitles/:stream_id
Authorization: Bearer <token>
```

#### Response (`200 OK`)
`Content-Type: text/vtt; charset=utf-8`

```
WEBVTT

00:01:14.200 --> 00:01:18.500
Welcome to the Kinema media server.

00:01:19.100 --> 00:01:23.400
Enjoy clean, high-performance streaming.
```

---

### Fetch Cached Artwork / Poster

Returns cached TMDB or TVmaze artwork:

```http
GET /api/images/:size/:name
```

#### Path Parameters
- `size`: `w500`, `original`, or `w300`.
- `name`: Image filename hash.

#### Response (`200 OK`)
`Content-Type: image/jpeg`  
`Cache-Control: public, max-age=31536000, immutable`
