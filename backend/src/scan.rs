//! Library scan: walk roots, add new files, probe changed ones, drop missing ones,
//! then fetch metadata for anything not looked up yet.

use crate::state::AppState;
use kinema_db::items::{self, Meta, NewItem, NewStream};
use kinema_db::{libraries, settings};
use kinema_meta::TmdbClient;
use kinema_scanner::{FoundFile, parse};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinSet;
use tracing::{info, warn};

pub fn is_scanning(state: &AppState, library_id: i64) -> bool {
    state.scans.is_running(library_id)
}

/// Starts a scan, or asks the running one to go again when it finishes.
/// Thumbnails for whatever the scan found follow in their own background job.
pub fn spawn_scan(state: AppState, library_id: i64) {
    let jobs = state.scans.clone();
    jobs.spawn(library_id, move || {
        let state = state.clone();
        async move {
            scan(&state, library_id).await?;
            crate::thumbs::spawn(state, library_id);
            Ok(())
        }
    });
}

/// Full rescans every `rescan_hours`, for drives that do not report changes
/// (network shares). Checks the setting every 5 minutes.
pub async fn scheduled_rescans(state: AppState) {
    let mut last = std::time::Instant::now();
    loop {
        tokio::time::sleep(Duration::from_secs(300)).await;
        let hours = state.settings().rescan_hours;
        if hours > 0 && last.elapsed() >= Duration::from_secs(u64::from(hours) * 3600) {
            last = std::time::Instant::now();
            if let Ok(libs) = libraries::list(&state.db).await {
                for lib in libs {
                    spawn_scan(state.clone(), lib.id);
                }
            }
        }
    }
}

/// Turns bursts of filesystem events into one scan per library, 10 s after the last event.
pub async fn debounce_changes(state: AppState, mut rx: UnboundedReceiver<i64>) {
    let mut pending = HashSet::new();
    loop {
        let next = if pending.is_empty() {
            rx.recv().await
        } else {
            match tokio::time::timeout(Duration::from_secs(10), rx.recv()).await {
                Ok(msg) => msg,
                Err(_) => {
                    for id in pending.drain() {
                        spawn_scan(state.clone(), id);
                    }
                    continue;
                }
            }
        };
        match next {
            Some(id) => {
                pending.insert(id);
            }
            None => return,
        }
    }
}

/// (Re)starts the filesystem watcher for a library, if watching is on.
pub fn watch(state: &AppState, library_id: i64, roots: &[String]) {
    if !state.settings().watch_changes {
        return;
    }
    let roots: Vec<PathBuf> = roots
        .iter()
        .map(PathBuf::from)
        .filter(|r| kinema_core::platform::is_allowed_library_root(r))
        .collect();
    match kinema_scanner::watch::watch_library(library_id, &roots, state.changes.clone()) {
        Ok(w) => {
            if let Ok(mut watchers) = state.watchers.lock() {
                watchers.insert(library_id, w);
            }
        }
        Err(e) => warn!("cannot watch library {library_id}: {e}"),
    }
}

/// Applies the watch setting to every library.
pub async fn rewatch_all(state: &AppState) {
    if let Ok(mut watchers) = state.watchers.lock() {
        watchers.clear();
    }
    if let Ok(libs) = libraries::list(&state.db).await {
        for lib in libs {
            watch(state, lib.id, &lib.roots);
        }
    }
}

pub fn unwatch(state: &AppState, library_id: i64) {
    if let Ok(mut watchers) = state.watchers.lock() {
        watchers.remove(&library_id);
    }
}

async fn scan(state: &AppState, library_id: i64) -> anyhow::Result<()> {
    let Some(lib) = libraries::get(&state.db, library_id).await? else {
        return Ok(());
    };
    // Libraries saved before root validation may still list "/" or /proc.
    let roots: Vec<PathBuf> = lib
        .roots
        .iter()
        .map(PathBuf::from)
        .filter(|r| {
            let ok = kinema_core::platform::is_allowed_library_root(r);
            if !ok {
                warn!(
                    "library {}: skipping folder {} (whole drive or system folder)",
                    lib.name,
                    r.display()
                );
            }
            ok
        })
        .collect();
    let walk_roots = roots.clone();
    let found: Vec<(PathBuf, FoundFile)> = tokio::task::spawn_blocking(move || {
        walk_roots
            .iter()
            .flat_map(|root| {
                kinema_scanner::walk_videos(root)
                    .into_iter()
                    .map(|f| (root.clone(), f))
            })
            .collect()
    })
    .await?;

    let series_dirs = numbered_series_dirs(&found);
    let mut seen = HashSet::new();
    let mut to_probe = Vec::new();
    let mut added = 0usize;
    let mut moved = 0usize;
    for (root, f) in &found {
        let path = f.path.to_string_lossy().into_owned();
        seen.insert(path.clone());
        // shortcut: size + mtime fingerprint; add a head/tail hash to follow renames (M1 spec).
        let fingerprint = format!("{}-{}", f.size, f.mtime);
        match items::file_by_path(&state.db, &path).await? {
            Some(existing) if existing.size == f.size as i64 && existing.mtime == f.mtime => {
                if existing.probe_status == "pending" {
                    to_probe.push((existing.id, f.path.clone()));
                }
                // Filed as a movie by an older parser: move it under its show.
                if let (Some(ep), Some(item_id)) = (
                    episode_of(&lib.kind, root, &f.path, &series_dirs),
                    existing.item_id,
                ) && items::kind_of(&state.db, item_id).await?.as_deref() != Some("episode")
                {
                    let episode = create_episode(state, library_id, &ep).await?;
                    items::move_file(&state.db, existing.id, episode).await?;
                    moved += 1;
                }
            }
            Some(existing) => {
                items::reset_file(&state.db, existing.id, f.size as i64, f.mtime, &fingerprint)
                    .await?;
                to_probe.push((existing.id, f.path.clone()));
            }
            None => {
                let item_id =
                    create_item(state, library_id, &lib.kind, root, &f.path, &series_dirs).await?;
                let id = items::insert_file(
                    &state.db,
                    item_id,
                    &path,
                    f.size as i64,
                    f.mtime,
                    &fingerprint,
                )
                .await?;
                to_probe.push((id, f.path.clone()));
                added += 1;
            }
        }
    }

    // A root that failed to mount looks empty; never wipe a library because of it.
    let roots_ok = roots.iter().all(|r| r.is_dir());
    let gone: Vec<i64> = items::library_files(&state.db, library_id)
        .await?
        .into_iter()
        .filter(|(_, p)| !seen.contains(p))
        .map(|(id, _)| id)
        .collect();
    if roots_ok && (!gone.is_empty() || moved > 0) {
        items::delete_files(&state.db, &gone).await?;
    }
    info!(
        "library {} scanned: {} files, {added} new, {moved} regrouped, {} removed, {} to probe",
        lib.name,
        found.len(),
        if roots_ok { gone.len() } else { 0 },
        to_probe.len()
    );

    probe_all(state, to_probe).await;
    if let Err(e) = fetch_metadata(state, library_id).await {
        warn!("metadata for library {}: {e}", lib.name);
    }
    Ok(())
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Folders where at least 3 files are named "<folder name> <number>": a series
/// without SxxEyy tags. Fewer could be a film and its sequel.
fn numbered_series_dirs(found: &[(PathBuf, FoundFile)]) -> HashSet<PathBuf> {
    let mut counts: std::collections::HashMap<&Path, usize> = std::collections::HashMap::new();
    for (root, f) in found {
        if let Some(dir) = f.path.parent().filter(|d| d != root)
            && let Some(name) = dir.file_name()
            && parse::numbered_episode(&file_stem(&f.path), &name.to_string_lossy()).is_some()
        {
            *counts.entry(dir).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .filter(|(_, n)| *n >= 3)
        .map(|(d, _)| d.to_path_buf())
        .collect()
}

/// Episode numbers in the name. Movie libraries detect them too, so a series dropped
/// into a movies folder still groups into Show, Season, Episode.
fn episode_of(
    kind: &str,
    root: &Path,
    path: &Path,
    series_dirs: &HashSet<PathBuf>,
) -> Option<parse::EpisodeName> {
    if kind != "movie" && kind != "show" {
        return None;
    }
    let rel = path.strip_prefix(root).unwrap_or(path);
    let parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let show_folder = (parts.len() >= 2).then(|| parts[0].as_str());
    parse::parse_episode(&file_stem(path), show_folder).or_else(|| {
        let dir = path.parent().filter(|d| series_dirs.contains(*d))?;
        parse::numbered_episode(&file_stem(path), &dir.file_name()?.to_string_lossy())
    })
}

async fn create_episode(
    state: &AppState,
    library_id: i64,
    ep: &parse::EpisodeName,
) -> anyhow::Result<i64> {
    let db = &state.db;
    let show = items::find_or_create(
        db,
        library_id,
        NewItem::Show {
            title: &ep.show,
            year: ep.year,
        },
    )
    .await?;
    let season = items::find_or_create(
        db,
        library_id,
        NewItem::Season {
            show_id: show,
            season: ep.season,
        },
    )
    .await?;
    let title = format!("Episode {}", ep.episode);
    Ok(items::find_or_create(
        db,
        library_id,
        NewItem::Episode {
            season_id: season,
            episode: ep.episode,
            title: &title,
        },
    )
    .await?)
}

async fn create_item(
    state: &AppState,
    library_id: i64,
    kind: &str,
    root: &Path,
    path: &Path,
    series_dirs: &HashSet<PathBuf>,
) -> anyhow::Result<i64> {
    if let Some(ep) = episode_of(kind, root, path, series_dirs) {
        return create_episode(state, library_id, &ep).await;
    }
    let db = &state.db;
    let id = if kind == "movie" {
        let mut m = parse::parse_movie(&file_stem(path));
        // "Inception (2010)/inception-1080p.mkv": the folder carries the real name.
        if m.year.is_none()
            && let Some(parent) = path.parent().filter(|p| *p != root)
        {
            let folder =
                parse::parse_movie(&parent.file_name().unwrap_or_default().to_string_lossy());
            if folder.year.is_some() {
                m = folder;
            }
        }
        items::find_or_create(
            db,
            library_id,
            NewItem::Movie {
                title: &m.title,
                year: m.year,
            },
        )
        .await?
    } else {
        // No episode number, or an "other videos" library: listed as a plain video.
        let title = parse::video_title(path);
        items::find_or_create(db, library_id, NewItem::Video { title: &title }).await?
    };
    Ok(id)
}

async fn probe_all(state: &AppState, files: Vec<(i64, PathBuf)>) {
    let Some(bins) = state.bins.clone() else {
        return;
    };
    let mut set = JoinSet::new();
    for (id, path) in files {
        let state = state.clone();
        let bins = bins.clone();
        set.spawn(async move {
            let Ok(_permit) = state.probe_slots.acquire().await else {
                return;
            };
            let result = match kinema_media::probe::probe(&bins, &path).await {
                Ok(p) => {
                    let mut streams: Vec<NewStream> = p
                        .streams
                        .iter()
                        .map(|s| NewStream {
                            idx: s.index,
                            kind: s.kind.clone(),
                            codec: s.codec.clone(),
                            profile: s.profile.clone(),
                            language: s.language.clone(),
                            title: s.title.clone(),
                            channels: s.channels,
                            width: s.width,
                            height: s.height,
                            is_default: s.is_default,
                            external_path: None,
                        })
                        .collect();
                    let sidecars = {
                        let path = path.clone();
                        tokio::task::spawn_blocking(move || {
                            kinema_scanner::sidecar_subtitles(&path)
                        })
                        .await
                        .unwrap_or_default()
                    };
                    for (n, sc) in sidecars.into_iter().enumerate() {
                        streams.push(NewStream {
                            idx: 10_000 + n as i64,
                            kind: "subtitle".into(),
                            codec: sc
                                .path
                                .extension()
                                .map(|e| e.to_string_lossy().to_lowercase()),
                            language: sc.language,
                            title: sc.title,
                            external_path: Some(sc.path.to_string_lossy().into_owned()),
                            ..Default::default()
                        });
                    }
                    items::save_probe(
                        &state.db,
                        id,
                        p.container.as_deref(),
                        p.duration_s,
                        p.bitrate,
                        &streams,
                    )
                    .await
                }
                Err(e) => {
                    warn!("probe {}: {e}", path.display());
                    items::mark_probe_failed(&state.db, id).await
                }
            };
            if let Err(e) = result {
                warn!("saving probe of {}: {e}", path.display());
            }
        });
    }
    while set.join_next().await.is_some() {}
}

pub async fn tmdb_key(state: &AppState) -> Option<String> {
    settings::get(&state.db, settings::TMDB_API_KEY)
        .await
        .ok()
        .flatten()
        .or_else(|| state.config.tmdb_api_key.clone())
        .filter(|k| !k.trim().is_empty())
}

/// TMDB image path "/abc.jpg" to an image key "w500/abc.jpg".
fn image_key(size: &str, path: Option<String>) -> Option<String> {
    path.map(|p| format!("{size}/{}", p.trim_start_matches('/')))
}

/// TMDB client with the configured key and metadata language.
pub async fn tmdb_client(state: &AppState) -> Option<TmdbClient> {
    let key = tmdb_key(state).await?;
    Some(TmdbClient::new(key).language(state.settings().metadata_language))
}

async fn fetch_metadata(state: &AppState, library_id: i64) -> anyhow::Result<()> {
    let Some(tmdb) = tmdb_client(state).await else {
        return Ok(());
    };
    let db = &state.db;

    for target in items::needing_meta(db, library_id).await? {
        let found = if target.kind == "movie" {
            tmdb.movie(&target.title, target.year).await
        } else {
            tmdb.show(&target.title, target.year).await
        };
        match found {
            Ok(Some(d)) => {
                let meta = Meta {
                    tmdb_id: Some(d.tmdb_id),
                    title: Some(d.title).filter(|t| !t.is_empty()),
                    year: d.year,
                    summary: d.overview,
                    rating: d.rating,
                    runtime_s: d.runtime_min.map(|m| m * 60),
                    poster: image_key("w500", d.poster),
                    backdrop: image_key("w1280", d.backdrop),
                    genres: d.genres,
                    cast: Some(
                        d.cast
                            .into_iter()
                            .map(|c| items::CastMember {
                                id: c.id,
                                name: c.name,
                                role: c.role,
                                image: c.image,
                            })
                            .collect(),
                    ),
                };
                items::save_meta(db, target.id, &meta).await?;
            }
            Ok(None) => items::mark_meta_tried(db, target.id).await?,
            // Network or key trouble: stop, and retry on the next scan.
            Err(e) => return Err(e.into()),
        }
    }

    for (show_id, tmdb_id) in items::shows_with_bare_episodes(db, library_id).await? {
        for (season_id, season_no) in items::numbered_children(db, show_id).await? {
            let season = match tmdb.season(tmdb_id, season_no).await {
                Ok(s) => s,
                Err(e) => {
                    warn!("tmdb season {tmdb_id}/{season_no}: {e}");
                    kinema_meta::Season::default()
                }
            };
            items::save_meta(
                db,
                season_id,
                &Meta {
                    poster: image_key("w500", season.poster),
                    ..Default::default()
                },
            )
            .await?;
            for (episode_id, episode_no) in items::numbered_children(db, season_id).await? {
                match season.episodes.iter().find(|e| e.number == episode_no) {
                    Some(e) => {
                        let meta = Meta {
                            title: Some(e.title.clone()).filter(|t| !t.is_empty()),
                            summary: e.overview.clone(),
                            rating: e.rating,
                            runtime_s: e.runtime_min.map(|m| m * 60),
                            backdrop: image_key("w300", e.still.clone()),
                            ..Default::default()
                        };
                        items::save_meta(db, episode_id, &meta).await?;
                    }
                    None => items::mark_meta_tried(db, episode_id).await?,
                }
            }
        }
    }
    Ok(())
}
