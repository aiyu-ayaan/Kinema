//! Share links: a read-only pass to one movie, show, season or episode. The token is a
//! public `selector` plus a secret verifier; only the verifier's SHA-256 is stored, and
//! the caller compares it in constant time after looking the row up by selector.

use crate::{DbError, now};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Share {
    pub id: i64,
    #[serde(skip)]
    pub token_hash: String,
    pub created_by: i64,
    pub creator_name: Option<String>,
    pub item_id: i64,
    pub item_title: Option<String>,
    pub item_kind: Option<String>,
    pub label: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
    pub revoked_at: Option<i64>,
}

const SELECT: &str = "SELECT s.id, s.token_hash, s.created_by, u.name AS creator_name,
  s.target_id AS item_id, i.title AS item_title, i.kind AS item_kind, s.label, s.created_at,
  COALESCE(s.expires_at, 0) AS expires_at, s.revoked_at
FROM share_links s
LEFT JOIN users u ON u.id = s.created_by
LEFT JOIN items i ON i.id = s.target_id";

pub struct NewShare<'a> {
    pub selector: &'a str,
    pub token_hash: &'a str,
    pub created_by: i64,
    pub item_id: i64,
    pub label: Option<&'a str>,
    pub expires_at: i64,
}

pub async fn create(pool: &SqlitePool, s: &NewShare<'_>) -> Result<i64, DbError> {
    Ok(sqlx::query(
        "INSERT INTO share_links (selector, token_hash, created_by, target_kind, target_id,
                                  expires_at, label, created_at)
         VALUES (?1, ?2, ?3, 'item', ?4, ?5, ?6, ?7)",
    )
    .bind(s.selector)
    .bind(s.token_hash)
    .bind(s.created_by)
    .bind(s.item_id)
    .bind(s.expires_at)
    .bind(s.label)
    .bind(now())
    .execute(pool)
    .await?
    .last_insert_rowid())
}

/// Live links first, newest first; expired and revoked ones stay listed for a while.
pub async fn list(pool: &SqlitePool) -> Result<Vec<Share>, DbError> {
    Ok(sqlx::query_as(&format!(
        "{SELECT} WHERE s.target_kind = 'item'
         ORDER BY (s.revoked_at IS NULL AND s.expires_at > ?1) DESC, s.created_at DESC"
    ))
    .bind(now())
    .fetch_all(pool)
    .await?)
}

pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<Share>, DbError> {
    Ok(sqlx::query_as(&format!("{SELECT} WHERE s.id = ?1"))
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

/// A live link by selector: not revoked, not expired, target still there.
pub async fn active_by_selector(
    pool: &SqlitePool,
    selector: &str,
) -> Result<Option<Share>, DbError> {
    Ok(sqlx::query_as(&format!(
        "{SELECT} WHERE s.selector = ?1 AND s.target_kind = 'item' AND s.revoked_at IS NULL
           AND s.expires_at > ?2 AND i.id IS NOT NULL"
    ))
    .bind(selector)
    .bind(now())
    .fetch_optional(pool)
    .await?)
}

/// Whether a link still works; checked again while a share streams.
pub async fn is_active(pool: &SqlitePool, id: i64) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM share_links
                       WHERE id = ?1 AND revoked_at IS NULL AND expires_at > ?2)",
    )
    .bind(id)
    .bind(now())
    .fetch_one(pool)
    .await?)
}

/// Whether an image key ("w500/abc.jpg") is the art of the target, of a season or episode
/// under it, or of the season / show above it (cards inherit art from their parents).
pub async fn image_in_scope(pool: &SqlitePool, target: i64, key: &str) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM items
          WHERE (id = ?1 OR parent_id = ?1
                 OR parent_id IN (SELECT id FROM items WHERE parent_id = ?1)
                 OR id = (SELECT parent_id FROM items WHERE id = ?1)
                 OR id = (SELECT p.parent_id FROM items c JOIN items p ON p.id = c.parent_id
                          WHERE c.id = ?1))
            AND (poster = ?2 OR backdrop = ?2))",
    )
    .bind(target)
    .bind(key)
    .fetch_one(pool)
    .await?)
}

/// False when there was no live link with this id.
pub async fn revoke(pool: &SqlitePool, id: i64) -> Result<bool, DbError> {
    let res =
        sqlx::query("UPDATE share_links SET revoked_at = ?2 WHERE id = ?1 AND revoked_at IS NULL")
            .bind(id)
            .bind(now())
            .execute(pool)
            .await?;
    Ok(res.rows_affected() == 1)
}
