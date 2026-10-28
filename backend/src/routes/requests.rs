//! Title requests: a profile asks for a movie or show to be added; the administrator
//! answers. The search uses TMDB when a key is set; without one the title is typed.

use crate::auth::AuthUser;
use crate::error::{ApiError, ApiResult};
use crate::notify::{self, Event};
use crate::scan;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, patch};
use axum::{Json, Router};
use kinema_db::requests::{self, MediaRequest, NewRequest};
use kinema_meta::Candidate;
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/requests", get(list).post(create))
        .route("/requests/search", get(search))
        .route("/requests/:id", patch(answer).delete(remove))
}

fn kind_ok(kind: &str) -> ApiResult<&'static str> {
    match kind {
        "movie" => Ok("movie"),
        "show" => Ok("show"),
        _ => Err(ApiError::bad_request("requests.kind_invalid")),
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    kind: String,
}

#[derive(Serialize)]
struct SearchResult {
    /// False without a TMDB key: the client offers the typed title as it is.
    tmdb: bool,
    results: Vec<Candidate>,
}

async fn search(
    State(state): State<AppState>,
    _: AuthUser,
    Query(q): Query<SearchQuery>,
) -> ApiResult<Json<SearchResult>> {
    let kind = kind_ok(&q.kind)?;
    let Some(tmdb) = scan::tmdb_client(&state).await else {
        return Ok(Json(SearchResult {
            tmdb: false,
            results: Vec::new(),
        }));
    };
    let query = q.q.trim();
    let results = if query.is_empty() {
        Vec::new()
    } else {
        tmdb.candidates(kind, query, None).await?
    };
    Ok(Json(SearchResult {
        tmdb: true,
        results,
    }))
}

/// Admins see every request; anyone else their own.
async fn list(
    State(state): State<AppState>,
    AuthUser(me): AuthUser,
) -> ApiResult<Json<Vec<MediaRequest>>> {
    let filter = (!me.is_admin).then_some(me.id);
    Ok(Json(requests::list(&state.db, filter).await?))
}

#[derive(Deserialize)]
struct NewBody {
    kind: String,
    title: String,
    year: Option<i64>,
    tmdb_id: Option<i64>,
    /// TMDB poster path ("/abc.jpg") from the search.
    poster: Option<String>,
    overview: Option<String>,
    note: Option<String>,
}

fn trimmed(s: Option<&str>, max: usize) -> Option<String> {
    s.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(max).collect())
}

async fn create(
    State(state): State<AppState>,
    AuthUser(me): AuthUser,
    Json(b): Json<NewBody>,
) -> ApiResult<(StatusCode, Json<MediaRequest>)> {
    let kind = kind_ok(&b.kind)?;
    let title = trimmed(Some(&b.title), 200)
        .ok_or_else(|| ApiError::bad_request("requests.title_required"))?;
    if let Some(tmdb_id) = b.tmdb_id {
        if requests::in_library(&state.db, kind, tmdb_id).await? {
            return Err(ApiError::new(StatusCode::CONFLICT, "requests.in_library"));
        }
        if requests::open_duplicate(&state.db, kind, tmdb_id).await? {
            return Err(ApiError::new(StatusCode::CONFLICT, "requests.duplicate"));
        }
    }
    // Only a plain TMDB file name becomes an image key, so the key cannot point elsewhere.
    let poster = b
        .poster
        .as_deref()
        .map(|p| p.trim_start_matches('/'))
        .filter(|p| {
            !p.is_empty()
                && p.len() < 100
                && p.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
        })
        .map(|p| format!("w500/{p}"));
    let id = requests::create(
        &state.db,
        &NewRequest {
            user_id: me.id,
            kind,
            title: &title,
            year: b.year.filter(|y| (1870..=2200).contains(y)),
            tmdb_id: b.tmdb_id,
            poster: poster.as_deref(),
            overview: trimmed(b.overview.as_deref(), 1000).as_deref(),
            note: trimmed(b.note.as_deref(), 500).as_deref(),
        },
    )
    .await?;
    let r = requests::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::internal)?;
    notify::spawn(
        &state,
        Event::MediaRequest,
        notify::media_request(&me.name, &r.title, kind),
    );
    Ok((StatusCode::CREATED, Json(r)))
}

#[derive(Deserialize)]
struct Answer {
    status: String,
    admin_note: Option<String>,
}

async fn answer(
    State(state): State<AppState>,
    AuthUser(me): AuthUser,
    Path(id): Path<i64>,
    Json(a): Json<Answer>,
) -> ApiResult<Json<MediaRequest>> {
    if !me.is_admin {
        return Err(ApiError::new(StatusCode::FORBIDDEN, "auth.admin_only"));
    }
    if !requests::STATUSES.contains(&a.status.as_str()) {
        return Err(ApiError::bad_request("requests.status_invalid"));
    }
    let note = trimmed(a.admin_note.as_deref(), 500);
    if !requests::set_status(&state.db, id, &a.status, note.as_deref()).await? {
        return Err(ApiError::not_found());
    }
    Ok(Json(
        requests::get(&state.db, id)
            .await?
            .ok_or_else(ApiError::not_found)?,
    ))
}

/// The requester may withdraw a pending request; the admin may delete any.
async fn remove(
    State(state): State<AppState>,
    AuthUser(me): AuthUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let r = requests::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !me.is_admin && (r.user_id != me.id || r.status != "pending") {
        return Err(ApiError::new(StatusCode::FORBIDDEN, "requests.not_yours"));
    }
    requests::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
