# Web UI

The web UI reuses the stack and commands of the sibling project Domus (`../domus`), so both projects feel and build the same.

## Stack

| Concern | Choice |
|---|---|
| Framework | React 18 + TypeScript (~5.7) |
| Build | Vite 6 |
| Styling | Tailwind CSS 3, `clsx`, `tailwind-merge` |
| Routing | `react-router-dom` 7 |
| Data | `@tanstack/react-query` 5 |
| Icons | `lucide-react`, `@phosphor-icons/react` |
| Font | `@fontsource-variable/inter` |
| Playback | native `<video>` for direct play, `hls.js` for remux/transcode |
| Package manager | pnpm workspace (`apps/*`) |

## Layout

```
apps/web/
  src/
    components/      shared UI: PosterCard, Row, ProgressBar, Player
    features/
      home/          Continue Watching, Recently Added rows
      library/       poster grids, filters, sort
      item/          movie, show, season, episode detail
      player/        video player, track menu, resume
      playlists/
      profiles/      profile picker, settings, sharing
      admin/         libraries, users, jobs
    hooks/
    lib/             api client, auth, flavor helpers
    App.tsx
  package.json      name: @kinema/web
design-system/       tokens and page specs (dark theme first)
```

## Screens

Matches the target reference screen:

- Left sidebar: Home, one entry per library, pinned shows, "More".
- Top bar: search, profile menu.
- Home: "Continue Watching" row (poster, title, subtitle, progress bar), "Recently Added in <library>" rows, genre shelves.
- Poster cards: 2:3 art, title, year or `S1 · E5`, watched check, partial-progress bar.
- Dark theme default, light theme optional.

## Data flow

```
React components
   | useQuery / useMutation (TanStack Query)
   v
lib/api.ts  --- fetch /api/... (Bearer token) --->  axum backend
   ^                                                  |
   |------ SSE /api/server/events (scan progress) ---|
Player: <video src=/api/stream/{id}>  or  hls.js -> /api/hls/{session}/master.m3u8
```

Query keys follow `['libraries']`, `['items', libraryId, filters]`, `['home']`. Progress is sent every 10 s with `PUT /api/items/{id}/progress`. Posters use `/api/images/{key}?w=400` with `loading="lazy"`.

## Flavors

As in Domus, dev and prod instances must not interfere.

- Dev: localStorage keys `kinema.dev.token`, `kinema.dev.theme`; favicon `/favicon-dev.svg`; `DEV` pill in the navigation. Built with `pnpm build:web:dev`.
- Prod: `kinema.token`, `kinema.theme`. Built with `pnpm build:web:prod`.

## Serving

`pnpm build:web` outputs `apps/web/dist`. The axum server serves it with `tower-http::ServeDir`, with an `index.html` fallback for client routes, and `/api` routes taking precedence. In dev, Vite proxies `/api` to the backend.

## Performance

- Route-level code splitting; the player and hls.js load only on the player route.
- Virtualized poster grids for libraries over 500 items.
- Images are WebP, lazy-loaded, sized by `w=` query to avoid downscaling in the browser.

## Architecture diagram

```mermaid
flowchart LR
  Browser["Browser: React SPA"] -->|REST + SSE| API["backend (axum)"]
  Browser -->|Range GET| Stream["/api/stream"]
  Browser -->|HLS| HLS["/api/hls"]
  API --> DB[(SQLite)]
  API --> Jobs["job queue"]
  Jobs --> FF["ffprobe / ffmpeg"]
  Jobs --> TMDB["TMDB API"]
  Stream --> Disk[("media files")]
  HLS --> FF
  API -->|static| Dist["apps/web/dist"]
```
