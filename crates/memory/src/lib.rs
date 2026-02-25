//! Memory management subsystem for MiniOS.
//!
//! Provides physical frame allocation, virtual-to-physical page mapping,
//! and a kernel heap allocator.

#![no_std]

extern crate alloc;

pub mod frame;
pub mod heap;
pub mod paging;

use bootloader_api::BootInfo;
use minios_common::error::MemoryError;

use frame::BitmapFrameAllocator;
use heap::KernelHeapAllocator;
use paging::PageTableManager;

/// Aggregate handle for all memory-management state.
pub struct MemoryManager {
    /// Physical frame allocator.
    pub frame_allocator: BitmapFrameAllocator,
    /// Virtual memory / page-table manager.
    pub page_table_manager: PageTableManager,
    /// Heap statistics handle.
    pub heap: KernelHeapAllocator,
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
        .into_option()
        .ok_or(MemoryError::InvalidAddress)?;

    let frame_allocator = BitmapFrameAllocator::new(&boot_info.memory_regions);

    let page_table_manager = unsafe { PageTableManager::new(phys_offset) };

    heap::init_heap(&page_table_manager, &frame_allocator)?;

    Ok(MemoryManager {
        frame_allocator,
        page_table_manager,
        heap: KernelHeapAllocator,
    })
}
