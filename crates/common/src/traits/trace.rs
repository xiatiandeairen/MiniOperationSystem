//! Trace engine contract.

use crate::id::SpanId;
use crate::types::{AttributeValue, SpanFilter, SpanStatus, TraceContext, TraceStats};

/// Core observability engine powering full-chain tracing.
pub trait Tracer: Send + Sync {
    fn begin_span(&self, name: &str, module: &str) -> SpanId;
    fn end_span(&self, span_id: SpanId, status: SpanStatus);
    fn add_event(&self, name: &str, attrs: &[(&str, AttributeValue)]);
    fn current_context(&self) -> Option<TraceContext>;
    fn set_context(&self, ctx: TraceContext);
    fn clear_context(&self);
    fn stats(&self) -> TraceStats;
    fn clear(&self);
}
