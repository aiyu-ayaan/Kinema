# HTTP API

Base path `/api`. JSON bodies. Auth: `Authorization: Bearer <session token>` or `kinema_session` cookie. Errors: `{"error": {"code": "...", "message": "..."}}` with the matching HTTP status.

This is Kinema's own API. It is not Plex-compatible.

## Implemented in the MVP

| Method | Path | Purpose |
|---|---|---|
| GET | `/auth/status` | `{needs_setup}`: true until the first account exists |
| POST | `/auth/setup` | `{name, password}`: create the administrator (first run only), returns `{token, user}` and sets the cookie |
| POST | `/auth/login` | `{name, password}` returns `{token, user}` and sets `kinema_session` |
| POST | `/auth/logout` | Invalidate session, clear cookie |
| GET | `/auth/me` | Current user |
| GET/POST | `/libraries` | List / admin create `{name, kind: movie\|show\|other, paths}` |
| DELETE | `/libraries/{id}` | Admin: remove (files on disk untouched) |
| POST | `/libraries/{id}/scan` | Admin: rescan |
| GET | `/libraries/{id}/items` | Top-level items with watch state |
| GET | `/fs?path=` | Admin: folders on the server, for the folder picker |
| GET | `/home` | `{continue_watching, shelves: [{library, kind: movies\|shows\|episodes, items}]}` |
| GET | `/search?q=` | Title search |
| GET | `/items/{id}` | Item, genres, files with streams, children, prev/next sibling |
| PUT | `/items/{id}/progress` | `{position_s, watched?}` |
| GET | `/files/{id}/plan?audio=&start=&hevc=&av1=` | `{mode: direct\|remux\|transcode, duration_s, audio, start}` |
| GET | `/stream/{id}?audio=&start=&hevc=&av1=` | Direct (Range) or fMP4 pipe, per the plan |
| GET | `/subtitles/{stream_id}` | Text subtitle as WebVTT, cached |
| GET | `/images/{size}/{name}` | TMDB art, cached |
| GET | `/items/{id}/thumb` | Video frame JPEG for items without art, cached per file version |
| GET | `/items/{id}/match?q=&year=` | Admin: TMDB candidates for a movie or show |
| POST | `/items/{id}/match` | Admin: `{tmdb_id}`, replace the item's metadata with that result |
| GET/PUT | `/settings` | Admin: `{tmdb_configured, ffmpeg_found}` / `{tmdb_api_key}` (verified before saving) |

The tables below are the full target API.

## Auth and profiles

| Method | Path | Purpose |
|---|---|---|
| POST | `/auth/login` | `{name, password?|pin?}` returns session token |
| POST | `/auth/logout` | Invalidate session |
| GET | `/users` | List profiles (names and avatars only, for the picker) |
| POST | `/users` | Admin: create profile |
| PATCH | `/users/{id}` | Edit own profile, or admin edits any |
| DELETE | `/users/{id}` | Admin |
| PUT | `/users/{id}/libraries` | Admin: set library access |

## Libraries and browsing

| Method | Path | Purpose |
|---|---|---|
| GET | `/libraries` | Libraries the user can see |
| POST | `/libraries` | Admin: create |
| POST | `/libraries/{id}/scan` | Admin: trigger scan |
| GET | `/libraries/{id}/items?sort=&filter=&offset=&limit=` | Paged items |
| GET | `/items/{id}` | Item detail with files and streams |
| GET | `/items/{id}/children` | Seasons of a show, episodes of a season |
| GET | `/home` | Continue Watching, Recently Added rows, shelves |
| GET | `/search?q=` | FTS search |

## Images

| Method | Path | Purpose |
|---|---|---|
| GET | `/images/{key}?w=400` | Poster/backdrop/thumb, WebP, `Cache-Control: immutable` |
| GET | `/items/{id}/timeline.vtt` | Scrub-thumbnail index |
| GET | `/items/{id}/sprite.jpg` | Scrub sprite sheet |

## Playback

| Method | Path | Purpose |
|---|---|---|
| POST | `/playback/decide` | Body: client codec/container capabilities and file id. Returns `direct`, `remux` or `transcode` and a URL |
| GET | `/stream/{file_id}` | Direct play, supports `Range` |
| GET | `/hls/{session}/master.m3u8` | Adaptive playlist |
| GET | `/hls/{session}/{segment}` | Segment |
| GET | `/subtitles/{file_id}/{stream_idx}.vtt` | Subtitle as WebVTT |
| PUT | `/items/{id}/progress` | `{position_s}`, sent every 10 s |
| POST | `/items/{id}/watched` | Mark watched, DELETE to unmark |

## Playlists

| Method | Path | Purpose |
|---|---|---|
| GET | `/playlists` | Own and shared |
| POST | `/playlists` | Create |
| PATCH | `/playlists/{id}` | Rename, share toggle |
| PUT | `/playlists/{id}/items` | Replace ordered list |
| POST | `/playlists/{id}/items` | Append |
| DELETE | `/playlists/{id}` | Delete |

## Sharing

| Method | Path | Purpose |
|---|---|---|
| POST | `/shares` | Create link for item, playlist or library, with expiry and max uses |
| GET | `/shares` | List own links |
| DELETE | `/shares/{id}` | Revoke |
| GET | `/s/{token}` | Public landing page for a link |

## Server

| Method | Path | Purpose |
|---|---|---|
| GET | `/server/info` | Version, name, capabilities |
| GET | `/server/jobs` | Admin: queue state |
| GET | `/server/events` | SSE stream: scan progress, new items |

Pagination: `offset` and `limit` (max 200). Sorting: `title`, `added`, `year`, `rating`, `lastPlayed`.
