//! Keyframe index per file version (migration 20261022100001).

use crate::DbError;
use sqlx::SqlitePool;

/// A stored index: (has_b_frames, times blob). None when missing, stale, or built with no source.
pub async fn get(pool: &SqlitePool, file_id: i64) -> Result<Option<(bool, Vec<u8>)>, DbError> {
    Ok(sqlx::query_as(
        "SELECT k.has_b_frames, k.times FROM keyframe_index k
         JOIN files f ON f.id = k.file_id AND f.fingerprint = k.fingerprint
         WHERE k.file_id = ?1 AND k.source != 'none'",
    )
    .bind(file_id)
    .fetch_optional(pool)
    .await?)
}

/// Stores the index for the file's current version; `source` "none" records that there is none.
pub async fn save(
    pool: &SqlitePool,
    file_id: i64,
    source: &str,
    has_b_frames: bool,
    times: &[u8],
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO keyframe_index (file_id, fingerprint, source, has_b_frames, times)
         SELECT id, fingerprint, ?2, ?3, ?4 FROM files WHERE id = ?1
         ON CONFLICT(file_id) DO UPDATE SET fingerprint = excluded.fingerprint,
           source = excluded.source, has_b_frames = excluded.has_b_frames, times = excluded.times",
    )
    .bind(file_id)
    .bind(source)
    .bind(has_b_frames)
    .bind(times)
    .execute(pool)
    .await?;
    Ok(())
}

/// Probed files with video in a library whose index is missing or from an older version:
/// (id, path, size).
pub async fn missing(
    pool: &SqlitePool,
    library_id: i64,
) -> Result<Vec<(i64, String, i64)>, DbError> {
    Ok(sqlx::query_as(
        "SELECT f.id, f.path, f.size FROM files f
         JOIN items i ON i.id = f.item_id
         LEFT JOIN keyframe_index k ON k.file_id = f.id AND k.fingerprint = f.fingerprint
         WHERE i.library_id = ?1 AND f.probe_status = 'ok' AND k.file_id IS NULL
           AND EXISTS (SELECT 1 FROM streams s WHERE s.file_id = f.id AND s.type = 'video')
         ORDER BY i.added_at DESC",
    )
    .bind(library_id)
    .fetch_all(pool)
    .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn pool() -> SqlitePool {
        // A file, not :memory:, which would give each pooled connection its own empty database.
        let dir = std::env::temp_dir().join(format!("kinema-db-kf-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let url = format!("sqlite://{}", dir.join("t.db").display());
        let pool = crate::init_pool(&url).await.unwrap();
        crate::run_migrations(&pool).await.unwrap();
        sqlx::query("INSERT INTO libraries (id, name, kind) VALUES (1, 'L', 'movie')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO items (id, library_id, kind, title, sort_title, added_at, updated_at) VALUES (1, 1, 'movie', 'M', 'M', 0, 0)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO files (id, item_id, path, size, mtime, fingerprint, probe_status) VALUES (1, 1, '/a.mkv', 10, 5, '10-5', 'ok')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO streams (file_id, idx, type) VALUES (1, 0, 'video')")
            .execute(&pool)
            .await
            .unwrap();
        pool
    }

    #[tokio::test]
    async fn index_is_keyed_by_file_version() {
        let pool = pool().await;
        assert_eq!(missing(&pool, 1).await.unwrap().len(), 1);
        save(&pool, 1, "cues", true, &[1, 0, 0, 0]).await.unwrap();
        assert_eq!(get(&pool, 1).await.unwrap(), Some((true, vec![1, 0, 0, 0])));
        assert!(missing(&pool, 1).await.unwrap().is_empty());
        // The file changes: the index is stale until rebuilt.
        sqlx::query("UPDATE files SET fingerprint = '11-6' WHERE id = 1")
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(get(&pool, 1).await.unwrap(), None);
        assert_eq!(missing(&pool, 1).await.unwrap().len(), 1);
        // No cheap source: remembered, so the job does not retry, but nothing to serve.
        save(&pool, 1, "none", false, &[]).await.unwrap();
        assert!(missing(&pool, 1).await.unwrap().is_empty());
        assert_eq!(get(&pool, 1).await.unwrap(), None);
    }
}
