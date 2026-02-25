//! RAII guard that automatically ends a span when dropped.

use crate::engine::TRACER;
use minios_common::id::SpanId;
use minios_common::traits::trace::Tracer;
use minios_common::types::SpanStatus;

/// Owns a span ID and calls [`Tracer::end_span`] on drop.
///
/// Create via [`trace_span!`](crate::trace_span) or manually with
/// [`SpanGuard::new`].
pub struct SpanGuard {
    span_id: SpanId,
}

impl SpanGuard {
    /// Wraps a span ID so it will be ended automatically.
    pub fn new(span_id: SpanId) -> Self {
        Self { span_id }
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        TRACER.end_span(self.span_id, SpanStatus::Ok);
    }
}
