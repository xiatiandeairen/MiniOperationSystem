//! Virtual Filesystem layer.
//!
//! Implements the [`FileSystem`] trait by delegating to a [`RamFs`] driver
//! and handling `/proc/` paths through the procfs module.

use crate::fd::FdTable;
use crate::procfs;
use crate::ramfs::RamFs;

use minios_common::error::FsError;
use minios_common::id::{FileDescriptor, InodeId};
use minios_common::traits::fs::{FileSystem, FileSystemDriver};
use minios_common::types::{FileStat, OpenFlags, SeekWhence};
use spin::Mutex;

/// The virtual filesystem, combining RamFS with a global FD table.
pub struct Vfs {
    driver: RamFs,
    fd_table: Mutex<FdTable>,
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}

impl Vfs {
    /// Creates a new VFS backed by a fresh RamFS instance.
    pub fn new() -> Self {
        Self {
            driver: RamFs::new(),
            fd_table: Mutex::new(FdTable::new()),
        }
    }

    /// Resolves an absolute path to an inode by walking the tree from root.
    fn resolve_path(&self, path: &str) -> Result<InodeId, FsError> {
        if !path.starts_with('/') {
            return Err(FsError::InvalidPath);
        }
        let mut current = InodeId(0);
        for component in path.split('/').filter(|s| !s.is_empty()) {
            current = self.driver.lookup(current, component)?;
        }
        Ok(current)
    }

    /// Splits `path` into (parent inode, child name).
    fn resolve_parent<'a>(&self, path: &'a str) -> Result<(InodeId, &'a str), FsError> {
        if !path.starts_with('/') {
            return Err(FsError::InvalidPath);
        }
        let trimmed = path.trim_end_matches('/');
        let last_slash = trimmed.rfind('/').ok_or(FsError::InvalidPath)?;
        let name = &trimmed[last_slash + 1..];
        if name.is_empty() {
            return Err(FsError::InvalidPath);
        }
        let parent_path = if last_slash == 0 {
            "/"
        } else {
            &trimmed[..last_slash]
        };
        let parent_id = self.resolve_path(parent_path)?;
        Ok((parent_id, name))
    }

    /// Opens a procfs virtual file by generating its content into an FD.
    fn open_procfs(&self, path: &str) -> Result<FileDescriptor, FsError> {
        let content = procfs::read_procfs(path)?;
        self.fd_table.lock().allocate_virtual(content)
    }

    /// Reads from a regular (non-virtual) file through the driver.
    fn read_regular(
        &self,
        fd: FileDescriptor,
        inode: InodeId,
        offset: usize,
        buf: &mut [u8],
    ) -> Result<usize, FsError> {
        let n = self.driver.read_data(inode, offset, buf)?;
        self.fd_table.lock().advance_offset(fd, n);
        Ok(n)
    }

    /// Writes to a regular (non-virtual) file through the driver.
    fn write_regular(
        &self,
        fd: FileDescriptor,
        inode: InodeId,
        offset: usize,
        buf: &[u8],
    ) -> Result<usize, FsError> {
        let n = self.driver.write_data(inode, offset, buf)?;
        self.fd_table.lock().advance_offset(fd, n);
        Ok(n)
    }

    /// Computes a new absolute seek offset.
    fn compute_seek_offset(
        current: usize,
        size: usize,
        offset: i64,
        whence: SeekWhence,
    ) -> Result<usize, FsError> {
        let base = match whence {
            SeekWhence::Start => 0usize,
            SeekWhence::Current => current,
            SeekWhence::End => size,
        };
        if offset >= 0 {
            Ok(base + offset as usize)
        } else {
            base.checked_sub((-offset) as usize)
                .ok_or(FsError::InvalidPath)
        }
    }
}

impl FileSystem for Vfs {
    fn open(&self, path: &str, flags: OpenFlags) -> Result<FileDescriptor, FsError> {
        if path.starts_with("/proc/") {
            return self.open_procfs(path);
        }
        let inode = match self.resolve_path(path) {
            Ok(id) => id,
            Err(FsError::NotFound) if flags.contains(OpenFlags::CREATE) => {
                let (parent, name) = self.resolve_parent(path)?;
                self.driver.create_file(parent, name)?
            }
            Err(e) => return Err(e),
        };
        self.fd_table.lock().allocate(inode, flags)
    }

    fn close(&self, fd: FileDescriptor) -> Result<(), FsError> {
        self.fd_table.lock().release(fd)
    }

    fn read(&self, fd: FileDescriptor, buf: &mut [u8]) -> Result<usize, FsError> {
        let (is_virtual, inode, offset) = self.fd_table.lock().get_info(fd)?;
        if is_virtual {
            return self.fd_table.lock().read_virtual(fd, buf);
        }
        self.read_regular(fd, inode, offset, buf)
    }

    fn write(&self, fd: FileDescriptor, buf: &[u8]) -> Result<usize, FsError> {
        let (is_virtual, inode, offset) = self.fd_table.lock().get_info(fd)?;
        if is_virtual {
            return Err(FsError::PermissionDenied);
        }
        self.write_regular(fd, inode, offset, buf)
    }

    fn seek(&self, fd: FileDescriptor, offset: i64, whence: SeekWhence) -> Result<u64, FsError> {
        let (is_virtual, inode, current) = self.fd_table.lock().get_info(fd)?;
        let size = if is_virtual {
            self.fd_table.lock().virtual_data_len(fd)?
        } else {
            self.driver.stat(inode)?.size
        };
        let new_offset = Self::compute_seek_offset(current, size, offset, whence)?;
        self.fd_table.lock().set_offset(fd, new_offset)?;
        Ok(new_offset as u64)
    }

    fn mkdir(&self, path: &str) -> Result<(), FsError> {
        let (parent, name) = self.resolve_parent(path)?;
        self.driver.create_dir(parent, name)?;
        Ok(())
    }

    fn rmdir(&self, path: &str) -> Result<(), FsError> {
        let (parent, name) = self.resolve_parent(path)?;
        self.driver.remove(parent, name)
    }

    fn unlink(&self, path: &str) -> Result<(), FsError> {
        let (parent, name) = self.resolve_parent(path)?;
        self.driver.remove(parent, name)
    }

    fn stat(&self, path: &str) -> Result<FileStat, FsError> {
        if path.starts_with("/proc/") {
            return procfs::stat_procfs(path);
        }
        let inode = self.resolve_path(path)?;
        self.driver.stat(inode)
    }
}
