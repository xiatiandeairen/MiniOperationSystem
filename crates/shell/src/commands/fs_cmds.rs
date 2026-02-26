//! Filesystem shell commands: ls, cat, mkdir, touch, write, pwd.

extern crate alloc;

use minios_common::traits::fs::FileSystem;
use minios_common::types::{InodeType, OpenFlags};
use minios_hal::println;

/// Current working directory (always "/" for now).
static CWD: &str = "/";

/// Lists directory contents.
pub fn cmd_ls(args: &[&str]) {
    let path = if args.is_empty() { CWD } else { args[0] };

    let vfs_guard = minios_fs::VFS.lock();
    let vfs = match vfs_guard.as_ref() {
        Some(v) => v,
        None => {
            println!("ls: filesystem not initialized");
            return;
        }
    };

    match vfs.list_dir(path) {
        Ok(entries) => {
            for (name, inode_type) in &entries {
                let type_char = match inode_type {
                    InodeType::Directory => 'd',
                    InodeType::File => '-',
                    InodeType::CharDevice => 'c',
                    InodeType::Special => 's',
                };
                let full = if path == "/" {
                    alloc::format!("/{}", name)
                } else {
                    alloc::format!("{}/{}", path, name)
                };
                let size = vfs.stat(&full).map(|s| s.size).unwrap_or(0);
                println!("  {} {:>6}  {}", type_char, size, name);
            }
        }
        Err(e) => println!("ls: {}: {}", path, e),
    }
    super::journey::mark(super::journey::STEP_LS);
}

/// Prints file contents.
pub fn cmd_cat(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: cat <file>");
        return;
    }

    let vfs_guard = minios_fs::VFS.lock();
    let vfs = match vfs_guard.as_ref() {
        Some(v) => v,
        None => {
            println!("cat: filesystem not initialized");
            return;
        }
    };

    let path = args[0];
    match vfs.open(path, OpenFlags::READ) {
        Ok(fd) => {
            let mut buf = [0u8; 512];
            loop {
                match vfs.read(fd, &mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(s) = core::str::from_utf8(&buf[..n]) {
                            minios_hal::print!("{}", s);
                        }
                    }
                    Err(e) => {
                        println!("cat: read error: {}", e);
                        break;
                    }
                }
            }
            let _ = vfs.close(fd);
            if path.starts_with("/proc") {
                super::journey::mark(super::journey::STEP_CAT_PROC);
            }
        }
        Err(e) => println!("cat: {}: {}", path, e),
    }
}

/// Creates a directory.
pub fn cmd_mkdir(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: mkdir <path>");
        return;
    }

    let vfs_guard = minios_fs::VFS.lock();
    let vfs = match vfs_guard.as_ref() {
        Some(v) => v,
        None => {
            println!("mkdir: filesystem not initialized");
            return;
        }
    };

    if let Err(e) = vfs.mkdir(args[0]) {
        println!("mkdir: {}: {}", args[0], e);
    }
}

/// Creates an empty file.
pub fn cmd_touch(args: &[&str]) {
    if args.is_empty() {
        println!("Usage: touch <file>");
        return;
    }

    let vfs_guard = minios_fs::VFS.lock();
    let vfs = match vfs_guard.as_ref() {
        Some(v) => v,
        None => {
            println!("touch: filesystem not initialized");
            return;
        }
    };

    let path = args[0];
    match vfs.open(path, OpenFlags::CREATE | OpenFlags::WRITE) {
        Ok(fd) => {
            let _ = vfs.close(fd);
        }
        Err(e) => println!("touch: {}: {}", path, e),
    }
}

/// Writes content to a file.
pub fn cmd_write(args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: write <file> <content...>");
        return;
    }

    let vfs_guard = minios_fs::VFS.lock();
    let vfs = match vfs_guard.as_ref() {
        Some(v) => v,
        None => {
            println!("write: filesystem not initialized");
            return;
        }
    };

    let path = args[0];
    match vfs.open(path, OpenFlags::CREATE | OpenFlags::WRITE) {
        Ok(fd) => {
            let mut first = true;
            for arg in &args[1..] {
                if !first {
                    let _ = vfs.write(fd, b" ");
                }
                let _ = vfs.write(fd, arg.as_bytes());
                first = false;
            }
            let _ = vfs.close(fd);
        }
        Err(e) => println!("write: {}: {}", path, e),
    }
}

/// Prints the current working directory.
pub fn cmd_pwd(_args: &[&str]) {
    println!("{}", CWD);
}
