//! HLS for remux and transcode: a fixed VOD playlist, segments cut by ffmpeg runs that can start at
//! any segment. A seek inside what is already cut (or about to be) is served from disk; only a seek
//! elsewhere starts a new run. The decisions live here, free of processes and clocks, so they are
//! tested directly; `backend/src/hls.rs` owns the processes.

use crate::MediaBinaries;
use crate::playback::{Encode, Segments, Track, ffmpeg_command};
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::process::Command;

/// Target segment length. Copied video can only cut at keyframes, so its segments run longer.
pub const SEGMENT_S: f64 = 4.0;
/// A session nobody asked for a segment in this long is torn down; a seek back within it is free.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(60);
/// A segment this close past the running cut point is waited for rather than restarted for.
pub const REUSE_AHEAD_S: f64 = 12.0;
/// Segments further than this from the one asked for are deleted, bounding the session on disk.
pub const KEEP_S: f64 = 600.0;

/// Segment start times for copied video: a new segment at the first keyframe at least
/// [SEGMENT_S] after the previous start, so every segment begins on a keyframe.
pub fn copy_segments(keyframes: &[f64], duration: f64) -> Vec<f64> {
    let mut starts = vec![0.0];
    for &k in keyframes {
        let last = starts.last().copied().unwrap_or(0.0);
        if k - last >= SEGMENT_S && k < duration {
            starts.push(k);
        }
    }
    starts
}

/// Segment start times for a re-encode, which places keyframes itself: every [SEGMENT_S].
pub fn grid_segments(duration: f64) -> Vec<f64> {
    let n = (duration / SEGMENT_S).ceil().max(1.0) as usize;
    (0..n).map(|i| i as f64 * SEGMENT_S).collect()
}

/// VOD playlist; `query` (already URL-safe) is repeated on every segment so it reaches the session.
pub fn playlist(starts: &[f64], duration: f64, query: &str) -> String {
    let lengths: Vec<f64> = starts
        .iter()
        .enumerate()
        .map(|(i, s)| starts.get(i + 1).copied().unwrap_or(duration) - s)
        .collect();
    let target = lengths.iter().copied().fold(SEGMENT_S, f64::max).ceil();
    let mut out = format!(
        "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:{target}\n#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-PLAYLIST-TYPE:VOD\n"
    );
    for (i, len) in lengths.iter().enumerate() {
        out.push_str(&format!("#EXTINF:{len:.3},\n{i}.ts?{query}\n"));
    }
    out.push_str("#EXT-X-ENDLIST\n");
    out
}

/// What is on disk and what the current ffmpeg run is doing.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Produced {
    /// Per segment: the run whose file holds it, once complete.
    pub have: Vec<Option<u32>>,
    /// The next segment the live run writes; None when no run is going.
    pub cursor: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fetch {
    /// Complete on disk: serve it.
    Ready(u32),
    /// The live run reaches it soon.
    Wait,
    /// Outside what is cut and what the run will cut soon: start a run at this segment.
    Restart,
}

impl Produced {
    pub fn new(segments: usize) -> Self {
        Self {
            have: vec![None; segments],
            cursor: None,
        }
    }

    pub fn fetch(&self, n: usize, starts: &[f64]) -> Fetch {
        if let Some(Some(run)) = self.have.get(n) {
            return Fetch::Ready(*run);
        }
        match self.cursor {
            Some(c) if c <= n && starts[n] - starts[c.min(starts.len() - 1)] <= REUSE_AHEAD_S => {
                Fetch::Wait
            }
            _ => Fetch::Restart,
        }
    }

    /// A segment `run` finished (from its segment list). Earlier copies win: they are already served.
    pub fn complete(&mut self, run: u32, n: usize) {
        if let Some(slot) = self.have.get_mut(n) {
            slot.get_or_insert(run);
        }
        if self.cursor.is_some_and(|c| c <= n) {
            self.cursor = Some(n + 1);
        }
    }

    /// Segments to delete around a request for `n`, forgotten here; the caller removes the files.
    pub fn prune(&mut self, n: usize, starts: &[f64]) -> Vec<(usize, u32)> {
        let at = starts.get(n).copied().unwrap_or(0.0);
        let mut gone = Vec::new();
        for (i, slot) in self.have.iter_mut().enumerate() {
            if (starts[i] - at).abs() > KEEP_S
                && let Some(run) = slot.take()
            {
                gone.push((i, run));
            }
        }
        gone
    }

    /// The live run is [crate::playback::THROTTLE_AHEAD_S] past the last segment asked for: pause it
    /// until the player catches up, like the progressive stream's throttle.
    pub fn should_pause(&self, requested: usize, starts: &[f64]) -> bool {
        let Some(c) = self.cursor else {
            return false;
        };
        let ahead = starts.get(c).copied().unwrap_or(f64::MAX) - starts[requested];
        ahead > crate::playback::THROTTLE_AHEAD_S
    }
}

/// No segment asked for since `last_access`, for the idle timeout.
pub fn is_idle(last_access: Instant, now: Instant) -> bool {
    now.saturating_duration_since(last_access) >= IDLE_TIMEOUT
}

/// Segments named in a run's `list-{run}.csv` (`{run}-{n}.ts,start,end` per line).
pub fn listed_segments(csv: &str, run: u32) -> Vec<usize> {
    let prefix = format!("{run}-");
    csv.lines()
        .filter_map(|l| {
            l.split(',')
                .next()?
                .strip_prefix(&prefix)?
                .strip_suffix(".ts")?
                .parse()
                .ok()
        })
        .collect()
}

/// An ffmpeg run writing segments from `first` on into `dir`.
#[allow(clippy::too_many_arguments)]
pub fn segment_command(
    bins: &MediaBinaries,
    path: &Path,
    starts: &[f64],
    first: usize,
    run: u32,
    dir: &Path,
    video: Option<Track>,
    audio: Option<Track>,
    copy_video: bool,
    copy_audio: bool,
    encode: Encode,
) -> Command {
    let start = starts.get(first).copied().unwrap_or(0.0);
    // Cut points relative to the run's first packet, just under each keyframe so a rounded index
    // time never pushes the cut to the keyframe after it.
    let cuts: Vec<f64> = if copy_video && video.is_some() {
        starts[first.min(starts.len())..]
            .iter()
            .skip(1)
            .map(|s| (s - start - 0.05).max(0.0))
            .collect()
    } else {
        Vec::new()
    };
    ffmpeg_command(
        bins,
        path,
        start,
        video,
        audio,
        copy_video,
        copy_audio,
        encode,
        Some(Segments {
            dir,
            run,
            first,
            cuts: &cuts,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const STARTS: [f64; 7] = [0.0, 4.0, 8.0, 12.0, 16.0, 20.0, 24.0];

    #[test]
    fn copy_segments_start_on_keyframes() {
        let kf = [0.0, 2.5, 7.0, 8.0, 13.25, 19.0, 21.0, 26.5];
        assert_eq!(copy_segments(&kf, 30.0), vec![0.0, 7.0, 13.25, 19.0, 26.5]);
        assert_eq!(copy_segments(&kf, 20.0), vec![0.0, 7.0, 13.25, 19.0]);
        assert_eq!(grid_segments(9.0), vec![0.0, 4.0, 8.0]);
        assert_eq!(grid_segments(0.0), vec![0.0]);
    }

    #[test]
    fn playlist_lists_every_segment_with_its_length() {
        let p = playlist(&[0.0, 7.0, 13.25], 15.0, "audio=1");
        assert!(p.contains("#EXT-X-TARGETDURATION:7\n"));
        assert!(p.contains("#EXTINF:7.000,\n0.ts?audio=1\n"));
        assert!(p.contains("#EXTINF:1.750,\n2.ts?audio=1\n#EXT-X-ENDLIST"));
    }

    #[test]
    fn seek_reuses_cut_segments_and_restarts_only_outside() {
        let mut p = Produced::new(STARTS.len());
        // Nothing cut and nothing running: the first request starts a run.
        assert_eq!(p.fetch(0, &STARTS), Fetch::Restart);
        p.cursor = Some(0);
        assert_eq!(p.fetch(0, &STARTS), Fetch::Wait);
        p.complete(1, 0);
        p.complete(1, 1);
        assert_eq!(p.cursor, Some(2));
        // Back into what is cut: from disk.
        assert_eq!(p.fetch(0, &STARTS), Fetch::Ready(1));
        // Just ahead of the run (segment 2 at 8 s): wait for it. 12 s ahead is close enough, 16 s is not.
        assert_eq!(p.fetch(5, &STARTS), Fetch::Wait);
        assert_eq!(p.fetch(6, &STARTS), Fetch::Restart);
        // A second run from segment 4; segment 0-1 from run 1 still serve.
        p.cursor = Some(4);
        p.complete(2, 4);
        assert_eq!(p.fetch(1, &STARTS), Fetch::Ready(1));
        assert_eq!(p.fetch(4, &STARTS), Fetch::Ready(2));
        // Behind the live run and never cut: restart.
        assert_eq!(p.fetch(2, &STARTS), Fetch::Restart);
        // The run dies: nothing to wait for.
        p.cursor = None;
        assert_eq!(p.fetch(5, &STARTS), Fetch::Restart);
        // A later run re-cutting a segment does not replace the copy already served.
        p.complete(3, 4);
        assert_eq!(p.have[4], Some(2));
    }

    #[test]
    fn prune_keeps_a_window_around_the_request() {
        let starts: Vec<f64> = (0..400).map(|i| i as f64 * 4.0).collect();
        let mut p = Produced::new(starts.len());
        for n in [0, 10, 200, 340] {
            p.complete(1, n);
        }
        // Asked for 1000 s: 0 and 40 s are over 600 s back; 800 s and 1360 s are kept.
        assert_eq!(p.prune(250, &starts), vec![(0, 1), (10, 1)]);
        assert_eq!(p.have[200], Some(1));
        assert_eq!(p.have[340], Some(1));
        assert_eq!(p.have[0], None);
    }

    #[test]
    fn throttle_pauses_past_the_lead() {
        let starts: Vec<f64> = (0..40).map(|i| i as f64 * 4.0).collect();
        let mut p = Produced::new(starts.len());
        assert!(!p.should_pause(0, &starts));
        p.cursor = Some(7); // 28 s ahead of segment 0
        assert!(!p.should_pause(0, &starts));
        p.cursor = Some(8); // 32 s
        assert!(p.should_pause(0, &starts));
        assert!(!p.should_pause(1, &starts));
    }

    #[test]
    fn idle_after_sixty_seconds() {
        let t0 = Instant::now();
        assert!(!is_idle(t0, t0 + Duration::from_secs(59)));
        assert!(is_idle(t0, t0 + Duration::from_secs(60)));
        // A clock that went backwards never reaps.
        assert!(!is_idle(t0 + Duration::from_secs(5), t0));
    }

    #[test]
    fn segment_list_names_map_to_numbers() {
        let csv = "3-12.ts,0.000000,7.499000\n3-13.ts,7.5,13.9\n2-14.ts,1,2\n";
        assert_eq!(listed_segments(csv, 3), vec![12, 13]);
    }

    #[test]
    fn copied_runs_keep_source_times_and_cut_on_keyframes() {
        let bins = MediaBinaries {
            ffmpeg: "ffmpeg".into(),
            ffprobe: "ffprobe".into(),
            hevc_drm: None,
            hw_encoder: None,
        };
        let encode = Encode {
            preset: "veryfast",
            crf: 23,
            max_height: 0,
            threads: 0,
            hardware: false,
        };
        let video = Track {
            index: 0,
            codec: Some("h264"),
            profile: Some("High"),
        };
        let args = |copy: bool| {
            let cmd = segment_command(
                &bins,
                Path::new("a.mkv"),
                &[0.0, 7.0, 13.25, 19.0],
                1,
                5,
                Path::new("/tmp/s"),
                Some(video),
                None,
                copy,
                false,
                encode,
            );
            cmd.as_std()
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        };
        let copy = args(true);
        let at = |v: &[String], flag: &str| {
            v.iter()
                .position(|a| a == flag)
                .and_then(|i| v.get(i + 1).cloned())
        };
        assert_eq!(at(&copy, "-ss").as_deref(), Some("7.200000"));
        assert!(copy.contains(&"-copyts".to_string()));
        assert_eq!(at(&copy, "-segment_times").as_deref(), Some("6.200,11.950"));
        assert_eq!(at(&copy, "-segment_start_number").as_deref(), Some("1"));
        assert!(copy.last().is_some_and(|a| a.ends_with("5-%d.ts")));
        let encode = args(false);
        assert_eq!(at(&encode, "-ss").as_deref(), Some("7.000000"));
        assert_eq!(
            at(&encode, "-output_ts_offset").as_deref(),
            Some("7.000000")
        );
        assert!(!encode.contains(&"-copyts".to_string()));
        assert!(encode.contains(&"expr:gte(t,n_forced*4)".to_string()));
    }
}
