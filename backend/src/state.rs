use crate::jobs::Jobs;
use crate::server_settings::ServerSettings;
use kinema_core::Config;
use kinema_media::MediaBinaries;
use notify::RecommendedWatcher;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use tokio::sync::{Semaphore, mpsc::UnboundedSender};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SqlitePool,
    /// None when ffmpeg/ffprobe are missing: files are still listed and direct-played.
    pub bins: Option<Arc<MediaBinaries>>,
    pub settings: Arc<RwLock<ServerSettings>>,
    pub scans: Jobs,
    pub thumb_jobs: Jobs,
    pub watchers: Arc<Mutex<HashMap<i64, RecommendedWatcher>>>,
    /// Library ids with filesystem changes, debounced into scans.
    pub changes: UnboundedSender<i64>,
    /// Bounds concurrent ffprobe runs so a big first scan stays light.
    pub probe_slots: Arc<Semaphore>,
    /// One background frame grab at a time keeps thumbnail generation near idle cost.
    pub thumb_slots: Arc<Semaphore>,
    /// ffmpeg pipes currently streaming (remux + transcode), and the re-encoding subset.
    pub active_streams: Arc<AtomicUsize>,
    pub active_transcodes: Arc<AtomicUsize>,
    pub started: Instant,
    pub http: reqwest::Client,
}

impl AppState {
    pub fn cache_dir(&self, name: &str) -> PathBuf {
        self.config.data_dir.join("cache").join(name)
    }

    pub fn settings(&self) -> ServerSettings {
        self.settings.read().map(|s| s.clone()).unwrap_or_default()
    }
}

/// Counts a running stream for as long as it lives.
pub struct Counted(Arc<AtomicUsize>);

impl Counted {
    pub fn new(counter: &Arc<AtomicUsize>) -> Self {
        counter.fetch_add(1, Ordering::SeqCst);
        Self(counter.clone())
    }
}

impl Drop for Counted {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
