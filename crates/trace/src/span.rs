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
