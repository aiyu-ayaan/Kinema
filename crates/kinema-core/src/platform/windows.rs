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

/// Pauses or resumes a child like SIGSTOP / SIGCONT: an HLS run far enough ahead of the player waits
/// without holding any CPU. Windows has no public API for it; ntdll's NtSuspendProcess is what Process
/// Explorer and the resource monitor use.
pub fn suspend_process(pid: u32, suspend: bool) -> Result<(), std::io::Error> {
    type Handle = *mut core::ffi::c_void;
    const PROCESS_SUSPEND_RESUME: u32 = 0x0800;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> Handle;
        fn CloseHandle(handle: Handle) -> i32;
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtSuspendProcess(handle: Handle) -> i32;
        fn NtResumeProcess(handle: Handle) -> i32;
    }
    // SAFETY: the pid is a child we spawned and still own; the handle is closed before returning.
    unsafe {
        let handle = OpenProcess(PROCESS_SUSPEND_RESUME, 0, pid);
        if handle.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        let status = if suspend {
            NtSuspendProcess(handle)
        } else {
            NtResumeProcess(handle)
        };
        CloseHandle(handle);
        if status < 0 {
            return Err(std::io::Error::other(format!("NTSTATUS {status:#x}")));
        }
    }
    Ok(())
}
