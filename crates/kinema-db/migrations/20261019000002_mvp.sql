-- MVP: server settings, metadata retry flag, richer stream info, external subtitles.

CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- 1 once a TMDB lookup ran (match or not), so scans do not repeat failed lookups.
ALTER TABLE items ADD COLUMN meta_tried INTEGER NOT NULL DEFAULT 0;

ALTER TABLE streams ADD COLUMN title TEXT;
ALTER TABLE streams ADD COLUMN profile TEXT;         -- e.g. "High 10": not browser playable
ALTER TABLE streams ADD COLUMN external_path TEXT;   -- sidecar subtitle file, NULL for embedded

CREATE INDEX files_item ON files(item_id);
CREATE INDEX streams_file ON streams(file_id);
CREATE INDEX items_lib_parent ON items(library_id, parent_id);
