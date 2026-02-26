//! Process table and lifecycle management.
//!
//! Maintains a fixed-size table of up to [`MAX_PROCESSES`] process control
//! blocks and provides operations for creating, exiting, and querying tasks.

extern crate alloc;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

use spin::Mutex;

use minios_common::error::ProcessError;
use minios_common::id::Pid;
use minios_common::types::{Priority, ProcessInfo, ProcessState};

use crate::context::CpuContext;
use crate::pcb::Process;

/// Maximum number of concurrent processes.
pub const MAX_PROCESSES: usize = 64;

/// Number of 4 KiB pages per kernel task stack.
const STACK_PAGES: usize = 4;

/// Size of one kernel task stack in bytes (16 KiB).
const STACK_SIZE: usize = STACK_PAGES * 4096;

/// Alignment requirement for stack allocations.
const STACK_ALIGN: usize = 16;

/// Monotonic PID counter.
static NEXT_PID: AtomicU32 = AtomicU32::new(0);

/// Global process table.
pub static PROCESS_TABLE: Mutex<ProcessTable> = Mutex::new(ProcessTable::new());

/// Fixed-size process table.
pub struct ProcessTable {
    slots: [Option<Process>; MAX_PROCESSES],
    current: Option<usize>,
}

impl ProcessTable {
    /// Creates an empty process table.
    const fn new() -> Self {
        const NONE: Option<Process> = None;
        Self {
            slots: [NONE; MAX_PROCESSES],
            current: None,
        }
    }
}

/// Allocates a 16 KiB kernel stack via the global allocator.
///
/// Returns the *top* of the stack (highest address, 16-byte aligned)
/// or `None` if allocation fails.
fn alloc_kernel_stack() -> Option<u64> {
    let layout = alloc::alloc::Layout::from_size_align(STACK_SIZE, STACK_ALIGN).ok()?;
    // SAFETY: Layout is non-zero and correctly aligned.
    let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
    if ptr.is_null() {
        return None;
    }
    Some(ptr as u64 + STACK_SIZE as u64)
}

/// Creates a new kernel task with the given name and entry function.
///
/// The task is placed in the [`Ready`](ProcessState::Ready) state.
pub fn create_kernel_task(
    name: &str,
    entry_fn: fn(),
    priority: Priority,
) -> Result<Pid, ProcessError> {
    let pid = Pid(NEXT_PID.fetch_add(1, Ordering::Relaxed));
    let stack_top = alloc_kernel_stack().ok_or(ProcessError::StackAllocationFailed)?;

    let mut proc = Process::new(pid, name, priority);
    proc.context = CpuContext::new(entry_fn as usize as u64, stack_top);
    proc.kernel_stack = stack_top;
    proc.state = ProcessState::Ready;

    let mut table = PROCESS_TABLE.lock();
    let slot = table
        .slots
        .iter()
        .position(|s| s.is_none())
        .ok_or(ProcessError::MaxProcessesReached)?;
    table.slots[slot] = Some(proc);

    minios_hal::klog!(
        Info,
        "process",
        "created task '{}' with PID {}",
        name,
        pid.0
    );

    Ok(pid)
}

/// Marks the current process as [`Terminated`](ProcessState::Terminated).
pub fn exit_current(_code: i32) {
    let mut table = PROCESS_TABLE.lock();
    if let Some(idx) = table.current {
        if let Some(ref mut p) = table.slots[idx] {
            p.state = ProcessState::Terminated;
        }
    }
    minios_hal::klog!(Info, "process", "process exited");
}

/// Returns the PID of the currently running process.
pub fn current_pid() -> Pid {
    let table = PROCESS_TABLE.lock();
    table
        .current
        .and_then(|i| table.slots[i].as_ref().map(|p| p.pid))
        .unwrap_or(Pid(0))
}

/// Sets the index of the currently running process.
pub fn set_current(pid: Pid) {
    let mut table = PROCESS_TABLE.lock();
    table.current = table
        .slots
        .iter()
        .position(|s| matches!(s, Some(p) if p.pid == pid));
}

/// Returns summary info for all non-empty process slots.
pub fn list_processes() -> Vec<ProcessInfo> {
    let table = PROCESS_TABLE.lock();
    table
        .slots
        .iter()
        .filter_map(|slot| {
            slot.as_ref().map(|p| ProcessInfo {
                pid: p.pid,
                state: p.state,
                priority: p.priority,
                cpu_time_ticks: p.cpu_time,
            })
        })
        .collect()
}

/// Sets a process state by PID. Returns an error if the PID is not found.
pub fn set_state(pid: Pid, state: ProcessState) -> Result<(), ProcessError> {
    let mut table = PROCESS_TABLE.lock();
    let proc = table
        .slots
        .iter_mut()
        .find_map(|s| s.as_mut().filter(|p| p.pid == pid))
        .ok_or(ProcessError::ProcessNotFound)?;
    proc.state = state;
    Ok(())
}

/// Returns a mutable pointer to the `CpuContext` of the process with `pid`.
///
/// # Safety
///
/// The caller must hold the process table lock or guarantee exclusive access.
pub fn context_ptr(pid: Pid) -> Option<*mut CpuContext> {
    let mut table = PROCESS_TABLE.lock();
    table
        .slots
        .iter_mut()
        .find_map(|s| s.as_mut().filter(|p| p.pid == pid))
        .map(|p| &mut p.context as *mut CpuContext)
}

/// Increments the CPU time counter for the given process by one tick.
pub fn tick_cpu_time(pid: Pid) {
    let mut table = PROCESS_TABLE.lock();
    if let Some(p) = table
        .slots
        .iter_mut()
        .find_map(|s| s.as_mut().filter(|p| p.pid == pid))
    {
        p.cpu_time += 1;
    }
}
