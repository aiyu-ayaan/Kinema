//! Franchise collections and the genre shelves on Home. Admins define both; every viewer sees
//! only the titles from libraries they may open, and a profile can hide any shelf for itself.

use crate::access;
use crate::auth::{AdminUser, AuthUser};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use kinema_db::collections::{self, Collection};
use kinema_db::items::{self, ItemCard};
use kinema_db::shelves::{self, Shelf};
use kinema_db::users::User;
use serde::{Deserialize, Serialize};

const TOP_LEVEL: &[&str] = &["movie", "show", "video"];
const SHELF_ITEMS: i64 = 24;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/collections", get(list).post(create))
        .route("/collections/:id", get(detail).patch(rename).delete(remove))
        .route("/collections/:id/items", put(set_items))
        .route("/shelves", get(list_shelves).post(create_shelf))
        .route(
            "/shelves/:id",
            axum::routing::patch(update_shelf).delete(delete_shelf),
        )
        .route("/shelves/:id/hidden", put(set_hidden))
        .route("/genres", get(genres))
}

#[derive(Serialize)]
pub struct CollectionSummary {
    #[serde(flatten)]
    collection: Collection,
    item_count: usize,
    poster: Option<String>,
}

#[derive(Serialize)]
struct CollectionDetail {
    #[serde(flatten)]
    summary: CollectionSummary,
    items: Vec<ItemCard>,
}

fn summarize(collection: Collection, items: &[ItemCard]) -> CollectionSummary {
    CollectionSummary {
        collection,
        item_count: items.len(),
        poster: items.iter().find_map(|i| i.poster.clone()),
    }
}

async fn visible_members(
    state: &AppState,
    user: &User,
    visible: &access::Policy,
    id: i64,
) -> ApiResult<Vec<ItemCard>> {
    let mut cards = collections::items(&state.db, user.id, id).await?;
    cards.retain(|c| visible.allows(c));
    Ok(cards)
}

/// Collections with at least one title the profile can open. Admins also see empty ones, to manage them.
pub async fn summaries(
    state: &AppState,
    user: &User,
    visible: &access::Policy,
) -> ApiResult<Vec<CollectionSummary>> {
    let mut out = Vec::new();
    for c in collections::list(&state.db).await? {
        let cards = visible_members(state, user, visible, c.id).await?;
        if !cards.is_empty() || user.is_admin {
            out.push(summarize(c, &cards));
        }
    }
    Ok(out)
}

async fn list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<Vec<CollectionSummary>>> {
    let visible = access::policy(&state, &user).await?;
    Ok(Json(summaries(&state, &user, &visible).await?))
}

async fn detail_of(state: &AppState, user: &User, id: i64) -> ApiResult<CollectionDetail> {
    let collection = collections::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let visible = access::policy(state, user).await?;
    let items = visible_members(state, user, &visible, id).await?;
    if items.is_empty() && !user.is_admin {
        return Err(ApiError::not_found());
    }
    Ok(CollectionDetail {
        summary: summarize(collection, &items),
        items,
    })
}

async fn detail(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<CollectionDetail>> {
    Ok(Json(detail_of(&state, &user, id).await?))
}

fn clean(text: &str, code: &'static str) -> ApiResult<String> {
    let text = text.trim();
    if text.is_empty() || text.chars().count() > 80 {
        return Err(ApiError::bad_request(code));
    }
    Ok(text.to_string())
}

async fn check_members(state: &AppState, ids: &[i64]) -> ApiResult<()> {
    for id in ids {
        let kind = items::kind_of(&state.db, *id)
            .await?
            .ok_or_else(ApiError::not_found)?;
        if !TOP_LEVEL.contains(&kind.as_str()) {
            return Err(ApiError::bad_request("collections.item_invalid"));
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct NewCollection {
    name: String,
    #[serde(default)]
    item_ids: Vec<i64>,
}

async fn create(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    Json(body): Json<NewCollection>,
) -> ApiResult<(StatusCode, Json<CollectionDetail>)> {
    let name = clean(&body.name, "collections.name_invalid")?;
    check_members(&state, &body.item_ids).await?;
    let id = collections::create(&state.db, &name, None).await?;
    collections::set_items(&state.db, id, &body.item_ids).await?;
    Ok((
        StatusCode::CREATED,
        Json(detail_of(&state, &user, id).await?),
    ))
}

#[derive(Deserialize)]
struct Name {
    name: String,
}

async fn rename(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    Path(id): Path<i64>,
    Json(body): Json<Name>,
) -> ApiResult<Json<CollectionDetail>> {
    let name = clean(&body.name, "collections.name_invalid")?;
    collections::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    collections::rename(&state.db, id, &name).await?;
    Ok(Json(detail_of(&state, &user, id).await?))
}

async fn remove(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    collections::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct Ids {
    item_ids: Vec<i64>,
}

async fn set_items(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    Path(id): Path<i64>,
    Json(body): Json<Ids>,
) -> ApiResult<Json<CollectionDetail>> {
    collections::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    check_members(&state, &body.item_ids).await?;
    collections::set_items(&state.db, id, &body.item_ids).await?;
    Ok(Json(detail_of(&state, &user, id).await?))
}

// ---- shelves ----

#[derive(Serialize)]
struct ShelfView {
    #[serde(flatten)]
    shelf: Shelf,
    hidden: bool,
}

/// Every shelf with this profile's hidden flag, so a hidden one can be brought back.
async fn list_shelves(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<Vec<ShelfView>>> {
    let hidden = shelves::hidden_by(&state.db, user.id).await?;
    Ok(Json(
        shelves::list(&state.db)
            .await?
            .into_iter()
            .map(|shelf| ShelfView {
                hidden: hidden.contains(&shelf.id),
                shelf,
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
struct NewShelf {
    title: String,
    genre: String,
}

async fn create_shelf(
    State(state): State<AppState>,
    _: AdminUser,
    Json(body): Json<NewShelf>,
) -> ApiResult<(StatusCode, Json<Shelf>)> {
    let title = clean(&body.title, "shelves.title_invalid")?;
    let genre = clean(&body.genre, "shelves.genre_invalid")?;
    let id = shelves::create(&state.db, &title, &genre).await?;
    let shelf = shelves::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::internal)?;
    Ok((StatusCode::CREATED, Json(shelf)))
}

#[derive(Deserialize)]
struct ShelfPatch {
    title: Option<String>,
    genre: Option<String>,
}

async fn update_shelf(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
    Json(body): Json<ShelfPatch>,
) -> ApiResult<Json<Shelf>> {
    shelves::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let title = body
        .title
        .as_deref()
        .map(|t| clean(t, "shelves.title_invalid"))
        .transpose()?;
    let genre = body
        .genre
        .as_deref()
        .map(|g| clean(g, "shelves.genre_invalid"))
        .transpose()?;
    shelves::update(&state.db, id, title.as_deref(), genre.as_deref()).await?;
    Ok(Json(
        shelves::get(&state.db, id)
            .await?
            .ok_or_else(ApiError::not_found)?,
    ))
}

async fn delete_shelf(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    shelves::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct Hidden {
    hidden: bool,
}

/// Any profile hides or shows a shelf for itself only.
async fn set_hidden(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<Hidden>,
) -> ApiResult<StatusCode> {
    shelves::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    shelves::set_hidden(&state.db, user.id, id, body.hidden).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Genre names in the library, for the shelf form. Admin only.
async fn genres(State(state): State<AppState>, _: AdminUser) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(shelves::genre_names(&state.db).await?))
}

#[derive(Serialize)]
pub struct GenreShelf {
    id: i64,
    title: String,
    genre: String,
    items: Vec<ItemCard>,
}

/// The Home rows of the shelves this profile shows, without the empty ones.
pub async fn home_shelves(
    state: &AppState,
    user: &User,
    visible: &access::Policy,
) -> ApiResult<Vec<GenreShelf>> {
    let scope = visible.library_scope();
    let mut out = Vec::new();
    for shelf in shelves::shown_to(&state.db, user.id).await? {
        let mut items = shelves::genre_items(
            &state.db,
            user.id,
            &shelf.genre,
            scope.as_deref(),
            SHELF_ITEMS,
        )
        .await?;
        visible.retain(&mut items);
        if !items.is_empty() {
            out.push(GenreShelf {
                id: shelf.id,
                title: shelf.title,
                genre: shelf.genre,
                items,
            });
        }
    }
    Ok(out)
}
