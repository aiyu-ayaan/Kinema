use crate::auth::AdminUser;
use crate::error::{ApiError, ApiResult};
use crate::scan;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use kinema_db::{items, libraries, settings};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new().route("/settings", get(read).put(write))
}

#[derive(Serialize)]
struct SettingsView {
    tmdb_configured: bool,
    ffmpeg_found: bool,
}

async fn view(state: &AppState) -> Json<SettingsView> {
    Json(SettingsView {
        tmdb_configured: scan::tmdb_key(state).await.is_some(),
        ffmpeg_found: state.bins.is_some(),
    })
}

async fn read(State(state): State<AppState>, _: AdminUser) -> Json<SettingsView> {
    view(&state).await
}

#[derive(Deserialize)]
struct Update {
    tmdb_api_key: String,
}

/// Saves a TMDB key after checking it, then looks up everything that has no match yet.
async fn write(
    State(state): State<AppState>,
    _: AdminUser,
    Json(u): Json<Update>,
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
