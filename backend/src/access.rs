//! Library access per profile. Admins and unrestricted profiles see everything; a
//! restricted profile (e.g. a child's) sees only the libraries it was given, and
//! anything outside them answers 404 as if it did not exist.

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use kinema_db::items;
use kinema_db::users::{self, User};
use std::collections::HashSet;

/// None = every library.
pub async fn visible(state: &AppState, user: &User) -> ApiResult<Option<HashSet<i64>>> {
    if user.is_admin {
        return Ok(None);
    }
    let ids = users::library_access(&state.db, user.id).await?;
    Ok((!ids.is_empty()).then(|| ids.into_iter().collect()))
}

pub fn allows(visible: &Option<HashSet<i64>>, library_id: i64) -> bool {
    visible.as_ref().is_none_or(|v| v.contains(&library_id))
}

pub async fn check_library(state: &AppState, user: &User, library_id: i64) -> ApiResult<()> {
    if allows(&visible(state, user).await?, library_id) {
        Ok(())
    } else {
        Err(ApiError::not_found())
    }
}

pub async fn check_item(state: &AppState, user: &User, item_id: i64) -> ApiResult<()> {
    let card = items::card(&state.db, user.id, item_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    check_library(state, user, card.library_id).await
}

pub async fn check_file(state: &AppState, user: &User, file_id: i64) -> ApiResult<()> {
    let item_id = items::file(&state.db, file_id)
        .await?
        .and_then(|f| f.item_id)
        .ok_or_else(ApiError::not_found)?;
    check_item(state, user, item_id).await
}
