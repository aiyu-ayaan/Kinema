# Kinema

Kinema is a self-hosted personal media server written in Rust, in the spirit of Plex Media Server. It scans local folders, builds a library with posters and metadata, and streams to clients over the network. Priorities: very low CPU use, support for any media file, and a small, dependency-light binary.

Out of scope: paid online services (live TV channels, ad-supported streams, Plex Pass features that depend on Plex's cloud).

## End goal

Runs on all Linux flavors (AppImage + tarball, x86_64 and aarch64), Windows (installer + portable zip) and Docker (multi-arch). Each bundles the backend, the built web UI and `ffmpeg`/`ffprobe`; ffmpeg is updatable (new bundled version per release, plus an optional verified in-app update). Decisions: GPL ffmpeg run as a separate process, static musl backend on Linux with `mimalloc`, MSVC on Windows. See `development/docs/packaging.md`.

Platform rule: OS-specific code (process priority, child-process kill, paths, autostart) lives only in `kinema-core::platform` behind `cfg`. Never assume Unix paths or `nice`.

## Status

MVP built: admin setup and sign-in, libraries with folder picker, scanner + watcher, TMDB metadata, browse UI and a player with direct play / remux / transcode and audio and subtitle switching. What it does and its shortcuts: `development/docs/mvp.md`. Build order: `development/docs/roadmap.md`. Full docs index: `development/docs/README.md`.

## Stack

- Language: Rust (stable, edition 2024).
- HTTP: `axum` + `tokio`, `tower-http` for static files, range requests and compression.
- DB: SQLite via `sqlx` (WAL mode, FTS5 for search).
- Media tooling: external `ffprobe` and `ffmpeg` processes. No linking to libav.
- Metadata: TMDB API with a local fallback (filename parsing + ffmpeg frame grab).
- Web UI: same stack as Domus (`../domus`): React 18 + TypeScript, Vite, Tailwind 3, `react-router-dom`, `@tanstack/react-query`, `lucide-react`/`@phosphor-icons/react`, `@fontsource-variable/inter`, plus `hls.js` for HLS playback. pnpm workspace, web app in `apps/web` (`@kinema/web`). Built `dist` is served by the axum server. See `development/docs/web-ui.md`.

## Planned layout

```
Cargo.toml            workspace
package.json          pnpm scripts (same as Domus)
pnpm-workspace.yaml   packages: apps/*
backend/              Rust binary crate: axum routes, auth, streaming, entry point
crates/
  kinema-core/        domain types, config, errors
  kinema-db/          sqlx migrations and queries
  kinema-scanner/     file walking, filename parsing, change watching
  kinema-media/       ffprobe/ffmpeg wrappers, thumbnails, HLS, direct-play decisions
  kinema-meta/        TMDB client, artwork cache, local fallback
apps/
  web/                React + TypeScript frontend
design-system/        UI tokens and page specs
development/docs/     design docs (source of truth for decisions)
development/TODO.md   task tracking
research/             scratch downloads, never committed, never built from
```

## Design rules

1. Direct play first. Transcode only when the client cannot play the file. Remux before re-encode. Re-encode last.
2. Never do work twice. Cache probe results, thumbnails and HLS segments keyed by file size + mtime + a content fingerprint.
3. Background work (scan, probe, thumbnail) runs at low priority with a bounded worker pool. Idle server uses near-zero CPU: use filesystem events, not polling.
4. Stream files with `sendfile`/ranged reads. Never load a whole media file into memory.
5. Any file: unknown containers must still be listed and served as raw downloads. Failure to probe never hides a file.
6. Keep dependencies few. Prefer std and tokio over extra crates when the gain is small.
7. Every user-facing path goes through auth. Share links use scoped, expiring tokens.

## Clean-room rule

Do not decompile, disassemble, copy or paraphrase Plex's proprietary code or assets. Design comes from public documentation, public API behavior and our own decisions. `research/` may hold downloads for reference but nothing there is built into Kinema. Details in `development/docs/legal-clean-room.md`.

## Conventions

- `cargo fmt` and `cargo clippy -- -D warnings` must pass.
- Errors: `thiserror` in library crates, `anyhow` only in the binary.
- No `unwrap()` outside tests.
- Migrations are append-only files in `crates/kinema-db/migrations/`.
- Comments explain why, not what.
- Update the relevant file in `development/docs/` in the same change as any design change.

## Commands (once code exists)

Same as Domus. Run from the repo root with pnpm.

```
pnpm dev            # backend + web together
pnpm dev:backend    # cargo run --manifest-path backend/Cargo.toml
pnpm dev:web        # pnpm --filter @kinema/web dev
pnpm build          # web build, then cargo build --release
pnpm build:web
pnpm build:web:dev
pnpm build:web:prod
pnpm build:backend  # cargo build --release --manifest-path backend/Cargo.toml
pnpm lint           # eslint + cargo clippy
pnpm test           # cargo test + web tests
```

```
scripts/package-appimage.sh   # Linux AppImage + tarball (bundles ffmpeg)
scripts/package-windows.ps1   # Windows zip + installer
scripts/package-docker.sh     # multi-arch Docker image
scripts/update-ffmpeg.sh      # bump pinned ffmpeg in scripts/ffmpeg.lock
```

Dev needs `ffmpeg` and `ffprobe` on PATH. Resolution order: `KINEMA_FFMPEG`, bundled `$APPDIR/usr/bin`, then PATH. Never hard-code `ffmpeg`: always go through the resolver in `kinema-media`.

Dev and prod flavors, as in Domus: dev isolates localStorage keys (`kinema.dev.token`, `kinema.dev.theme`), uses `/favicon-dev.svg` and shows a `DEV` pill. Prod uses `kinema.token`, `kinema.theme`.


## AI Agents

- Never add yourself as co-auther in any project.
- Commit message will able have the `scope(Change) : Message` structure
- `development/` folder is for you. Put docs in `development/docs/`; track tasks in `development/TODO.md` (create a new one if needed).
- You are allow to update the claude.md file or any other file releated to ai brain.
- Always use `codegraph` to scan the code.
- For architecture changes, always add or update an architecture diagram in docs.

## String

- For web/desktop or backend the i18n string so that in future it will easily got migrated to any launguage. For now, we will use English as the default language. All string should be in seperate files. 
