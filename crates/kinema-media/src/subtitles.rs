use crate::{MediaBinaries, MediaError};
use std::path::Path;

/// Text formats ffmpeg converts to WebVTT. Bitmap subtitles (PGS, VobSub) are not.
pub fn is_text_subtitle(codec: Option<&str>) -> bool {
    matches!(
        codec.unwrap_or_default(),
        "subrip" | "srt" | "ass" | "ssa" | "webvtt" | "mov_text" | "text"
    )
}

/// Writes subtitle stream `index` of `input` (or the whole file when `index` is None,
/// for sidecar files) to `out` as WebVTT. Writes to a temp file first so a
/// half-written cache entry is never served.
pub async fn extract_vtt(
    bins: &MediaBinaries,
    input: &Path,
    index: Option<i64>,
    out: &Path,
) -> Result<(), MediaError> {
    let tmp = out.with_extension("vtt.tmp");
    let mut cmd = std::process::Command::new(&bins.ffmpeg);
    cmd.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"])
        .arg(input);
    if let Some(i) = index {
        cmd.arg("-map").arg(format!("0:{i}"));
    }
    cmd.args(["-f", "webvtt"]).arg(&tmp);
    kinema_core::platform::configure_low_priority_process(&mut cmd);
    let output = tokio::process::Command::from(cmd)
        .kill_on_drop(true)
        .output()
        .await?;
    if !output.status.success() {
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(MediaError::ProbeFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    tokio::fs::rename(&tmp, out).await?;
    Ok(())
}
