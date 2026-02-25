//! Process-related system calls.

/// Returns the PID of the currently running process.
pub fn sys_getpid() -> i64 {
    let pid = minios_process::manager::current_pid();
    pid.0 as i64
}

/// Yields the CPU, allowing the scheduler to pick another task.
///
/// Always returns `0`.
pub fn sys_yield() -> i64 {
    minios_scheduler::SCHEDULER.lock().next_task();
    0
}

/// Terminates the current process with the given exit code.
///
/// Marks the process as terminated and enters a halt loop.
pub fn sys_exit(code: i64) -> ! {
    minios_process::manager::exit_current(code as i32);
    minios_hal::cpu::hlt_loop();
}
