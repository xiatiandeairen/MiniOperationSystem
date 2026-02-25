//! Task scheduling subsystem for MiniOS.
//!
//! Provides a Multi-Level Feedback Queue (MLFQ) scheduler with four
//! priority levels and automatic priority demotion / periodic boost.

#![no_std]

extern crate alloc;

pub mod mlfq;

pub use mlfq::MlfqScheduler;

use spin::Mutex;

/// Global scheduler instance.
pub static SCHEDULER: Mutex<MlfqScheduler> = Mutex::new(MlfqScheduler::new());
