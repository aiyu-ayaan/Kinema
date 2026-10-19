use crate::{DbError, now};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub is_admin: bool,
}

pub async fn count(pool: &SqlitePool) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?)
}

/// Creates the first user as admin. Returns None when a user already exists,
/// so two racing setup requests cannot both become admin.
pub async fn create_first_admin(
    pool: &SqlitePool,
    name: &str,
    password_hash: &str,
) -> Result<Option<i64>, DbError> {
    let res = sqlx::query(
        "INSERT INTO users (name, password_hash, is_admin, created_at)
         SELECT ?1, ?2, 1, ?3 WHERE NOT EXISTS (SELECT 1 FROM users)",
    )
    .bind(name)
    .bind(password_hash)
    .bind(now())
    .execute(pool)
    .await?;
    Ok((res.rows_affected() == 1).then(|| res.last_insert_rowid()))
}

/// (id, password_hash) for a login name, case-insensitive.
pub async fn credentials(
    pool: &SqlitePool,
    name: &str,
) -> Result<Option<(i64, Option<String>)>, DbError> {
    Ok(
        sqlx::query_as("SELECT id, password_hash FROM users WHERE name = ?1 COLLATE NOCASE")
            .bind(name)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn create_session(
    pool: &SqlitePool,
    token_hash: &str,
    user_id: i64,
    ttl_s: i64,
) -> Result<(), DbError> {
    let t = now();
    sqlx::query("INSERT INTO sessions (id, user_id, created_at, expires_at) VALUES (?1, ?2, ?3, ?4)")
        .bind(token_hash)
        .bind(user_id)
        .bind(t)
        .bind(t + ttl_s)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn session_user(pool: &SqlitePool, token_hash: &str) -> Result<Option<User>, DbError> {
    Ok(sqlx::query_as(
        "SELECT u.id, u.name, u.is_admin FROM sessions s JOIN users u ON u.id = s.user_id
         WHERE s.id = ?1 AND s.expires_at > ?2",
    )
    .bind(token_hash)
    .bind(now())
    .fetch_optional(pool)
    .await?)
}

pub async fn delete_session(pool: &SqlitePool, token_hash: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM sessions WHERE id = ?1 OR expires_at <= ?2")
        .bind(token_hash)
        .bind(now())
        .execute(pool)
        .await?;
    Ok(())
}
