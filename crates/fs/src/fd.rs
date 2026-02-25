//! File descriptor table mapping `FileDescriptor` → `OpenFile`.
//!
//! For now this is a single global table (not per-process).
//! Maximum 256 simultaneously open files.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use minios_common::error::FsError;
use minios_common::id::{FileDescriptor, InodeId};
use minios_common::types::OpenFlags;

const MAX_OPEN_FILES: usize = 256;

/// Metadata for a single open file.
pub struct OpenFile {
    pub inode: InodeId,
    pub offset: usize,
    pub flags: OpenFlags,
    /// If set, reads come from this buffer (used by procfs virtual files).
    pub virtual_data: Option<Vec<u8>>,
}

/// Maps file descriptors to open-file state.
pub struct FdTable {
    table: BTreeMap<i32, OpenFile>,
    next_fd: i32,
}

impl Default for FdTable {
    fn default() -> Self {
        Self::new()
    }
}

impl FdTable {
    /// Creates an empty file descriptor table.
    pub fn new() -> Self {
        Self {
            table: BTreeMap::new(),
            next_fd: 3, // 0/1/2 reserved for stdin/stdout/stderr
        }
    }

    /// Allocates a new FD backed by a real inode.
    pub fn allocate(
        &mut self,
        inode: InodeId,
        flags: OpenFlags,
    ) -> Result<FileDescriptor, FsError> {
        if self.table.len() >= MAX_OPEN_FILES {
            return Err(FsError::TooManyOpenFiles);
        }
        let fd = self.next_fd;
        self.next_fd += 1;
        self.table.insert(
            fd,
            OpenFile {
                inode,
                offset: 0,
                flags,
                virtual_data: None,
            },
        );
        Ok(FileDescriptor(fd))
    }

    /// Allocates a new FD backed by in-memory data (procfs).
    pub fn allocate_virtual(&mut self, data: Vec<u8>) -> Result<FileDescriptor, FsError> {
        if self.table.len() >= MAX_OPEN_FILES {
            return Err(FsError::TooManyOpenFiles);
        }
        let fd = self.next_fd;
        self.next_fd += 1;
        self.table.insert(
            fd,
            OpenFile {
                inode: InodeId(u64::MAX),
                offset: 0,
                flags: OpenFlags::READ,
                virtual_data: Some(data),
            },
        );
        Ok(FileDescriptor(fd))
    }

    /// Returns `(is_virtual, inode, offset)` for the given descriptor.
    pub fn get_info(&self, fd: FileDescriptor) -> Result<(bool, InodeId, usize), FsError> {
        let f = self.table.get(&fd.0).ok_or(FsError::InvalidDescriptor)?;
        Ok((f.virtual_data.is_some(), f.inode, f.offset))
    }

    /// Reads from a virtual file buffer, advancing the offset.
    pub fn read_virtual(&mut self, fd: FileDescriptor, buf: &mut [u8]) -> Result<usize, FsError> {
        let f = self
            .table
            .get_mut(&fd.0)
            .ok_or(FsError::InvalidDescriptor)?;
        let data = f.virtual_data.as_ref().ok_or(FsError::InvalidDescriptor)?;
        if f.offset >= data.len() {
            return Ok(0);
        }
        let available = data.len() - f.offset;
        let to_copy = buf.len().min(available);
        buf[..to_copy].copy_from_slice(&data[f.offset..f.offset + to_copy]);
        f.offset += to_copy;
        Ok(to_copy)
    }

    /// Returns the length of virtual data for a procfs descriptor.
    pub fn virtual_data_len(&self, fd: FileDescriptor) -> Result<usize, FsError> {
        let f = self.table.get(&fd.0).ok_or(FsError::InvalidDescriptor)?;
        f.virtual_data
            .as_ref()
            .map(|d| d.len())
            .ok_or(FsError::InvalidDescriptor)
    }

    /// Advances the read/write offset by `n` bytes.
    pub fn advance_offset(&mut self, fd: FileDescriptor, n: usize) {
        if let Some(f) = self.table.get_mut(&fd.0) {
            f.offset += n;
        }
    }

    /// Sets the offset to an absolute position.
    pub fn set_offset(&mut self, fd: FileDescriptor, offset: usize) -> Result<(), FsError> {
        let f = self
            .table
            .get_mut(&fd.0)
            .ok_or(FsError::InvalidDescriptor)?;
        f.offset = offset;
        Ok(())
    }

    /// Releases a file descriptor.
    pub fn release(&mut self, fd: FileDescriptor) -> Result<(), FsError> {
        self.table.remove(&fd.0).ok_or(FsError::InvalidDescriptor)?;
        Ok(())
    }
}
