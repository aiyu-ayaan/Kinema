-- A show's audio and subtitle choice for all its episodes, as positions among
-- the first episode's tracks: episodes of one release share the same layout.
ALTER TABLE items ADD COLUMN audio_track INTEGER;     -- NULL: server language settings
ALTER TABLE items ADD COLUMN subtitle_track INTEGER;  -- NULL: server settings, -1: off
