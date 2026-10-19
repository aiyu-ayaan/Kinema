use crate::{DbError, now};
use serde::Serialize;
use sqlx::SqlitePool;

/// What a poster card or list row needs. Episodes and seasons inherit missing
/// art from their parents; an episode's still lives in `backdrop`.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ItemCard {
    pub id: i64,
    pub library_id: i64,
    pub parent_id: Option<i64>,
    pub kind: String,
    pub title: String,
    pub year: Option<i64>,
    pub season_no: Option<i64>,
    pub episode_no: Option<i64>,
    pub summary: Option<String>,
    pub rating: Option<f64>,
    pub poster: Option<String>,
    pub backdrop: Option<String>,
    pub show_title: Option<String>,
    pub position_s: Option<f64>,
    pub watched: bool,
    pub duration_s: Option<f64>,
    pub child_count: i64,
    pub added_at: i64,
}

const CARD_SELECT: &str = "SELECT i.id, i.library_id, i.parent_id, i.kind, i.title, i.year, i.season_no, i.episode_no,
  i.summary, i.rating,
  COALESCE(i.poster, p.poster, g.poster) AS poster,
  COALESCE(i.backdrop, p.backdrop, g.backdrop) AS backdrop,
  COALESCE(g.title, p.title) AS show_title,
  w.position_s, COALESCE(w.watched, 0) AS watched,
  (SELECT MAX(f.duration_s) FROM files f WHERE f.item_id = i.id) AS duration_s,
  (SELECT COUNT(*) FROM items c WHERE c.parent_id = i.id) AS child_count,
  i.added_at
FROM items i
LEFT JOIN items p ON p.id = i.parent_id
LEFT JOIN items g ON g.id = p.parent_id
LEFT JOIN watch_state w ON w.item_id = i.id AND w.user_id = ?1";

pub async fn library_items(
    pool: &SqlitePool,
    user_id: i64,
    library_id: i64,
) -> Result<Vec<ItemCard>, DbError> {
    let sql = format!(
        "{CARD_SELECT} WHERE i.library_id = ?2 AND i.parent_id IS NULL ORDER BY i.sort_title"
    );
    Ok(sqlx::query_as(&sql)
        .bind(user_id)
        .bind(library_id)
        .fetch_all(pool)
        .await?)
}

pub async fn children(
    pool: &SqlitePool,
    user_id: i64,
    parent_id: i64,
) -> Result<Vec<ItemCard>, DbError> {
    let sql = format!(
        "{CARD_SELECT} WHERE i.parent_id = ?2
         ORDER BY i.season_no, i.episode_no, i.sort_title"
    );
    Ok(sqlx::query_as(&sql)
        .bind(user_id)
        .bind(parent_id)
        .fetch_all(pool)
        .await?)
}

pub async fn card(
    pool: &SqlitePool,
    user_id: i64,
    item_id: i64,
) -> Result<Option<ItemCard>, DbError> {
    let sql = format!("{CARD_SELECT} WHERE i.id = ?2");
    Ok(sqlx::query_as(&sql)
        .bind(user_id)
        .bind(item_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn recently_added(
    pool: &SqlitePool,
    user_id: i64,
    library_id: i64,
    limit: i64,
) -> Result<Vec<ItemCard>, DbError> {
    let sql = format!(
        "{CARD_SELECT} WHERE i.library_id = ?2 AND i.parent_id IS NULL
         ORDER BY i.added_at DESC, i.id DESC LIMIT ?3"
    );
    Ok(sqlx::query_as(&sql)
        .bind(user_id)
        .bind(library_id)
        .bind(limit)
        .fetch_all(pool)
        .await?)
}

pub async fn continue_watching(
    pool: &SqlitePool,
    user_id: i64,
    limit: i64,
) -> Result<Vec<ItemCard>, DbError> {
    let sql = format!(
        "{CARD_SELECT} WHERE w.position_s > 0 AND w.watched = 0
         ORDER BY w.last_played_at DESC LIMIT ?2"
    );
    Ok(sqlx::query_as(&sql)
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await?)
}

/// shortcut: LIKE scan instead of FTS5; move to items_fts once libraries reach tens of thousands of items.
pub async fn search(
    pool: &SqlitePool,
    user_id: i64,
    query: &str,
    limit: i64,
) -> Result<Vec<ItemCard>, DbError> {
    let sql = format!(
        "{CARD_SELECT} WHERE i.kind IN ('movie','show','episode','video') AND i.title LIKE ?2 ESCAPE '\\'
         ORDER BY i.kind = 'episode', i.sort_title LIMIT ?3"
    );
    let pattern = format!(
        "%{}%",
        query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
    );
    Ok(sqlx::query_as(&sql)
        .bind(user_id)
        .bind(pattern)
        .bind(limit)
        .fetch_all(pool)
        .await?)
}

pub async fn genres(pool: &SqlitePool, item_id: i64) -> Result<Vec<String>, DbError> {
    Ok(sqlx::query_scalar(
        "SELECT g.name FROM item_genres ig JOIN genres g ON g.id = ig.genre_id
         WHERE ig.item_id = ?1 ORDER BY g.name",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await?)
}

// ---- files and streams ----

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct MediaFile {
    pub id: i64,
    pub item_id: Option<i64>,
    pub path: String,
    pub size: i64,
    pub mtime: i64,
    pub container: Option<String>,
    pub duration_s: Option<f64>,
    pub bitrate: Option<i64>,
    pub probe_status: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Stream {
    pub id: i64,
    pub file_id: i64,
    pub idx: i64,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub kind: String,
    pub codec: Option<String>,
    pub profile: Option<String>,
    pub language: Option<String>,
    pub title: Option<String>,
    pub channels: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub is_default: bool,
    #[serde(skip)]
    pub external_path: Option<String>,
}

/// Stream fields written by a probe; `id` and `file_id` are assigned on insert.
#[derive(Debug, Clone, Default)]
pub struct NewStream {
    pub idx: i64,
    pub kind: String,
    pub codec: Option<String>,
    pub profile: Option<String>,
    pub language: Option<String>,
    pub title: Option<String>,
    pub channels: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub is_default: bool,
    pub external_path: Option<String>,
}

const FILE_COLS: &str =
    "id, item_id, path, size, mtime, container, duration_s, bitrate, probe_status";

pub async fn file(pool: &SqlitePool, id: i64) -> Result<Option<MediaFile>, DbError> {
    Ok(
        sqlx::query_as(&format!("SELECT {FILE_COLS} FROM files WHERE id = ?1"))
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn file_by_path(pool: &SqlitePool, path: &str) -> Result<Option<MediaFile>, DbError> {
    Ok(
        sqlx::query_as(&format!("SELECT {FILE_COLS} FROM files WHERE path = ?1"))
            .bind(path)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn item_files(pool: &SqlitePool, item_id: i64) -> Result<Vec<MediaFile>, DbError> {
    Ok(sqlx::query_as(&format!(
        "SELECT {FILE_COLS} FROM files WHERE item_id = ?1 ORDER BY size DESC"
    ))
    .bind(item_id)
    .fetch_all(pool)
    .await?)
}

/// (file id, path) of every file in a library.
pub async fn library_files(
    pool: &SqlitePool,
    library_id: i64,
) -> Result<Vec<(i64, String)>, DbError> {
    Ok(sqlx::query_as(
        "SELECT f.id, f.path FROM files f JOIN items i ON i.id = f.item_id WHERE i.library_id = ?1",
    )
    .bind(library_id)
    .fetch_all(pool)
    .await?)
}

pub async fn streams(pool: &SqlitePool, file_id: i64) -> Result<Vec<Stream>, DbError> {
    Ok(sqlx::query_as(
        "SELECT id, file_id, idx, type, codec, profile, language, title, channels, width, height,
           is_default, external_path
         FROM streams WHERE file_id = ?1 ORDER BY external_path IS NOT NULL, idx",
    )
    .bind(file_id)
    .fetch_all(pool)
    .await?)
}

pub async fn stream(pool: &SqlitePool, id: i64) -> Result<Option<Stream>, DbError> {
    Ok(sqlx::query_as(
        "SELECT id, file_id, idx, type, codec, profile, language, title, channels, width, height,
           is_default, external_path
         FROM streams WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn insert_file(
    pool: &SqlitePool,
    item_id: i64,
    path: &str,
    size: i64,
    mtime: i64,
    fingerprint: &str,
) -> Result<i64, DbError> {
    Ok(sqlx::query(
        "INSERT INTO files (item_id, path, size, mtime, fingerprint) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(item_id)
    .bind(path)
    .bind(size)
    .bind(mtime)
    .bind(fingerprint)
    .execute(pool)
    .await?
    .last_insert_rowid())
}

/// File changed on disk: keep the item, forget the old probe.
pub async fn reset_file(
    pool: &SqlitePool,
    id: i64,
    size: i64,
    mtime: i64,
    fingerprint: &str,
) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE files SET size = ?2, mtime = ?3, fingerprint = ?4, probe_status = 'pending' WHERE id = ?1",
    )
    .bind(id)
    .bind(size)
    .bind(mtime)
    .bind(fingerprint)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn save_probe(
    pool: &SqlitePool,
    file_id: i64,
    container: Option<&str>,
    duration_s: Option<f64>,
    bitrate: Option<i64>,
    streams: &[NewStream],
) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE files SET container = ?2, duration_s = ?3, bitrate = ?4, probe_status = 'ok' WHERE id = ?1",
    )
    .bind(file_id)
    .bind(container)
    .bind(duration_s)
    .bind(bitrate)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM streams WHERE file_id = ?1")
        .bind(file_id)
        .execute(&mut *tx)
        .await?;
    for s in streams {
        sqlx::query(
            "INSERT INTO streams (file_id, idx, type, codec, profile, language, title, channels,
               width, height, is_default, external_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .bind(file_id)
        .bind(s.idx)
        .bind(&s.kind)
        .bind(&s.codec)
        .bind(&s.profile)
        .bind(&s.language)
        .bind(&s.title)
        .bind(s.channels)
        .bind(s.width)
        .bind(s.height)
        .bind(s.is_default)
        .bind(&s.external_path)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn mark_probe_failed(pool: &SqlitePool, file_id: i64) -> Result<(), DbError> {
    sqlx::query("UPDATE files SET probe_status = 'failed' WHERE id = ?1")
        .bind(file_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Removes files gone from disk, then items left without files or children.
pub async fn delete_files(pool: &SqlitePool, ids: &[i64]) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    for id in ids {
        sqlx::query("DELETE FROM files WHERE id = ?1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    // Leaves first (movies, episodes), then seasons, then shows.
    for kinds in ["'movie','episode','video'", "'season'", "'show'"] {
        sqlx::query(&format!(
            "DELETE FROM items WHERE kind IN ({kinds})
               AND NOT EXISTS (SELECT 1 FROM files f WHERE f.item_id = items.id)
               AND NOT EXISTS (SELECT 1 FROM items c WHERE c.parent_id = items.id)"
        ))
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

// ---- item creation and metadata ----

/// What the scanner learned from a path.
#[derive(Debug, Clone, Copy)]
pub enum NewItem<'a> {
    Movie { title: &'a str, year: Option<i64> },
    Show { title: &'a str, year: Option<i64> },
    Season { show_id: i64, season: i64 },
    Episode { season_id: i64, episode: i64, title: &'a str },
    Video { title: &'a str },
}

/// Finds the matching item or creates it. Movies with the same title and year
/// merge, so two copies of a film become two files of one item.
pub async fn find_or_create(
    pool: &SqlitePool,
    library_id: i64,
    item: NewItem<'_>,
) -> Result<i64, DbError> {
    let existing: Option<i64> = match item {
        NewItem::Movie { title, year } => {
            sqlx::query_scalar(
                "SELECT id FROM items WHERE library_id = ?1 AND kind = 'movie'
                   AND title = ?2 COLLATE NOCASE AND year IS ?3",
            )
            .bind(library_id)
            .bind(title)
            .bind(year)
            .fetch_optional(pool)
            .await?
        }
        NewItem::Show { title, .. } => {
            sqlx::query_scalar(
                "SELECT id FROM items WHERE library_id = ?1 AND kind = 'show' AND title = ?2 COLLATE NOCASE",
            )
            .bind(library_id)
            .bind(title)
            .fetch_optional(pool)
            .await?
        }
        NewItem::Season { show_id, season } => {
            sqlx::query_scalar(
                "SELECT id FROM items WHERE parent_id = ?1 AND kind = 'season' AND season_no = ?2",
            )
            .bind(show_id)
            .bind(season)
            .fetch_optional(pool)
            .await?
        }
        NewItem::Episode {
            season_id, episode, ..
        } => {
            sqlx::query_scalar(
                "SELECT id FROM items WHERE parent_id = ?1 AND kind = 'episode' AND episode_no = ?2",
            )
            .bind(season_id)
            .bind(episode)
            .fetch_optional(pool)
            .await?
        }
        // Each loose video is its own item.
        NewItem::Video { .. } => None,
    };
    if let Some(id) = existing {
        return Ok(id);
    }

    let (kind, parent, title, year, season_no, episode_no) = match item {
        NewItem::Movie { title, year } => ("movie", None, title.to_string(), year, None, None),
        NewItem::Show { title, year } => ("show", None, title.to_string(), year, None, None),
        NewItem::Season { show_id, season } => (
            "season",
            Some(show_id),
            format!("Season {season}"),
            None,
            Some(season),
            None,
        ),
        NewItem::Episode {
            season_id,
            episode,
            title,
        } => {
            let season_no: Option<i64> =
                sqlx::query_scalar("SELECT season_no FROM items WHERE id = ?1")
                    .bind(season_id)
                    .fetch_optional(pool)
                    .await?
                    .flatten();
            (
                "episode",
                Some(season_id),
                title.to_string(),
                None,
                season_no,
                Some(episode),
            )
        }
        NewItem::Video { title } => ("video", None, title.to_string(), None, None, None),
    };
    let t = now();
    Ok(sqlx::query(
        "INSERT INTO items (library_id, parent_id, kind, title, sort_title, year, season_no, episode_no, added_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
    )
    .bind(library_id)
    .bind(parent)
    .bind(kind)
    .bind(&title)
    .bind(sort_title(&title))
    .bind(year)
    .bind(season_no)
    .bind(episode_no)
    .bind(t)
    .execute(pool)
    .await?
    .last_insert_rowid())
}

/// "The Matrix" sorts under M.
pub fn sort_title(title: &str) -> String {
    let lower = title.to_lowercase();
    for article in ["the ", "a ", "an "] {
        if let Some(rest) = lower.strip_prefix(article) {
            return rest.to_string();
        }
    }
    lower
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MetaTarget {
    pub id: i64,
    pub kind: String,
    pub title: String,
    pub year: Option<i64>,
}

/// Movies and shows in a library that have never been looked up.
pub async fn needing_meta(pool: &SqlitePool, library_id: i64) -> Result<Vec<MetaTarget>, DbError> {
    Ok(sqlx::query_as(
        "SELECT id, kind, title, year FROM items
         WHERE library_id = ?1 AND kind IN ('movie','show') AND meta_tried = 0",
    )
    .bind(library_id)
    .fetch_all(pool)
    .await?)
}

/// Allow lookups again, e.g. after an API key was added.
pub async fn reset_meta_tried(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("UPDATE items SET meta_tried = 0 WHERE tmdb_id IS NULL")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_meta_tried(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    sqlx::query("UPDATE items SET meta_tried = 1 WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct Meta {
    pub tmdb_id: Option<i64>,
    pub title: Option<String>,
    pub year: Option<i64>,
    pub summary: Option<String>,
    pub rating: Option<f64>,
    pub runtime_s: Option<i64>,
    pub poster: Option<String>,
    pub backdrop: Option<String>,
    pub genres: Vec<String>,
}

/// Fills metadata; `None` fields keep what the scanner found.
pub async fn save_meta(pool: &SqlitePool, id: i64, m: &Meta) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE items SET
           tmdb_id = COALESCE(?2, tmdb_id),
           title = COALESCE(?3, title),
           sort_title = COALESCE(?4, sort_title),
           year = COALESCE(?5, year),
           summary = COALESCE(?6, summary),
           rating = COALESCE(?7, rating),
           runtime_s = COALESCE(?8, runtime_s),
           poster = COALESCE(?9, poster),
           backdrop = COALESCE(?10, backdrop),
           meta_tried = 1,
           updated_at = ?11
         WHERE id = ?1",
    )
    .bind(id)
    .bind(m.tmdb_id)
    .bind(&m.title)
    .bind(m.title.as_deref().map(sort_title))
    .bind(m.year)
    .bind(&m.summary)
    .bind(m.rating)
    .bind(m.runtime_s)
    .bind(&m.poster)
    .bind(&m.backdrop)
    .bind(now())
    .execute(&mut *tx)
    .await?;
    for g in &m.genres {
        sqlx::query("INSERT OR IGNORE INTO genres (name) VALUES (?1)")
            .bind(g)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT OR IGNORE INTO item_genres (item_id, genre_id) SELECT ?1, id FROM genres WHERE name = ?2",
        )
        .bind(id)
        .bind(g)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// (id, season_no or episode_no) of an item's children.
pub async fn numbered_children(
    pool: &SqlitePool,
    parent_id: i64,
) -> Result<Vec<(i64, i64)>, DbError> {
    Ok(sqlx::query_as(
        "SELECT id, COALESCE(CASE kind WHEN 'season' THEN season_no ELSE episode_no END, 0)
         FROM items WHERE parent_id = ?1",
    )
    .bind(parent_id)
    .fetch_all(pool)
    .await?)
}

/// tmdb ids of shows whose seasons got new episodes without metadata.
pub async fn shows_with_bare_episodes(
    pool: &SqlitePool,
    library_id: i64,
) -> Result<Vec<(i64, i64)>, DbError> {
    Ok(sqlx::query_as(
        "SELECT DISTINCT sh.id, sh.tmdb_id FROM items e
         JOIN items se ON se.id = e.parent_id
         JOIN items sh ON sh.id = se.parent_id
         WHERE sh.library_id = ?1 AND sh.tmdb_id IS NOT NULL AND e.kind = 'episode' AND e.meta_tried = 0",
    )
    .bind(library_id)
    .fetch_all(pool)
    .await?)
}

// ---- watch state ----

pub async fn set_progress(
    pool: &SqlitePool,
    user_id: i64,
    item_id: i64,
    position_s: f64,
    watched: bool,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO watch_state (user_id, item_id, position_s, watched, play_count, last_played_at)
         VALUES (?1, ?2, ?3, ?4, ?4, ?5)
         ON CONFLICT(user_id, item_id) DO UPDATE SET
           position_s = excluded.position_s,
           play_count = play_count + (excluded.watched AND NOT watch_state.watched),
           watched = excluded.watched,
           last_played_at = excluded.last_played_at",
    )
    .bind(user_id)
    .bind(item_id)
    .bind(if watched { 0.0 } else { position_s })
    .bind(watched)
    .bind(now())
    .execute(pool)
    .await?;
    Ok(())
}
