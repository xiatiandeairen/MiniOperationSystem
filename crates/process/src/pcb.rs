//! Process Control Block (PCB) — per-process kernel state.

use crate::context::CpuContext;
use minios_common::id::Pid;
use minios_common::types::{Priority, ProcessState};

/// Maximum length of a process name in bytes.
pub const MAX_NAME_LEN: usize = 32;

/// Kernel-mode process descriptor.
///
/// All tasks currently run in Ring 0 and share the same address space.
pub struct Process {
    /// Unique process identifier.
    pub pid: Pid,
    /// Human-readable name stored inline (no heap allocation required).
    pub name: [u8; MAX_NAME_LEN],
    /// Number of valid bytes in [`name`](Self::name).
    pub name_len: usize,
    /// Current lifecycle state.
    pub state: ProcessState,
    /// Scheduling priority.
    pub priority: Priority,
    /// Saved CPU register context for context switching.
    pub context: CpuContext,
    /// Top of the allocated kernel stack (highest address).
    pub kernel_stack: u64,
    /// Accumulated CPU time in timer ticks.
    pub cpu_time: u64,
}

impl Process {
    /// Creates a new process in the [`Created`](ProcessState::Created) state.
    pub fn new(pid: Pid, name: &str, priority: Priority) -> Self {
        let mut name_buf = [0u8; MAX_NAME_LEN];
        let len = name.len().min(MAX_NAME_LEN);
        name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);

        Self {
            pid,
            name: name_buf,
            name_len: len,
            state: ProcessState::Created,
            priority,
            context: CpuContext::empty(),
            kernel_stack: 0,
            cpu_time: 0,
        }
    }

    /// Returns the process name as a `&str`.
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("???")
    }
}
