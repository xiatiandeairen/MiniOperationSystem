//! Fixed-Priority scheduler — highest priority task always runs.
//!
//! Unlike MLFQ, priorities never change. This can cause starvation.

extern crate alloc;
use alloc::collections::BTreeMap;
use minios_common::id::Pid;
use minios_common::types::{Priority, ScheduleDecision, SchedulerStats};

pub struct PriorityScheduler {
    tasks: BTreeMap<Pid, Priority>,
    current: Option<Pid>,
    total_ticks: u64,
    total_switches: u64,
}

impl Default for PriorityScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl PriorityScheduler {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            current: None,
            total_ticks: 0,
            total_switches: 0,
        }
    }

    pub fn add_task(&mut self, pid: Pid, priority: Priority) {
        self.tasks.insert(pid, priority);
    }

    pub fn tick(&mut self) -> ScheduleDecision {
        self.total_ticks += 1;
        let best = self
            .tasks
            .iter()
            .min_by_key(|(_, p)| p.0)
            .map(|(pid, _)| *pid);
        match (self.current, best) {
            (Some(curr), Some(best)) if curr == best => ScheduleDecision::Continue,
            (_, Some(best)) => {
                self.current = Some(best);
                self.total_switches += 1;
                ScheduleDecision::Switch(best)
            }
            _ => ScheduleDecision::Idle,
        }
    }

    pub fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            total_switches: self.total_switches,
            total_ticks: self.total_ticks,
            queue_lengths: [self.tasks.len(), 0, 0, 0],
            idle_ticks: 0,
        }
    }
}
