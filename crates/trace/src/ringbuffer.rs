//! Mutex-protected ring buffer for storing trace spans.
//!
//! The buffer uses a fixed-size array with a modular write index.
//! All access is serialised by the [`TraceEngine`]'s `Mutex`.

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

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::boxed::Box;
    use minios_common::id::{SpanId, TraceId};
    use minios_common::types::SpanStatus;

    fn default_spans<const N: usize>() -> [Span; N] {
        core::array::from_fn(|_| Span::default())
    }

    fn make_span(id: u64) -> Span {
        let mut s = Span::default();
        s.span_id = SpanId(id);
        s.trace_id = TraceId(id);
        s
    }

    #[test]
    fn write_and_read_recent() {
        let mut rb = Box::new(RingBuffer::new());
        rb.write(make_span(1));
        rb.write(make_span(2));
        rb.write(make_span(3));

        let mut out = default_spans::<3>();
        let n = rb.read_recent(3, &mut out);
        assert_eq!(n, 3);
        assert_eq!(out[0].span_id, SpanId(1));
        assert_eq!(out[1].span_id, SpanId(2));
        assert_eq!(out[2].span_id, SpanId(3));
    }

    #[test]
    fn read_recent_fewer_than_available() {
        let mut rb = Box::new(RingBuffer::new());
        rb.write(make_span(10));
        rb.write(make_span(20));
        rb.write(make_span(30));

        let mut out = default_spans::<2>();
        let n = rb.read_recent(2, &mut out);
        assert_eq!(n, 2);
        assert_eq!(out[0].span_id, SpanId(20));
        assert_eq!(out[1].span_id, SpanId(30));
    }

    #[test]
    fn read_recent_more_than_available() {
        let mut rb = Box::new(RingBuffer::new());
        rb.write(make_span(1));

        let mut out = default_spans::<4>();
        let n = rb.read_recent(4, &mut out);
        assert_eq!(n, 1);
        assert_eq!(out[0].span_id, SpanId(1));
    }

    #[test]
    fn wrap_around_overwrites_oldest() {
        let mut rb = Box::new(RingBuffer::new());
        for i in 0..(RING_CAPACITY as u64 + 5) {
            rb.write(make_span(i));
        }

        let mut out = default_spans::<3>();
        let n = rb.read_recent(3, &mut out);
        assert_eq!(n, 3);
        let last_id = RING_CAPACITY as u64 + 4;
        assert_eq!(out[2].span_id, SpanId(last_id));
        assert_eq!(out[1].span_id, SpanId(last_id - 1));
        assert_eq!(out[0].span_id, SpanId(last_id - 2));
    }

    #[test]
    fn clear_resets_counters() {
        let mut rb = Box::new(RingBuffer::new());
        rb.write(make_span(1));
        rb.write(make_span(2));

        let (_, used_before, total_before) = rb.stats();
        assert_eq!(used_before, 2);
        assert_eq!(total_before, 2);

        rb.clear();

        let (cap, used, total) = rb.stats();
        assert_eq!(cap, RING_CAPACITY);
        assert_eq!(used, 0);
        assert_eq!(total, 0);
    }

    #[test]
    fn update_span_finds_and_updates() {
        let mut rb = Box::new(RingBuffer::new());
        rb.write(make_span(10));
        rb.write(make_span(20));
        rb.write(make_span(30));

        let found = rb.update_span(SpanId(20), 9999, SpanStatus::Ok);
        assert!(found);

        let mut out = default_spans::<3>();
        rb.read_recent(3, &mut out);
        let updated = out.iter().find(|s| s.span_id == SpanId(20)).unwrap();
        assert_eq!(updated.end_tsc, 9999);
        assert_eq!(updated.status, SpanStatus::Ok);
    }

    #[test]
    fn update_span_not_found() {
        let mut rb = Box::new(RingBuffer::new());
        rb.write(make_span(1));
        let found = rb.update_span(SpanId(999), 100, SpanStatus::Error);
        assert!(!found);
    }

    #[test]
    fn stats_accuracy() {
        let mut rb = Box::new(RingBuffer::new());
        let (cap, used, total) = rb.stats();
        assert_eq!(cap, RING_CAPACITY);
        assert_eq!(used, 0);
        assert_eq!(total, 0);

        for i in 0..10u64 {
            rb.write(make_span(i));
        }
        let (cap, used, total) = rb.stats();
        assert_eq!(cap, RING_CAPACITY);
        assert_eq!(used, 10);
        assert_eq!(total, 10);
    }

    #[test]
    fn stats_after_wrap() {
        let mut rb = Box::new(RingBuffer::new());
        let count = RING_CAPACITY + 50;
        for i in 0..count as u64 {
            rb.write(make_span(i));
        }
        let (cap, used, total) = rb.stats();
        assert_eq!(cap, RING_CAPACITY);
        assert_eq!(used, RING_CAPACITY);
        assert_eq!(total, count as u64);
    }

    #[test]
    fn empty_read() {
        let rb = Box::new(RingBuffer::new());
        let mut out = default_spans::<2>();
        let n = rb.read_recent(2, &mut out);
        assert_eq!(n, 0);
    }
}
