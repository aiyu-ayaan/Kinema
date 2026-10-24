pub mod collections;
pub mod error;
pub mod items;
pub mod keyframes;
pub mod libraries;
pub mod markers;
pub mod playlists;
pub mod profiles;
pub mod settings;
pub mod shares;
pub mod shelves;
#[cfg(test)]
mod tests;
pub mod users;

pub use error::DbError;
pub use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

pub async fn init_pool(database_url: &str) -> Result<SqlitePool, DbError> {
    // Configure SQLite for WAL mode and synchronous=NORMAL per architectural design
    let connection_options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        // Reads are the hot path: a bigger page cache (8 MiB per connection, filled only as used), the file mapped
        // into memory, and sort/temp tables kept off disk.
        .pragma("cache_size", "-8000")
        .pragma("mmap_size", "134217728")
        .pragma("temp_store", "MEMORY");

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(connection_options)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

/// Unix seconds; all timestamps in the schema use this.
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
