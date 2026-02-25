//! Bitmap-based physical frame allocator.
//!
//! Tracks 4 KiB physical frames using a fixed-size bitmap large enough
//! for 256 MiB of RAM (65 536 frames = 8 192 bytes).

use bootloader_api::info::{MemoryRegion, MemoryRegionKind};
use minios_common::error::MemoryError;
use minios_common::traits::memory::FrameAllocator;
use spin::Mutex;

/// Size of a single physical frame in bytes.
const FRAME_SIZE: u64 = 4096;

/// Maximum number of frames the bitmap can track (256 MiB / 4 KiB).
const MAX_FRAMES: usize = 65_536;

/// Byte length of the bitmap array.
const BITMAP_BYTES: usize = MAX_FRAMES / 8;

/// Inner state of the bitmap frame allocator, protected by a spinlock.
struct BitmapInner {
    bitmap: [u8; BITMAP_BYTES],
    total_frames: usize,
    used_frames: usize,
}

/// A physical frame allocator backed by a bitmap.
///
/// Each bit in the bitmap represents one 4 KiB frame.
/// A set bit means the frame is **in use**; a clear bit means **free**.
pub struct BitmapFrameAllocator {
    inner: Mutex<BitmapInner>,
}

impl BitmapFrameAllocator {
    /// Creates a new allocator by scanning the bootloader memory map.
    ///
    /// Usable regions are marked as free; everything else is marked as used.
    pub fn new(memory_regions: &[MemoryRegion]) -> Self {
        let mut inner = BitmapInner {
            bitmap: [0xFF; BITMAP_BYTES],
            total_frames: 0,
            used_frames: 0,
        };
        Self::mark_usable_regions(&mut inner, memory_regions);
        Self {
            inner: Mutex::new(inner),
        }
    }

    /// Marks frames corresponding to usable memory regions as free.
    fn mark_usable_regions(inner: &mut BitmapInner, regions: &[MemoryRegion]) {
        let mut total = 0usize;
        let mut free = 0usize;

        for region in regions {
            let start_frame = region.start / FRAME_SIZE;
            let end_frame = region.end / FRAME_SIZE;
            let region_end = (end_frame as usize).min(MAX_FRAMES);
            let region_start = (start_frame as usize).min(MAX_FRAMES);

            if region_start >= MAX_FRAMES {
                continue;
            }

            let count = region_end - region_start;
            total += count;

            if region.kind == MemoryRegionKind::Usable {
                for frame_num in region_start..region_end {
                    clear_bit(&mut inner.bitmap, frame_num);
                }
                free += count;
            }
        }

        inner.total_frames = total;
        inner.used_frames = total - free;
    }
}

impl FrameAllocator for BitmapFrameAllocator {
    fn allocate_frame(&self) -> Result<u64, MemoryError> {
        let mut inner = self.inner.lock();
        for byte_idx in 0..BITMAP_BYTES {
            if inner.bitmap[byte_idx] == 0xFF {
                continue;
            }
            for bit in 0u8..8 {
                let frame_num = byte_idx * 8 + bit as usize;
                if frame_num >= inner.total_frames {
                    return Err(MemoryError::OutOfMemory);
                }
                if !is_bit_set(&inner.bitmap, frame_num) {
                    set_bit(&mut inner.bitmap, frame_num);
                    inner.used_frames += 1;
                    return Ok(frame_num as u64);
                }
            }
        }
        Err(MemoryError::OutOfMemory)
    }

    fn deallocate_frame(&self, frame_number: u64) -> Result<(), MemoryError> {
        let mut inner = self.inner.lock();
        let idx = frame_number as usize;
        if idx >= inner.total_frames {
            return Err(MemoryError::InvalidAddress);
        }
        if !is_bit_set(&inner.bitmap, idx) {
            return Err(MemoryError::NotMapped);
        }
        clear_bit(&mut inner.bitmap, idx);
        inner.used_frames -= 1;
        Ok(())
    }

    fn free_frame_count(&self) -> usize {
        let inner = self.inner.lock();
        inner.total_frames - inner.used_frames
    }

    fn total_frame_count(&self) -> usize {
        self.inner.lock().total_frames
    }
}

/// Returns `true` if the bit at `index` is set (frame in use).
#[inline]
fn is_bit_set(bitmap: &[u8; BITMAP_BYTES], index: usize) -> bool {
    bitmap[index / 8] & (1 << (index % 8)) != 0
}

/// Sets the bit at `index` (marks frame as used).
#[inline]
fn set_bit(bitmap: &mut [u8; BITMAP_BYTES], index: usize) {
    bitmap[index / 8] |= 1 << (index % 8);
}

/// Clears the bit at `index` (marks frame as free).
#[inline]
fn clear_bit(bitmap: &mut [u8; BITMAP_BYTES], index: usize) {
    bitmap[index / 8] &= !(1 << (index % 8));
}
