use crate::auth::{AdminUser, AuthUser};
use crate::error::{ApiError, ApiResult};
use crate::scan;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use kinema_db::items::{self, ItemCard};
use kinema_db::libraries::{self, Library};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/libraries", get(list).post(create))
        .route("/libraries/:id", axum::routing::delete(remove))
        .route("/libraries/:id/roots", axum::routing::put(set_roots))
        .route("/libraries/:id/scan", post(rescan))
        .route("/libraries/:id/thumbnails", post(regenerate_thumbnails))
        .route("/libraries/:id/match", post(match_missing))
        .route("/libraries/:id/items", get(library_items))
        .route("/fs", get(browse))
}

#[derive(Serialize)]
pub struct LibraryView {
    #[serde(flatten)]
    pub library: Library,
    pub scanning: bool,
    pub generating_thumbnails: bool,
}

pub fn view(state: &AppState, library: Library) -> LibraryView {
    LibraryView {
        scanning: scan::is_scanning(state, library.id),
        generating_thumbnails: crate::thumbs::is_running(state, library.id),
        library,
    }
}

async fn list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<Vec<LibraryView>>> {
    let visible = crate::access::visible(&state, &user).await?;
    let mut libs = libraries::list(&state.db).await?;
    libs.retain(|l| crate::access::allows(&visible, l.id));
    Ok(Json(libs.into_iter().map(|l| view(&state, l)).collect()))
}

#[derive(Deserialize)]
struct NewLibrary {
    name: String,
    kind: String,
    paths: Vec<String>,
}

async fn create(
    State(state): State<AppState>,
    _: AdminUser,
    Json(body): Json<NewLibrary>,
) -> ApiResult<(StatusCode, Json<LibraryView>)> {
    let name = body.name.trim();
    if name.is_empty() {
        return Err(ApiError::bad_request("library.name_required"));
    }
    if !["movie", "show", "other"].contains(&body.kind.as_str()) {
        return Err(ApiError::bad_request("library.invalid_kind"));
    }
    if body.paths.iter().all(|p| p.trim().is_empty()) {
        return Err(ApiError::bad_request("library.path_required"));
    }
    let mut roots = Vec::new();
    for p in body.paths.iter().filter(|p| !p.trim().is_empty()) {
        let dir = std::fs::canonicalize(p.trim())
            .ok()
            .filter(|d| d.is_dir())
            .ok_or_else(|| ApiError::bad_request("library.path_invalid"))?;
        if !kinema_core::platform::is_allowed_library_root(&dir) {
            return Err(ApiError::bad_request("library.path_too_broad"));
        }
        roots.push(dir.to_string_lossy().into_owned());
    }
    let id = libraries::create(&state.db, name, &body.kind, &roots).await?;
    scan::watch(&state, id, &roots);
    scan::spawn_scan(state.clone(), id);
    let lib = libraries::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok((StatusCode::CREATED, Json(view(&state, lib))))
}

async fn remove(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    scan::unwatch(&state, id);
    libraries::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct RootOptions {
    roots: Vec<libraries::RootOption>,
}

/// Admin: per-folder scan mode. A polled folder is listed every `poll_interval_min` minutes (1 to
/// 1440) instead of watched for filesystem events, for network shares that send none.
async fn set_roots(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
    Json(body): Json<RootOptions>,
) -> ApiResult<Json<LibraryView>> {
    libraries::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    libraries::set_root_options(&state.db, id, &body.roots).await?;
    let lib = libraries::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if state.settings().watch_changes {
        scan::watch_lib(&state, &lib);
    }
    Ok(Json(view(&state, lib)))
}

async fn rescan(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    libraries::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    scan::spawn_scan(state, id);
    Ok(StatusCode::ACCEPTED)
}

async fn regenerate_thumbnails(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    libraries::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    crate::thumbs::regenerate(state, id).await.map_err(|e| {
        tracing::error!("regenerate thumbnails: {e}");
        ApiError::internal()
    })?;
    Ok(StatusCode::ACCEPTED)
}

/// Looks up unmatched titles again (new key, new language, or TMDB added them since).
async fn match_missing(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    libraries::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    items::reset_meta_tried_library(&state.db, id).await?;
    scan::spawn_scan(state, id);
    Ok(StatusCode::ACCEPTED)
}

async fn library_items(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Vec<ItemCard>>> {
    crate::access::check_library(&state, &user, id).await?;
    Ok(Json(items::library_items(&state.db, user.id, id).await?))
}

#[derive(Deserialize)]
struct BrowseQuery {
    #[serde(default)]
    path: String,
}

#[derive(Serialize)]
struct Dir {
    name: String,
    path: String,
}

#[derive(Serialize)]
struct Listing {
    path: String,
    parent: Option<String>,
    dirs: Vec<Dir>,
}

/// Folder picker for the add-library dialog. Admin only: it reveals the server's tree.
async fn browse(_: AdminUser, Query(q): Query<BrowseQuery>) -> ApiResult<Json<Listing>> {
    if q.path.trim().is_empty() {
        let dirs = kinema_core::platform::filesystem_roots()
            .into_iter()
            .map(|p| Dir {
                name: p.to_string_lossy().into_owned(),
                path: p.to_string_lossy().into_owned(),
            })
            .collect();
        return Ok(Json(Listing {
            path: String::new(),
            parent: None,
            dirs,
        }));
    }
    let listing = tokio::task::spawn_blocking(move || -> Option<Listing> {
        let path = std::fs::canonicalize(PathBuf::from(q.path.trim())).ok()?;
        let mut dirs: Vec<Dir> = std::fs::read_dir(&path)
            .ok()?
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .map(|e| Dir {
                name: e.file_name().to_string_lossy().into_owned(),
                path: e.path().to_string_lossy().into_owned(),
            })
            .filter(|d| !d.name.starts_with('.'))
            .collect();
        dirs.sort_by_key(|d| d.name.to_lowercase());
        Some(Listing {
            parent: Some(
                path.parent()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ),
            path: path.to_string_lossy().into_owned(),
            dirs,
        })
    })
    .await?;
    listing
        .map(Json)
        .ok_or_else(|| ApiError::bad_request("fs.not_found"))
}
