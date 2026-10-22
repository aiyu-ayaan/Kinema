-- Profiles and household: per-profile languages and content limits, LAN-only unlocking,
-- certifications on titles, and scoped share links (users-and-sharing.md).

-- One row per profile; a missing row means "follow the server, no limits".
CREATE TABLE profile_prefs (
  user_id           INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  audio_language    TEXT,                       -- ISO 639-2; NULL follows the server
  subtitle_language TEXT,                       -- ISO 639-2; NULL follows the server
  subtitle_mode     TEXT,                       -- off|always|foreign; NULL follows the server
  max_rating        INTEGER,                    -- minimum-age scale 0..18; NULL = no limit
  allow_unrated     INTEGER NOT NULL DEFAULT 0, -- titles without a certification, under a limit
  lan_unlock        INTEGER NOT NULL DEFAULT 0  -- a PIN profile opens without it from the LAN
);

-- Certification ("PG-13", "TV-14") from TMDB, or set by an admin (manual = 1, never replaced).
ALTER TABLE items ADD COLUMN content_rating TEXT;
ALTER TABLE items ADD COLUMN content_rating_manual INTEGER NOT NULL DEFAULT 0;

-- 1 when the session was opened without a lock from the LAN: it only works from the LAN.
ALTER TABLE sessions ADD COLUMN lan_only INTEGER NOT NULL DEFAULT 0;

-- share_links (initial schema) was never used. Tokens are `selector` + secret verifier:
-- the selector finds the row, token_hash is SHA-256 of the verifier, compared in constant time.
ALTER TABLE share_links ADD COLUMN selector TEXT;
ALTER TABLE share_links ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0;
ALTER TABLE share_links ADD COLUMN revoked_at INTEGER;
ALTER TABLE share_links ADD COLUMN label TEXT;
CREATE UNIQUE INDEX share_links_selector ON share_links(selector);

-- share_links.created_by has no ON DELETE: drop a profile's links with it.
CREATE TRIGGER share_links_owner_deleted BEFORE DELETE ON users
BEGIN
  DELETE FROM share_links WHERE created_by = OLD.id;
END;
