# TODO

## M0: Skeleton
- [x] Cargo workspace with `backend/` and `crates/*`
- [x] Root `package.json` and `pnpm-workspace.yaml` with the Domus scripts
- [x] `apps/web` Vite + React + TS + Tailwind scaffold (`@kinema/web`)
- [x] Dev/prod flavors (`kinema.dev.*` storage keys, DEV pill)
- [x] `design-system/` tokens from the reference screen
- [x] CI: fmt, clippy, eslint, tests
- [x] SQLite migrations for users, libraries, items, files, streams
- [x] `GET /api/server/info`

## MVP (done 2026-10-19)
- [x] First-run admin setup, sign-in, Argon2 + hashed session tokens, Bearer + cookie
- [x] Libraries: movies / shows / other, folder picker, multiple roots, delete, rescan
- [x] Scanner: release-name parser with tests, fs watcher with debounce, missing-file pruning
- [x] ffprobe at 2 concurrent, cached by size + mtime; sidecar subtitles
- [x] TMDB metadata for movies, shows, seasons, episodes; lazy image cache
- [x] Home, library grid, detail pages, search, light/dark theme (Domus-style kit, Ember palette)
- [x] Player: direct play / remux / transcode, exact seek offset, audio switch, WebVTT subtitles, resume, next episode
- [ ] Safari playback of remuxed streams (needs HLS, M4)
- [x] Family profiles: picker, PIN, lockout, per-profile library access
- [x] Per-profile language preferences; LAN-only unlocked profiles (2026-10-22, see below)
- [x] Fix-match UI for wrong TMDB matches
- [x] Episodes inside Movies libraries grouped as shows; Movies / TV Shows sidebar sub-entries
- [x] Sidecar subtitles picked up without a video change

## Android app (2026-10-19)
- [x] Android: baseline + startup profile (generated, shipped, profileinstaller); `pnpm android:profile`
- [x] Android: cold start measured on a phone: 1053 -> 976 ms median
- [x] Android: profile recorded signed in (Home, Library, title page, Settings): cold start 605 -> 534 ms on a phone
- [x] HTTPS by default with a self-signed certificate; fingerprint on the dashboard, pinned by the app
- [x] LAN discovery beacon (UDP 48555) and app-side scan, emulator / adb fallback
- [x] Public address: server setting plus per-app override; automatic home/public switching on network change
- [x] Compose app (watch only): connect, trust, profiles, home hero, library, search, title page, settings
- [x] Media3 player: direct / remux / transcode, audio + subtitle switch, resume, next episode, double-tap seek
- [x] Predictive back on every page (Navigation 3) and inside the tab host
- [x] Picture-in-picture (back / play / forward actions) and a media session (lock-screen controls); not run on a device yet
- [x] Seek inside the buffered part of a remux stream: decision logic (`StreamSeek`) and player hook, only applies when the player reports the stream seekable
- [ ] Make the piped fMP4 remux/transcode stream seekable inside the buffer for ExoPlayer (custom SeekMap + back buffer, or sidx/HLS) so the above takes effect; verify on a device
- [x] Translations scaffolding: strings audited, es / fr / de / hi drafts for 51 core strings (machine quality, needs native review)
- [ ] Finish and review the translations (Connect, Trust, Settings, Updates, About, watch page, plurals)

## Desktop app (2026-10-20)
- [x] Electron scaffold (`apps/desktop`, `@kinema/desktop`) with isolated dev/prod flavors
- [x] Remove default menu items (`Menu.setApplicationMenu(null)` / `win.setMenu(null)`)
- [x] LAN server discovery beacon probe (UDP 48555) with local fallback
- [x] Server switcher and manual address input with TLS certificate verification and pin approval
- [x] Web integration: server change button on profile picker, setup, settings, and user menu

## Playback speed (Plex pain points)
- [x] Store keyframe/Cues index at probe time (M1): Matroska Cues, MP4 stss, bounded ffprobe packet scan; remux plan start from it
- [ ] Direct play: ranged `sendfile`, no session (M1)
- [x] Scan-time subtitle extraction to cached WebVTT (M4)
- [x] Client-side subtitle track switch, no player reload (M4)
- [x] Optional OCR of PGS/VobSub to WebVTT (M4)
- [ ] Android: offer picture subtitles that have `ocr: true` (web and desktop mpv do)
- [ ] Bundle `tesseract` (+ eng model) in AppImage / Windows / Docker, or document installing it
- [ ] Prune subtitle cache entries of old file versions (they stay until the cache is cleared)
- [ ] OCR: check accuracy on real Blu-ray PGS and DVD VobSub (only a generated PGS sample was tested)
- [x] HLS seek reuses existing segments; keep idle session 60 s (M4): server side, `/api/hls/{id}/index.m3u8` + `{n}.ts`
- [x] Pause HLS runs on Windows (`platform::suspend_process` via NtSuspendProcess); compiles for `x86_64-pc-windows-gnu`, not run on a Windows machine
- [ ] fMP4 HLS segments (HEVC copy for players without HEVC-in-TS)

## Packaging (end goal)
- [x] Decisions: GPL ffmpeg as separate process; static musl (Linux) + MSVC (Windows); aarch64 in first release
- [x] `kinema-core::platform` (priority, child kill via `kill_on_drop`, paths, data dirs)
- [ ] Spike: static ffmpeg loads host VAAPI libs
- [x] `scripts/package-windows.ps1`: server zip (kinema.exe + web + ffmpeg, checksum-verified); not run on Windows yet. Inno Setup installer for the server: not done (the desktop app has its own `-Setup.exe`)
- [x] Dockerfile + docker-compose.yml (Alpine, static musl backend, non-root 1000:1000; no PUID/PGID or tini)
- [x] GitHub Actions matrix (`release.yml`: server x86_64/aarch64, image, Linux and Windows desktop, Android)
- [ ] Release smoke tests (start the binary, hit `/api/server/info`, probe a sample with the bundled ffprobe)
- [ ] Windows server zip job in `release.yml`
- [ ] Release signing key, Windows Authenticode
- [x] `scripts/ffmpeg.lock` + `scripts/update-ffmpeg.sh` (checks BtbN publishes the version for all three platforms; hashes come from the published `checksums.sha256` at package time)
- [x] ffmpeg/ffprobe resolver in `kinema-media`
- [x] ~~`scripts/package-appimage.sh`~~ replaced: the server ships as `kinema-server-*.tar.gz` (`package-server.sh`), the desktop app as AppImage / .deb through electron-builder
- [ ] In-app ffmpeg update with hash + signature check
- [ ] AppImage self-update (zsync)

See `docs/roadmap.md` for later milestones.

## Watch page, tablets and skip markers (2026-10-20)
- [x] Android: home hero and title page sized for tablets and foldables
- [x] Android + web: Up next card only from the credits (else the last minute), with countdown and Play now
- [x] Android: YouTube-style watch page on tablets / open foldables, episodes or suggestions list, full screen toggle; phone side panel
- [x] Android: player opened from a player no longer comes back in portrait with bars showing
- [x] Web: watch page with up-next list, theater mode (t), full window width
- [x] Server: intro / credits markers from chapters, audio shared across a season, black frames; admin override API
- [x] Android + web: Skip intro, Skip credits
- [ ] Admin UI to set or clear a marker: built in the web player (flag button, `MarkerEditor.tsx`); tsc/eslint clean and the API calls curl-tested, but not yet clicked through in a browser. Android has none
- [ ] Credits over lit footage: stable picture + edge detail check added (`source = text`, media-pipeline.md §7); tuned and tested on synthetic ffmpeg footage only. Check it on real films and episodes (`KINEMA_CREDITS_FILE=... look_at_a_file`), adjust thresholds; busy-backdrop credits still missed; no loudness check
- [x] Android tablet keyboard shortcuts (f, t, space, k, arrows) like the web player; volume / mute / stop keys not mapped

## Cheaper HEVC playback (2026-10-21)
Trigger: F1 (1080p HEVC Main 10) transcoded at ~90 % CPU on the Pi 5 because Chrome on Linux has no HEVC decoder.
- [x] ~~Desktop: Chromium VAAPI + HEVC switches on Linux~~ reverted 2026-10-22: Chromium then claimed HEVC it could not decode and F1 stopped with "could not be played"
- [x] Web player: a decode failure with HEVC/AV1 claimed retries with them off, so the server transcodes
- [x] Web player: repeated skips build on the pending seek target instead of the old position
- [x] Transcode: half-second fragments, play/seek start 6-9 s -> ~2.5 s on the Pi; last frame held during a restart
- [x] Transcode throttle: 60 s lead, then real time (`playback::Progress`)
- [x] Real process priority: `nice 10` live streams, `nice 19` background (`kinema-core::platform`)
- [x] Pi 5 HEVC hardware decode through the distribution ffmpeg (`MediaBinaries::hevc_drm`)
- [x] Desktop: play in mpv through a loopback proxy (`apps/desktop/src/mpv.js`), server `raw=true`
- [x] Transcode froze the picture after a seek while the clock ran (2026-10-22): copied audio started at the keyframe before the seek, video at the seek point; audio is now re-encoded in a transcode. Checked in Chrome and the desktop app (Electron 33): F1 seeks resume in ~1 s at 24 fps
- [ ] Deploy to the Pi: `sudo apt install ffmpeg` is already there; restart Kinema and check the `HEVC hardware decode` log line
- [ ] Try the mpv path by hand in the desktop app (mpv not installed on the dev laptop; proxy and raw seeks tested with ffmpeg)
- [x] Bundle mpv with the desktop app: resolver order, fetch script, lock, extraResources, notice, in-app fallback (`scripts/fetch-mpv.mjs`)
- [ ] Pin `scripts/mpv.lock` (Windows x64, Linux x86_64 and aarch64): `node scripts/fetch-mpv.mjs --pin <platform> <url>`, compare with the publisher checksum; until then installers ship without mpv
- [ ] Try the bundled mpv by hand: AppImage, .deb and Windows install start it from `resources/mpv` (the nested AppRun wrapper is untested)
- [ ] Optimized versions: pre-convert files no browser plays, at low priority, cached by fingerprint
- [ ] LAN transcode helper (offload re-encodes to a machine with a GPU, like rffmpeg)

## Transcode CPU (2026-10-22)
Trigger: transcode-only viewing took every core while Plex stayed quiet on the same files.
- [x] Throttle lead 60 s -> 30 s: the full-speed burst after every play or seek is half as long
- [x] `-threads` per re-encode: setting `transcode_threads`, Auto = cores / `max_transcodes`, at least 4
- [x] libx264 peak rate by output height (`-maxrate`/`-bufsize`), lookahead 10 for presets slower than veryfast
- [x] GPU encode: NVENC, Quick Sync, VA-API found by a test encode at startup, settings switch
- [x] NVENC measured on the i5-10200H laptop (GTX 1650): 13.6 -> 2.2 CPU-seconds per 20 s of 1080p
- [ ] Measure x264 changes (lead, threads, maxrate) on real files, and everything on the Pi 5
- [ ] Quick Sync: needs `-init_hw_device qsv` and hw frames; rejected as written on the laptop
- [ ] VA-API: bundled ffmpeg aborts against the host libva (`vaMapBuffer2`); see the packaging spike
- [ ] Per-stream fallback to libx264 when the GPU encoder fails after startup
- [ ] AMF (AMD on Windows)

## Locked fields, polling, playlists (2026-10-22)
- [x] `items.locked_fields`: edited fields survive rescans, TMDB refresh and Fix match; Edit details dialog with a lock per field
- [x] Per-root polling for NFS/SMB (`library_roots.poll`, interval 1 to 1440 min, default 5), directory-hash diff, bounded at 500k entries
- [x] Playlists: CRUD, add from the item page, playlist page with reorder, owner-only edits, shared read-only, autoplay of the next entry
- [x] Collections: TMDB `belongs_to_collection` on match, manual admin collections, Home row and collection page
- [x] Genre shelves on Home ("Oh, The Horror!" seeded), admin settings, per-profile hide
- [x] Add to playlist from poster cards and the watch page, not only the item page
- [ ] Drag and drop reordering in playlists (buttons only now), shuffle at play time (`[x]` shuffle implemented), playlists of music tracks
- [x] Remove a single title from a collection in the UI (API only)
- [ ] Keep an edited title when a new file of the same release is scanned (grouping still uses the parsed name)
- [ ] Upload or pick a custom poster and backdrop (only locking the current art exists)
- [x] Android: playlists, collections and genre shelves
- [ ] Smart playlists, share links for playlists
- [ ] Try polling on a real NFS/SMB mount (tested on a local folder only)

## Scrub thumbnails (2026-10-22)
- [x] Server: lazy sprite sheets + WebVTT index per file (keyframe-only, `nice 19`, shared background slot, one run per file, cached by id+mtime+size), `GET /items/{id}/timeline.vtt` and `/items/{id}/sprites/{n}` behind auth; curl-tested end to end on a synthetic file
- [ ] Web: hover / drag preview on the seek bar (`ScrubPreview.tsx`); tsc/eslint clean, not yet seen in a browser
- [ ] Measure sprite generation on real 4K HEVC / long files and on the Pi 5
- [x] Android player: use the same sprites for its seek bar
- [ ] Sprites for audio-only or very sparse-GOP files (one keyframe per minute gives repeated tiles)

## Profiles and household (2026-10-22)
- [x] Per-profile audio / subtitle language and subtitle mode, used by the plan (`profile_prefs`)
- [x] LAN-only unlocked profiles: no-lock profiles and `lan_unlock` PIN skip only from LAN; `lan_only` sessions; `[network] trusted_proxies`
- [x] Content limits per profile (age scale, unrated toggle) on every listing, search, item and stream path; TMDB certifications + admin override
- [x] Share links: selector + hashed verifier, constant-time check, expiry, revoke, scoped `/api/share/{token}/...` routes; admin UI and `/s/{token}` page
- [x] Profile export / import (watch history, preferences; lock hashes only on explicit admin request)
- [x] Access-control tests over the router (`backend/src/access_tests.rs`)
- [ ] Share links: per-IP rate limit, max uses, per-link transcode cap
- [ ] Non-admin users creating share links for titles they can see (admin only for now)
- [ ] Content limits in SQL instead of after the query, so limited profiles get full search / shelf pages
- [ ] Android app: show `home_only` profiles and handle `auth.lan_session` (picker and re-sign-in)
- [ ] Global IPv6 on the home network is treated as remote; consider matching the server's own /64 prefixes
- [x] Re-match keeps the old certification: Fix match now replaces the rating (`items::replace_content_rating`); a manual rating stays
- [x] `/api/images/...` checks the file against the titles using it (`access::check_image`); unlimited profiles skip it. Cast photos and art no title uses stay open to any profile
- [x] Scrub thumbnails: admin toggle `scrub_thumbnails` (Settings › Library), default off. E2E checked: 404 when off, 202 then 200 when on; API calls 0.3–1.5 ms.
- [x] Page `GET /libraries/{id}/items`; web grid and Android library tab load pages (see api.md, performance.md).
- [ ] Verify scrub `-hwaccel auto` on a machine with VA-API / NVDEC.
- [ ] Lighthouse pass on the built web bundle.

## Accounts: devices, activity, requests, notifications (2026-10-28)
- [x] Backend: device approval at sign-in (`admit`, `/auth/device/poll`, `/devices`), one web/desktop + one mobile free per non-admin profile, LAN-only exemption, pending cap 5
- [x] Backend: watch time from progress reports, `/activity/now`, `/activity/users`, `/profile/stats`
- [x] Backend: title requests (`/requests`, TMDB search, duplicate / in-library checks)
- [x] Backend: admin notifications (ntfy, Gotify, Discord, Slack, Telegram, webhook, SMTP via `lettre`) with masked secrets
- [x] Backend integration tests (`backend/src/accounts_tests.rs`) and kinema-db unit tests; docs updated
- [x] `lettre` added to the web credits list
- [ ] Web: send `device`, handle `approval_required` + polling, admin devices / activity / requests / notifications screens (pending verification)
- [ ] Android: same sign-in flow, requests and stats screens (pending verification)
- [ ] Desktop: send `kind: desktop` device info (pending verification)
- [ ] Notifications: live-test every channel kind and SMTP security mode against real servers
- [ ] Device approval: push the result to the waiting client (SSE) instead of polling

## Future scope
- [ ] Android: measure frame times (`FrameTimingMetric`, JankStats) for Library scroll and title open, with and without the baseline profile; tune what shows up slow
