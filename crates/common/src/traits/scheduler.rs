//! Task scheduling contract.

use crate::id::Pid;
use crate::types::{BlockReason, Priority, ScheduleDecision, SchedulerStats};

/// Decides which task runs next.
pub trait Scheduler: Send + Sync {
    fn add_task(&self, pid: Pid, priority: Priority);
    fn remove_task(&self, pid: Pid);
    fn tick(&self) -> ScheduleDecision;
    fn next_task(&self) -> Option<Pid>;
    fn yield_current(&self);
    fn block_current(&self, reason: BlockReason);
    fn unblock(&self, pid: Pid);
    fn set_priority(&self, pid: Pid, priority: Priority);
    fn stats(&self) -> SchedulerStats;
}
