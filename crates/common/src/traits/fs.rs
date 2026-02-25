//! Filesystem contracts (VFS layer and driver-level).

use crate::error::FsError;
use crate::id::{FileDescriptor, InodeId};
use crate::types::{DirEntry, FileStat, InodeType, OpenFlags, SeekWhence};

/// Virtual filesystem operations visible to userspace.
pub trait FileSystem: Send + Sync {
    fn open(&self, path: &str, flags: OpenFlags) -> Result<FileDescriptor, FsError>;
    fn close(&self, fd: FileDescriptor) -> Result<(), FsError>;
    fn read(&self, fd: FileDescriptor, buf: &mut [u8]) -> Result<usize, FsError>;
    fn write(&self, fd: FileDescriptor, buf: &[u8]) -> Result<usize, FsError>;
    fn seek(&self, fd: FileDescriptor, offset: i64, whence: SeekWhence) -> Result<u64, FsError>;
    fn mkdir(&self, path: &str) -> Result<(), FsError>;
    fn rmdir(&self, path: &str) -> Result<(), FsError>;
    fn unlink(&self, path: &str) -> Result<(), FsError>;
    fn stat(&self, path: &str) -> Result<FileStat, FsError>;
}

/// Low-level filesystem driver operating on inodes.
pub trait FileSystemDriver: Send + Sync {
    fn name(&self) -> &str;
    fn create_file(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    fn create_dir(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    fn read_data(&self, inode: InodeId, offset: usize, buf: &mut [u8]) -> Result<usize, FsError>;
    fn write_data(&self, inode: InodeId, offset: usize, buf: &[u8]) -> Result<usize, FsError>;
    fn lookup(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError>;
    fn remove(&self, parent: InodeId, name: &str) -> Result<(), FsError>;
    fn stat(&self, inode: InodeId) -> Result<FileStat, FsError>;
}
