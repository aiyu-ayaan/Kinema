use kinema_core::Config;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SqlitePool,
}
