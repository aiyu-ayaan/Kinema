//! Video-frame thumbnails, made by a background job after scans so browsing never
//! waits on ffmpeg. One frame at a time at low priority; cached per file version.

use crate::state::AppState;
use kinema_db::items::{self, MediaFile};
use std::path::PathBuf;
use tracing::warn;

/// (thumbnail, failure marker) for a file version.
pub fn paths(state: &AppState, file: &MediaFile) -> (PathBuf, PathBuf) {
    let dir = state.cache_dir("thumbs");
    (
        dir.join(format!("{}-{}.jpg", file.id, file.mtime)),
        dir.join(format!("{}-{}.fail", file.id, file.mtime)),
    )
}

pub fn is_running(state: &AppState, library_id: i64) -> bool {
    state.thumb_jobs.is_running(library_id)
}

/// Fills in missing thumbnails for a library, unless thumbnails are turned off.
pub fn spawn(state: AppState, library_id: i64) {
    if state.bins.is_none() || !state.settings().thumbnails {
        return;
    }
    let jobs = state.thumb_jobs.clone();
    jobs.spawn(library_id, move || run(state.clone(), library_id));
}

async fn run(state: AppState, library_id: i64) -> anyhow::Result<()> {
    let Some(bins) = state.bins.clone() else {
        return Ok(());
    };
    tokio::fs::create_dir_all(state.cache_dir("thumbs")).await?;
    for file in items::library_media_files(&state.db, library_id).await? {
        let settings = state.settings();
        if !settings.thumbnails {
            break;
        }
        let (out, failed) = paths(&state, &file);
        if out.exists() || failed.exists() {
            continue;
        }
        let _permit = state.background_slot(&state.thumb_slots).await?;
        let position = f64::from(settings.thumbnail_position) / 100.0;
        // Short clips use their middle; long ones skip logos and cold opens.
        let at = file
            .duration_s
            .map_or(30.0, |d| if d > 120.0 { d * position } else { d / 2.0 });
        if let Err(e) =
            kinema_media::thumbs::frame(&bins, std::path::Path::new(&file.path), at, &out).await
        {
            warn!("thumbnail for {}: {e}", file.path);
            let _ = tokio::fs::write(&failed, b"").await;
        }
    }
    Ok(())
}

/// Deletes a library's thumbnails (and failure markers) and makes them again.
pub async fn regenerate(state: AppState, library_id: i64) -> anyhow::Result<()> {
    for file in items::library_media_files(&state.db, library_id).await? {
        let (out, failed) = paths(&state, &file);
        let _ = tokio::fs::remove_file(out).await;
        let _ = tokio::fs::remove_file(failed).await;
    }
    spawn(state, library_id);
    Ok(())
}
