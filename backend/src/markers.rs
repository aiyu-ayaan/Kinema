//! Intro and end-credit markers (kinema_media::markers), found by a background job after each scan. Every ffmpeg
//! run shares the thumbnail slot, so background media work stays one low-priority process at a time. A file is
//! looked at once per version, and audio fingerprints are cached, so a new episode is compared with its
//! neighbours without decoding them again.

use crate::state::AppState;
use kinema_db::markers::{self, Marker, MarkerFile};
use kinema_media::MediaBinaries;
use kinema_media::markers::{self as detect, HASH_SECONDS, Span};
use std::path::{Path, PathBuf};
use tracing::warn;

/// How much of an episode's start is searched for the intro, and of its end for the credits.
const HEAD_S: f64 = 600.0;
const TAIL_S: f64 = 480.0;
/// Shared audio shorter than this is a sound effect or a jingle, not an intro or credits.
const MIN_SHARED_S: f64 = 15.0;
/// Longer "intros" are a recap or a repeated scene.
const MAX_INTRO_S: f64 = 200.0;

pub fn spawn(state: AppState, library_id: i64) {
    if state.bins.is_none() {
        return;
    }
    let jobs = state.marker_jobs.clone();
    jobs.spawn(library_id, move || run(state.clone(), library_id));
}

async fn run(state: AppState, library_id: i64) -> anyhow::Result<()> {
    let Some(bins) = state.bins.clone() else {
        return Ok(());
    };
    tokio::fs::create_dir_all(state.cache_dir("prints")).await?;
    let files = markers::library_files(&state.db, library_id).await?;
    for file in files.iter().filter(|f| !f.done()) {
        let found = find(&state, &bins, file, &files).await;
        markers::save_detected(&state.db, file.id, file.mtime, &found).await?;
    }
    Ok(())
}

/// Chapters first; for an episode, audio shared with the nearest episodes of its season; for credits still
/// missing, a black run to the end.
async fn find(
    state: &AppState,
    bins: &MediaBinaries,
    file: &MarkerFile,
    all: &[MarkerFile],
) -> Vec<Marker> {
    let path = Path::new(&file.path);
    let duration = file.duration_s.unwrap_or(0.0);
    let (mut intro, mut credits) = (None, None);

    match slot(state, detect::chapters(bins, path)).await {
        Some(Ok(chapters)) => {
            let (i, c) = detect::chapter_spans(&chapters);
            intro = i.map(|s| (s, "chapter"));
            credits = c.map(|s| (s, "chapter"));
        }
        Some(Err(e)) => warn!("chapters of {}: {e}", file.path),
        None => {}
    }

    if file.kind == "episode" && duration > 0.0 && (intro.is_none() || credits.is_none()) {
        let mut siblings: Vec<&MarkerFile> = all
            .iter()
            .filter(|f| f.id != file.id && f.season_id.is_some() && f.season_id == file.season_id)
            .collect();
        siblings.sort_by_key(|f| (f.episode_no.unwrap_or(0) - file.episode_no.unwrap_or(0)).abs());
        for other in siblings.into_iter().take(2) {
            if intro.is_none() {
                intro = shared(state, bins, file, other, Part::Head)
                    .await
                    .filter(|s| s.end - s.start <= MAX_INTRO_S)
                    .map(|s| (s, "audio"));
            }
            if credits.is_none() {
                credits = shared(state, bins, file, other, Part::Tail)
                    .await
                    .filter(|s| s.start >= duration / 2.0)
                    .map(|s| (s, "audio"));
            }
        }
    }

    let tail_from = (duration - TAIL_S).max(duration / 2.0);
    let span = |start| Span {
        start,
        end: duration,
    };
    if credits.is_none() && duration > 300.0 {
        match slot(state, detect::black_runs(bins, path, tail_from)).await {
            Some(Ok(runs)) => {
                credits = detect::credits_from_black(&runs, duration).map(|s| (span(s), "black"));
            }
            Some(Err(e)) => warn!("black frames of {}: {e}", file.path),
            None => {}
        }
    }

    // Credits over lit footage: no black, and no shared song (a movie, or an episode without a neighbour).
    if credits.is_none() && duration > 300.0 {
        match slot(state, detect::tail_looks(bins, path, tail_from)).await {
            Some(Ok(looks)) => {
                credits = detect::credits_from_looks(&looks, duration).map(|s| (span(s), "text"));
            }
            Some(Err(e)) => warn!("credit text check of {}: {e}", file.path),
            None => {}
        }
    }

    [("intro", intro), ("credits", credits)]
        .into_iter()
        .filter_map(|(kind, found)| {
            found.map(|(span, source)| Marker {
                kind: kind.into(),
                start_s: span.start,
                end_s: span.end,
                source: source.into(),
            })
        })
        .collect()
}

#[derive(Clone, Copy)]
enum Part {
    Head,
    Tail,
}

impl Part {
    /// (from, seconds) of the file this part covers.
    fn window(self, duration: f64) -> (f64, f64) {
        match self {
            Part::Head => (0.0, HEAD_S.min(duration / 2.0)),
            Part::Tail => {
                let from = (duration - TAIL_S).max(duration / 2.0);
                (from, duration - from)
            }
        }
    }

    fn name(self) -> &'static str {
        match self {
            Part::Head => "head",
            Part::Tail => "tail",
        }
    }
}

/// The audio `file` and `other` share in this part, in `file`'s seconds.
async fn shared(
    state: &AppState,
    bins: &MediaBinaries,
    file: &MarkerFile,
    other: &MarkerFile,
    part: Part,
) -> Option<Span> {
    let a = print(state, bins, file, part).await?;
    let b = print(state, bins, other, part).await?;
    let min = (MIN_SHARED_S / HASH_SECONDS) as usize;
    let ((start, end), _) = tokio::task::spawn_blocking(move || detect::shared(&a, &b, min))
        .await
        .ok()??;
    let (from, _) = part.window(file.duration_s?);
    Some(Span {
        start: from + start as f64 * HASH_SECONDS,
        end: from + end as f64 * HASH_SECONDS,
    })
}

/// A file part's fingerprint, from the cache or ffmpeg. A file without audio caches an empty one, so it is not
/// decoded again.
async fn print(
    state: &AppState,
    bins: &MediaBinaries,
    file: &MarkerFile,
    part: Part,
) -> Option<Vec<u32>> {
    let cached = print_path(state, file, part);
    if let Ok(bytes) = tokio::fs::read(&cached).await {
        return Some(
            bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| u32::from_le_bytes(*b))
                .collect(),
        );
    }
    let (from, secs) = part.window(file.duration_s.filter(|d| *d > 0.0)?);
    let print = match slot(
        state,
        detect::audio_print(bins, Path::new(&file.path), from, secs),
    )
    .await?
    {
        Ok(p) => p,
        Err(e) => {
            warn!("audio fingerprint of {}: {e}", file.path);
            Vec::new()
        }
    };
    let bytes: Vec<u8> = print.iter().flat_map(|h| h.to_le_bytes()).collect();
    if let Err(e) = tokio::fs::write(&cached, bytes).await {
        warn!("caching fingerprint {}: {e}", cached.display());
    }
    Some(print)
}

fn print_path(state: &AppState, file: &MarkerFile, part: Part) -> PathBuf {
    state
        .cache_dir("prints")
        .join(format!("{}-{}-{}.bin", file.id, file.mtime, part.name()))
}

/// Runs `work` holding the background media slot; None when the server is shutting down.
async fn slot<T>(state: &AppState, work: impl std::future::Future<Output = T>) -> Option<T> {
    let _permit = state.background_slot(&state.thumb_slots).await.ok()?;
    Some(work.await)
}
