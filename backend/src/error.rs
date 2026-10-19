use crate::i18n::t;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// `{"error": {"code", "message"}}`. The code is a key in `locales/en.json`.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str) -> Self {
        Self { status, code }
    }
    pub fn bad_request(code: &'static str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code)
    }
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "common.not_found")
    }
    pub fn internal() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "common.internal")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = json!({ "error": { "code": self.code, "message": t(self.code) } });
        (self.status, Json(body)).into_response()
    }
}

macro_rules! internal_from {
    ($($t:ty),*) => {$(
        impl From<$t> for ApiError {
            fn from(e: $t) -> Self {
                tracing::error!("{e}");
                ApiError::internal()
            }
        }
    )*};
}

internal_from!(
    kinema_db::DbError,
    kinema_media::MediaError,
    kinema_meta::MetaError,
    std::io::Error,
    tokio::task::JoinError
);
