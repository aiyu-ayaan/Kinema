use std::process::Command;

pub fn configure_low_priority_process(_cmd: &mut Command) {}

pub fn configure_playback_process(_cmd: &mut Command) {}

pub fn suspend_process(_pid: u32, _suspend: bool) -> Result<(), std::io::Error> {
    Err(std::io::ErrorKind::Unsupported.into())
}

pub fn kill_child_process_group(_pid: u32) -> Result<(), std::io::Error> {
    Ok(())
}
