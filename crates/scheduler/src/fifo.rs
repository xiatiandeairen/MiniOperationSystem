//! First-In-First-Out (FIFO) scheduler — simplest possible scheduler.
//!
//! Each task runs to completion (or until it yields). No preemption.

extern crate alloc;
use alloc::collections::VecDeque;
use minios_common::id::Pid;
use minios_common::types::{ScheduleDecision, SchedulerStats};

/// FIFO scheduler — tasks run in arrival order until they yield.
pub struct FifoScheduler {
    queue: VecDeque<Pid>,
    current: Option<Pid>,
    total_ticks: u64,
    total_switches: u64,
}

impl FifoScheduler {
    pub const fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current: None,
            total_ticks: 0,
            total_switches: 0,
        }
    }

    pub fn add_task(&mut self, pid: Pid) {
        if !self.queue.contains(&pid) {
            self.queue.push_back(pid);
        }
    }

    pub fn tick(&mut self) -> ScheduleDecision {
        self.total_ticks += 1;
        if self.current.is_some() {
            ScheduleDecision::Continue
        } else {
            self.pick_next()
        }
    }

    pub fn yield_current(&mut self) -> ScheduleDecision {
        if let Some(pid) = self.current.take() {
            self.queue.push_back(pid);
        }
        self.pick_next()
    }

    fn pick_next(&mut self) -> ScheduleDecision {
        if let Some(pid) = self.queue.pop_front() {
            self.current = Some(pid);
            self.total_switches += 1;
            ScheduleDecision::Switch(pid)
        } else {
            ScheduleDecision::Idle
        }
    }

    pub fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            total_switches: self.total_switches,
            total_ticks: self.total_ticks,
            queue_lengths: [self.queue.len(), 0, 0, 0],
            idle_ticks: 0,
        }
    }
}
