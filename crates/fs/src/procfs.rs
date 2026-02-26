//! ProcFS — read-only virtual filesystem generating live content.
//!
//! Content is generated at read time (not cached), so values always
//! reflect the current kernel state.

extern crate alloc;

use alloc::format;
use alloc::vec::Vec;

use minios_common::error::FsError;
use minios_common::types::{FileStat, InodeType};

/// Generates live content for a procfs virtual file.
///
/// Each call produces fresh data from the kernel's current state.
pub fn read_procfs(path: &str) -> Result<Vec<u8>, FsError> {
    match path {
        "/proc/meminfo" => Ok(generate_meminfo()),
        "/proc/uptime" => Ok(generate_uptime()),
        "/proc/interrupts" => Ok(generate_interrupts()),
        "/proc/scheduler" => Ok(generate_scheduler()),
        "/proc/version" => Ok(generate_version()),
        "/proc/trace" => Ok(generate_trace_stats()),
        _ if path.starts_with("/proc/") && path.ends_with("/status") => {
            let pid_str = &path[6..path.len() - 7];
            let pid_num = pid_str.bytes().fold(0u32, |a, b| {
                if (b'0'..=b'9').contains(&b) {
                    a * 10 + (b - b'0') as u32
                } else {
                    a
                }
            });
            Ok(generate_process_status(pid_num))
        }
        _ => Err(FsError::NotFound),
    }
}

fn generate_process_status(pid: u32) -> Vec<u8> {
    let procs = minios_process::manager::list_processes();
    for p in &procs {
        if p.pid.0 == pid {
            let name = core::str::from_utf8(&p.name[..p.name_len]).unwrap_or("?");
            return format!(
                "PID: {}\nName: {}\nState: {}\nPriority: {}\nCPU Time: {} ticks\n",
                p.pid, name, p.state, p.priority.0, p.cpu_time_ticks
            )
            .into_bytes();
        }
    }
    format!("Process {} not found\n", pid).into_bytes()
}

/// Returns metadata for a procfs entry.
pub fn stat_procfs(path: &str) -> Result<FileStat, FsError> {
    match path {
        "/proc/meminfo" | "/proc/uptime" | "/proc/interrupts" | "/proc/scheduler"
        | "/proc/version" | "/proc/trace" | "/proc" => Ok(FileStat {
            size: 0,
            inode_type: InodeType::Special,
            created_at: 0,
            modified_at: 0,
        }),
        _ if path.starts_with("/proc/") && path.ends_with("/status") => Ok(FileStat {
            size: 0,
            inode_type: InodeType::Special,
            created_at: 0,
            modified_at: 0,
        }),
        _ => Err(FsError::NotFound),
    }
}

/// Returns `true` if this path belongs to procfs.
pub fn is_procfs_path(path: &str) -> bool {
    path == "/proc" || path.starts_with("/proc/")
}

fn generate_meminfo() -> Vec<u8> {
    let stats = minios_memory::get_stats();
    format!(
        "MemTotal:  {} frames ({} KiB)\n\
         MemFree:   {} frames ({} KiB)\n\
         HeapUsed:  {} bytes\n\
         HeapFree:  {} bytes\n",
        stats.total_frames,
        stats.total_frames * 4,
        stats.free_frames,
        stats.free_frames * 4,
        stats.heap_used,
        stats.heap_free,
    )
    .into_bytes()
}

fn generate_uptime() -> Vec<u8> {
    let ticks = minios_hal::interrupts::tick_count();
    format!("{} ticks\n", ticks).into_bytes()
}

fn generate_interrupts() -> Vec<u8> {
    let stats = minios_hal::interrupts::interrupt_stats();
    let uptime_secs = stats.timer_count / 100;
    format!(
        "IRQ 0 (Timer):    {} ({}/s)\n\
         IRQ 1 (Keyboard): {}\n",
        stats.timer_count,
        stats.timer_count / uptime_secs.max(1),
        stats.keyboard_count,
    )
    .into_bytes()
}

fn generate_scheduler() -> Vec<u8> {
    let sched = minios_scheduler::SCHEDULER.lock();
    let stats = sched.stats();
    format!(
        "Total switches: {}\n\
         Total ticks: {}\n\
         Idle ticks: {}\n\
         Queue lengths: {:?}\n",
        stats.total_switches, stats.total_ticks, stats.idle_ticks, stats.queue_lengths,
    )
    .into_bytes()
}

fn generate_version() -> Vec<u8> {
    "MiniOS v0.3.0 (x86_64)\n\
     Build: Rust nightly, bootloader_api 0.11\n\
     Shell commands: 25+\n"
        .as_bytes()
        .to_vec()
}

fn generate_trace_stats() -> Vec<u8> {
    use minios_common::traits::trace::Tracer;
    let stats = minios_trace::TRACER.stats();
    format!(
        "Total spans: {}\n\
         Buffer capacity: {}\n\
         Buffer used: {}\n",
        stats.total_spans_written, stats.buffer_capacity, stats.buffer_used,
    )
    .into_bytes()
}
