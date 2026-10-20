//! Where a file's intro and end credits are, for Skip intro and the up-next card. Three sources, cheapest first:
//! chapters named like an intro or credits, audio a season's episodes share (the theme song, the end song), and for
//! credits a long near-black stretch running to the end (white text on black reads as black). The audio
//! fingerprint is our own small one (band-energy deltas, after Haitsma and Kalker), so no chromaprint build is needed.

use crate::{MediaBinaries, MediaError};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// A stretch of a file, in seconds from its start.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone)]
pub struct Chapter {
    pub start: f64,
    pub end: f64,
    pub title: String,
}

#[derive(Deserialize)]
struct RawChapters {
    #[serde(default)]
    chapters: Vec<RawChapter>,
}

#[derive(Deserialize)]
struct RawChapter {
    start_time: Option<String>,
    end_time: Option<String>,
    #[serde(default)]
    tags: HashMap<String, String>,
}

/// The file's chapters, from ffprobe.
pub async fn chapters(bins: &MediaBinaries, path: &Path) -> Result<Vec<Chapter>, MediaError> {
    let mut cmd = std::process::Command::new(&bins.ffprobe);
    cmd.args(["-v", "error", "-print_format", "json", "-show_chapters"])
        .arg(path);
    let out = run(cmd).await?;
    parse_chapters(&out.stdout)
}

fn parse_chapters(json: &[u8]) -> Result<Vec<Chapter>, MediaError> {
    let raw: RawChapters = serde_json::from_slice(json)?;
    Ok(raw
        .chapters
        .into_iter()
        .filter_map(|c| {
            let title = c
                .tags
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("title"))
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            Some(Chapter {
                start: c.start_time?.parse().ok()?,
                end: c.end_time?.parse().ok()?,
                title,
            })
        })
        .collect())
}

/// The first chapter named like an intro and the last named like credits ("Opening Credits" is an intro).
pub fn chapter_spans(chapters: &[Chapter]) -> (Option<Span>, Option<Span>) {
    let mut intro = None;
    let mut credits = None;
    for c in chapters {
        let title = c.title.to_lowercase();
        let words: Vec<&str> = title
            .split(|ch: char| !ch.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .collect();
        let has = |w: &str| words.contains(&w);
        let span = Span {
            start: c.start,
            end: c.end,
        };
        if c.end <= c.start {
            continue;
        }
        if title.contains("intro")
            || title.contains("opening")
            || has("op")
            || title.contains("title sequence")
        {
            intro.get_or_insert(span);
        } else if title.contains("credit")
            || title.contains("ending")
            || title.contains("outro")
            || has("ed")
        {
            credits = Some(span);
        }
    }
    (intro, credits)
}

/// Fingerprint sample rate: speech and music bands below 2 kHz are all it looks at.
const RATE: usize = 8000;
const FRAME: usize = 2048;
/// An eighth of a frame, so two copies of the same audio cut a few milliseconds apart still hash alike.
const HOP: usize = 256;
/// Seconds between two fingerprint values.
pub const HASH_SECONDS: f64 = HOP as f64 / RATE as f64;
const BANDS: usize = 33;

/// The audio fingerprint of `secs` seconds from `from`: one u32 per [HASH_SECONDS], 0 for silence.
pub async fn audio_print(
    bins: &MediaBinaries,
    path: &Path,
    from: f64,
    secs: f64,
) -> Result<Vec<u32>, MediaError> {
    let mut cmd = std::process::Command::new(&bins.ffmpeg);
    cmd.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-ss"])
        .arg(format!("{:.3}", from.max(0.0)))
        .arg("-i")
        .arg(path)
        .arg("-t")
        .arg(format!("{secs:.3}"))
        .args(["-map", "0:a:0", "-vn", "-ac", "1", "-ar"])
        .arg(RATE.to_string())
        .args(["-f", "s16le", "-"]);
    let out = run(cmd).await?;
    let samples: Vec<i16> = out
        .stdout
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| i16::from_le_bytes(*b))
        .collect();
    // Hashing a few minutes of audio is real CPU work: keep it off the async workers.
    tokio::task::spawn_blocking(move || fingerprint(&samples))
        .await
        .map_err(|e| MediaError::ProbeFailed(e.to_string()))
}

/// Band-energy fingerprint: bit b is set when the energy step between bands b and b+1 grew since the last frame.
pub fn fingerprint(samples: &[i16]) -> Vec<u32> {
    // 33 bands from 300 Hz to 2 kHz, evenly spaced on a log scale, as FFT bin edges.
    let bin_hz = RATE as f64 / FRAME as f64;
    let edges: Vec<usize> = (0..=BANDS)
        .map(|i| {
            (300.0 * (2000.0f64 / 300.0).powf(i as f64 / BANDS as f64) / bin_hz).round() as usize
        })
        .collect();
    let window: Vec<f32> = (0..FRAME)
        .map(|i| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / FRAME as f32).cos())
        .collect();
    let mut re = vec![0f32; FRAME];
    let mut im = vec![0f32; FRAME];
    let mut prev = [0f32; BANDS];
    let mut out = Vec::new();
    let mut start = 0;
    while start + FRAME <= samples.len() {
        let frame = &samples[start..start + FRAME];
        let power = frame
            .iter()
            .map(|&s| (s as f32 / 32768.0).powi(2))
            .sum::<f32>()
            / FRAME as f32;
        for i in 0..FRAME {
            re[i] = frame[i] as f32 / 32768.0 * window[i];
            im[i] = 0.0;
        }
        fft(&mut re, &mut im);
        let mut energy = [0f32; BANDS];
        for (b, e) in energy.iter_mut().enumerate() {
            *e = (edges[b]..edges[b + 1])
                .map(|k| re[k] * re[k] + im[k] * im[k])
                .sum();
        }
        let mut hash = 0u32;
        for b in 0..BANDS - 1 {
            if (energy[b] - energy[b + 1]) - (prev[b] - prev[b + 1]) > 0.0 {
                hash |= 1 << b;
            }
        }
        // Below about -50 dBFS is silence, which every file shares: it must never count as a match.
        let silent = start == 0 || power < 1e-5;
        out.push(if silent { 0 } else { hash.max(1) });
        prev = energy;
        start += HOP;
    }
    out
}

/// In-place radix-2 FFT; `re.len()` is a power of two.
fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let angle = -2.0 * std::f32::consts::PI / len as f32;
        for start in (0..n).step_by(len) {
            for k in 0..len / 2 {
                let (s, c) = (angle * k as f32).sin_cos();
                let (a, b) = (start + k, start + k + len / 2);
                let tr = re[b] * c - im[b] * s;
                let ti = re[b] * s + im[b] * c;
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
            }
        }
        len <<= 1;
    }
}

/// The longest stretch of audio two fingerprints share, at least `min_len` values long, as index ranges
/// (start, end) in `a` and in `b`. Exact hash hits vote for an alignment; the best few are walked allowing small
/// differences and short gaps (a sound effect over the theme song).
pub fn shared(a: &[u32], b: &[u32], min_len: usize) -> Option<((usize, usize), (usize, usize))> {
    const MAX_BITS: u32 = 8;
    const STRONG_BITS: u32 = 4;
    const GAP: usize = 32;
    let mut index: HashMap<u32, Vec<usize>> = HashMap::new();
    for (j, &h) in b.iter().enumerate() {
        if h != 0 {
            index.entry(h).or_default().push(j);
        }
    }
    let mut votes: HashMap<isize, u32> = HashMap::new();
    for (i, &h) in a.iter().enumerate() {
        // A hash found all over is a drone or a tone: it says nothing about alignment.
        if let Some(js) = index.get(&h).filter(|js| js.len() <= 32) {
            for &j in js {
                *votes.entry(i as isize - j as isize).or_default() += 1;
            }
        }
    }
    let mut shifts: Vec<(isize, u32)> = votes.into_iter().collect();
    shifts.sort_by_key(|x| std::cmp::Reverse(x.1));

    let mut best: Option<((usize, usize), (usize, usize))> = None;
    let mut best_len = 0;
    for &(shift, _) in shifts.iter().take(8) {
        let first = shift.max(0) as usize;
        let last = (b.len() as isize + shift).min(a.len() as isize).max(0) as usize;
        // A run starts and ends on a near-exact hit; weaker ones only bridge it, so stray look-alikes before or
        // after the shared audio do not stretch it.
        let mut run: Option<usize> = None;
        let mut seen = 0;
        for i in first..last {
            let (x, y) = (a[i], b[(i as isize - shift) as usize]);
            if x == 0 || y == 0 {
                continue;
            }
            let bits = (x ^ y).count_ones();
            if bits > MAX_BITS {
                continue;
            }
            let start = match run {
                Some(s) if i - seen <= GAP => s,
                _ if bits <= STRONG_BITS => i,
                _ => {
                    run = None;
                    continue;
                }
            };
            run = Some(start);
            seen = i;
            let len = i + 1 - start;
            if bits <= STRONG_BITS && len > best_len {
                best_len = len;
                let (bs, be) = (
                    (start as isize - shift) as usize,
                    (i as isize + 1 - shift) as usize,
                );
                best = Some(((start, i + 1), (bs, be)));
            }
        }
    }
    best.filter(|_| best_len >= min_len)
}

/// Near-black stretches from `from` to the end, from keyframes only so even a 4K file costs a few hundred frames.
pub async fn black_runs(
    bins: &MediaBinaries,
    path: &Path,
    from: f64,
) -> Result<Vec<Span>, MediaError> {
    let mut cmd = std::process::Command::new(&bins.ffmpeg);
    cmd.args([
        "-hide_banner",
        "-nostats",
        "-loglevel",
        "info",
        "-nostdin",
        "-skip_frame",
        "nokey",
        "-ss",
    ])
    .arg(format!("{:.3}", from.max(0.0)))
    // Keeps the file's own timestamps, so the runs come out in seconds from its start.
    .arg("-copyts")
    .arg("-i")
    .arg(path)
    .args([
        "-map",
        "0:v:0",
        "-an",
        "-sn",
        "-vf",
        // Credit text covers a few percent of the picture: up to 8 % lit still counts as black.
        "scale=320:-2,blackdetect=d=2:pic_th=0.92:pix_th=0.12",
        "-f",
        "null",
        "-",
    ]);
    let out = run(cmd).await?;
    Ok(parse_black(&String::from_utf8_lossy(&out.stderr)))
}

fn parse_black(log: &str) -> Vec<Span> {
    let value = |line: &str, key: &str| -> Option<f64> {
        let rest = &line[line.find(key)? + key.len()..];
        rest.split_whitespace().next()?.parse().ok()
    };
    log.lines()
        .filter(|l| l.contains("blackdetect"))
        .filter_map(|l| {
            Some(Span {
                start: value(l, "black_start:")?,
                end: value(l, "black_end:")?,
            })
        })
        .collect()
}

/// Where end credits start: black runs (joined across short lit gaps) that last at least 30 s and reach the end.
pub fn credits_from_black(runs: &[Span], duration: f64) -> Option<f64> {
    let mut merged: Vec<Span> = Vec::new();
    for r in runs {
        match merged.last_mut() {
            Some(m) if r.start - m.end <= 8.0 => m.end = m.end.max(r.end),
            _ => merged.push(*r),
        }
    }
    merged
        .last()
        .filter(|m| m.end >= duration - 20.0 && m.end - m.start >= 30.0)
        .map(|m| m.start)
}

async fn run(mut cmd: std::process::Command) -> Result<std::process::Output, MediaError> {
    kinema_core::platform::configure_low_priority_process(&mut cmd);
    let out = tokio::process::Command::from(cmd)
        .kill_on_drop(true)
        .output()
        .await?;
    if !out.status.success() {
        return Err(MediaError::ProbeFailed(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chapter(start: f64, end: f64, title: &str) -> Chapter {
        Chapter {
            start,
            end,
            title: title.into(),
        }
    }

    #[test]
    fn names_chapters() {
        let (intro, credits) = chapter_spans(&[
            chapter(0.0, 60.0, "Prologue"),
            chapter(60.0, 150.0, "Opening Credits"),
            chapter(150.0, 1300.0, "Part A"),
            chapter(1300.0, 1390.0, "ED"),
            chapter(1390.0, 1420.0, "Preview"),
        ]);
        assert_eq!(
            intro,
            Some(Span {
                start: 60.0,
                end: 150.0
            })
        );
        assert_eq!(
            credits,
            Some(Span {
                start: 1300.0,
                end: 1390.0
            })
        );
        assert_eq!(
            chapter_spans(&[chapter(0.0, 10.0, "Education")]),
            (None, None)
        );
    }

    #[test]
    fn parses_ffprobe_chapters() {
        let json = br#"{"chapters":[{"start_time":"0.000000","end_time":"91.5","tags":{"title":"Intro"}}]}"#;
        let c = parse_chapters(json).unwrap();
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].end, 91.5);
        assert_eq!(c[0].title, "Intro");
    }

    /// Deterministic noise with a slowly changing tone, so every second sounds different.
    fn signal(seed: u32, secs: usize) -> Vec<i16> {
        let mut x = seed;
        (0..secs * RATE)
            .map(|i| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                let tone = (i as f32 * (0.05 + (i / RATE) as f32 * 0.013)).sin() * 6000.0;
                (tone + (x % 4000) as f32 - 2000.0) as i16
            })
            .collect()
    }

    #[test]
    fn finds_shared_audio_at_different_offsets() {
        // The same 40 s "theme" after 20 s of one cold open and 55 s of another.
        let theme = signal(7, 40);
        let a: Vec<i16> = signal(1, 20)
            .into_iter()
            .chain(theme.iter().copied())
            .chain(signal(2, 30))
            .collect();
        let b: Vec<i16> = signal(3, 55)
            .into_iter()
            .chain(theme.iter().copied())
            .chain(signal(4, 30))
            .collect();
        let (pa, pb) = (fingerprint(&a), fingerprint(&b));
        let min = (15.0 / HASH_SECONDS) as usize;
        let ((a0, a1), (b0, _)) = shared(&pa, &pb, min).expect("theme found");
        let secs = |i: usize| i as f64 * HASH_SECONDS;
        assert!((secs(a0) - 20.0).abs() < 1.0, "a starts at {}", secs(a0));
        assert!((secs(b0) - 55.0).abs() < 1.0, "b starts at {}", secs(b0));
        assert!(
            (secs(a1 - a0) - 40.0).abs() < 1.5,
            "length {}",
            secs(a1 - a0)
        );
        // Two unrelated tracks share nothing.
        assert!(
            shared(
                &fingerprint(&signal(5, 60)),
                &fingerprint(&signal(6, 60)),
                min
            )
            .is_none()
        );
    }

    #[test]
    fn silence_never_matches() {
        let quiet = vec![0i16; 60 * RATE];
        let p = fingerprint(&quiet);
        assert!(p.iter().all(|&h| h == 0));
        assert!(shared(&p, &p, 10).is_none());
    }

    #[test]
    fn credits_from_black_runs() {
        let log = "[blackdetect @ 0x1] black_start:5290.1 black_end:5300.4 black_duration:10.3\n\
                   [blackdetect @ 0x1] black_start:5304 black_end:5395.9 black_duration:91.9\n";
        let runs = parse_black(log);
        assert_eq!(runs.len(), 2);
        assert_eq!(credits_from_black(&runs, 5400.0), Some(5290.1));
        // A dark scene in the middle is not credits.
        assert_eq!(
            credits_from_black(
                &[Span {
                    start: 3000.0,
                    end: 3060.0
                }],
                5400.0
            ),
            None
        );
    }
}
