use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Platform error: {0}")]
    Platform(String),
}
