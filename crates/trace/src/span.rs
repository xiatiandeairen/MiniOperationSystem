//! Span data structure representing a single traced operation.

use minios_common::id::{SpanId, TraceId};
use minios_common::types::SpanStatus;

/// Maximum length of a span name in bytes.
pub const MAX_NAME_LEN: usize = 64;

/// Maximum length of a module name in bytes.
pub const MAX_MODULE_LEN: usize = 32;

/// A single trace span stored in the ring buffer.
///
/// Uses fixed-size arrays to avoid heap allocation. Names are stored as
/// UTF-8 byte slices with an accompanying length field.
#[repr(C)]
pub struct Span {
    /// Trace this span belongs to.
    pub trace_id: TraceId,
    /// Unique identifier for this span.
    pub span_id: SpanId,
    /// Parent span, if any.
    pub parent_span_id: Option<SpanId>,
    /// UTF-8 name bytes (fixed buffer).
    pub name: [u8; MAX_NAME_LEN],
    /// Actual length of the name.
    pub name_len: usize,
    /// UTF-8 module name bytes (fixed buffer).
    pub module: [u8; MAX_MODULE_LEN],
    /// Actual length of the module name.
    pub module_len: usize,
    /// TSC timestamp when the span started.
    pub start_tsc: u64,
    /// TSC timestamp when the span ended (0 while in progress).
    pub end_tsc: u64,
    /// Current completion status.
    pub status: SpanStatus,
    /// Process that owns this span.
    pub pid: u32,
    /// Nesting depth within the current trace.
    pub depth: u16,
}

impl Span {
    /// Creates a new span with the given metadata.
    pub fn new(
        name: &str,
        module: &str,
        trace_id: TraceId,
        span_id: SpanId,
        parent: Option<SpanId>,
        pid: u32,
        depth: u16,
    ) -> Self {
        let mut name_buf = [0u8; MAX_NAME_LEN];
        let n = name.len().min(MAX_NAME_LEN);
        name_buf[..n].copy_from_slice(&name.as_bytes()[..n]);

        let mut module_buf = [0u8; MAX_MODULE_LEN];
        let m = module.len().min(MAX_MODULE_LEN);
        module_buf[..m].copy_from_slice(&module.as_bytes()[..m]);

        Self {
            trace_id,
            span_id,
            parent_span_id: parent,
            name: name_buf,
            name_len: n,
            module: module_buf,
            module_len: m,
            start_tsc: read_tsc(),
            end_tsc: 0,
            status: SpanStatus::InProgress,
            pid,
            depth,
        }
    }

    /// Returns the span name as a `&str`.
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("<invalid>")
    }

    /// Returns the module name as a `&str`.
    pub fn module_str(&self) -> &str {
        core::str::from_utf8(&self.module[..self.module_len]).unwrap_or("<invalid>")
    }
}

impl Default for Span {
    fn default() -> Self {
        Self {
            trace_id: TraceId(0),
            span_id: SpanId(0),
            parent_span_id: None,
            name: [0u8; MAX_NAME_LEN],
            name_len: 0,
            module: [0u8; MAX_MODULE_LEN],
            module_len: 0,
            start_tsc: 0,
            end_tsc: 0,
            status: SpanStatus::InProgress,
            pid: 0,
            depth: 0,
        }
    }
}

impl Clone for Span {
    fn clone(&self) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: self.span_id,
            parent_span_id: self.parent_span_id,
            name: self.name,
            name_len: self.name_len,
            module: self.module,
            module_len: self.module_len,
            start_tsc: self.start_tsc,
            end_tsc: self.end_tsc,
            status: self.status,
            pid: self.pid,
            depth: self.depth,
        }
    }
}

/// Reads the CPU timestamp counter (or returns 0 on unsupported platforms).
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

#[cfg(test)]
mod tests {
    use super::*;
    use minios_common::id::{SpanId, TraceId};

    #[test]
    fn span_new_fields() {
        let span = Span::new(
            "my_op",
            "kernel::mem",
            TraceId(100),
            SpanId(7),
            Some(SpanId(3)),
            42,
            2,
        );
        assert_eq!(span.trace_id, TraceId(100));
        assert_eq!(span.span_id, SpanId(7));
        assert_eq!(span.parent_span_id, Some(SpanId(3)));
        assert_eq!(span.pid, 42);
        assert_eq!(span.depth, 2);
        assert_eq!(span.status, SpanStatus::InProgress);
        assert_eq!(span.end_tsc, 0);
    }

    #[test]
    fn name_str_returns_correct_slice() {
        let span = Span::new("hello", "mod", TraceId(1), SpanId(1), None, 0, 0);
        assert_eq!(span.name_str(), "hello");
    }

    #[test]
    fn module_str_returns_correct_slice() {
        let span = Span::new("op", "scheduler", TraceId(1), SpanId(1), None, 0, 0);
        assert_eq!(span.module_str(), "scheduler");
    }

    #[test]
    fn name_truncation() {
        let long_name = "a]".repeat(MAX_NAME_LEN + 10);
        let span = Span::new(&long_name, "m", TraceId(1), SpanId(1), None, 0, 0);
        assert_eq!(span.name_len, MAX_NAME_LEN);
        assert_eq!(span.name_str().len(), MAX_NAME_LEN);
    }

    #[test]
    fn module_truncation() {
        let long_mod = "b".repeat(MAX_MODULE_LEN + 20);
        let span = Span::new("op", &long_mod, TraceId(1), SpanId(1), None, 0, 0);
        assert_eq!(span.module_len, MAX_MODULE_LEN);
        assert_eq!(span.module_str().len(), MAX_MODULE_LEN);
    }

    #[test]
    fn empty_name_and_module() {
        let span = Span::new("", "", TraceId(1), SpanId(1), None, 0, 0);
        assert_eq!(span.name_str(), "");
        assert_eq!(span.module_str(), "");
        assert_eq!(span.name_len, 0);
        assert_eq!(span.module_len, 0);
    }

    #[test]
    fn default_span() {
        let span = Span::default();
        assert_eq!(span.trace_id, TraceId(0));
        assert_eq!(span.span_id, SpanId(0));
        assert_eq!(span.parent_span_id, None);
        assert_eq!(span.name_str(), "");
        assert_eq!(span.module_str(), "");
        assert_eq!(span.status, SpanStatus::InProgress);
    }

    #[test]
    fn clone_preserves_fields() {
        let span = Span::new("cloned", "test", TraceId(9), SpanId(4), None, 1, 3);
        let copy = span.clone();
        assert_eq!(copy.trace_id, span.trace_id);
        assert_eq!(copy.span_id, span.span_id);
        assert_eq!(copy.name_str(), span.name_str());
        assert_eq!(copy.module_str(), span.module_str());
        assert_eq!(copy.pid, span.pid);
        assert_eq!(copy.depth, span.depth);
    }
}
