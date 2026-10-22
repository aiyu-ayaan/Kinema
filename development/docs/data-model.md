# Data Model

SQLite, WAL mode. Timestamps are unix seconds (INTEGER). IDs are INTEGER primary keys exposed to clients as opaque strings.

## Tables

```sql
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  password_hash TEXT,              -- argon2id; NULL for PIN-only profile
  pin_hash TEXT,
  is_admin INTEGER NOT NULL DEFAULT 0,
  avatar TEXT,
  created_at INTEGER NOT NULL
);

CREATE TABLE libraries (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('movie','show','music','photo','other')),
  language TEXT NOT NULL DEFAULT 'en',
  scan_interval_min INTEGER
);

CREATE TABLE library_roots (
  library_id INTEGER NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  poll INTEGER NOT NULL DEFAULT 0,               -- 1: list the folder on a timer instead of watching it (migration 20261022500002)
  poll_interval_min INTEGER NOT NULL DEFAULT 5,  -- 1..1440
  PRIMARY KEY (library_id, path)
);

-- A logical thing a user browses: movie, show, season, episode, album, track, photo.
CREATE TABLE items (
  id INTEGER PRIMARY KEY,
  library_id INTEGER NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
  parent_id INTEGER REFERENCES items(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,              -- movie|show|season|episode|artist|album|track|photo|video
  title TEXT NOT NULL,
  sort_title TEXT NOT NULL,
  year INTEGER,
  season_no INTEGER,
  episode_no INTEGER,
  summary TEXT,
  runtime_s INTEGER,
  rating REAL,
  tmdb_id INTEGER,
  tvmaze_id INTEGER,               -- shows matched on TVmaze when TMDB had no key or no match
  poster TEXT,                     -- image cache key
  backdrop TEXT,
  locked_fields TEXT NOT NULL DEFAULT '',  -- comma separated: title,year,summary,rating,poster,backdrop,genres (migration 20261022500001)
  content_rating TEXT,             -- certification; '' = looked up, unrated (migration 20261022300001)
  content_rating_manual INTEGER NOT NULL DEFAULT 0,  -- set by an admin, TMDB never replaces it
  added_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE INDEX items_parent ON items(parent_id);
CREATE INDEX items_lib_added ON items(library_id, added_at DESC);

-- A physical file. One item can have several files (versions).
CREATE TABLE files (
  id INTEGER PRIMARY KEY,
  item_id INTEGER REFERENCES items(id) ON DELETE SET NULL,
  path TEXT NOT NULL UNIQUE,
  size INTEGER NOT NULL,
  mtime INTEGER NOT NULL,
  fingerprint TEXT NOT NULL,       -- size + head/tail hash
  container TEXT,
  duration_s REAL,
  bitrate INTEGER,
  probe_json TEXT,                 -- raw ffprobe output
  probe_status TEXT NOT NULL DEFAULT 'pending',  -- pending|ok|failed
  markers_mtime INTEGER            -- file version markers were last looked for (migration 7)
);

-- Intro and end credits per file, for Skip intro and the up-next card (media-pipeline.md §7).
CREATE TABLE markers (
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  kind TEXT NOT NULL CHECK (kind IN ('intro','credits')),
  start_s REAL NOT NULL,
  end_s REAL NOT NULL,
  source TEXT NOT NULL,            -- chapter|audio|black|text|manual; detection never replaces manual (migration 20261022400001 re-queues files without credits once for `text`)
  PRIMARY KEY (file_id, kind)
);

-- Keyframes of a file's first video stream (media-pipeline.md 6a), migration 20261022100001.
-- Valid while fingerprint matches files.fingerprint; a stale row is rebuilt by the keyframe job.
CREATE TABLE keyframe_index (
  file_id INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
  fingerprint TEXT NOT NULL,
  source TEXT NOT NULL,            -- cues|stss|packets|none (none: no cheap source, not retried)
  has_b_frames INTEGER NOT NULL DEFAULT 0,
  times BLOB NOT NULL              -- little-endian u32 milliseconds, ascending (14 KB for 2 h at 2 s GOPs)
);

CREATE TABLE streams (
  id INTEGER PRIMARY KEY,
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  idx INTEGER NOT NULL,
  type TEXT NOT NULL,              -- video|audio|subtitle
  codec TEXT,
  language TEXT,
  channels INTEGER,
  width INTEGER,
  height INTEGER,
  is_default INTEGER NOT NULL DEFAULT 0,
  title TEXT, profile TEXT,
  external_path TEXT,              -- sidecar subtitle file, NULL for embedded
  external_fingerprint TEXT,       -- sidecar "size-mtime"; a change re-syncs sidecars without a probe
  ocr INTEGER NOT NULL DEFAULT 0   -- picture subtitle read by OCR into the WebVTT cache
);

CREATE TABLE genres (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE);
CREATE TABLE item_genres (
  item_id INTEGER REFERENCES items(id) ON DELETE CASCADE,
  genre_id INTEGER REFERENCES genres(id) ON DELETE CASCADE,
  PRIMARY KEY (item_id, genre_id)
);

CREATE TABLE watch_state (
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  position_s REAL NOT NULL DEFAULT 0,
  watched INTEGER NOT NULL DEFAULT 0,
  play_count INTEGER NOT NULL DEFAULT 0,
  last_played_at INTEGER,
  PRIMARY KEY (user_id, item_id)
);
CREATE INDEX watch_recent ON watch_state(user_id, last_played_at DESC);

-- A profile's own audio / subtitle pick, saved by any player (web, Android) and
-- applied by /files/{id}/plan on every device. Kept on the movie, or on the show for
-- all its episodes. Positions among the streams; subtitle -1 is off.
CREATE TABLE track_choice (
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  audio INTEGER, subtitle INTEGER,
  PRIMARY KEY (user_id, item_id)
);

CREATE TABLE playlists (
  id INTEGER PRIMARY KEY,
  owner_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  is_shared INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL
);
CREATE TABLE playlist_items (
  playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
  item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  PRIMARY KEY (playlist_id, position)
);

-- Franchise groups (migration 20261022500003). tmdb_id is set for groups TMDB reported, NULL for manual ones.
CREATE TABLE collections (id INTEGER PRIMARY KEY, name TEXT NOT NULL, tmdb_id INTEGER UNIQUE, created_at INTEGER NOT NULL);
CREATE TABLE collection_items (
  collection_id INTEGER NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
  item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  PRIMARY KEY (collection_id, item_id)
);

-- Home shelves: titled genre queries. Admin defines them, a profile hides any for itself.
CREATE TABLE shelves (id INTEGER PRIMARY KEY, title TEXT NOT NULL, genre TEXT NOT NULL, position INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL);
CREATE TABLE shelf_hidden (
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  shelf_id INTEGER NOT NULL REFERENCES shelves(id) ON DELETE CASCADE,
  PRIMARY KEY (user_id, shelf_id)
);

CREATE TABLE user_library_access (
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  library_id INTEGER NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
  PRIMARY KEY (user_id, library_id)
);

CREATE TABLE share_links (
  id INTEGER PRIMARY KEY,
  token_hash TEXT NOT NULL UNIQUE, -- SHA-256 of the secret verifier
  created_by INTEGER NOT NULL REFERENCES users(id),  -- trigger deletes links with the user
  target_kind TEXT NOT NULL,       -- 'item' (movie/show/season/episode/video)
  target_id INTEGER NOT NULL,
  expires_at INTEGER,
  max_uses INTEGER,                -- unused for now
  uses INTEGER NOT NULL DEFAULT 0, -- unused for now
  selector TEXT UNIQUE,            -- public half of the token (migration 20261022300001)
  created_at INTEGER NOT NULL DEFAULT 0,
  revoked_at INTEGER,
  label TEXT
);

CREATE TABLE sessions (
  id TEXT PRIMARY KEY,             -- random 256-bit, stored hashed
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  lan_only INTEGER NOT NULL DEFAULT 0  -- opened without a lock at home: LAN only
);

-- Per-profile languages, content limit and LAN unlock (migration 20261022300001).
CREATE TABLE profile_prefs (
  user_id INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  audio_language TEXT, subtitle_language TEXT, subtitle_mode TEXT,  -- NULL follows the server
  max_rating INTEGER,              -- minimum-age scale 0..18, NULL = no limit
  allow_unrated INTEGER NOT NULL DEFAULT 0,
  lan_unlock INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE jobs (
  id INTEGER PRIMARY KEY,
  kind TEXT NOT NULL,
  key TEXT NOT NULL,               -- idempotency key (fingerprint + kind)
  payload TEXT,
  status TEXT NOT NULL DEFAULT 'queued',  -- queued|running|done|failed
  attempts INTEGER NOT NULL DEFAULT 0,
  priority INTEGER NOT NULL DEFAULT 0,
  UNIQUE (kind, key)
);

CREATE VIRTUAL TABLE items_fts USING fts5(title, summary, content='items', content_rowid='id');
```

## Notes

- `files.fingerprint` lets a moved or renamed file keep its item, metadata and watch state.
- Deleting a file sets `item_id` to NULL only if other files remain; otherwise the scanner removes the orphan item.
- Watch state is per user, never per item globally.
