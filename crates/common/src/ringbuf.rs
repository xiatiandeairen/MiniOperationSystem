//! Generic fixed-size ring buffer for `#![no_std]` environments.

use core::sync::atomic::{AtomicUsize, Ordering};

/// A fixed-capacity ring buffer that overwrites oldest entries when full.
pub struct RingBuf<T, const N: usize> {
    slots: [T; N],
    head: AtomicUsize,
    count: AtomicUsize,
}

impl<T: Clone + Default, const N: usize> RingBuf<T, N> {
    /// Pushes an item, overwriting the oldest if full.
    pub fn push(&mut self, item: T) {
        let idx = self.head.load(Ordering::Relaxed) % N;
        self.slots[idx] = item;
        self.head.fetch_add(1, Ordering::Release);
        let count = self.count.load(Ordering::Relaxed);
        if count < N {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Returns the number of items currently stored.
    pub fn len(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reads up to `count` most recent items into `out`.
    pub fn read_recent(&self, count: usize, out: &mut [T]) -> usize {
        let stored = self.len();
        let head = self.head.load(Ordering::Acquire);
        let n = count.min(stored).min(out.len());
        for (i, slot) in out.iter_mut().enumerate().take(n) {
            let idx = (head + N - n + i) % N;
            *slot = self.slots[idx].clone();
        }
        n
    }
}
