# Media Pipeline

## 1. Discovery

- Initial walk of each library root with `walkdir`. Ignore hidden dirs, `@eaDir`, `.DS_Store`, sample files under 20 MB named `sample`.
- Live updates with `notify`. Events are debounced 2 s and coalesced per path. A file still growing (size changes between two checks 5 s apart) is deferred.
- No periodic full rescan by default. Optional per library schedule.

## 2. Identify "any file"

Extension is a hint only. Order:

1. Run `ffprobe -v quiet -print_format json -show_format -show_streams -show_chapters`.
2. If probe succeeds: classify by streams (video, audio-only, image).
3. If it fails: record `probe_status=failed`, still create an item of kind `other` with filename as title, serve as a raw download.

Probe output is stored as JSON in `files.probe_json`. Probe is skipped when the fingerprint is unchanged.

Fingerprint: `size` + xxh3 of first 64 KiB + last 64 KiB. Cheap on spinning disks and network mounts.

## 3. Playback decision

Client sends supported containers, video codecs (with profile/level), audio codecs, max bitrate, max resolution.

| Condition | Mode | CPU cost |
|---|---|---|
| Container, video, audio all supported | Direct play | Near zero |
| Video supported, audio or container not | Remux to HLS fMP4, transcode audio only if needed | Very low |
| Video unsupported or over bitrate limit | Transcode video | High |
| Subtitles are image-based (PGS) and client cannot render | Burn in (transcode) | High |

Text subtitles never force a transcode: convert to WebVTT and send as a side track.

## 4. Transcode

- Hardware first: VAAPI (Intel/AMD), NVENC, QSV, VideoToolbox, detected at startup by test-encoding a short clip.
- Software fallback: `libx264 -preset veryfast -crf 23`, capped threads.
- Output: HLS, 4 s segments, started at the requested position (`-ss` before `-i`).
- Session limits: default 2 concurrent. A third request gets HTTP 503 with a retry hint.
- Process runs under `nice 19` so playback of direct streams is never starved.
- Segments are deleted as the client moves past them. Session dir purged on idle timeout (60 s) and at startup.

## 5. Thumbnails

- Poster fallback: single frame at 10% of duration, scaled to 400 px wide, WebP. `-skip_frame nokey` and input seeking keep it to roughly one keyframe decode.
- Scrub sprites: one frame every 10 s, 160 px wide, tiled into a sprite JPEG plus a WebVTT map. Generated lazily on first request, low priority.
- Audio files: embedded cover art via ffmpeg `-an -vcodec copy`.

## 6. Subtitles

- External files `name.lang.srt|ass|vtt` matched by basename.
- Embedded text tracks extracted on demand and cached.
- `ass` converted to WebVTT with styling stripped by default.
- Extraction runs at scan time (low priority), not at play time, so the first track switch is a cache hit.
- Switching a text track is client-side only: the player swaps the `<track>` source. No server session restart, no ffmpeg run, no video reload.
- Image tracks (PGS, VobSub) are OCRed at scan time to WebVTT (optional, off by default, runs after probe at lowest priority). Without OCR they fall back to burn-in, which restarts the transcode on each change. Burn-in is the only case where a caption change costs a restart.

## 6a. Fast start and seek on large files

Goal: no Plex-style stall on multi-GB files, on seek, or on caption change.

```
 play request
      |
      v
 cached probe + keyframe index  (size+mtime+fingerprint key; miss => ffprobe once)
      |
      +-- direct play ---> ranged read / sendfile  (seek = byte-range jump, no server session)
      |
      +-- remux ---------> ffmpeg -c copy -> HLS fMP4, started at -ss, seek inside cut segments
      |
      +-- transcode -----> HLS, session kept alive across seeks; reuse segments already cut
```

- Probe and keyframe index are stored in the DB at scan time, never computed on the play path.
- MKV with Cues at the end: read the Cues once at scan time and store the offsets, so the first seek does not scan the file.
- Direct play uses `Range` with no read-ahead cap and `sendfile`; no transcode session is created.
- A seek inside an active HLS session first checks for existing segments; only a seek outside the cut range restarts ffmpeg, and it uses input seeking (`-ss` before `-i`) to the nearest keyframe.
- Idle sessions are kept 60 s so a quick seek back does not pay startup again.

## 7. Chapters and intro skip

Chapters from ffprobe are stored. A chapter titled "Intro", "Opening" or "OP" drives the skip button. Audio fingerprint intro detection is a later feature.
