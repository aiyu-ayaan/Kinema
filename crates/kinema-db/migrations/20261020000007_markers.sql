-- Where a file's intro and end credits are, so players can offer Skip intro and bring up the next episode when
-- the credits start. Found in the background (chapters, audio a season's episodes share, a black run to the end)
-- or set by an admin; an admin's marker is never replaced by detection.
CREATE TABLE markers (
  file_id  INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  kind     TEXT NOT NULL CHECK (kind IN ('intro', 'credits')),
  start_s  REAL NOT NULL,
  end_s    REAL NOT NULL,
  source   TEXT NOT NULL,  -- chapter | audio | black | manual
  PRIMARY KEY (file_id, kind)
);

-- The file version (mtime) markers were last looked for; a changed file is looked at again.
ALTER TABLE files ADD COLUMN markers_mtime INTEGER;
