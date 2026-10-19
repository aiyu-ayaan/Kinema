# TODO

## M0: Skeleton
- [ ] Cargo workspace with `backend/` and `crates/*`
- [ ] Root `package.json` and `pnpm-workspace.yaml` with the Domus scripts
- [ ] `apps/web` Vite + React + TS + Tailwind scaffold (`@kinema/web`)
- [ ] Dev/prod flavors (`kinema.dev.*` storage keys, DEV pill)
- [ ] `design-system/` tokens from the reference screen
- [ ] CI: fmt, clippy, eslint, tests
- [ ] SQLite migrations for users, libraries, items, files, streams
- [ ] `GET /api/server/info`

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
