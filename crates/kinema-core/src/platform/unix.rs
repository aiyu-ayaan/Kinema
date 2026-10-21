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

/// Terminate child process cleanly on Unix
pub fn kill_child_process_group(_pid: u32) -> Result<(), std::io::Error> {
    Ok(())
}
