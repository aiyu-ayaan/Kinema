pub mod error;

pub use error::ScannerError;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Find media files in a directory tree without blocking async runtime
pub fn walk_directory(root: &Path) -> Result<Vec<PathBuf>, ScannerError> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }
    Ok(files)
}
