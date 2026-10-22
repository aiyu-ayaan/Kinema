//! Scrub thumbnails: one small frame every few seconds tiled into JPEG sheets, plus the WebVTT index that tells a
//! player which tile of which sheet shows a given time (the `#xywh=` convention). Frames come from keyframes only
//! (`-skip_frame nokey`), so a two-hour 4K file costs a few thousand decoded frames, not a few hundred thousand.

use crate::{MediaBinaries, MediaError};
use std::fmt::Write as _;
use std::path::Path;

pub const TILE_W: u32 = 160;
pub const TILE_H: u32 = 90;
const COLS: usize = 10;
const ROWS: usize = 10;
/// Tiles in one sheet: 1600x900 px, about 100-300 KB as JPEG.
pub const SHEET_TILES: usize = COLS * ROWS;
const MIN_INTERVAL_S: u32 = 10;
/// A very long file gets a wider interval instead of hundreds of sheets.
const MAX_TILES: u32 = 2000;

/// How a file's timeline is cut into tiles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plan {
    pub interval_s: u32,
    pub tiles: usize,
    pub duration_s: f64,
}

impl Plan {
    /// None for a file without a usable duration.
    pub fn new(duration_s: f64) -> Option<Plan> {
        if !duration_s.is_finite() || duration_s < 1.0 {
            return None;
        }
        let interval_s = MIN_INTERVAL_S.max((duration_s / f64::from(MAX_TILES)).ceil() as u32);
        let tiles = (duration_s / f64::from(interval_s)).ceil() as usize;
        Some(Plan {
            interval_s,
            tiles,
            duration_s,
        })
    }

    pub fn sheets(&self) -> usize {
        self.tiles.div_ceil(SHEET_TILES)
    }

    /// The same plan cut short to what `sheets` sheets hold, for an encode that ended early.
    pub fn limited_to(self, sheets: usize) -> Plan {
        Plan {
            tiles: self.tiles.min(sheets * SHEET_TILES),
            ..self
        }
    }
}

/// `HH:MM:SS.mmm`.
fn stamp(secs: f64) -> String {
    let ms = (secs.max(0.0) * 1000.0).round() as u64;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        ms % 1000
    )
}

/// The WebVTT index: cue `i` covers `[i * interval, (i + 1) * interval)` and points at its tile. `sheet_url(n)` is
/// the address of sheet `n`, relative to the VTT.
pub fn vtt(plan: &Plan, sheet_url: impl Fn(usize) -> String) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for i in 0..plan.tiles {
        let (sheet, slot) = (i / SHEET_TILES, i % SHEET_TILES);
        let from = i as f64 * f64::from(plan.interval_s);
        let to = (from + f64::from(plan.interval_s)).min(plan.duration_s);
        if to <= from {
            break;
        }
        let _ = write!(
            out,
            "{} --> {}\n{}#xywh={},{},{},{}\n\n",
            stamp(from),
            stamp(to),
            sheet_url(sheet),
            (slot % COLS) as u32 * TILE_W,
            (slot / COLS) as u32 * TILE_H,
            TILE_W,
            TILE_H
        );
    }
    out
}

/// Writes `sheet-0.jpg`, `sheet-1.jpg`, ... into `dir` (which must exist) and returns how many it wrote.
pub async fn generate(
    bins: &MediaBinaries,
    input: &Path,
    dir: &Path,
    plan: &Plan,
) -> Result<usize, MediaError> {
    let filter = format!(
        "fps=1/{},scale={TILE_W}:{TILE_H}:force_original_aspect_ratio=decrease,\
         pad={TILE_W}:{TILE_H}:(ow-iw)/2:(oh-ih)/2,tile={COLS}x{ROWS}",
        plan.interval_s
    );
    let mut cmd = std::process::Command::new(&bins.ffmpeg);
    cmd.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-y",
        "-skip_frame",
        "nokey",
        "-i",
    ])
    .arg(input)
    .args(["-map", "0:v:0", "-an", "-sn", "-vf"])
    .arg(filter)
    .args([
        "-q:v",
        "6",
        "-start_number",
        "0",
        "-f",
        "image2",
        "-c:v",
        "mjpeg",
    ])
    .arg(dir.join("sheet-%d.jpg"));
    kinema_core::platform::configure_low_priority_process(&mut cmd);
    let out = tokio::process::Command::from(cmd)
        .kill_on_drop(true)
        .output()
        .await?;
    let mut made = 0;
    while tokio::fs::metadata(dir.join(format!("sheet-{made}.jpg")))
        .await
        .is_ok_and(|m| m.len() > 0)
    {
        made += 1;
    }
    if !out.status.success() || made == 0 {
        return Err(MediaError::ProbeFailed(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(made)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_ten_second_tiles() {
        let p = Plan::new(125.0).expect("plan");
        assert_eq!((p.interval_s, p.tiles, p.sheets()), (10, 13, 1));
        let p = Plan::new(7200.0).expect("plan");
        assert_eq!((p.interval_s, p.tiles, p.sheets()), (10, 720, 8));
        assert!(Plan::new(0.0).is_none());
        assert!(Plan::new(f64::NAN).is_none());
    }

    #[test]
    fn long_files_widen_the_interval() {
        // 12 h would be 4320 tiles at 10 s.
        let p = Plan::new(43_200.0).expect("plan");
        assert_eq!(p.interval_s, 22);
        assert!(p.tiles <= MAX_TILES as usize);
    }

    #[test]
    fn writes_the_index() {
        let p = Plan::new(125.0).expect("plan");
        let text = vtt(&p, |n| format!("sprites/{n}.jpg"));
        let cues: Vec<&str> = text.trim_end().split("\n\n").skip(1).collect();
        assert_eq!(cues.len(), 13);
        assert_eq!(
            cues[0],
            "00:00:00.000 --> 00:00:10.000\nsprites/0.jpg#xywh=0,0,160,90"
        );
        // Tile 11 is the second row's second cell; the last cue ends with the file.
        assert_eq!(
            cues[11],
            "00:01:50.000 --> 00:02:00.000\nsprites/0.jpg#xywh=160,90,160,90"
        );
        assert!(cues[12].starts_with("00:02:00.000 --> 00:02:05.000\n"));
    }

    #[test]
    fn second_sheet_starts_after_one_hundred_tiles() {
        let p = Plan::new(1500.0).expect("plan");
        let text = vtt(&p, |n| format!("{n}.jpg"));
        assert!(text.contains("00:16:40.000 --> 00:16:50.000\n1.jpg#xywh=0,0,160,90"));
        // An encode that stopped after one sheet leaves no cue for the missing one.
        let short = vtt(&p.limited_to(1), |n| format!("{n}.jpg"));
        assert!(!short.contains("1.jpg"));
        assert_eq!(short.matches("-->").count(), 100);
    }
}
