//! ProcFS — read-only virtual filesystem generating content on the fly.
//!
//! Supported entries:
//! - `/proc/meminfo` — placeholder memory statistics
//! - `/proc/uptime`  — placeholder uptime tick count

extern crate alloc;

use alloc::vec::Vec;

use minios_common::error::FsError;
use minios_common::types::{FileStat, InodeType};

/// Generates the content for a procfs virtual file.
pub fn read_procfs(path: &str) -> Result<Vec<u8>, FsError> {
    match path {
        "/proc/meminfo" => Ok(Vec::from(*b"MemTotal: unknown\nMemFree: unknown\n")),
        "/proc/uptime" => Ok(Vec::from(*b"0\n")),
        _ => Err(FsError::NotFound),
    }
}

/// Returns metadata for a procfs virtual file.
pub fn stat_procfs(path: &str) -> Result<FileStat, FsError> {
    match path {
        "/proc/meminfo" | "/proc/uptime" => Ok(FileStat {
            size: 0,
            inode_type: InodeType::Special,
            created_at: 0,
            modified_at: 0,
        }),
        _ => Err(FsError::NotFound),
    }
}
