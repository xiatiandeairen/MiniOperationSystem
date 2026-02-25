//! Trait contracts forming the decoupled boundaries between kernel subsystems.
//!
//! Every subsystem depends only on these traits, never on concrete implementations.

pub mod device;
pub mod fs;
pub mod hal;
pub mod ipc;
pub mod memory;
pub mod process;
pub mod scheduler;
pub mod trace;
