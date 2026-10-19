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

async fn probe_lines(bins: &MediaBinaries, args: &[String], path: &Path) -> String {
    Command::new(&bins.ffprobe)
        .args(["-v", "error"])
        .args(args)
        .arg(path)
        .kill_on_drop(true)
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// Where `ffmpeg -ss t` with stream copy really starts, in source seconds. Copy can
/// only cut at a seek point (keyframe or Matroska cue), so the stream begins before
/// `t`; the player uses this as its time offset to stay in sync with subtitles.
/// Asks ffprobe to do the same seek and reads the first packets, so it costs milliseconds.
pub async fn seek_landing(bins: &MediaBinaries, path: &Path, t: f64, streams: &[i64]) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    // ffmpeg's demuxer seeks 3/23 s early when the video has B-frames (fftools/ffmpeg_demux.c).
    let header = probe_lines(
        bins,
        &[
            "-select_streams".into(),
            "v:0".into(),
            "-show_entries".into(),
            "stream=has_b_frames".into(),
            "-of".into(),
            "csv=p=0".into(),
        ],
        path,
    )
    .await;
    let b_frames = header
        .trim()
        .trim_end_matches(',')
        .parse::<i64>()
        .unwrap_or(0)
        > 0;
    let seek = if b_frames { t - 3.0 / 23.0 } else { t };
    let packets = probe_lines(
        bins,
        &[
            "-read_intervals".into(),
            format!("{:.6}%+#24", seek.max(0.0)),
            "-show_entries".into(),
            "packet=stream_index,pts_time".into(),
            "-of".into(),
            "csv=p=0".into(),
        ],
        path,
    )
    .await;
    landing_from_packets(&packets, streams).unwrap_or(t)
}

/// First timestamp among the mapped streams: `avoid_negative_ts make_zero` turns it into 0.
fn landing_from_packets(csv: &str, streams: &[i64]) -> Option<f64> {
    csv.lines()
        .filter_map(|l| {
            let mut f = l.trim().trim_end_matches(',').split(',');
            let idx: i64 = f.next()?.parse().ok()?;
            let pts: f64 = f.next()?.parse().ok()?;
            streams.contains(&idx).then_some(pts)
        })
        .reduce(f64::min)
}

/// ffmpeg writing fragmented MP4 to stdout, starting at `start_s`. Seeking means
/// a new request with a new start; the process dies when the client disconnects.
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
        cmd.arg("-ss").arg(format!("{start_s:.6}"));
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
    fn landing_ignores_unmapped_streams() {
        let csv = "0,6.000000\n1,5.921000\n2,5.100000\n0,6.041000\n";
        assert_eq!(landing_from_packets(csv, &[0, 1]), Some(5.921));
        assert_eq!(landing_from_packets(csv, &[0]), Some(6.0));
        assert_eq!(landing_from_packets("", &[0]), None);
    }

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
