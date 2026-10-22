//! Scrub-thumbnail sheets, made on the first request for a title (not for the whole library: most titles are never
//! scrubbed). The work shares the one background media slot with thumbnails and markers, at low priority, and
//! one file is never made twice at once. A version of a file (id, mtime, size) is made once and cached.

use crate::state::AppState;
use kinema_db::items::MediaFile;
use kinema_media::sprites::{self, Plan};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use tracing::warn;

/// Files being made right now.
static RUNNING: LazyLock<Mutex<HashSet<i64>>> = LazyLock::new(Mutex::default);

pub const INDEX: &str = "index.vtt";

pub enum Status {
    /// The directory holding `index.vtt` and `sheet-N.jpg`.
    Ready(PathBuf),
    Pending,
    /// No sheets can be made (no video, unreadable): do not ask again for this version.
    Unavailable,
}

fn key(file: &MediaFile) -> String {
    format!("{}-{}-{}", file.id, file.mtime, file.size)
}

fn root(state: &AppState) -> PathBuf {
    state.cache_dir("sprites")
}

/// Where the sheets are, or a nudge to make them.
pub async fn status(state: &AppState, file: &MediaFile) -> Status {
    let dir = root(state).join(key(file));
    if tokio::fs::metadata(dir.join(INDEX)).await.is_ok() {
        return Status::Ready(dir);
    }
    if tokio::fs::metadata(dir.with_extension("fail"))
        .await
        .is_ok()
    {
        return Status::Unavailable;
    }
    let Some(plan) = file.duration_s.and_then(Plan::new) else {
        return Status::Unavailable;
    };
    if state.bins.is_none() {
        return Status::Unavailable;
    }
    let started = RUNNING.lock().is_ok_and(|mut r| r.insert(file.id));
    if started {
        let (state, file) = (state.clone(), file.clone());
        tokio::spawn(async move {
            if let Err(e) = make(&state, &file, plan).await {
                warn!("scrub thumbnails for {}: {e}", file.path);
                let _ =
                    tokio::fs::write(root(&state).join(format!("{}.fail", key(&file))), b"").await;
            }
            if let Ok(mut r) = RUNNING.lock() {
                r.remove(&file.id);
            }
        });
    }
    Status::Pending
}

async fn make(state: &AppState, file: &MediaFile, plan: Plan) -> anyhow::Result<()> {
    let Some(bins) = state.bins.clone() else {
        return Ok(());
    };
    let root = root(state);
    let name = key(file);
    let (dir, tmp) = (root.join(&name), root.join(format!("{name}.tmp")));
    tokio::fs::create_dir_all(&root).await?;
    drop_other_versions(&root, file.id, &name).await;
    let _ = tokio::fs::remove_dir_all(&tmp).await;
    tokio::fs::create_dir(&tmp).await?;

    let _permit = state.thumb_slots.acquire().await?;
    let made = match sprites::generate(&bins, Path::new(&file.path), &tmp, &plan).await {
        Ok(n) => n,
        Err(e) => {
            let _ = tokio::fs::remove_dir_all(&tmp).await;
            return Err(e.into());
        }
    };
    let index = sprites::vtt(&plan.limited_to(made), |n| format!("sprites/{n}.jpg"));
    tokio::fs::write(tmp.join(INDEX), index).await?;
    // The index exists only in a finished directory, so a reader never sees half of one.
    let _ = tokio::fs::remove_dir_all(&dir).await;
    tokio::fs::rename(&tmp, &dir).await?;
    Ok(())
}

/// A changed file leaves its old sheets behind: remove every directory of this file but `keep`.
async fn drop_other_versions(root: &Path, file_id: i64, keep: &str) {
    let prefix = format!("{file_id}-");
    let Ok(mut entries) = tokio::fs::read_dir(root).await else {
        return;
    };
    while let Ok(Some(e)) = entries.next_entry().await {
        let name = e.file_name().to_string_lossy().into_owned();
        let stem = name.split('.').next().unwrap_or_default();
        if name.starts_with(&prefix) && stem != keep {
            let path = e.path();
            let _ = if path.is_dir() {
                tokio::fs::remove_dir_all(path).await
            } else {
                tokio::fs::remove_file(path).await
            };
        }
    }
}
