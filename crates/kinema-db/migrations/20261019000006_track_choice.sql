-- A profile's own audio and subtitle choice, made in any player (web, Android), so
-- every device starts the title the same way. Kept per movie, or per show for all
-- its episodes, as positions among the title's audio / subtitle streams (like
-- items.audio_track). It wins over the show's admin choice and the server settings.
CREATE TABLE track_choice (
  user_id  INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  item_id  INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  audio    INTEGER,  -- NULL: no choice
  subtitle INTEGER,  -- NULL: no choice, -1: off
  PRIMARY KEY (user_id, item_id)
);
