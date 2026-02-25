//! Lock-free ring buffer for storing trace spans.
//!
//! Uses atomic operations for the write index so that concurrent writers
//! can append spans without taking a lock. Readers copy out recent entries.

use crate::span::Span;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Number of span slots in the ring buffer.
pub const RING_CAPACITY: usize = 4096;

/// Fixed-size circular buffer of [`Span`] records.
///
/// Writers atomically claim the next slot via `write_head` and copy the
/// span in. Readers snapshot recent entries via [`read_recent`].
pub struct RingBuffer {
    /// Storage for span records.
    slots: [Span; RING_CAPACITY],
    /// Monotonically increasing write index (wraps via modulo).
    write_head: AtomicUsize,
    /// Total number of spans ever written.
    total_written: AtomicU64,
}

impl Default for RingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl RingBuffer {
    /// Creates a new empty ring buffer.
    pub const fn new() -> Self {
        const DEFAULT_SPAN: Span = Span {
            trace_id: minios_common::id::TraceId(0),
            span_id: minios_common::id::SpanId(0),
            parent_span_id: None,
            name: [0u8; 64],
            name_len: 0,
            module: [0u8; 32],
            module_len: 0,
            start_tsc: 0,
            end_tsc: 0,
            status: minios_common::types::SpanStatus::InProgress,
            pid: 0,
            depth: 0,
        };

        Self {
            slots: [DEFAULT_SPAN; RING_CAPACITY],
            write_head: AtomicUsize::new(0),
            total_written: AtomicU64::new(0),
        }
    }

    /// Appends a span to the buffer, overwriting the oldest entry if full.
    pub fn write(&mut self, span: Span) {
        let idx = self.write_head.load(Ordering::Relaxed) % RING_CAPACITY;
        self.slots[idx] = span;
        self.write_head.fetch_add(1, Ordering::Release);
        self.total_written.fetch_add(1, Ordering::Relaxed);
    }

    /// Copies up to `count` of the most recent spans into `out`.
    ///
    /// Returns the number of spans actually copied.
    pub fn read_recent(&self, count: usize, out: &mut [Span]) -> usize {
        let head = self.write_head.load(Ordering::Acquire);
        let available = head.min(RING_CAPACITY);
        let n = count.min(available).min(out.len());

        for (i, slot) in out.iter_mut().enumerate().take(n) {
            let idx = (head - n + i) % RING_CAPACITY;
            *slot = self.slots[idx].clone();
        }
        n
    }

    /// Resets the buffer, discarding all recorded spans.
    pub fn clear(&mut self) {
        self.write_head.store(0, Ordering::Release);
        self.total_written.store(0, Ordering::Release);
    }

    /// Returns `(capacity, used, total_written)`.
    pub fn stats(&self) -> (usize, usize, u64) {
        let head = self.write_head.load(Ordering::Relaxed);
        let used = head.min(RING_CAPACITY);
        let total = self.total_written.load(Ordering::Relaxed);
        (RING_CAPACITY, used, total)
    }

    /// Searches recent entries for a span matching `span_id` and updates it.
    ///
    /// Returns `true` if the span was found and updated.
    pub fn update_span(
        &mut self,
        span_id: minios_common::id::SpanId,
        end_tsc: u64,
        status: minios_common::types::SpanStatus,
    ) -> bool {
        let head = self.write_head.load(Ordering::Relaxed);
        let available = head.min(RING_CAPACITY);

        for i in 0..available {
            let idx = (head - 1 - i) % RING_CAPACITY;
            if self.slots[idx].span_id == span_id {
                self.slots[idx].end_tsc = end_tsc;
                self.slots[idx].status = status;
                return true;
            }
        }
        false
    }
}
