# Boot checks and client features

Things the server must find out about the machine (is ffmpeg there, is there a GPU encoder, is `tesseract` installed) are checked **once at startup** in `backend/src/boot.rs`, not per request and not by clients.

```
 startup                      request time
 -------                      ------------
 boot::run()                  AppState::features()
   media_tools()  -> bins  ─┐        │
   ocr()          -> ocr   ─┼─ Report ├─ + live admin settings (ServerSettings)
   <your check>   -> ...   ─┘        ▼
                                  Features ──> GET /api/items/{id}  (`features`)
                                                   │
                                  web (`detail.features`) / Android (`detail.features`)
                                  ask for a feature only when its flag is true
```

## Rules

- A check runs in `boot::run()`, logs what it found at `info` (or `warn` when a feature is lost), and stores the result in `boot::Report`. It never runs again while the process lives.
- `Features` is what clients may use now: the boot result **and** the admin's switch. It is computed per request from memory (no I/O) by `AppState::features()`, so an admin toggle takes effect without a restart.
- Server code that does expensive work for a feature (for example `sprites::status`) checks `state.features()` first, so a disabled feature costs nothing even if a stale client still asks. The endpoint then answers 404.
- Clients read `features` from the item detail they already load before playing. They do not probe the endpoint to find out. A client that does not know a flag treats it as off.

## Adding a boot task

1. Add a field to `boot::Report` and fill it in `boot::run()` with a small function that logs its outcome.
2. Move the field into `AppState` in `main.rs` (the `let boot::Report { .. } = boot::run();` line) if request code needs it.
3. If clients must know, add a bool to `boot::Features`, set it in `Features::new`, and add a test next to `scrub_needs_ffmpeg_and_the_switch`.
4. Mirror the flag in `apps/web/src/lib/types.ts` (`ItemDetail.features`) and `Models.kt` (`Features`, default off), then gate the client request on it.
5. Document the check here and any admin setting in `api.md`.

## Current checks

| Check | Report field | Client flag | Admin setting |
|---|---|---|---|
| ffmpeg / ffprobe, hardware encoder, DRM HEVC decode | `bins` | `scrub_thumbnails` (needs ffmpeg) | `scrub_thumbnails` (off by default) |
| `tesseract` and its languages | `ocr` | none (the stream's `ocr` flag is enough) | `subtitle_ocr` |
