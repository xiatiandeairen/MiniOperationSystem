//! Page table management using the x86_64 `OffsetPageTable` mapper.
//!
//! Provides helpers to initialise paging, map individual pages, and
//! translate virtual addresses to physical addresses.

use spin::Mutex;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    FrameAllocator as X86FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame,
    Size4KiB, Translate,
};
use x86_64::{PhysAddr, VirtAddr};

use minios_common::error::MemoryError;
use minios_common::traits::memory::VirtualMemoryManager;

use crate::frame::BitmapFrameAllocator;

/// Wrapper around `OffsetPageTable` that satisfies `Send + Sync` via a spinlock.
pub struct PageTableManager {
    mapper: Mutex<OffsetPageTable<'static>>,
    phys_offset: u64,
}

// SAFETY: The inner `OffsetPageTable` is protected by a `Mutex`.
unsafe impl Send for PageTableManager {}
unsafe impl Sync for PageTableManager {}

impl PageTableManager {
    /// Initialises the page table manager from the physical memory offset.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `physical_memory_offset` correctly maps
    /// the complete physical address space.
    pub unsafe fn new(physical_memory_offset: u64) -> Self {
        let mapper = unsafe { init_offset_page_table(physical_memory_offset) };
        Self {
            mapper: Mutex::new(mapper),
            phys_offset: physical_memory_offset,
        }
    }

    /// Returns the physical memory offset used by this mapper.
    pub fn phys_offset(&self) -> u64 {
        self.phys_offset
    }

    /// Maps a virtual page to a physical frame using the given frame allocator.
    pub fn map_page_with_allocator(
        &self,
        page: Page<Size4KiB>,
        frame: PhysFrame<Size4KiB>,
        flags: PageTableFlags,
        allocator: &BitmapFrameAllocator,
    ) -> Result<(), MemoryError> {
        let mut mapper = self.mapper.lock();
        let mut wrapper = FrameAllocWrapper(allocator);
        unsafe {
            mapper
                .map_to(page, frame, flags, &mut wrapper)
                .map_err(|_| MemoryError::AlreadyMapped)?
                .flush();
        }
        Ok(())
    }
}

impl VirtualMemoryManager for PageTableManager {
    fn map_page(
        &self,
        virtual_addr: u64,
        physical_addr: u64,
        writable: bool,
    ) -> Result<(), MemoryError> {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virtual_addr));
        let frame = PhysFrame::containing_address(PhysAddr::new(physical_addr));
        let mut flags = PageTableFlags::PRESENT;
        if writable {
            flags |= PageTableFlags::WRITABLE;
        }
        let mut mapper = self.mapper.lock();
        let mut empty_alloc = EmptyFrameAllocator;
        unsafe {
            mapper
                .map_to(page, frame, flags, &mut empty_alloc)
                .map_err(|_| MemoryError::AlreadyMapped)?
                .flush();
        }
        Ok(())
    }

    fn unmap_page(&self, virtual_addr: u64) -> Result<u64, MemoryError> {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virtual_addr));
        let mut mapper = self.mapper.lock();
        let (frame, flush) = mapper.unmap(page).map_err(|_| MemoryError::NotMapped)?;
        flush.flush();
        Ok(frame.start_address().as_u64())
    }

    fn translate_addr(&self, virtual_addr: u64) -> Option<u64> {
        let mapper = self.mapper.lock();
        mapper
            .translate_addr(VirtAddr::new(virtual_addr))
            .map(|pa| pa.as_u64())
    }
}

/// Creates an `OffsetPageTable` from the active level-4 page table and
/// the given physical memory offset.
///
/// # Safety
///
/// The physical memory must be completely mapped at `physical_memory_offset`.
unsafe fn init_offset_page_table(physical_memory_offset: u64) -> OffsetPageTable<'static> {
    let phys_offset = VirtAddr::new(physical_memory_offset);
    let (level_4_frame, _) = Cr3::read();
    let phys = level_4_frame.start_address();
    let virt = phys_offset + phys.as_u64();
    let table = unsafe { &mut *virt.as_mut_ptr() };
    unsafe { OffsetPageTable::new(table, phys_offset) }
}

/// Adapter that bridges our `BitmapFrameAllocator` to the x86_64 crate's
/// `FrameAllocator<Size4KiB>` trait.
struct FrameAllocWrapper<'a>(&'a BitmapFrameAllocator);

unsafe impl X86FrameAllocator<Size4KiB> for FrameAllocWrapper<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        use minios_common::traits::memory::FrameAllocator;
        let frame_num = self.0.allocate_frame().ok()?;
        let addr = PhysAddr::new(frame_num * 4096);
        Some(PhysFrame::containing_address(addr))
    }
}

/// A dummy allocator that always returns `None`.
///
/// Used by the `VirtualMemoryManager` trait implementation where no
/// extra page-table frames should be allocated (caller provides all frames).
struct EmptyFrameAllocator;

unsafe impl X86FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}
