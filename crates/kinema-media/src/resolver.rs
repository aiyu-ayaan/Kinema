use crate::error::MediaError;
use crate::playback::HwEncoder;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct MediaBinaries {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
    /// An ffmpeg that decodes HEVC on the GPU through `-hwaccel drm` (Raspberry Pi 5). Used for
    /// HEVC re-encodes only. It may be another build than `ffmpeg`: the bundled static ffmpeg has
    /// no V4L2 request support, the distribution's Raspberry Pi build does.
    pub hevc_drm: Option<PathBuf>,
    /// The first GPU encoder `ffmpeg` could use in a test encode; re-encodes fall back to libx264
    /// without one.
    pub hw_encoder: Option<HwEncoder>,
}

impl MediaBinaries {
    /// Resolve ffmpeg and ffprobe paths.
    /// Order:
    /// 1. KINEMA_FFMPEG / KINEMA_FFPROBE env vars
    /// 2. User-updated copy in $XDG_DATA_HOME/kinema/ffmpeg/
    /// 3. Bundled AppDir ($APPDIR/usr/bin), or next to the executable (server tarball)
    /// 4. PATH lookup
    pub fn resolve() -> Result<Self, MediaError> {
        let ffmpeg = Self::resolve_binary("ffmpeg", "KINEMA_FFMPEG")?;
        let ffprobe = Self::resolve_binary("ffprobe", "KINEMA_FFPROBE")?;
        let hevc_drm = find_hevc_drm(&ffmpeg);
        let hw_encoder = find_hw_encoder(&ffmpeg);

        Ok(Self {
            ffmpeg,
            ffprobe,
            hevc_drm,
            hw_encoder,
        })
    }

    fn resolve_binary(binary_name: &str, env_var: &str) -> Result<PathBuf, MediaError> {
        // 1. Explicit env var
        if let Ok(path) = std::env::var(env_var) {
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(p);
            }
        }

        // 2. User-updated directory
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            let candidate = PathBuf::from(xdg_data)
                .join("kinema")
                .join("ffmpeg")
                .join(binary_name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        // 3. Bundled AppDir
        if let Ok(appdir) = std::env::var("APPDIR") {
            let candidate = PathBuf::from(appdir)
                .join("usr")
                .join("bin")
                .join(binary_name);
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        // 3b. Next to the executable
        if let Some(dir) = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(PathBuf::from))
        {
            let candidate = dir.join(format!("{binary_name}{}", std::env::consts::EXE_SUFFIX));
            if candidate.is_file() {
                return Ok(candidate);
            }
        }

        // 4. PATH lookup
        if let Ok(path) = which(binary_name) {
            return Ok(path);
        }

        Err(MediaError::BinaryNotFound(format!(
            "Could not locate '{binary_name}'. Ensure ffmpeg/ffprobe is installed and on PATH or set {env_var}."
        )))
    }
}

/// The resolved ffmpeg first, then the one on PATH. Runs only where the kernel has an HEVC
/// decoder, so other machines pay nothing at startup.
fn find_hevc_drm(resolved: &Path) -> Option<PathBuf> {
    if !kinema_core::platform::has_v4l2_hevc_decoder() {
        return None;
    }
    std::iter::once(resolved.to_path_buf())
        .chain(which("ffmpeg").ok())
        .find(|ffmpeg| hevc_decodes_with_drm(ffmpeg))
}

fn hevc_decodes_with_drm(ffmpeg: &Path) -> bool {
    std::process::Command::new(ffmpeg)
        .args(["-hide_banner", "-h", "decoder=hevc"])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .is_ok_and(|o| lists_drm(&String::from_utf8_lossy(&o.stdout)))
}

/// `Supported hardware devices: cuda drm vaapi vdpau` in `ffmpeg -h decoder=hevc`.
fn lists_drm(help: &str) -> bool {
    help.lines()
        .filter_map(|l| l.trim().strip_prefix("Supported hardware devices:"))
        .any(|devices| devices.split_whitespace().any(|d| d == "drm"))
}

/// An encoder listed by `ffmpeg -encoders` can still lack its driver, GPU or runtime, so each
/// candidate encodes a few frames of black with the exact arguments a stream would use.
/// shortcut: tested once at startup, so a GPU that fails later (driver reset, session limit) fails the
/// stream instead of falling back; retry with libx264 per stream if that shows up.
fn find_hw_encoder(ffmpeg: &Path) -> Option<HwEncoder> {
    [HwEncoder::Nvenc, HwEncoder::Qsv]
        .into_iter()
        .chain(
            kinema_core::platform::render_nodes()
                .into_iter()
                .map(HwEncoder::Vaapi),
        )
        .find(|e| test_encode(ffmpeg, e))
}

fn test_encode(ffmpeg: &Path, encoder: &HwEncoder) -> bool {
    let child = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error"])
        .args(encoder.device_args())
        .args([
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=320x180:r=25:d=0.2",
            "-vf",
        ])
        .arg(format!("scale=-2:180{}", encoder.upload()))
        .args(encoder.codec_args(23, 2_000))
        .args(["-f", "null", "-"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let Ok(mut child) = child else {
        return false;
    };
    // A wedged driver must not hold up startup.
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(50)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
        }
    }
}

pub(crate) fn which(name: &str) -> Result<PathBuf, ()> {
    if let Some(paths) = std::env::var_os("PATH") {
        for path in std::env::split_paths(&paths) {
            let full_path = path.join(name);
            if full_path.is_file() {
                return Ok(full_path);
            }
            #[cfg(windows)]
            {
                let exe_path = path.join(format!("{name}.exe"));
                if exe_path.is_file() {
                    return Ok(exe_path);
                }
            }
        }
    }
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drm_only_from_the_hardware_devices_line() {
        assert!(lists_drm(
            "Decoder hevc\n    Supported hardware devices: cuda drm vaapi vdpau \n"
        ));
        assert!(!lists_drm(
            "    Supported hardware devices: cuda vaapi vulkan \n"
        ));
        assert!(!lists_drm("Supported pixel formats: drm_prime\n"));
    }
}
