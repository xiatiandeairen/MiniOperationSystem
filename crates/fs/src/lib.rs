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
use spin::Mutex;

/// Global VFS instance, set during [`init`] and accessible by all subsystems.
pub static VFS: Mutex<Option<Vfs>> = Mutex::new(None);

/// Initialises the filesystem with default directories and a welcome message.
///
/// Creates: `/dev/`, `/proc/`, `/tmp/`, `/etc/`, and `/etc/motd`.
/// The VFS is also stored in the global [`VFS`] static for shell access.
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

    let init_fd = vfs
        .open("/etc/init.sh", OpenFlags::CREATE | OpenFlags::WRITE)
        .expect("failed to create /etc/init.sh");
    vfs.write(
        init_fd,
        b"# MiniOS init script\necho MiniOS initialized successfully.\n",
    )
    .expect("failed to write /etc/init.sh");
    vfs.close(init_fd).expect("failed to close /etc/init.sh");

    vfs
}

/// Stores a VFS instance into the global static for use by the shell
/// and other subsystems that need filesystem access.
pub fn set_global_vfs(vfs: Vfs) {
    *VFS.lock() = Some(vfs);
}
