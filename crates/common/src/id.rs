//! Type-safe identifiers used across all kernel subsystems.

use core::fmt;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Globally unique trace identifier linking all spans in one call chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TraceId(pub u64);

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016X}", self.0)
    }
}

/// Identifies a single operation span within a trace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpanId(pub u64);

impl fmt::Display for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:04X}", self.0)
    }
}

/// Process identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pid(pub u32);

impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// File descriptor handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileDescriptor(pub i32);

/// Filesystem inode identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InodeId(pub u64);

/// IPC message queue identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct QueueId(pub u32);

/// Shared memory region identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShmId(pub u32);

/// Atomic PID generator. Each call returns a unique, monotonically increasing PID.
pub struct PidAllocator {
    next: AtomicU32,
}

impl Default for PidAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl PidAllocator {
    pub const fn new() -> Self {
        Self {
            next: AtomicU32::new(0),
        }
    }

    pub fn allocate(&self) -> Pid {
        Pid(self.next.fetch_add(1, Ordering::Relaxed))
    }
}

/// Atomic SpanId generator.
pub struct SpanIdGenerator {
    next: AtomicU64,
}

impl Default for SpanIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SpanIdGenerator {
    pub const fn new() -> Self {
        Self {
            next: AtomicU64::new(1),
        }
    }

    pub fn next(&self) -> SpanId {
        SpanId(self.next.fetch_add(1, Ordering::Relaxed))
    }
}

/// Atomic TraceId generator.
pub struct TraceIdGenerator {
    next: AtomicU64,
}

impl Default for TraceIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceIdGenerator {
    pub const fn new() -> Self {
        Self {
            next: AtomicU64::new(1),
        }
    }

    pub fn next(&self) -> TraceId {
        TraceId(self.next.fetch_add(1, Ordering::Relaxed))
    }
}
