//! Simple Round-Robin scheduler for comparison with MLFQ.

extern crate alloc;
use alloc::collections::VecDeque;
use minios_common::id::Pid;
use minios_common::types::{ScheduleDecision, SchedulerStats};

/// Round-Robin scheduler with a single queue and fixed time slice.
pub struct RoundRobinScheduler {
    queue: VecDeque<Pid>,
    current: Option<(Pid, u64)>,
    time_slice: u64,
    total_ticks: u64,
    total_switches: u64,
}

impl RoundRobinScheduler {
    pub const fn new(time_slice: u64) -> Self {
        Self {
            queue: VecDeque::new(),
            current: None,
            time_slice,
            total_ticks: 0,
            total_switches: 0,
        }
    }

    pub fn add_task(&mut self, pid: Pid) {
        if !self.queue.contains(&pid) {
            self.queue.push_back(pid);
        }
    }

    pub fn remove_task(&mut self, pid: Pid) {
        self.queue.retain(|&p| p != pid);
        if matches!(self.current, Some((p, _)) if p == pid) {
            self.current = None;
        }
    }

    pub fn tick(&mut self) -> ScheduleDecision {
        self.total_ticks += 1;
        if let Some((pid, ref mut remaining)) = self.current {
            *remaining = remaining.saturating_sub(1);
            if *remaining == 0 {
                self.queue.push_back(pid);
                self.current = None;
                return self.pick_next();
            }
            ScheduleDecision::Continue
        } else {
            self.pick_next()
        }
    }

    fn pick_next(&mut self) -> ScheduleDecision {
        if let Some(pid) = self.queue.pop_front() {
            self.current = Some((pid, self.time_slice));
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
