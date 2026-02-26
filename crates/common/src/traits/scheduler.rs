//! Task scheduling contract.

use crate::id::Pid;
use crate::types::{BlockReason, Priority, ScheduleDecision, SchedulerStats};

/// Decides which task runs next.
pub trait Scheduler: Send + Sync {
    /// Adds a task to the scheduler at the given priority.
    fn add_task(&self, pid: Pid, priority: Priority);
    /// Removes a task from all queues.
    fn remove_task(&self, pid: Pid);
    /// Processes a timer tick and returns a scheduling decision.
    fn tick(&self) -> ScheduleDecision;
    /// Returns the next task to run without consuming it.
    fn next_task(&self) -> Option<Pid>;
    /// Voluntarily yields the current task's time slice.
    fn yield_current(&self);
    /// Blocks the current task for the given reason.
    fn block_current(&self, reason: BlockReason);
    /// Unblocks a previously blocked task.
    fn unblock(&self, pid: Pid);
    /// Changes the priority of a task.
    fn set_priority(&self, pid: Pid, priority: Priority);
    /// Returns runtime scheduler statistics.
    fn stats(&self) -> SchedulerStats;
}
