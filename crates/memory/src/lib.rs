//! Memory management subsystem for MiniOS.
//!
//! Provides physical frame allocation, virtual-to-physical page mapping,
//! and a kernel heap allocator.

#![no_std]

extern crate alloc;

pub mod buddy;
pub mod frame;
pub mod heap;
pub mod paging;

use bootloader_api::BootInfo;
use minios_common::error::MemoryError;
use spin::Mutex;

use frame::BitmapFrameAllocator;
use heap::KernelHeapAllocator;
use paging::PageTableManager;

/// Snapshot of memory statistics for read-only consumption by the shell.
#[derive(Clone)]
pub struct MemoryStats {
    pub free_frames: usize,
    pub total_frames: usize,
    pub heap_used: usize,
    pub heap_free: usize,
}

/// Global memory stats snapshot, populated after init.
pub static MEMORY_STATS: Mutex<Option<MemoryStats>> = Mutex::new(None);

/// Returns a copy of the latest memory stats, or a zeroed snapshot if
/// stats haven't been published yet.
pub fn get_stats() -> MemoryStats {
    MEMORY_STATS.lock().clone().unwrap_or(MemoryStats {
        free_frames: 0,
        total_frames: 0,
        heap_used: 0,
        heap_free: 0,
    })
}

/// Aggregate handle for all memory-management state.
pub struct MemoryManager {
    /// Physical frame allocator.
    pub frame_allocator: BitmapFrameAllocator,
    /// Virtual memory / page-table manager.
    pub page_table_manager: PageTableManager,
    /// Heap statistics handle.
    pub heap: KernelHeapAllocator,
}

impl MemoryManager {
    /// Captures a snapshot of current memory stats into the global static.
    pub fn publish_stats(&self) {
        use minios_common::traits::memory::{FrameAllocator, HeapAllocator};
        *MEMORY_STATS.lock() = Some(MemoryStats {
            free_frames: self.frame_allocator.free_frame_count(),
            total_frames: self.frame_allocator.total_frame_count(),
            heap_used: self.heap.used_bytes(),
            heap_free: self.heap.free_bytes(),
        });
    }
}

/// Initialises the complete memory subsystem.
///
/// 1. Builds a bitmap frame allocator from the bootloader memory map.
/// 2. Creates a page-table mapper using the physical-memory offset.
/// 3. Maps and initialises the kernel heap.
///
/// Returns a [`MemoryManager`] that owns all subsystem state.
pub fn init(boot_info: &'static BootInfo) -> Result<MemoryManager, MemoryError> {
    let phys_offset = boot_info
        .physical_memory_offset
        .as_ref()
        .ok_or(MemoryError::InvalidAddress)?;

    let frame_allocator = BitmapFrameAllocator::new(&boot_info.memory_regions);

    let page_table_manager = unsafe { PageTableManager::new(*phys_offset) };

    heap::init_heap(&page_table_manager, &frame_allocator)?;

    Ok(MemoryManager {
        frame_allocator,
        page_table_manager,
        heap: KernelHeapAllocator,
    })
}
