use crate::error::ApiError;
use crate::state::AppState;
use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::header::{AUTHORIZATION, COOKIE};
use axum::http::request::Parts;
use kinema_db::users::{self, User};
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

pub const SESSION_COOKIE: &str = "kinema_session";
pub const SESSION_TTL_S: i64 = 30 * 24 * 3600;

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| {
            tracing::error!("argon2: {e}");
            ApiError::internal()
        })
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|h| {
        Argon2::default()
            .verify_password(password.as_bytes(), &h)
            .is_ok()
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Random 256-bit token for the client; only its SHA-256 is stored.
pub fn new_token() -> (String, String) {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token = hex(&bytes);
    let hash = token_hash(&token);
    (token, hash)
}

pub fn token_hash(token: &str) -> String {
    hex(&Sha256::digest(token.as_bytes()))
}

/// Bearer header for API calls; the cookie covers `<video>`, `<img>` and subtitle
/// fetches, which cannot set headers.
pub fn token_from(parts: &Parts) -> Option<String> {
    if let Some(bearer) = parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        return Some(bearer.trim().to_string());
    }
    parts
        .headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|c| {
            c.trim()
                .strip_prefix(&format!("{SESSION_COOKIE}=")[..])
                .map(str::to_string)
        })
}

pub fn session_cookie(token: &str, max_age: i64) -> String {
    format!("{SESSION_COOKIE}={token}; Path=/api; HttpOnly; SameSite=Strict; Max-Age={max_age}")
}

/// Any signed-in user.
pub struct AuthUser(pub User);

/// The administrator.
pub struct AdminUser(#[allow(dead_code)] pub User);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let token = token_from(parts)
            .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "auth.required"))?;
        users::session_user(&state.db, &token_hash(&token))
            .await?
            .map(AuthUser)
            .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "auth.required"))
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state).await?;
        if !user.is_admin {
            return Err(ApiError::new(StatusCode::FORBIDDEN, "auth.admin_only"));
        }
        Ok(AdminUser(user))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_roundtrip() {
        let h = hash_password("correct horse").unwrap();
        assert!(verify_password("correct horse", &h));
        assert!(!verify_password("wrong", &h));
    }

    #[test]
    fn token_hash_is_stable() {
        let (token, hash) = new_token();
        assert_eq!(token.len(), 64);
        assert_eq!(token_hash(&token), hash);
    }
}
