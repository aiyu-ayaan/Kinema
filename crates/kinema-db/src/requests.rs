//! Titles a profile asked the administrator to add.

use crate::{DbError, now};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct MediaRequest {
    pub id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub kind: String,
    pub title: String,
    pub year: Option<i64>,
    pub tmdb_id: Option<i64>,
    pub poster: Option<String>,
    pub overview: Option<String>,
    pub note: Option<String>,
    pub status: String,
    pub admin_note: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

const SELECT: &str = "SELECT r.id, r.user_id, u.name AS user_name, r.kind, r.title, r.year,
  r.tmdb_id, r.poster, r.overview, r.note, r.status, r.admin_note, r.created_at, r.updated_at
FROM media_requests r JOIN users u ON u.id = r.user_id";

pub const STATUSES: [&str; 4] = ["pending", "approved", "declined", "available"];

pub struct NewRequest<'a> {
    pub user_id: i64,
    pub kind: &'a str,
    pub title: &'a str,
    pub year: Option<i64>,
    pub tmdb_id: Option<i64>,
    pub poster: Option<&'a str>,
    pub overview: Option<&'a str>,
    pub note: Option<&'a str>,
}

pub async fn create(pool: &SqlitePool, r: &NewRequest<'_>) -> Result<i64, DbError> {
    let t = now();
    Ok(sqlx::query(
        "INSERT INTO media_requests
           (user_id, kind, title, year, tmdb_id, poster, overview, note, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
    )
    .bind(r.user_id)
    .bind(r.kind)
    .bind(r.title)
    .bind(r.year)
    .bind(r.tmdb_id)
    .bind(r.poster)
    .bind(r.overview)
    .bind(r.note)
    .bind(t)
    .execute(pool)
    .await?
    .last_insert_rowid())
}

pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<MediaRequest>, DbError> {
    Ok(sqlx::query_as(&format!("{SELECT} WHERE r.id = ?1"))
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

/// Every request (`user_id` None) or one profile's; open ones first, newest first.
pub async fn list(pool: &SqlitePool, user_id: Option<i64>) -> Result<Vec<MediaRequest>, DbError> {
    Ok(sqlx::query_as(&format!(
        "{SELECT} WHERE ?1 IS NULL OR r.user_id = ?1
         ORDER BY r.status = 'pending' DESC, r.created_at DESC"
    ))
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

/// An open request (pending or approved) for the same TMDB title, by anyone.
pub async fn open_duplicate(pool: &SqlitePool, kind: &str, tmdb_id: i64) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM media_requests
         WHERE kind = ?1 AND tmdb_id = ?2 AND status IN ('pending','approved')",
    )
    .bind(kind)
    .bind(tmdb_id)
    .fetch_one(pool)
    .await?
        > 0)
}

/// Whether a library already has the title.
pub async fn in_library(pool: &SqlitePool, kind: &str, tmdb_id: i64) -> Result<bool, DbError> {
    Ok(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM items WHERE kind = ?1 AND tmdb_id = ?2")
            .bind(kind)
            .bind(tmdb_id)
            .fetch_one(pool)
            .await?
            > 0,
    )
}

pub async fn set_status(
    pool: &SqlitePool,
    id: i64,
    status: &str,
    admin_note: Option<&str>,
) -> Result<bool, DbError> {
    Ok(sqlx::query(
        "UPDATE media_requests SET status = ?2, admin_note = ?3, updated_at = ?4 WHERE id = ?1",
    )
    .bind(id)
    .bind(status)
    .bind(admin_note)
    .bind(now())
    .execute(pool)
    .await?
    .rows_affected()
        == 1)
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    sqlx::query("DELETE FROM media_requests WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
