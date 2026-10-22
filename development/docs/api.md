# HTTP API

Base path `/api`, over HTTPS (self-signed by default, see `security.md`). JSON bodies. Auth: `Authorization: Bearer <session token>` or `kinema_session` cookie. Errors: `{"error": {"code": "...", "message": "..."}}` with the matching HTTP status.

This is Kinema's own API. It is not Plex-compatible.

## Implemented in the MVP

| Method | Path | Purpose |
|---|---|---|
| GET | `/auth/status` | `{needs_setup}`: true until the first account exists |
| POST | `/auth/setup` | `{name, password}`: create the administrator (first run only), returns `{token, user}` and sets the cookie |
| POST | `/auth/login` | `{user_id \| name, password? \| pin?}` returns `{token, user}` and sets `kinema_session`; 429 after 5 failures |
| POST | `/auth/logout` | Invalidate session, clear cookie |
| GET | `/auth/me` | Current user |
| GET | `/auth/profiles` | Public profile picker: names, avatars, lock kind |
| GET/POST | `/users` | Admin: list / create profile `{name, avatar, is_admin, password?, pin?, library_ids}` |
| PATCH/DELETE | `/users/{id}` | Admin: edit (omitted lock keeps it, `""` removes it) / delete |
| GET/POST | `/libraries` | List / admin create `{name, kind: movie\|show\|other, paths}` |
| DELETE | `/libraries/{id}` | Admin: remove (files on disk untouched) |
| POST | `/libraries/{id}/scan` | Admin: rescan |
| GET | `/libraries/{id}/items` | Top-level items with watch state |
| GET | `/fs?path=` | Admin: folders on the server, for the folder picker |
| GET | `/home` | `{continue_watching, shelves: [{library, kind: movies\|shows\|episodes, items}]}` |
| GET | `/search?q=` | Title search |
| GET | `/items/{id}` | Item, genres, files with streams, children, prev/next sibling; show/season `next_up`; show `tracks: {audio, subtitle, streams}` (first episode's streams) |
| PUT | `/items/{id}/progress` | `{position_s, watched?}` |
| PUT | `/items/{id}/watched` | `{watched}`, for the item and every episode under it |
| PUT | `/items/{id}/track-choice` | `{audio, subtitle}` positions (subtitle `-1` off): this profile's pick, saved by every player and kept on the movie or the show; the plan prefers it over the show's admin choice and the Languages settings |
| PUT / DELETE | `/items/{id}/markers/{intro\|credits}` | Admin: `{start_s, end_s}` sets a marker on the title's main file that detection never replaces; DELETE drops it and looks for it again. Markers come with each file in `GET /items/{id}` as `markers: [{kind, start_s, end_s, source}]` |
| PUT | `/items/{id}/tracks` | Admin, show only: `{audio, subtitle}` as positions among an episode's audio / subtitle streams; `null` follows the Languages settings, subtitle `-1` is off |
| GET | `/files/{id}/plan?audio=&start=&hevc=&av1=&raw=` | `{mode: direct\|remux\|transcode, duration_s, audio, start, subtitle}`; audio and subtitle follow the show's track choice, then the Languages settings. `raw=true` always plans direct (a player that decodes anything, mpv) |
| GET | `/stream/{id}?audio=&start=&hevc=&av1=&raw=` | Direct (Range) or fMP4 pipe, per the plan; `raw=true` always serves the file as it is |
| GET | `/subtitles/{stream_id}` | Text subtitle as WebVTT, cached |
| GET | `/images/{size}/{name}` | TMDB art, cached |
| GET | `/items/{id}/thumb` | Video frame JPEG from the background thumbnail cache; 404 until made |
| POST | `/libraries/{id}/thumbnails` | Admin: delete and regenerate a library's thumbnails |
| POST | `/libraries/{id}/match` | Admin: look up unmatched titles again |
| GET | `/server/info` | Public: `{name, version, public_url, tls_fingerprint, capabilities}`; apps read `public_url` to stream away from home and `tls_fingerprint` to recognise the same server on the LAN |
| GET | `/`, `/index.html`, any client route | The web UI page with `__KINEMA_SERVER_NAME__` replaced by the HTML-escaped server name (`backend/src/web.rs`), so tab titles, bookmarks and link previews show it before scripts run |
| GET | `/server/status` | Admin: dashboard data |
| POST | `/server/cache/{thumbnails\|subtitles\|images}/clear` | Admin: clear a cache |
| GET | `/items/{id}/match?q=&year=` | Admin: TMDB candidates for a movie or show |
| POST | `/items/{id}/match` | Admin: `{tmdb_id}`, replace the item's metadata with that result |
| GET | `/settings` | Admin: `{server, tmdb_configured, ffmpeg_found, options}` |
| PUT | `/settings/server` | Admin: save server options (clamped to allowed values) |
| PUT | `/settings/tmdb` | Admin: `{tmdb_api_key}`, verified before saving |

### LAN discovery

Not HTTP: apps broadcast the bytes `KINEMA_DISCOVER` to UDP port 48555. Each server answers the sender with `{"service":"kinema","name","port","version"}` (backend `discovery.rs`). A server bound to loopback does not answer.

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
| GET | `/items/{id}/timeline.vtt` | Implemented. Scrub-thumbnail index of the title's main file (`sprites/{n}.jpg#xywh=x,y,w,h` cues, relative to the VTT). 202 + `Retry-After: 15` while the sheets are being made (the first request starts them), 404 when none can be made |
| GET | `/items/{id}/sprites/{n}` | Implemented. Scrub sprite sheet `n` (JPEG, 10x10 tiles of 160x90); same 202/404 rules |

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
