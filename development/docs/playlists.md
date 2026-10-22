# Playlists and Collections

Everything here follows library access (`access.rs`): a profile only ever sees titles from libraries it may open. Counts, covers and lists are computed after that filter, and a playlist or collection with nothing visible is hidden from that profile.

## Playlists

- Ordered list of playable items (movie, episode, video), owned by one profile. Music tracks arrive with the music library.
- Operations: create (optionally with a first item), rename, add (append, duplicates skipped), remove, reorder (the web page has up, down and remove buttons; `PUT /api/playlists/{id}/items` sets the full order), delete. Shuffle at play time is not built yet.
- Add from the item page: "Add to playlist" lists the profile's own playlists and can start a new one.
- Play: "Play all" opens the first item and remembers the playlist for the tab (`lib/activePlaylist.ts`, sessionStorage). The player then uses the next playlist entry as "up next" instead of the next episode; playing something outside the list leaves playlist mode.
- Permissions: only the owner changes or deletes a playlist (403 `playlists.not_owner` for a profile that can see but not edit it; 404 when it cannot see it). Admins get no extra rights over other profiles' playlists.
- Sharing: `is_shared=1` makes it visible read-only to all profiles; each viewer sees only the items from libraries they can open. Adding or replacing items checks that the owner can access each one (404 otherwise) and that it is playable (400 `playlists.item_unplayable`). Limit 5000 items.
- Share link: see `users-and-sharing.md` (not built).
- Smart playlists (phase 2): saved filters such as "unwatched horror from the 2010s", evaluated at read time.

## Collections

- Automatic: when a movie is matched on TMDB, its `belongs_to_collection` creates (or joins) a collection with `tmdb_id` set. Fix match files the movie again: TMDB-made memberships are dropped and re-created from the new match; manual ones stay.
- Manual: an admin groups movies and shows into a named collection ("Add to collection" on the item page), renames or deletes it from its page. Removing a single title from a collection is API-only (`PUT /api/collections/{id}/items`).
- A collection appears as one cover on Home (`collections` row) and opens a grid of its titles, oldest first.

## Home shelves

A shelf is a saved query with a title: genre plus newest first. "Oh, The Horror!" (genre Horror) is created by migration `20261022500003`. Admins add, edit and remove shelves in Settings, Home shelves; each profile can hide any shelf for itself from Home and bring it back from the "Hidden shelves" line. Empty shelves are not shown. Items are limited to the profile's libraries in SQL.

## Locked fields

Admins edit a title's details on its page (Edit details). Every field saved this way is locked, and the lock icons next to each field (title, year, summary, rating, poster, backdrop, genres) lock or unlock fields on their own. Locked fields are skipped by scans, TMDB refreshes and Fix match. See `metadata-and-artwork.md`.

## Storage

Tables `playlists`, `playlist_items`, `collections`, `collection_items`, `shelves` and `shelf_hidden` in `data-model.md`. Reordering rewrites positions in one transaction.
