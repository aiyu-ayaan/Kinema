//! Change detection by polling, for folders that send no filesystem events (NFS, SMB).
//!
//! A snapshot keeps one hash per directory: its entry names plus size and mtime of video files.
//! Comparing two snapshots names the directories that gained, lost or changed something, so a poll
//! costs one directory listing per folder plus a stat per video, and nothing is kept per file.
//! The walk stops after `MAX_ENTRIES` entries, which bounds both time and memory on huge shares.

use crate::is_video;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Upper bound on entries (files and folders) looked at per root and poll.
pub const MAX_ENTRIES: usize = 500_000;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Snapshot {
    dirs: BTreeMap<PathBuf, u64>,
    /// The root was a readable folder. A share that failed to mount is not a change.
    pub reachable: bool,
    /// The walk hit `MAX_ENTRIES`; what lies beyond is not watched.
    pub truncated: bool,
}

impl Snapshot {
    pub fn dir_count(&self) -> usize {
        self.dirs.len()
    }
}

/// Directories that differ between two snapshots.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diff {
    pub added: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
    pub changed: Vec<PathBuf>,
}

impl Diff {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }
}

type Listing = Vec<(String, bool, u64, i64)>;

/// Blocking: call from `spawn_blocking`.
pub fn snapshot(root: &Path, max_entries: usize) -> Snapshot {
    let mut snap = Snapshot {
        reachable: root.is_dir(),
        ..Snapshot::default()
    };
    if !snap.reachable {
        return snap;
    }
    let mut listings: BTreeMap<PathBuf, Listing> = BTreeMap::new();
    let mut seen = 0usize;
    let walk = WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            e.depth() == 0
                || (!e.file_name().to_string_lossy().starts_with('.')
                    && !kinema_core::platform::is_virtual_fs(e.path()))
        })
        .filter_map(Result::ok);
    for entry in walk {
        if entry.file_type().is_dir() {
            listings.entry(entry.path().to_path_buf()).or_default();
        }
        if entry.depth() == 0 {
            continue;
        }
        seen += 1;
        if seen > max_entries {
            snap.truncated = true;
            break;
        }
        let Some(parent) = entry.path().parent() else {
            continue;
        };
        let is_dir = entry.file_type().is_dir();
        // Only videos are stat-ed: that is what a scan would pick up or re-probe.
        let (size, mtime) = if !is_dir && is_video(entry.path()) {
            entry
                .metadata()
                .map(|m| {
                    let mtime = m
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map_or(0, |d| d.as_secs() as i64);
                    (m.len(), mtime)
                })
                .unwrap_or((0, 0))
        } else {
            (0, 0)
        };
        listings.entry(parent.to_path_buf()).or_default().push((
            entry.file_name().to_string_lossy().into_owned(),
            is_dir,
            size,
            mtime,
        ));
    }
    for (dir, mut names) in listings {
        names.sort();
        let mut h = std::hash::DefaultHasher::new();
        names.hash(&mut h);
        snap.dirs.insert(dir, h.finish());
    }
    snap
}

pub fn diff(old: &Snapshot, new: &Snapshot) -> Diff {
    let mut d = Diff::default();
    let keys: BTreeSet<&PathBuf> = old.dirs.keys().chain(new.dirs.keys()).collect();
    for k in keys {
        match (old.dirs.get(k), new.dirs.get(k)) {
            (None, Some(_)) => d.added.push(k.clone()),
            (Some(_), None) => d.removed.push(k.clone()),
            (Some(a), Some(b)) if a != b => d.changed.push(k.clone()),
            _ => {}
        }
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct Tmp(PathBuf);
    impl Tmp {
        fn new() -> Self {
            static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let p = std::env::temp_dir().join(format!("kinema-poll-{}-{n}", std::process::id()));
            fs::create_dir_all(&p).expect("tmp dir");
            Tmp(p)
        }
    }
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn unchanged_tree_has_no_diff() {
        let t = Tmp::new();
        fs::create_dir_all(t.0.join("Show/Season 1")).expect("dirs");
        fs::write(t.0.join("Show/Season 1/e1.mkv"), b"x").expect("file");
        let a = snapshot(&t.0, 1000);
        assert!(a.reachable && !a.truncated);
        assert_eq!(a.dir_count(), 3);
        assert!(diff(&a, &snapshot(&t.0, 1000)).is_empty());
    }

    #[test]
    fn new_removed_and_resized_videos_are_found() {
        let t = Tmp::new();
        fs::create_dir_all(t.0.join("A")).expect("dirs");
        fs::write(t.0.join("A/one.mkv"), b"x").expect("file");
        let base = snapshot(&t.0, 1000);

        fs::write(t.0.join("A/two.mp4"), b"x").expect("file");
        let d = diff(&base, &snapshot(&t.0, 1000));
        assert_eq!(d.changed, [t.0.join("A")]);

        let base = snapshot(&t.0, 1000);
        fs::write(t.0.join("A/one.mkv"), b"longer now").expect("rewrite");
        assert_eq!(diff(&base, &snapshot(&t.0, 1000)).changed, [t.0.join("A")]);

        let base = snapshot(&t.0, 1000);
        fs::create_dir_all(t.0.join("B")).expect("dir");
        let d = diff(&base, &snapshot(&t.0, 1000));
        assert_eq!(d.added, [t.0.join("B")]);
        assert_eq!(d.changed, std::slice::from_ref(&t.0));

        let base = snapshot(&t.0, 1000);
        fs::remove_file(t.0.join("A/two.mp4")).expect("rm");
        assert_eq!(diff(&base, &snapshot(&t.0, 1000)).changed, [t.0.join("A")]);
    }

    #[test]
    fn hidden_folders_are_ignored() {
        let t = Tmp::new();
        fs::create_dir_all(t.0.join(".hidden")).expect("dir");
        let base = snapshot(&t.0, 1000);
        fs::write(t.0.join(".hidden/x.mkv"), b"x").expect("file");
        assert!(diff(&base, &snapshot(&t.0, 1000)).is_empty());
    }

    #[test]
    fn walk_is_bounded_and_missing_roots_are_flagged() {
        let t = Tmp::new();
        for i in 0..20 {
            fs::write(t.0.join(format!("f{i}.mkv")), b"x").expect("file");
        }
        assert!(snapshot(&t.0, 5).truncated);
        let gone = snapshot(&t.0.join("not-mounted"), 5);
        assert!(!gone.reachable);
        assert_eq!(gone.dir_count(), 0);
    }
}
