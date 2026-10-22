//! Playlists: personal, ordered lists of playable items (movies, episodes, videos).
//! Only the owner edits. A shared playlist is read-only for other profiles, and every viewer
//! sees only the items from libraries they may open.

use crate::access;
use crate::auth::AuthUser;
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use kinema_db::items::{self, ItemCard};
use kinema_db::playlists::{self, Playlist};
use kinema_db::users::User;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const MAX_ITEMS: usize = 5000;
const PLAYABLE: &[&str] = &["movie", "episode", "video"];

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/playlists", get(list).post(create))
        .route("/playlists/:id", get(detail).patch(update).delete(remove))
        .route("/playlists/:id/items", put(replace).post(append))
}

#[derive(Serialize)]
pub struct PlaylistSummary {
    #[serde(flatten)]
    playlist: Playlist,
    mine: bool,
    /// Items this viewer may see.
    item_count: usize,
    /// Art of the first visible item, as the playlist's cover.
    poster: Option<String>,
}

#[derive(Serialize)]
struct PlaylistDetail {
    #[serde(flatten)]
    summary: PlaylistSummary,
    items: Vec<ItemCard>,
}

fn visible_items(cards: Vec<ItemCard>, visible: &Option<HashSet<i64>>) -> Vec<ItemCard> {
    cards
        .into_iter()
        .filter(|c| access::allows(visible, c.library_id))
        .collect()
}

fn summary(playlist: Playlist, user: &User, items: &[ItemCard]) -> PlaylistSummary {
    PlaylistSummary {
        mine: playlist.owner_id == user.id,
        item_count: items.len(),
        poster: items.iter().find_map(|i| i.poster.clone()),
        playlist,
    }
}

/// The profile's own playlists and the shared ones, for the playlists page and the Home row.
pub async fn summaries(
    state: &AppState,
    user: &User,
    visible: &Option<HashSet<i64>>,
) -> ApiResult<Vec<PlaylistSummary>> {
    let mut out = Vec::new();
    for p in playlists::list_visible(&state.db, user.id).await? {
        let cards = visible_items(playlists::items(&state.db, user.id, p.id).await?, visible);
        out.push(summary(p, user, &cards));
    }
    Ok(out)
}

async fn list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<Vec<PlaylistSummary>>> {
    let visible = access::visible(&state, &user).await?;
    Ok(Json(summaries(&state, &user, &visible).await?))
}

/// A playlist the profile may see, or 404 as if it did not exist.
async fn visible_playlist(state: &AppState, user: &User, id: i64) -> ApiResult<Playlist> {
    playlists::get(&state.db, id)
        .await?
        .filter(|p| p.visible_to(user.id))
        .ok_or_else(ApiError::not_found)
}

/// A playlist the profile owns; 403 when it can only read it.
async fn owned_playlist(state: &AppState, user: &User, id: i64) -> ApiResult<Playlist> {
    let p = visible_playlist(state, user, id).await?;
    if p.editable_by(user.id) {
        Ok(p)
    } else {
        Err(ApiError::new(StatusCode::FORBIDDEN, "playlists.not_owner"))
    }
}

/// Every id must be a playable item in a library the profile can open.
async fn check_playable(state: &AppState, user: &User, ids: &[i64]) -> ApiResult<()> {
    if ids.len() > MAX_ITEMS {
        return Err(ApiError::bad_request("playlists.too_many"));
    }
    let visible = access::visible(state, user).await?;
    for id in ids {
        let card = items::card(&state.db, user.id, *id)
            .await?
            .filter(|c| access::allows(&visible, c.library_id))
            .ok_or_else(ApiError::not_found)?;
        if !PLAYABLE.contains(&card.kind.as_str()) {
            return Err(ApiError::bad_request("playlists.item_unplayable"));
        }
    }
    Ok(())
}

fn clean_name(name: &str) -> ApiResult<&str> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(ApiError::bad_request("playlists.name_invalid"));
    }
    Ok(name)
}

#[derive(Deserialize)]
struct NewPlaylist {
    name: String,
    #[serde(default)]
    item_ids: Vec<i64>,
}

async fn create(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<NewPlaylist>,
) -> ApiResult<(StatusCode, Json<PlaylistDetail>)> {
    let name = clean_name(&body.name)?;
    check_playable(&state, &user, &body.item_ids).await?;
    let id = playlists::create(&state.db, user.id, name).await?;
    playlists::append(&state.db, id, &body.item_ids).await?;
    let detail = detail_of(&state, &user, id).await?;
    Ok((StatusCode::CREATED, Json(detail)))
}

async fn detail_of(state: &AppState, user: &User, id: i64) -> ApiResult<PlaylistDetail> {
    let playlist = visible_playlist(state, user, id).await?;
    let visible = access::visible(state, user).await?;
    let items = visible_items(playlists::items(&state.db, user.id, id).await?, &visible);
    Ok(PlaylistDetail {
        summary: summary(playlist, user, &items),
        items,
    })
}

async fn detail(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<PlaylistDetail>> {
    Ok(Json(detail_of(&state, &user, id).await?))
}

#[derive(Deserialize)]
struct Patch {
    name: Option<String>,
    is_shared: Option<bool>,
}

async fn update(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Patch>,
) -> ApiResult<Json<PlaylistDetail>> {
    owned_playlist(&state, &user, id).await?;
    let name = body.name.as_deref().map(clean_name).transpose()?;
    playlists::update(&state.db, id, name, body.is_shared).await?;
    Ok(Json(detail_of(&state, &user, id).await?))
}

async fn remove(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    owned_playlist(&state, &user, id).await?;
    playlists::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct Ids {
    item_ids: Vec<i64>,
}

/// Replaces the whole list and order (drag and drop, removal).
async fn replace(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Ids>,
) -> ApiResult<Json<PlaylistDetail>> {
    owned_playlist(&state, &user, id).await?;
    check_playable(&state, &user, &body.item_ids).await?;
    playlists::set_items(&state.db, id, &body.item_ids).await?;
    Ok(Json(detail_of(&state, &user, id).await?))
}

async fn append(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Ids>,
) -> ApiResult<Json<PlaylistDetail>> {
    owned_playlist(&state, &user, id).await?;
    check_playable(&state, &user, &body.item_ids).await?;
    let have = playlists::item_ids(&state.db, id).await?.len();
    if have + body.item_ids.len() > MAX_ITEMS {
        return Err(ApiError::bad_request("playlists.too_many"));
    }
    playlists::append(&state.db, id, &body.item_ids).await?;
    Ok(Json(detail_of(&state, &user, id).await?))
}
