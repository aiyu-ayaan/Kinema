use crate::error::MediaError;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MediaBinaries {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
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

        Ok(Self { ffmpeg, ffprobe })
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

fn which(name: &str) -> Result<PathBuf, ()> {
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
