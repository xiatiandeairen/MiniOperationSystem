//! In-memory filesystem driver (RamFS).
//!
//! Inode-based tree structure backed by a `BTreeMap` protected with a
//! spin mutex.  Root inode is always `InodeId(0)`.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use minios_common::error::FsError;
use minios_common::id::InodeId;
use minios_common::traits::fs::FileSystemDriver;
use minios_common::types::{FileStat, InodeType};
use spin::Mutex;

/// A single inode stored in the RAM filesystem.
#[allow(dead_code)]
struct RamFsInode {
    id: InodeId,
    inode_type: InodeType,
    name: String,
    data: Vec<u8>,
    children: Vec<InodeId>,
    parent: Option<InodeId>,
    size: usize,
    created_at: u64,
    modified_at: u64,
}

/// In-memory filesystem backed by a `BTreeMap` of inodes.
pub struct RamFs {
    inodes: Mutex<BTreeMap<InodeId, RamFsInode>>,
    next_inode: AtomicU64,
}

impl Default for RamFs {
    fn default() -> Self {
        Self::new()
    }
}

impl RamFs {
    /// Creates a new RamFS with a root directory at `InodeId(0)`.
    pub fn new() -> Self {
        let mut inodes = BTreeMap::new();
        let root = RamFsInode {
            id: InodeId(0),
            inode_type: InodeType::Directory,
            name: String::from("/"),
            data: Vec::new(),
            children: Vec::new(),
            parent: None,
            size: 0,
            created_at: 0,
            modified_at: 0,
        };
        inodes.insert(InodeId(0), root);
        Self {
            inodes: Mutex::new(inodes),
            next_inode: AtomicU64::new(1),
        }
    }

    /// Allocates a fresh inode ID.
    fn alloc_inode_id(&self) -> InodeId {
        InodeId(self.next_inode.fetch_add(1, Ordering::Relaxed))
    }

    /// Shared logic for creating both files and directories.
    fn create_inode(
        &self,
        parent: InodeId,
        name: &str,
        inode_type: InodeType,
    ) -> Result<InodeId, FsError> {
        let id = self.alloc_inode_id();
        let mut inodes = self.inodes.lock();
        validate_parent(&inodes, parent, name)?;
        let inode = RamFsInode {
            id,
            inode_type,
            name: String::from(name),
            data: Vec::new(),
            children: Vec::new(),
            parent: Some(parent),
            size: 0,
            created_at: 0,
            modified_at: 0,
        };
        inodes.insert(id, inode);
        inodes.get_mut(&parent).unwrap().children.push(id);
        Ok(id)
    }
}

/// Validates that `parent` exists, is a directory, and has no child named `name`.
fn validate_parent(
    inodes: &BTreeMap<InodeId, RamFsInode>,
    parent: InodeId,
    name: &str,
) -> Result<(), FsError> {
    let parent_inode = inodes.get(&parent).ok_or(FsError::NotFound)?;
    if parent_inode.inode_type != InodeType::Directory {
        return Err(FsError::NotADirectory);
    }
    for &child_id in &parent_inode.children {
        if let Some(child) = inodes.get(&child_id) {
            if child.name == name {
                return Err(FsError::AlreadyExists);
            }
        }
    }
    Ok(())
}

impl RamFs {
    /// Returns the names and types of all children of a directory inode.
    pub fn list_dir(&self, inode: InodeId) -> Result<Vec<(String, InodeType)>, FsError> {
        let inodes = self.inodes.lock();
        let node = inodes.get(&inode).ok_or(FsError::NotFound)?;
        if node.inode_type != InodeType::Directory {
            return Err(FsError::NotADirectory);
        }
        let mut entries = Vec::new();
        for &child_id in &node.children {
            if let Some(child) = inodes.get(&child_id) {
                entries.push((child.name.clone(), child.inode_type));
            }
        }
        Ok(entries)
    }
}

impl FileSystemDriver for RamFs {
    fn name(&self) -> &str {
        "ramfs"
    }

    fn create_file(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError> {
        self.create_inode(parent, name, InodeType::File)
    }

    fn create_dir(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError> {
        self.create_inode(parent, name, InodeType::Directory)
    }

    fn read_data(&self, inode: InodeId, offset: usize, buf: &mut [u8]) -> Result<usize, FsError> {
        let inodes = self.inodes.lock();
        let node = inodes.get(&inode).ok_or(FsError::NotFound)?;
        if node.inode_type != InodeType::File {
            return Err(FsError::NotAFile);
        }
        if offset >= node.data.len() {
            return Ok(0);
        }
        let available = node.data.len() - offset;
        let to_copy = buf.len().min(available);
        buf[..to_copy].copy_from_slice(&node.data[offset..offset + to_copy]);
        Ok(to_copy)
    }

    fn write_data(&self, inode: InodeId, offset: usize, buf: &[u8]) -> Result<usize, FsError> {
        let mut inodes = self.inodes.lock();
        let node = inodes.get_mut(&inode).ok_or(FsError::NotFound)?;
        if node.inode_type != InodeType::File {
            return Err(FsError::NotAFile);
        }
        let end = offset + buf.len();
        if end > node.data.len() {
            node.data.resize(end, 0);
        }
        node.data[offset..end].copy_from_slice(buf);
        node.size = node.data.len();
        Ok(buf.len())
    }

    fn lookup(&self, parent: InodeId, name: &str) -> Result<InodeId, FsError> {
        let inodes = self.inodes.lock();
        let parent_inode = inodes.get(&parent).ok_or(FsError::NotFound)?;
        if parent_inode.inode_type != InodeType::Directory {
            return Err(FsError::NotADirectory);
        }
        for &child_id in &parent_inode.children {
            if let Some(child) = inodes.get(&child_id) {
                if child.name == name {
                    return Ok(child_id);
                }
            }
        }
        Err(FsError::NotFound)
    }

    fn remove(&self, parent: InodeId, name: &str) -> Result<(), FsError> {
        let mut inodes = self.inodes.lock();
        let child_id = find_child(&inodes, parent, name)?;
        inodes
            .get_mut(&parent)
            .unwrap()
            .children
            .retain(|&id| id != child_id);
        inodes.remove(&child_id);
        Ok(())
    }

    fn stat(&self, inode: InodeId) -> Result<FileStat, FsError> {
        let inodes = self.inodes.lock();
        let node = inodes.get(&inode).ok_or(FsError::NotFound)?;
        Ok(FileStat {
            size: node.size,
            inode_type: node.inode_type,
            created_at: node.created_at,
            modified_at: node.modified_at,
        })
    }
}

/// Finds a child inode by name within a parent directory.
fn find_child(
    inodes: &BTreeMap<InodeId, RamFsInode>,
    parent: InodeId,
    name: &str,
) -> Result<InodeId, FsError> {
    let parent_inode = inodes.get(&parent).ok_or(FsError::NotFound)?;
    for &child_id in &parent_inode.children {
        if let Some(child) = inodes.get(&child_id) {
            if child.name == name {
                return Ok(child_id);
            }
        }
    }
    Err(FsError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use minios_common::id::InodeId;
    use minios_common::types::InodeType;

    const ROOT: InodeId = InodeId(0);

    #[test]
    fn create_file_in_root() {
        let fs = RamFs::new();
        let id = fs.create_file(ROOT, "foo").unwrap();
        assert!(id.0 > 0);
        let found = fs.lookup(ROOT, "foo").unwrap();
        assert_eq!(found, id);
    }

    #[test]
    fn create_dir() {
        let fs = RamFs::new();
        let id = fs.create_dir(ROOT, "mydir").unwrap();
        let stat = fs.stat(id).unwrap();
        assert_eq!(stat.inode_type, InodeType::Directory);
    }

    #[test]
    fn write_and_read_data() {
        let fs = RamFs::new();
        let id = fs.create_file(ROOT, "f").unwrap();
        let data = b"hello world";
        fs.write_data(id, 0, data).unwrap();
        let mut buf = [0u8; 32];
        let n = fs.read_data(id, 0, &mut buf).unwrap();
        assert_eq!(n, 11);
        assert_eq!(&buf[..11], data);
    }

    #[test]
    fn write_extends_data() {
        let fs = RamFs::new();
        let id = fs.create_file(ROOT, "f").unwrap();
        fs.write_data(id, 0, b"ab").unwrap();
        fs.write_data(id, 5, b"X").unwrap();
        let mut buf = [0u8; 16];
        let n = fs.read_data(id, 0, &mut buf).unwrap();
        assert_eq!(n, 6);
        assert_eq!(&buf[..6], b"ab\0\0\0X");
    }

    #[test]
    fn lookup_not_found() {
        let fs = RamFs::new();
        let r = fs.lookup(ROOT, "nonexistent");
        assert!(matches!(r, Err(FsError::NotFound)));
    }

    #[test]
    fn remove_file() {
        let fs = RamFs::new();
        fs.create_file(ROOT, "f").unwrap();
        fs.remove(ROOT, "f").unwrap();
        let r = fs.lookup(ROOT, "f");
        assert!(matches!(r, Err(FsError::NotFound)));
    }

    #[test]
    fn create_duplicate_fails() {
        let fs = RamFs::new();
        fs.create_file(ROOT, "dup").unwrap();
        let r = fs.create_file(ROOT, "dup");
        assert!(matches!(r, Err(FsError::AlreadyExists)));
    }

    #[test]
    fn list_dir_entries() {
        let fs = RamFs::new();
        fs.create_file(ROOT, "a").unwrap();
        fs.create_file(ROOT, "b").unwrap();
        fs.create_dir(ROOT, "c").unwrap();
        let entries = fs.list_dir(ROOT).unwrap();
        assert_eq!(entries.len(), 3);
        let mut found = [false, false, false];
        for (name, _) in &entries {
            match name.as_str() {
                "a" => found[0] = true,
                "b" => found[1] = true,
                "c" => found[2] = true,
                _ => {}
            }
        }
        assert!(found[0] && found[1] && found[2]);
    }
}
