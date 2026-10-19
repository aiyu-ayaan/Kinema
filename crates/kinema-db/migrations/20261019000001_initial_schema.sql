-- Initial Kinema Schema

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
  poster TEXT,                     -- image cache key
  backdrop TEXT,
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
  probe_status TEXT NOT NULL DEFAULT 'pending'  -- pending|ok|failed
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
  is_default INTEGER NOT NULL DEFAULT 0
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

CREATE TABLE user_library_access (
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  library_id INTEGER NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
  PRIMARY KEY (user_id, library_id)
);

CREATE TABLE share_links (
  id INTEGER PRIMARY KEY,
  token_hash TEXT NOT NULL UNIQUE,
  created_by INTEGER NOT NULL REFERENCES users(id),
  target_kind TEXT NOT NULL,       -- item|playlist|library
  target_id INTEGER NOT NULL,
  expires_at INTEGER,
  max_uses INTEGER,
  uses INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE sessions (
  id TEXT PRIMARY KEY,             -- random 256-bit, stored hashed
  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
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
