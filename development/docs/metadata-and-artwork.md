# Metadata and Artwork

## Filename parsing

Written in-house, table tested. Handles:

- Movies: `Title (2023).mkv`, `Title.2023.1080p.BluRay.x264-GRP.mkv`, `Title [2023]`.
- Shows: `Show/Season 01/Show - S01E02 - Name.mkv`, `Show.S01E02.mkv`, `1x02`, absolute numbering for anime (`Show - 045`).
- Multi-episode: `S01E01-E02`.
- Noise tokens stripped: resolution, source, codec, release group, language tags, `REPACK`, `PROPER`.
- Folder names are used when the filename is weak.

Output: `{kind, title, year?, season?, episode?, confidence}`.

## Matching (TMDB)

1. Search TMDB with title and year. For TV, use `/search/tv`, then `/tv/{id}/season/{n}` for episodes.
2. Score candidates: normalized title similarity, year match, runtime within 10% of probe duration.
3. Auto-accept at score >= 0.85. Below that, mark `unmatched` and show a "Fix match" UI.
4. Store `tmdb_id` so later refreshes are exact lookups.

Rate limit: token bucket at 30 req/s max, with retry and backoff. Responses cached in SQLite for 7 days.

A TMDB API key is configured by the user in `kinema.toml` (`tmdb_api_key`). Without a key, matching is skipped and only local fallbacks run.

## Fields stored

Title, original title, year, summary, runtime, rating, genres, cast and crew (top 15), content rating, poster, backdrop, logo, episode still, air date.

## Artwork

- Download once, original size kept only for backdrop. Posters resized to 200/400/800 px WebP on demand and cached by `hash(source_url, width)`.
- Local art wins when present: `poster.jpg`, `folder.jpg`, `cover.jpg`, `<name>-poster.jpg`, `fanart.jpg`.
- Image resizing uses the `image` crate in `spawn_blocking`, or `ffmpeg` for exotic formats.

## Local fallback (offline or no key)

1. Title from filename parse.
2. Poster from embedded cover art if present, else frame grab (see `media-pipeline.md`).
3. Summary and runtime from container tags (`comment`, `description`) and ffprobe duration.

## Local metadata sidecars

Read `.nfo` (Kodi style XML) and embedded tags when present, so existing libraries keep their metadata.

## Refresh

- New items: matched within seconds of probe.
- Existing: refresh on user request, or every 30 days for items with recent TMDB changes.
- Never overwrite fields the user edited manually (`locked_fields` column planned).
