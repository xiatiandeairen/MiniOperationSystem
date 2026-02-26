//! Typed error definitions for every kernel subsystem.

use core::fmt;

/// Unified kernel error covering all subsystem failures.
#[derive(Debug)]
pub enum KernelError {
    Memory(MemoryError),
    Process(ProcessError),
    FileSystem(FsError),
    Ipc(IpcError),
    Driver(DriverError),
    Trace(TraceError),
}

// --- Memory -----------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    OutOfMemory,
    InvalidAddress,
    AlreadyMapped,
    NotMapped,
    AlignmentError,
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfMemory => write!(f, "out of physical memory"),
            Self::InvalidAddress => write!(f, "invalid address"),
            Self::AlreadyMapped => write!(f, "page already mapped"),
            Self::NotMapped => write!(f, "page not mapped"),
            Self::AlignmentError => write!(f, "address alignment error"),
        }
    }
}

impl From<MemoryError> for KernelError {
    fn from(e: MemoryError) -> Self {
        Self::Memory(e)
    }
}

// --- Process ----------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessError {
    MaxProcessesReached,
    InvalidPid,
    ProcessNotFound,
    InvalidStateTransition,
    StackAllocationFailed,
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaxProcessesReached => write!(f, "max processes reached"),
            Self::InvalidPid => write!(f, "invalid pid"),
            Self::ProcessNotFound => write!(f, "process not found"),
            Self::InvalidStateTransition => write!(f, "invalid state transition"),
            Self::StackAllocationFailed => write!(f, "stack allocation failed"),
        }
    }
}

impl From<ProcessError> for KernelError {
    fn from(e: ProcessError) -> Self {
        Self::Process(e)
    }
}

// --- FileSystem -------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    NotADirectory,
    NotAFile,
    PermissionDenied,
    NoSpace,
    InvalidPath,
    TooManyOpenFiles,
    InvalidDescriptor,
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found"),
            Self::AlreadyExists => write!(f, "already exists"),
            Self::NotADirectory => write!(f, "not a directory"),
            Self::NotAFile => write!(f, "not a file"),
            Self::PermissionDenied => write!(f, "permission denied"),
            Self::NoSpace => write!(f, "no space"),
            Self::InvalidPath => write!(f, "invalid path"),
            Self::TooManyOpenFiles => write!(f, "too many open files"),
            Self::InvalidDescriptor => write!(f, "invalid descriptor"),
        }
    }
}

impl From<FsError> for KernelError {
    fn from(e: FsError) -> Self {
        Self::FileSystem(e)
    }
}

// --- IPC --------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    QueueFull,
    QueueEmpty,
    QueueNotFound,
    Timeout,
    InvalidMessage,
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueueFull => write!(f, "queue full"),
            Self::QueueEmpty => write!(f, "queue empty"),
            Self::QueueNotFound => write!(f, "queue not found"),
            Self::Timeout => write!(f, "timeout"),
            Self::InvalidMessage => write!(f, "invalid message"),
        }
    }
}

impl From<IpcError> for KernelError {
    fn from(e: IpcError) -> Self {
        Self::Ipc(e)
    }
}

// --- Driver -----------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    NotInitialized,
    DeviceNotFound,
    IoError,
    Unsupported,
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "not initialized"),
            Self::DeviceNotFound => write!(f, "device not found"),
            Self::IoError => write!(f, "I/O error"),
            Self::Unsupported => write!(f, "unsupported operation"),
        }
    }
}

impl From<DriverError> for KernelError {
    fn from(e: DriverError) -> Self {
        Self::Driver(e)
    }
}

// --- Trace ------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceError {
    BufferFull,
    MaxDepthExceeded,
    NotInitialized,
}

impl fmt::Display for TraceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferFull => write!(f, "trace buffer full"),
            Self::MaxDepthExceeded => write!(f, "max trace depth exceeded"),
            Self::NotInitialized => write!(f, "tracer not initialized"),
        }
    }
}

impl From<TraceError> for KernelError {
    fn from(e: TraceError) -> Self {
        Self::Trace(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::format;

    #[test]
    fn kernel_error_from_memory() {
        let e = KernelError::from(MemoryError::OutOfMemory);
        assert!(matches!(e, KernelError::Memory(MemoryError::OutOfMemory)));
    }

    #[test]
    fn kernel_error_from_process() {
        let e = KernelError::from(ProcessError::InvalidPid);
        assert!(matches!(e, KernelError::Process(ProcessError::InvalidPid)));
    }

    #[test]
    fn kernel_error_from_fs() {
        let e = KernelError::from(FsError::NotFound);
        assert!(matches!(e, KernelError::FileSystem(FsError::NotFound)));
    }

    #[test]
    fn kernel_error_from_ipc() {
        let e = KernelError::from(IpcError::QueueFull);
        assert!(matches!(e, KernelError::Ipc(IpcError::QueueFull)));
    }

    #[test]
    fn kernel_error_from_driver() {
        let e = KernelError::from(DriverError::IoError);
        assert!(matches!(e, KernelError::Driver(DriverError::IoError)));
    }

    #[test]
    fn kernel_error_from_trace() {
        let e = KernelError::from(TraceError::BufferFull);
        assert!(matches!(e, KernelError::Trace(TraceError::BufferFull)));
    }

    #[test]
    fn memory_error_display() {
        assert_eq!(
            format!("{}", MemoryError::OutOfMemory),
            "out of physical memory"
        );
        assert_eq!(
            format!("{}", MemoryError::InvalidAddress),
            "invalid address"
        );
        assert_eq!(
            format!("{}", MemoryError::AlreadyMapped),
            "page already mapped"
        );
        assert_eq!(format!("{}", MemoryError::NotMapped), "page not mapped");
        assert_eq!(
            format!("{}", MemoryError::AlignmentError),
            "address alignment error"
        );
    }

    #[test]
    fn process_error_display() {
        assert_eq!(
            format!("{}", ProcessError::MaxProcessesReached),
            "max processes reached"
        );
        assert_eq!(format!("{}", ProcessError::InvalidPid), "invalid pid");
        assert_eq!(
            format!("{}", ProcessError::ProcessNotFound),
            "process not found"
        );
        assert_eq!(
            format!("{}", ProcessError::InvalidStateTransition),
            "invalid state transition"
        );
        assert_eq!(
            format!("{}", ProcessError::StackAllocationFailed),
            "stack allocation failed"
        );
    }

    #[test]
    fn fs_error_display() {
        assert_eq!(format!("{}", FsError::NotFound), "not found");
        assert_eq!(format!("{}", FsError::AlreadyExists), "already exists");
        assert_eq!(
            format!("{}", FsError::PermissionDenied),
            "permission denied"
        );
        assert_eq!(format!("{}", FsError::NoSpace), "no space");
        assert_eq!(
            format!("{}", FsError::TooManyOpenFiles),
            "too many open files"
        );
        assert_eq!(
            format!("{}", FsError::InvalidDescriptor),
            "invalid descriptor"
        );
    }

    #[test]
    fn ipc_error_display() {
        assert_eq!(format!("{}", IpcError::QueueFull), "queue full");
        assert_eq!(format!("{}", IpcError::QueueEmpty), "queue empty");
        assert_eq!(format!("{}", IpcError::Timeout), "timeout");
    }

    #[test]
    fn driver_error_display() {
        assert_eq!(
            format!("{}", DriverError::NotInitialized),
            "not initialized"
        );
        assert_eq!(
            format!("{}", DriverError::DeviceNotFound),
            "device not found"
        );
        assert_eq!(format!("{}", DriverError::IoError), "I/O error");
        assert_eq!(
            format!("{}", DriverError::Unsupported),
            "unsupported operation"
        );
    }

    #[test]
    fn trace_error_display() {
        assert_eq!(format!("{}", TraceError::BufferFull), "trace buffer full");
        assert_eq!(
            format!("{}", TraceError::MaxDepthExceeded),
            "max trace depth exceeded"
        );
        assert_eq!(
            format!("{}", TraceError::NotInitialized),
            "tracer not initialized"
        );
    }

    #[test]
    fn all_from_conversions_round_trip() {
        assert!(matches!(
            KernelError::from(MemoryError::AlignmentError),
            KernelError::Memory(MemoryError::AlignmentError)
        ));
        assert!(matches!(
            KernelError::from(ProcessError::StackAllocationFailed),
            KernelError::Process(ProcessError::StackAllocationFailed)
        ));
        assert!(matches!(
            KernelError::from(FsError::InvalidPath),
            KernelError::FileSystem(FsError::InvalidPath)
        ));
        assert!(matches!(
            KernelError::from(IpcError::Timeout),
            KernelError::Ipc(IpcError::Timeout)
        ));
        assert!(matches!(
            KernelError::from(DriverError::Unsupported),
            KernelError::Driver(DriverError::Unsupported)
        ));
        assert!(matches!(
            KernelError::from(TraceError::NotInitialized),
            KernelError::Trace(TraceError::NotInitialized)
        ));
    }

    #[test]
    fn display_all_fs_error_variants() {
        assert_eq!(format!("{}", FsError::NotADirectory), "not a directory");
        assert_eq!(format!("{}", FsError::NotAFile), "not a file");
        assert_eq!(format!("{}", FsError::InvalidPath), "invalid path");
    }

    #[test]
    fn display_all_ipc_error_variants() {
        assert_eq!(format!("{}", IpcError::QueueNotFound), "queue not found");
        assert_eq!(
            format!("{}", IpcError::InvalidMessage),
            "invalid message"
        );
    }

    #[test]
    fn memory_error_equality() {
        assert_eq!(MemoryError::OutOfMemory, MemoryError::OutOfMemory);
        assert_ne!(MemoryError::OutOfMemory, MemoryError::InvalidAddress);
        assert_ne!(MemoryError::AlreadyMapped, MemoryError::NotMapped);
    }

    #[test]
    fn process_error_equality() {
        assert_eq!(ProcessError::InvalidPid, ProcessError::InvalidPid);
        assert_ne!(
            ProcessError::MaxProcessesReached,
            ProcessError::InvalidPid
        );
    }
}
