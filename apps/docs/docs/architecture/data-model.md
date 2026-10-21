---
id: data-model
title: Data Model & SQLite
sidebar_position: 2
---

# Data Model & Storage Architecture

Kinema uses an embedded SQLite database configured in **WAL (Write-Ahead Logging)** mode. This design delivers millisecond query response times, zero external database daemon dependencies, and robust crash resilience.

---

## Entity-Relationship Diagram

The complete database schema across all 7 migrations is modeled below:

```mermaid
erDiagram
    users ||--o{ sessions : "has active"
    users ||--o{ watch_state : "tracks progress"
    users ||--o{ track_choice : "saves audio/subtitle"
    users ||--o{ user_library_access : "granted permissions"
    users ||--o{ playlists : "creates"

    libraries ||--|{ library_roots : "contains paths"
    libraries ||--o{ items : "organizes"
    libraries ||--o{ user_library_access : "restricts"

    items ||--o{ items : "parent/child (season/episode)"
    items ||--o{ files : "backed by"
    items ||--o{ item_genres : "tagged with"
    items ||--o{ watch_state : "playback history"
    items ||--o{ track_choice : "custom stream selection"
    items ||--o{ playlist_items : "collected in"

    genres ||--o{ item_genres : "categorizes"

    files ||--o{ streams : "contains tracks"
    files ||--o{ markers : "detected segments"

    playlists ||--o{ playlist_items : "orders"

    users {
        int id PK
        string name UK
        string password_hash
        string pin_hash
        int is_admin
        string avatar
        int created_at
    }

    libraries {
        int id PK
        string name
        string kind
        string language
        int scan_interval_min
    }

    items {
        int id PK
        int library_id FK
        int parent_id FK
        string kind
        string title
        string sort_title
        int year
        int season_no
        int episode_no
        string summary
        int runtime_s
        real rating
        int tmdb_id
        int tvmaze_id
        string poster
        string backdrop
        string cast_json
        int meta_tried
        int audio_track
        int subtitle_track
    }

    files {
        int id PK
        int item_id FK
        string path UK
        int size
        int mtime
        string fingerprint
        string container
        real duration_s
        int bitrate
        string probe_json
        string probe_status
        int markers_mtime
    }

    streams {
        int id PK
        int file_id FK
        int idx
        string type
        string codec
        string profile
        string language
        string title
        int channels
        int width
        int height
        int is_default
        string external_path
    }

    markers {
        int file_id PK,FK
        string kind PK
        real start_s
        real end_s
        string source
    }

    watch_state {
        int user_id PK,FK
        int item_id PK,FK
        real position_s
        int watched
        int play_count
        int last_played_at
    }

    track_choice {
        int user_id PK,FK
        int item_id PK,FK
        int audio
        int subtitle
    }

    sessions {
        string id PK
        int user_id FK
        int created_at
        int expires_at
    }
```

---

## Schema Migration History

Migrations are strictly append-only SQL files located in `crates/kinema-db/migrations/`:

| Migration | Version Tag | Description |
|---|---|---|
| `20261019000001` | Initial Schema | Creates `users`, `libraries`, `items`, `files`, `streams`, `genres`, `watch_state`, `sessions`, `jobs`, and `items_fts`. |
| `20261019000002` | MVP Updates | Adds `settings` key-value table, `items.meta_tried` flag, and stream metadata (`title`, `external_path`). |
| `20261019000003` | Cast Members | Adds `items.cast_json` for actor, director, and character billing. |
| `20261019000004` | TVmaze Support | Adds `items.tvmaze_id` column for keyless TV show metadata fallback. |
| `20261019000005` | Series Track Defaults | Adds `items.audio_track` and `items.subtitle_track` to series items for show-level defaults. |
| `20261019000006` | User Track Preferences | Adds `track_choice` table for per-user, per-show audio and subtitle stream stickiness. |
| `20261020000007` | Intro & Credit Markers | Adds `markers` table (`file_id`, `kind`, `start_s`, `end_s`, `source`) and `files.markers_mtime`. |

---

## Performance Optimizations

1. **WAL Mode**:
   ```sql
   PRAGMA journal_mode = WAL;
   PRAGMA synchronous = NORMAL;
   PRAGMA foreign_keys = ON;
   ```
   Readers never block writers, and writers never block readers.
2. **Compound Indexes**:
   - `CREATE INDEX idx_items_parent ON items(parent_id);` — Accelerates fetching season and episode lists.
   - `CREATE INDEX idx_watch_state_user ON watch_state(user_id, last_played_at DESC);` — Instant home shelf generation for "Continue Watching".
   - `CREATE INDEX idx_files_path ON files(path);` — Speeds up delta checks during directory rescans.
3. **Full-Text Search (FTS5)**:
   - Backed by an SQLite virtual table:
     ```sql
     CREATE VIRTUAL TABLE items_fts USING fts5(title, summary, content='items', content_rowid='id');
     ```
   - Automatic triggers sync inserts, updates, and deletes from `items` into `items_fts`.
