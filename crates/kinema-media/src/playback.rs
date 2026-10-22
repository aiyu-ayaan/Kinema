//! Direct play first, remux (copy) second, re-encode last.

use crate::MediaBinaries;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, Command};

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
    // vorbis does not go into MP4; the rest of audio_ok does. A re-encoded video starts exactly at
    // the seek point, but copied audio can only start at the keyframe before it: the video then has
    // a hole of up to a keyframe interval (5.6 s on F1) and the browser freezes the picture over it
    // while the audio clock runs. Re-encoding the audio too (AAC, a sliver of x264's CPU) cuts both
    // at the same frame.
    let reencode_video = video.is_some() && !video_copy;
    let audio_copy =
        !reencode_video && audio.is_none_or(|a| audio_ok(a.codec) && a.codec != Some("vorbis"));
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
/// Re-encode settings, from the admin's Transcoder options.
#[derive(Debug, Clone, Copy)]
pub struct Encode<'a> {
    pub preset: &'a str,
    pub crf: u32,
    /// 0 keeps the source height.
    pub max_height: u32,
    /// Decoder and encoder threads; 0 lets ffmpeg take every core.
    pub threads: u32,
}

/// Peak video bitrate in kbit/s for an output height (0 = source, counted as 4K). CRF alone lets a
/// grainy or fast scene take several times the average; the cap keeps those scenes from spiking the
/// encoder's work and the network.
fn max_kbps(max_height: u32) -> u32 {
    match max_height {
        1..=480 => 2_000,
        481..=720 => 4_000,
        721..=1080 => 8_000,
        1081..=1440 => 12_000,
        _ => 20_000,
    }
}

/// Threads for one re-encode: an equal share of the cores per allowed conversion, so the cap on
/// conversions also caps CPU. `configured` 0 is automatic. Never fewer than 4: a Raspberry Pi 5
/// encodes 1080p at only about 1.4x real time on all four cores, so a smaller share would stall it.
pub fn transcode_threads(configured: u32, max_transcodes: u32) -> u32 {
    if configured > 0 {
        return configured;
    }
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get() as u32);
    (cores / max_transcodes.max(1)).max(4)
}

/// How far a re-encode may run ahead of the wall clock. The browser holds that much, so a seek
/// inside it stays local; past it ffmpeg keeps pace with playback instead of taking every core.
/// Every play and seek encodes this lead at full speed, so a shorter lead is a shorter CPU spike.
pub const THROTTLE_AHEAD_S: f64 = 30.0;

/// Seconds of output a re-encode has written, read from ffmpeg's `-progress` lines on stderr.
#[derive(Debug, Clone, Default)]
pub struct Progress(Arc<AtomicU64>);

/// Stored in place of the time once ffmpeg is done: nothing left to throttle.
const DONE: u64 = 0;

impl Progress {
    pub fn watch(stderr: ChildStderr) -> Self {
        let progress = Self::default();
        let out = progress.0.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(us) = line.strip_prefix("out_time_us=") {
                    if let Ok(us) = us.trim().parse::<i64>() {
                        out.store((us.max(0) as u64 / 1000).max(1), Ordering::Relaxed);
                    }
                } else if line == "progress=end" {
                    out.store(DONE, Ordering::Relaxed);
                }
            }
            out.store(DONE, Ordering::Relaxed);
        });
        progress
    }

    /// Waits while the output is more than [THROTTLE_AHEAD_S] ahead of `started`. Not reading the
    /// pipe meanwhile blocks ffmpeg's next write, which pauses the encode.
    pub async fn pace(&self, started: Instant) {
        while self.ahead_of(started.elapsed().as_secs_f64()) {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    fn ahead_of(&self, elapsed_s: f64) -> bool {
        let ms = self.0.load(Ordering::Relaxed);
        ms != DONE && ms as f64 / 1000.0 - elapsed_s > THROTTLE_AHEAD_S
    }
}

#[allow(clippy::too_many_arguments)]
pub fn stream_command(
    bins: &MediaBinaries,
    path: &Path,
    start_s: f64,
    video: Option<Track>,
    audio: Option<Track>,
    copy_video: bool,
    copy_audio: bool,
    encode: Encode,
) -> Command {
    let reencode = !copy_video && video.is_some();
    // Decoding 10-bit HEVC is a third of a re-encode's CPU on a Raspberry Pi 5; its GPU does it for free.
    let hw_decode = bins
        .hevc_drm
        .as_ref()
        .filter(|_| reencode && video.is_some_and(|v| v.codec == Some("hevc")));
    let mut cmd = Command::new(hw_decode.unwrap_or(&bins.ffmpeg));
    kinema_core::platform::configure_playback_process(cmd.as_std_mut());
    cmd.args(["-hide_banner", "-loglevel", "error", "-nostdin"]);
    if reencode {
        // Progress for the throttle (Progress::watch); remux is cheap and left to run.
        cmd.args(["-progress", "pipe:2", "-nostats"]);
    }
    if hw_decode.is_some() {
        cmd.args(["-hwaccel", "drm"]);
    }
    if start_s > 0.0 {
        // Copied video can only start at a keyframe before `start_s`. Accurate seek
        // would still trim re-encoded audio to `start_s` exactly, then both land on
        // 0 and the audio runs out of sync; cut audio at the same seek point instead.
        if copy_video && video.is_some() {
            cmd.arg("-noaccurate_seek");
        }
        cmd.arg("-ss").arg(format!("{start_s:.6}"));
    }
    // Decoding a 4K source takes as many threads as encoding; both get the same share.
    let threads = (reencode && encode.threads > 0).then(|| encode.threads.to_string());
    if let Some(n) = &threads {
        cmd.args(["-threads", n]);
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
            // Never upscale; -2 keeps the width even, which libx264 needs.
            let scale = if encode.max_height > 0 {
                format!("scale=-2:'min({},ih)'", encode.max_height)
            } else {
                "scale=-2:ih".to_string()
            };
            let kbps = max_kbps(encode.max_height);
            cmd.args(["-c:v", "libx264", "-preset", encode.preset, "-crf"])
                .arg(encode.crf.to_string())
                .arg("-maxrate")
                .arg(format!("{kbps}k"))
                .arg("-bufsize")
                .arg(format!("{}k", kbps * 2))
                .args(["-pix_fmt", "yuv420p", "-vf"])
                .arg(scale);
            // Slower presets look 20-40 frames ahead; a live stream gains little from more than
            // veryfast's 10, and each lookahead frame is analysed on top of the encode.
            if !matches!(encode.preset, "ultrafast" | "superfast" | "veryfast") {
                cmd.args(["-rc-lookahead", "10"]);
            }
            if let Some(n) = &threads {
                cmd.args(["-threads", n]);
            }
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
    if reencode {
        // Without it a fragment closes only at the next keyframe, up to 10 s of encoding on a Pi: play and
        // seek waited 6-9 s for the first frames. Half-second fragments start playback in about 2.5 s.
        cmd.args(["-frag_duration", "500000"]);
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
        .stderr(if reencode {
            std::process::Stdio::piped()
        } else {
            std::process::Stdio::null()
        })
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
    fn copied_video_seeks_audio_to_the_same_keyframe() {
        let bins = MediaBinaries {
            ffmpeg: "ffmpeg".into(),
            ffprobe: "ffprobe".into(),
            hevc_drm: None,
        };
        let encode = Encode {
            preset: "veryfast",
            crf: 23,
            max_height: 0,
            threads: 0,
        };
        let has_flag = |start: f64, copy_video: bool| {
            let cmd = stream_command(
                &bins,
                Path::new("a.mkv"),
                start,
                Some(H264),
                Some(DTS),
                copy_video,
                false,
                encode,
            );
            cmd.as_std().get_args().any(|a| a == "-noaccurate_seek")
        };
        assert!(has_flag(600.0, true));
        assert!(!has_flag(600.0, false));
        assert!(!has_flag(0.0, true));
    }

    #[test]
    fn reencode_uses_gpu_decode_and_short_fragments() {
        let bins = MediaBinaries {
            ffmpeg: "ffmpeg".into(),
            ffprobe: "ffprobe".into(),
            hevc_drm: Some("/usr/bin/ffmpeg".into()),
        };
        let encode = Encode {
            preset: "veryfast",
            crf: 23,
            max_height: 0,
            threads: 0,
        };
        let hevc = Track {
            codec: Some("hevc"),
            ..H264
        };
        let run = |video: Track, copy_video: bool| {
            let cmd = stream_command(
                &bins,
                Path::new("a.mkv"),
                0.0,
                Some(video),
                Some(AAC),
                copy_video,
                true,
                encode,
            );
            let std = cmd.as_std();
            (
                std.get_program().to_string_lossy().into_owned(),
                std.get_args().any(|a| a == "drm"),
                std.get_args().any(|a| a == "-frag_duration"),
            )
        };
        assert_eq!(run(hevc, false), ("/usr/bin/ffmpeg".into(), true, true));
        assert_eq!(run(hevc, true), ("ffmpeg".into(), false, false));
        assert_eq!(run(H264, false), ("ffmpeg".into(), false, true));
    }

    #[test]
    fn threads_cap_decode_and_encode_of_a_reencode_only() {
        let bins = MediaBinaries {
            ffmpeg: "ffmpeg".into(),
            ffprobe: "ffprobe".into(),
            hevc_drm: None,
        };
        let encode = Encode {
            preset: "veryfast",
            crf: 23,
            max_height: 0,
            threads: 3,
        };
        let count = |copy_video: bool| {
            let cmd = stream_command(
                &bins,
                Path::new("a.mkv"),
                0.0,
                Some(H264),
                Some(AAC),
                copy_video,
                true,
                encode,
            );
            cmd.as_std().get_args().filter(|a| *a == "-threads").count()
        };
        assert_eq!(count(false), 2);
        assert_eq!(count(true), 0);
        assert_eq!(max_kbps(1080), 8_000);
        assert_eq!(max_kbps(0), 20_000);
        assert_eq!(transcode_threads(5, 2), 5);
        assert!(transcode_threads(0, 8) >= 4);
    }

    #[test]
    fn throttle_holds_only_past_the_lead() {
        let p = Progress::default();
        p.0.store(30_000, Ordering::Relaxed);
        assert!(!p.ahead_of(0.0));
        p.0.store(90_000, Ordering::Relaxed);
        assert!(p.ahead_of(10.0));
        assert!(!p.ahead_of(70.0));
        p.0.store(DONE, Ordering::Relaxed);
        assert!(!p.ahead_of(0.0));
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
            plan(mkv, Some(hevc), Some(AAC), true, caps),
            Plan::Stream {
                copy_video: false,
                copy_audio: false
            }
        );
        // No video to re-encode: copied audio has nothing to drift from.
        assert_eq!(
            plan(mkv, None, Some(AAC), true, caps),
            Plan::Stream {
                copy_video: false,
                copy_audio: true
            }
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
