//! Process management contract.

use crate::error::ProcessError;
use crate::id::Pid;
use crate::types::ProcessInfo;

/// Lifecycle management for kernel tasks and processes.
pub trait ProcessManager: Send + Sync {
    /// Creates a new process with the given name and entry point.
    fn create_process(&self, name: &str, entry: fn()) -> Result<Pid, ProcessError>;
    /// Terminates a process with an exit code.
    fn exit_process(&self, pid: Pid, code: i32) -> Result<(), ProcessError>;
    /// Forcefully kills a process.
    fn kill_process(&self, pid: Pid) -> Result<(), ProcessError>;
    /// Returns the PID of the currently running process.
    fn current_pid(&self) -> Pid;
    /// Returns summary information about a process.
    fn process_info(&self, pid: Pid) -> Option<ProcessInfo>;
}
