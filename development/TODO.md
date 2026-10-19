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
- [ ] Per-profile language preferences; LAN-only unlocked profiles
- [x] Fix-match UI for wrong TMDB matches
- [x] Episodes inside Movies libraries grouped as shows; Movies / TV Shows sidebar sub-entries
- [ ] Sidecar subtitles picked up without a video change

## Android app (2026-10-19)
- [x] HTTPS by default with a self-signed certificate; fingerprint on the dashboard, pinned by the app
- [x] LAN discovery beacon (UDP 48555) and app-side scan, emulator / adb fallback
- [x] Public address: server setting plus per-app override; automatic home/public switching on network change
- [x] Compose app (watch only): connect, trust, profiles, home hero, library, search, title page, settings
- [x] Media3 player: direct / remux / transcode, audio + subtitle switch, resume, next episode, double-tap seek
- [x] Predictive back on every page (Navigation 3) and inside the tab host
- [ ] Picture-in-picture and a media session (lock-screen controls)
- [ ] Seek inside the buffered part of a remux stream without restarting it
- [ ] Translations (`values-<lang>/strings.xml`)

## Desktop app (2026-10-20)
- [x] Electron scaffold (`apps/desktop`, `@kinema/desktop`) with isolated dev/prod flavors
- [x] Remove default menu items (`Menu.setApplicationMenu(null)` / `win.setMenu(null)`)
- [x] LAN server discovery beacon probe (UDP 48555) with local fallback
- [x] Server switcher and manual address input with TLS certificate verification and pin approval
- [x] Web integration: server change button on profile picker, setup, settings, and user menu

## Playback speed (Plex pain points)
- [ ] Store keyframe/Cues index at probe time (M1)
- [ ] Direct play: ranged `sendfile`, no session (M1)
- [ ] Scan-time subtitle extraction to cached WebVTT (M4)
- [ ] Client-side subtitle track switch, no player reload (M4)
- [ ] HLS seek reuses existing segments; keep idle session 60 s (M4)
- [ ] Optional OCR of PGS/VobSub to WebVTT (M4)

## Packaging (end goal)
- [x] Decisions: GPL ffmpeg as separate process; static musl (Linux) + MSVC (Windows); aarch64 in first release
- [ ] `kinema-core::platform` (priority, child kill, paths, data dirs)
- [ ] Spike: static ffmpeg loads host VAAPI libs
- [ ] `scripts/package-windows.ps1` (zip + Inno Setup)
- [ ] Dockerfile + docker-compose.yml (debian-slim, PUID/PGID, tini)
- [ ] GitHub Actions matrix + smoke tests
- [ ] Release signing key, Windows Authenticode
- [ ] `scripts/ffmpeg.lock` + `scripts/update-ffmpeg.sh`
- [x] ffmpeg/ffprobe resolver in `kinema-media`
- [ ] `scripts/package-appimage.sh`
- [ ] In-app ffmpeg update with hash + signature check
- [ ] AppImage self-update (zsync)

See `docs/roadmap.md` for later milestones.
