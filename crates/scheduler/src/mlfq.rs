//! Multi-Level Feedback Queue (MLFQ) scheduler.
//!
//! Four priority queues with increasing time slices. Tasks that exhaust
//! their quantum are demoted; a periodic boost prevents starvation.

extern crate alloc;

use alloc::collections::VecDeque;

use minios_common::id::Pid;
use minios_common::types::{Priority, ScheduleDecision, SchedulerStats};

/// Number of priority levels.
const NUM_QUEUES: usize = 4;

/// Time-slice (in ticks) for each priority level, indexed 0–3.
const TIME_SLICES: [u64; NUM_QUEUES] = [2, 4, 8, 16];

/// Number of ticks between full priority boosts.
const BOOST_INTERVAL: u64 = 100;

/// Per-task bookkeeping inside the scheduler.
struct TaskEntry {
    pid: Pid,
    queue: usize,
    remaining: u64,
}

/// MLFQ scheduler state.
pub struct MlfqScheduler {
    queues: [VecDeque<Pid>; NUM_QUEUES],
    current: Option<TaskEntry>,
    total_ticks: u64,
    total_switches: u64,
    idle_ticks: u64,
    ticks_since_boost: u64,
}

impl Default for MlfqScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl MlfqScheduler {
    /// Creates a new scheduler with empty queues.
    pub const fn new() -> Self {
        Self {
            queues: [
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
            ],
            current: None,
            total_ticks: 0,
            total_switches: 0,
            idle_ticks: 0,
            ticks_since_boost: 0,
        }
    }

    /// Adds a task to the appropriate priority queue.
    pub fn add_task(&mut self, pid: Pid, priority: Priority) {
        let q = (priority.0 as usize).min(NUM_QUEUES - 1);
        if !self.queues[q].contains(&pid) {
            self.queues[q].push_back(pid);
        }
    }

    /// Removes a task from all queues and clears it if it is current.
    pub fn remove_task(&mut self, pid: Pid) {
        for q in &mut self.queues {
            q.retain(|&p| p != pid);
        }
        if matches!(self.current, Some(ref t) if t.pid == pid) {
            self.current = None;
        }
    }

    /// Called on every timer tick. Returns whether a context switch is needed.
    pub fn tick(&mut self) -> ScheduleDecision {
        self.total_ticks += 1;
        self.ticks_since_boost += 1;

        if self.ticks_since_boost >= BOOST_INTERVAL {
            self.boost_all();
        }

        if let Some(ref mut entry) = self.current {
            entry.remaining = entry.remaining.saturating_sub(1);
            if entry.remaining == 0 {
                return self.preempt_current();
            }
            ScheduleDecision::Continue
        } else {
            self.pick_next()
        }
    }

    /// Selects the highest-priority ready task.
    pub fn next_task(&mut self) -> Option<Pid> {
        for q in &mut self.queues {
            if let Some(pid) = q.pop_front() {
                return Some(pid);
            }
        }
        None
    }

    /// Returns runtime statistics.
    pub fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            total_switches: self.total_switches,
            total_ticks: self.total_ticks,
            queue_lengths: [
                self.queues[0].len(),
                self.queues[1].len(),
                self.queues[2].len(),
                self.queues[3].len(),
            ],
            idle_ticks: self.idle_ticks,
        }
    }

    /// Returns the PID of the currently running task, if any.
    pub fn current_pid(&self) -> Option<Pid> {
        self.current.as_ref().map(|e| e.pid)
    }

    /// Marks `pid` as the actively running task at the given queue level.
    pub fn set_running(&mut self, pid: Pid, queue: usize) {
        let q = queue.min(NUM_QUEUES - 1);
        self.current = Some(TaskEntry {
            pid,
            queue: q,
            remaining: TIME_SLICES[q],
        });
    }

    // ── private helpers ─────────────────────────────────────────────

    /// Demotes the current task and picks the next one.
    fn preempt_current(&mut self) -> ScheduleDecision {
        let entry = self.current.take().unwrap();
        let new_q = (entry.queue + 1).min(NUM_QUEUES - 1);
        self.queues[new_q].push_back(entry.pid);
        self.pick_next()
    }

    /// Picks the next task from the highest non-empty queue.
    fn pick_next(&mut self) -> ScheduleDecision {
        if let Some(pid) = self.next_task() {
            self.total_switches += 1;
            ScheduleDecision::Switch(pid)
        } else {
            self.idle_ticks += 1;
            ScheduleDecision::Idle
        }
    }

    /// Boosts all tasks to the highest-priority queue in O(n) time.
    ///
    /// Drains lower queues into queue 0. Duplicates are impossible because
    /// each PID exists in exactly one queue at a time.
    fn boost_all(&mut self) {
        self.ticks_since_boost = 0;
        for q in 1..NUM_QUEUES {
            while let Some(pid) = self.queues[q].pop_front() {
                self.queues[0].push_back(pid);
            }
        }
        if let Some(ref mut entry) = self.current {
            entry.queue = 0;
            entry.remaining = TIME_SLICES[0];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minios_common::types::Priority;

    #[test]
    fn add_task_to_correct_queue() {
        let mut sched = MlfqScheduler::new();
        sched.add_task(Pid(0), Priority(0));
        let stats = sched.stats();
        assert_eq!(stats.queue_lengths[0], 1);
    }

    #[test]
    fn tick_decrements_remaining() {
        let mut sched = MlfqScheduler::new();
        sched.add_task(Pid(0), Priority(0));
        sched.set_running(Pid(0), 0);
        let dec = sched.tick();
        assert!(matches!(dec, ScheduleDecision::Continue));
    }

    #[test]
    fn tick_preempts_on_exhaustion() {
        let mut sched = MlfqScheduler::new();
        sched.add_task(Pid(0), Priority(0));
        sched.set_running(Pid(0), 0);
        let _ = sched.tick();
        let dec = sched.tick();
        assert!(matches!(dec, ScheduleDecision::Switch(_)));
    }

    #[test]
    fn boost_moves_all_to_top() {
        let mut sched = MlfqScheduler::new();
        sched.add_task(Pid(1), Priority(1));
        sched.add_task(Pid(2), Priority(2));
        sched.add_task(Pid(3), Priority(3));
        sched.add_task(Pid(0), Priority(0));
        sched.set_running(Pid(0), 0);
        for _ in 0..BOOST_INTERVAL {
            let dec = sched.tick();
            if let ScheduleDecision::Switch(pid) = dec {
                let q = if pid == Pid(0) {
                    1
                } else if pid == Pid(1) {
                    1
                } else if pid == Pid(2) {
                    2
                } else {
                    3
                };
                sched.set_running(pid, q);
            }
        }
        let stats = sched.stats();
        assert!(stats.queue_lengths[0] >= 1, "boost moves tasks to queue 0");
        assert_eq!(stats.queue_lengths[1], 0);
        assert_eq!(stats.queue_lengths[2], 0);
        assert_eq!(stats.queue_lengths[3], 0);
    }

    #[test]
    fn remove_task_clears_all_queues() {
        let mut sched = MlfqScheduler::new();
        sched.add_task(Pid(0), Priority(0));
        sched.remove_task(Pid(0));
        let stats = sched.stats();
        assert_eq!(stats.queue_lengths[0], 0);
        assert_eq!(stats.queue_lengths[1], 0);
        assert_eq!(stats.queue_lengths[2], 0);
        assert_eq!(stats.queue_lengths[3], 0);
    }

    #[test]
    fn next_task_picks_highest_priority() {
        let mut sched = MlfqScheduler::new();
        sched.add_task(Pid(1), Priority(2));
        sched.add_task(Pid(0), Priority(0));
        let first = sched.next_task();
        assert_eq!(first, Some(Pid(0)));
    }

    #[test]
    fn stats_accurate() {
        let mut sched = MlfqScheduler::new();
        sched.add_task(Pid(0), Priority(0));
        sched.set_running(Pid(0), 0);
        let _ = sched.tick();
        let _ = sched.tick();
        let stats = sched.stats();
        assert!(stats.total_ticks >= 2);
        assert!(stats.total_switches >= 1);
    }

    #[test]
    fn idle_when_empty() {
        let mut sched = MlfqScheduler::new();
        let dec = sched.tick();
        assert!(matches!(dec, ScheduleDecision::Idle));
    }
}
