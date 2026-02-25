//! Process management contract.

use crate::error::ProcessError;
use crate::id::Pid;
use crate::types::ProcessInfo;

/// Lifecycle management for kernel tasks and processes.
pub trait ProcessManager: Send + Sync {
    fn create_process(&self, name: &str, entry: fn()) -> Result<Pid, ProcessError>;
    fn exit_process(&self, pid: Pid, code: i32) -> Result<(), ProcessError>;
    fn kill_process(&self, pid: Pid) -> Result<(), ProcessError>;
    fn current_pid(&self) -> Pid;
    fn process_info(&self, pid: Pid) -> Option<ProcessInfo>;
}
