//! Subtitles to cached WebVTT, made by a background job after scans so the first switch
//! to a track in the player is a cache hit. Text tracks of a video are extracted in one
//! ffmpeg run; picture tracks (PGS, VobSub) are read by OCR when the admin turned it on and
//! `tesseract` was found at startup. Every run shares the thumbnail slot, so background media
//! work stays one lowest-priority process at a time. A track that failed leaves a marker and
//! is not tried again until its file changes.

use crate::state::AppState;
use kinema_db::items::{self, SubtitleJob};
use kinema_media::subtitles::{self, CacheKey};
use std::path::{Path, PathBuf};
use tracing::warn;

/// The cache key of a subtitle track's current version.
pub fn key(job: &SubtitleJob) -> CacheKey<'_> {
    CacheKey {
        file_id: job.file_id,
        size: job.size,
        mtime: job.mtime,
        idx: job.idx,
        sidecar: job
            .external_path
            .as_deref()
            .map(|p| (p, job.external_fingerprint.as_deref().unwrap_or("0"))),
    }
}

pub fn is_running(state: &AppState, library_id: i64) -> bool {
    state.subtitle_jobs.is_running(library_id)
}

pub fn spawn(state: AppState, library_id: i64) {
    if state.bins.is_none() {
        return;
    }
    let jobs = state.subtitle_jobs.clone();
    jobs.spawn(library_id, move || run(state.clone(), library_id));
}

async fn run(state: AppState, library_id: i64) -> anyhow::Result<()> {
    let Some(bins) = state.bins.clone() else {
        return Ok(());
    };
    let dir = state.cache_dir("subtitles");
    tokio::fs::create_dir_all(&dir).await?;
    let all = items::library_subtitles(&state.db, library_id).await?;
    for file in all.chunk_by(|a, b| a.file_id == b.file_id) {
        let mut embedded: Vec<(i64, PathBuf, PathBuf)> = Vec::new();
        for job in file {
            let (out, failed) = key(job).paths(&dir);
            let image = subtitles::is_image_subtitle(job.codec.as_deref());
            if failed.exists() || (out.exists() && (!image || job.ocr)) {
                continue;
            }
            if image {
                if out.exists() {
                    // Read before the cache flag was saved, or after a crash in between.
                    items::set_stream_ocr(&state.db, job.stream_id, true).await?;
                } else {
                    ocr(&state, &bins, job, &out, &failed).await?;
                }
            } else if !subtitles::is_text_subtitle(job.codec.as_deref()) {
                continue;
            } else if let Some(ext) = &job.external_path {
                let _slot = state.thumb_slots.acquire().await?;
                if let Err(e) = subtitles::extract_vtt(&bins, Path::new(ext), None, &out).await {
                    warn!("subtitle {ext}: {e}");
                    mark_failed(&failed).await;
                }
            } else {
                embedded.push((job.idx, out, failed));
            }
        }
        if let Some(first) = file.first()
            && !embedded.is_empty()
        {
            extract_embedded(&state, &bins, Path::new(&first.path), &embedded).await?;
        }
    }
    Ok(())
}

/// All of a video's text tracks in one read of the file; on failure each track alone, so
/// one broken track does not leave the others without a cache entry.
async fn extract_embedded(
    state: &AppState,
    bins: &kinema_media::MediaBinaries,
    path: &Path,
    tracks: &[(i64, PathBuf, PathBuf)],
) -> anyhow::Result<()> {
    let _slot = state.thumb_slots.acquire().await?;
    let all: Vec<(i64, PathBuf)> = tracks.iter().map(|(i, o, _)| (*i, o.clone())).collect();
    if subtitles::extract_tracks(bins, path, &all).await.is_ok() {
        return Ok(());
    }
    for (idx, out, failed) in tracks {
        if let Err(e) = subtitles::extract_tracks(bins, path, &[(*idx, out.clone())]).await {
            warn!("subtitle {idx} of {}: {e}", path.display());
            mark_failed(failed).await;
        }
    }
    Ok(())
}

async fn ocr(
    state: &AppState,
    bins: &kinema_media::MediaBinaries,
    job: &SubtitleJob,
    out: &Path,
    failed: &Path,
) -> anyhow::Result<()> {
    // Checked per track, so turning the setting off stops a long first pass.
    if !state.settings().subtitle_ocr {
        return Ok(());
    }
    let Some(tool) = state.ocr.as_deref() else {
        return Ok(());
    };
    let Some(lang) = tool.language_for(job.language.as_deref()) else {
        return Ok(());
    };
    let _slot = state.thumb_slots.acquire().await?;
    match kinema_media::ocr::ocr_to_vtt(
        bins,
        tool,
        Path::new(&job.path),
        job.idx,
        lang,
        job.duration_s,
        out,
    )
    .await
    {
        Ok(_) => items::set_stream_ocr(&state.db, job.stream_id, true).await?,
        Err(e) => {
            warn!("subtitle OCR of track {} of {}: {e}", job.idx, job.path);
            mark_failed(failed).await;
        }
    }
    Ok(())
}

async fn mark_failed(path: &Path) {
    let _ = tokio::fs::write(path, b"").await;
}
