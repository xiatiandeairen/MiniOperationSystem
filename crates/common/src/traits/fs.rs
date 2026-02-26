//! Filesystem contracts (VFS layer and driver-level).

use crate::error::FsError;
use crate::id::{FileDescriptor, InodeId};
use crate::types::{FileStat, OpenFlags, SeekWhence};

/// Virtual filesystem operations visible to userspace.
pub trait FileSystem: Send + Sync {
    /// Opens a file or directory at `path` with the given flags.
    fn open(&self, path: &str, flags: OpenFlags) -> Result<FileDescriptor, FsError>;
    /// Closes an open file descriptor.
    fn close(&self, fd: FileDescriptor) -> Result<(), FsError>;
    /// Reads bytes from an open file descriptor into `buf`.
    fn read(&self, fd: FileDescriptor, buf: &mut [u8]) -> Result<usize, FsError>;
    /// Writes bytes from `buf` to an open file descriptor.
    fn write(&self, fd: FileDescriptor, buf: &[u8]) -> Result<usize, FsError>;
    /// Repositions the file offset for an open descriptor.
    fn seek(&self, fd: FileDescriptor, offset: i64, whence: SeekWhence) -> Result<u64, FsError>;
    /// Creates a directory at `path`.
    fn mkdir(&self, path: &str) -> Result<(), FsError>;
    /// Removes an empty directory at `path`.
    fn rmdir(&self, path: &str) -> Result<(), FsError>;
    /// Removes a file at `path`.
    fn unlink(&self, path: &str) -> Result<(), FsError>;
    /// Returns metadata for the file or directory at `path`.
    fn stat(&self, path: &str) -> Result<FileStat, FsError>;
}

/// Low-level filesystem driver operating on inodes.
pub trait FileSystemDriver: Send + Sync {
    /// Returns the filesystem driver name.
    fn name(&self) -> &str;
    /// Creates a regular file under `parent`.
    fn create_file(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    /// Creates a directory under `parent`.
    fn create_dir(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    /// Reads data from an inode starting at `offset`.
    fn read_data(&self, inode: InodeId, offset: usize, buf: &mut [u8]) -> Result<usize, FsError>;
    /// Writes data to an inode starting at `offset`.
    fn write_data(&self, inode: InodeId, offset: usize, buf: &[u8]) -> Result<usize, FsError>;
    /// Looks up a child entry by name under `parent`.
    fn lookup(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    /// Removes a child entry by name from `parent`.
    fn remove(&self, parent: InodeId, name: &str) -> Result<(), FsError>;
    /// Returns metadata for the given inode.
    fn stat(&self, inode: InodeId) -> Result<FileStat, FsError>;
}
