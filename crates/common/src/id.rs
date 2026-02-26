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

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::format;

    #[test]
    fn pid_display() {
        assert_eq!(format!("{}", Pid(0)), "0");
        assert_eq!(format!("{}", Pid(42)), "42");
        assert_eq!(format!("{}", Pid(u32::MAX)), format!("{}", u32::MAX));
    }

    #[test]
    fn trace_id_display_hex() {
        assert_eq!(format!("{}", TraceId(0)), "0x0000000000000000");
        assert_eq!(format!("{}", TraceId(0xABCD)), "0x000000000000ABCD");
        assert_eq!(format!("{}", TraceId(u64::MAX)), "0xFFFFFFFFFFFFFFFF");
    }

    #[test]
    fn span_id_display_hex() {
        assert_eq!(format!("{}", SpanId(0)), "0x0000");
        assert_eq!(format!("{}", SpanId(255)), "0x00FF");
    }

    #[test]
    fn pid_allocator_monotonic() {
        let alloc = PidAllocator::new();
        assert_eq!(alloc.allocate(), Pid(0));
        assert_eq!(alloc.allocate(), Pid(1));
        assert_eq!(alloc.allocate(), Pid(2));
    }

    #[test]
    fn pid_allocator_default() {
        let alloc = PidAllocator::default();
        assert_eq!(alloc.allocate(), Pid(0));
    }

    #[test]
    fn span_id_generator_unique() {
        let gen = SpanIdGenerator::new();
        let a = gen.next();
        let b = gen.next();
        let c = gen.next();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_eq!(a, SpanId(1));
        assert_eq!(b, SpanId(2));
        assert_eq!(c, SpanId(3));
    }

    #[test]
    fn trace_id_generator_unique() {
        let gen = TraceIdGenerator::new();
        let a = gen.next();
        let b = gen.next();
        assert_ne!(a, b);
        assert_eq!(a, TraceId(1));
        assert_eq!(b, TraceId(2));
    }

    #[test]
    fn pid_ordering() {
        assert!(Pid(1) < Pid(2));
        assert_eq!(Pid(5), Pid(5));
    }

    #[test]
    fn file_descriptor_equality() {
        assert_eq!(FileDescriptor(3), FileDescriptor(3));
        assert_ne!(FileDescriptor(1), FileDescriptor(2));
    }

    #[test]
    fn inode_id_equality() {
        assert_eq!(InodeId(0), InodeId(0));
        assert_ne!(InodeId(1), InodeId(2));
    }

    #[test]
    fn queue_id_equality() {
        assert_eq!(QueueId(1), QueueId(1));
        assert_ne!(QueueId(0), QueueId(1));
    }

    #[test]
    fn shm_id_equality() {
        assert_eq!(ShmId(0), ShmId(0));
        assert_ne!(ShmId(1), ShmId(2));
    }

    #[test]
    fn span_id_generator_starts_at_one() {
        let gen = SpanIdGenerator::default();
        assert_eq!(gen.next(), SpanId(1));
    }

    #[test]
    fn trace_id_generator_default() {
        let gen = TraceIdGenerator::default();
        assert_eq!(gen.next(), TraceId(1));
    }
}
