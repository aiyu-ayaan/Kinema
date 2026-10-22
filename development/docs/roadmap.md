# Roadmap

## M0: Skeleton
- Cargo workspace, CI (fmt, clippy, test), config loading, tracing.
- SQLite migrations for `users`, `libraries`, `items`, `files`, `streams`.
- `/server/info`.

## M1: Scan and direct play
- Walker + watcher, fingerprinting, ffprobe job.
- Filename parser with test corpus.
- `GET /api/stream/{id}` with `Range`.
- Minimal web page: list items, play in `<video>`.
- Keyframe/Cues index stored at probe time so large-file seek is instant (`media-pipeline.md` 6a).

## M2: Metadata and artwork
- TMDB client, matcher, artwork cache and resizer.
- Local art, `.nfo` reader, frame-grab fallback.
- Poster grid UI, item detail page.

## M3: Profiles and watch state
- Setup flow, login, sessions, profile picker.
- Progress, watched flags, Continue Watching, Recently Added rows, home screen.
- FTS search.

## M4: Remux and transcode
- Capability negotiation, playback decision.
- HLS remux, audio transcode, software video transcode.
- Hardware encoder detection (VAAPI, NVENC, QSV). Done 2026-10-22 for the progressive MP4 stream.
- WebVTT subtitles, track selection. Extracted at scan time; track switch is client-side with no reload.
- Seek reuses live HLS segments; session restart only outside the cut range.
- Optional OCR of PGS/VobSub to WebVTT at scan time (avoids burn-in restarts).

## M5: Playlists, collections, sharing
- Playlists, collections, shelves.
- Share links, restricted profiles, library access control.
- Profile export and import.

## M6: Polish
- Scrub sprites, intro skip, next-episode autoplay.
- Music and photo libraries.
- mDNS discovery, ACME TLS.
- Packaging (end goal): Linux AppImage and tarball, Windows installer and zip, multi-arch Docker image, all with bundled ffmpeg/ffprobe, ffmpeg update flow and app self-update. See `packaging.md`.

## M7: Later
- Smart playlists, mobile and TV apps, DLNA, trakt-style scrobbling, intro detection by audio fingerprint.

Each milestone ends with updated docs and a passing benchmark against `performance.md` budgets.
