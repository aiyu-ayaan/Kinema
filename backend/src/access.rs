//! What a profile may see. Admins see everything. Other profiles see only the libraries
//! they were given (none listed = all) and, with a content limit, only titles whose
//! certification fits it (`kinema_core::content_rating`). Anything outside answers 404
//! as if it did not exist, on every listing, detail, progress, plan, stream, subtitle and
//! thumbnail path, so a limit cannot be stepped around by calling the API directly.

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use kinema_core::content_rating;
use kinema_db::items::{self, ItemCard};
use kinema_db::profiles;
use kinema_db::users::{self, User};
use std::collections::HashSet;

/// A profile's visibility rules, loaded once per request.
#[derive(Debug, Clone, Default)]
pub struct Policy {
    /// None = every library.
    libraries: Option<HashSet<i64>>,
    /// None = no content limit.
    max_rating: Option<u8>,
    allow_unrated: bool,
}

impl Policy {
    pub fn allows_library(&self, library_id: i64) -> bool {
        allows(&self.libraries, library_id)
    }

    pub fn allows(&self, card: &ItemCard) -> bool {
        self.allows_library(card.library_id)
            && content_rating::allowed(
                card.content_rating.as_deref(),
                self.max_rating,
                self.allow_unrated,
            )
    }

    /// The libraries the profile may open, for queries that filter in SQL; None = all.
    pub fn library_scope(&self) -> Option<Vec<i64>> {
        self.libraries.as_ref().map(|v| v.iter().copied().collect())
    }

    pub fn retain(&self, cards: &mut Vec<ItemCard>) {
        cards.retain(|c| self.allows(c));
    }
}

pub async fn policy(state: &AppState, user: &User) -> ApiResult<Policy> {
    if user.is_admin {
        return Ok(Policy::default());
    }
    let prefs = profiles::get(&state.db, user.id).await?;
    Ok(Policy {
        libraries: visible(state, user).await?,
        max_rating: prefs
            .max_rating
            .map(|r| r.clamp(0, i64::from(content_rating::MAX_AGE)) as u8),
        allow_unrated: prefs.allow_unrated,
    })
}

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

/// The item's card when this profile may see it, else 404.
pub async fn visible_item(state: &AppState, user: &User, item_id: i64) -> ApiResult<ItemCard> {
    let card = items::card(&state.db, user.id, item_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if policy(state, user).await?.allows(&card) {
        Ok(card)
    } else {
        Err(ApiError::not_found())
    }
}

pub async fn check_item(state: &AppState, user: &User, item_id: i64) -> ApiResult<()> {
    visible_item(state, user, item_id).await.map(|_| ())
}

pub async fn check_file(state: &AppState, user: &User, file_id: i64) -> ApiResult<()> {
    let item_id = items::file(&state.db, file_id)
        .await?
        .and_then(|f| f.item_id)
        .ok_or_else(ApiError::not_found)?;
    check_item(state, user, item_id).await
}
