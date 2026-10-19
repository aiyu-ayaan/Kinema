use crate::DbError;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Library {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub item_count: i64,
    #[sqlx(skip)]
    pub roots: Vec<String>,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Library>, DbError> {
    let mut libs: Vec<Library> = sqlx::query_as(
        "SELECT l.id, l.name, l.kind,
           (SELECT COUNT(*) FROM items i WHERE i.library_id = l.id AND i.parent_id IS NULL) AS item_count
         FROM libraries l ORDER BY l.name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;
    let roots: Vec<(i64, String)> =
        sqlx::query_as("SELECT library_id, path FROM library_roots ORDER BY path")
            .fetch_all(pool)
            .await?;
    for lib in &mut libs {
        lib.roots = roots
            .iter()
            .filter(|(id, _)| *id == lib.id)
            .map(|(_, p)| p.clone())
            .collect();
    }
    Ok(libs)
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
    sqlx::query(
        "DELETE FROM files WHERE item_id IN (SELECT id FROM items WHERE library_id = ?1)",
    )
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
