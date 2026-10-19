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

A TMDB API key is configured by the user in `kinema.toml` (`tmdb_api_key`). Without a key, movies are not matched and only local fallbacks run. Kinema cannot ship its own key: TMDB's free key is for personal use only.

## Matching (TVmaze fallback for TV)

TVmaze (`kinema-meta::tvmaze`) needs no key, so shows still get metadata without TMDB. It covers TV only, in English. The data is CC BY-SA.

```
show without metadata
  -> TMDB key set?  yes -> TMDB /search/tv -> match? yes -> tmdb_id, TMDB seasons
                                                     no  -+
                    no  --------------------------------- +-> TVmaze /search/shows (prefers a hit that premiered in the file's year)
                                                              -> /shows/{id}?embed[]=seasons,episodes,images,cast (one call)
                                                              -> tvmaze_id, posters, episodes
```

- TMDB always wins. A show is only matched on TVmaze when TMDB has no key or finds nothing. Adding a key later resets `meta_tried` for rows without a `tmdb_id`, so TVmaze matches are upgraded to TMDB on the next scan.
- "Fix match" still uses TMDB only. A manual match clears `tvmaze_id`.
- Rate limit is about 20 calls per 10 s. On HTTP 429 the client waits 10 s and retries, up to 3 times.
- Summaries are HTML. Tags are stripped before saving.
- Image keys are `tvmaze/<variant>-<dir>-<file>`, for example `tvmaze/original_untouched-501-1253519.jpg`. The image route maps them back to `static.tvmaze.com/uploads/images/<variant>/<dir>/<file>`. Only the `original_untouched`, `medium_portrait` and `medium_landscape` variants are allowed.

## Fields stored

Title, original title, year, summary, runtime, rating, genres, cast and crew (top 20 cast plus key directors/creators in `cast_json`), content rating, poster, backdrop, logo, episode still, air date. Episodes and seasons inherit cast from their parent show.

## Artwork

- Download once, original size kept only for backdrop. Posters resized to 200/400/800 px WebP on demand and cached by `hash(source_url, width)`. Cast profile photos use TMDB `w185` and are cached on disk.
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
