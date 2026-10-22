pub mod error;
pub mod parse;
pub mod poll;
pub mod watch;

pub use error::ScannerError;
use std::collections::HashMap;
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

/// Video files under `root`, and the subtitle files beside them.
#[derive(Debug, Default)]
pub struct MediaWalk {
    pub videos: Vec<FoundFile>,
    pub subtitles: Vec<FoundFile>,
}

/// Walks `root` once for videos and subtitle files, so a subtitle added next to an
/// unchanged video is seen by the same pass. Blocking: call from `spawn_blocking`.
/// Hidden folders and macOS "._" files are skipped.
pub fn walk_media(root: &Path) -> MediaWalk {
    let mut out = MediaWalk::default();
    let entries = WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            e.depth() == 0
                || (!e.file_name().to_string_lossy().starts_with('.')
                    && !kinema_core::platform::is_virtual_fs(e.path()))
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file());
    for e in entries {
        let video = is_video(e.path());
        if !video && !has_ext(e.path(), SUBTITLE_EXTENSIONS) {
            continue;
        }
        let Some(found) = found_file(&e) else {
            continue;
        };
        if video {
            out.videos.push(found);
        } else {
            out.subtitles.push(found);
        }
    }
    out
}

fn found_file(e: &walkdir::DirEntry) -> Option<FoundFile> {
    let meta = e.metadata().ok()?;
    let mtime = meta
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    Some(FoundFile {
        path: e.path().to_path_buf(),
        size: meta.len(),
        mtime,
    })
}

/// Sidecar subtitle next to a video: "Movie.srt", "Movie.en.srt", "Movie.eng.forced.srt".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sidecar {
    pub path: PathBuf,
    pub language: Option<String>,
    pub title: Option<String>,
    /// "size-mtime": a changed subtitle file gets a new cache entry.
    pub fingerprint: String,
}

/// Subtitle files grouped by folder, for matching against the videos of the same walk.
pub fn subtitles_by_dir(subs: &[FoundFile]) -> HashMap<&Path, Vec<&FoundFile>> {
    let mut by_dir: HashMap<&Path, Vec<&FoundFile>> = HashMap::new();
    for f in subs {
        if let Some(dir) = f.path.parent() {
            by_dir.entry(dir).or_default().push(f);
        }
    }
    by_dir
}

/// The sidecars of `video` among the subtitle files of its folder, sorted by path.
pub fn sidecars_for(video: &Path, subs_in_dir: &[&FoundFile]) -> Vec<Sidecar> {
    let Some(stem) = video.file_stem() else {
        return Vec::new();
    };
    let stem = stem.to_string_lossy();
    let mut out: Vec<Sidecar> = subs_in_dir
        .iter()
        .filter(|f| f.path.parent() == video.parent())
        .filter_map(|f| {
            let sub_stem = f.path.file_stem()?.to_string_lossy().into_owned();
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
                path: f.path.clone(),
                language,
                title,
                fingerprint: format!("{}-{}", f.size, f.mtime),
            })
        })
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sub(path: &str) -> FoundFile {
        FoundFile {
            path: PathBuf::from(path),
            size: 12,
            mtime: 34,
        }
    }

    #[test]
    fn matches_sidecars_by_basename() {
        let files = [
            sub("/m/Movie.srt"),
            sub("/m/Movie.en.srt"),
            sub("/m/Movie.eng.forced.ass"),
            sub("/m/Movie 2.srt"),
            sub("/m/Moviex.srt"),
            sub("/other/Movie.srt"),
        ];
        let refs: Vec<&FoundFile> = files.iter().collect();
        let found = sidecars_for(Path::new("/m/Movie.mkv"), &refs);
        let names: Vec<_> = found
            .iter()
            .map(|s| s.path.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            names,
            ["/m/Movie.en.srt", "/m/Movie.eng.forced.ass", "/m/Movie.srt"]
        );
        assert_eq!(found[0].language.as_deref(), Some("en"));
        assert_eq!(found[1].language.as_deref(), Some("eng"));
        assert_eq!(found[1].title.as_deref(), Some("FORCED"));
        assert_eq!(found[2].language, None);
        assert_eq!(found[0].fingerprint, "12-34");
    }

    #[test]
    fn walk_finds_subtitles_beside_videos() {
        let dir = std::env::temp_dir().join(format!("kinema-walk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".hidden")).unwrap();
        for f in ["Film.mkv", "Film.en.SRT", "notes.txt", ".hidden/Film.srt"] {
            std::fs::write(dir.join(f), b"x").unwrap();
        }
        let walk = walk_media(&dir);
        assert_eq!(walk.videos.len(), 1);
        assert_eq!(walk.subtitles.len(), 1);
        let by_dir = subtitles_by_dir(&walk.subtitles);
        let in_dir = by_dir.get(dir.as_path()).cloned().unwrap_or_default();
        let found = sidecars_for(&walk.videos[0].path, &in_dir);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].language.as_deref(), Some("en"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
