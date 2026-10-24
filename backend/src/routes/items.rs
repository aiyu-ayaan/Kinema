use crate::access;
use crate::auth::{AdminUser, AuthUser};
use crate::error::{ApiError, ApiResult};
use crate::routes::collections::{self, CollectionSummary, GenreShelf};
use crate::routes::libraries::{LibraryView, view};
use crate::routes::playlists::{self, PlaylistSummary};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use kinema_db::items::{self, ItemCard, MediaFile, Stream};
use kinema_db::libraries;
use kinema_db::markers::{self, Marker};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/home", get(home))
        .route("/search", get(search))
        .route("/items/:id", get(detail))
        .route("/items/:id/progress", put(progress))
        .route("/items/:id/watched", put(watched))
        .route("/items/:id/tracks", put(set_tracks))
        .route("/items/:id/track-choice", put(set_track_choice))
        .route("/items/:id/content-rating", put(set_content_rating))
        .route(
            "/items/:id/markers/:kind",
            put(set_marker).delete(clear_marker),
        )
}

#[derive(Serialize)]
struct Shelf {
    library: LibraryView,
    kind: items::Shelf,
    items: Vec<ItemCard>,
}

#[derive(Serialize)]
struct Home {
    continue_watching: Vec<ItemCard>,
    shelves: Vec<Shelf>,
    /// The profile's own playlists and shared ones.
    playlists: Vec<PlaylistSummary>,
    collections: Vec<CollectionSummary>,
    /// Admin-defined genre rows ("Oh, The Horror!") the profile has not hidden.
    genre_shelves: Vec<GenreShelf>,
}

async fn home(State(state): State<AppState>, AuthUser(user): AuthUser) -> ApiResult<Json<Home>> {
    let policy = access::policy(&state, &user).await?;
    let mut shelves = Vec::new();
    for lib in libraries::list(&state.db).await? {
        if !policy.allows_library(lib.id) {
            continue;
        }
        let movies = lib.item_count > lib.show_count;
        let shows = lib.show_count > 0;
        let wanted = [
            (items::Shelf::Movies, movies),
            (items::Shelf::Episodes, shows),
            (items::Shelf::Shows, shows),
        ];
        for (kind, on) in wanted {
            if !on {
                continue;
            }
            let mut items = items::recently_added(&state.db, user.id, lib.id, kind, 24).await?;
            policy.retain(&mut items);
            if !items.is_empty() {
                shelves.push(Shelf {
                    library: view(&state, lib.clone()),
                    kind,
                    items,
                });
            }
        }
    }
    let mut continue_watching = items::continue_watching(&state.db, user.id, 20).await?;
    policy.retain(&mut continue_watching);
    Ok(Json(Home {
        continue_watching,
        shelves,
        playlists: playlists::summaries(&state, &user, &policy).await?,
        collections: collections::summaries(&state, &user, &policy).await?,
        genre_shelves: collections::home_shelves(&state, &user, &policy).await?,
    }))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

async fn search(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(q): Query<SearchQuery>,
) -> ApiResult<Json<Vec<ItemCard>>> {
    let q = q.q.trim();
    if q.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let mut hits = items::search(&state.db, user.id, q, 50).await?;
    access::policy(&state, &user).await?.retain(&mut hits);
    Ok(Json(hits))
}

#[derive(Serialize)]
struct FileView {
    #[serde(flatten)]
    file: MediaFile,
    streams: Vec<Stream>,
    /// Intro and end credits, for Skip intro and the up-next card.
    markers: Vec<Marker>,
}

#[derive(Serialize)]
struct Detail {
    item: ItemCard,
    genres: Vec<String>,
    /// Fields an admin edited by hand; scans and refreshes never overwrite them.
    locked_fields: Vec<String>,
    /// Franchise collections the title belongs to.
    collections: Vec<kinema_db::collections::Collection>,
    files: Vec<FileView>,
    /// What the server will serve right now, so clients skip requests it would refuse (see `boot.rs`).
    features: crate::boot::Features,
    children: Vec<ItemCard>,
    cast: Vec<items::CastMember>,
    /// Previous and next sibling, for episode navigation and autoplay.
    prev_id: Option<i64>,
    next_id: Option<i64>,
    /// The episode "Play" starts on a show or season page.
    next_up: Option<ItemCard>,
    /// Show only: its audio and subtitle choice, and the first episode's streams to pick from.
    tracks: Option<TracksView>,
}

#[derive(Serialize)]
struct TracksView {
    #[serde(flatten)]
    chosen: items::SeriesTracks,
    streams: Vec<Stream>,
}

async fn detail(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Detail>> {
    let db = &state.db;
    let policy = access::policy(&state, &user).await?;
    let item = items::card(db, user.id, id)
        .await?
        .filter(|c| policy.allows(c))
        .ok_or_else(ApiError::not_found)?;
    let mut files = Vec::new();
    for file in items::item_files(db, id).await? {
        let streams = items::streams(db, file.id).await?;
        let markers = markers::for_file(db, file.id).await?;
        files.push(FileView {
            file,
            streams,
            markers,
        });
    }
    let (mut prev_id, mut next_id) = (None, None);
    if item.kind == "episode"
        && let Some(parent) = item.parent_id
    {
        let mut siblings = items::children(db, user.id, parent).await?;
        policy.retain(&mut siblings);
        if let Some(pos) = siblings.iter().position(|s| s.id == id) {
            prev_id = pos.checked_sub(1).map(|p| siblings[p].id);
            next_id = siblings.get(pos + 1).map(|s| s.id);
        }
    }
    let next_up = match item.kind.as_str() {
        "show" | "season" => items::next_up(db, user.id, id)
            .await?
            .filter(|c| policy.allows(c)),
        _ => None,
    };
    let tracks = if item.kind == "show" {
        let streams = match items::thumb_file(db, id).await? {
            Some(f) => items::streams(db, f.id).await?,
            None => Vec::new(),
        };
        Some(TracksView {
            chosen: items::series_tracks(db, id).await?,
            streams,
        })
    } else {
        None
    };
    let cast = items::item_cast(db, id).await?;
    let mut children = items::children(db, user.id, id).await?;
    policy.retain(&mut children);
    Ok(Json(Detail {
        genres: items::genres(db, id).await?,
        locked_fields: items::locked_fields(db, id).await?,
        collections: kinema_db::collections::of_item(db, id).await?,
        children,
        files,
        features: state.features(),
        item,
        cast,
        prev_id,
        next_id,
        next_up,
        tracks,
    }))
}

#[derive(Deserialize)]
struct Progress {
    position_s: f64,
    #[serde(default)]
    watched: bool,
}

async fn progress(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Json(p): Json<Progress>,
) -> ApiResult<StatusCode> {
    access::check_item(&state, &user, id).await?;
    items::set_progress(&state.db, user.id, id, p.position_s.max(0.0), p.watched).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct Watched {
    watched: bool,
}

async fn watched(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Json(w): Json<Watched>,
) -> ApiResult<StatusCode> {
    access::check_item(&state, &user, id).await?;
    items::set_watched_tree(&state.db, user.id, id, w.watched).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct TracksBody {
    audio: Option<i64>,
    subtitle: Option<i64>,
}

async fn set_tracks(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
    Json(b): Json<TracksBody>,
) -> ApiResult<StatusCode> {
    if items::kind_of(&state.db, id).await?.as_deref() != Some("show") {
        return Err(ApiError::not_found());
    }
    let tracks = items::SeriesTracks {
        audio: b.audio.filter(|&n| n >= 0),
        subtitle: b.subtitle.filter(|&n| n >= -1),
    };
    items::set_series_tracks(&state.db, id, tracks).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The viewer picked an audio or subtitle track: remember it for this profile on every device.
async fn set_track_choice(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Json(b): Json<TracksBody>,
) -> ApiResult<StatusCode> {
    access::check_item(&state, &user, id).await?;
    let tracks = items::SeriesTracks {
        audio: b.audio.filter(|&n| n >= 0),
        subtitle: b.subtitle.filter(|&n| n >= -1),
    };
    items::set_track_choice(&state.db, user.id, id, tracks).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct MarkerBody {
    start_s: f64,
    end_s: f64,
}

/// The title's main file (the one players open) and the marker kind, or 404.
async fn marker_target(state: &AppState, id: i64, kind: &str) -> ApiResult<MediaFile> {
    if kind != "intro" && kind != "credits" {
        return Err(ApiError::not_found());
    }
    items::item_files(&state.db, id)
        .await?
        .into_iter()
        .next()
        .ok_or_else(ApiError::not_found)
}

/// Admin: sets where the intro or the credits are; detection never replaces it.
async fn set_marker(
    State(state): State<AppState>,
    _: AdminUser,
    Path((id, kind)): Path<(i64, String)>,
    Json(b): Json<MarkerBody>,
) -> ApiResult<StatusCode> {
    let file = marker_target(&state, id, &kind).await?;
    let fits = b.start_s >= 0.0
        && b.end_s > b.start_s
        && file.duration_s.is_none_or(|d| b.end_s <= d + 1.0);
    if !fits {
        return Err(ApiError::bad_request("markers.invalid"));
    }
    markers::set_manual(&state.db, file.id, &kind, Some((b.start_s, b.end_s))).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct ContentRatingBody {
    /// "PG-13", "TV-14", "16"...; null or "" clears the override.
    content_rating: Option<String>,
}

/// Admin: sets a title's certification by hand (for files TMDB does not rate). It is kept
/// over metadata and applies to every season and episode that has none of its own.
async fn set_content_rating(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
    Json(b): Json<ContentRatingBody>,
) -> ApiResult<StatusCode> {
    items::kind_of(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let rating = b
        .content_rating
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty());
    let valid = |r: &str| {
        r.len() <= 16
            && r.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '+' | ' '))
    };
    if rating.is_some_and(|r| !valid(r)) {
        return Err(ApiError::bad_request("items.content_rating_invalid"));
    }
    items::set_manual_content_rating(&state.db, id, rating).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Admin: drops a marker and looks for it again.
async fn clear_marker(
    State(state): State<AppState>,
    _: AdminUser,
    Path((id, kind)): Path<(i64, String)>,
) -> ApiResult<StatusCode> {
    let file = marker_target(&state, id, &kind).await?;
    markers::set_manual(&state.db, file.id, &kind, None).await?;
    if let Some(card) = items::card(&state.db, 0, id).await? {
        crate::markers::spawn(state, card.library_id);
    }
    Ok(StatusCode::NO_CONTENT)
}
