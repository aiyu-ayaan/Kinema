//! Intro and end-credit markers per file (migration 7).

use crate::DbError;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow)]
pub struct Marker {
    /// intro | credits
    pub kind: String,
    pub start_s: f64,
    pub end_s: f64,
    /// chapter | audio | black | manual
    pub source: String,
}

pub async fn for_file(pool: &SqlitePool, file_id: i64) -> Result<Vec<Marker>, DbError> {
    Ok(sqlx::query_as(
        "SELECT kind, start_s, end_s, source FROM markers WHERE file_id = ?1 ORDER BY start_s",
    )
    .bind(file_id)
    .fetch_all(pool)
    .await?)
}

/// A file the marker job looks at: its episode's season groups it with the files it is compared to.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MarkerFile {
    pub id: i64,
    pub path: String,
    pub mtime: i64,
    pub duration_s: Option<f64>,
    pub kind: String,
    pub season_id: Option<i64>,
    pub episode_no: Option<i64>,
    pub markers_mtime: Option<i64>,
}

impl MarkerFile {
    /// Markers were looked for in this version of the file.
    pub fn done(&self) -> bool {
        self.markers_mtime == Some(self.mtime)
    }
}

/// Probed movie, episode and video files of a library, episodes in season order.
pub async fn library_files(pool: &SqlitePool, library_id: i64) -> Result<Vec<MarkerFile>, DbError> {
    Ok(sqlx::query_as(
        "SELECT f.id, f.path, f.mtime, f.duration_s, i.kind,
                CASE WHEN i.kind = 'episode' THEN i.parent_id END AS season_id, i.episode_no, f.markers_mtime
         FROM files f JOIN items i ON i.id = f.item_id
         WHERE i.library_id = ?1 AND f.probe_status = 'ok' AND i.kind IN ('movie', 'episode', 'video')
         ORDER BY i.parent_id, i.episode_no, f.id",
    )
    .bind(library_id)
    .fetch_all(pool)
    .await?)
}

/// Replaces a file's detected markers with `found` and records the version they belong to. An admin's markers stay.
pub async fn save_detected(
    pool: &SqlitePool,
    file_id: i64,
    mtime: i64,
    found: &[Marker],
) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM markers WHERE file_id = ?1 AND source != 'manual'")
        .bind(file_id)
        .execute(&mut *tx)
        .await?;
    for m in found {
        sqlx::query(
            "INSERT OR IGNORE INTO markers (file_id, kind, start_s, end_s, source) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(file_id)
        .bind(&m.kind)
        .bind(m.start_s)
        .bind(m.end_s)
        .bind(&m.source)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("UPDATE files SET markers_mtime = ?2 WHERE id = ?1")
        .bind(file_id)
        .bind(mtime)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// An admin's marker for `kind`, or None to drop it and let detection run again for the file.
pub async fn set_manual(
    pool: &SqlitePool,
    file_id: i64,
    kind: &str,
    span: Option<(f64, f64)>,
) -> Result<(), DbError> {
    match span {
        Some((start, end)) => {
            sqlx::query(
                "INSERT INTO markers (file_id, kind, start_s, end_s, source) VALUES (?1, ?2, ?3, ?4, 'manual')
                 ON CONFLICT (file_id, kind) DO UPDATE SET start_s = ?3, end_s = ?4, source = 'manual'",
            )
            .bind(file_id)
            .bind(kind)
            .bind(start)
            .bind(end)
            .execute(pool)
            .await?;
        }
        None => {
            sqlx::query("DELETE FROM markers WHERE file_id = ?1 AND kind = ?2")
                .bind(file_id)
                .bind(kind)
                .execute(pool)
                .await?;
            sqlx::query("UPDATE files SET markers_mtime = NULL WHERE id = ?1")
                .bind(file_id)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}
