//! Scheduler shell commands: spawn, kill, sched, nice.

use minios_common::id::Pid;
use minios_common::types::{Priority, ProcessState};
use minios_hal::println;

/// Parses a decimal string into `u32` without relying on `std`.
fn parse_u32(s: &str) -> Option<u32> {
    let mut result: u32 = 0;
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }
    Some(result)
}

/// Entry point for a background kernel task (loops halting).
fn background_task() {
    loop {
        minios_hal::cpu::hlt();
    }
}

/// Spawns a new background kernel task and adds it to the scheduler.
pub fn cmd_spawn(args: &[&str]) {
    let name = if args.is_empty() { "task" } else { args[0] };
    let pid = minios_process::manager::create_kernel_task(name, background_task, Priority::MEDIUM);
    match pid {
        Ok(pid) => {
            minios_scheduler::SCHEDULER
                .lock()
                .add_task(pid, Priority::MEDIUM);
            println!("Spawned '{}' with PID {}", name, pid);
        }
        Err(e) => println!("spawn: {}", e),
    }
}

/// Terminates a process by PID and removes it from the scheduler.
pub fn cmd_kill(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: kill <pid>");
        return;
    }
    let pid_num = match parse_u32(args[0]) {
        Some(n) => n,
        None => {
            println!("kill: invalid pid '{}'", args[0]);
            return;
        }
    };
    let pid = Pid(pid_num);
    minios_process::manager::set_state(pid, ProcessState::Terminated).ok();
    minios_scheduler::SCHEDULER.lock().remove_task(pid);
    println!("Process {} terminated.", pid);
}

/// Displays scheduler queue lengths, runtime statistics, or configuration.
pub fn cmd_sched(args: &[&str]) {
    let sub = if args.is_empty() { "status" } else { args[0] };
    match sub {
        "status" | "" => sched_status(),
        "config" => sched_config(&args[1..]),
        _ => println!("Usage: sched [status|config [boost <N>]]"),
    }
}

fn sched_status() {
    let sched = minios_scheduler::SCHEDULER.lock();
    let stats = sched.stats();
    let names = ["HIGH", "MED", "LOW", "IDLE"];
    for (i, name) in names.iter().enumerate() {
        println!("Queue {} [{}]: {} tasks", i, name, stats.queue_lengths[i]);
    }
    if let Some(pid) = sched.current_pid() {
        println!("Running: PID {}", pid);
    }
    println!(
        "Total switches: {}, Total ticks: {}",
        stats.total_switches, stats.total_ticks
    );
}

fn sched_config(args: &[&str]) {
    if args.is_empty() {
        println!("MLFQ Configuration:");
        println!("  Queue 0 [HIGH]: time_slice = 2 ticks");
        println!("  Queue 1 [MED]:  time_slice = 4 ticks");
        println!("  Queue 2 [LOW]:  time_slice = 8 ticks");
        println!("  Queue 3 [IDLE]: time_slice = 16 ticks");
        let sched = minios_scheduler::SCHEDULER.lock();
        let stats = sched.stats();
        println!("  Total switches: {}", stats.total_switches);
        println!("  Total ticks:    {}", stats.total_ticks);
        println!("  Idle ticks:     {}", stats.idle_ticks);
        return;
    }
    println!("Config display only in v0.3.0. Runtime modification coming later.");
}

/// Changes the scheduling priority of a process.
pub fn cmd_nice(args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: nice <pid> <priority 0-3>");
        return;
    }
    let pid = Pid(parse_u32(args[0]).unwrap_or(0));
    let prio = parse_u32(args[1]).unwrap_or(0) as u8;
    let clamped = prio.min(3);
    minios_scheduler::SCHEDULER.lock().remove_task(pid);
    minios_scheduler::SCHEDULER
        .lock()
        .add_task(pid, Priority(clamped));
    println!("Set PID {} priority to {}", pid, clamped);
}
