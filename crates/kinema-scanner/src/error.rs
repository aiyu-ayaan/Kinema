use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScannerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Notify watch error: {0}")]
    Notify(#[from] notify::Error),

    #[error("Walk error: {0}")]
    Walk(#[from] walkdir::Error),

    #[error("Parsing error: {0}")]
    Parse(String),
}
