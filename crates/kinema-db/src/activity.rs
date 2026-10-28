//! Watch time per profile: seconds played per title and UTC day, and the summaries built on it.

use crate::{DbError, now};
use serde::Serialize;
use sqlx::SqlitePool;

pub fn today() -> i64 {
    now() / 86_400
}

pub async fn add_watch_time(
    pool: &SqlitePool,
    user_id: i64,
    item_id: i64,
    seconds: i64,
) -> Result<(), DbError> {
    if seconds <= 0 {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO watch_time (user_id, item_id, day, seconds) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(user_id, item_id, day) DO UPDATE SET seconds = seconds + excluded.seconds",
    )
    .bind(user_id)
    .bind(item_id)
    .bind(today())
    .bind(seconds)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Clone, Default, Serialize, sqlx::FromRow)]
pub struct Totals {
    pub total_s: i64,
    pub last_7d_s: i64,
    pub last_30d_s: i64,
    /// Titles with any watch time.
    pub titles: i64,
    /// Movies and episodes marked watched.
    pub completed: i64,
    pub last_played_at: Option<i64>,
}

pub async fn totals(pool: &SqlitePool, user_id: i64) -> Result<Totals, DbError> {
    Ok(sqlx::query_as(
        "SELECT
           COALESCE(SUM(seconds), 0) AS total_s,
           COALESCE(SUM(CASE WHEN day > ?2 - 7 THEN seconds END), 0) AS last_7d_s,
           COALESCE(SUM(CASE WHEN day > ?2 - 30 THEN seconds END), 0) AS last_30d_s,
           COUNT(DISTINCT item_id) AS titles,
           (SELECT COUNT(*) FROM watch_state WHERE user_id = ?1 AND watched = 1) AS completed,
           (SELECT MAX(last_played_at) FROM watch_state WHERE user_id = ?1) AS last_played_at
         FROM watch_time WHERE user_id = ?1",
    )
    .bind(user_id)
    .bind(today())
    .fetch_one(pool)
    .await?)
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Day {
    pub day: i64,
    pub seconds: i64,
}

/// Seconds per day for the last `days` days; days without play are left out.
pub async fn daily(pool: &SqlitePool, user_id: i64, days: i64) -> Result<Vec<Day>, DbError> {
    Ok(sqlx::query_as(
        "SELECT day, SUM(seconds) AS seconds FROM watch_time
         WHERE user_id = ?1 AND day > ?2 - ?3 GROUP BY day ORDER BY day",
    )
    .bind(user_id)
    .bind(today())
    .bind(days)
    .fetch_all(pool)
    .await?)
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TopTitle {
    pub item_id: i64,
    pub title: String,
    pub kind: String,
    pub poster: Option<String>,
    pub seconds: i64,
}

/// Most watched movies and shows; episodes count towards their show.
pub async fn top_titles(
    pool: &SqlitePool,
    user_id: i64,
    limit: i64,
) -> Result<Vec<TopTitle>, DbError> {
    Ok(sqlx::query_as(
        "SELECT r.id AS item_id, r.title, r.kind, r.poster, SUM(t.seconds) AS seconds
         FROM watch_time t
         JOIN items i ON i.id = t.item_id
         LEFT JOIN items p ON p.id = i.parent_id
         LEFT JOIN items g ON g.id = p.parent_id
         JOIN items r ON r.id = COALESCE(g.id, p.id, i.id)
         WHERE t.user_id = ?1
         GROUP BY r.id ORDER BY seconds DESC LIMIT ?2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct GenreTime {
    pub genre: String,
    pub seconds: i64,
}

pub async fn top_genres(
    pool: &SqlitePool,
    user_id: i64,
    limit: i64,
) -> Result<Vec<GenreTime>, DbError> {
    Ok(sqlx::query_as(
        "SELECT gn.name AS genre, SUM(t.seconds) AS seconds
         FROM watch_time t
         JOIN items i ON i.id = t.item_id
         LEFT JOIN items p ON p.id = i.parent_id
         LEFT JOIN items g ON g.id = p.parent_id
         JOIN item_genres ig ON ig.item_id = COALESCE(g.id, p.id, i.id)
         JOIN genres gn ON gn.id = ig.genre_id
         WHERE t.user_id = ?1
         GROUP BY gn.id ORDER BY seconds DESC LIMIT ?2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

/// Recently played titles: (item id, last played at), newest first.
pub async fn history(
    pool: &SqlitePool,
    user_id: i64,
    limit: i64,
) -> Result<Vec<(i64, i64)>, DbError> {
    Ok(sqlx::query_as(
        "SELECT item_id, last_played_at FROM watch_state
         WHERE user_id = ?1 AND last_played_at IS NOT NULL
         ORDER BY last_played_at DESC LIMIT ?2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}
