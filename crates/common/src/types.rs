//! Shared data types used across kernel subsystems.

use crate::id::{Pid, SpanId, TraceId};
use core::fmt;

// ---------------------------------------------------------------------------
// VGA Color
// ---------------------------------------------------------------------------

/// Standard 16-color VGA palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    /// Black (0).
    Black = 0,
    /// Blue (1).
    Blue = 1,
    /// Green (2).
    Green = 2,
    /// Cyan (3).
    Cyan = 3,
    /// Red (4).
    Red = 4,
    /// Magenta (5).
    Magenta = 5,
    /// Brown (6).
    Brown = 6,
    /// Light gray (7).
    LightGray = 7,
    /// Dark gray (8).
    DarkGray = 8,
    /// Light blue (9).
    LightBlue = 9,
    /// Light green (10).
    LightGreen = 10,
    /// Light cyan (11).
    LightCyan = 11,
    /// Light red (12).
    LightRed = 12,
    /// Pink (13).
    Pink = 13,
    /// Yellow (14).
    Yellow = 14,
    /// White (15).
    White = 15,
}

/// Packed VGA foreground + background color byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    /// Packs a foreground and background color into a single byte.
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

// ---------------------------------------------------------------------------
// Process types
// ---------------------------------------------------------------------------

/// Lifecycle state of a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Newly created, not yet scheduled.
    Created,
    /// Waiting in the run queue.
    Ready,
    /// Currently executing on the CPU.
    Running,
    /// Waiting for an event (I/O, IPC, sleep).
    Blocked,
    /// Exited or killed.
    Terminated,
}

impl fmt::Display for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "CREATED"),
            Self::Ready => write!(f, "READY"),
            Self::Running => write!(f, "RUNNING"),
            Self::Blocked => write!(f, "BLOCKED"),
            Self::Terminated => write!(f, "TERMINATED"),
        }
    }
}

/// Scheduling priority (lower value = higher priority).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(pub u8);

impl Priority {
    /// Highest priority (0).
    pub const HIGH: Self = Self(0);
    /// Medium priority (1).
    pub const MEDIUM: Self = Self(1);
    /// Low priority (2).
    pub const LOW: Self = Self(2);
    /// Idle priority (3) — only runs when nothing else is ready.
    pub const IDLE: Self = Self(3);
}

/// Summary information about a process, suitable for `ps` output.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Process identifier.
    pub pid: Pid,
    /// Human-readable name (fixed-size buffer).
    pub name: [u8; 32],
    /// Actual length of the name.
    pub name_len: usize,
    /// Current lifecycle state.
    pub state: ProcessState,
    /// Scheduling priority.
    pub priority: Priority,
    /// Accumulated CPU ticks consumed.
    pub cpu_time_ticks: u64,
}

/// Reason a process was blocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockReason {
    /// Waiting for I/O completion.
    Io,
    /// Waiting for an IPC message.
    IpcReceive,
    /// Sleeping for a specified duration.
    Sleep,
    /// Waiting for a child process to exit.
    WaitChild,
}

// ---------------------------------------------------------------------------
// Scheduler types
// ---------------------------------------------------------------------------

/// Decision returned by the scheduler after each timer tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleDecision {
    /// Current task keeps running.
    Continue,
    /// Switch to the specified task.
    Switch(Pid),
    /// No runnable task — enter idle.
    Idle,
}

/// Runtime statistics from the scheduler.
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    /// Number of context switches performed.
    pub total_switches: u64,
    /// Total timer ticks processed.
    pub total_ticks: u64,
    /// Number of tasks in each priority queue.
    pub queue_lengths: [usize; 4],
    /// Ticks spent in idle (no runnable task).
    pub idle_ticks: u64,
}

// ---------------------------------------------------------------------------
// Filesystem types
// ---------------------------------------------------------------------------

bitflags::bitflags! {
    /// Flags passed to `open()`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct OpenFlags: u32 {
        /// Open for reading.
        const READ   = 0b0001;
        /// Open for writing.
        const WRITE  = 0b0010;
        /// Create the file if it does not exist.
        const CREATE = 0b0100;
        /// Append to the end of the file.
        const APPEND = 0b1000;
    }
}

/// Origin for `seek()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekWhence {
    /// Offset from the beginning of the file.
    Start,
    /// Offset from the current position.
    Current,
    /// Offset from the end of the file.
    End,
}

/// Type of an inode (file, directory, device, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InodeType {
    /// Regular file.
    File,
    /// Directory.
    Directory,
    /// Character device.
    CharDevice,
    /// Special (e.g. proc, pipe).
    Special,
}

/// A single directory entry returned by `readdir()`.
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// File name bytes (up to 255).
    pub name: [u8; 255],
    /// Actual length of the name.
    pub name_len: usize,
    /// Type of the inode.
    pub inode_type: InodeType,
}

/// Metadata about a file or directory.
#[derive(Debug, Clone)]
pub struct FileStat {
    /// Size in bytes.
    pub size: usize,
    /// Type of the inode.
    pub inode_type: InodeType,
    /// Creation timestamp (ticks).
    pub created_at: u64,
    /// Last modification timestamp (ticks).
    pub modified_at: u64,
}

// ---------------------------------------------------------------------------
// Device driver types
// ---------------------------------------------------------------------------

/// Broad device classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Character-oriented device (serial, keyboard).
    CharDevice,
    /// Block-oriented device (disk).
    BlockDevice,
}

// ---------------------------------------------------------------------------
// Trace types
// ---------------------------------------------------------------------------

/// Trace context propagated through the call chain.
#[derive(Clone, Copy, Debug)]
pub struct TraceContext {
    /// Root trace identifier.
    pub trace_id: TraceId,
    /// Currently active span.
    pub current_span_id: SpanId,
    /// Nesting depth of the current span.
    pub depth: u16,
}

/// Value stored inside a span attribute.
#[derive(Clone, Copy, Debug)]
pub enum AttributeValue {
    /// Unsigned 64-bit integer.
    U64(u64),
    /// Signed 64-bit integer.
    I64(i64),
    /// Boolean.
    Bool(bool),
}

/// Completion status of a trace span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanStatus {
    /// Span completed successfully.
    Ok,
    /// Span ended with an error.
    Error,
    /// Span is still open.
    InProgress,
}

/// Runtime statistics of the trace engine.
#[derive(Debug, Clone)]
pub struct TraceStats {
    /// Total spans ever written to the buffer.
    pub total_spans_written: u64,
    /// Maximum number of spans the buffer can hold.
    pub buffer_capacity: usize,
    /// Spans currently stored in the buffer.
    pub buffer_used: usize,
    /// Spans that are still open (in-progress).
    pub active_spans: usize,
}

/// Filter criteria for reading spans from the trace buffer.
#[derive(Debug, Clone, Default)]
pub struct SpanFilter {
    /// Filter by module name.
    pub module: Option<[u8; 32]>,
    /// Filter by trace ID.
    pub trace_id: Option<TraceId>,
    /// Filter by owning process.
    pub pid: Option<Pid>,
    /// Filter by span completion status.
    pub status: Option<SpanStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::format;

    #[test]
    fn color_code_packs_correctly() {
        let cc = ColorCode::new(Color::White, Color::Blue);
        assert_eq!(cc, ColorCode(0x1F));
    }

    #[test]
    fn color_code_black_on_black() {
        let cc = ColorCode::new(Color::Black, Color::Black);
        assert_eq!(cc, ColorCode(0x00));
    }

    #[test]
    fn color_code_white_on_white() {
        let cc = ColorCode::new(Color::White, Color::White);
        assert_eq!(cc, ColorCode(0xFF));
    }

    #[test]
    fn priority_ordering() {
        assert!(Priority::HIGH < Priority::LOW);
        assert!(Priority::HIGH < Priority::MEDIUM);
        assert!(Priority::MEDIUM < Priority::LOW);
        assert!(Priority::LOW < Priority::IDLE);
    }

    #[test]
    fn process_state_display() {
        assert_eq!(format!("{}", ProcessState::Created), "CREATED");
        assert_eq!(format!("{}", ProcessState::Running), "RUNNING");
        assert_eq!(format!("{}", ProcessState::Terminated), "TERMINATED");
    }

    #[test]
    fn open_flags_bitwise() {
        let flags = OpenFlags::READ | OpenFlags::WRITE;
        assert!(flags.contains(OpenFlags::READ));
        assert!(flags.contains(OpenFlags::WRITE));
        assert!(!flags.contains(OpenFlags::CREATE));
        assert!(!flags.contains(OpenFlags::APPEND));
    }

    #[test]
    fn open_flags_all() {
        let all = OpenFlags::READ | OpenFlags::WRITE | OpenFlags::CREATE | OpenFlags::APPEND;
        assert!(all.contains(OpenFlags::READ));
        assert!(all.contains(OpenFlags::CREATE));
    }

    #[test]
    fn open_flags_empty() {
        let empty = OpenFlags::empty();
        assert!(!empty.contains(OpenFlags::READ));
    }

    #[test]
    fn schedule_decision_variants() {
        let cont = ScheduleDecision::Continue;
        let sw = ScheduleDecision::Switch(Pid(5));
        let idle = ScheduleDecision::Idle;
        assert_eq!(cont, ScheduleDecision::Continue);
        assert_eq!(sw, ScheduleDecision::Switch(Pid(5)));
        assert_ne!(sw, ScheduleDecision::Switch(Pid(6)));
        assert_eq!(idle, ScheduleDecision::Idle);
    }

    #[test]
    fn process_state_all_variants_display() {
        let states = [
            (ProcessState::Created, "CREATED"),
            (ProcessState::Ready, "READY"),
            (ProcessState::Running, "RUNNING"),
            (ProcessState::Blocked, "BLOCKED"),
            (ProcessState::Terminated, "TERMINATED"),
        ];
        for (state, expected) in &states {
            assert_eq!(format!("{}", state), *expected);
        }
    }

    #[test]
    fn span_status_equality() {
        assert_eq!(SpanStatus::Ok, SpanStatus::Ok);
        assert_ne!(SpanStatus::Ok, SpanStatus::Error);
        assert_ne!(SpanStatus::Error, SpanStatus::InProgress);
    }

    #[test]
    fn seek_whence_variants() {
        assert_eq!(SeekWhence::Start, SeekWhence::Start);
        assert_ne!(SeekWhence::Start, SeekWhence::Current);
        assert_ne!(SeekWhence::Current, SeekWhence::End);
    }

    #[test]
    fn inode_type_variants() {
        assert_eq!(InodeType::File, InodeType::File);
        assert_ne!(InodeType::File, InodeType::Directory);
        assert_ne!(InodeType::Directory, InodeType::CharDevice);
        assert_ne!(InodeType::CharDevice, InodeType::Special);
    }

    #[test]
    fn block_reason_all_variants_exist() {
        let reasons = [
            BlockReason::Io,
            BlockReason::IpcReceive,
            BlockReason::Sleep,
            BlockReason::WaitChild,
        ];
        for (i, r) in reasons.iter().enumerate() {
            for (j, s) in reasons.iter().enumerate() {
                if i == j {
                    assert_eq!(r, s);
                } else {
                    assert_ne!(r, s);
                }
            }
        }
    }

    #[test]
    fn scheduler_stats_default_values() {
        let stats = SchedulerStats {
            total_switches: 0,
            total_ticks: 0,
            queue_lengths: [0; 4],
            idle_ticks: 0,
        };
        assert_eq!(stats.total_switches, 0);
        assert_eq!(stats.total_ticks, 0);
        assert_eq!(stats.queue_lengths, [0, 0, 0, 0]);
        assert_eq!(stats.idle_ticks, 0);
    }

    #[test]
    fn seek_whence_all_variants() {
        let variants = [SeekWhence::Start, SeekWhence::Current, SeekWhence::End];
        assert_eq!(variants.len(), 3);
        assert_ne!(variants[0], variants[1]);
        assert_ne!(variants[1], variants[2]);
        assert_ne!(variants[0], variants[2]);
    }

    #[test]
    fn trace_context_clone() {
        let ctx = TraceContext {
            trace_id: crate::id::TraceId(42),
            current_span_id: crate::id::SpanId(7),
            depth: 3,
        };
        let copy = ctx;
        assert_eq!(copy.trace_id, crate::id::TraceId(42));
        assert_eq!(copy.current_span_id, crate::id::SpanId(7));
        assert_eq!(copy.depth, 3);
    }

    #[test]
    fn attribute_value_variants() {
        let u = AttributeValue::U64(100);
        let i = AttributeValue::I64(-50);
        let b = AttributeValue::Bool(true);
        assert!(matches!(u, AttributeValue::U64(100)));
        assert!(matches!(i, AttributeValue::I64(-50)));
        assert!(matches!(b, AttributeValue::Bool(true)));
    }

    #[test]
    fn device_type_variants() {
        assert_eq!(DeviceType::CharDevice, DeviceType::CharDevice);
        assert_eq!(DeviceType::BlockDevice, DeviceType::BlockDevice);
        assert_ne!(DeviceType::CharDevice, DeviceType::BlockDevice);
    }
}
