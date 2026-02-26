//! Trace engine contract.

use crate::id::SpanId;
use crate::types::{AttributeValue, SpanStatus, TraceContext, TraceStats};

/// Core observability engine powering full-chain tracing.
pub trait Tracer: Send + Sync {
    /// Opens a new span and returns its ID.
    fn begin_span(&self, name: &str, module: &str) -> SpanId;
    /// Closes a span with the given completion status.
    fn end_span(&self, span_id: SpanId, status: SpanStatus);
    /// Records a point-in-time event with optional attributes.
    fn add_event(&self, name: &str, attrs: &[(&str, AttributeValue)]);
    /// Returns the current trace context, if any.
    fn current_context(&self) -> Option<TraceContext>;
    /// Sets the active trace context.
    fn set_context(&self, ctx: TraceContext);
    /// Clears the active trace context.
    fn clear_context(&self);
    /// Returns runtime trace engine statistics.
    fn stats(&self) -> TraceStats;
    /// Clears all recorded spans from the buffer.
    fn clear(&self);
}
