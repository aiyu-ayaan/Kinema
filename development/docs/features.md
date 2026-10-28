# Features

Modeled on the Plex home screen: left sidebar of libraries, "Continue Watching" row, "Recently Added" rows per library, poster grids, and a profile menu. Poster art, title, year or season/episode label, and a progress bar under partially watched items.

## Libraries

| Type | Notes |
|---|---|
| Movies | One item per movie. Grouped into collections (franchises). |
| TV Shows | Show, season, episode hierarchy. Each show can also pin as its own sidebar entry. |
| Music | Artist, album, track. |
| Photos | Folders and albums, EXIF date. |
| Other videos | Home videos with no online lookup. |

Each library has: name, type, one or more root folders, language, and a scan schedule. Each root is either watched with filesystem events (default) or polled every 1 to 1440 minutes (default 5) for network shares that send no events: a poll lists the folder tree (names, plus size and mtime of video files), compares one hash per directory with the last snapshot and scans only when one differs. Polls are bounded to 500,000 entries per folder and run one folder at a time; a folder that is not mounted is skipped instead of looking empty. See `architecture.md`.

## Home screen

- Continue Watching: items with progress between 5% and 95%, plus the next unwatched episode of a show being watched. Sorted by last played.
- Recently Added in each library.
- Custom rows: genre shelves (for example "Oh, The Horror!"), playlists, collections.
- Search box across all libraries (FTS5).

## Playback

- Direct play, remux and transcode (see `media-pipeline.md`).
- Resume position per profile.
- Audio and subtitle track selection. External subtitles (srt, ass, vtt) next to the file. Embedded subtitles extracted to WebVTT on demand.
- Next-episode autoplay, skip intro (phase 2, based on chapter markers).
- Mark watched / unwatched.

## Thumbnails and artwork

- Poster, backdrop and logo from TMDB.
- Fallback: frame grab from the video at 10% of runtime.
- Timeline scrub thumbnails (sprite sheet plus WebVTT index) generated lazily.
- Image sizes cached: 200, 400, 800 px WebP.

## Playlists and collections

See `playlists.md`.

## Users

See `users-and-sharing.md`. Also there:

- Device approval: a non-admin profile signs in on one web/desktop and one mobile device freely; more devices wait for the admin.
- Title requests: profiles ask for a movie or show, the admin answers.
- Activity: live now playing, watch time per profile (totals, daily, top titles and genres, history).
- Admin notifications to ntfy, Gotify, Discord, Slack, Telegram, a webhook or e-mail.

## Network

- LAN discovery via mDNS.
- Remote access via reverse proxy or built-in HTTPS with ACME.
- Range requests, HTTP/2, gzip/brotli for JSON.

## Explicitly excluded

Live TV channels, paid or ad-supported online streaming, Plex Pass-only cloud features.
