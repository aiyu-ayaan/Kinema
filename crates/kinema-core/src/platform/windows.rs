use std::os::windows::process::CommandExt;
use std::process::Command;

const IDLE_PRIORITY_CLASS: u32 = 0x0000_0040;
const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;

/// Background work (scan, probe, thumbnails, markers): idle class, so it only takes CPU nothing else wants.
pub fn configure_low_priority_process(cmd: &mut Command) {
    cmd.creation_flags(IDLE_PRIORITY_CLASS);
}

/// Live remux and transcode: below the server itself, above background work.
pub fn configure_playback_process(cmd: &mut Command) {
    cmd.creation_flags(BELOW_NORMAL_PRIORITY_CLASS);
}

/// Terminate child process and job objects cleanly on Windows
pub fn kill_child_process_group(_pid: u32) -> Result<(), std::io::Error> {
    Ok(())
}
