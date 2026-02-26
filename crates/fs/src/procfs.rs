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
        _ => Err(FsError::NotFound),
    }
}

/// Returns metadata for a procfs entry.
pub fn stat_procfs(path: &str) -> Result<FileStat, FsError> {
    match path {
        "/proc/meminfo" | "/proc/uptime" | "/proc" => Ok(FileStat {
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
