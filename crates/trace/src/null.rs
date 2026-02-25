//! No-op tracer for use in tests and contexts where tracing is disabled.

use minios_common::id::SpanId;
use minios_common::traits::trace::Tracer;
use minios_common::types::{AttributeValue, SpanStatus, TraceContext, TraceStats};

/// A tracer that silently discards all spans and events.
pub struct NullTracer;

impl Tracer for NullTracer {
    fn begin_span(&self, _name: &str, _module: &str) -> SpanId {
        SpanId(0)
    }

    fn end_span(&self, _span_id: SpanId, _status: SpanStatus) {}

    fn add_event(&self, _name: &str, _attrs: &[(&str, AttributeValue)]) {}

    fn current_context(&self) -> Option<TraceContext> {
        None
    }

    fn set_context(&self, _ctx: TraceContext) {}

    fn clear_context(&self) {}

    fn stats(&self) -> TraceStats {
        TraceStats {
            total_spans_written: 0,
            buffer_capacity: 0,
            buffer_used: 0,
            active_spans: 0,
        }
    }

    fn clear(&self) {}
}
