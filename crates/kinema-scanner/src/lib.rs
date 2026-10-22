pub mod error;
pub mod parse;
pub mod poll;
pub mod watch;

pub use error::ScannerError;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const VIDEO_EXTENSIONS: &[&str] = &[
    "mkv", "mp4", "m4v", "mov", "avi", "wmv", "webm", "ts", "m2ts", "mts", "mpg", "mpeg", "flv",
    "ogv", "3gp", "vob", "divx", "rmvb", "asf",
];

const SUBTITLE_EXTENSIONS: &[&str] = &["srt", "vtt", "ass", "ssa"];

fn has_ext(path: &Path, list: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| list.contains(&e.to_ascii_lowercase().as_str()))
}

pub fn is_video(path: &Path) -> bool {
    has_ext(path, VIDEO_EXTENSIONS)
}

/// A file seen on disk.
#[derive(Debug, Clone)]
pub struct FoundFile {
    pub path: PathBuf,
    pub size: u64,
    pub mtime: i64,
}

/// Video files under `root`. Blocking: call from `spawn_blocking`.
/// Hidden folders and macOS "._" files are skipped.
pub fn walk_videos(root: &Path) -> Vec<FoundFile> {
    WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            e.depth() == 0
                || (!e.file_name().to_string_lossy().starts_with('.')
                    && !kinema_core::platform::is_virtual_fs(e.path()))
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file() && is_video(e.path()))
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            let mtime = meta
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs() as i64;
            Some(FoundFile {
                path: e.into_path(),
                size: meta.len(),
                mtime,
            })
        })
        .collect()
}

/// Sidecar subtitle next to a video: "Movie.srt", "Movie.en.srt", "Movie.eng.forced.srt".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sidecar {
    pub path: PathBuf,
    pub language: Option<String>,
    pub title: Option<String>,
}

pub fn sidecar_subtitles(video: &Path) -> Vec<Sidecar> {
    let (Some(dir), Some(stem)) = (video.parent(), video.file_stem()) else {
        return Vec::new();
    };
    let stem = stem.to_string_lossy();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<Sidecar> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| has_ext(p, SUBTITLE_EXTENSIONS))
        .filter_map(|p| {
            let sub_stem = p.file_stem()?.to_string_lossy().into_owned();
            let rest = sub_stem.strip_prefix(stem.as_ref())?;
            if !(rest.is_empty() || rest.starts_with('.')) {
                return None;
            }
            let tags: Vec<&str> = rest.split('.').filter(|t| !t.is_empty()).collect();
            let language = tags
                .iter()
                .find(|t| (2..=3).contains(&t.len()) && t.chars().all(|c| c.is_ascii_alphabetic()))
                .map(|t| t.to_lowercase());
            let title = tags
                .iter()
                .find(|t| ["forced", "sdh", "cc"].contains(&t.to_lowercase().as_str()))
                .map(|t| t.to_uppercase());
            Some(Sidecar {
                path: p,
                language,
                title,
            })
        })
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}
