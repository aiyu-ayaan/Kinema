use crate::{DbError, now};
use serde::Serialize;
use sqlx::SqlitePool;

/// A profile. Hashes never leave this module; only whether a lock is set.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub is_admin: bool,
    pub avatar: Option<String>,
    pub has_password: bool,
    pub has_pin: bool,
}

const USER_COLS: &str = "u.id, u.name, u.is_admin, u.avatar,
  u.password_hash IS NOT NULL AS has_password, u.pin_hash IS NOT NULL AS has_pin";

pub async fn count(pool: &SqlitePool) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?)
}

pub async fn admin_count(pool: &SqlitePool) -> Result<i64, DbError> {
    Ok(
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_admin = 1")
            .fetch_one(pool)
            .await?,
    )
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

pub async fn list(pool: &SqlitePool) -> Result<Vec<User>, DbError> {
    Ok(sqlx::query_as(&format!(
        "SELECT {USER_COLS} FROM users u ORDER BY u.is_admin DESC, u.name COLLATE NOCASE"
    ))
    .fetch_all(pool)
    .await?)
}

pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<User>, DbError> {
    Ok(
        sqlx::query_as(&format!("SELECT {USER_COLS} FROM users u WHERE u.id = ?1"))
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn find_by_name(pool: &SqlitePool, name: &str) -> Result<Option<User>, DbError> {
    Ok(sqlx::query_as(&format!(
        "SELECT {USER_COLS} FROM users u WHERE u.name = ?1 COLLATE NOCASE"
    ))
    .bind(name)
    .fetch_optional(pool)
    .await?)
}

/// (password_hash, pin_hash) for verifying a sign-in.
pub async fn secrets(
    pool: &SqlitePool,
    id: i64,
) -> Result<(Option<String>, Option<String>), DbError> {
    Ok(
        sqlx::query_as("SELECT password_hash, pin_hash FROM users WHERE id = ?1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .unwrap_or((None, None)),
    )
}

pub struct NewUser<'a> {
    pub name: &'a str,
    pub is_admin: bool,
    pub avatar: &'a str,
    pub password_hash: Option<&'a str>,
    pub pin_hash: Option<&'a str>,
}

/// None when the name is taken.
pub async fn create(pool: &SqlitePool, u: &NewUser<'_>) -> Result<Option<i64>, DbError> {
    let res = sqlx::query(
        "INSERT INTO users (name, password_hash, pin_hash, is_admin, avatar, created_at)
         SELECT ?1, ?2, ?3, ?4, ?5, ?6
         WHERE NOT EXISTS (SELECT 1 FROM users WHERE name = ?1 COLLATE NOCASE)",
    )
    .bind(u.name)
    .bind(u.password_hash)
    .bind(u.pin_hash)
    .bind(u.is_admin)
    .bind(u.avatar)
    .bind(now())
    .execute(pool)
    .await?;
    Ok((res.rows_affected() == 1).then(|| res.last_insert_rowid()))
}

/// How a lock field changes on update.
pub enum Secret<'a> {
    Keep,
    Clear,
    Set(&'a str),
}

pub struct UserUpdate<'a> {
    pub name: &'a str,
    pub is_admin: bool,
    pub avatar: &'a str,
    pub password_hash: Secret<'a>,
    pub pin_hash: Secret<'a>,
}

/// False when the new name belongs to someone else.
pub async fn update(pool: &SqlitePool, id: i64, u: &UserUpdate<'_>) -> Result<bool, DbError> {
    let taken: Option<i64> =
        sqlx::query_scalar("SELECT id FROM users WHERE name = ?1 COLLATE NOCASE AND id != ?2")
            .bind(u.name)
            .bind(id)
            .fetch_optional(pool)
            .await?;
    if taken.is_some() {
        return Ok(false);
    }
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE users SET name = ?2, is_admin = ?3, avatar = ?4 WHERE id = ?1")
        .bind(id)
        .bind(u.name)
        .bind(u.is_admin)
        .bind(u.avatar)
        .execute(&mut *tx)
        .await?;
    for (column, secret) in [
        ("password_hash", &u.password_hash),
        ("pin_hash", &u.pin_hash),
    ] {
        let value = match secret {
            Secret::Keep => continue,
            Secret::Clear => None,
            Secret::Set(h) => Some(*h),
        };
        sqlx::query(&format!("UPDATE users SET {column} = ?2 WHERE id = ?1"))
            .bind(id)
            .bind(value)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(true)
}

/// Watch history, sessions and library access go with the profile (ON DELETE CASCADE).
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    sqlx::query("DELETE FROM users WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Libraries a profile is limited to; empty means every library.
pub async fn library_access(pool: &SqlitePool, user_id: i64) -> Result<Vec<i64>, DbError> {
    Ok(
        sqlx::query_scalar("SELECT library_id FROM user_library_access WHERE user_id = ?1")
            .bind(user_id)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn set_library_access(
    pool: &SqlitePool,
    user_id: i64,
    library_ids: &[i64],
) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM user_library_access WHERE user_id = ?1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    for id in library_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO user_library_access (user_id, library_id)
             SELECT ?1, id FROM libraries WHERE id = ?2",
        )
        .bind(user_id)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn create_session(
    pool: &SqlitePool,
    token_hash: &str,
    user_id: i64,
    ttl_s: i64,
) -> Result<(), DbError> {
    let t = now();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, created_at, expires_at) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(token_hash)
    .bind(user_id)
    .bind(t)
    .bind(t + ttl_s)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn session_user(pool: &SqlitePool, token_hash: &str) -> Result<Option<User>, DbError> {
    Ok(sqlx::query_as(&format!(
        "SELECT {USER_COLS} FROM sessions s JOIN users u ON u.id = s.user_id
         WHERE s.id = ?1 AND s.expires_at > ?2"
    ))
    .bind(token_hash)
    .bind(now())
    .fetch_optional(pool)
    .await?)
}

/// Signs a profile out everywhere, e.g. after its lock changed.
pub async fn delete_user_sessions(pool: &SqlitePool, user_id: i64) -> Result<(), DbError> {
    sqlx::query("DELETE FROM sessions WHERE user_id = ?1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_session(pool: &SqlitePool, token_hash: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM sessions WHERE id = ?1 OR expires_at <= ?2")
        .bind(token_hash)
        .bind(now())
        .execute(pool)
        .await?;
    Ok(())
}
