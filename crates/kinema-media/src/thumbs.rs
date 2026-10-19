use crate::{MediaBinaries, MediaError};
use std::path::Path;

/// One JPEG frame at `at_s`, 640 px wide, for items without artwork. Seeks on the
/// input and decodes only the next keyframe, so a 4K HEVC file costs one frame.
/// Falls back to a full decode when no keyframe follows `at_s` (short or sparse-GOP files).
pub async fn frame(
    bins: &MediaBinaries,
    input: &Path,
    at_s: f64,
    out: &Path,
) -> Result<(), MediaError> {
    match grab(bins, input, at_s, out, true).await {
        Ok(()) => Ok(()),
        Err(_) => grab(bins, input, at_s, out, false).await,
    }
}

async fn grab(
    bins: &MediaBinaries,
    input: &Path,
    at_s: f64,
    out: &Path,
    keyframes_only: bool,
) -> Result<(), MediaError> {
    let tmp = out.with_extension("jpg.tmp");
    let mut cmd = std::process::Command::new(&bins.ffmpeg);
    cmd.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y"]);
    if keyframes_only {
        cmd.args(["-skip_frame", "nokey"]);
    }
    cmd.arg("-ss")
        .arg(format!("{:.3}", at_s.max(0.0)))
        .arg("-i")
        .arg(input)
        .args([
            "-map",
            "0:v:0",
            "-frames:v",
            "1",
            "-vf",
            "scale=640:-2",
            "-q:v",
            "4",
            "-f",
            "image2",
            "-c:v",
            "mjpeg",
        ])
        .arg(&tmp);
    kinema_core::platform::configure_low_priority_process(&mut cmd);
    let output = tokio::process::Command::from(cmd)
        .kill_on_drop(true)
        .output()
        .await?;
    let made = tokio::fs::metadata(&tmp)
        .await
        .map(|m| m.len() > 0)
        .unwrap_or(false);
    if !output.status.success() || !made {
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(MediaError::ProbeFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    tokio::fs::rename(&tmp, out).await?;
    Ok(())
}
