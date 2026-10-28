use crate::{DbError, now};
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::HashMap;

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
    /// Season or episode: the show it belongs to, for links back to it.
    pub show_id: Option<i64>,
    pub position_s: Option<f64>,
    pub watched: bool,
    pub duration_s: Option<f64>,
    pub child_count: i64,
    pub added_at: i64,
    /// Certification, inherited from the season or show when the item has none.
    pub content_rating: Option<String>,
}

pub(crate) const CARD_SELECT: &str =
    "SELECT i.id, i.library_id, i.parent_id, i.kind, i.title, i.year, i.season_no, i.episode_no,
  i.summary, i.rating,
  COALESCE(i.poster, p.poster, g.poster) AS poster,
  COALESCE(i.backdrop, p.backdrop, g.backdrop) AS backdrop,
  COALESCE(g.title, p.title) AS show_title,
  COALESCE(g.id, p.id) AS show_id,
  w.position_s, COALESCE(w.watched, 0) AS watched,
  (SELECT MAX(f.duration_s) FROM files f WHERE f.item_id = i.id) AS duration_s,
  (SELECT COUNT(*) FROM items c WHERE c.parent_id = i.id) AS child_count,
  i.added_at,
  COALESCE(i.content_rating, p.content_rating, g.content_rating) AS content_rating
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

/// Which top-level titles of a library a page lists.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KindFilter {
    Any,
    Movies,
    Shows,
}

impl KindFilter {
    fn sql(self) -> &'static str {
        match self {
            Self::Any => "1",
            Self::Movies => "i.kind != 'show'",
            Self::Shows => "i.kind = 'show'",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LibrarySort {
    Title,
    Added,
    Year,
    Rating,
}

impl LibrarySort {
    /// `i.id` last, so equal keys never swap places between pages.
    fn sql(self) -> &'static str {
        match self {
            Self::Title => "i.sort_title, i.id",
            Self::Added => "i.added_at DESC, i.id DESC",
            Self::Year => "COALESCE(i.year, 0) DESC, i.sort_title, i.id",
            Self::Rating => "COALESCE(i.rating, 0) DESC, i.sort_title, i.id",
        }
    }
}

/// One page of a library's top-level titles. The inner query orders and cuts the page by id, so the card joins and
/// subqueries run for `limit` rows, not the whole library.
pub async fn library_page(
    pool: &SqlitePool,
    user_id: i64,
    library_id: i64,
    kind: KindFilter,
    sort: LibrarySort,
    offset: i64,
    limit: i64,
) -> Result<Vec<ItemCard>, DbError> {
    let (filter, order) = (kind.sql(), sort.sql());
    let sql = format!(
        "{CARD_SELECT} WHERE i.id IN (
           SELECT i.id FROM items i WHERE i.library_id = ?2 AND i.parent_id IS NULL AND {filter}
           ORDER BY {order} LIMIT ?3 OFFSET ?4)
         ORDER BY {order}"
    );
    Ok(sqlx::query_as(&sql)
        .bind(user_id)
        .bind(library_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?)
}

/// How many top-level titles `library_page` walks through.
pub async fn library_count(
    pool: &SqlitePool,
    library_id: i64,
    kind: KindFilter,
) -> Result<i64, DbError> {
    let sql = format!(
        "SELECT COUNT(*) FROM items i WHERE i.library_id = ?1 AND i.parent_id IS NULL AND {}",
        kind.sql()
    );
    Ok(sqlx::query_scalar(&sql)
        .bind(library_id)
        .fetch_one(pool)
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

/// Which home row a "recently added" query fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Shelf {
    /// Movies and loose videos.
    Movies,
    Shows,
    Episodes,
}

pub async fn recently_added(
    pool: &SqlitePool,
    user_id: i64,
    library_id: i64,
    shelf: Shelf,
    limit: i64,
) -> Result<Vec<ItemCard>, DbError> {
    let filter = match shelf {
        Shelf::Movies => "i.parent_id IS NULL AND i.kind != 'show'",
        Shelf::Shows => "i.parent_id IS NULL AND i.kind = 'show'",
        Shelf::Episodes => "i.kind = 'episode'",
    };
    // The inner query picks the ids, so the card joins and subqueries run for `limit` rows, not the whole library.
    let sql = format!(
        "{CARD_SELECT} WHERE i.id IN (
           SELECT id FROM items i WHERE i.library_id = ?2 AND {filter}
           ORDER BY i.added_at DESC, i.id DESC LIMIT ?3)
         ORDER BY i.added_at DESC, i.id DESC"
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
    // Ids first, then the card joins for the `limit` rows kept (a broad query matches most of the library).
    let sql = format!(
        "{CARD_SELECT} WHERE i.id IN (
           SELECT id FROM items i WHERE i.kind IN ('movie','show','episode','video') AND i.title LIKE ?2 ESCAPE '\\'
           ORDER BY i.kind = 'episode', i.sort_title LIMIT ?3)
         ORDER BY i.kind = 'episode', i.sort_title"
    );
    let pattern = format!(
        "%{}%",
        query
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
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
    /// Sidecar version ("size-mtime"); part of the subtitle cache key.
    #[serde(skip)]
    pub external_fingerprint: Option<String>,
    /// A picture subtitle read by OCR into WebVTT: players may show it like a text track.
    pub ocr: bool,
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
    pub external_fingerprint: Option<String>,
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

/// The file whose frame stands in for an item's art: its own, or a show's or
/// season's first episode.
pub async fn thumb_file(pool: &SqlitePool, item_id: i64) -> Result<Option<MediaFile>, DbError> {
    Ok(sqlx::query_as(
        "SELECT f.id, f.item_id, f.path, f.size, f.mtime, f.container, f.duration_s, f.bitrate, f.probe_status
         FROM files f JOIN items i ON i.id = f.item_id
         WHERE i.id = ?1 OR i.parent_id = ?1 OR i.parent_id IN (SELECT id FROM items WHERE parent_id = ?1)
         ORDER BY i.id = ?1 DESC, i.season_no = 0, i.season_no, i.episode_no, f.size DESC
         LIMIT 1",
    )
    .bind(item_id)
    .fetch_optional(pool)
    .await?)
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

/// Probed files of a library, in browsing order, for background thumbnail work.
pub async fn library_media_files(
    pool: &SqlitePool,
    library_id: i64,
) -> Result<Vec<MediaFile>, DbError> {
    Ok(sqlx::query_as(
        "SELECT f.id, f.item_id, f.path, f.size, f.mtime, f.container, f.duration_s, f.bitrate, f.probe_status
         FROM files f JOIN items i ON i.id = f.item_id
         WHERE i.library_id = ?1 AND f.probe_status = 'ok'
         ORDER BY i.added_at DESC, i.season_no, i.episode_no",
    )
    .bind(library_id)
    .fetch_all(pool)
    .await?)
}

/// Totals for the dashboard.
#[derive(Debug, Clone, Default, Serialize, sqlx::FromRow)]
pub struct Stats {
    pub movies: i64,
    pub shows: i64,
    pub episodes: i64,
    pub files: i64,
    pub bytes: i64,
    pub hours: f64,
}

pub async fn stats(pool: &SqlitePool) -> Result<Stats, DbError> {
    Ok(sqlx::query_as(
        "SELECT
           (SELECT COUNT(*) FROM items WHERE kind IN ('movie','video')) AS movies,
           (SELECT COUNT(*) FROM items WHERE kind = 'show') AS shows,
           (SELECT COUNT(*) FROM items WHERE kind = 'episode') AS episodes,
           COUNT(*) AS files,
           COALESCE(SUM(size), 0) AS bytes,
           COALESCE(SUM(duration_s), 0) / 3600.0 AS hours
         FROM files",
    )
    .fetch_one(pool)
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

const STREAM_COLS: &str = "id, file_id, idx, type, codec, profile, language, title, channels,
    width, height, is_default, external_path, external_fingerprint, ocr";

pub async fn streams(pool: &SqlitePool, file_id: i64) -> Result<Vec<Stream>, DbError> {
    Ok(sqlx::query_as(&format!(
        "SELECT {STREAM_COLS}
         FROM streams WHERE file_id = ?1 ORDER BY external_path IS NOT NULL, idx"
    ))
    .bind(file_id)
    .fetch_all(pool)
    .await?)
}

pub async fn stream(pool: &SqlitePool, id: i64) -> Result<Option<Stream>, DbError> {
    Ok(
        sqlx::query_as(&format!("SELECT {STREAM_COLS} FROM streams WHERE id = ?1"))
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
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
        insert_stream(&mut tx, file_id, s).await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn insert_stream(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    file_id: i64,
    s: &NewStream,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO streams (file_id, idx, type, codec, profile, language, title, channels,
           width, height, is_default, external_path, external_fingerprint)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
    .bind(&s.external_fingerprint)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// (path, fingerprint) of each sidecar subtitle stored for the probed files of a library,
/// keyed by file id, so a scan can tell which sidecars were added, changed or removed.
pub async fn library_sidecars(
    pool: &SqlitePool,
    library_id: i64,
) -> Result<HashMap<i64, Vec<(String, Option<String>)>>, DbError> {
    let rows: Vec<(i64, String, Option<String>)> = sqlx::query_as(
        "SELECT s.file_id, s.external_path, s.external_fingerprint
         FROM streams s JOIN files f ON f.id = s.file_id JOIN items i ON i.id = f.item_id
         WHERE i.library_id = ?1 AND s.external_path IS NOT NULL
         ORDER BY s.file_id, s.idx",
    )
    .bind(library_id)
    .fetch_all(pool)
    .await?;
    let mut out: HashMap<i64, Vec<(String, Option<String>)>> = HashMap::new();
    for (file_id, path, fp) in rows {
        out.entry(file_id).or_default().push((path, fp));
    }
    Ok(out)
}

/// Swaps a file's sidecar subtitle streams for `sidecars`, leaving its probed streams alone:
/// a subtitle file added next to an unchanged video needs no new probe.
pub async fn replace_sidecars(
    pool: &SqlitePool,
    file_id: i64,
    sidecars: &[NewStream],
) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM streams WHERE file_id = ?1 AND external_path IS NOT NULL")
        .bind(file_id)
        .execute(&mut *tx)
        .await?;
    for s in sidecars {
        insert_stream(&mut tx, file_id, s).await?;
    }
    tx.commit().await?;
    Ok(())
}

/// A subtitle stream with what the background extraction needs of its file.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SubtitleJob {
    pub stream_id: i64,
    pub file_id: i64,
    pub idx: i64,
    pub codec: Option<String>,
    pub language: Option<String>,
    pub external_path: Option<String>,
    pub external_fingerprint: Option<String>,
    pub ocr: bool,
    pub path: String,
    pub size: i64,
    pub mtime: i64,
    pub duration_s: Option<f64>,
}

/// Every subtitle stream of a library's probed files, grouped by file.
pub async fn library_subtitles(
    pool: &SqlitePool,
    library_id: i64,
) -> Result<Vec<SubtitleJob>, DbError> {
    Ok(sqlx::query_as(
        "SELECT s.id AS stream_id, s.file_id, s.idx, s.codec, s.language, s.external_path,
           s.external_fingerprint, s.ocr, f.path, f.size, f.mtime, f.duration_s
         FROM streams s JOIN files f ON f.id = s.file_id JOIN items i ON i.id = f.item_id
         WHERE i.library_id = ?1 AND f.probe_status = 'ok' AND s.type = 'subtitle'
         ORDER BY i.added_at DESC, f.id, s.idx",
    )
    .bind(library_id)
    .fetch_all(pool)
    .await?)
}

pub async fn set_stream_ocr(pool: &SqlitePool, stream_id: i64, ocr: bool) -> Result<(), DbError> {
    sqlx::query("UPDATE streams SET ocr = ?2 WHERE id = ?1")
        .bind(stream_id)
        .bind(ocr)
        .execute(pool)
        .await?;
    Ok(())
}

/// The subtitle cache was cleared: OCR results are gone with it.
pub async fn clear_stream_ocr(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("UPDATE streams SET ocr = 0 WHERE ocr = 1")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_probe_failed(pool: &SqlitePool, file_id: i64) -> Result<(), DbError> {
    sqlx::query("UPDATE files SET probe_status = 'failed' WHERE id = ?1")
        .bind(file_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn kind_of(pool: &SqlitePool, item_id: i64) -> Result<Option<String>, DbError> {
    Ok(sqlx::query_scalar("SELECT kind FROM items WHERE id = ?1")
        .bind(item_id)
        .fetch_optional(pool)
        .await?)
}

/// Re-files a file under another item; the old item is pruned by `delete_files`.
pub async fn move_file(pool: &SqlitePool, file_id: i64, item_id: i64) -> Result<(), DbError> {
    sqlx::query("UPDATE files SET item_id = ?2 WHERE id = ?1")
        .bind(file_id)
        .bind(item_id)
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
    Movie {
        title: &'a str,
        year: Option<i64>,
    },
    Show {
        title: &'a str,
        year: Option<i64>,
    },
    Season {
        show_id: i64,
        season: i64,
    },
    Episode {
        season_id: i64,
        episode: i64,
        title: &'a str,
    },
    Video {
        title: &'a str,
    },
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

/// "Match missing": look up a library's unmatched movies, shows and episodes again.
pub async fn reset_meta_tried_library(pool: &SqlitePool, library_id: i64) -> Result<(), DbError> {
    sqlx::query("UPDATE items SET meta_tried = 0 WHERE library_id = ?1 AND tmdb_id IS NULL")
        .bind(library_id)
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

#[derive(Debug, Clone, Serialize, serde::Deserialize, Default, PartialEq, Eq)]
pub struct CastMember {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Meta {
    pub tmdb_id: Option<i64>,
    pub tvmaze_id: Option<i64>,
    pub title: Option<String>,
    pub year: Option<i64>,
    pub summary: Option<String>,
    pub rating: Option<f64>,
    pub runtime_s: Option<i64>,
    pub poster: Option<String>,
    pub backdrop: Option<String>,
    pub genres: Vec<String>,
    pub cast: Option<Vec<CastMember>>,
}

/// Fields a user can edit and lock. A locked field is never overwritten by a rescan,
/// a TMDB refresh or Fix match.
pub const LOCKABLE: &[&str] = &[
    "title", "year", "summary", "rating", "poster", "backdrop", "genres",
];

fn is_locked(locked: &str, field: &str) -> bool {
    locked.split(',').any(|f| f == field)
}

/// Keeps only known field names, in `LOCKABLE` order, without duplicates.
pub fn normalize_locked<S: AsRef<str>>(fields: &[S]) -> String {
    LOCKABLE
        .iter()
        .filter(|f| fields.iter().any(|g| g.as_ref() == **f))
        .copied()
        .collect::<Vec<_>>()
        .join(",")
}

impl Meta {
    /// Drops every field the item has locked, so saving the rest cannot touch them.
    pub fn without_locked(&self, locked: &str) -> Meta {
        let mut m = self.clone();
        if is_locked(locked, "title") {
            m.title = None;
        }
        if is_locked(locked, "year") {
            m.year = None;
        }
        if is_locked(locked, "summary") {
            m.summary = None;
        }
        if is_locked(locked, "rating") {
            m.rating = None;
        }
        if is_locked(locked, "poster") {
            m.poster = None;
        }
        if is_locked(locked, "backdrop") {
            m.backdrop = None;
        }
        if is_locked(locked, "genres") {
            m.genres.clear();
        }
        m
    }
}

/// SQL that resets a column to NULL unless the row has it locked.
fn reset_unless_locked(field: &str) -> String {
    format!(
        "{field} = CASE WHEN instr(',' || locked_fields || ',', ',{field},') > 0 THEN {field} ELSE NULL END"
    )
}

pub async fn locked_fields(pool: &SqlitePool, id: i64) -> Result<Vec<String>, DbError> {
    let s: Option<String> = sqlx::query_scalar("SELECT locked_fields FROM items WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(s.unwrap_or_default()
        .split(',')
        .filter(|f| !f.is_empty())
        .map(str::to_string)
        .collect())
}

/// Replaces the item's locked set (unknown names are ignored).
pub async fn set_locked_fields<S: AsRef<str>>(
    pool: &SqlitePool,
    id: i64,
    fields: &[S],
) -> Result<(), DbError> {
    sqlx::query("UPDATE items SET locked_fields = ?2 WHERE id = ?1")
        .bind(id)
        .bind(normalize_locked(fields))
        .execute(pool)
        .await?;
    Ok(())
}

/// A manual edit. Every field given is saved and locked.
#[derive(Debug, Clone, Default)]
pub struct Edit {
    pub title: Option<String>,
    pub year: Option<i64>,
    pub summary: Option<String>,
    pub rating: Option<f64>,
}

pub async fn edit_meta(pool: &SqlitePool, id: i64, e: &Edit) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    let mut lock: Vec<&str> = Vec::new();
    if let Some(title) = &e.title {
        sqlx::query("UPDATE items SET title = ?2, sort_title = ?3 WHERE id = ?1")
            .bind(id)
            .bind(title)
            .bind(sort_title(title))
            .execute(&mut *tx)
            .await?;
        lock.push("title");
    }
    if let Some(year) = e.year {
        sqlx::query("UPDATE items SET year = ?2 WHERE id = ?1")
            .bind(id)
            .bind(Some(year).filter(|y| *y > 0))
            .execute(&mut *tx)
            .await?;
        lock.push("year");
    }
    if let Some(summary) = &e.summary {
        sqlx::query("UPDATE items SET summary = ?2 WHERE id = ?1")
            .bind(id)
            .bind(Some(summary.as_str()).filter(|s| !s.trim().is_empty()))
            .execute(&mut *tx)
            .await?;
        lock.push("summary");
    }
    if let Some(rating) = e.rating {
        sqlx::query("UPDATE items SET rating = ?2 WHERE id = ?1")
            .bind(id)
            .bind(Some(rating).filter(|r| *r > 0.0))
            .execute(&mut *tx)
            .await?;
        lock.push("rating");
    }
    let current: String = sqlx::query_scalar("SELECT locked_fields FROM items WHERE id = ?1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .unwrap_or_default();
    let mut all: Vec<&str> = current.split(',').collect();
    all.extend(lock);
    sqlx::query("UPDATE items SET locked_fields = ?2, updated_at = ?3 WHERE id = ?1")
        .bind(id)
        .bind(normalize_locked(&all))
        .bind(now())
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// Fills metadata; `None` fields keep what the scanner found, and locked fields are never touched.
pub async fn save_meta(pool: &SqlitePool, id: i64, m: &Meta) -> Result<(), DbError> {
    let locked: String = sqlx::query_scalar("SELECT locked_fields FROM items WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .unwrap_or_default();
    let m = &m.without_locked(&locked);
    let cast_json = m.cast.as_ref().and_then(|c| serde_json::to_string(c).ok());
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
           cast_json = COALESCE(?11, cast_json),
           tvmaze_id = COALESCE(?13, tvmaze_id),
           meta_tried = 1,
           updated_at = ?12
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
    .bind(cast_json)
    .bind(now())
    .bind(m.tvmaze_id)
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

/// Forgets an item's metadata before a manual re-match, so nothing from the wrong
/// match survives. For a show, seasons and episodes are reset and looked up again.
pub async fn clear_meta(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    // Locked fields survive: the user edited them by hand.
    sqlx::query(
        "DELETE FROM item_genres WHERE item_id = ?1
           AND NOT EXISTS (SELECT 1 FROM items WHERE id = ?1 AND instr(',' || locked_fields || ',', ',genres,') > 0)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(&format!(
        "UPDATE items SET tmdb_id = NULL, tvmaze_id = NULL, {}, {}, {}, {}, {}, runtime_s = NULL, cast_json = NULL WHERE id = ?1",
        reset_unless_locked("year"),
        reset_unless_locked("summary"),
        reset_unless_locked("rating"),
        reset_unless_locked("poster"),
        reset_unless_locked("backdrop"),
    ))
    .bind(id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(&format!(
        "UPDATE items SET meta_tried = 0, runtime_s = NULL, cast_json = NULL, {}, {}, {}, {},
           title = CASE WHEN instr(',' || locked_fields || ',', ',title,') > 0 THEN title
                        WHEN kind = 'episode' THEN 'Episode ' || episode_no ELSE title END,
           sort_title = CASE WHEN instr(',' || locked_fields || ',', ',title,') > 0 THEN sort_title
                             WHEN kind = 'episode' THEN 'episode ' || episode_no ELSE sort_title END
         WHERE parent_id = ?1 OR parent_id IN (SELECT id FROM items WHERE parent_id = ?1)",
        reset_unless_locked("summary"),
        reset_unless_locked("rating"),
        reset_unless_locked("poster"),
        reset_unless_locked("backdrop"),
    ))
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Cast and crew for an item; episodes and seasons inherit from their parent show.
pub async fn item_cast(pool: &SqlitePool, id: i64) -> Result<Vec<CastMember>, DbError> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT COALESCE(i.cast_json, p.cast_json, g.cast_json)
         FROM items i
         LEFT JOIN items p ON p.id = i.parent_id
         LEFT JOIN items g ON g.id = p.parent_id
         WHERE i.id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some((Some(json_str),)) = row else {
        return Ok(Vec::new());
    };
    Ok(serde_json::from_str(&json_str).unwrap_or_default())
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

/// TVmaze ids of shows without a TMDB match whose seasons got new episodes without metadata.
pub async fn tvmaze_shows_with_bare_episodes(
    pool: &SqlitePool,
    library_id: i64,
) -> Result<Vec<(i64, i64)>, DbError> {
    Ok(sqlx::query_as(
        "SELECT DISTINCT sh.id, sh.tvmaze_id FROM items e
         JOIN items se ON se.id = e.parent_id
         JOIN items sh ON sh.id = se.parent_id
         WHERE sh.library_id = ?1 AND sh.tmdb_id IS NULL AND sh.tvmaze_id IS NOT NULL
           AND e.kind = 'episode' AND e.meta_tried = 0",
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

/// Where "Play" on a show or season starts: the first unwatched episode, or the
/// first episode once all are watched (then its `watched` is true).
pub async fn next_up(
    pool: &SqlitePool,
    user_id: i64,
    item_id: i64,
) -> Result<Option<ItemCard>, DbError> {
    let sql = format!(
        "{CARD_SELECT} WHERE i.kind = 'episode'
           AND (i.parent_id = ?2 OR i.parent_id IN (SELECT id FROM items WHERE parent_id = ?2))
           AND EXISTS (SELECT 1 FROM files f WHERE f.item_id = i.id)
         ORDER BY COALESCE(w.watched, 0), p.season_no = 0, p.season_no, i.episode_no
         LIMIT 1"
    );
    Ok(sqlx::query_as(&sql)
        .bind(user_id)
        .bind(item_id)
        .fetch_optional(pool)
        .await?)
}

/// Marks an item and every playable item under it (a show's episodes) watched or not.
pub async fn set_watched_tree(
    pool: &SqlitePool,
    user_id: i64,
    item_id: i64,
    watched: bool,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO watch_state (user_id, item_id, position_s, watched, play_count, last_played_at)
         SELECT ?1, i.id, 0, ?3, ?3, ?4 FROM items i
         WHERE (i.id = ?2 OR i.parent_id = ?2 OR i.parent_id IN (SELECT id FROM items WHERE parent_id = ?2))
           AND EXISTS (SELECT 1 FROM files f WHERE f.item_id = i.id)
         ON CONFLICT(user_id, item_id) DO UPDATE SET
           position_s = 0,
           play_count = play_count + (excluded.watched AND NOT watch_state.watched),
           watched = excluded.watched",
    )
    .bind(user_id)
    .bind(item_id)
    .bind(watched)
    .bind(now())
    .execute(pool)
    .await?;
    Ok(())
}

// ---- series track choice ----

/// A show's audio and subtitle choice: positions among an episode's audio and
/// subtitle streams. `None` follows the server settings; subtitle `-1` is off.
#[derive(Debug, Clone, Copy, Default, Serialize, sqlx::FromRow)]
pub struct SeriesTracks {
    pub audio: Option<i64>,
    pub subtitle: Option<i64>,
}

pub async fn series_tracks(pool: &SqlitePool, show_id: i64) -> Result<SeriesTracks, DbError> {
    Ok(sqlx::query_as(
        "SELECT audio_track AS audio, subtitle_track AS subtitle FROM items WHERE id = ?1",
    )
    .bind(show_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or_default())
}

/// The choice of the show a file's episode belongs to; default for anything else.
pub async fn file_series_tracks(pool: &SqlitePool, file_id: i64) -> Result<SeriesTracks, DbError> {
    Ok(sqlx::query_as(
        "SELECT g.audio_track AS audio, g.subtitle_track AS subtitle
         FROM files f
         JOIN items i ON i.id = f.item_id AND i.kind = 'episode'
         JOIN items p ON p.id = i.parent_id
         JOIN items g ON g.id = p.parent_id AND g.kind = 'show'
         WHERE f.id = ?1",
    )
    .bind(file_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or_default())
}

pub async fn set_series_tracks(
    pool: &SqlitePool,
    show_id: i64,
    tracks: SeriesTracks,
) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE items SET audio_track = ?2, subtitle_track = ?3 WHERE id = ?1 AND kind = 'show'",
    )
    .bind(show_id)
    .bind(tracks.audio)
    .bind(tracks.subtitle)
    .execute(pool)
    .await?;
    Ok(())
}

// ---- a profile's own track choice ----

/// The movie itself, or the show an episode belongs to: where a profile's choice is kept.
const TRACK_ROOT: &str = "CASE WHEN i.kind = 'episode' AND g.kind = 'show' THEN g.id ELSE i.id END";

/// The profile's choice for the title a file belongs to; default when none.
pub async fn file_track_choice(
    pool: &SqlitePool,
    user_id: i64,
    file_id: i64,
) -> Result<SeriesTracks, DbError> {
    let sql = format!(
        "SELECT t.audio, t.subtitle FROM files f
         JOIN items i ON i.id = f.item_id
         LEFT JOIN items p ON p.id = i.parent_id
         LEFT JOIN items g ON g.id = p.parent_id
         JOIN track_choice t ON t.user_id = ?2 AND t.item_id = {TRACK_ROOT}
         WHERE f.id = ?1"
    );
    Ok(sqlx::query_as(&sql)
        .bind(file_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .unwrap_or_default())
}

/// Saves the profile's choice made while playing [item_id], on its movie or show.
pub async fn set_track_choice(
    pool: &SqlitePool,
    user_id: i64,
    item_id: i64,
    tracks: SeriesTracks,
) -> Result<(), DbError> {
    let sql = format!(
        "INSERT INTO track_choice (user_id, item_id, audio, subtitle)
         SELECT ?1, {TRACK_ROOT}, ?3, ?4 FROM items i
         LEFT JOIN items p ON p.id = i.parent_id
         LEFT JOIN items g ON g.id = p.parent_id
         WHERE i.id = ?2
         ON CONFLICT(user_id, item_id) DO UPDATE SET audio = excluded.audio, subtitle = excluded.subtitle"
    );
    sqlx::query(&sql)
        .bind(user_id)
        .bind(item_id)
        .bind(tracks.audio)
        .bind(tracks.subtitle)
        .execute(pool)
        .await?;
    Ok(())
}

// ---- content ratings ----

/// Certification from a metadata lookup. Leaves an admin's manual rating alone. A lookup
/// without one stores "" (looked up, unrated) unless a rating is already known, so the
/// backfill below does not ask again.
pub async fn set_content_rating(
    pool: &SqlitePool,
    item_id: i64,
    certification: Option<&str>,
) -> Result<(), DbError> {
    let c = certification.map(str::trim).unwrap_or_default();
    sqlx::query(
        "UPDATE items SET content_rating = ?2
         WHERE id = ?1 AND content_rating_manual = 0 AND (?2 != '' OR content_rating IS NULL)",
    )
    .bind(item_id)
    .bind(c)
    .execute(pool)
    .await?;
    Ok(())
}

/// Certification of a new match (Fix match): replaces the old match's rating even when the new one has
/// none, since the old rating belongs to a different title. A manual rating stays.
pub async fn replace_content_rating(
    pool: &SqlitePool,
    item_id: i64,
    certification: Option<&str>,
) -> Result<(), DbError> {
    sqlx::query("UPDATE items SET content_rating = ?2 WHERE id = ?1 AND content_rating_manual = 0")
        .bind(item_id)
        .bind(certification.map(str::trim).unwrap_or_default())
        .execute(pool)
        .await?;
    Ok(())
}

/// Items whose poster or backdrop is this image file, at any size ("w300/x.jpg" is "w1280/x.jpg" bigger),
/// to decide who may fetch it. The file name is validated to letters, digits and `._-` by the caller.
pub async fn ids_with_image(pool: &SqlitePool, name: &str) -> Result<Vec<i64>, DbError> {
    Ok(sqlx::query_scalar(
        "SELECT id FROM items WHERE poster LIKE '%/' || ?1 OR backdrop LIKE '%/' || ?1 LIMIT 50",
    )
    .bind(name)
    .fetch_all(pool)
    .await?)
}

/// Matched movies and shows never checked for a certification (matched before ratings
/// were stored): (id, kind, tmdb_id).
pub async fn needing_content_rating(
    pool: &SqlitePool,
    library_id: i64,
    limit: i64,
) -> Result<Vec<(i64, String, i64)>, DbError> {
    Ok(sqlx::query_as(
        "SELECT id, kind, tmdb_id FROM items
         WHERE library_id = ?1 AND parent_id IS NULL AND kind IN ('movie', 'show')
           AND tmdb_id IS NOT NULL AND content_rating IS NULL AND content_rating_manual = 0
         LIMIT ?2",
    )
    .bind(library_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

/// Admin override; None clears it so metadata can fill it again on the next match.
pub async fn set_manual_content_rating(
    pool: &SqlitePool,
    item_id: i64,
    certification: Option<&str>,
) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE items SET content_rating = ?2, content_rating_manual = (?2 IS NOT NULL)
         WHERE id = ?1",
    )
    .bind(item_id)
    .bind(certification)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn pool() -> SqlitePool {
        // One connection: every in-memory connection is its own empty database.
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        crate::run_migrations(&pool).await.unwrap();
        pool
    }

    fn sidecar(path: &str, fp: &str) -> NewStream {
        NewStream {
            idx: 10_000,
            kind: "subtitle".into(),
            codec: Some("srt".into()),
            external_path: Some(path.into()),
            external_fingerprint: Some(fp.into()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn a_new_match_replaces_the_old_rating_but_not_a_manual_one() {
        let db = pool().await;
        let lib = crate::libraries::create(&db, "Movies", "movie", &[])
            .await
            .unwrap();
        let movie = |title| NewItem::Movie { title, year: None };
        let rating = |id| {
            let db = db.clone();
            async move {
                sqlx::query_scalar::<_, Option<String>>(
                    "SELECT content_rating FROM items WHERE id = ?1",
                )
                .bind(id)
                .fetch_one(&db)
                .await
                .unwrap()
            }
        };
        let matched = find_or_create(&db, lib, movie("Wrong")).await.unwrap();
        set_content_rating(&db, matched, Some("R")).await.unwrap();
        // A lookup without a certification keeps what was known; a re-match must not.
        set_content_rating(&db, matched, None).await.unwrap();
        assert_eq!(rating(matched).await.as_deref(), Some("R"));
        replace_content_rating(&db, matched, None).await.unwrap();
        assert_eq!(rating(matched).await.as_deref(), Some(""));

        let manual = find_or_create(&db, lib, movie("Mine")).await.unwrap();
        set_manual_content_rating(&db, manual, Some("PG"))
            .await
            .unwrap();
        replace_content_rating(&db, manual, Some("R"))
            .await
            .unwrap();
        assert_eq!(rating(manual).await.as_deref(), Some("PG"));
    }

    #[tokio::test]
    async fn sidecars_are_replaced_without_touching_probed_streams() {
        let db = pool().await;
        let lib = crate::libraries::create(&db, "Movies", "movie", &[])
            .await
            .unwrap();
        let item = find_or_create(
            &db,
            lib,
            NewItem::Movie {
                title: "Film",
                year: None,
            },
        )
        .await
        .unwrap();
        let file = insert_file(&db, item, "/m/Film.mkv", 10, 1, "10-1")
            .await
            .unwrap();
        let video = NewStream {
            idx: 0,
            kind: "video".into(),
            ..Default::default()
        };
        let pgs = NewStream {
            idx: 2,
            kind: "subtitle".into(),
            codec: Some("hdmv_pgs_subtitle".into()),
            ..Default::default()
        };
        save_probe(
            &db,
            file,
            None,
            Some(60.0),
            None,
            &[video, pgs, sidecar("/m/Film.en.srt", "5-1")],
        )
        .await
        .unwrap();
        assert_eq!(
            library_sidecars(&db, lib).await.unwrap()[&file],
            vec![("/m/Film.en.srt".to_string(), Some("5-1".to_string()))]
        );

        replace_sidecars(&db, file, &[sidecar("/m/Film.fr.srt", "7-2")])
            .await
            .unwrap();
        let all = streams(&db, file).await.unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(
            all.last().and_then(|s| s.external_path.as_deref()),
            Some("/m/Film.fr.srt")
        );

        let subs = library_subtitles(&db, lib).await.unwrap();
        assert_eq!(subs.len(), 2);
        let image = subs.iter().find(|s| s.idx == 2).unwrap();
        set_stream_ocr(&db, image.stream_id, true).await.unwrap();
        assert!(stream(&db, image.stream_id).await.unwrap().unwrap().ocr);
        clear_stream_ocr(&db).await.unwrap();
        assert!(!stream(&db, image.stream_id).await.unwrap().unwrap().ocr);
    }
}
