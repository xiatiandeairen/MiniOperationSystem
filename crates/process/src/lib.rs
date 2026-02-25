//! Process management subsystem for MiniOS.
//!
//! Provides process control blocks, CPU context for context switching,
//! and a process table managing up to 64 kernel tasks.

#![no_std]

extern crate alloc;

pub mod context;
pub mod manager;
pub mod pcb;
