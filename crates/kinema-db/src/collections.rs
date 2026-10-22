//! Franchise collections (several movies as one poster) and the genre shelves on Home.
//! Collections come from TMDB (`tmdb_id` set) or are made by an admin. Which items a viewer may
//! see is decided by the caller (library access), not here.

use crate::items::{CARD_SELECT, ItemCard};
use crate::{DbError, now};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub tmdb_id: Option<i64>,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Collection>, DbError> {
    Ok(
        sqlx::query_as("SELECT id, name, tmdb_id FROM collections ORDER BY name COLLATE NOCASE")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<Collection>, DbError> {
    Ok(
        sqlx::query_as("SELECT id, name, tmdb_id FROM collections WHERE id = ?1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn create(pool: &SqlitePool, name: &str, tmdb_id: Option<i64>) -> Result<i64, DbError> {
    Ok(
        sqlx::query("INSERT INTO collections (name, tmdb_id, created_at) VALUES (?1, ?2, ?3)")
            .bind(name)
            .bind(tmdb_id)
            .bind(now())
            .execute(pool)
            .await?
            .last_insert_rowid(),
    )
}

pub async fn rename(pool: &SqlitePool, id: i64, name: &str) -> Result<(), DbError> {
    sqlx::query("UPDATE collections SET name = ?2 WHERE id = ?1")
        .bind(id)
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    sqlx::query("DELETE FROM collections WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Members, oldest film first.
pub async fn items(pool: &SqlitePool, viewer_id: i64, id: i64) -> Result<Vec<ItemCard>, DbError> {
    let sql = format!(
        "{CARD_SELECT} JOIN collection_items ci ON ci.item_id = i.id WHERE ci.collection_id = ?2
         ORDER BY i.year IS NULL, i.year, i.sort_title"
    );
    Ok(sqlx::query_as(&sql)
        .bind(viewer_id)
        .bind(id)
        .fetch_all(pool)
        .await?)
}

pub async fn set_items(pool: &SqlitePool, id: i64, item_ids: &[i64]) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM collection_items WHERE collection_id = ?1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for item in item_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO collection_items (collection_id, item_id) VALUES (?1, ?2)",
        )
        .bind(id)
        .bind(item)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Puts a movie in the collection TMDB reported for it, creating the collection on first sight.
/// An admin's rename of the collection is kept.
pub async fn assign_tmdb(
    pool: &SqlitePool,
    item_id: i64,
    tmdb_collection_id: i64,
    name: &str,
) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT OR IGNORE INTO collections (name, tmdb_id, created_at) VALUES (?1, ?2, ?3)",
    )
    .bind(name)
    .bind(tmdb_collection_id)
    .bind(now())
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT OR IGNORE INTO collection_items (collection_id, item_id)
         SELECT id, ?1 FROM collections WHERE tmdb_id = ?2",
    )
    .bind(item_id)
    .bind(tmdb_collection_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Collections an item belongs to, for its detail page.
pub async fn of_item(pool: &SqlitePool, item_id: i64) -> Result<Vec<Collection>, DbError> {
    Ok(sqlx::query_as(
        "SELECT c.id, c.name, c.tmdb_id FROM collections c
         JOIN collection_items ci ON ci.collection_id = c.id WHERE ci.item_id = ?1
         ORDER BY c.name COLLATE NOCASE",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?)
}
