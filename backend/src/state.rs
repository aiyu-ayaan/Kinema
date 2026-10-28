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
use tokio::sync::{AcquireError, Semaphore, SemaphorePermit, mpsc::UnboundedSender};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SqlitePool,
    /// None when ffmpeg/ffprobe are missing: files are still listed and direct-played.
    pub bins: Option<Arc<MediaBinaries>>,
    pub settings: Arc<RwLock<ServerSettings>>,
    pub scans: Jobs,
    pub thumb_jobs: Jobs,
    pub marker_jobs: Jobs,
    pub subtitle_jobs: Jobs,
    /// `tesseract` for picture subtitles, found at startup; None leaves them picture-only.
    pub ocr: Option<Arc<kinema_media::ocr::OcrTool>>,
    pub keyframe_jobs: Jobs,
    /// Live HLS sessions (remux and transcode segments).
    pub hls: crate::hls::HlsSessions,
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
    /// Failed sign-ins per profile: (count, last failure), for PIN lockout.
    pub login_failures: Arc<Mutex<HashMap<i64, (u32, Instant)>>>,
    /// Who is playing what right now, from progress reports; see `routes/activity.rs`.
    pub now_playing: crate::routes::activity::NowPlayingMap,
    pub http: reqwest::Client,
    /// SHA-256 of the TLS certificate; apps show it so the owner can compare. None without TLS.
    pub tls_fingerprint: Option<Arc<str>>,
}

impl AppState {
    pub fn cache_dir(&self, name: &str) -> PathBuf {
        self.config.data_dir.join("cache").join(name)
    }

    /// What clients may request now; see `boot.rs`.
    pub fn features(&self) -> crate::boot::Features {
        crate::boot::Features::new(self.bins.as_deref(), &self.settings())
    }

    pub fn settings(&self) -> ServerSettings {
        self.settings.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// A slot of `slots` for background work (probe, thumbnails, markers, subtitles), held back
    /// while a re-encode runs or the machine is busy so playback keeps the disk and the cores.
    /// The wait is capped: a box that is always busy still gets its library scanned.
    pub async fn background_slot<'a>(
        &self,
        slots: &'a Semaphore,
    ) -> Result<SemaphorePermit<'a>, AcquireError> {
        // Taken first, so only the few slot holders poll, not every queued file.
        let permit = slots.acquire().await?;
        let started = Instant::now();
        while started.elapsed() < BACKGROUND_MAX_WAIT && self.playback_busy() {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
        Ok(permit)
    }

    fn playback_busy(&self) -> bool {
        self.active_transcodes.load(Ordering::SeqCst) > 0
            || kinema_core::platform::load_per_core().is_some_and(|l| l > 0.8)
    }
}

const BACKGROUND_MAX_WAIT: std::time::Duration = std::time::Duration::from_secs(600);

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
