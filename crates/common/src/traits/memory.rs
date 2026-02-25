//! Memory management contracts.

use crate::error::MemoryError;

/// Physical frame allocation and deallocation.
pub trait FrameAllocator: Send + Sync {
    fn allocate_frame(&self) -> Result<u64, MemoryError>;
    fn deallocate_frame(&self, frame_number: u64) -> Result<(), MemoryError>;
    fn free_frame_count(&self) -> usize;
    fn total_frame_count(&self) -> usize;
}

/// Virtual address space manipulation.
pub trait VirtualMemoryManager: Send + Sync {
    fn map_page(
        &self,
        virtual_addr: u64,
        physical_addr: u64,
        writable: bool,
    ) -> Result<(), MemoryError>;
    fn unmap_page(&self, virtual_addr: u64) -> Result<u64, MemoryError>;
    fn translate_addr(&self, virtual_addr: u64) -> Option<u64>;
}

/// Heap usage statistics.
pub trait HeapAllocator: Send + Sync {
    fn used_bytes(&self) -> usize;
    fn free_bytes(&self) -> usize;
}
