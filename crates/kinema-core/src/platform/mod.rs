use std::path::{Path, PathBuf};

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(not(any(unix, windows)))]
mod fallback;
#[cfg(not(any(unix, windows)))]
pub use fallback::*;

/// Common trait/helper for platform specific operations
pub fn default_data_dir() -> PathBuf {
    #[cfg(unix)]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return PathBuf::from(xdg).join("kinema");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("kinema");
        }
    }

    #[cfg(windows)]
    {
        if let Ok(app_data) = std::env::var("APPDATA") {
            return PathBuf::from(app_data).join("Kinema");
        }
    }

    PathBuf::from("data")
}

/// This machine's network name, for the self-signed certificate.
pub fn hostname() -> Option<String> {
    #[cfg(unix)]
    let name = std::fs::read_to_string("/etc/hostname")
        .ok()
        .or_else(|| std::env::var("HOSTNAME").ok());
    #[cfg(windows)]
    let name = std::env::var("COMPUTERNAME").ok();
    #[cfg(not(any(unix, windows)))]
    let name: Option<String> = None;
    name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty())
}

/// Writes a secret (TLS key) readable only by this user where the OS supports file modes.
pub fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)?.write_all(bytes)
}

/// A V4L2 HEVC decoder in the kernel, like the Raspberry Pi 5's `rpi-hevc-dec`. Only an ffmpeg
/// built with the V4L2 request API can use it, through `-hwaccel drm`.
pub fn has_v4l2_hevc_decoder() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_dir("/sys/class/video4linux")
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|d| std::fs::read_to_string(d.path().join("name")).ok())
            .any(|name| name.trim().ends_with("hevc-dec"))
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// One-minute load average per core. None where the OS has no `/proc/loadavg` (Windows).
pub fn load_per_core() -> Option<f64> {
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/loadavg").ok()?;
        let load: f64 = text.split_whitespace().next()?.parse().ok()?;
        let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
        Some(load / cores as f64)
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// GPU render nodes (`/dev/dri/renderD128`, ...), the devices VA-API encodes on. Linux only:
/// Windows reaches the GPU through NVENC and Quick Sync without a device path.
pub fn render_nodes() -> Vec<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let mut nodes: Vec<PathBuf> = std::fs::read_dir("/dev/dri")
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("renderD"))
            })
            .collect();
        nodes.sort();
        nodes
    }
    #[cfg(not(target_os = "linux"))]
    {
        Vec::new()
    }
}

/// Kernel and runtime pseudo-filesystems: never media, and walking them can hang
/// (`/proc` has self-referencing links and files that block on read).
pub fn is_virtual_fs(path: &Path) -> bool {
    #[cfg(unix)]
    {
        ["/proc", "/sys", "/dev", "/run"]
            .iter()
            .any(|p| path.starts_with(p))
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        false
    }
}

/// A folder that can be a library root: not a whole drive and not a pseudo-filesystem.
pub fn is_allowed_library_root(path: &Path) -> bool {
    path.parent().is_some() && !is_virtual_fs(path)
}

/// Top-level folders for a folder picker: "/" on Unix, drive letters on Windows.
pub fn filesystem_roots() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        (b'A'..=b'Z')
            .map(|d| PathBuf::from(format!("{}:\\", d as char)))
            .filter(|p| p.is_dir())
            .collect()
    }
    #[cfg(not(windows))]
    {
        vec![PathBuf::from("/")]
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn library_roots() {
        assert!(!is_allowed_library_root(Path::new("/")));
        assert!(!is_allowed_library_root(Path::new("/proc/tty")));
        assert!(is_allowed_library_root(Path::new("/mnt/Void/Movies")));
        assert!(is_allowed_library_root(Path::new("/home/a/proc")));
    }
}
