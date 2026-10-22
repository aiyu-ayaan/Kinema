-- Fields a user edited by hand, comma separated (title, year, summary, rating, poster, backdrop, genres).
-- Rescans, TMDB refreshes and Fix match never overwrite a locked field.
ALTER TABLE items ADD COLUMN locked_fields TEXT NOT NULL DEFAULT '';
