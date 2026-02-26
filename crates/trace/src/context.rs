//! Per-CPU trace context stack.
//!
//! Maintains a stack of [`TraceContext`] values so that nested spans can
//! discover their parent trace and span IDs.

use minios_common::types::TraceContext;
use spin::Mutex;

/// Maximum nesting depth for trace contexts.
const MAX_DEPTH: usize = 64;

/// Stack of active trace contexts protected by a spin lock.
static CONTEXT_STACK: Mutex<TraceContextStack> = Mutex::new(TraceContextStack::new());

/// Fixed-capacity stack of [`TraceContext`] frames.
pub struct TraceContextStack {
    /// Inline storage for context frames.
    entries: [Option<TraceContext>; MAX_DEPTH],
    /// Number of frames currently on the stack.
    len: usize,
}

impl TraceContextStack {
    /// Creates an empty context stack.
    const fn new() -> Self {
        Self {
            entries: [None; MAX_DEPTH],
            len: 0,
        }
    }
}

/// Pushes a trace context onto the global stack.
///
/// Returns `false` if the stack is full (depth exceeded).
pub fn push(ctx: TraceContext) -> bool {
    let mut stack = CONTEXT_STACK.lock();
    if stack.len >= MAX_DEPTH {
        return false;
    }
    let idx = stack.len;
    stack.entries[idx] = Some(ctx);
    stack.len += 1;
    true
}

/// Pops the most recent trace context from the global stack.
pub fn pop() -> Option<TraceContext> {
    let mut stack = CONTEXT_STACK.lock();
    if stack.len == 0 {
        return None;
    }
    stack.len -= 1;
    let idx = stack.len;
    stack.entries[idx].take()
}

/// Returns a copy of the current (top-of-stack) trace context without
/// removing it.
pub fn current() -> Option<TraceContext> {
    let stack = CONTEXT_STACK.lock();
    if stack.len == 0 {
        None
    } else {
        stack.entries[stack.len - 1]
    }
}

/// Clears all entries from the context stack.
pub fn clear() {
    let mut stack = CONTEXT_STACK.lock();
    stack.len = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use minios_common::id::{SpanId, TraceId};

    fn make_ctx(trace: u64, span: u64, depth: u16) -> TraceContext {
        TraceContext {
            trace_id: TraceId(trace),
            current_span_id: SpanId(span),
            depth,
        }
    }

    #[test]
    fn push_and_current() {
        clear();
        assert!(current().is_none());
        push(make_ctx(1, 10, 0));
        let c = current().unwrap();
        assert_eq!(c.trace_id, TraceId(1));
        assert_eq!(c.current_span_id, SpanId(10));
        clear();
    }

    #[test]
    fn push_pop_lifo() {
        clear();
        push(make_ctx(1, 1, 0));
        push(make_ctx(2, 2, 1));
        let top = pop().unwrap();
        assert_eq!(top.trace_id, TraceId(2));
        let next = pop().unwrap();
        assert_eq!(next.trace_id, TraceId(1));
        assert!(pop().is_none());
        clear();
    }

    #[test]
    fn pop_empty_returns_none() {
        clear();
        assert!(pop().is_none());
    }

    #[test]
    fn clear_empties_stack() {
        clear();
        push(make_ctx(1, 1, 0));
        push(make_ctx(2, 2, 1));
        clear();
        assert!(current().is_none());
        assert!(pop().is_none());
    }

    #[test]
    fn push_returns_false_at_max_depth() {
        clear();
        for i in 0..MAX_DEPTH {
            assert!(push(make_ctx(i as u64, i as u64, i as u16)));
        }
        assert!(!push(make_ctx(999, 999, 99)));
        clear();
    }
}
