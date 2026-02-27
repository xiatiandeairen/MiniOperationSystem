# minios-scheduler — MLFQ Task Scheduler

A `#![no_std]` Multi-Level Feedback Queue (MLFQ) scheduler.

## Features
- 4 priority levels with configurable time slices
- Automatic priority demotion on quantum exhaustion
- Periodic priority boost to prevent starvation
- O(n) boost, O(1) tick decisions

## Standalone Usage
```toml
[dependencies]
minios-scheduler = { version = "0.1", default-features = false }
```

```rust
use minios_scheduler::MlfqScheduler;
use minios_common::id::Pid;
use minios_common::types::Priority;

let mut sched = MlfqScheduler::new();
sched.add_task(Pid(1), Priority::HIGH);
sched.set_running(Pid(1), 0);
let decision = sched.tick(); // Returns Continue, Switch, or Idle
```
