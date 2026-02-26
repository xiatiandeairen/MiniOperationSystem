//! Memory management contracts.

use crate::error::MemoryError;

/// Physical frame allocation and deallocation.
pub trait FrameAllocator: Send + Sync {
    /// Allocates a single physical frame, returning its frame number.
    fn allocate_frame(&self) -> Result<u64, MemoryError>;
    /// Frees a previously allocated frame.
    fn deallocate_frame(&self, frame_number: u64) -> Result<(), MemoryError>;
    /// Returns the number of free frames.
    fn free_frame_count(&self) -> usize;
    /// Returns the total number of frames.
    fn total_frame_count(&self) -> usize;
}

/// Virtual address space manipulation.
pub trait VirtualMemoryManager: Send + Sync {
    /// Maps a virtual address to a physical address.
    fn map_page(
        &self,
        virtual_addr: u64,
        physical_addr: u64,
        writable: bool,
    ) -> Result<(), MemoryError>;
    /// Unmaps a virtual page, returning its physical address.
    fn unmap_page(&self, virtual_addr: u64) -> Result<u64, MemoryError>;
    /// Translates a virtual address to its physical address.
    fn translate_addr(&self, virtual_addr: u64) -> Option<u64>;
}

/// Heap usage statistics.
pub trait HeapAllocator: Send + Sync {
    /// Returns the number of heap bytes currently in use.
    fn used_bytes(&self) -> usize;
    /// Returns the number of free heap bytes.
    fn free_bytes(&self) -> usize;
}
