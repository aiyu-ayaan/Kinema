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
