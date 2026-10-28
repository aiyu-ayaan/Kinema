//! Devices a profile signed in from. A non-admin profile gets one approved web or desktop
//! device and one mobile device for free; every other device waits for the administrator.

use crate::{DbError, now};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Device {
    pub id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub kind: String,
    pub name: String,
    pub status: String,
    pub created_at: i64,
    pub last_seen: i64,
}

const SELECT: &str = "SELECT d.id, d.user_id, u.name AS user_name, d.kind, d.name, d.status,
  d.created_at, d.last_seen
FROM devices d JOIN users u ON u.id = d.user_id";

/// Web and desktop share one free slot, mobile has its own.
pub fn slot(kind: &str) -> &'static str {
    if kind == "mobile" { "mobile" } else { "screen" }
}

pub async fn find(
    pool: &SqlitePool,
    user_id: i64,
    client_id: &str,
) -> Result<Option<Device>, DbError> {
    Ok(sqlx::query_as(&format!(
        "{SELECT} WHERE d.user_id = ?1 AND d.client_id = ?2"
    ))
    .bind(user_id)
    .bind(client_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<Device>, DbError> {
    Ok(sqlx::query_as(&format!("{SELECT} WHERE d.id = ?1"))
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

/// Approved devices of the profile that use the same free slot as `kind`.
pub async fn approved_in_slot(pool: &SqlitePool, user_id: i64, kind: &str) -> Result<i64, DbError> {
    let mobile = slot(kind) == "mobile";
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*) FROM devices
         WHERE user_id = ?1 AND status = 'approved' AND (kind = 'mobile') = ?2",
    )
    .bind(user_id)
    .bind(mobile)
    .fetch_one(pool)
    .await?)
}

pub async fn pending_count(pool: &SqlitePool, user_id: i64) -> Result<i64, DbError> {
    Ok(
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM devices WHERE user_id = ?1 AND status = 'pending'",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?,
    )
}

pub struct NewDevice<'a> {
    pub user_id: i64,
    pub client_id: &'a str,
    pub kind: &'a str,
    pub name: &'a str,
    pub approved: bool,
    pub poll_hash: Option<&'a str>,
}

/// Inserts the device, or refreshes name, kind and poll secret of the same client.
/// An approved device stays approved.
pub async fn upsert(pool: &SqlitePool, d: &NewDevice<'_>) -> Result<i64, DbError> {
    let t = now();
    Ok(sqlx::query_scalar(
        "INSERT INTO devices (user_id, client_id, kind, name, status, poll_hash, created_at, last_seen)
         VALUES (?1, ?2, ?3, ?4, CASE WHEN ?5 THEN 'approved' ELSE 'pending' END, ?6, ?7, ?7)
         ON CONFLICT(user_id, client_id) DO UPDATE SET
           kind = excluded.kind, name = excluded.name, last_seen = excluded.last_seen,
           status = CASE WHEN devices.status = 'approved' OR ?5 THEN 'approved' ELSE 'pending' END,
           poll_hash = excluded.poll_hash
         RETURNING id",
    )
    .bind(d.user_id)
    .bind(d.client_id)
    .bind(d.kind)
    .bind(d.name)
    .bind(d.approved)
    .bind(d.poll_hash)
    .bind(t)
    .fetch_one(pool)
    .await?)
}

/// The device a waiting client polls for, by the SHA-256 of its poll secret.
pub async fn by_poll_hash(pool: &SqlitePool, hash: &str) -> Result<Option<Device>, DbError> {
    Ok(sqlx::query_as(&format!("{SELECT} WHERE d.poll_hash = ?1"))
        .bind(hash)
        .fetch_optional(pool)
        .await?)
}

/// The poll secret works once: after it has been exchanged for a session.
pub async fn clear_poll(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    sqlx::query("UPDATE devices SET poll_hash = NULL WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn approve(pool: &SqlitePool, id: i64) -> Result<bool, DbError> {
    Ok(
        sqlx::query("UPDATE devices SET status = 'approved' WHERE id = ?1")
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected()
            == 1,
    )
}

/// Denies a request or removes a device; its sessions end with it (ON DELETE CASCADE).
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<bool, DbError> {
    Ok(sqlx::query("DELETE FROM devices WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected()
        == 1)
}

pub async fn touch(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    sqlx::query("UPDATE devices SET last_seen = ?2 WHERE id = ?1")
        .bind(id)
        .bind(now())
        .execute(pool)
        .await?;
    Ok(())
}

/// Requests waiting for approval first, then the most recently used devices.
pub async fn list(pool: &SqlitePool) -> Result<Vec<Device>, DbError> {
    Ok(sqlx::query_as(&format!(
        "{SELECT} ORDER BY d.status = 'pending' DESC, d.last_seen DESC"
    ))
    .fetch_all(pool)
    .await?)
}

/// The device behind a session token, for "now playing".
pub async fn for_session(pool: &SqlitePool, token_hash: &str) -> Result<Option<Device>, DbError> {
    Ok(sqlx::query_as(&format!(
        "{SELECT} JOIN sessions s ON s.device_id = d.id WHERE s.id = ?1"
    ))
    .bind(token_hash)
    .fetch_optional(pool)
    .await?)
}
