//! Startup checks. Each check looks at the machine once, logs what it found and records the result in [`Report`];
//! nothing re-probes while the server runs. What clients may ask for is [`Features`]: the report combined with the
//! live admin settings, sent with the item detail so a client never requests a feature the server will refuse.
//!
//! To add a boot task: add a field to [`Report`], fill it in [`run`] (log the outcome there), then expose it through
//! a field of [`Features`] if clients need to know. See `development/docs/boot-checks.md`.

use crate::server_settings::ServerSettings;
use kinema_media::MediaBinaries;
use kinema_media::ocr::OcrTool;
use serde::Serialize;
use std::sync::Arc;
use tracing::{info, warn};

/// What the startup checks found. Immutable for the life of the process.
pub struct Report {
    /// `None` when ffmpeg/ffprobe are missing: files are still listed and direct-played.
    pub bins: Option<Arc<MediaBinaries>>,
    /// `tesseract` for picture subtitles; `None` leaves them picture-only.
    pub ocr: Option<Arc<OcrTool>>,
}

/// Runs every boot check, in order.
pub fn run() -> Report {
    Report {
        bins: media_tools(),
        ocr: ocr(),
    }
}

fn media_tools() -> Option<Arc<MediaBinaries>> {
    match MediaBinaries::resolve() {
        Ok(b) => {
            info!("ffmpeg: {}", b.ffmpeg.display());
            if let Some(hw) = &b.hevc_drm {
                info!(
                    "HEVC hardware decode for transcodes: {} (drm)",
                    hw.display()
                );
            }
            match &b.hw_encoder {
                Some(e) => info!("Hardware encoder for transcodes: {}", e.name()),
                None => info!("No hardware encoder found; transcodes encode with libx264"),
            }
            Some(Arc::new(b))
        }
        Err(e) => {
            warn!("{e} Files will be listed and direct-played only.");
            None
        }
    }
}

fn ocr() -> Option<Arc<OcrTool>> {
    let tool = OcrTool::detect().map(Arc::new);
    match &tool {
        Some(t) => info!(
            "Subtitle OCR: {} ({})",
            t.tesseract.display(),
            t.languages.join(", ")
        ),
        None => info!("tesseract not found; picture subtitles (PGS, VobSub) cannot be read"),
    }
    tool
}

/// What clients may use right now. A feature is on only when the machine can do it (boot report) and the admin
/// left it on (settings).
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct Features {
    /// Seek-bar preview tiles: needs ffmpeg and the admin's `scrub_thumbnails` switch.
    pub scrub_thumbnails: bool,
}

impl Features {
    pub fn new(bins: Option<&MediaBinaries>, settings: &ServerSettings) -> Self {
        Self {
            scrub_thumbnails: bins.is_some() && settings.scrub_thumbnails,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrub_needs_ffmpeg_and_the_switch() {
        let on = ServerSettings {
            scrub_thumbnails: true,
            ..Default::default()
        };
        let bins = MediaBinaries::default();
        assert!(Features::new(Some(&bins), &on).scrub_thumbnails);
        assert!(!Features::new(None, &on).scrub_thumbnails);
        assert!(!Features::new(Some(&bins), &ServerSettings::default()).scrub_thumbnails);
    }
}
