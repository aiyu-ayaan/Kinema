use crate::access;
use crate::auth::AuthUser;
use crate::error::{ApiError, ApiResult};
use crate::routes::libraries::{LibraryView, view};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use kinema_db::items::{self, ItemCard, MediaFile, Stream};
use kinema_db::libraries;
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/home", get(home))
        .route("/search", get(search))
        .route("/items/:id", get(detail))
        .route("/items/:id/progress", put(progress))
        .route("/items/:id/watched", put(watched))
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
}

async fn home(State(state): State<AppState>, AuthUser(user): AuthUser) -> ApiResult<Json<Home>> {
    let visible = access::visible(&state, &user).await?;
    let mut shelves = Vec::new();
    for lib in libraries::list(&state.db).await? {
        if !access::allows(&visible, lib.id) {
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
            let items = items::recently_added(&state.db, user.id, lib.id, kind, 24).await?;
            if !items.is_empty() {
                shelves.push(Shelf {
                    library: view(&state, lib.clone()),
                    kind,
                    items,
                });
            }
        }
    }
    Ok(Json(Home {
        continue_watching: items::continue_watching(&state.db, user.id, 20)
            .await?
            .into_iter()
            .filter(|i| access::allows(&visible, i.library_id))
            .collect(),
        shelves,
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
    let visible = access::visible(&state, &user).await?;
    let mut hits = items::search(&state.db, user.id, q, 50).await?;
    hits.retain(|i| access::allows(&visible, i.library_id));
    Ok(Json(hits))
}

#[derive(Serialize)]
struct FileView {
    #[serde(flatten)]
    file: MediaFile,
    streams: Vec<Stream>,
}

#[derive(Serialize)]
struct Detail {
    item: ItemCard,
    genres: Vec<String>,
    files: Vec<FileView>,
    children: Vec<ItemCard>,
    cast: Vec<items::CastMember>,
    /// Previous and next sibling, for episode navigation and autoplay.
    prev_id: Option<i64>,
    next_id: Option<i64>,
    /// The episode "Play" starts on a show or season page.
    next_up: Option<ItemCard>,
}

async fn detail(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Detail>> {
    let db = &state.db;
    let item = items::card(db, user.id, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    access::check_library(&state, &user, item.library_id).await?;
    let mut files = Vec::new();
    for file in items::item_files(db, id).await? {
        let streams = items::streams(db, file.id).await?;
        files.push(FileView { file, streams });
    }
    let (mut prev_id, mut next_id) = (None, None);
    if item.kind == "episode"
        && let Some(parent) = item.parent_id
    {
        let siblings = items::children(db, user.id, parent).await?;
        if let Some(pos) = siblings.iter().position(|s| s.id == id) {
            prev_id = pos.checked_sub(1).map(|p| siblings[p].id);
            next_id = siblings.get(pos + 1).map(|s| s.id);
        }
    }
    let next_up = match item.kind.as_str() {
        "show" | "season" => items::next_up(db, user.id, id).await?,
        _ => None,
    };
    let cast = items::item_cast(db, id).await?;
    Ok(Json(Detail {
        genres: items::genres(db, id).await?,
        children: items::children(db, user.id, id).await?,
        files,
        item,
        cast,
        prev_id,
        next_id,
        next_up,
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
