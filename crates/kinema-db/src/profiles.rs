//! Per-profile preferences (languages, content limit, LAN unlock) and the watch history
//! rows a profile export carries. Watch rows are keyed by what survives a move to another
//! server: TMDB / TVmaze ids, then title and year, and season / episode numbers.

use crate::{DbError, now};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// A missing row reads as the default: follow the server, no limits, PIN always.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Prefs {
    pub audio_language: Option<String>,
    pub subtitle_language: Option<String>,
    pub subtitle_mode: Option<String>,
    pub max_rating: Option<i64>,
    pub allow_unrated: bool,
    pub lan_unlock: bool,
}

pub async fn get(pool: &SqlitePool, user_id: i64) -> Result<Prefs, DbError> {
    Ok(sqlx::query_as(
        "SELECT audio_language, subtitle_language, subtitle_mode, max_rating, allow_unrated,
                lan_unlock
         FROM profile_prefs WHERE user_id = ?1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or_default())
}

pub async fn set(pool: &SqlitePool, user_id: i64, p: &Prefs) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO profile_prefs (user_id, audio_language, subtitle_language, subtitle_mode,
                                    max_rating, allow_unrated, lan_unlock)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(user_id) DO UPDATE SET
           audio_language = excluded.audio_language,
           subtitle_language = excluded.subtitle_language,
           subtitle_mode = excluded.subtitle_mode,
           max_rating = excluded.max_rating,
           allow_unrated = excluded.allow_unrated,
           lan_unlock = excluded.lan_unlock",
    )
    .bind(user_id)
    .bind(&p.audio_language)
    .bind(&p.subtitle_language)
    .bind(&p.subtitle_mode)
    .bind(p.max_rating)
    .bind(p.allow_unrated)
    .bind(p.lan_unlock)
    .execute(pool)
    .await?;
    Ok(())
}

/// Identifies a title on any Kinema server. For an episode, `tmdb_id`, `tvmaze_id`, `title`
/// and `year` describe its show, and `season_no` / `episode_no` the episode.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct TitleKey {
    pub kind: String,
    pub tmdb_id: Option<i64>,
    pub tvmaze_id: Option<i64>,
    pub title: String,
    pub year: Option<i64>,
    pub season_no: Option<i64>,
    pub episode_no: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct WatchRow {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub key: TitleKey,
    pub position_s: f64,
    pub watched: bool,
    pub play_count: i64,
    pub last_played_at: Option<i64>,
}

/// Every watch row of a profile that can be keyed (movies, shows, episodes, videos).
pub async fn watch_rows(pool: &SqlitePool, user_id: i64) -> Result<Vec<WatchRow>, DbError> {
    Ok(sqlx::query_as(
        "SELECT i.kind,
                CASE WHEN i.kind = 'episode' THEN g.tmdb_id ELSE i.tmdb_id END AS tmdb_id,
                CASE WHEN i.kind = 'episode' THEN g.tvmaze_id ELSE i.tvmaze_id END AS tvmaze_id,
                CASE WHEN i.kind = 'episode' THEN g.title ELSE i.title END AS title,
                CASE WHEN i.kind = 'episode' THEN g.year ELSE i.year END AS year,
                CASE WHEN i.kind = 'episode' THEN i.season_no END AS season_no,
                CASE WHEN i.kind = 'episode' THEN i.episode_no END AS episode_no,
                w.position_s, w.watched, w.play_count, w.last_played_at
         FROM watch_state w
         JOIN items i ON i.id = w.item_id
         LEFT JOIN items p ON p.id = i.parent_id
         LEFT JOIN items g ON g.id = p.parent_id
         WHERE w.user_id = ?1 AND i.kind IN ('movie', 'show', 'episode', 'video')
           AND (i.kind != 'episode' OR g.kind = 'show')
         ORDER BY w.last_played_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

/// The local item a key points at: ids first, then title + year. Ambiguous titles
/// (two matches) resolve to none rather than guess.
pub async fn find_item(pool: &SqlitePool, key: &TitleKey) -> Result<Option<i64>, DbError> {
    let top_kind = if key.kind == "episode" {
        "show"
    } else {
        key.kind.as_str()
    };
    let rows: Vec<(i64, bool)> = sqlx::query_as(
        "SELECT id, ((?2 IS NOT NULL AND tmdb_id = ?2) OR (?3 IS NOT NULL AND tvmaze_id = ?3))
                AS by_id
         FROM items
         WHERE parent_id IS NULL AND kind = ?1 AND (
           (?2 IS NOT NULL AND tmdb_id = ?2) OR (?3 IS NOT NULL AND tvmaze_id = ?3)
           OR (title = ?4 COLLATE NOCASE AND year IS ?5))
         ORDER BY by_id DESC
         LIMIT 2",
    )
    .bind(top_kind)
    .bind(key.tmdb_id)
    .bind(key.tvmaze_id)
    .bind(&key.title)
    .bind(key.year)
    .fetch_all(pool)
    .await?;
    let top = match rows.as_slice() {
        [] => None,
        [(one, _)] => Some(*one),
        [(first, true), (_, false)] => Some(*first),
        _ => None,
    };
    let Some(top) = top else { return Ok(None) };
    if key.kind != "episode" {
        return Ok(Some(top));
    }
    Ok(sqlx::query_scalar(
        "SELECT e.id FROM items e JOIN items s ON s.id = e.parent_id
         WHERE s.parent_id = ?1 AND e.kind = 'episode' AND s.season_no IS ?2 AND e.episode_no IS ?3
         LIMIT 1",
    )
    .bind(top)
    .bind(key.season_no)
    .bind(key.episode_no)
    .fetch_optional(pool)
    .await?)
}

/// Writes an imported watch row unless this server has a newer one for the item.
/// Returns false when the local row was kept.
pub async fn import_watch(
    pool: &SqlitePool,
    user_id: i64,
    item_id: i64,
    row: &WatchRow,
) -> Result<bool, DbError> {
    let played = row.last_played_at.unwrap_or(0).min(now());
    let res = sqlx::query(
        "INSERT INTO watch_state (user_id, item_id, position_s, watched, play_count, last_played_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(user_id, item_id) DO UPDATE SET
           position_s = excluded.position_s,
           watched = excluded.watched,
           play_count = MAX(watch_state.play_count, excluded.play_count),
           last_played_at = excluded.last_played_at
         WHERE COALESCE(watch_state.last_played_at, 0) <= excluded.last_played_at",
    )
    .bind(user_id)
    .bind(item_id)
    .bind(row.position_s)
    .bind(row.watched)
    .bind(row.play_count)
    .bind(played)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() == 1)
}

/// Lock hashes for an explicit admin export; never part of a plain export.
pub async fn lock_hashes(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<(Option<String>, Option<String>), DbError> {
    crate::users::secrets(pool, user_id).await
}

/// Admin import of lock hashes: replaces both, and signs the profile out everywhere.
pub async fn set_lock_hashes(
    pool: &SqlitePool,
    user_id: i64,
    password_hash: Option<&str>,
    pin_hash: Option<&str>,
) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE users SET password_hash = ?2, pin_hash = ?3 WHERE id = ?1")
        .bind(user_id)
        .bind(password_hash)
        .bind(pin_hash)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE user_id = ?1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
