//! Direct play first, remux (copy) second, re-encode last.

use crate::MediaBinaries;
use std::path::Path;
use tokio::process::Command;

/// Codecs the browser reported beyond the baseline (h264, vp9, aac, mp3, opus).
#[derive(Debug, Clone, Copy, Default)]
pub struct ClientCaps {
    pub hevc: bool,
    pub av1: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Track<'a> {
    pub index: i64,
    pub codec: Option<&'a str>,
    pub profile: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// Serve the file as is with Range support.
    Direct,
    /// Pipe through ffmpeg into fragmented MP4; copy whatever the browser can decode.
    Stream { copy_video: bool, copy_audio: bool },
}

impl Plan {
    pub fn name(self) -> &'static str {
        match self {
            Plan::Direct => "direct",
            Plan::Stream {
                copy_video: true, ..
            } => "remux",
            Plan::Stream { .. } => "transcode",
        }
    }
}

fn video_ok(v: &Track, caps: ClientCaps) -> bool {
    let profile = v.profile.unwrap_or_default();
    match v.codec.unwrap_or_default() {
        // 10-bit and 4:2:2/4:4:4 H.264 do not decode in browsers.
        "h264" => !profile.contains("10") && !profile.contains("4:"),
        "vp9" => true,
        "hevc" => caps.hevc,
        "av1" => caps.av1,
        _ => false,
    }
}

/// Audio the browser decodes from an MP4 or WebM.
fn audio_ok(codec: Option<&str>) -> bool {
    matches!(
        codec.unwrap_or_default(),
        "aac" | "mp3" | "opus" | "vorbis" | "flac"
    )
}

/// `audio_is_first`: browsers only play the first audio track of a file, so any
/// other choice needs a remux.
pub fn plan(
    path: &Path,
    video: Option<Track>,
    audio: Option<Track>,
    audio_is_first: bool,
    caps: ClientCaps,
) -> Plan {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let direct_container = matches!(ext.as_str(), "mp4" | "m4v" | "mov" | "webm");
    let video_copy = video.is_some_and(|v| video_ok(&v, caps));
    let audio_copy = audio.is_none_or(|a| audio_ok(a.codec));
    if direct_container && video_copy && audio_copy && audio_is_first {
        return Plan::Direct;
    }
    // vorbis does not go into MP4; the rest of audio_ok does.
    let audio_copy = audio.is_none_or(|a| audio_ok(a.codec) && a.codec != Some("vorbis"));
    Plan::Stream {
        copy_video: video_copy,
        copy_audio: audio_copy,
    }
}

/// ffmpeg writing fragmented MP4 to stdout, starting at `start_s`. Seeking means
/// a new request with a new start; the process dies when the client disconnects.
/// shortcut: input seek with stream copy lands on the keyframe before `start_s`, so
/// the player can be up to one GOP early; move to HLS with keyframe index (M4) for exact seeks.
pub fn stream_command(
    bins: &MediaBinaries,
    path: &Path,
    start_s: f64,
    video: Option<Track>,
    audio: Option<Track>,
    copy_video: bool,
    copy_audio: bool,
) -> Command {
    let mut cmd = Command::new(&bins.ffmpeg);
    cmd.args(["-hide_banner", "-loglevel", "error", "-nostdin"]);
    if start_s > 0.0 {
        cmd.arg("-ss").arg(format!("{start_s:.3}"));
    }
    cmd.arg("-i").arg(path);
    if let Some(v) = video {
        cmd.arg("-map").arg(format!("0:{}", v.index));
        if copy_video {
            cmd.args(["-c:v", "copy"]);
            if v.codec == Some("hevc") {
                cmd.args(["-tag:v", "hvc1"]);
            }
        } else {
            cmd.args([
                "-c:v",
                "libx264",
                "-preset",
                "veryfast",
                "-crf",
                "22",
                "-pix_fmt",
                "yuv420p",
                "-vf",
                "scale='min(1920,iw)':-2",
            ]);
        }
    }
    if let Some(a) = audio {
        cmd.arg("-map").arg(format!("0:{}", a.index));
        if copy_audio {
            cmd.args(["-c:a", "copy"]);
        } else {
            cmd.args(["-c:a", "aac", "-ac", "2", "-b:a", "192k"]);
        }
    }
    cmd.args([
        "-sn",
        "-dn",
        "-map_metadata",
        "-1",
        "-map_chapters",
        "-1",
        "-avoid_negative_ts",
        "make_zero",
        "-movflags",
        "frag_keyframe+empty_moov+default_base_moof",
        "-f",
        "mp4",
        "pipe:1",
    ]);
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    const H264: Track = Track {
        index: 0,
        codec: Some("h264"),
        profile: Some("High"),
    };
    const AAC: Track = Track {
        index: 1,
        codec: Some("aac"),
        profile: None,
    };
    const DTS: Track = Track {
        index: 2,
        codec: Some("dts"),
        profile: None,
    };

    #[test]
    fn decisions() {
        let caps = ClientCaps::default();
        let mp4 = Path::new("a.mp4");
        let mkv = Path::new("a.mkv");
        assert_eq!(plan(mp4, Some(H264), Some(AAC), true, caps), Plan::Direct);
        assert_eq!(
            plan(mp4, Some(H264), Some(AAC), false, caps).name(),
            "remux"
        );
        assert_eq!(plan(mkv, Some(H264), Some(AAC), true, caps).name(), "remux");
        assert_eq!(
            plan(mkv, Some(H264), Some(DTS), true, caps),
            Plan::Stream {
                copy_video: true,
                copy_audio: false
            }
        );
        let hevc = Track {
            codec: Some("hevc"),
            ..H264
        };
        assert_eq!(
            plan(mkv, Some(hevc), Some(AAC), true, caps).name(),
            "transcode"
        );
        let caps = ClientCaps {
            hevc: true,
            av1: false,
        };
        assert_eq!(plan(mkv, Some(hevc), Some(AAC), true, caps).name(), "remux");
        let hi10 = Track {
            profile: Some("High 10"),
            ..H264
        };
        assert_eq!(
            plan(mp4, Some(hi10), Some(AAC), true, caps).name(),
            "transcode"
        );
    }
}
