//! System call interface for MiniOS.
//!
//! Provides a dispatcher that maps syscall numbers to handler functions.
//! For now, syscalls are direct function calls from kernel space — no
//! `int 0x80` trap mechanism is involved yet.

#![no_std]

extern crate alloc;

pub mod dispatcher;
pub mod info;
pub mod io;
pub mod process;

// Syscall numbers
/// Read from a file descriptor.
pub const SYS_READ: u64 = 0;
/// Write to a file descriptor.
pub const SYS_WRITE: u64 = 1;
/// Terminate the calling process.
pub const SYS_EXIT: u64 = 12;
/// Return the current process ID.
pub const SYS_GETPID: u64 = 14;
/// Yield the CPU to another task.
pub const SYS_YIELD: u64 = 15;
/// Return system uptime in ticks.
pub const SYS_UPTIME: u64 = 50;
/// Write memory info into a user buffer.
pub const SYS_MEMINFO: u64 = 51;

/// Dispatches a system call by number.
///
/// Returns the syscall result, or `-1` for unknown syscall numbers.
pub fn dispatch(num: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    dispatcher::dispatch(num, arg1, arg2, arg3)
}
