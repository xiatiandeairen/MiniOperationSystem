//! Kernel heap allocator backed by `linked_list_allocator::LockedHeap`.
//!
//! The heap is placed at a fixed virtual address range and mapped to
//! physical frames on demand during initialisation.

extern crate alloc;

use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

use minios_common::error::MemoryError;
use minios_common::traits::memory::{FrameAllocator, HeapAllocator};

use crate::frame::BitmapFrameAllocator;
use crate::paging::PageTableManager;

/// Virtual base address of the kernel heap.
pub const HEAP_START: u64 = 0x4444_4444_0000;

/// Initial heap size (1 MiB).
pub const HEAP_SIZE: usize = 1024 * 1024;

/// Global kernel heap allocator instance.
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Wrapper providing [`HeapAllocator`] trait stats for the global heap.
pub struct KernelHeapAllocator;

impl HeapAllocator for KernelHeapAllocator {
    fn used_bytes(&self) -> usize {
        ALLOCATOR.lock().used()
    }

    fn free_bytes(&self) -> usize {
        ALLOCATOR.lock().free()
    }
}

/// Maps the heap virtual pages to physical frames and initialises the allocator.
///
/// This must be called exactly once during boot, after the page table mapper
/// and frame allocator are ready.
pub fn init_heap(
    page_table_mgr: &PageTableManager,
    frame_allocator: &BitmapFrameAllocator,
) -> Result<(), MemoryError> {
    let heap_start = VirtAddr::new(HEAP_START);
    let heap_end = heap_start + HEAP_SIZE as u64;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let mut addr = heap_start;
    while addr < heap_end {
        let page = Page::<Size4KiB>::containing_address(addr);
        let frame_num = FrameAllocator::allocate_frame(frame_allocator)?;
        let frame = PhysFrame::containing_address(PhysAddr::new(frame_num * 4096));
        page_table_mgr.map_page_with_allocator(page, frame, flags, frame_allocator)?;
        addr += 4096u64;
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    minios_hal::klog!(
        Info,
        "memory",
        "heap initialized: {} bytes at {:#x}",
        HEAP_SIZE,
        HEAP_START
    );

    Ok(())
}
