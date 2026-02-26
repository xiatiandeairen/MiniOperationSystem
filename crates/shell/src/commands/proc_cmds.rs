//! Process shell commands: ps, top.

use minios_common::types::ProcessState;
use minios_hal::println;

/// Sets text color based on process state.
fn color_for_state(state: ProcessState) {
    match state {
        ProcessState::Running => {
            minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::GREEN)
        }
        ProcessState::Blocked | ProcessState::Terminated => {
            minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::RED)
        }
        _ => minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT),
    }
}

/// Lists all processes with their PID, name, state, priority, and CPU time.
pub fn cmd_ps(_args: &[&str]) {
    let procs = minios_process::manager::list_processes();
    println!(
        "{:>5} {:8} {:>10} {:>5} {:>10}",
        "PID", "NAME", "STATE", "PRIO", "CPU_TIME"
    );
    for p in &procs {
        let name = core::str::from_utf8(&p.name[..p.name_len]).unwrap_or("?");
        color_for_state(p.state);
        println!(
            "{:>5} {:8} {:>10} {:>5} {:>10}",
            p.pid, name, p.state, p.priority.0, p.cpu_time_ticks
        );
        minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
    }
    super::journey::mark(super::journey::STEP_PS);
}

/// Displays a simple process tree rooted at PID 0.
pub fn cmd_pstree(_args: &[&str]) {
    let procs = minios_process::manager::list_processes();
    println!("Process Tree:");
    for p in &procs {
        let name = core::str::from_utf8(&p.name[..p.name_len]).unwrap_or("?");
        if p.pid.0 == 0 {
            println!("  PID 0 [{}] {}", name, p.state);
        } else {
            println!(
                "  \u{2514}\u{2500} PID {} [{}] {} (cpu: {})",
                p.pid, name, p.state, p.cpu_time_ticks
            );
        }
    }
}

/// Shows a snapshot of system status (processes + memory + interrupts).
pub fn cmd_top(_args: &[&str]) {
    let stats = minios_memory::get_stats();
    let int_stats = minios_hal::interrupts::interrupt_stats();
    let sched = minios_scheduler::SCHEDULER.lock();
    let sched_stats = sched.stats();
    drop(sched);
    let procs = minios_process::manager::list_processes();

    println!("=== System Monitor ===");
    println!();
    println!(
        "Uptime: {} ticks (~{} s)",
        int_stats.timer_count,
        int_stats.timer_count / 100
    );
    println!(
        "Memory: {} / {} frames free ({} KiB)",
        stats.free_frames,
        stats.total_frames,
        stats.free_frames * 4
    );
    println!(
        "Heap:   {} used / {} free",
        stats.heap_used, stats.heap_free
    );
    println!(
        "Sched:  {} switches, {} ticks, {} idle",
        sched_stats.total_switches, sched_stats.total_ticks, sched_stats.idle_ticks
    );
    println!(
        "IRQs:   timer={}, keyboard={}",
        int_stats.timer_count, int_stats.keyboard_count
    );
    println!();
    println!(
        "{:>5} {:8} {:>10} {:>5} {:>10}",
        "PID", "NAME", "STATE", "PRIO", "CPU_TIME"
    );
    for p in &procs {
        let name = core::str::from_utf8(&p.name[..p.name_len]).unwrap_or("?");
        color_for_state(p.state);
        println!(
            "{:>5} {:8} {:>10} {:>5} {:>10}",
            p.pid, name, p.state, p.priority.0, p.cpu_time_ticks
        );
        minios_hal::framebuffer::set_color(minios_hal::framebuffer::colors::DEFAULT);
    }
}
