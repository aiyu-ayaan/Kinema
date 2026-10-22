//! Personal playlists: an ordered list of playable items owned by one profile. The owner edits;
//! a shared playlist (`is_shared`) is read-only for everyone else. Which items a viewer may see
//! is decided by the caller (library access), not here.

use crate::items::{CARD_SELECT, ItemCard};
use crate::{DbError, now};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Playlist {
    pub id: i64,
    pub owner_id: i64,
    pub owner_name: String,
    pub name: String,
    pub is_shared: bool,
    pub created_at: i64,
}

impl Playlist {
    /// Only the owner changes a playlist, whatever the share flag says.
    pub fn editable_by(&self, user_id: i64) -> bool {
        self.owner_id == user_id
    }

    pub fn visible_to(&self, user_id: i64) -> bool {
        self.is_shared || self.owner_id == user_id
    }
}

const SELECT: &str =
    "SELECT p.id, p.owner_id, u.name AS owner_name, p.name, p.is_shared, p.created_at
   FROM playlists p JOIN users u ON u.id = p.owner_id";

/// The profile's own playlists first, then other profiles' shared ones.
pub async fn list_visible(pool: &SqlitePool, user_id: i64) -> Result<Vec<Playlist>, DbError> {
    Ok(sqlx::query_as(&format!(
        "{SELECT} WHERE p.owner_id = ?1 OR p.is_shared = 1
         ORDER BY p.owner_id != ?1, p.name COLLATE NOCASE"
    ))
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<Playlist>, DbError> {
    Ok(sqlx::query_as(&format!("{SELECT} WHERE p.id = ?1"))
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn create(pool: &SqlitePool, owner_id: i64, name: &str) -> Result<i64, DbError> {
    Ok(
        sqlx::query("INSERT INTO playlists (owner_id, name, created_at) VALUES (?1, ?2, ?3)")
            .bind(owner_id)
            .bind(name)
            .bind(now())
            .execute(pool)
            .await?
            .last_insert_rowid(),
    )
}

/// Omitted fields stay as they are.
pub async fn update(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    is_shared: Option<bool>,
) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE playlists SET name = COALESCE(?2, name), is_shared = COALESCE(?3, is_shared) WHERE id = ?1",
    )
    .bind(id)
    .bind(name)
    .bind(is_shared)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    sqlx::query("DELETE FROM playlists WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Item ids in play order.
pub async fn item_ids(pool: &SqlitePool, id: i64) -> Result<Vec<i64>, DbError> {
    Ok(sqlx::query_scalar(
        "SELECT item_id FROM playlist_items WHERE playlist_id = ?1 ORDER BY position",
    )
    .bind(id)
    .fetch_all(pool)
    .await?)
}

/// Cards in play order, with the viewer's watch state.
pub async fn items(pool: &SqlitePool, viewer_id: i64, id: i64) -> Result<Vec<ItemCard>, DbError> {
    let sql = format!(
        "{CARD_SELECT} JOIN playlist_items pi ON pi.item_id = i.id WHERE pi.playlist_id = ?2
         ORDER BY pi.position"
    );
    Ok(sqlx::query_as(&sql)
        .bind(viewer_id)
        .bind(id)
        .fetch_all(pool)
        .await?)
}

/// Replaces the whole order in one transaction (drag and drop, removal).
pub async fn set_items(pool: &SqlitePool, id: i64, item_ids: &[i64]) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM playlist_items WHERE playlist_id = ?1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for (pos, item) in item_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO playlist_items (playlist_id, item_id, position) VALUES (?1, ?2, ?3)",
        )
        .bind(id)
        .bind(item)
        .bind(pos as i64)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Adds to the end, skipping items already in the playlist.
pub async fn append(pool: &SqlitePool, id: i64, item_ids: &[i64]) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    for item in item_ids {
        sqlx::query(
            "INSERT INTO playlist_items (playlist_id, item_id, position)
             SELECT ?1, ?2, COALESCE(MAX(position) + 1, 0) FROM playlist_items WHERE playlist_id = ?1
             HAVING NOT EXISTS (SELECT 1 FROM playlist_items WHERE playlist_id = ?1 AND item_id = ?2)",
        )
        .bind(id)
        .bind(item)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
