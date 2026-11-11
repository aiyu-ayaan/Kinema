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

    /// Rate limited or a server error upstream; worth one retry.
    #[error("TMDB unavailable: {0}")]
    Unavailable(String),

    #[error("TVmaze error: {0}")]
    Tvmaze(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl MetaError {
    /// Fails now but may pass on a retry: a dropped or refused connection, a timeout, a 429 or 5xx.
    pub fn is_transient(&self) -> bool {
        match self {
            Self::Reqwest(e) => e.is_connect() || e.is_timeout() || e.is_request(),
            Self::Unavailable(_) => true,
            _ => false,
        }
    }

    /// The error with its causes, for the log: reqwest's own message hides the DNS or TLS reason.
    pub fn chain(&self) -> String {
        let mut out = self.to_string();
        let mut source = std::error::Error::source(self);
        while let Some(e) = source {
            out.push_str(": ");
            out.push_str(&e.to_string());
            source = e.source();
        }
        out
    }
}
