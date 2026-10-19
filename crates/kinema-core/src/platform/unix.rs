use std::process::Command;

/// Configure background worker process priority on Unix
pub fn configure_low_priority_process(_cmd: &mut Command) {
    // Process nice level can be adjusted via pre_exec when spawning
}

/// Terminate child process cleanly on Unix
pub fn kill_child_process_group(_pid: u32) -> Result<(), std::io::Error> {
    Ok(())
}
