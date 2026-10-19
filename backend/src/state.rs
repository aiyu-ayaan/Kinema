use kinema_core::Config;
use kinema_media::MediaBinaries;
use notify::RecommendedWatcher;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::{Semaphore, mpsc::UnboundedSender};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SqlitePool,
    /// None when ffmpeg/ffprobe are missing: files are still listed and direct-played.
    pub bins: Option<Arc<MediaBinaries>>,
    /// Running scans; the flag asks for one more pass when it ends.
    pub scans: Arc<Mutex<HashMap<i64, bool>>>,
    pub watchers: Arc<Mutex<HashMap<i64, RecommendedWatcher>>>,
    /// Library ids with filesystem changes, debounced into scans.
    pub changes: UnboundedSender<i64>,
    /// Bounds concurrent ffprobe runs so a big first scan stays light.
    pub probe_slots: Arc<Semaphore>,
    /// Bounds frame grabs, so opening a grid of 30 episodes runs 2 ffmpegs at a time.
    pub thumb_slots: Arc<Semaphore>,
    pub http: reqwest::Client,
}

impl AppState {
    pub fn cache_dir(&self, name: &str) -> PathBuf {
        self.config.data_dir.join("cache").join(name)
    }
}
