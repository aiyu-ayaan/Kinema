use crate::{MediaBinaries, MediaError};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Probe {
    pub container: Option<String>,
    pub duration_s: Option<f64>,
    pub bitrate: Option<i64>,
    pub streams: Vec<ProbeStream>,
}

#[derive(Debug, Clone)]
pub struct ProbeStream {
    pub index: i64,
    /// video | audio | subtitle; data and attachment streams are dropped.
    pub kind: String,
    pub codec: Option<String>,
    pub profile: Option<String>,
    pub language: Option<String>,
    pub title: Option<String>,
    pub channels: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub is_default: bool,
}

#[derive(Deserialize)]
struct Raw {
    #[serde(default)]
    format: RawFormat,
    #[serde(default)]
    streams: Vec<RawStream>,
}

#[derive(Deserialize, Default)]
struct RawFormat {
    format_name: Option<String>,
    duration: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Deserialize)]
struct RawStream {
    index: i64,
    codec_type: Option<String>,
    codec_name: Option<String>,
    profile: Option<String>,
    channels: Option<i64>,
    width: Option<i64>,
    height: Option<i64>,
    #[serde(default)]
    tags: HashMap<String, String>,
    #[serde(default)]
    disposition: HashMap<String, i64>,
}

/// Runs ffprobe once per file; callers cache the result by size + mtime.
pub async fn probe(bins: &MediaBinaries, path: &Path) -> Result<Probe, MediaError> {
    let mut cmd = std::process::Command::new(&bins.ffprobe);
    cmd.args(["-v", "error", "-print_format", "json", "-show_format", "-show_streams"])
        .arg(path);
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
    parse(&out.stdout)
}

fn parse(json: &[u8]) -> Result<Probe, MediaError> {
    let raw: Raw = serde_json::from_slice(json)?;
    let streams = raw
        .streams
        .into_iter()
        .filter(|s| {
            matches!(
                s.codec_type.as_deref(),
                Some("video" | "audio" | "subtitle")
            )
        })
        // Cover art in MKV/MP4 shows up as a one-frame video stream.
        .filter(|s| s.disposition.get("attached_pic").copied().unwrap_or(0) == 0)
        .map(|s| {
            let tag = |k: &str| {
                s.tags
                    .iter()
                    .find(|(key, _)| key.eq_ignore_ascii_case(k))
                    .map(|(_, v)| v.clone())
            };
            ProbeStream {
                index: s.index,
                kind: s.codec_type.clone().unwrap_or_default(),
                codec: s.codec_name.clone(),
                profile: s.profile.clone(),
                language: tag("language").filter(|l| l != "und"),
                title: tag("title"),
                channels: s.channels,
                width: s.width,
                height: s.height,
                is_default: s.disposition.get("default").copied().unwrap_or(0) == 1,
            }
        })
        .collect();
    Ok(Probe {
        container: raw.format.format_name,
        duration_s: raw.format.duration.and_then(|d| d.parse().ok()),
        bitrate: raw.format.bit_rate.and_then(|b| b.parse().ok()),
        streams,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ffprobe_json() {
        let json = br#"{"streams":[
            {"index":0,"codec_type":"video","codec_name":"h264","profile":"High","width":1920,"height":1080,"disposition":{"default":1}},
            {"index":1,"codec_type":"audio","codec_name":"ac3","channels":6,"tags":{"language":"eng","title":"Surround"}},
            {"index":2,"codec_type":"subtitle","codec_name":"subrip","tags":{"LANGUAGE":"und"}},
            {"index":3,"codec_type":"video","codec_name":"mjpeg","disposition":{"attached_pic":1}},
            {"index":4,"codec_type":"attachment"}],
          "format":{"format_name":"matroska,webm","duration":"5400.5","bit_rate":"8000000"}}"#;
        let p = parse(json).unwrap();
        assert_eq!(p.container.as_deref(), Some("matroska,webm"));
        assert_eq!(p.duration_s, Some(5400.5));
        assert_eq!(p.streams.len(), 3);
        assert!(p.streams[0].is_default);
        assert_eq!(p.streams[1].language.as_deref(), Some("eng"));
        assert_eq!(p.streams[1].title.as_deref(), Some("Surround"));
        assert_eq!(p.streams[2].language, None);
    }
}
