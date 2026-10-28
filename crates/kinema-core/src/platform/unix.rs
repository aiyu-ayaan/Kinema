use std::os::unix::process::CommandExt;
use std::process::Command;

/// Background work (scan, probe, thumbnails, markers): nice 19, so it only takes CPU nothing else wants.
pub fn configure_low_priority_process(cmd: &mut Command) {
    lower_priority(cmd, 19);
}

/// Live remux and transcode: below the server itself, so the UI and other streams stay quick under a
/// full re-encode, and above background work, so a running scan never stalls playback.
pub fn configure_playback_process(cmd: &mut Command) {
    lower_priority(cmd, 10);
}

fn lower_priority(cmd: &mut Command, increment: i32) {
    // SAFETY: nice() is async-signal-safe and only changes the forked child's own priority.
    unsafe {
        cmd.pre_exec(move || {
            libc::nice(increment);
            Ok(())
        });
    }
}

/// Pauses (SIGSTOP) or resumes (SIGCONT) a child: an HLS run far enough ahead of the player waits
/// without holding any CPU, and keeps its place.
pub fn suspend_process(pid: u32, suspend: bool) -> Result<(), std::io::Error> {
    let signal = if suspend {
        libc::SIGSTOP
    } else {
        libc::SIGCONT
    };
    let pid = libc::pid_t::try_from(pid).map_err(std::io::Error::other)?;
    // SAFETY: kill() only sends a signal; the pid is a child we spawned and still own.
    if unsafe { libc::kill(pid, signal) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}
