-- Sidecar subtitle version ("size-mtime"), so a changed or replaced .srt is noticed
-- by a scan without the video changing. NULL for embedded tracks.
ALTER TABLE streams ADD COLUMN external_fingerprint TEXT;
-- 1 when a picture subtitle (PGS, VobSub) was read by OCR into a cached WebVTT,
-- so players may offer it like a text track.
ALTER TABLE streams ADD COLUMN ocr INTEGER NOT NULL DEFAULT 0;
