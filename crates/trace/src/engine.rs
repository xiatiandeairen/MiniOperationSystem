//! Core trace engine implementing the [`Tracer`] trait.
//!
//! Manages span lifecycle, context propagation, and ring buffer storage.

use crate::context;
use crate::ringbuffer::RingBuffer;
use crate::span::Span;
use minios_common::id::{SpanId, SpanIdGenerator, TraceIdGenerator};
use minios_common::traits::trace::Tracer;
use minios_common::types::{AttributeValue, SpanStatus, TraceContext, TraceStats};
use spin::Mutex;

/// The global trace engine instance.
pub static TRACER: spin::Lazy<TraceEngine> = spin::Lazy::new(TraceEngine::new);

/// Trace engine that records spans into a fixed-size ring buffer.
pub struct TraceEngine {
    /// Ring buffer holding completed and in-progress spans.
    buffer: Mutex<RingBuffer>,
    /// Generator for unique trace IDs.
    trace_ids: TraceIdGenerator,
    /// Generator for unique span IDs.
    span_ids: SpanIdGenerator,
}

impl TraceEngine {
    /// Creates a new trace engine with empty buffers.
    fn new() -> Self {
        Self {
            buffer: Mutex::new(RingBuffer::new()),
            trace_ids: TraceIdGenerator::new(),
            span_ids: SpanIdGenerator::new(),
        }
    }
}

// SAFETY: All interior state is protected by Mutex or atomic operations.
unsafe impl Send for TraceEngine {}
unsafe impl Sync for TraceEngine {}

impl Tracer for TraceEngine {
    fn begin_span(&self, name: &str, module: &str) -> SpanId {
        let span_id = self.span_ids.next();
        let (trace_id, parent, depth) = match context::current() {
            Some(ctx) => (ctx.trace_id, Some(ctx.current_span_id), ctx.depth + 1),
            None => (self.trace_ids.next(), None, 0),
        };

        let span = Span::new(name, module, trace_id, span_id, parent, 0, depth);

        context::push(TraceContext {
            trace_id,
            current_span_id: span_id,
            depth,
        });

        self.buffer.lock().write(span);
        span_id
    }

    fn end_span(&self, span_id: SpanId, status: SpanStatus) {
        context::pop();
        let tsc = read_tsc();
        self.buffer.lock().update_span(span_id, tsc, status);
    }

    fn add_event(&self, name: &str, _attrs: &[(&str, AttributeValue)]) {
        let span_id = self.span_ids.next();
        let (trace_id, parent, depth) = match context::current() {
            Some(ctx) => (ctx.trace_id, Some(ctx.current_span_id), ctx.depth + 1),
            None => (self.trace_ids.next(), None, 0),
        };

        let tsc = read_tsc();
        let mut span = Span::new(name, "event", trace_id, span_id, parent, 0, depth);
        span.end_tsc = tsc;
        span.status = SpanStatus::Ok;

        self.buffer.lock().write(span);
    }

    fn current_context(&self) -> Option<TraceContext> {
        context::current()
    }

    fn set_context(&self, ctx: TraceContext) {
        context::push(ctx);
    }

    fn clear_context(&self) {
        context::clear();
    }

    fn stats(&self) -> TraceStats {
        let buf = self.buffer.lock();
        let (capacity, used, total) = buf.stats();
        TraceStats {
            total_spans_written: total,
            buffer_capacity: capacity,
            buffer_used: used,
            active_spans: 0,
        }
    }

    fn clear(&self) {
        self.buffer.lock().clear();
        context::clear();
    }
}

impl TraceEngine {
    /// Copies the most recent `count` spans into the provided buffer.
    ///
    /// Returns the number of spans actually read.
    pub fn read_recent(&self, count: usize, out: &mut [Span]) -> usize {
        self.buffer.lock().read_recent(count, out)
    }
}

/// Reads the CPU timestamp counter.
fn read_tsc() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        unsafe { core::arch::x86_64::_rdtsc() }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        0
    }
}
