-- Keyframe index per file version (media-pipeline.md 6a): seek landings and HLS remux segment cuts
-- without running ffprobe on the play path. Valid while `fingerprint` matches the file's.
CREATE TABLE keyframe_index (
  file_id INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
  fingerprint TEXT NOT NULL,
  source TEXT NOT NULL,                     -- cues | stss | packets | none (no cheap source; not retried)
  has_b_frames INTEGER NOT NULL DEFAULT 0,
  times BLOB NOT NULL                       -- little-endian u32 milliseconds, ascending
);
