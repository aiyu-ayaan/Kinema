use crate::auth::{
    AuthUser, SESSION_TTL_S, hash_password, new_token, session_cookie, token_from, token_hash,
    verify_password,
};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::extract::{Request, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use kinema_db::users::{self, User};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(status))
        .route("/setup", post(setup))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

#[derive(Serialize)]
struct Status {
    needs_setup: bool,
}

async fn status(State(state): State<AppState>) -> ApiResult<Json<Status>> {
    Ok(Json(Status {
        needs_setup: users::count(&state.db).await? == 0,
    }))
}

#[derive(Deserialize)]
struct Credentials {
    name: String,
    password: String,
}

#[derive(Serialize)]
struct SessionResponse {
    token: String,
    user: User,
}

fn signed_in(token: String, user: User) -> Response {
    let mut headers = HeaderMap::new();
    if let Ok(v) = session_cookie(&token, SESSION_TTL_S).parse() {
        headers.insert(SET_COOKIE, v);
    }
    (headers, Json(SessionResponse { token, user })).into_response()
}

async fn start_session(state: &AppState, user: User) -> ApiResult<Response> {
    let (token, hash) = new_token();
    users::create_session(&state.db, &hash, user.id, SESSION_TTL_S).await?;
    Ok(signed_in(token, user))
}

/// First run: the first account becomes the administrator.
async fn setup(State(state): State<AppState>, Json(c): Json<Credentials>) -> ApiResult<Response> {
    let name = c.name.trim();
    if name.is_empty() {
        return Err(ApiError::bad_request("auth.name_required"));
    }
    if c.password.chars().count() < 8 {
        return Err(ApiError::bad_request("auth.password_too_short"));
    }
    let password = c.password.clone();
    let hash = tokio::task::spawn_blocking(move || hash_password(&password)).await??;
    let id = users::create_first_admin(&state.db, name, &hash)
        .await?
        .ok_or_else(|| ApiError::new(StatusCode::CONFLICT, "auth.setup_done"))?;
    start_session(
        &state,
        User {
            id,
            name: name.to_string(),
            is_admin: true,
        },
    )
    .await
}

async fn login(State(state): State<AppState>, Json(c): Json<Credentials>) -> ApiResult<Response> {
    let invalid = || ApiError::new(StatusCode::UNAUTHORIZED, "auth.invalid_credentials");
    let (id, hash) = users::credentials(&state.db, c.name.trim())
        .await?
        .ok_or_else(invalid)?;
    let hash = hash.ok_or_else(invalid)?;
    let password = c.password;
    let ok = tokio::task::spawn_blocking(move || verify_password(&password, &hash)).await?;
    if !ok {
        return Err(invalid());
    }
    let user = users::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    start_session(&state, user).await
}

async fn logout(State(state): State<AppState>, req: Request) -> ApiResult<Response> {
    let (parts, _) = req.into_parts();
    if let Some(token) = token_from(&parts) {
        users::delete_session(&state.db, &token_hash(&token)).await?;
    }
    let mut headers = HeaderMap::new();
    if let Ok(v) = session_cookie("", 0).parse() {
        headers.insert(SET_COOKIE, v);
    }
    Ok((StatusCode::NO_CONTENT, headers).into_response())
}

async fn me(AuthUser(user): AuthUser) -> Json<User> {
    Json(user)
}
