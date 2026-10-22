use crate::{MediaBinaries, MediaError};
use std::path::{Path, PathBuf};

/// Text formats ffmpeg converts to WebVTT. Bitmap subtitles (PGS, VobSub) are not.
pub fn is_text_subtitle(codec: Option<&str>) -> bool {
    matches!(
        codec.unwrap_or_default(),
        "subrip" | "srt" | "ass" | "ssa" | "webvtt" | "vtt" | "mov_text" | "text"
    )
}

/// Picture formats that only OCR can turn into WebVTT.
pub fn is_image_subtitle(codec: Option<&str>) -> bool {
    matches!(
        codec.unwrap_or_default(),
        "hdmv_pgs_subtitle" | "pgssub" | "dvd_subtitle" | "dvdsub" | "dvb_subtitle"
    )
}

/// Identifies one version of one subtitle track in the WebVTT cache. A changed video
/// (size or mtime) or a changed sidecar gets a new name, so a stale entry is never served
/// and an unchanged one is never made twice.
#[derive(Debug, Clone, Copy)]
pub struct CacheKey<'a> {
    pub file_id: i64,
    pub size: i64,
    pub mtime: i64,
    /// Stream index inside the video; unused for sidecars.
    pub idx: i64,
    /// Sidecar (path, "size-mtime"). The path is hashed because sidecar indexes are
    /// positions that shift when another sidecar is added.
    pub sidecar: Option<(&'a str, &'a str)>,
}

impl CacheKey<'_> {
    pub fn stem(&self) -> String {
        match self.sidecar {
            Some((path, fingerprint)) => {
                format!("{}-s{:016x}-{fingerprint}", self.file_id, fnv1a(path))
            }
            None => format!("{}-{}-{}-{}", self.file_id, self.size, self.mtime, self.idx),
        }
    }

    /// (WebVTT, failure marker) inside `dir`.
    pub fn paths(&self, dir: &Path) -> (PathBuf, PathBuf) {
        let stem = self.stem();
        (
            dir.join(format!("{stem}.vtt")),
            dir.join(format!("{stem}.fail")),
        )
    }
}

/// Stable across runs and platforms, unlike `DefaultHasher`.
fn fnv1a(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Writes subtitle stream `index` of `input` (or the whole file when `index` is None,
/// for sidecar files) to `out` as WebVTT. Writes to a temp file first so a
/// half-written cache entry is never served.
pub async fn extract_vtt(
    bins: &MediaBinaries,
    input: &Path,
    index: Option<i64>,
    out: &Path,
) -> Result<(), MediaError> {
    match index {
        Some(i) => extract_tracks(bins, input, &[(i, out.to_path_buf())]).await,
        None => run_extract(bins, input, &[(None, out.to_path_buf())]).await,
    }
}

/// Writes several embedded text tracks of `input` to WebVTT in one ffmpeg run: the
/// container is read once, however many tracks it holds.
pub async fn extract_tracks(
    bins: &MediaBinaries,
    input: &Path,
    tracks: &[(i64, PathBuf)],
) -> Result<(), MediaError> {
    let tracks: Vec<(Option<i64>, PathBuf)> =
        tracks.iter().map(|(i, p)| (Some(*i), p.clone())).collect();
    run_extract(bins, input, &tracks).await
}

async fn run_extract(
    bins: &MediaBinaries,
    input: &Path,
    tracks: &[(Option<i64>, PathBuf)],
) -> Result<(), MediaError> {
    if tracks.is_empty() {
        return Ok(());
    }
    let tmps: Vec<PathBuf> = tracks
        .iter()
        .map(|(_, out)| out.with_extension("vtt.tmp"))
        .collect();
    let mut cmd = std::process::Command::new(&bins.ffmpeg);
    cmd.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"])
        .arg(input);
    for ((index, _), tmp) in tracks.iter().zip(&tmps) {
        if let Some(i) = index {
            cmd.arg("-map").arg(format!("0:{i}"));
        }
        cmd.args(["-f", "webvtt"]).arg(tmp);
    }
    kinema_core::platform::configure_low_priority_process(&mut cmd);
    let output = tokio::process::Command::from(cmd)
        .kill_on_drop(true)
        .output()
        .await?;
    if !output.status.success() {
        for tmp in &tmps {
            let _ = tokio::fs::remove_file(tmp).await;
        }
        return Err(MediaError::ProbeFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    for ((_, out), tmp) in tracks.iter().zip(&tmps) {
        tokio::fs::rename(tmp, out).await?;
    }
    Ok(())
}

/// One timed caption.
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

/// "HH:MM:SS.mmm".
fn stamp(t: f64) -> String {
    let ms = (t.max(0.0) * 1000.0).round() as u64;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        ms % 1000
    )
}

/// Escapes what WebVTT gives meaning to in cue text.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Serializes cues as a WebVTT document.
pub fn to_vtt(cues: &[Cue]) -> String {
    let mut out = String::from("WEBVTT\n");
    for c in cues {
        out.push_str(&format!(
            "\n{} --> {}\n{}\n",
            stamp(c.start),
            stamp(c.end),
            escape(&c.text)
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_changes_with_the_file_version() {
        let key = CacheKey {
            file_id: 7,
            size: 100,
            mtime: 5,
            idx: 3,
            sidecar: None,
        };
        assert_eq!(key.stem(), "7-100-5-3");
        let changed = CacheKey { mtime: 6, ..key };
        assert_ne!(key.stem(), changed.stem());
        let (vtt, fail) = key.paths(Path::new("/c"));
        assert_eq!(vtt, Path::new("/c/7-100-5-3.vtt"));
        assert_eq!(fail, Path::new("/c/7-100-5-3.fail"));
    }

    #[test]
    fn sidecar_key_follows_path_and_version_not_position() {
        let a = CacheKey {
            file_id: 7,
            size: 100,
            mtime: 5,
            idx: 10_000,
            sidecar: Some(("/m/Film.en.srt", "10-1")),
        };
        let moved = CacheKey { idx: 10_001, ..a };
        assert_eq!(a.stem(), moved.stem());
        let edited = CacheKey {
            sidecar: Some(("/m/Film.en.srt", "11-2")),
            ..a
        };
        assert_ne!(a.stem(), edited.stem());
        let other = CacheKey {
            sidecar: Some(("/m/Film.fr.srt", "10-1")),
            ..a
        };
        assert_ne!(a.stem(), other.stem());
        // Fixed hash: cache names must survive restarts and upgrades.
        assert_eq!(fnv1a("a"), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn writes_webvtt() {
        let vtt = to_vtt(&[
            Cue {
                start: 1.0,
                end: 3.5,
                text: "Hello <there> & you".into(),
            },
            Cue {
                start: 3725.25,
                end: 3726.0,
                text: "Two\nlines".into(),
            },
        ]);
        assert_eq!(
            vtt,
            "WEBVTT\n\n00:00:01.000 --> 00:00:03.500\nHello &lt;there&gt; &amp; you\n\n\
             01:02:05.250 --> 01:02:06.000\nTwo\nlines\n"
        );
    }

    #[test]
    fn kinds() {
        assert!(is_text_subtitle(Some("subrip")));
        assert!(!is_text_subtitle(Some("hdmv_pgs_subtitle")));
        assert!(is_image_subtitle(Some("hdmv_pgs_subtitle")));
        assert!(is_image_subtitle(Some("dvd_subtitle")));
        assert!(!is_image_subtitle(Some("ass")));
        assert!(!is_image_subtitle(None));
    }

    /// Real ffmpeg: one run turns an SRT and a styled ASS track into WebVTT.
    /// Skipped where ffmpeg is not installed.
    #[tokio::test]
    async fn extracts_text_tracks_in_one_run() {
        let Ok(bins) = MediaBinaries::resolve() else {
            return;
        };
        let dir = std::env::temp_dir().join(format!("kinema-subs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let srt = dir.join("a.srt");
        std::fs::write(&srt, "1\n00:00:01,000 --> 00:00:02,500\nPlain line\n\n").unwrap();
        let ass = dir.join("b.ass");
        std::fs::write(
            &ass,
            "[Script Info]\nScriptType: v4.00+\n\n[V4+ Styles]\n\
             Format: Name, Fontname, Fontsize, PrimaryColour, Bold, Italic\n\
             Style: Default,Arial,20,&H00FFFFFF,0,0\n\n[Events]\n\
             Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n\
             Dialogue: 0,0:00:03.00,0:00:04.00,Default,,0,0,0,,{\\i1}Styled{\\i0} line\n",
        )
        .unwrap();
        let mkv = dir.join("v.mkv");
        let ok = std::process::Command::new(&bins.ffmpeg)
            .args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
            .arg(&srt)
            .arg("-i")
            .arg(&ass)
            .args(["-map", "0", "-map", "1", "-c:s:0", "srt", "-c:s:1", "ass"])
            .arg(&mkv)
            .status()
            .is_ok_and(|s| s.success());
        assert!(ok, "ffmpeg could not build the test file");

        let (a, b) = (dir.join("0.vtt"), dir.join("1.vtt"));
        extract_tracks(&bins, &mkv, &[(0, a.clone()), (1, b.clone())])
            .await
            .unwrap();
        let a = std::fs::read_to_string(a).unwrap();
        let b = std::fs::read_to_string(b).unwrap();
        assert!(a.starts_with("WEBVTT"));
        assert!(a.contains("00:01.000 --> 00:02.500") && a.contains("Plain line"));
        assert!(b.contains("Styled") && !b.contains("\\i1"));

        let side = dir.join("side.vtt");
        extract_vtt(&bins, &srt, None, &side).await.unwrap();
        assert!(
            std::fs::read_to_string(side)
                .unwrap()
                .contains("Plain line")
        );
        assert!(!dir.join("side.vtt.tmp").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
