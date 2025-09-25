//! Process tools

use std::{thread, time::Duration};

use libc;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

/// Find the process ID (PID) of a program by its name.
///
/// # Arguments
/// * `name` - The name of the process to find.
///
/// # Returns
/// `Some(u32)` containing the PID if the process is found, or `None` if the process
/// is not found after the maximum wait time.
///
/// # Description
/// This function attempts to find a process with the specified name by checking
/// the system's process list. It will retry for up to 10 seconds (1 second intervals)
/// before giving up. This is useful when waiting for a process to start.
pub fn pidof(name: &str) -> Option<u32> {
    let mut wait_time = 0;
    let max_wait_time = 10;
    log::debug!("Get pid of {}...", name);
    let mut sys = System::new_all();
    while wait_time < max_wait_time {
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything().without_tasks(),
        );
        for (pid, process) in sys.processes() {
            log::debug!("Process {}: {:?}", pid.as_u32(), process.name());
            if *name == *process.name() {
                return Some(pid.as_u32());
            }
        }
        thread::sleep(Duration::from_secs(1));
        wait_time += 1;
    }
    None
}

/// Check if a process with the given PID is alive.
///
/// # Arguments
/// * `pid` - The process ID to check.
///
/// # Returns
/// `true` if the process is alive, `false` otherwise.
///
/// # Description
/// This function uses the POSIX `kill` system call with signal 0 to check
/// if a process exists and the calling process has permission to send signals to it.
/// It does not actually send a signal to the process.
pub fn is_proc_alive(pid: u32) -> bool {
    let status = unsafe { libc::kill(pid as i32, 0) };
    if status == -1 { false } else { true }
}
