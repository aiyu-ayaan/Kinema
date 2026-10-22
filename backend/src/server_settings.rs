//! Admin-editable server options, stored as one JSON row in `settings` and cached in
//! `AppState`. Every field is read by the code that uses it; nothing here is decorative.

use kinema_db::{DbError, SqlitePool, settings};
use serde::{Deserialize, Serialize};

const KEY: &str = "server";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ServerSettings {
    /// Shown on the sign-in screen and in the browser title.
    pub server_name: String,
    /// Address apps use away from home (reverse proxy, tunnel), e.g. `https://media.example.com`.
    /// Empty when the server is only reachable on the LAN.
    pub public_url: String,
    /// Rescan a library when files change (filesystem events).
    pub watch_changes: bool,
    /// Generate video-frame thumbnails in the background after scans.
    pub thumbnails: bool,
    /// Where in the video the frame is taken, in percent of the runtime.
    pub thumbnail_position: u32,
    /// TMDB `language`, e.g. "en-US".
    pub metadata_language: String,
    /// ISO 639-2 code ("eng") or "" for the file's default track.
    pub preferred_audio_language: String,
    pub preferred_subtitle_language: String,
    /// off | always | foreign (only when the audio is not in the preferred audio language).
    pub subtitle_mode: String,
    pub max_transcodes: u32,
    /// libx264 preset.
    pub transcode_preset: String,
    /// libx264 CRF: lower is better quality and more bits.
    pub transcode_crf: u32,
    /// 0 keeps the source height.
    pub transcode_max_height: u32,
    /// Threads per conversion; 0 shares the cores between `max_transcodes`.
    pub transcode_threads: u32,
    /// Encode on the GPU when ffmpeg found one at startup.
    pub transcode_hardware: bool,
    /// Periodic full rescan for drives without filesystem events (network shares). 0 = off.
    pub rescan_hours: u32,
    /// Read picture subtitles (PGS, VobSub) into WebVTT with OCR after scans, when
    /// `tesseract` is installed. Off by default: it is slow on a large library.
    pub subtitle_ocr: bool,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            server_name: "Kinema".into(),
            public_url: String::new(),
            watch_changes: true,
            thumbnails: true,
            thumbnail_position: 15,
            metadata_language: "en-US".into(),
            preferred_audio_language: String::new(),
            preferred_subtitle_language: String::new(),
            subtitle_mode: "off".into(),
            max_transcodes: 2,
            transcode_preset: "veryfast".into(),
            transcode_crf: 22,
            transcode_max_height: 1080,
            transcode_threads: 0,
            transcode_hardware: true,
            rescan_hours: 0,
            subtitle_ocr: false,
        }
    }
}

pub const PRESETS: &[&str] = &[
    "ultrafast",
    "superfast",
    "veryfast",
    "faster",
    "fast",
    "medium",
];
pub const MAX_HEIGHTS: &[u32] = &[0, 2160, 1440, 1080, 720, 480];
pub const THREADS: &[u32] = &[0, 1, 2, 3, 4, 6, 8, 12, 16];
pub const SUBTITLE_MODES: &[&str] = &["off", "always", "foreign"];
pub const METADATA_LANGUAGES: &[&str] = &[
    "en-US", "en-GB", "hi-IN", "ta-IN", "te-IN", "ml-IN", "kn-IN", "bn-IN", "es-ES", "es-MX",
    "fr-FR", "de-DE", "it-IT", "pt-BR", "pt-PT", "ru-RU", "ja-JP", "ko-KR", "zh-CN", "zh-TW",
    "ar-SA", "tr-TR", "nl-NL", "pl-PL", "sv-SE",
];

/// Track languages offered in the UI: ISO 639-2 code and the ISO 639-1 / bibliographic
/// aliases files also use.
pub const TRACK_LANGUAGES: &[(&str, &[&str])] = &[
    ("eng", &["en"]),
    ("hin", &["hi"]),
    ("jpn", &["ja"]),
    ("kor", &["ko"]),
    ("spa", &["es"]),
    ("fra", &["fr", "fre"]),
    ("deu", &["de", "ger"]),
    ("ita", &["it"]),
    ("por", &["pt"]),
    ("rus", &["ru"]),
    ("zho", &["zh", "chi"]),
    ("ara", &["ar"]),
    ("tam", &["ta"]),
    ("tel", &["te"]),
    ("mal", &["ml"]),
    ("kan", &["kn"]),
    ("ben", &["bn"]),
    ("mar", &["mr"]),
    ("urd", &["ur"]),
    ("tur", &["tr"]),
    ("tha", &["th"]),
    ("ind", &["id"]),
    ("vie", &["vi"]),
    ("nld", &["nl", "dut"]),
    ("pol", &["pl"]),
    ("swe", &["sv"]),
];

/// Does a track tagged `lang` match the preferred ISO 639-2 code?
pub fn lang_matches(lang: Option<&str>, preferred: &str) -> bool {
    let Some(lang) = lang.map(str::to_ascii_lowercase) else {
        return false;
    };
    if preferred.is_empty() {
        return false;
    }
    lang == preferred
        || TRACK_LANGUAGES
            .iter()
            .find(|(code, _)| *code == preferred)
            .is_some_and(|(_, aliases)| aliases.contains(&lang.as_str()))
}

impl ServerSettings {
    /// Clamps numbers and falls back to defaults for unknown choices, so a bad
    /// request can never store a value the transcoder or scanner cannot use.
    pub fn sanitized(mut self) -> Self {
        let d = Self::default();
        self.server_name = self.server_name.trim().chars().take(60).collect();
        if self.server_name.is_empty() {
            self.server_name = d.server_name;
        }
        self.public_url = clean_url(&self.public_url);
        self.thumbnail_position = self.thumbnail_position.clamp(5, 90);
        if !METADATA_LANGUAGES.contains(&self.metadata_language.as_str()) {
            self.metadata_language = d.metadata_language;
        }
        let known = |l: &str| l.is_empty() || TRACK_LANGUAGES.iter().any(|(c, _)| *c == l);
        if !known(&self.preferred_audio_language) {
            self.preferred_audio_language.clear();
        }
        if !known(&self.preferred_subtitle_language) {
            self.preferred_subtitle_language.clear();
        }
        if !SUBTITLE_MODES.contains(&self.subtitle_mode.as_str()) {
            self.subtitle_mode = d.subtitle_mode;
        }
        self.max_transcodes = self.max_transcodes.clamp(1, 8);
        if !PRESETS.contains(&self.transcode_preset.as_str()) {
            self.transcode_preset = d.transcode_preset;
        }
        self.transcode_crf = self.transcode_crf.clamp(16, 30);
        if !MAX_HEIGHTS.contains(&self.transcode_max_height) {
            self.transcode_max_height = d.transcode_max_height;
        }
        if !THREADS.contains(&self.transcode_threads) {
            self.transcode_threads = d.transcode_threads;
        }
        self.rescan_hours = self.rescan_hours.min(168);
        self
    }
}

/// `http(s)://host[:port][/path]` without a trailing slash, or "" for anything else.
fn clean_url(url: &str) -> String {
    let url = url.trim().trim_end_matches('/');
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    match rest {
        Some(r) if !r.is_empty() && url.len() <= 200 && !r.contains(char::is_whitespace) => {
            url.to_string()
        }
        _ => String::new(),
    }
}

pub async fn load(db: &SqlitePool) -> Result<ServerSettings, DbError> {
    Ok(settings::get(db, KEY)
        .await?
        .and_then(|json| serde_json::from_str::<ServerSettings>(&json).ok())
        .unwrap_or_default()
        .sanitized())
}

pub async fn save(db: &SqlitePool, s: &ServerSettings) -> Result<(), DbError> {
    let json = serde_json::to_string(s).unwrap_or_default();
    settings::set(db, KEY, &json).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_bad_values() {
        let s = ServerSettings {
            server_name: "  ".into(),
            transcode_crf: 99,
            transcode_preset: "placebo".into(),
            transcode_max_height: 999,
            transcode_threads: 5,
            preferred_audio_language: "xx".into(),
            subtitle_mode: "sometimes".into(),
            ..Default::default()
        }
        .sanitized();
        let d = ServerSettings::default();
        assert_eq!(s.server_name, d.server_name);
        assert_eq!(s.transcode_crf, 30);
        assert_eq!(s.transcode_preset, d.transcode_preset);
        assert_eq!(s.transcode_max_height, d.transcode_max_height);
        assert_eq!(s.transcode_threads, 0);
        assert_eq!(s.preferred_audio_language, "");
        assert_eq!(s.subtitle_mode, "off");
    }

    #[test]
    fn public_url() {
        assert_eq!(
            clean_url(" https://media.example.com/ "),
            "https://media.example.com"
        );
        assert_eq!(clean_url("http://1.2.3.4:3000"), "http://1.2.3.4:3000");
        assert_eq!(clean_url("media.example.com"), "");
        assert_eq!(clean_url("https://"), "");
        assert_eq!(clean_url("ftp://x"), "");
    }

    #[test]
    fn languages() {
        assert!(lang_matches(Some("eng"), "eng"));
        assert!(lang_matches(Some("EN"), "eng"));
        assert!(lang_matches(Some("ger"), "deu"));
        assert!(!lang_matches(Some("hin"), "eng"));
        assert!(!lang_matches(None, "eng"));
        assert!(!lang_matches(Some("eng"), ""));
    }
}
