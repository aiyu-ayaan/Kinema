use crate::DbError;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Library {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub item_count: i64,
    /// Top-level shows; the rest of `item_count` are movies and videos.
    pub show_count: i64,
    #[sqlx(skip)]
    pub roots: Vec<String>,
    /// Per-folder scan mode: filesystem events, or polling for network shares.
    #[sqlx(skip)]
    pub root_options: Vec<RootOption>,
}

pub const POLL_DEFAULT_MIN: i64 = 5;
pub const POLL_MIN: i64 = 1;
pub const POLL_MAX: i64 = 24 * 60;

#[derive(Debug, Clone, Serialize, serde::Deserialize, PartialEq, Eq, sqlx::FromRow)]
pub struct RootOption {
    pub path: String,
    /// Poll this folder instead of watching it for filesystem events (NFS, SMB).
    pub poll: bool,
    pub poll_interval_min: i64,
}

impl RootOption {
    pub fn clamped(mut self) -> Self {
        self.poll_interval_min = self.poll_interval_min.clamp(POLL_MIN, POLL_MAX);
        self
    }
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Library>, DbError> {
    let mut libs: Vec<Library> = sqlx::query_as(
        "SELECT l.id, l.name, l.kind,
           (SELECT COUNT(*) FROM items i WHERE i.library_id = l.id AND i.parent_id IS NULL) AS item_count,
           (SELECT COUNT(*) FROM items i WHERE i.library_id = l.id AND i.parent_id IS NULL AND i.kind = 'show') AS show_count
         FROM libraries l ORDER BY l.name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;
    let roots: Vec<(i64, String, bool, i64)> = sqlx::query_as(
        "SELECT library_id, path, poll, poll_interval_min FROM library_roots ORDER BY path",
    )
    .fetch_all(pool)
    .await?;
    for lib in &mut libs {
        for (_, path, poll, poll_interval_min) in roots.iter().filter(|r| r.0 == lib.id) {
            lib.roots.push(path.clone());
            lib.root_options.push(RootOption {
                path: path.clone(),
                poll: *poll,
                poll_interval_min: *poll_interval_min,
            });
        }
    }
    Ok(libs)
}

/// Saves the scan mode of folders the library already has; unknown paths are ignored.
pub async fn set_root_options(
    pool: &SqlitePool,
    library_id: i64,
    options: &[RootOption],
) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    for o in options {
        let o = o.clone().clamped();
        sqlx::query(
            "UPDATE library_roots SET poll = ?3, poll_interval_min = ?4 WHERE library_id = ?1 AND path = ?2",
        )
        .bind(library_id)
        .bind(&o.path)
        .bind(o.poll)
        .bind(o.poll_interval_min)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn get(pool: &SqlitePool, id: i64) -> Result<Option<Library>, DbError> {
    Ok(list(pool).await?.into_iter().find(|l| l.id == id))
}

pub async fn create(
    pool: &SqlitePool,
    name: &str,
    kind: &str,
    roots: &[String],
) -> Result<i64, DbError> {
    let mut tx = pool.begin().await?;
    let id = sqlx::query("INSERT INTO libraries (name, kind) VALUES (?1, ?2)")
        .bind(name)
        .bind(kind)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
    for root in roots {
        sqlx::query("INSERT OR IGNORE INTO library_roots (library_id, path) VALUES (?1, ?2)")
            .bind(id)
            .bind(root)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(id)
}

/// Files point at items with ON DELETE SET NULL, so delete them explicitly first.
pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM files WHERE item_id IN (SELECT id FROM items WHERE library_id = ?1)")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM libraries WHERE id = ?1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
