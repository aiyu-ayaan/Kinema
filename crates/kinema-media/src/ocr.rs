//! Picture subtitles (PGS, VobSub) to WebVTT through OCR, so a viewer can switch to them
//! like a text track instead of burning them into a transcode.
//!
//! Flow: ffmpeg renders the subtitle track alone (its own "sub2video" frames, no video
//! decode) into one grayscale PNG per caption change, named by its time in milliseconds.
//! Identical consecutive frames are dropped, then a single `tesseract` run reads all the
//! remaining images from a list file. Each image's text lasts until the next image.
//! `tesseract` is an optional external tool found at startup; without it nothing here runs.

use crate::subtitles::{Cue, to_vtt};
use crate::{MediaBinaries, MediaError};
use std::path::{Path, PathBuf};
use std::process::Stdio;

/// The OCR engine found at startup.
#[derive(Debug, Clone)]
pub struct OcrTool {
    pub tesseract: PathBuf,
    /// Installed language models, e.g. "eng", "fra", "chi_sim".
    pub languages: Vec<String>,
}

impl OcrTool {
    /// `KINEMA_TESSERACT`, then PATH. None when the tool is missing, does not run, or has
    /// no language model: OCR then stays off and picture subtitles stay picture-only.
    pub fn detect() -> Option<Self> {
        let tesseract = std::env::var_os("KINEMA_TESSERACT")
            .map(PathBuf::from)
            .filter(|p| p.is_file())
            .or_else(|| crate::resolver::which("tesseract").ok())?;
        let out = std::process::Command::new(&tesseract)
            .arg("--list-langs")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()
            .filter(|o| o.status.success())?;
        let languages = parse_languages(&String::from_utf8_lossy(&out.stdout));
        (!languages.is_empty()).then_some(Self {
            tesseract,
            languages,
        })
    }

    /// The model for a track tagged `lang` ("en", "eng", "fre"), else English, else none.
    pub fn language_for(&self, lang: Option<&str>) -> Option<&str> {
        let wanted = lang.map(tesseract_language);
        let has = |l: &str| self.languages.iter().any(|x| x == l);
        wanted
            .filter(|l| has(l))
            .or_else(|| has("eng").then_some("eng"))
            .and_then(|l| self.languages.iter().find(|x| *x == l))
            .map(String::as_str)
    }
}

/// `tesseract --list-langs` prints a header line, then one model per line.
fn parse_languages(out: &str) -> Vec<String> {
    out.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.contains(' ') && *l != "osd")
        .map(str::to_string)
        .collect()
}

/// Tesseract names its models after ISO 639-2/T codes, with a few exceptions.
fn tesseract_language(lang: &str) -> &'static str {
    match lang.to_ascii_lowercase().as_str() {
        "en" | "eng" => "eng",
        "fr" | "fre" | "fra" => "fra",
        "de" | "ger" | "deu" => "deu",
        "es" | "spa" => "spa",
        "it" | "ita" => "ita",
        "pt" | "por" => "por",
        "nl" | "dut" | "nld" => "nld",
        "ru" | "rus" => "rus",
        "pl" | "pol" => "pol",
        "sv" | "swe" => "swe",
        "tr" | "tur" => "tur",
        "ja" | "jpn" => "jpn",
        "ko" | "kor" => "kor",
        "zh" | "chi" | "zho" => "chi_sim",
        "ar" | "ara" => "ara",
        "hi" | "hin" => "hin",
        _ => "",
    }
}

/// Reads picture subtitle stream `index` of `input` and writes it to `out` as WebVTT.
/// Returns the number of cues. Work files live next to `out` and are removed afterwards.
pub async fn ocr_to_vtt(
    bins: &MediaBinaries,
    tool: &OcrTool,
    input: &Path,
    index: i64,
    language: &str,
    duration_s: Option<f64>,
    out: &Path,
) -> Result<usize, MediaError> {
    let work = out.with_extension("ocr");
    let _ = tokio::fs::remove_dir_all(&work).await;
    tokio::fs::create_dir_all(&work).await?;
    let result = run(bins, tool, input, index, language, duration_s, out, &work).await;
    let _ = tokio::fs::remove_dir_all(&work).await;
    result
}

#[allow(clippy::too_many_arguments)]
async fn run(
    bins: &MediaBinaries,
    tool: &OcrTool,
    input: &Path,
    index: i64,
    language: &str,
    duration_s: Option<f64>,
    out: &Path,
    work: &Path,
) -> Result<usize, MediaError> {
    // Frame names are the caption's time in ms (1/1000 time base), so no timing is lost.
    let mut cmd = std::process::Command::new(&bins.ffmpeg);
    cmd.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"])
        .arg(input)
        .arg("-filter_complex")
        .arg(format!("[0:{index}]format=gray"))
        .args([
            "-fps_mode",
            "passthrough",
            "-enc_time_base",
            "1/1000",
            "-frame_pts",
            "1",
        ])
        .arg(work.join("%d.png"));
    output_ok(cmd).await?;

    let mut frames = Vec::new();
    let mut dir = tokio::fs::read_dir(work).await?;
    while let Some(e) = dir.next_entry().await? {
        if let Some(ms) = frame_ms(&e.file_name().to_string_lossy()) {
            frames.push((ms, e.path()));
        }
    }
    frames.sort_by_key(|(ms, _)| *ms);

    // A caption held on screen is rendered again and again; read each picture once.
    let mut kept: Vec<(u64, PathBuf)> = Vec::new();
    let mut last: Option<Vec<u8>> = None;
    for (ms, path) in frames {
        let bytes = tokio::fs::read(&path).await?;
        if last.as_ref() != Some(&bytes) {
            kept.push((ms, path));
            last = Some(bytes);
        }
    }

    let texts = if kept.is_empty() {
        Vec::new()
    } else {
        let list = work.join("frames.txt");
        let lines: String = kept
            .iter()
            .map(|(_, p)| format!("{}\n", p.display()))
            .collect();
        tokio::fs::write(&list, lines).await?;
        let mut cmd = std::process::Command::new(&tool.tesseract);
        // psm 6: one uniform block of text, which is what a caption is.
        // One OpenMP thread: with several, tesseract spins on busy cores and took 50x
        // longer on a loaded 4-core machine (6.4 s against 0.13 s per caption).
        cmd.arg(&list)
            .arg("stdout")
            .args(["-l", language, "--psm", "6"])
            .env("OMP_THREAD_LIMIT", "1");
        let stdout = output_ok(cmd).await?;
        split_pages(&String::from_utf8_lossy(&stdout), kept.len())
    };

    let timed: Vec<(f64, String)> = kept
        .iter()
        .map(|(ms, _)| *ms as f64 / 1000.0)
        .zip(texts)
        .collect();
    let cues = assemble_cues(&timed, duration_s);
    let tmp = out.with_extension("vtt.tmp");
    tokio::fs::write(&tmp, to_vtt(&cues)).await?;
    tokio::fs::rename(&tmp, out).await?;
    Ok(cues.len())
}

/// Runs a background tool at the lowest priority and returns its stdout.
async fn output_ok(mut cmd: std::process::Command) -> Result<Vec<u8>, MediaError> {
    cmd.stdin(Stdio::null());
    kinema_core::platform::configure_low_priority_process(&mut cmd);
    let output = tokio::process::Command::from(cmd)
        .kill_on_drop(true)
        .output()
        .await?;
    if !output.status.success() {
        return Err(MediaError::ProbeFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    Ok(output.stdout)
}

/// "12345.png" to 12345.
fn frame_ms(name: &str) -> Option<u64> {
    name.strip_suffix(".png")?.parse().ok()
}

/// Tesseract ends every page of a list run with a form feed, empty pages included.
fn split_pages(stdout: &str, pages: usize) -> Vec<String> {
    let mut out: Vec<String> = stdout.split('\u{c}').take(pages).map(clean_text).collect();
    out.resize(pages, String::new());
    out
}

/// Trims each line and drops empty ones; OCR noise of a single character is no caption.
fn clean_text(page: &str) -> String {
    let lines: Vec<&str> = page
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let text = lines.join("\n");
    if text.chars().filter(|c| c.is_alphanumeric()).count() < 2 {
        String::new()
    } else {
        text
    }
}

/// Each picture's text lasts until the next picture; an empty one ends a caption.
/// Neighbours read as the same text merge into one cue.
fn assemble_cues(frames: &[(f64, String)], duration_s: Option<f64>) -> Vec<Cue> {
    let mut cues: Vec<Cue> = Vec::new();
    for (i, (start, text)) in frames.iter().enumerate() {
        if text.is_empty() {
            continue;
        }
        // A last caption with no clear event: hold it a few seconds, not to the end credits.
        let end = frames
            .get(i + 1)
            .map(|(t, _)| *t)
            .unwrap_or_else(|| duration_s.unwrap_or(start + 5.0).min(start + 5.0));
        if end <= *start {
            continue;
        }
        match cues.last_mut() {
            Some(prev) if prev.text == *text && (prev.end - start).abs() < 0.001 => prev.end = end,
            _ => cues.push(Cue {
                start: *start,
                end,
                text: text.clone(),
            }),
        }
    }
    cues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_tesseract_language_list() {
        let out = "List of available languages in \"/usr/share/tessdata/\" (3):\neng\nosd\nfra\n";
        assert_eq!(parse_languages(out), ["eng", "fra"]);
    }

    #[test]
    fn picks_a_model_for_the_track() {
        let tool = OcrTool {
            tesseract: "tesseract".into(),
            languages: vec!["eng".into(), "fra".into()],
        };
        assert_eq!(tool.language_for(Some("fre")), Some("fra"));
        assert_eq!(tool.language_for(Some("fr")), Some("fra"));
        assert_eq!(tool.language_for(Some("jpn")), Some("eng"));
        assert_eq!(tool.language_for(None), Some("eng"));
        let no_english = OcrTool {
            languages: vec!["deu".into()],
            ..tool
        };
        assert_eq!(no_english.language_for(Some("ger")), Some("deu"));
        assert_eq!(no_english.language_for(Some("eng")), None);
    }

    #[test]
    fn frame_names() {
        assert_eq!(frame_ms("2999.png"), Some(2999));
        assert_eq!(frame_ms("frames.txt"), None);
        assert_eq!(frame_ms("x.png"), None);
    }

    #[test]
    fn splits_pages_and_cleans_noise() {
        let pages = split_pages("Hello there\n\u{c}\u{c}  Two\n lines \n\n\u{c}|\n\u{c}", 4);
        assert_eq!(pages, ["Hello there", "", "Two\nlines", ""]);
        // Fewer pages than images (a crash mid-run) pads with nothing, not a shifted caption.
        assert_eq!(split_pages("A1\u{c}", 3), ["A1", "", ""]);
    }

    #[test]
    fn builds_cues_from_caption_changes() {
        let frames = [
            (0.0, String::new()),
            (1.0, "Hello".to_string()),
            (2.0, "Hello".to_string()),
            (3.0, String::new()),
            (4.0, "Bye".to_string()),
        ];
        let cues = assemble_cues(&frames, Some(100.0));
        assert_eq!(
            cues,
            [
                Cue {
                    start: 1.0,
                    end: 3.0,
                    text: "Hello".into()
                },
                Cue {
                    start: 4.0,
                    end: 9.0,
                    text: "Bye".into()
                },
            ]
        );
        assert_eq!(assemble_cues(&frames[4..], Some(6.0))[0].end, 6.0);
    }

    /// Real ffmpeg + tesseract on a PGS file with two captions (made by
    /// `tests/fixtures/mksup.py`: 1.0-3.0 s and 4.0-6.5 s). Skipped where either is missing.
    #[tokio::test]
    async fn reads_a_pgs_track() {
        let (Ok(bins), Some(tool)) = (MediaBinaries::resolve(), OcrTool::detect()) else {
            return;
        };
        let Some(lang) = tool.language_for(Some("eng")) else {
            return;
        };
        let sup = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/two-captions.sup");
        let dir = std::env::temp_dir().join(format!("kinema-ocr-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("x.vtt");
        let n = ocr_to_vtt(&bins, &tool, &sup, 0, lang, Some(8.0), &out)
            .await
            .unwrap();
        let vtt = std::fs::read_to_string(&out).unwrap();
        assert_eq!(n, 2, "{vtt}");
        assert!(
            vtt.contains("00:00:00.000 --> 00:00:02.000\nHello there friend"),
            "{vtt}"
        );
        assert!(vtt.contains("00:00:03.000 --> "), "{vtt}");
        assert!(vtt.contains("Second line of text"), "{vtt}");
        assert!(!out.with_extension("ocr").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
