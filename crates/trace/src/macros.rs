//! Convenience macros for trace instrumentation.

/// Begins a named trace span and returns a [`SpanGuard`](crate::guard::SpanGuard)
/// that automatically ends the span when dropped.
///
/// # Examples
///
/// ```ignore
/// let _guard = trace_span!("alloc_page", module = "memory");
/// ```
#[macro_export]
macro_rules! trace_span {
    ($name:expr, module = $module:expr) => {
        $crate::guard::SpanGuard::new($crate::engine::TRACER.begin_span($name, $module))
    };
}

/// Records an instant trace event (a span with zero duration).
///
/// # Examples
///
/// ```ignore
/// trace_event!("page_fault");
/// ```
#[macro_export]
macro_rules! trace_event {
    ($name:expr) => {
        $crate::engine::TRACER.add_event($name, &[])
    };
}
