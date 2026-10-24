# Performance and Low-CPU Strategy

## Biggest wins, in order

1. Direct play. Serving a file costs a few syscalls per MB. Transcoding costs thousands of times more. Maximize the share of playback that is direct or remux-only.
2. Hardware transcode when transcode is unavoidable.
3. Never repeat work: fingerprint-keyed caches for probe, thumbnails, sprites, subtitles, images.
4. Event-driven scanning (inotify) instead of polling.
5. Low-priority background workers.

## Budgets

| Scenario | Target |
|---|---|
| Idle, 10k items | < 30 MB RSS, < 0.1% CPU |
| Direct play 4K remux | < 2% of one core |
| API list of 50 items | < 10 ms p95 |
| Cold start | < 1 s |
| Image request (cached) | < 2 ms |

## Techniques

- SQLite: WAL, prepared statements, covering indexes on `(library_id, added_at)`, `(parent_id)`, `(user_id, last_played_at)`. Pagination uses keyset where possible.
- Home screen rows are cheap single queries; no N+1.
- JSON via `serde_json` with pre-sized buffers; compression only for responses over 1 KiB.
- Images: serve WebP with `ETag` and `Cache-Control: immutable`; resizing happens once.
- Streaming: `tokio::fs::File` + `ReaderStream` with a 256 KiB buffer; `Range` parsed once; `sendfile` through hyper when available.
- ffprobe: one process per file, bounded parallelism of 2 on HDD, 4 on SSD (autodetected via `/sys/block/*/queue/rotational`).
- Thumbnail: input seeking and keyframe-only decode.
- Scrub sprites: off by default (admin setting `scrub_thumbnails`); keyframe-only decode of the whole file once, on first scrub, at `nice 19`. Measured on a synthetic 20 min 1080p H.264 file (GOP 240, 1.9 GB, 4-core VM): 3.3 s wall, 1.9 s CPU for 2 sheets. Real footage with HEVC 4K decodes slower per keyframe; not measured.
- Allocator: `mimalloc` or `jemalloc` optional feature.
- Release profile: `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`, `opt-level = 3`.
- Avoid holding the DB writer across awaits; batch inserts in transactions of ~500 rows during scans.

## Throttling

- A live re-encode is held to 30 s ahead of real time, then paced to playback (see `media-pipeline.md` § 4).
- An HLS run (remux or re-encode) is stopped (SIGSTOP) once it is 30 s past the last segment asked for and continued when the player nears; idle sessions end after 60 s (`media-pipeline.md` 6a). Not on Windows yet.
- Each re-encode gets a share of the cores (`-threads`, Auto = cores / `max_transcodes`), so even the full-speed lead never takes the whole machine.

- Scan and thumbnail workers pause while any transcode session is active or while system load per core exceeds 0.8.
- Network and disk reads for background jobs are rate limited so playback gets priority.

## Measuring

Benchmarks in `benches/` with `criterion`; load tests with `oha`; CPU profiling with `perf` and `cargo flamegraph`. Regressions against budgets fail CI.

## Measured 2026-10-24 (8 cores, release build, Python client with keep-alive, server and client on one box)

Library: 6000 hardlinked 2 s clips (one folder). Scrub clip: synthetic 4K HEVC, 240 s, keyframe every 2 s.

| What | Before | After | Change |
|---|---|---|---|
| `/home`, 1 connection | 166 rps, 4.8 ms | 588 rps, 1.4 ms | library counts read a covering index; shelf cards joined for 24 rows, not 6000 (`20261024000001_home_indexes.sql`) |
| `/home`, 64 connections | 463 rps, p99 255 ms | 1642 rps, p99 68 ms | same |
| mixed `/home`, `/items/1`, `/search`, `/auth/me`, 100 connections | 837 rps, p99 336 ms | 1777 rps, p99 137 ms | same |
| `/items/{id}`, 64 connections | 3940 rps | 3970 rps | already cheap |
| `/search?q=Movie 0042` (few hits) | 1203 rps | 1376 rps | |
| `/search?q=movie` (6000 hits) | 518 rps | 592 rps | still a LIKE scan plus sort of every hit; FTS5 only matters beyond tens of thousands of titles |
| web `assets/index-*.js` | 439 KB, no cache header | 110 KB gzip, `immutable` for a year | `web::cache_assets`, `CompressionLayer` on static files only |

- First scan and probe of 6000 files: about 35 s, 15 s of server CPU, 40 MB RSS (ffprobe children not counted, 2 at a time).
- Idle server: no CPU time accrued over 30 s; about 17 MB RSS.
- Scrub sheets, 4K HEVC (nice 19): 5.4 s CPU for 240 s of video with ffmpeg's default threads, 5.3 s with `-threads 1` (the keyframe-only decode barely uses more than one core anyway) but 305 MB RSS drops to 181 MB. That is about 2.2 s of CPU per minute of video with a 2 s GOP, so a 2 h file costs about 160 s of one core, once. A longer GOP costs proportionally less (cost follows keyframes, not the tile interval, so a wider interval does not help).
- Two simultaneous 4K HEVC to 1080p software transcodes: each held to its 4 threads (about 250 % CPU each, nice 15 = server nice 5 + 10), 500 % of 800 %. The 30 s read-ahead throttle never engaged because the encode ran slower than real time.
- `-hwaccel auto` could not be verified here: this ffmpeg build lists only `vdpau`, device creation fails and ffmpeg continues in software with exit 0.

### Open findings

- `GET /libraries/{id}/items` returns the whole library: 2 MB for 6000 titles, 20 rps at 64 connections. It needs paging (web library grid and Android library tab change with it).
- `libraries::list` still counts per request; cache the counts per library and invalidate on scan if libraries grow past ~50k titles.
- Not run: Lighthouse, `perf`/`strace` (not permitted here), a real 2 h file, GPU decode.
