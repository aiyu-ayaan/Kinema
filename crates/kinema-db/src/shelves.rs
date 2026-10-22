//! Home shelves: a titled genre query ("Oh, The Horror!" = Horror). An admin defines them and each
//! profile can hide any. Items come back newest first.

use crate::items::{CARD_SELECT, ItemCard};
use crate::{DbError, now};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Shelf {
    pub id: i64,
    pub title: String,
    pub genre: String,
    pub position: i64,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Shelf>, DbError> {
    Ok(
        sqlx::query_as("SELECT id, title, genre, position FROM shelves ORDER BY position, id")
            .fetch_all(pool)
            .await?,
    )
}

/// The shelves a profile has not hidden.
pub async fn shown_to(pool: &SqlitePool, user_id: i64) -> Result<Vec<Shelf>, DbError> {
    Ok(sqlx::query_as(
        "SELECT id, title, genre, position FROM shelves
         WHERE id NOT IN (SELECT shelf_id FROM shelf_hidden WHERE user_id = ?1)
         ORDER BY position, id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

/// Ids this profile hid, for the "show again" list.
pub async fn hidden_by(pool: &SqlitePool, user_id: i64) -> Result<Vec<i64>, DbError> {
    Ok(
        sqlx::query_scalar("SELECT shelf_id FROM shelf_hidden WHERE user_id = ?1")
            .bind(user_id)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<Shelf>, DbError> {
    Ok(
        sqlx::query_as("SELECT id, title, genre, position FROM shelves WHERE id = ?1")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn create(pool: &SqlitePool, title: &str, genre: &str) -> Result<i64, DbError> {
    Ok(sqlx::query(
        "INSERT INTO shelves (title, genre, position, created_at)
         VALUES (?1, ?2, (SELECT COALESCE(MAX(position) + 1, 0) FROM shelves), ?3)",
    )
    .bind(title)
    .bind(genre)
    .bind(now())
    .execute(pool)
    .await?
    .last_insert_rowid())
}

pub async fn update(
    pool: &SqlitePool,
    id: i64,
    title: Option<&str>,
    genre: Option<&str>,
) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE shelves SET title = COALESCE(?2, title), genre = COALESCE(?3, genre) WHERE id = ?1",
    )
    .bind(id)
    .bind(title)
    .bind(genre)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    sqlx::query("DELETE FROM shelves WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_hidden(
    pool: &SqlitePool,
    user_id: i64,
    shelf_id: i64,
    hidden: bool,
) -> Result<(), DbError> {
    if hidden {
        sqlx::query("INSERT OR IGNORE INTO shelf_hidden (user_id, shelf_id) VALUES (?1, ?2)")
    } else {
        sqlx::query("DELETE FROM shelf_hidden WHERE user_id = ?1 AND shelf_id = ?2")
    }
    .bind(user_id)
    .bind(shelf_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Top-level titles of a genre, newest first, limited to `libraries` (None = every library).
pub async fn genre_items(
    pool: &SqlitePool,
    viewer_id: i64,
    genre: &str,
    libraries: Option<&[i64]>,
    limit: i64,
) -> Result<Vec<ItemCard>, DbError> {
    // Library ids are integers, so inlining them cannot inject anything.
    let scope = libraries
        .map(|ids| {
            let list: Vec<String> = ids.iter().map(i64::to_string).collect();
            format!("AND i.library_id IN ({})", list.join(","))
        })
        .unwrap_or_default();
    let sql = format!(
        "{CARD_SELECT} WHERE i.parent_id IS NULL AND i.kind IN ('movie','show','video') {scope}
           AND i.id IN (SELECT ig.item_id FROM item_genres ig JOIN genres g ON g.id = ig.genre_id
                        WHERE g.name = ?2 COLLATE NOCASE)
         ORDER BY i.added_at DESC, i.id DESC LIMIT ?3"
    );
    Ok(sqlx::query_as(&sql)
        .bind(viewer_id)
        .bind(genre)
        .bind(limit)
        .fetch_all(pool)
        .await?)
}
