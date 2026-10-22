use crate::auth::AdminUser;
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use crate::{scan, thumbs};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Json, Router, extract::State, routing::get, routing::post};
use kinema_db::{items, libraries};
use serde::Serialize;
use std::sync::atomic::Ordering;

#[derive(Serialize)]
pub struct ServerInfoResponse {
    pub name: String,
    pub version: String,
    /// The admin-set address for use away from home; "" when there is none.
    pub public_url: String,
    /// SHA-256 of this server's own TLS certificate ("" without TLS). An app that reached the
    /// server through the public address learns it over that verified channel, so it can
    /// trust the same server's home address without asking.
    pub tls_fingerprint: String,
    pub capabilities: Vec<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/info", get(server_info))
        .route("/status", get(status))
        .route("/cache/:kind/clear", post(clear_cache))
}

/// Public: the sign-in screen shows the server name; apps learn the public address here.
async fn server_info(State(state): State<AppState>) -> Json<ServerInfoResponse> {
    let settings = state.settings();
    Json(ServerInfoResponse {
        name: settings.server_name,
        public_url: settings.public_url,
        tls_fingerprint: state
            .tls_fingerprint
            .as_deref()
            .unwrap_or_default()
            .to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        capabilities: vec![
            "direct_play".to_string(),
            "remux".to_string(),
            "transcode".to_string(),
            "metadata_tmdb".to_string(),
        ],
    })
}

#[derive(Serialize)]
struct LibraryStatus {
    id: i64,
    name: String,
    kind: String,
    titles: i64,
    files: usize,
    thumbnails: usize,
    scanning: bool,
    generating_thumbnails: bool,
}

#[derive(Serialize)]
struct Caches {
    thumbnails: u64,
    subtitles: u64,
    images: u64,
}

#[derive(Serialize)]
struct Status {
    name: String,
    version: String,
    uptime_s: u64,
    ffmpeg: Option<String>,
    ffprobe: Option<String>,
    data_dir: String,
    listen: String,
    lan_urls: Vec<String>,
    /// SHA-256 of the TLS certificate, to compare with what an app shows on first connect.
    tls_fingerprint: Option<String>,
    stats: items::Stats,
    libraries: Vec<LibraryStatus>,
    active_streams: usize,
    active_transcodes: usize,
    caches: Caches,
}

fn dir_size(path: &std::path::Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|e| match e.metadata() {
            Ok(m) if m.is_dir() => dir_size(&e.path()),
            Ok(m) => m.len(),
            Err(_) => 0,
        })
        .sum()
}

async fn status(State(state): State<AppState>, _: AdminUser) -> ApiResult<Json<Status>> {
    let mut libs = Vec::new();
    for lib in libraries::list(&state.db).await? {
        let files = items::library_media_files(&state.db, lib.id).await?;
        let paths: Vec<_> = files.iter().map(|f| thumbs::paths(&state, f).0).collect();
        let thumbnails =
            tokio::task::spawn_blocking(move || paths.iter().filter(|p| p.exists()).count())
                .await?;
        libs.push(LibraryStatus {
            scanning: scan::is_scanning(&state, lib.id),
            generating_thumbnails: thumbs::is_running(&state, lib.id),
            id: lib.id,
            name: lib.name,
            kind: lib.kind,
            titles: lib.item_count,
            files: files.len(),
            thumbnails,
        });
    }
    let cache = |n: &str| state.cache_dir(n);
    let (t, s, i) = (cache("thumbs"), cache("subtitles"), cache("images"));
    let caches = tokio::task::spawn_blocking(move || Caches {
        thumbnails: dir_size(&t),
        subtitles: dir_size(&s),
        images: dir_size(&i),
    })
    .await?;
    let port = state.config.server.port;
    let host = &state.config.server.host;
    let scheme = if state.config.tls.enabled {
        "https"
    } else {
        "http"
    };
    let lan_urls = match (host.as_str(), crate::tls::lan_ip()) {
        ("127.0.0.1" | "localhost" | "::1", _) | (_, None) => Vec::new(),
        (_, Some(ip)) => vec![format!("{scheme}://{ip}:{port}")],
    };
    Ok(Json(Status {
        name: state.settings().server_name,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_s: state.started.elapsed().as_secs(),
        ffmpeg: state.bins.as_ref().map(|b| b.ffmpeg.display().to_string()),
        ffprobe: state.bins.as_ref().map(|b| b.ffprobe.display().to_string()),
        data_dir: state.config.data_dir.display().to_string(),
        listen: format!("{host}:{port}"),
        lan_urls,
        tls_fingerprint: state.tls_fingerprint.as_deref().map(str::to_string),
        stats: items::stats(&state.db).await?,
        libraries: libs,
        active_streams: state.active_streams.load(Ordering::SeqCst),
        active_transcodes: state.active_transcodes.load(Ordering::SeqCst),
        caches,
    }))
}

/// Troubleshooting: drop a cache. Thumbnails are made again in the background.
async fn clear_cache(
    State(state): State<AppState>,
    _: AdminUser,
    Path(kind): Path<String>,
) -> ApiResult<StatusCode> {
    let dir = match kind.as_str() {
        "thumbnails" => "thumbs",
        "subtitles" => "subtitles",
        "images" => "images",
        _ => return Err(ApiError::bad_request("settings.cache_unknown")),
    };
    let path = state.cache_dir(dir);
    if path.exists() {
        tokio::fs::remove_dir_all(&path).await?;
    }
    if dir == "thumbs" {
        // Scrub sheets are thumbnails too; they come back on the next scrub.
        let _ = tokio::fs::remove_dir_all(state.cache_dir("sprites")).await;
        for lib in libraries::list(&state.db).await? {
            thumbs::spawn(state.clone(), lib.id);
        }
    }
    Ok(StatusCode::NO_CONTENT)
}
