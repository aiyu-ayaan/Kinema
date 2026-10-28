-- Device approval, watch-time analytics and title requests (users-and-sharing.md).

-- A client install a profile signed in from. `client_id` is random and kept by the client.
-- A non-admin profile gets one approved web/desktop device and one mobile device for free;
-- any other device waits for an administrator (status 'pending').
CREATE TABLE devices (
  id          INTEGER PRIMARY KEY,
  user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  client_id   TEXT NOT NULL,
  kind        TEXT NOT NULL CHECK (kind IN ('web','desktop','mobile')),
  name        TEXT NOT NULL,
  status      TEXT NOT NULL CHECK (status IN ('approved','pending')),
  -- SHA-256 of the secret a pending client polls with; NULL once approved.
  poll_hash   TEXT,
  created_at  INTEGER NOT NULL,
  last_seen   INTEGER NOT NULL,
  UNIQUE (user_id, client_id)
);

-- Removing a device signs it out.
ALTER TABLE sessions ADD COLUMN device_id INTEGER REFERENCES devices(id) ON DELETE CASCADE;

-- Seconds actually played, per profile, title and UTC day (unix seconds / 86400).
CREATE TABLE watch_time (
  user_id  INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  item_id  INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  day      INTEGER NOT NULL,
  seconds  INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (user_id, item_id, day)
);
CREATE INDEX watch_time_day ON watch_time(day);

-- A movie or show a profile asked the administrator to add.
CREATE TABLE media_requests (
  id          INTEGER PRIMARY KEY,
  user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  kind        TEXT NOT NULL CHECK (kind IN ('movie','show')),
  title       TEXT NOT NULL,
  year        INTEGER,
  tmdb_id     INTEGER,
  poster      TEXT,                 -- image key "w500/abc.jpg"
  overview    TEXT,
  note        TEXT,                 -- from the requester
  status      TEXT NOT NULL DEFAULT 'pending'
              CHECK (status IN ('pending','approved','declined','available')),
  admin_note  TEXT,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);
CREATE INDEX media_requests_status ON media_requests(status, created_at DESC);
