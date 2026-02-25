//! Process shell commands: ps.

use minios_hal::println;

/// Lists all processes with their PID, state, priority, and CPU time.
pub fn cmd_ps(_args: &[&str]) {
    let procs = minios_process::manager::list_processes();
    println!(
        "{:<6} {:<12} {:<10} {}",
        "PID", "STATE", "PRIORITY", "CPU_TIME"
    );
    for p in &procs {
        println!(
            "{:<6} {:<12} {:<10} {}",
            p.pid, p.state, p.priority.0, p.cpu_time_ticks
        );
    }
}
