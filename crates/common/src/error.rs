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
