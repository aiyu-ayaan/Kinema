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
- [ ] ffmpeg/ffprobe resolver in `kinema-media`
- [ ] `scripts/package-appimage.sh`
- [ ] In-app ffmpeg update with hash + signature check
- [ ] AppImage self-update (zsync)

See `docs/roadmap.md` for later milestones.
