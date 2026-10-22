-- The marker job learned to find credits over lit footage (rolling text over a still or slowly moving picture).
-- Files that were looked at before it and ended up without a credits marker are looked at once more; audio
-- fingerprints are cached, so the extra cost is one keyframe pass over each file's last minutes.
UPDATE files SET markers_mtime = NULL
WHERE markers_mtime IS NOT NULL
  AND id NOT IN (SELECT file_id FROM markers WHERE kind = 'credits');
