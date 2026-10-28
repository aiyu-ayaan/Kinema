//! Admin: devices profiles signed in from, and requests from devices beyond the free ones.

use crate::auth::AdminUser;
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use kinema_db::devices::{self, Device};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/devices", get(list))
        .route("/devices/:id/approve", post(approve))
        .route("/devices/:id", delete(remove))
}

async fn list(State(state): State<AppState>, _: AdminUser) -> ApiResult<Json<Vec<Device>>> {
    Ok(Json(devices::list(&state.db).await?))
}

/// The waiting client picks up its session on its next poll.
async fn approve(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Device>> {
    if !devices::approve(&state.db, id).await? {
        return Err(ApiError::not_found());
    }
    Ok(Json(
        devices::get(&state.db, id)
            .await?
            .ok_or_else(ApiError::not_found)?,
    ))
}

/// Denies a request, or removes a device and signs it out. Removing frees the profile's slot.
async fn remove(
    State(state): State<AppState>,
    AdminUser(me): AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let device = devices::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    // Removing the admin's own device would end the session making this request.
    if device.user_id == me.id && device.status == "approved" {
        let mine = devices::list(&state.db)
            .await?
            .into_iter()
            .filter(|d| d.user_id == me.id && d.status == "approved")
            .count();
        if mine <= 1 {
            return Err(ApiError::bad_request("devices.self_last"));
        }
    }
    devices::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
