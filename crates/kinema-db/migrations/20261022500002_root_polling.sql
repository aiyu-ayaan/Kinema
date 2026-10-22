-- Network shares (NFS, SMB) send no filesystem events: a root can be polled instead.
ALTER TABLE library_roots ADD COLUMN poll INTEGER NOT NULL DEFAULT 0;
ALTER TABLE library_roots ADD COLUMN poll_interval_min INTEGER NOT NULL DEFAULT 5;
