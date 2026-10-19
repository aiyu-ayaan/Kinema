use crate::auth::{AdminUser, hash_password};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch};
use axum::{Json, Router};
use kinema_db::users::{self, NewUser, Secret, User, UserUpdate};
use serde::{Deserialize, Serialize};

/// Admin: family profiles.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list).post(create))
        .route("/users/:id", patch(update).delete(remove))
}

pub const AVATARS: &[&str] = &[
    "ember", "ocean", "forest", "plum", "sun", "rose", "slate", "teal",
];

#[derive(Serialize)]
struct UserView {
    #[serde(flatten)]
    user: User,
    /// Empty: every library.
    library_ids: Vec<i64>,
}

async fn view(state: &AppState, user: User) -> ApiResult<UserView> {
    Ok(UserView {
        library_ids: users::library_access(&state.db, user.id).await?,
        user,
    })
}

async fn list(State(state): State<AppState>, _: AdminUser) -> ApiResult<Json<Vec<UserView>>> {
    let mut out = Vec::new();
    for u in users::list(&state.db).await? {
        out.push(view(&state, u).await?);
    }
    Ok(Json(out))
}

#[derive(Deserialize)]
struct Profile {
    name: String,
    #[serde(default)]
    is_admin: bool,
    #[serde(default)]
    avatar: String,
    /// Create: the password, if any. Update: None keeps it, "" removes it.
    password: Option<String>,
    /// 4-8 digits. Same None / "" rules as `password`.
    pin: Option<String>,
    #[serde(default)]
    library_ids: Vec<i64>,
}

/// Validates and hashes a profile's fields. Admins need a password so the
/// settings can never be opened from the picker with one tap.
async fn prepare(p: &Profile, had_password: bool) -> ApiResult<(String, String, Lock, Lock)> {
    let name = p.name.trim().to_string();
    if name.is_empty() || name.chars().count() > 40 {
        return Err(ApiError::bad_request("users.name_invalid"));
    }
    let avatar = if AVATARS.contains(&p.avatar.as_str()) {
        p.avatar.clone()
    } else {
        AVATARS[0].to_string()
    };
    let password = match p.password.as_deref() {
        None => Lock::Keep,
        Some("") => Lock::Clear,
        Some(pw) if pw.chars().count() < 8 => {
            return Err(ApiError::bad_request("auth.password_too_short"));
        }
        Some(pw) => {
            let pw = pw.to_string();
            Lock::Set(tokio::task::spawn_blocking(move || hash_password(&pw)).await??)
        }
    };
    let pin = match p.pin.as_deref() {
        None => Lock::Keep,
        Some("") => Lock::Clear,
        Some(pin) if !(4..=8).contains(&pin.len()) || !pin.chars().all(|c| c.is_ascii_digit()) => {
            return Err(ApiError::bad_request("users.pin_invalid"));
        }
        Some(pin) => {
            let pin = pin.to_string();
            Lock::Set(tokio::task::spawn_blocking(move || hash_password(&pin)).await??)
        }
    };
    let will_have_password = match password {
        Lock::Keep => had_password,
        Lock::Clear => false,
        Lock::Set(_) => true,
    };
    if p.is_admin && !will_have_password {
        return Err(ApiError::bad_request("users.admin_needs_password"));
    }
    Ok((name, avatar, password, pin))
}

/// A lock change with an owned hash, before it is borrowed into `Secret`.
enum Lock {
    Keep,
    Clear,
    Set(String),
}

fn borrowed(s: &Lock) -> Secret<'_> {
    match s {
        Lock::Keep => Secret::Keep,
        Lock::Clear => Secret::Clear,
        Lock::Set(h) => Secret::Set(h.as_str()),
    }
}

async fn create(
    State(state): State<AppState>,
    _: AdminUser,
    Json(p): Json<Profile>,
) -> ApiResult<(StatusCode, Json<UserView>)> {
    let (name, avatar, password, pin) = prepare(&p, false).await?;
    let hash = |s: &Lock| match s {
        Lock::Set(h) => Some(h.clone()),
        _ => None,
    };
    let (password_hash, pin_hash) = (hash(&password), hash(&pin));
    let id = users::create(
        &state.db,
        &NewUser {
            name: &name,
            is_admin: p.is_admin,
            avatar: &avatar,
            password_hash: password_hash.as_deref(),
            pin_hash: pin_hash.as_deref(),
        },
    )
    .await?
    .ok_or_else(|| ApiError::new(StatusCode::CONFLICT, "users.name_taken"))?;
    users::set_library_access(&state.db, id, if p.is_admin { &[] } else { &p.library_ids }).await?;
    let user = users::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok((StatusCode::CREATED, Json(view(&state, user).await?)))
}

async fn update(
    State(state): State<AppState>,
    AdminUser(me): AdminUser,
    Path(id): Path<i64>,
    Json(p): Json<Profile>,
) -> ApiResult<Json<UserView>> {
    let current = users::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if current.is_admin && !p.is_admin && users::admin_count(&state.db).await? <= 1 {
        return Err(ApiError::bad_request("users.last_admin"));
    }
    if id == me.id && !p.is_admin {
        return Err(ApiError::bad_request("users.self_demote"));
    }
    let (name, avatar, password, pin) = prepare(&p, current.has_password).await?;
    let lock_changed = !matches!(password, Lock::Keep) || !matches!(pin, Lock::Keep);
    let saved = users::update(
        &state.db,
        id,
        &UserUpdate {
            name: &name,
            is_admin: p.is_admin,
            avatar: &avatar,
            password_hash: borrowed(&password),
            pin_hash: borrowed(&pin),
        },
    )
    .await?;
    if !saved {
        return Err(ApiError::new(StatusCode::CONFLICT, "users.name_taken"));
    }
    users::set_library_access(&state.db, id, if p.is_admin { &[] } else { &p.library_ids }).await?;
    // A new lock means old sessions were opened with the old one.
    if lock_changed && id != me.id {
        users::delete_user_sessions(&state.db, id).await?;
    }
    let user = users::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(view(&state, user).await?))
}

async fn remove(
    State(state): State<AppState>,
    AdminUser(me): AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    if id == me.id {
        return Err(ApiError::bad_request("users.self_delete"));
    }
    users::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
