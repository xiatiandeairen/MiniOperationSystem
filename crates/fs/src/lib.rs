//! MiniOS filesystem subsystem.
//!
//! Provides a Virtual Filesystem (VFS) backed by an in-memory RamFS driver,
//! a global file-descriptor table, and a read-only procfs for `/proc/`.

#![no_std]

extern crate alloc;

pub mod fd;
pub mod procfs;
pub mod ramfs;
pub mod vfs;

pub use vfs::Vfs;

use minios_common::traits::fs::FileSystem;
use minios_common::types::OpenFlags;

/// Initialises the filesystem with default directories and a welcome message.
///
/// Creates: `/dev/`, `/proc/`, `/tmp/`, `/etc/`, and `/etc/motd`.
pub fn init() -> Vfs {
    let vfs = Vfs::new();
    vfs.mkdir("/dev").ok();
    vfs.mkdir("/proc").ok();
    vfs.mkdir("/tmp").ok();
    vfs.mkdir("/etc").ok();

    let fd = vfs
        .open("/etc/motd", OpenFlags::CREATE | OpenFlags::WRITE)
        .expect("failed to create /etc/motd");
    vfs.write(fd, b"Welcome to MiniOS!\n")
        .expect("failed to write /etc/motd");
    vfs.close(fd).expect("failed to close /etc/motd");

    vfs
}
