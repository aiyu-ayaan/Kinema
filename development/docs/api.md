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
| GET | `/auth/profiles` | Public profile picker: names, avatars, lock kind, and for this device `opens_without_lock` / `home_only` |
| GET | `/profile` | `{user, preferences, options: {track_languages, subtitle_modes, rating_levels}}` |
| PUT | `/profile/preferences` | `{audio_language, subtitle_language, subtitle_mode}` (null follows the server, `""` file default); limits are not settable here |
| GET | `/profile/export` | Own export `{format: "kinema.profile", version: 1, profile, preferences, watch}`; never lock hashes |
| POST | `/profile/import` | Own import: watch history + languages; `{imported, unmatched, kept, lock_applied}` |
| GET | `/users/{id}/export?include_lock=` | Admin: export, hashes only with `include_lock=true` |
| POST | `/users/{id}/import?apply_lock=` | Admin: import incl. limits and LAN unlock; lock hashes only with `apply_lock=true` |
| PUT | `/items/{id}/content-rating` | Admin: `{content_rating: "PG-13" \| null}` manual certification (null returns to TMDB's) |
| GET/POST | `/shares` | Admin: list / create `{item_id, expires_in_hours: 1..2160, label?}` → `{share, token, path: "/s/<token>"}` (token shown once) |
| DELETE | `/shares/{id}` | Admin: revoke (204, 404 if not live) |
| GET | `/share/{token}`, `/share/{token}/items/{id}` | Public, scoped: `{share: {label, expires_at, target_id}, item, children, files: [{id, container, duration_s, streams}]}` |
| GET | `/share/{token}/files/{id}/plan`, `/share/{token}/stream/{id}`, `/share/{token}/subtitles/{stream_id}`, `/share/{token}/images/{size}/{name}` | Public, scoped playback; same query as `/files/{id}/plan` and `/stream/{id}` |
| GET/POST | `/users` | Admin: list / create profile `{name, avatar, is_admin, password?, pin?, library_ids}` |
| PATCH/DELETE | `/users/{id}` | Admin: edit (omitted lock keeps it, `""` removes it; optional `preferences: {audio_language, subtitle_language, subtitle_mode, max_rating, allow_unrated, lan_unlock}`, omitted keeps them) / delete. Responses carry `preferences` |
| GET/POST | `/libraries` | List / admin create `{name, kind: movie\|show\|other, paths}` |
| DELETE | `/libraries/{id}` | Admin: remove (files on disk untouched) |
| POST | `/libraries/{id}/scan` | Admin: rescan |
| GET | `/libraries/{id}/items?type=&sort=&offset=&limit=` | Top-level items with watch state. `type`: `movie` (everything but shows) or `show`; `sort`: `title` (default), `added`, `year`, `rating`. Without `limit`: the whole list as an array (older apps; use only for short lists such as `type=show`). With `limit` (1 to 500): one page `{items, total, next}`; `next` is the offset of the following page (null at the end), `total` is null while a rating limit hides an unknown share. Unknown `type`/`sort`: 400 |
| GET | `/fs?path=` | Admin: folders on the server, for the folder picker |
| GET | `/home` | `{continue_watching, shelves: [{library, kind: movies\|shows\|episodes, items}], playlists, collections, genre_shelves: [{id, title, genre, items}]}`; the last three follow library access, and genre shelves skip the ones this profile hid |
| GET | `/search?q=` | Title search |
| GET | `/items/{id}` | Item, `features` (`{scrub_thumbnails}`: what the server serves now, see `boot-checks.md`), genres, files with streams, children, prev/next sibling; show/season `next_up`; show `tracks: {audio, subtitle, streams}` (first episode's streams) |
| PUT | `/items/{id}/progress` | `{position_s, watched?}` |
| PUT | `/items/{id}/watched` | `{watched}`, for the item and every episode under it |
| PUT | `/items/{id}/track-choice` | `{audio, subtitle}` positions (subtitle `-1` off): this profile's pick, saved by every player and kept on the movie or the show; the plan prefers it over the show's admin choice and the Languages settings |
| PUT / DELETE | `/items/{id}/markers/{intro\|credits}` | Admin: `{start_s, end_s}` sets a marker on the title's main file that detection never replaces; DELETE drops it and looks for it again. Markers come with each file in `GET /items/{id}` as `markers: [{kind, start_s, end_s, source}]` (`source`: `chapter`, `audio`, `black`, `text`, `manual`). The web player's admin panel (Skip markers) uses these two calls |
| PATCH | `/items/{id}/metadata` | Admin: `{title?, year?, summary?, rating?}` saves the given fields and locks them (empty summary, year 0 or rating 0 clears); returns `{locked_fields}` |
| PUT | `/items/{id}/locks` | Admin: `{fields}` sets exactly which of `title, year, summary, rating, poster, backdrop, genres` are locked against rescans, refreshes and Fix match; `GET /items/{id}` returns `locked_fields` and `collections` |
| PUT | `/libraries/{id}/roots` | Admin: `{roots: [{path, poll, poll_interval_min}]}` sets each folder's scan mode (events, or polling every 1 to 1440 min); `GET /libraries` returns `root_options` next to `roots` |
| GET/POST | `/playlists` | Own and shared playlists `{id, name, owner_id, owner_name, is_shared, mine, item_count, poster}` (counts only items the profile may see) / create `{name, item_ids?}` |
| GET/PATCH/DELETE | `/playlists/{id}` | Detail with `items` / owner only: `{name?, is_shared?}` / owner only. 404 when not visible, 403 `playlists.not_owner` when visible but not the owner's |
| PUT/POST | `/playlists/{id}/items` | Owner only: `{item_ids}` replaces the whole order / appends, skipping duplicates. Movies, episodes and videos the owner can access only |
| GET/POST | `/collections` | Collections with at least one visible title (admins see all) / admin create `{name, item_ids?}` |
| GET/PATCH/DELETE | `/collections/{id}` | Detail with `items` / admin rename `{name}` / admin delete |
| PUT | `/collections/{id}/items` | Admin: `{item_ids}` replaces the members (movies, shows, videos) |
| GET/POST | `/shelves` | All shelves with this profile's `hidden` flag / admin create `{title, genre}` |
| PATCH/DELETE | `/shelves/{id}` | Admin: `{title?, genre?}` / delete |
| PUT | `/shelves/{id}/hidden` | `{hidden}`: hide or show a shelf for this profile only |
| GET | `/genres` | Admin: genre names, for the shelf form |
| PUT | `/items/{id}/tracks` | Admin, show only: `{audio, subtitle}` as positions among an episode's audio / subtitle streams; `null` follows the Languages settings, subtitle `-1` is off |
| GET | `/files/{id}/plan?audio=&start=&hevc=&av1=&raw=` | `{mode: direct\|remux\|transcode, duration_s, audio, start, subtitle}`; audio and subtitle follow the show's track choice, then the Languages settings. `raw=true` always plans direct (a player that decodes anything, mpv). For a remux, `start` comes from the stored keyframe index when there is one (ffprobe otherwise) |
| GET | `/stream/{id}?audio=&start=&hevc=&av1=&raw=` | Direct (Range) or fMP4 pipe, per the plan; `raw=true` always serves the file as it is |
| GET | `/hls/{id}/index.m3u8?audio=&hevc=&av1=` | VOD HLS playlist for the plan's remux or transcode (a direct-play file is remuxed). Segment URIs repeat the chosen audio. 409 `media.hls_unavailable` when the file is unprobed, or for copied video without a keyframe index yet (`media-pipeline.md` 6a) |
| GET | `/hls/{id}/{n}.ts?audio=&hevc=&av1=` | MPEG-TS segment `n`: from disk when already cut, else waits for the live run or restarts ffmpeg at `n`; session kept 60 s idle. 503 `media.transcode_busy` past the transcode limit |
| GET | `/subtitles/{stream_id}` | Text subtitle, or picture subtitle read by OCR (`ocr: true` on the stream), as WebVTT from the cache; a text track not cached yet is extracted once. 400 `media.subtitle_unsupported` for a picture track without OCR, `media.subtitle_failed` when conversion failed |
| GET | `/images/{size}/{name}` | TMDB art, cached |
| GET | `/items/{id}/thumb` | Video frame JPEG from the background thumbnail cache; 404 until made |
| POST | `/libraries/{id}/thumbnails` | Admin: delete and regenerate a library's thumbnails |
| POST | `/libraries/{id}/match` | Admin: look up unmatched titles again |
| GET | `/server/info` | Public: `{name, version, public_url, tls_fingerprint, capabilities}`; apps read `public_url` to stream away from home and `tls_fingerprint` to recognise the same server on the LAN |
| GET | `/`, `/index.html`, any client route | The web UI page with `__KINEMA_SERVER_NAME__` replaced by the HTML-escaped server name (`backend/src/web.rs`), so tab titles, bookmarks and link previews show it before scripts run |
| GET | `/server/status` | Admin: dashboard data; per library `scanning`, `generating_thumbnails`, `extracting_subtitles` |
| POST | `/server/cache/{thumbnails\|subtitles\|images}/clear` | Admin: clear a cache |
| GET | `/items/{id}/match?q=&year=` | Admin: TMDB candidates for a movie or show |
| POST | `/items/{id}/match` | Admin: `{tmdb_id}`, replace the item's metadata with that result |
| GET | `/settings` | Admin: `{server, tmdb_configured, ffmpeg_found, hardware_encoder, ocr_languages, options}`; `ocr_languages` is null without `tesseract`. `server.subtitle_ocr` turns picture-subtitle OCR on |
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

### Devices, activity, requests, notifications (2026-10-28)

`POST /auth/login` and `/auth/setup` accept `device: {id, kind: web|desktop|mobile, name}`. The response is either `{token, user}` or `{approval_required: {poll_token, device_name}}` (see users-and-sharing.md). Errors: `429 auth.device_requests_full`.

| Method | Path | Who | Purpose |
|---|---|---|---|
| POST | `/auth/device/poll` | public | `{poll_token}`: `{pending:true}`, or the session once approved, or 404 (denied / used) |
| GET | `/devices` | admin | Devices, waiting first: `{id, user_id, user_name, kind, name, status, created_at, last_seen}` |
| POST | `/devices/{id}/approve` | admin | Approve a waiting device; returns it |
| DELETE | `/devices/{id}` | admin | Deny or remove; ends its sessions. 400 `devices.self_last` for the admin's own last device |
| GET | `/activity/now` | admin | Live playback: user, item card, `position_s`, `device_kind`, `device_name`, `started_at`, `idle_s` |
| GET | `/activity/users` | admin | Every profile with its totals |
| GET | `/activity/users/{id}` | admin | Stats of one profile |
| GET | `/profile/stats` | any | Own stats: `totals{total_s,last_7d_s,last_30d_s,titles,completed,last_played_at}`, `daily[{day,seconds}]`, `top_titles`, `top_genres`, `history` |
| GET | `/requests` | any | Own requests; all for the admin |
| GET | `/requests/search?q&kind` | any | `{tmdb, results[]}`; `tmdb:false` without a TMDB key |
| POST | `/requests` | any | Create (201). 400 `requests.kind_invalid` / `requests.title_required`; 409 `requests.in_library` / `requests.duplicate` |
| PATCH | `/requests/{id}` | admin | `{status: pending|approved|declined|available, admin_note?}` |
| DELETE | `/requests/{id}` | owner (pending) / admin | Withdraw or delete; 403 `requests.not_yours` |
| GET | `/settings/notifications` | admin | Channels with secrets masked, plus `on_device_request`, `on_media_request` |
| PUT | `/settings/notifications` | admin | Replace; `••••••••` keeps the stored secret; unknown kinds are dropped; max 20 channels |
| POST | `/settings/notifications/test/{index}` | admin | Test one saved channel: 204, 400 `notify.channel_missing`, 502 `notify.send_failed` |

`PUT /items/{id}/progress` now also feeds watch time and now playing, and refreshes the device's `last_seen`.

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
| GET | `/items/{id}/timeline.vtt` | Implemented. Scrub-thumbnail index of the title's main file (`sprites/{n}.jpg#xywh=x,y,w,h` cues, relative to the VTT). 202 + `Retry-After: 15` while the sheets are being made (the first request starts them), 404 when none can be made or `server.scrub_thumbnails` is off (default) |
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
