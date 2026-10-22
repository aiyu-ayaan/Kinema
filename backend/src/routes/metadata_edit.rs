//! Manual metadata edits. Every field an admin saves is locked, so rescans, TMDB refreshes and
//! Fix match leave it alone; locks can also be set or cleared on their own.

use crate::auth::AdminUser;
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{patch, put};
use axum::{Json, Router};
use kinema_db::items::{self, Edit};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/items/:id/metadata", patch(edit))
        .route("/items/:id/locks", put(set_locks))
}

#[derive(Serialize)]
struct Locks {
    locked_fields: Vec<String>,
}

#[derive(Deserialize)]
struct Body {
    title: Option<String>,
    year: Option<i64>,
    summary: Option<String>,
    rating: Option<f64>,
}

async fn exists(state: &AppState, id: i64) -> ApiResult<()> {
    items::kind_of(&state.db, id)
        .await?
        .map(|_| ())
        .ok_or_else(ApiError::not_found)
}

/// Saves the given fields and locks them. An empty summary, year 0 or rating 0 clears the field (still locked).
async fn edit(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
    Json(b): Json<Body>,
) -> ApiResult<Json<Locks>> {
    exists(&state, id).await?;
    let title = b.title.map(|t| t.trim().to_string());
    let valid = title
        .as_ref()
        .is_none_or(|t| !t.is_empty() && t.chars().count() <= 300)
        && b.year.is_none_or(|y| (0..=2200).contains(&y))
        && b.rating.is_none_or(|r| (0.0..=10.0).contains(&r))
        && b.summary
            .as_ref()
            .is_none_or(|s| s.chars().count() <= 10_000);
    if !valid {
        return Err(ApiError::bad_request("metadata.invalid"));
    }
    items::edit_meta(
        &state.db,
        id,
        &Edit {
            title,
            year: b.year,
            summary: b.summary,
            rating: b.rating,
        },
    )
    .await?;
    Ok(Json(Locks {
        locked_fields: items::locked_fields(&state.db, id).await?,
    }))
}

#[derive(Deserialize)]
struct LocksBody {
    fields: Vec<String>,
}

/// Sets exactly which fields are locked, e.g. to lock the poster as it is or to unlock a title.
async fn set_locks(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
    Json(b): Json<LocksBody>,
) -> ApiResult<Json<Locks>> {
    exists(&state, id).await?;
    items::set_locked_fields(&state.db, id, &b.fields).await?;
    Ok(Json(Locks {
        locked_fields: items::locked_fields(&state.db, id).await?,
    }))
}
