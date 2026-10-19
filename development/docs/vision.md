# Vision

## Goal

A single static Rust binary that turns a folder of media into a browsable, streamable library with posters, metadata, playlists and multiple user profiles, using as little CPU and RAM as possible.

## Goals

- Index movies, TV series, music, photos and home videos from local or mounted folders.
- Stream to browsers and apps over LAN and the internet, with direct play wherever possible.
- Fetch posters, backdrops, descriptions, cast and episode info automatically.
- Generate thumbnails for every video, including files with no online match.
- Multiple user profiles with watch state, "continue watching", and per-profile library access.
- Playlists and collections.
- Share a library or a single item with other people through scoped links or invited accounts.
- Handle any file format: if ffprobe can read it, play it; if not, still list it and allow download.
- Idle CPU near 0%. Scanning and thumbnailing must not disturb playback.

- Run on all Linux flavors, Windows and Docker from one codebase, with ffmpeg bundled (see `packaging.md`).

## Non-goals

- Live TV channels, ad-supported streaming, or any paid online catalog.
- DRM-protected content.
- Plex cloud account compatibility. Kinema has its own accounts and API.
- Linking to libav. Kinema shells out to ffmpeg.

## Target users

A person or household running a home server, a NAS or a small VPS, who wants a lightweight alternative to Plex or Jellyfin.

## Success measures

- Idle: under 30 MB RSS and under 0.1% CPU with a 10,000 item library.
- Direct play of a 4K file uses under 2% of one core on the server.
- First scan of 10,000 files finishes without saturating CPU (bounded workers, low nice value).
- Cold start under 1 second.
