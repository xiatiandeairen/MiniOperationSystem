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
