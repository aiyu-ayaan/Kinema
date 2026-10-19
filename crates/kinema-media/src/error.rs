use thiserror::Error;

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("FFmpeg not found: {0}")]
    BinaryNotFound(String),

    #[error("Probe failed: {0}")]
    ProbeFailed(String),

    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
}
