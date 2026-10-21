---
id: backup-and-maintenance
title: Backup & Maintenance
sidebar_position: 3
---

# Backup & Maintenance

Because Kinema stores all server state in a single data directory (`$KINEMA_DATA_DIR`), backups, migrations, and disaster recovery are straightforward.

---

## Directory Structure

```
$KINEMA_DATA_DIR/
├── kinema.db            # Primary SQLite database file
├── kinema.db-wal        # SQLite Write-Ahead Log
├── kinema.db-shm        # Shared memory file
├── tls/                 # Auto-generated self-signed TLS certificates
│   ├── cert.pem
│   └── key.pem
└── cache/               # Ephemeral caches (safe to delete anytime)
    ├── images/          # Cached TMDB / TVmaze posters and backdrops
    ├── thumbs/          # Video keyframe JPEG thumbnails
    ├── subtitles/       # Extracted WebVTT subtitle files
    └── prints/          # Acoustic audio fingerprint bit vectors
```

---

## Safe Online Database Backup

Because SQLite operates in WAL mode, you can safely copy or backup `kinema.db` even while streams are actively playing:

### Using the SQLite CLI

```bash
sqlite3 data/kinema.db ".backup 'backup-$(date +%Y%m%d).db'"
```

This creates a clean, atomic snapshot without locking database writes.

---

## Cache Management & Purging

Cache directories (`images`, `thumbs`, `subtitles`, `prints`) are entirely ephemeral:
- If deleted, Kinema will automatically re-extract or re-download assets on demand.
- You can clear individual caches from the Admin Settings dashboard or via the REST API:
  ```bash
  curl -X POST -H "Authorization: Bearer <ADMIN_TOKEN>" \
    https://localhost:3000/api/server/cache/thumbnails/clear
  ```
