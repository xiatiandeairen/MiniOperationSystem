//! Buddy System allocator — a power-of-2 block splitting allocator.
//!
//! Educational implementation for comparison with the bitmap allocator.
//! Not integrated into the kernel; use via `lab allocator-compare`.

extern crate alloc;

/// Simulated Buddy allocator for educational comparison.
pub struct BuddyAllocator {
    max_order: usize,
    free_lists: [alloc::vec::Vec<u64>; 16],
    total_frames: usize,
    allocated: usize,
}

impl BuddyAllocator {
    /// Creates a new buddy allocator managing `total_frames` frames.
    pub fn new(total_frames: usize) -> Self {
        let max_order = (total_frames as f64).log2() as usize;
        let max_order = max_order.min(15);
        let mut alloc = Self {
            max_order,
            free_lists: Default::default(),
            total_frames,
            allocated: 0,
        };
        alloc.free_lists[max_order].push(0);
        alloc
    }

    /// Allocates a block of 2^order frames.
    pub fn allocate(&mut self, order: usize) -> Option<u64> {
        if order > self.max_order {
            return None;
        }
        for o in order..=self.max_order {
            if let Some(block) = self.free_lists[o].pop() {
                let mut current_order = o;
                let mut addr = block;
                while current_order > order {
                    current_order -= 1;
                    let buddy = addr + (1u64 << current_order);
                    self.free_lists[current_order].push(buddy);
                }
                self.allocated += 1 << order;
                return Some(addr);
            }
        }
        None
    }

    /// Returns the number of allocated frames.
    pub fn allocated_frames(&self) -> usize {
        self.allocated
    }

    /// Returns the number of free frames.
    pub fn free_frames(&self) -> usize {
        self.total_frames - self.allocated
    }
}
