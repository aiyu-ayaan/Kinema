//! Scrub thumbnails for the player's seek bar: `timeline.vtt` maps times to tiles, `sprites/{n}.jpg` are the sheets.
//! The first request starts the background job and answers 202 until the sheets exist (see `sprites.rs`).

use crate::access;
use crate::auth::AuthUser;
use crate::error::{ApiError, ApiResult};
use crate::sprites::{self, Status};
use crate::state::AppState;
use axum::Router;
use axum::extract::{Path, Request, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, RETRY_AFTER};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use kinema_db::items;
use kinema_db::users::User;
use std::path::PathBuf;
use tower::ServiceExt;
use tower_http::services::ServeFile;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/items/:id/timeline.vtt", get(timeline))
        .route("/items/:id/sprites/:n", get(sheet))
}

/// The directory of ready sheets for the title's main file; `Err` is the answer to send while they are not ready.
async fn ready(state: &AppState, user: &User, id: i64) -> ApiResult<Result<PathBuf, Response>> {
    access::check_item(state, user, id).await?;
    let file = items::item_files(&state.db, id)
        .await?
        .into_iter()
        .next()
        .ok_or_else(ApiError::not_found)?;
    Ok(match sprites::status(state, &file).await {
        Status::Ready(dir) => Ok(dir),
        Status::Pending => Err((
            StatusCode::ACCEPTED,
            [(RETRY_AFTER, HeaderValue::from_static("15"))],
        )
            .into_response()),
        Status::Unavailable => return Err(ApiError::not_found()),
    })
}

async fn timeline(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> ApiResult<Response> {
    let dir = match ready(&state, &user, id).await? {
        Ok(dir) => dir,
        Err(wait) => return Ok(wait),
    };
    let body = tokio::fs::read_to_string(dir.join(sprites::INDEX)).await?;
    Ok((
        [
            (
                CONTENT_TYPE,
                HeaderValue::from_static("text/vtt; charset=utf-8"),
            ),
            (
                CACHE_CONTROL,
                HeaderValue::from_static("private, max-age=3600"),
            ),
        ],
        body,
    )
        .into_response())
}

async fn sheet(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, file)): Path<(i64, String)>,
    req: Request,
) -> ApiResult<Response> {
    let name = file.strip_suffix(".jpg").unwrap_or(&file);
    let name = name.strip_prefix("sheet-").unwrap_or(name);
    let n: usize = name.parse().map_err(|_| ApiError::not_found())?;
    let dir = match ready(&state, &user, id).await? {
        Ok(dir) => dir,
        Err(wait) => return Ok(wait),
    };
    let mut res = ServeFile::new(dir.join(format!("sheet-{n}.jpg")))
        .oneshot(req)
        .await
        .map_err(|_| ApiError::internal())?
        .into_response();
    if res.status() == StatusCode::NOT_FOUND {
        return Err(ApiError::not_found());
    }
    res.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=3600"),
    );
    Ok(res)
}
