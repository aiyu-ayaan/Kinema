# Playlists and Collections

## Playlists

- Ordered list of items, owned by one user.
- Can mix movies, episodes, tracks. Audio and video playlists play in order with autoplay.
- Operations: create, rename, add, remove, reorder (drag and drop sets the full order via `PUT /api/playlists/{id}/items`), shuffle at play time.
- Sharing: `is_shared=1` makes it visible read-only to all users with access to the underlying items. Items a viewer cannot access are hidden for that viewer.
- Share link: see `users-and-sharing.md`.
- Smart playlists (phase 2): saved filters such as "unwatched horror from the 2010s", evaluated at read time.

## Collections

- Automatic: TMDB `belongs_to_collection` groups franchise movies.
- Manual: admin groups items into a named collection.
- A collection appears as one poster that opens a grid of its items.

## Home shelves

A shelf is a saved query with a title, for example "Oh, The Horror!" = genre Horror, sorted by recently added. Admin can define shelves; each user can hide any.

## Storage

Tables `playlists` and `playlist_items` in `data-model.md`. Reordering rewrites positions in one transaction.
