use crate::ScannerError;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;

/// Sends `library_id` on every filesystem change under `roots`. Event-driven,
/// so an idle library costs no CPU. Dropping the watcher stops it.
pub fn watch_library(
    library_id: i64,
    roots: &[PathBuf],
    tx: UnboundedSender<i64>,
) -> Result<RecommendedWatcher, ScannerError> {
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res
            && !event.kind.is_access()
        {
            let _ = tx.send(library_id);
        }
    })?;
    for root in roots {
        watcher.watch(root, RecursiveMode::Recursive)?;
    }
    Ok(watcher)
}
