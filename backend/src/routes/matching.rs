use crate::auth::AdminUser;
use crate::error::{ApiError, ApiResult};
use crate::scan;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use kinema_db::items::{self, Meta};
use kinema_meta::{Candidate, TmdbClient};
use serde::Deserialize;

/// Fix match: search TMDB by hand and pin an item to the chosen result.
pub fn router() -> Router<AppState> {
    Router::new().route("/items/:id/match", get(search).post(apply))
}

async fn target(state: &AppState, id: i64) -> ApiResult<(String, i64)> {
    let card = items::card(&state.db, 0, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if card.kind != "movie" && card.kind != "show" {
        return Err(ApiError::bad_request("match.unsupported"));
    }
    Ok((card.kind, card.library_id))
}

async fn client(state: &AppState) -> ApiResult<TmdbClient> {
    scan::tmdb_client(state)
        .await
        .ok_or_else(|| ApiError::bad_request("match.no_key"))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    year: Option<i64>,
}

async fn search(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
    Query(q): Query<SearchQuery>,
) -> ApiResult<Json<Vec<Candidate>>> {
    let (kind, _) = target(&state, id).await?;
    if q.q.trim().is_empty() {
        return Ok(Json(Vec::new()));
    }
    let tmdb = client(&state).await?;
    Ok(Json(tmdb.candidates(&kind, q.q.trim(), q.year).await?))
}

#[derive(Deserialize)]
struct Pick {
    tmdb_id: i64,
}

async fn apply(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
    Json(pick): Json<Pick>,
) -> ApiResult<StatusCode> {
    let (kind, library_id) = target(&state, id).await?;
    let d = client(&state).await?.by_id(&kind, pick.tmdb_id).await?;
    let image =
        |size: &str, p: Option<String>| p.map(|p| format!("{size}/{}", p.trim_start_matches('/')));
    items::clear_meta(&state.db, id).await?;
    items::save_meta(
        &state.db,
        id,
        &Meta {
            tmdb_id: Some(d.tmdb_id),
            title: Some(d.title).filter(|t| !t.is_empty()),
            year: d.year,
            summary: d.overview,
            rating: d.rating,
            runtime_s: d.runtime_min.map(|m| m * 60),
            poster: image("w500", d.poster),
            backdrop: image("w1280", d.backdrop),
            genres: d.genres,
            cast: Some(
                d.cast
                    .into_iter()
                    .map(|c| items::CastMember {
                        id: c.id,
                        name: c.name,
                        role: c.role,
                        image: c.image,
                    })
                    .collect(),
            ),
        },
    )
    .await?;
    // A show's seasons and episodes were reset; the scan's metadata pass refills them.
    if kind == "show" {
        scan::spawn_scan(state, library_id);
    }
    Ok(StatusCode::NO_CONTENT)
}
