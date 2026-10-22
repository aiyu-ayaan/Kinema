//! Keyframe index: the times ffmpeg can start a copied stream at. Built once per file version in the
//! background, so a seek on the play path is a binary search instead of an ffprobe run.
//!
//! Matroska Cues and MP4 `stss` are read straight from the file (kilobytes, milliseconds even on a
//! 50 GB file); they are also exactly the points ffmpeg's demuxers seek to. Other containers fall back
//! to an ffprobe packet scan, which reads the whole file, so it is bounded by size and time.

use crate::{MediaBinaries, MediaError};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Larger files skip the packet scan: it reads every byte.
pub const SCAN_MAX_BYTES: u64 = 4 << 30;
/// A packet scan still running after this is abandoned.
pub const SCAN_TIMEOUT: Duration = Duration::from_secs(120);
/// More keyframes than this is an intra-only file: any time is a keyframe, an index adds nothing.
const MAX_KEYFRAMES: usize = 50_000;
/// Elements read whole from a Matroska or MP4 header; a real Cues or `moov` is a few MB at most.
const MAX_ELEMENT: u64 = 64 << 20;

#[derive(Debug, Clone, PartialEq)]
pub struct KeyframeIndex {
    /// cues | stss | packets
    pub source: &'static str,
    /// ffmpeg's demuxer seeks 3/23 s early when the video has B-frames (fftools/ffmpeg_demux.c).
    pub has_b_frames: bool,
    /// Ascending keyframe times of the first video stream, in seconds.
    pub times: Vec<f64>,
}

impl KeyframeIndex {
    /// Where `ffmpeg -noaccurate_seek -ss t` with a copied video starts: the keyframe at or before
    /// the demuxer's seek target.
    pub fn landing(&self, t: f64) -> f64 {
        if t <= 0.0 {
            return 0.0;
        }
        let target = if self.has_b_frames { t - 3.0 / 23.0 } else { t };
        let i = self.times.partition_point(|&k| k <= target + 1e-6);
        // Before the first keyframe ffmpeg starts at the first one.
        self.times.get(i.saturating_sub(1)).copied().unwrap_or(0.0)
    }

    /// Times as little-endian u32 milliseconds: 4 bytes a keyframe, 14 KB for two hours at 2 s GOPs.
    pub fn encode_times(&self) -> Vec<u8> {
        self.times
            .iter()
            .flat_map(|t| ((t * 1000.0).round().max(0.0) as u32).to_le_bytes())
            .collect()
    }

    pub fn decode_times(blob: &[u8]) -> Vec<f64> {
        blob.chunks_exact(4)
            .map(|c| f64::from(u32::from_le_bytes([c[0], c[1], c[2], c[3]])) / 1000.0)
            .collect()
    }
}

/// Builds the index for a file, `Ok(None)` when no cheap source exists (a big file in a container
/// without an index, or a scan that timed out). Runs at background priority.
pub async fn build(
    bins: &MediaBinaries,
    path: &Path,
    size: u64,
) -> Result<Option<KeyframeIndex>, MediaError> {
    let owned = path.to_path_buf();
    let native = tokio::task::spawn_blocking(move || read_native(&owned))
        .await
        .map_err(|e| MediaError::Io(std::io::Error::other(e)))?;
    let (source, times) = match native {
        Some(found) => found,
        None if size <= SCAN_MAX_BYTES => match scan_packets(bins, path).await? {
            Some(times) => ("packets", times),
            None => return Ok(None),
        },
        None => return Ok(None),
    };
    let mut times = times;
    times.sort_by(f64::total_cmp);
    times.dedup_by(|a, b| (*a - *b).abs() < 0.0005);
    if times.is_empty() || times.len() > MAX_KEYFRAMES {
        return Ok(None);
    }
    Ok(Some(KeyframeIndex {
        source,
        has_b_frames: has_b_frames(bins, path).await,
        times,
    }))
}

pub(crate) async fn has_b_frames(bins: &MediaBinaries, path: &Path) -> bool {
    let mut cmd = std::process::Command::new(&bins.ffprobe);
    cmd.args([
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=has_b_frames",
        "-of",
        "csv=p=0",
    ])
    .arg(path);
    kinema_core::platform::configure_low_priority_process(&mut cmd);
    Command::from(cmd)
        .kill_on_drop(true)
        .output()
        .await
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .trim_end_matches(',')
                .parse::<i64>()
                .ok()
        })
        .is_some_and(|n| n > 0)
}

fn read_native(path: &Path) -> Option<(&'static str, Vec<f64>)> {
    let mut f = File::open(path).ok()?;
    let mut magic = [0u8; 12];
    f.read_exact(&mut magic).ok()?;
    f.seek(SeekFrom::Start(0)).ok()?;
    if magic[..4] == [0x1A, 0x45, 0xDF, 0xA3] {
        return matroska_cues(&mut f).map(|t| ("cues", t));
    }
    if &magic[4..8] == b"ftyp" || &magic[4..8] == b"moov" {
        return mp4_sync_samples(&mut f).map(|t| ("stss", t));
    }
    None
}

/// ffprobe over every packet of the first video stream, keeping keyframes. Demux only, no decode.
async fn scan_packets(bins: &MediaBinaries, path: &Path) -> Result<Option<Vec<f64>>, MediaError> {
    let mut cmd = std::process::Command::new(&bins.ffprobe);
    cmd.args([
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "packet=pts_time,flags",
        "-of",
        "csv=p=0",
    ])
    .arg(path)
    .stdout(std::process::Stdio::piped())
    .stdin(std::process::Stdio::null())
    .stderr(std::process::Stdio::null());
    kinema_core::platform::configure_low_priority_process(&mut cmd);
    let mut child = Command::from(cmd).kill_on_drop(true).spawn()?;
    let Some(stdout) = child.stdout.take() else {
        return Ok(None);
    };
    let read = async {
        let mut times = Vec::new();
        let mut lines = BufReader::new(stdout).lines();
        while let Some(line) = lines.next_line().await? {
            if let Some(t) = keyframe_from_csv(&line) {
                times.push(t);
                if times.len() > MAX_KEYFRAMES {
                    break;
                }
            }
        }
        Ok::<_, std::io::Error>(times)
    };
    match tokio::time::timeout(SCAN_TIMEOUT, read).await {
        Ok(times) => Ok(Some(times?)),
        Err(_) => Ok(None),
    }
}

/// `pts_time,flags` with a `K` flag.
fn keyframe_from_csv(line: &str) -> Option<f64> {
    let mut f = line.trim().split(',');
    let pts: f64 = f.next()?.parse().ok()?;
    f.next()?.starts_with('K').then_some(pts)
}

// ---------- Matroska ----------

const SEGMENT: u32 = 0x1853_8067;
const SEEK_HEAD: u32 = 0x114D_9B74;
const SEEK: u32 = 0x4DBB;
const SEEK_ID: u32 = 0x53AB;
const SEEK_POSITION: u32 = 0x53AC;
const INFO: u32 = 0x1549_A966;
const TIMECODE_SCALE: u32 = 0x2A_D7B1;
const TRACKS: u32 = 0x1654_AE6B;
const TRACK_ENTRY: u32 = 0xAE;
const TRACK_NUMBER: u32 = 0xD7;
const TRACK_TYPE: u32 = 0x83;
const CUES: u32 = 0x1C53_BB6B;
const CUE_POINT: u32 = 0xBB;
const CUE_TIME: u32 = 0xB3;
const CUE_TRACK_POSITIONS: u32 = 0xB7;
const CUE_TRACK: u32 = 0xF7;
const CLUSTER: u32 = 0x1F43_B675;

/// An EBML variable-length number at `buf[*pos..]`. IDs keep their length marker; sizes drop it.
/// `None` size means "unknown" (all value bits set).
fn vint(buf: &[u8], pos: &mut usize, keep_marker: bool) -> Option<Option<u64>> {
    let first = *buf.get(*pos)?;
    let len = first.leading_zeros() as usize + 1;
    if len > 8 || *pos + len > buf.len() {
        return None;
    }
    let mut v = if keep_marker {
        u64::from(first)
    } else {
        u64::from(first) & (0xFF >> len)
    };
    for &b in &buf[*pos + 1..*pos + len] {
        v = (v << 8) | u64::from(b);
    }
    *pos += len;
    let all_ones = (1u64 << (7 * len)) - 1;
    Some((keep_marker || v != all_ones).then_some(v))
}

/// Children of an element body: (id, body).
fn children(buf: &[u8]) -> Vec<(u32, &[u8])> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < buf.len() {
        let Some(Some(id)) = vint(buf, &mut pos, true) else {
            break;
        };
        let Some(Some(size)) = vint(buf, &mut pos, false) else {
            break;
        };
        let end = pos.saturating_add(size as usize).min(buf.len());
        out.push((id as u32, &buf[pos..end]));
        pos = end;
    }
    out
}

fn uint(body: &[u8]) -> u64 {
    body.iter().take(8).fold(0, |v, &b| (v << 8) | u64::from(b))
}

/// Reads an element header at the file's current position, leaving it at the body: (id, size).
fn element_header(f: &mut File) -> Option<(u32, Option<u64>)> {
    let mut head = [0u8; 12];
    let n = f.read(&mut head).ok()?;
    let mut pos = 0;
    let id = vint(&head[..n], &mut pos, true)??;
    let size = vint(&head[..n], &mut pos, false)?;
    f.seek(SeekFrom::Current(pos as i64 - n as i64)).ok()?;
    Some((id as u32, size))
}

fn read_body(f: &mut File, size: u64) -> Option<Vec<u8>> {
    if size > MAX_ELEMENT {
        return None;
    }
    let mut body = vec![0u8; size as usize];
    f.read_exact(&mut body).ok()?;
    Some(body)
}

fn matroska_cues(f: &mut File) -> Option<Vec<f64>> {
    // EBML header, then the Segment.
    let (_, Some(header_size)) = element_header(f)? else {
        return None;
    };
    f.seek(SeekFrom::Current(header_size as i64)).ok()?;
    let (id, _) = element_header(f)?;
    if id != SEGMENT {
        return None;
    }
    let segment_start = f.stream_position().ok()?;
    let mut scale = 1_000_000u64;
    let mut video_track = None;
    let mut cues_at = None;
    let mut tracks_at = None;
    // Top-level elements before the first Cluster: SeekHead says where Cues (often at the end) live.
    while let Some((id, Some(size))) = element_header(f) {
        match id {
            CLUSTER => break,
            SEEK_HEAD | INFO | TRACKS | CUES => {
                let body = read_body(f, size)?;
                match id {
                    SEEK_HEAD => {
                        for (_, seek) in children(&body).into_iter().filter(|(i, _)| *i == SEEK) {
                            let kids = children(seek);
                            let target = kids
                                .iter()
                                .find(|(i, _)| *i == SEEK_ID)
                                .map(|(_, b)| uint(b));
                            let pos = kids
                                .iter()
                                .find(|(i, _)| *i == SEEK_POSITION)
                                .map(|(_, b)| uint(b));
                            match (target, pos) {
                                (Some(t), Some(p)) if t == u64::from(CUES) => cues_at = Some(p),
                                (Some(t), Some(p)) if t == u64::from(TRACKS) => tracks_at = Some(p),
                                _ => {}
                            }
                        }
                    }
                    INFO => {
                        if let Some((_, b)) = children(&body)
                            .into_iter()
                            .find(|(i, _)| *i == TIMECODE_SCALE)
                        {
                            scale = uint(b).max(1);
                        }
                    }
                    TRACKS => video_track = first_video_track(&body),
                    _ => return Some(cue_times(&body, video_track?, scale)),
                }
            }
            _ => {
                f.seek(SeekFrom::Current(i64::try_from(size).ok()?)).ok()?;
            }
        }
    }
    if video_track.is_none() {
        f.seek(SeekFrom::Start(segment_start + tracks_at?)).ok()?;
        let (id, Some(size)) = element_header(f)? else {
            return None;
        };
        if id != TRACKS {
            return None;
        }
        video_track = first_video_track(&read_body(f, size)?);
    }
    f.seek(SeekFrom::Start(segment_start + cues_at?)).ok()?;
    let (id, Some(size)) = element_header(f)? else {
        return None;
    };
    if id != CUES {
        return None;
    }
    Some(cue_times(&read_body(f, size)?, video_track?, scale))
}

fn first_video_track(tracks: &[u8]) -> Option<u64> {
    children(tracks)
        .into_iter()
        .filter(|(i, _)| *i == TRACK_ENTRY)
        .find_map(|(_, entry)| {
            let kids = children(entry);
            let kind = kids
                .iter()
                .find(|(i, _)| *i == TRACK_TYPE)
                .map(|(_, b)| uint(b));
            let number = kids
                .iter()
                .find(|(i, _)| *i == TRACK_NUMBER)
                .map(|(_, b)| uint(b));
            (kind == Some(1)).then_some(number).flatten()
        })
}

fn cue_times(cues: &[u8], track: u64, scale: u64) -> Vec<f64> {
    children(cues)
        .into_iter()
        .filter(|(i, _)| *i == CUE_POINT)
        .filter_map(|(_, point)| {
            let kids = children(point);
            let time = kids
                .iter()
                .find(|(i, _)| *i == CUE_TIME)
                .map(|(_, b)| uint(b))?;
            let ours = kids
                .iter()
                .filter(|(i, _)| *i == CUE_TRACK_POSITIONS)
                .any(|(_, p)| {
                    children(p)
                        .iter()
                        .any(|(i, b)| *i == CUE_TRACK && uint(b) == track)
                });
            ours.then(|| time as f64 * scale as f64 / 1e9)
        })
        .collect()
}

// ---------- MP4 ----------

/// Boxes of a body: (type, body). 64-bit sizes are honoured; size 0 runs to the end.
fn boxes(buf: &[u8]) -> Vec<([u8; 4], &[u8])> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos + 8 <= buf.len() {
        let size32 = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
        let kind = [buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]];
        let (head, size) = match size32 {
            0 => (8, buf.len() - pos),
            1 if pos + 16 <= buf.len() => (16, be_u64(&buf[pos + 8..]) as usize),
            n => (8, n as usize),
        };
        if size < head || pos + size > buf.len() {
            break;
        }
        out.push((kind, &buf[pos + head..pos + size]));
        pos += size;
    }
    out
}

fn child<'a>(buf: &'a [u8], kind: &[u8; 4]) -> Option<&'a [u8]> {
    boxes(buf)
        .into_iter()
        .find(|(k, _)| k == kind)
        .map(|(_, b)| b)
}

fn be_u32(b: &[u8]) -> u32 {
    b.get(..4)
        .map_or(0, |s| u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}

fn be_u64(b: &[u8]) -> u64 {
    b.get(..8).map_or(0, |s| {
        u64::from_be_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]])
    })
}

/// The `moov` box, wherever it sits; `mdat` before it is skipped, not read.
fn find_moov(f: &mut File) -> Option<Vec<u8>> {
    let len = f.metadata().ok()?.len();
    let mut pos = 0u64;
    while pos + 8 <= len {
        f.seek(SeekFrom::Start(pos)).ok()?;
        let mut head = [0u8; 16];
        f.read_exact(&mut head[..8]).ok()?;
        let mut size = u64::from(be_u32(&head));
        let mut header = 8;
        if size == 1 {
            f.read_exact(&mut head[8..16]).ok()?;
            size = be_u64(&head[8..]);
            header = 16;
        } else if size == 0 {
            size = len - pos;
        }
        if size < header {
            return None;
        }
        if &head[4..8] == b"moov" {
            return read_body(f, size - header);
        }
        pos += size;
    }
    None
}

fn mp4_sync_samples(f: &mut File) -> Option<Vec<f64>> {
    let moov = find_moov(f)?;
    boxes(&moov)
        .into_iter()
        .filter(|(k, _)| k == b"trak")
        .find_map(|(_, trak)| video_trak_keyframes(trak))
}

fn video_trak_keyframes(trak: &[u8]) -> Option<Vec<f64>> {
    let mdia = child(trak, b"mdia")?;
    let hdlr = child(mdia, b"hdlr")?;
    if hdlr.get(8..12)? != b"vide" {
        return None;
    }
    let mdhd = child(mdia, b"mdhd")?;
    let timescale = if mdhd.first() == Some(&1) {
        be_u32(mdhd.get(20..)?)
    } else {
        be_u32(mdhd.get(12..)?)
    };
    if timescale == 0 {
        return None;
    }
    let stbl = child(child(mdia, b"minf")?, b"stbl")?;
    // No stss: every sample is a keyframe (intra-only); nothing worth indexing.
    let stss = child(stbl, b"stss")?;
    let stts = child(stbl, b"stts")?;
    let ctts = child(stbl, b"ctts");
    // The first edit's media time shifts presentation (B-frame delay); ffmpeg applies it too.
    let shift = child(trak, b"edts")
        .and_then(|e| child(e, b"elst"))
        .and_then(edit_media_time)
        .unwrap_or(0);

    let sync: Vec<u32> = (0..be_u32(stss.get(4..)?) as usize)
        .filter_map(|i| stss.get(8 + i * 4..).map(be_u32))
        .collect();
    let dts_of = run_lookup(stts, false);
    let ctts_of = ctts.map(|c| run_lookup(c, true));
    let times = sync
        .iter()
        .filter(|&&s| s > 0)
        .filter_map(|&s| {
            let n = u64::from(s - 1);
            let dts = dts_of(n)?;
            let offset = ctts_of.as_ref().and_then(|c| c(n)).unwrap_or(0);
            let pts = dts + offset - shift;
            Some(pts.max(0) as f64 / f64::from(timescale))
        })
        .collect();
    Some(times)
}

fn edit_media_time(elst: &[u8]) -> Option<i64> {
    let v1 = elst.first() == Some(&1);
    let count = be_u32(elst.get(4..)?) as usize;
    let entry = if v1 { 20 } else { 12 };
    (0..count).find_map(|i| {
        let e = elst.get(8 + i * entry..)?;
        let media_time = if v1 {
            be_u64(e.get(8..)?) as i64
        } else {
            i64::from(be_u32(e.get(4..)?) as i32)
        };
        // -1 is an empty edit (a delay), not a shift.
        (media_time >= 0).then_some(media_time)
    })
}

/// `stts` (cumulative: decode time of sample n) or `ctts` (`offsets`: the composition offset of
/// sample n) as a lookup over their (count, value) runs.
fn run_lookup(table: &[u8], offsets: bool) -> impl Fn(u64) -> Option<i64> + '_ {
    let count = be_u32(table.get(4..).unwrap_or_default()) as usize;
    let runs: Vec<(u64, i64)> = (0..count)
        .filter_map(|i| {
            let e = table.get(8 + i * 8..)?;
            Some((u64::from(be_u32(e)), i64::from(be_u32(e.get(4..)?) as i32)))
        })
        .collect();
    move |n| {
        let mut first = 0u64;
        let mut t = 0i64;
        for &(count, value) in &runs {
            if n < first + count {
                return Some(if offsets {
                    value
                } else {
                    t + (n - first) as i64 * value
                });
            }
            first += count;
            t += count as i64 * value;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index(times: &[f64], has_b_frames: bool) -> KeyframeIndex {
        KeyframeIndex {
            source: "cues",
            has_b_frames,
            times: times.to_vec(),
        }
    }

    #[test]
    fn landing_snaps_to_the_keyframe_ffmpeg_seeks_to() {
        let idx = index(&[0.0, 2.5, 7.0, 8.0, 13.25], false);
        assert_eq!(idx.landing(0.0), 0.0);
        assert_eq!(idx.landing(7.0), 7.0);
        assert_eq!(idx.landing(7.9), 7.0);
        assert_eq!(idx.landing(100.0), 13.25);
        // With B-frames the demuxer aims 3/23 s early: exactly on a keyframe lands one before it.
        let b = index(&[0.0, 2.5, 7.0], true);
        assert_eq!(b.landing(7.0), 2.5);
        assert_eq!(b.landing(7.2), 7.0);
        // Before the first keyframe, the first one.
        assert_eq!(index(&[1.0, 3.0], false).landing(0.5), 1.0);
    }

    #[test]
    fn times_round_trip_as_milliseconds() {
        let idx = index(&[0.0, 2.521, 3600.042], false);
        assert_eq!(KeyframeIndex::decode_times(&idx.encode_times()), idx.times);
    }

    #[test]
    fn packet_csv_keeps_keyframes_only() {
        assert_eq!(keyframe_from_csv("2.500000,K__"), Some(2.5));
        assert_eq!(keyframe_from_csv("2.541000,___"), None);
        assert_eq!(keyframe_from_csv("N/A,K__"), None);
    }

    fn ebml(id: u32, body: &[u8]) -> Vec<u8> {
        let id_bytes = id.to_be_bytes();
        let skip = id_bytes.iter().take_while(|b| **b == 0).count();
        let mut out = id_bytes[skip..].to_vec();
        // 8-byte size: 0x01 marker then 7 bytes.
        out.push(0x01);
        out.extend_from_slice(&(body.len() as u64).to_be_bytes()[1..]);
        out.extend_from_slice(body);
        out
    }

    fn cue_point(time_ms: u8, track: u8) -> Vec<u8> {
        let pos = [ebml(CUE_TRACK, &[track]), ebml(0xF1, &[0x10])].concat();
        ebml(
            CUE_POINT,
            &[
                ebml(CUE_TIME, &[0, time_ms]),
                ebml(CUE_TRACK_POSITIONS, &pos),
            ]
            .concat(),
        )
    }

    /// Cues after the clusters, found through the SeekHead, as mkvmerge and ffmpeg write them.
    #[test]
    fn matroska_cues_at_the_end_via_seek_head() {
        let tracks = ebml(
            TRACKS,
            &[
                ebml(
                    TRACK_ENTRY,
                    &[ebml(TRACK_NUMBER, &[1]), ebml(TRACK_TYPE, &[2])].concat(),
                ),
                ebml(
                    TRACK_ENTRY,
                    &[ebml(TRACK_NUMBER, &[2]), ebml(TRACK_TYPE, &[1])].concat(),
                ),
            ]
            .concat(),
        );
        let info = ebml(INFO, &ebml(TIMECODE_SCALE, &[0x0F, 0x42, 0x40]));
        let cluster = ebml(CLUSTER, &[0u8; 300]);
        let cues = ebml(
            CUES,
            &[cue_point(0, 2), cue_point(1, 1), cue_point(250, 2)].concat(),
        );
        // SeekHead with a fixed size: its own length does not depend on the offset it holds.
        let seek_head = |cues_at: u64| {
            let seek = ebml(
                SEEK,
                &[
                    ebml(SEEK_ID, &CUES.to_be_bytes()),
                    ebml(SEEK_POSITION, &cues_at.to_be_bytes()),
                ]
                .concat(),
            );
            ebml(SEEK_HEAD, &seek)
        };
        let cues_at = (seek_head(0).len() + info.len() + tracks.len() + cluster.len()) as u64;
        let segment = [seek_head(cues_at), info, tracks, cluster, cues].concat();
        let file = [
            ebml(0x1A45_DFA3, &ebml(0x4282, b"matroska")),
            ebml(SEGMENT, &segment),
        ]
        .concat();
        let dir = std::env::temp_dir().join(format!("kinema-kf-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.mkv");
        std::fs::write(&path, file).unwrap();
        let found = read_native(&path);
        std::fs::remove_dir_all(&dir).unwrap();
        // Track 2 is the video: the audio cue at 1 ms is not a keyframe of it.
        assert_eq!(found, Some(("cues", vec![0.0, 0.25])));
    }

    fn mp4box(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        [&((body.len() + 8) as u32).to_be_bytes()[..], kind, body].concat()
    }

    fn full(entries: &[u32]) -> Vec<u8> {
        let mut b = vec![0u8; 4];
        b.extend(entries.iter().flat_map(|e| e.to_be_bytes()));
        b
    }

    /// moov after mdat; 24 fps at timescale 12288, 2-frame B delay undone by the edit list.
    #[test]
    fn mp4_sync_samples_with_composition_offsets() {
        let mut hdlr = vec![0u8; 8];
        hdlr.extend_from_slice(b"vide");
        hdlr.extend_from_slice(&[0u8; 12]);
        let mut mdhd = vec![0u8; 12];
        mdhd.extend_from_slice(&12288u32.to_be_bytes());
        mdhd.extend_from_slice(&[0u8; 8]);
        let stbl = [
            mp4box(b"stss", &full(&[3, 1, 61, 169])),
            mp4box(b"stts", &full(&[1, 240, 512])),
            mp4box(b"ctts", &full(&[1, 240, 1024])),
        ]
        .concat();
        let mdia = [
            mp4box(b"mdhd", &mdhd),
            mp4box(b"hdlr", &hdlr),
            mp4box(b"minf", &mp4box(b"stbl", &stbl)),
        ]
        .concat();
        let elst = full(&[1, 10, 1024, 0x0001_0000]);
        let trak = [
            mp4box(b"edts", &mp4box(b"elst", &elst)),
            mp4box(b"mdia", &mdia),
        ]
        .concat();
        let file = [
            mp4box(b"ftyp", b"isom\0\0\0\0"),
            mp4box(b"mdat", &[0u8; 64]),
            mp4box(b"moov", &mp4box(b"trak", &trak)),
        ]
        .concat();
        let dir = std::env::temp_dir().join(format!("kinema-kf-mp4-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("a.mp4");
        std::fs::write(&path, file).unwrap();
        let found = read_native(&path);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(found, Some(("stss", vec![0.0, 2.5, 7.0])));
    }
}
