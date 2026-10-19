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
use std::time::{Duration, Instant};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(status))
        .route("/setup", post(setup))
        .route("/login", post(login))
        .route("/profiles", get(profiles))
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

fn signed_in(token: String, user: User, secure: bool) -> Response {
    let mut headers = HeaderMap::new();
    if let Ok(v) = session_cookie(&token, SESSION_TTL_S, secure).parse() {
        headers.insert(SET_COOKIE, v);
    }
    (headers, Json(SessionResponse { token, user })).into_response()
}

async fn start_session(state: &AppState, user: User) -> ApiResult<Response> {
    let (token, hash) = new_token();
    users::create_session(&state.db, &hash, user.id, SESSION_TTL_S).await?;
    Ok(signed_in(token, user, state.config.tls.enabled))
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
    let user = users::get(&state.db, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    start_session(&state, user).await
}

/// The profile picker. Public, like a TV's "who's watching" screen: names and
/// avatars only, plus which lock each profile has.
async fn profiles(State(state): State<AppState>) -> ApiResult<Json<Vec<User>>> {
    Ok(Json(users::list(&state.db).await?))
}

#[derive(Deserialize)]
struct Login {
    /// From the profile picker.
    user_id: Option<i64>,
    /// Typed name, for clients without the picker.
    name: Option<String>,
    password: Option<String>,
    pin: Option<String>,
}

const MAX_FAILURES: u32 = 5;
const LOCKOUT: Duration = Duration::from_secs(60);

/// A 4-digit PIN has 10 000 values; five tries a minute makes guessing take days.
fn locked_out(state: &AppState, user_id: i64) -> bool {
    state.login_failures.lock().is_ok_and(|f| {
        f.get(&user_id)
            .is_some_and(|(n, at)| *n >= MAX_FAILURES && at.elapsed() < LOCKOUT)
    })
}

fn record_failure(state: &AppState, user_id: i64) {
    if let Ok(mut f) = state.login_failures.lock() {
        let entry = f.entry(user_id).or_insert((0, Instant::now()));
        if entry.1.elapsed() >= LOCKOUT {
            *entry = (0, Instant::now());
        }
        entry.0 += 1;
        entry.1 = Instant::now();
    }
}

async fn login(State(state): State<AppState>, Json(c): Json<Login>) -> ApiResult<Response> {
    let invalid = || ApiError::new(StatusCode::UNAUTHORIZED, "auth.invalid_credentials");
    let user = match (c.user_id, c.name.as_deref()) {
        (Some(id), _) => users::get(&state.db, id).await?,
        (None, Some(name)) => users::find_by_name(&state.db, name.trim()).await?,
        (None, None) => None,
    }
    .ok_or_else(invalid)?;
    if locked_out(&state, user.id) {
        return Err(ApiError::new(StatusCode::TOO_MANY_REQUESTS, "auth.locked"));
    }
    let (password_hash, pin_hash) = users::secrets(&state.db, user.id).await?;
    // A profile with no lock opens directly; otherwise the password or the PIN must match.
    let ok = match (password_hash, pin_hash) {
        (None, None) => true,
        (pw, pin) => {
            let password = c.password.unwrap_or_default();
            let entered_pin = c.pin.unwrap_or_default();
            tokio::task::spawn_blocking(move || {
                pw.is_some_and(|h| !password.is_empty() && verify_password(&password, &h))
                    || pin.is_some_and(|h| {
                        !entered_pin.is_empty() && verify_password(&entered_pin, &h)
                    })
            })
            .await?
        }
    };
    if !ok {
        record_failure(&state, user.id);
        return Err(invalid());
    }
    if let Ok(mut f) = state.login_failures.lock() {
        f.remove(&user.id);
    }
    start_session(&state, user).await
}

async fn logout(State(state): State<AppState>, req: Request) -> ApiResult<Response> {
    let (parts, _) = req.into_parts();
    if let Some(token) = token_from(&parts) {
        users::delete_session(&state.db, &token_hash(&token)).await?;
    }
    let mut headers = HeaderMap::new();
    if let Ok(v) = session_cookie("", 0, state.config.tls.enabled).parse() {
        headers.insert(SET_COOKIE, v);
    }
    Ok((StatusCode::NO_CONTENT, headers).into_response())
}

async fn me(AuthUser(user): AuthUser) -> Json<User> {
    Json(user)
}
