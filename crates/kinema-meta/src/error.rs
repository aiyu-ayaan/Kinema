use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetaError {
    #[error("Network error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TMDB error: {0}")]
    Tmdb(String),

    #[error("TVmaze error: {0}")]
    Tvmaze(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
