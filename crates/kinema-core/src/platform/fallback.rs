use std::process::Command;

pub fn configure_low_priority_process(_cmd: &mut Command) {}

pub fn kill_child_process_group(_pid: u32) -> Result<(), std::io::Error> {
    Ok(())
}
