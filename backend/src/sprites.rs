//! Scrub-thumbnail sheets, made on the first request for a title (not for the whole library: most titles are never
//! scrubbed). The work shares the one background media slot with thumbnails and markers, at low priority, and
//! one file is never made twice at once. A version of a file (id, mtime, size) is made once and cached.

use crate::state::AppState;
use kinema_db::items::{self, MediaFile};
use kinema_media::sprites::{self, Plan};
use std::collections::{HashMap, HashSet};
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
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

/// Bytes read from each end of the file for its content fingerprint.
const FINGERPRINT_BYTES: usize = 64 * 1024;

/// Content fingerprints by (file id, size, mtime), so the file is read once per version and not per request.
type Version = (i64, i64, i64);
static FINGERPRINTS: LazyLock<Mutex<HashMap<Version, String>>> = LazyLock::new(Mutex::default);

/// FNV-1a over the size and the first and last 64 KiB. Not `DefaultHasher`: its output may change between Rust
/// releases, which would silently throw the whole sheet cache away.
async fn content_hash(path: &str, size: u64) -> std::io::Result<u64> {
    let mut f = tokio::fs::File::open(path).await?;
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut feed = |bytes: &[u8]| {
        for b in bytes {
            hash = (hash ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3);
        }
    };
    feed(&size.to_le_bytes());
    let mut buf = vec![0; FINGERPRINT_BYTES];
    let n = f.read(&mut buf).await?;
    feed(&buf[..n]);
    if size > FINGERPRINT_BYTES as u64 {
        f.seek(SeekFrom::End(-(FINGERPRINT_BYTES as i64))).await?;
        let n = f.read(&mut buf).await?;
        feed(&buf[..n]);
    }
    Ok(hash)
}

/// Names a version of a file's sheets. Size plus a content hash, not mtime: a copy or `touch` that keeps the
/// bytes keeps the sheets. If the file cannot be read the key falls back to mtime, so it still changes with edits.
async fn key(file: &MediaFile) -> String {
    let id = (file.id, file.size, file.mtime);
    if let Some(known) = FINGERPRINTS.lock().ok().and_then(|m| m.get(&id).cloned()) {
        return known;
    }
    let key = match content_hash(&file.path, file.size.max(0) as u64).await {
        Ok(h) => format!("{}-{}-{h:016x}", file.id, file.size),
        Err(_) => return format!("{}-{}-m{}", file.id, file.size, file.mtime),
    };
    if let Ok(mut m) = FINGERPRINTS.lock() {
        // shortcut: entries are only dropped on restart; each is ~40 bytes, fine for a personal library.
        m.insert(id, key.clone());
    }
    key
}

/// Sheets are worth a GPU decode for codecs and sizes that are slow in software.
async fn wants_gpu(state: &AppState, file: &MediaFile) -> bool {
    let Ok(streams) = items::streams(&state.db, file.id).await else {
        return false;
    };
    streams.iter().any(|s| {
        s.kind == "video"
            && (matches!(s.codec.as_deref(), Some("hevc" | "av1" | "vp9"))
                || s.height.is_some_and(|h| h >= 1440))
    })
}

fn root(state: &AppState) -> PathBuf {
    state.cache_dir("sprites")
}

/// Where the sheets are, or a nudge to make them.
pub async fn status(state: &AppState, file: &MediaFile) -> Status {
    if !state.features().scrub_thumbnails {
        return Status::Unavailable;
    }
    let dir = root(state).join(key(file).await);
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
                    tokio::fs::write(root(&state).join(format!("{}.fail", key(&file).await)), b"")
                        .await;
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
    let name = key(file).await;
    let (dir, tmp) = (root.join(&name), root.join(format!("{name}.tmp")));
    tokio::fs::create_dir_all(&root).await?;
    drop_other_versions(&root, file.id, &name).await;
    let _ = tokio::fs::remove_dir_all(&tmp).await;
    tokio::fs::create_dir(&tmp).await?;

    let hw = wants_gpu(state, file).await;
    let _permit = state.thumb_slots.acquire().await?;
    let made = match sprites::generate(&bins, Path::new(&file.path), &tmp, &plan, hw).await {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hash_follows_content_not_mtime() {
        let dir = std::env::temp_dir().join(format!("kinema-sprites-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let (a, b, c) = (dir.join("a"), dir.join("b"), dir.join("c"));
        let big = vec![7u8; FINGERPRINT_BYTES * 3];
        std::fs::write(&a, &big).expect("a");
        std::fs::write(&b, &big).expect("b");
        let mut other = big.clone();
        *other.last_mut().expect("byte") = 8;
        std::fs::write(&c, &other).expect("c");
        let size = big.len() as u64;
        let (ha, hb, hc) = (
            content_hash(a.to_str().unwrap_or_default(), size).await,
            content_hash(b.to_str().unwrap_or_default(), size).await,
            content_hash(c.to_str().unwrap_or_default(), size).await,
        );
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(ha.as_ref().ok(), hb.as_ref().ok());
        assert_ne!(ha.ok(), hc.ok());
    }
}
