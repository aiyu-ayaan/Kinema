use crate::DbError;
use sqlx::SqlitePool;

pub const TMDB_API_KEY: &str = "tmdb_api_key";

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, DbError> {
    Ok(sqlx::query_scalar("SELECT value FROM settings WHERE key = ?1")
        .bind(key)
        .fetch_optional(pool)
        .await?)
}

/// Empty value deletes the key.
pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), DbError> {
    if value.is_empty() {
        sqlx::query("DELETE FROM settings WHERE key = ?1")
            .bind(key)
            .execute(pool)
            .await?;
    } else {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    }
    Ok(())
}
