use crate::auth::AdminUser;
use crate::error::{ApiError, ApiResult};
use crate::server_settings::{self, ServerSettings};
use crate::state::AppState;
use crate::{scan, subtitles, thumbs};
use axum::extract::State;
use axum::routing::{get, put};
use axum::{Json, Router};
use kinema_db::{items, libraries, settings};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/settings", get(read))
        .route("/settings/server", put(write_server))
        .route("/settings/tmdb", put(write_tmdb))
}

/// The allowed values, so the UI never offers a choice the server would reject.
#[derive(Serialize)]
struct Options {
    presets: &'static [&'static str],
    max_heights: &'static [u32],
    threads: &'static [u32],
    subtitle_modes: &'static [&'static str],
    metadata_languages: &'static [&'static str],
    track_languages: Vec<&'static str>,
}

#[derive(Serialize)]
struct SettingsView {
    server: ServerSettings,
    tmdb_configured: bool,
    ffmpeg_found: bool,
    /// The GPU encoder found at startup, e.g. "NVENC".
    hardware_encoder: Option<&'static str>,
    /// Language models of the OCR tool found at startup; None without `tesseract`.
    ocr_languages: Option<Vec<String>>,
    options: Options,
}

async fn view(state: &AppState) -> Json<SettingsView> {
    Json(SettingsView {
        server: state.settings(),
        tmdb_configured: scan::tmdb_key(state).await.is_some(),
        ffmpeg_found: state.bins.is_some(),
        hardware_encoder: state
            .bins
            .as_ref()
            .and_then(|b| b.hw_encoder.as_ref())
            .map(|e| e.name()),
        ocr_languages: state.ocr.as_ref().map(|t| t.languages.clone()),
        options: Options {
            presets: server_settings::PRESETS,
            max_heights: server_settings::MAX_HEIGHTS,
            threads: server_settings::THREADS,
            subtitle_modes: server_settings::SUBTITLE_MODES,
            metadata_languages: server_settings::METADATA_LANGUAGES,
            track_languages: server_settings::TRACK_LANGUAGES
                .iter()
                .map(|(c, _)| *c)
                .collect(),
        },
    })
}

async fn read(State(state): State<AppState>, _: AdminUser) -> Json<SettingsView> {
    view(&state).await
}

/// Saves the server options and applies the ones that need more than a stored value.
async fn write_server(
    State(state): State<AppState>,
    _: AdminUser,
    Json(new): Json<ServerSettings>,
) -> ApiResult<Json<SettingsView>> {
    let new = new.sanitized();
    let old = state.settings();
    server_settings::save(&state.db, &new).await?;
    if let Ok(mut s) = state.settings.write() {
        *s = new.clone();
    }
    if old.watch_changes != new.watch_changes {
        scan::rewatch_all(&state).await;
    }
    if new.subtitle_ocr && !old.subtitle_ocr && state.ocr.is_some() {
        for lib in libraries::list(&state.db).await? {
            subtitles::spawn(state.clone(), lib.id);
        }
    }
    if new.thumbnails && (!old.thumbnails || old.thumbnail_position != new.thumbnail_position) {
        for lib in libraries::list(&state.db).await? {
            thumbs::spawn(state.clone(), lib.id);
        }
    }
    Ok(view(&state).await)
}

#[derive(Deserialize)]
struct TmdbUpdate {
    tmdb_api_key: String,
}

/// Saves a TMDB key after checking it, then looks up everything that has no match yet.
async fn write_tmdb(
    State(state): State<AppState>,
    _: AdminUser,
    Json(u): Json<TmdbUpdate>,
) -> ApiResult<Json<SettingsView>> {
    let key = u.tmdb_api_key.trim();
    if !key.is_empty() {
        kinema_meta::TmdbClient::new(key.to_string())
            .verify()
            .await
            .map_err(|_| ApiError::bad_request("settings.tmdb_invalid"))?;
    }
    settings::set(&state.db, settings::TMDB_API_KEY, key).await?;
    if !key.is_empty() {
        items::reset_meta_tried(&state.db).await?;
        for lib in libraries::list(&state.db).await? {
            scan::spawn_scan(state.clone(), lib.id);
        }
    }
    Ok(view(&state).await)
}
